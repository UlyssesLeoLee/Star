// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider/sse.rs` — W2 (ULYS-178) SSE → [`AgentStreamEvent`] adapters.
//!
//! Per SRS-PI-BORROW-001 PI-1 (FR-1 ~ FR-6) + PI-3 (FR-16 ~ FR-18):
//! - [`SseDecoder`] — incremental `text/event-stream` framing (byte-level
//!   buffering, so a UTF-8 code point split across TCP chunks is safe).
//! - [`AnthropicTranslator`] — Messages API stream events
//!   (`message_start` / `content_block_*` / `message_delta` / `message_stop` / `error`).
//! - [`OpenAiTranslator`] — Chat Completions `chat.completion.chunk` + `[DONE]`.
//! - [`drive`] — HTTP body stream + translator → event stream.
//! - [`no_throw`] — outer guard: catches panics, guarantees exactly one
//!   terminal event, drops anything after it (PI-3 no-throw contract).
//!
//! 守门合规 (per 守门 #5 v2): error events carry status + provider error
//! text only, never request headers / API keys.

use std::collections::{HashMap, VecDeque};
use std::panic::AssertUnwindSafe;

use futures_util::stream::{self, BoxStream, Stream, StreamExt};
use serde_json::Value;
use tracing::warn;
use uuid::Uuid;

use crate::events::{AgentStreamEvent, StopReason, StreamError, Usage};

// =====================================================================
// Error helpers
// =====================================================================

/// `Error` + `Done { Error }` pair — the canonical way a v2 stream fails.
pub(crate) fn fail(error: StreamError, usage: Usage) -> Vec<AgentStreamEvent> {
    let recoverable = error.recoverable;
    vec![
        AgentStreamEvent::Error { error, recoverable },
        AgentStreamEvent::Done {
            stop_reason: StopReason::Error,
            usage,
        },
    ]
}

/// Map a non-2xx HTTP status into a [`StreamError`].
///
/// 408 / 429 / 5xx are transient (retry); every other 4xx is a caller or
/// configuration problem and is not recoverable.
pub(crate) fn http_status_error(provider: &str, status: u16) -> StreamError {
    let (kind, recoverable) = match status {
        408 | 429 => ("http_4xx", true),
        400..=499 => ("http_4xx", false),
        _ => ("http_5xx", true),
    };
    StreamError::new(kind, format!("{provider}: HTTP {status}"), recoverable).with_status(status)
}

/// Map a transport-level `reqwest` failure (connect / send / body read).
pub(crate) fn transport_error(provider: &str, e: &reqwest::Error) -> StreamError {
    if e.is_timeout() {
        StreamError::new("timeout", format!("{provider}: request timed out"), true)
    } else {
        // `without_url` keeps query strings out of the event (守门 #5 v2).
        let msg = format!("{provider}: transport failure: {}", strip_url(e));
        StreamError::new("network", msg, true)
    }
}

fn strip_url(e: &reqwest::Error) -> String {
    let mut s = e.to_string();
    if let Some(url) = e.url() {
        s = s.replace(url.as_str(), "<url>");
    }
    s
}

fn protocol_error(provider: &str, detail: impl std::fmt::Display) -> StreamError {
    StreamError::new("protocol_violation", format!("{provider}: {detail}"), false)
}

/// A stream that fails immediately (`Error` + `Done { Error }`).
pub(crate) fn failed_stream(error: StreamError) -> BoxStream<'static, AgentStreamEvent> {
    stream::iter(fail(error, Usage::default())).boxed()
}

/// Send `request` and translate its SSE body. Transport failures and
/// non-2xx statuses are encoded as events (PI-3 FR-17).
pub(crate) async fn open<T: SseTranslator>(
    request: reqwest::RequestBuilder,
    translator: T,
) -> BoxStream<'static, AgentStreamEvent> {
    let provider = translator.provider();
    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => return failed_stream(transport_error(provider, &e)),
    };
    let status = response.status();
    if !status.is_success() {
        // 守门 #5 v2: status only, the body is not echoed.
        warn!(provider, status = status.as_u16(), "LLM stream rejected");
        return failed_stream(http_status_error(provider, status.as_u16()));
    }
    drive(response.bytes_stream().boxed(), translator)
}

// =====================================================================
// SSE framing
// =====================================================================

/// One `text/event-stream` frame (`event:` + joined `data:` lines).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SseFrame {
    /// `event:` field, if present.
    pub(crate) event: Option<String>,
    /// `data:` lines joined with `\n`.
    pub(crate) data: String,
}

/// Incremental SSE decoder. Buffers raw bytes until a blank line closes a
/// frame, so multi-byte UTF-8 split across network chunks decodes intact.
#[derive(Debug, Default)]
pub(crate) struct SseDecoder {
    buf: Vec<u8>,
}

impl SseDecoder {
    /// Feed a network chunk; returns every frame completed by it.
    ///
    /// `\r` is dropped on ingest: SSE allows CRLF line endings and JSON
    /// payloads never contain a raw carriage return.
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Result<Vec<SseFrame>, std::str::Utf8Error> {
        self.buf
            .extend(chunk.iter().copied().filter(|b| *b != b'\r'));
        let mut frames = Vec::new();
        while let Some(pos) = self.buf.windows(2).position(|w| w == b"\n\n") {
            let raw: Vec<u8> = self.buf.drain(..pos + 2).collect();
            if let Some(frame) = Self::parse_frame(std::str::from_utf8(&raw[..pos])?) {
                frames.push(frame);
            }
        }
        Ok(frames)
    }

    /// Flush a trailing frame that was not closed by a blank line.
    pub(crate) fn finish(&mut self) -> Result<Option<SseFrame>, std::str::Utf8Error> {
        let raw = std::mem::take(&mut self.buf);
        Ok(Self::parse_frame(std::str::from_utf8(&raw)?))
    }

    fn parse_frame(block: &str) -> Option<SseFrame> {
        let mut event = None;
        let mut data: Option<String> = None;
        for line in block.split('\n') {
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            let (field, value) = match line.split_once(':') {
                Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
                None => (line, ""),
            };
            match field {
                "event" => event = Some(value.to_string()),
                "data" => match &mut data {
                    Some(d) => {
                        d.push('\n');
                        d.push_str(value);
                    }
                    None => data = Some(value.to_string()),
                },
                _ => {}
            }
        }
        data.map(|data| SseFrame { event, data })
    }
}

// =====================================================================
// Translator contract
// =====================================================================

/// Provider-specific SSE frame → [`AgentStreamEvent`] mapping.
pub(crate) trait SseTranslator: Send + 'static {
    /// Provider label used in error messages.
    fn provider(&self) -> &'static str;
    /// Translate one frame. May return several events (or none).
    fn on_frame(&mut self, frame: SseFrame) -> Vec<AgentStreamEvent>;
    /// Body ended. Must return a terminal event unless one was already
    /// emitted from `on_frame`.
    fn on_eof(&mut self) -> Vec<AgentStreamEvent>;
    /// Usage accumulated so far (attached to failure `Done` events).
    fn usage(&self) -> Usage;
}

fn parse_json(provider: &str, data: &str) -> Result<Value, StreamError> {
    serde_json::from_str(data)
        .map_err(|e| protocol_error(provider, format!("invalid JSON frame: {e}")))
}

fn u64_at(v: &Value, key: &str) -> Option<u64> {
    v.get(key).and_then(Value::as_u64)
}

fn str_at<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

/// `MetaUpdate` carrying the provider-issued response id (e.g. `msg_…`,
/// `chatcmpl-…`); `StreamStart.id` is the Star request uuid.
fn provider_id_meta(id: &str) -> AgentStreamEvent {
    AgentStreamEvent::MetaUpdate {
        key: "provider_response_id".to_string(),
        value: id.to_string(),
        payload: None,
    }
}

// =====================================================================
// Anthropic Messages API streaming
// =====================================================================

#[derive(Debug, Clone)]
enum AnthropicBlock {
    Text,
    Thinking,
    ToolUse(String),
    Other,
}

/// Anthropic `/v1/messages` (`stream: true`) translator.
/// Ref: <https://docs.anthropic.com/en/api/messages-streaming>.
#[derive(Debug)]
pub(crate) struct AnthropicTranslator {
    request_id: Uuid,
    model: String,
    started: bool,
    blocks: HashMap<u64, AnthropicBlock>,
    stop_reason: Option<StopReason>,
    usage: Usage,
}

impl AnthropicTranslator {
    pub(crate) fn new(request_id: Uuid, model: String) -> Self {
        Self {
            request_id,
            model,
            started: false,
            blocks: HashMap::new(),
            stop_reason: None,
            usage: Usage::default(),
        }
    }

    fn apply_usage(&mut self, u: &Value) {
        if let Some(n) = u64_at(u, "input_tokens") {
            self.usage.input_tokens = n;
        }
        if let Some(n) = u64_at(u, "output_tokens") {
            self.usage.output_tokens = n;
        }
        if let Some(n) = u64_at(u, "cache_read_input_tokens") {
            self.usage.cache_read_tokens = Some(n);
        }
        if let Some(n) = u64_at(u, "cache_creation_input_tokens") {
            self.usage.cache_write_tokens = Some(n);
        }
    }

    fn start_events(&mut self, provider_id: Option<&str>) -> Vec<AgentStreamEvent> {
        if self.started {
            return Vec::new();
        }
        self.started = true;
        let mut out = vec![AgentStreamEvent::StreamStart {
            id: self.request_id,
            model: self.model.clone(),
        }];
        if let Some(id) = provider_id {
            out.push(provider_id_meta(id));
        }
        out
    }

    fn translate(&mut self, event: &str, v: &Value) -> Vec<AgentStreamEvent> {
        match event {
            "message_start" => {
                let msg = v.get("message").cloned().unwrap_or(Value::Null);
                if let Some(m) = str_at(&msg, "model") {
                    self.model = m.to_string();
                }
                if let Some(u) = msg.get("usage") {
                    self.apply_usage(u);
                }
                self.start_events(str_at(&msg, "id"))
            }
            "content_block_start" => {
                let mut out = self.start_events(None);
                let index = u64_at(v, "index").unwrap_or(0);
                let block = v.get("content_block").cloned().unwrap_or(Value::Null);
                let kind = match str_at(&block, "type") {
                    Some("text") => {
                        if let Some(t) = str_at(&block, "text").filter(|t| !t.is_empty()) {
                            out.push(AgentStreamEvent::text_delta(t));
                        }
                        AnthropicBlock::Text
                    }
                    Some("thinking") => AnthropicBlock::Thinking,
                    Some("tool_use") => {
                        let id = str_at(&block, "id").unwrap_or_default().to_string();
                        out.push(AgentStreamEvent::ToolCallStart {
                            id: id.clone(),
                            name: str_at(&block, "name").unwrap_or_default().to_string(),
                        });
                        AnthropicBlock::ToolUse(id)
                    }
                    _ => AnthropicBlock::Other,
                };
                self.blocks.insert(index, kind);
                out
            }
            "content_block_delta" => {
                let index = u64_at(v, "index").unwrap_or(0);
                let delta = v.get("delta").cloned().unwrap_or(Value::Null);
                let text = |k: &str| str_at(&delta, k).unwrap_or_default().to_string();
                match str_at(&delta, "type") {
                    Some("text_delta") => vec![AgentStreamEvent::TextDelta {
                        delta: text("text"),
                    }],
                    Some("thinking_delta") => vec![AgentStreamEvent::ThinkingDelta {
                        delta: text("thinking"),
                    }],
                    Some("input_json_delta") => match self.blocks.get(&index) {
                        Some(AnthropicBlock::ToolUse(id)) => {
                            vec![AgentStreamEvent::ToolCallDelta {
                                id: id.clone(),
                                args_delta: text("partial_json"),
                            }]
                        }
                        _ => Vec::new(),
                    },
                    // signature_delta / citations_delta / future types: no UI payload.
                    _ => Vec::new(),
                }
            }
            "content_block_stop" => {
                let index = u64_at(v, "index").unwrap_or(0);
                match self.blocks.remove(&index) {
                    Some(AnthropicBlock::ToolUse(id)) => vec![AgentStreamEvent::ToolCallEnd { id }],
                    _ => Vec::new(),
                }
            }
            "message_delta" => {
                if let Some(r) = v.get("delta").and_then(|d| str_at(d, "stop_reason")) {
                    self.stop_reason = Some(StopReason::parse_loose(r));
                }
                if let Some(u) = v.get("usage") {
                    self.apply_usage(u);
                }
                Vec::new()
            }
            "message_stop" => vec![AgentStreamEvent::Done {
                stop_reason: self.stop_reason.unwrap_or(StopReason::Stop),
                usage: self.usage,
            }],
            "error" => {
                let err = v.get("error").cloned().unwrap_or(Value::Null);
                let ty = str_at(&err, "type").unwrap_or("error");
                let recoverable = matches!(
                    ty,
                    "overloaded_error" | "api_error" | "rate_limit_error" | "timeout_error"
                );
                let msg = format!(
                    "anthropic: {ty}: {}",
                    str_at(&err, "message").unwrap_or_default()
                );
                fail(
                    StreamError::new("provider_error", msg, recoverable),
                    self.usage,
                )
            }
            // ping + unknown future events are ignored (forward compat).
            _ => Vec::new(),
        }
    }
}

impl SseTranslator for AnthropicTranslator {
    fn provider(&self) -> &'static str {
        "anthropic"
    }

    fn on_frame(&mut self, frame: SseFrame) -> Vec<AgentStreamEvent> {
        let v = match parse_json("anthropic", &frame.data) {
            Ok(v) => v,
            Err(e) => return fail(e, self.usage),
        };
        // `event:` names the frame; the JSON `type` field repeats it.
        let event = frame
            .event
            .or_else(|| str_at(&v, "type").map(str::to_string))
            .unwrap_or_default();
        self.translate(&event, &v)
    }

    fn on_eof(&mut self) -> Vec<AgentStreamEvent> {
        // Truncated body (proxy reset, server crash): transient, retryable.
        fail(
            StreamError::new(
                "protocol_violation",
                "anthropic: stream ended before message_stop",
                true,
            ),
            self.usage,
        )
    }

    fn usage(&self) -> Usage {
        self.usage
    }
}

// =====================================================================
// OpenAI Chat Completions streaming
// =====================================================================

/// OpenAI `/v1/chat/completions` (`stream: true`) translator.
/// Ref: <https://platform.openai.com/docs/api-reference/chat-streaming>.
#[derive(Debug)]
pub(crate) struct OpenAiTranslator {
    request_id: Uuid,
    model: String,
    started: bool,
    /// `tool_calls[i].index` → tool-call id, in open order.
    open_tools: Vec<(u64, String)>,
    stop_reason: Option<StopReason>,
    usage: Usage,
}

impl OpenAiTranslator {
    pub(crate) fn new(request_id: Uuid, model: String) -> Self {
        Self {
            request_id,
            model,
            started: false,
            open_tools: Vec::new(),
            stop_reason: None,
            usage: Usage::default(),
        }
    }

    fn close_tools(&mut self) -> Vec<AgentStreamEvent> {
        self.open_tools
            .drain(..)
            .map(|(_, id)| AgentStreamEvent::ToolCallEnd { id })
            .collect()
    }

    fn done(&mut self) -> Vec<AgentStreamEvent> {
        let mut out = self.close_tools();
        out.push(AgentStreamEvent::Done {
            stop_reason: self.stop_reason.unwrap_or(StopReason::Stop),
            usage: self.usage,
        });
        out
    }

    fn translate_chunk(&mut self, v: &Value) -> Vec<AgentStreamEvent> {
        let mut out = Vec::new();
        if let Some(err) = v.get("error") {
            let msg = format!(
                "openai: {}",
                str_at(err, "message").unwrap_or("provider error")
            );
            return fail(StreamError::new("provider_error", msg, false), self.usage);
        }
        if !self.started {
            self.started = true;
            if let Some(m) = str_at(v, "model") {
                self.model = m.to_string();
            }
            out.push(AgentStreamEvent::StreamStart {
                id: self.request_id,
                model: self.model.clone(),
            });
            if let Some(id) = str_at(v, "id") {
                out.push(provider_id_meta(id));
            }
        }
        if let Some(u) = v.get("usage").filter(|u| u.is_object()) {
            self.usage.input_tokens = u64_at(u, "prompt_tokens").unwrap_or(0);
            self.usage.output_tokens = u64_at(u, "completion_tokens").unwrap_or(0);
            self.usage.cache_read_tokens = u
                .get("prompt_tokens_details")
                .and_then(|d| u64_at(d, "cached_tokens"));
        }
        let Some(choice) = v.get("choices").and_then(|c| c.get(0)) else {
            return out;
        };
        let delta = choice.get("delta").cloned().unwrap_or(Value::Null);
        // `reasoning_content` is the de-facto field on OpenAI-compatible
        // reasoning endpoints (DeepSeek, vLLM, OpenRouter).
        if let Some(t) = str_at(&delta, "reasoning_content").filter(|t| !t.is_empty()) {
            out.push(AgentStreamEvent::ThinkingDelta {
                delta: t.to_string(),
            });
        }
        if let Some(t) = str_at(&delta, "content").filter(|t| !t.is_empty()) {
            out.push(AgentStreamEvent::text_delta(t));
        }
        for call in delta
            .get("tool_calls")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let index = u64_at(call, "index").unwrap_or(0);
            let func = call.get("function").cloned().unwrap_or(Value::Null);
            let known = self
                .open_tools
                .iter()
                .find(|(i, _)| *i == index)
                .map(|(_, id)| id.clone());
            let id = match known {
                Some(id) => id,
                None => {
                    let id = str_at(call, "id")
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("call_{index}"));
                    self.open_tools.push((index, id.clone()));
                    out.push(AgentStreamEvent::ToolCallStart {
                        id: id.clone(),
                        name: str_at(&func, "name").unwrap_or_default().to_string(),
                    });
                    id
                }
            };
            if let Some(args) = str_at(&func, "arguments").filter(|a| !a.is_empty()) {
                out.push(AgentStreamEvent::ToolCallDelta {
                    id,
                    args_delta: args.to_string(),
                });
            }
        }
        if let Some(r) = str_at(choice, "finish_reason") {
            self.stop_reason = Some(StopReason::parse_loose(r));
            out.extend(self.close_tools());
        }
        out
    }
}

impl SseTranslator for OpenAiTranslator {
    fn provider(&self) -> &'static str {
        "openai"
    }

    fn on_frame(&mut self, frame: SseFrame) -> Vec<AgentStreamEvent> {
        if frame.data.trim() == "[DONE]" {
            return self.done();
        }
        match parse_json("openai", &frame.data) {
            Ok(v) => self.translate_chunk(&v),
            Err(e) => fail(e, self.usage),
        }
    }

    fn on_eof(&mut self) -> Vec<AgentStreamEvent> {
        // Some OpenAI-compatible servers omit `[DONE]`; a seen
        // `finish_reason` is enough to close cleanly.
        if self.stop_reason.is_some() {
            return self.done();
        }
        fail(
            StreamError::new(
                "protocol_violation",
                "openai: stream ended before finish_reason / [DONE]",
                true,
            ),
            self.usage,
        )
    }

    fn usage(&self) -> Usage {
        self.usage
    }
}

// =====================================================================
// Driver + no-throw guard
// =====================================================================

struct DriveState<S, T> {
    body: S,
    decoder: SseDecoder,
    translator: T,
    pending: VecDeque<AgentStreamEvent>,
    finished: bool,
}

/// Pump an HTTP body through [`SseDecoder`] + `translator`.
///
/// Transport errors mid-body become `Error` + `Done { Error }`; a body that
/// ends without a terminal frame is delegated to [`SseTranslator::on_eof`].
pub(crate) fn drive<S, B, T>(body: S, translator: T) -> BoxStream<'static, AgentStreamEvent>
where
    S: Stream<Item = Result<B, reqwest::Error>> + Send + Unpin + 'static,
    B: AsRef<[u8]> + Send + 'static,
    T: SseTranslator,
{
    let state = DriveState {
        body,
        decoder: SseDecoder::default(),
        translator,
        pending: VecDeque::new(),
        finished: false,
    };
    stream::unfold(state, |mut st| async move {
        loop {
            if let Some(ev) = st.pending.pop_front() {
                return Some((ev, st));
            }
            if st.finished {
                return None;
            }
            let provider = st.translator.provider();
            match st.body.next().await {
                Some(Ok(bytes)) => match st.decoder.push(bytes.as_ref()) {
                    Ok(frames) => {
                        for f in frames {
                            let events = st.translator.on_frame(f);
                            let terminal = events.iter().any(AgentStreamEvent::is_terminal);
                            st.pending.extend(events);
                            // Stop at the terminal frame; trailing bytes are ignored.
                            if terminal {
                                st.finished = true;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        st.pending.extend(fail(
                            protocol_error(provider, format!("invalid UTF-8 in SSE frame: {e}")),
                            st.translator.usage(),
                        ));
                        st.finished = true;
                    }
                },
                Some(Err(e)) => {
                    st.pending
                        .extend(fail(transport_error(provider, &e), st.translator.usage()));
                    st.finished = true;
                }
                None => {
                    match st.decoder.finish() {
                        Ok(Some(f)) => st.pending.extend(st.translator.on_frame(f)),
                        Ok(None) => {}
                        Err(e) => st.pending.extend(fail(
                            protocol_error(provider, format!("invalid UTF-8 in SSE frame: {e}")),
                            st.translator.usage(),
                        )),
                    }
                    if !st.pending.iter().any(AgentStreamEvent::is_terminal) {
                        st.pending.extend(st.translator.on_eof());
                    }
                    st.finished = true;
                }
            }
        }
    })
    .boxed()
}

/// PI-3 no-throw guard (FR-16 / FR-17).
///
/// - A panic anywhere inside `inner` (SDK, parser, translator) becomes
///   `Error { kind: "sdk_panic" }` + `Done { Error }`.
/// - Events after the first terminal event are dropped.
/// - An `inner` that ends without a terminal event gets a
///   `protocol_violation` error + `Done { Error }` appended.
pub(crate) fn no_throw(
    provider: &'static str,
    inner: BoxStream<'static, AgentStreamEvent>,
) -> BoxStream<'static, AgentStreamEvent> {
    let caught = AssertUnwindSafe(inner).catch_unwind();
    stream::unfold(
        (caught.boxed(), VecDeque::new(), false),
        move |(mut s, mut pending, mut terminated)| async move {
            loop {
                if let Some(ev) = pending.pop_front() {
                    return Some((ev, (s, pending, terminated)));
                }
                if terminated {
                    return None;
                }
                match s.next().await {
                    Some(Ok(ev)) => {
                        terminated = ev.is_terminal();
                        return Some((ev, (s, pending, terminated)));
                    }
                    Some(Err(_panic)) => {
                        pending.extend(fail(
                            StreamError::new(
                                "sdk_panic",
                                format!("{provider}: provider stream panicked"),
                                false,
                            ),
                            Usage::default(),
                        ));
                        terminated = true;
                    }
                    None => {
                        pending.extend(fail(
                            protocol_error(provider, "stream ended without a terminal event"),
                            Usage::default(),
                        ));
                        terminated = true;
                    }
                }
            }
        },
    )
    .boxed()
}

// =====================================================================
// Unit Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(chunks: &[&[u8]]) -> Vec<SseFrame> {
        let mut d = SseDecoder::default();
        let mut out = Vec::new();
        for c in chunks {
            out.extend(d.push(c).unwrap());
        }
        out.extend(d.finish().unwrap());
        out
    }

    fn run<T: SseTranslator>(mut t: T, sse: &str) -> Vec<AgentStreamEvent> {
        let mut out = Vec::new();
        for f in frames(&[sse.as_bytes()]) {
            out.extend(t.on_frame(f));
            if out.iter().any(AgentStreamEvent::is_terminal) {
                return out;
            }
        }
        out.extend(t.on_eof());
        out
    }

    #[test]
    fn sse_decoder_handles_event_multiline_data_comments_and_crlf() {
        let f = frames(&[b": keep-alive\r\n\r\nevent: a\r\ndata: x\r\ndata: y\r\n\r\ndata:z\n\n"]);
        assert_eq!(
            f,
            vec![
                SseFrame {
                    event: Some("a".into()),
                    data: "x\ny".into()
                },
                SseFrame {
                    event: None,
                    data: "z".into()
                },
            ]
        );
    }

    #[test]
    fn sse_decoder_reassembles_utf8_split_across_chunks() {
        let bytes = "data: 你好\n\n".as_bytes();
        // Split inside the 3-byte encoding of '你'.
        let f = frames(&[&bytes[..7], &bytes[7..]]);
        assert_eq!(f[0].data, "你好");
    }

    #[test]
    fn sse_decoder_finish_flushes_unterminated_frame() {
        let f = frames(&[b"data: tail"]);
        assert_eq!(f[0].data, "tail");
    }

    #[test]
    fn http_status_error_classifies_recoverability() {
        assert!(!http_status_error("x", 400).recoverable);
        assert!(!http_status_error("x", 401).recoverable);
        assert!(http_status_error("x", 429).recoverable);
        assert!(http_status_error("x", 503).recoverable);
        assert_eq!(http_status_error("x", 503).kind, "http_5xx");
        assert_eq!(http_status_error("x", 404).http_status, Some(404));
    }

    const ANTHROPIC_OK: &str = concat!(
        "event: message_start\n",
        r#"data: {"type":"message_start","message":{"id":"msg_1","model":"claude-x","usage":{"input_tokens":10,"output_tokens":1,"cache_read_input_tokens":4}}}"#,
        "\n\nevent: ping\ndata: {\"type\":\"ping\"}\n\n",
        "event: content_block_start\n",
        r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":""}}"#,
        "\n\nevent: content_block_delta\n",
        r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"hmm"}}"#,
        "\n\nevent: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "event: content_block_start\n",
        r#"data: {"type":"content_block_start","index":1,"content_block":{"type":"text","text":""}}"#,
        "\n\nevent: content_block_delta\n",
        r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"text_delta","text":"Hi"}}"#,
        "\n\nevent: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":1}\n\n",
        "event: content_block_start\n",
        r#"data: {"type":"content_block_start","index":2,"content_block":{"type":"tool_use","id":"toolu_1","name":"read_file","input":{}}}"#,
        "\n\nevent: content_block_delta\n",
        r#"data: {"type":"content_block_delta","index":2,"delta":{"type":"input_json_delta","partial_json":"{\"p\":1}"}}"#,
        "\n\nevent: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":2}\n\n",
        "event: message_delta\n",
        r#"data: {"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":42}}"#,
        "\n\nevent: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
    );

    #[test]
    fn anthropic_translator_maps_full_stream() {
        let id = Uuid::new_v4();
        let ev = run(
            AnthropicTranslator::new(id, "req-model".into()),
            ANTHROPIC_OK,
        );
        assert_eq!(
            ev,
            vec![
                AgentStreamEvent::StreamStart {
                    id,
                    model: "claude-x".into()
                },
                provider_id_meta("msg_1"),
                AgentStreamEvent::ThinkingDelta {
                    delta: "hmm".into()
                },
                AgentStreamEvent::text_delta("Hi"),
                AgentStreamEvent::ToolCallStart {
                    id: "toolu_1".into(),
                    name: "read_file".into()
                },
                AgentStreamEvent::ToolCallDelta {
                    id: "toolu_1".into(),
                    args_delta: "{\"p\":1}".into()
                },
                AgentStreamEvent::ToolCallEnd {
                    id: "toolu_1".into()
                },
                AgentStreamEvent::Done {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage {
                        input_tokens: 10,
                        output_tokens: 42,
                        cache_read_tokens: Some(4),
                        cache_write_tokens: None,
                        cost_usd: 0.0,
                    },
                },
            ]
        );
    }

    #[test]
    fn anthropic_translator_error_event_is_encoded_not_thrown() {
        let sse = "event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n\n";
        let ev = run(AnthropicTranslator::new(Uuid::nil(), "m".into()), sse);
        match &ev[0] {
            AgentStreamEvent::Error { error, recoverable } => {
                assert!(*recoverable);
                assert_eq!(error.kind, "provider_error");
                assert!(error.message.contains("Overloaded"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
        assert!(matches!(
            ev[1],
            AgentStreamEvent::Done {
                stop_reason: StopReason::Error,
                ..
            }
        ));
    }

    #[test]
    fn anthropic_translator_truncated_stream_is_protocol_violation() {
        let sse = "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"m\",\"model\":\"c\"}}\n\n";
        let ev = run(AnthropicTranslator::new(Uuid::nil(), "c".into()), sse);
        assert!(matches!(
            &ev[ev.len() - 2],
            AgentStreamEvent::Error { error, .. } if error.kind == "protocol_violation"
        ));
        assert!(ev.last().unwrap().is_terminal());
    }

    #[test]
    fn anthropic_translator_malformed_json_is_protocol_violation() {
        let ev = run(
            AnthropicTranslator::new(Uuid::nil(), "c".into()),
            "event: message_start\ndata: {not json\n\n",
        );
        assert!(matches!(
            &ev[0],
            AgentStreamEvent::Error { error, recoverable: false } if error.kind == "protocol_violation"
        ));
    }

    const OPENAI_OK: &str = concat!(
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"reasoning_content":"think"},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"content":"Hel"},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"content":"lo"},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_a","type":"function","function":{"name":"bash","arguments":""}}]},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"cmd\":"}}]},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"ls\"}"}}]},"finish_reason":null}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#,
        "\n\n",
        r#"data: {"id":"chatcmpl-1","model":"gpt-x","choices":[],"usage":{"prompt_tokens":7,"completion_tokens":9,"prompt_tokens_details":{"cached_tokens":2}}}"#,
        "\n\ndata: [DONE]\n\n",
    );

    #[test]
    fn openai_translator_maps_full_stream() {
        let id = Uuid::new_v4();
        let ev = run(OpenAiTranslator::new(id, "req".into()), OPENAI_OK);
        assert_eq!(
            ev,
            vec![
                AgentStreamEvent::StreamStart {
                    id,
                    model: "gpt-x".into()
                },
                provider_id_meta("chatcmpl-1"),
                AgentStreamEvent::ThinkingDelta {
                    delta: "think".into()
                },
                AgentStreamEvent::text_delta("Hel"),
                AgentStreamEvent::text_delta("lo"),
                AgentStreamEvent::ToolCallStart {
                    id: "call_a".into(),
                    name: "bash".into()
                },
                AgentStreamEvent::ToolCallDelta {
                    id: "call_a".into(),
                    args_delta: "{\"cmd\":".into()
                },
                AgentStreamEvent::ToolCallDelta {
                    id: "call_a".into(),
                    args_delta: "\"ls\"}".into()
                },
                AgentStreamEvent::ToolCallEnd {
                    id: "call_a".into()
                },
                AgentStreamEvent::Done {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage {
                        input_tokens: 7,
                        output_tokens: 9,
                        cache_read_tokens: Some(2),
                        cache_write_tokens: None,
                        cost_usd: 0.0,
                    },
                },
            ]
        );
    }

    #[test]
    fn openai_translator_eof_after_finish_reason_closes_cleanly() {
        let sse = r#"data: {"id":"c","choices":[{"index":0,"delta":{"content":"x"},"finish_reason":"length"}]}"#;
        let ev = run(
            OpenAiTranslator::new(Uuid::nil(), "m".into()),
            &format!("{sse}\n\n"),
        );
        assert_eq!(
            ev.last(),
            Some(&AgentStreamEvent::Done {
                stop_reason: StopReason::Length,
                usage: Usage::default()
            })
        );
    }

    #[test]
    fn openai_translator_eof_without_finish_is_error() {
        let sse = r#"data: {"id":"c","choices":[{"index":0,"delta":{"content":"x"},"finish_reason":null}]}"#;
        let ev = run(
            OpenAiTranslator::new(Uuid::nil(), "m".into()),
            &format!("{sse}\n\n"),
        );
        assert!(matches!(
            &ev[ev.len() - 2],
            AgentStreamEvent::Error { error, recoverable: true } if error.kind == "protocol_violation"
        ));
    }

    #[test]
    fn openai_translator_inline_error_object_is_encoded() {
        let ev = run(
            OpenAiTranslator::new(Uuid::nil(), "m".into()),
            "data: {\"error\":{\"message\":\"bad things\"}}\n\n",
        );
        assert!(matches!(
            &ev[0],
            AgentStreamEvent::Error { error, .. } if error.message.contains("bad things")
        ));
    }

    #[tokio::test]
    async fn drive_reassembles_frames_split_across_body_chunks() {
        let bytes = ANTHROPIC_OK.as_bytes();
        let chunks: Vec<Result<Vec<u8>, reqwest::Error>> =
            bytes.chunks(13).map(|c| Ok(c.to_vec())).collect();
        let ev: Vec<_> = drive(
            stream::iter(chunks),
            AnthropicTranslator::new(Uuid::nil(), "m".into()),
        )
        .collect()
        .await;
        assert_eq!(
            ev,
            run(
                AnthropicTranslator::new(Uuid::nil(), "m".into()),
                ANTHROPIC_OK
            )
        );
    }

    /// FR-18 class "SDK panic": a panicking provider stream is converted
    /// into `Error { sdk_panic }` + `Done { Error }`, never a propagated panic.
    #[tokio::test]
    async fn no_throw_converts_panic_into_error_and_done() {
        let inner = stream::iter(vec![0u8, 1])
            .map(|i| {
                if i == 1 {
                    panic!("simulated SDK panic");
                }
                AgentStreamEvent::text_delta("before")
            })
            .boxed();
        let ev: Vec<_> = no_throw("test", inner).collect().await;
        assert_eq!(ev[0], AgentStreamEvent::text_delta("before"));
        assert!(matches!(
            &ev[1],
            AgentStreamEvent::Error { error, recoverable: false } if error.kind == "sdk_panic"
        ));
        assert!(matches!(
            ev[2],
            AgentStreamEvent::Done {
                stop_reason: StopReason::Error,
                ..
            }
        ));
        assert_eq!(ev.len(), 3);
    }

    #[tokio::test]
    async fn no_throw_drops_events_after_terminal_and_closes_open_streams() {
        let after_done = stream::iter(vec![
            AgentStreamEvent::done_default(),
            AgentStreamEvent::text_delta("late"),
        ])
        .boxed();
        let ev: Vec<_> = no_throw("t", after_done).collect().await;
        assert_eq!(ev, vec![AgentStreamEvent::done_default()]);

        let unterminated = stream::iter(vec![AgentStreamEvent::text_delta("x")]).boxed();
        let ev: Vec<_> = no_throw("t", unterminated).collect().await;
        assert_eq!(ev.len(), 3);
        assert!(ev[2].is_terminal());
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! ULYS-178 W2 IT — `stream_completion_v2` over real HTTP (loopback mock server).
//!
//! Per SRS-PI-BORROW-001 FR-18: provider failures (HTTP 4xx / HTTP 5xx /
//! timeout / protocol error) must be encoded as `AgentStreamEvent::Error`
//! plus a terminal `Done`, never as `Err`. The 5th class (SDK panic) cannot
//! be injected through HTTP; the same guard is covered by the unit test
//! `no_throw_converts_panic_into_error_and_done` in `provider/sse.rs`.

use std::time::Duration;

use domain_llm::provider::{AnthropicProvider, OpenAiProvider};
use domain_llm::{
    AgentStreamEvent, ChatMessage, ChatRequest, LlmProvider, StopReason, StreamError,
};
use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const SSE_HEAD: &str =
    "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n";

fn request(model: &str) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage::user("hi")],
        temperature: None,
        max_tokens: Some(64),
        request_id: None,
        thinking_level: None,
    }
}

/// Consume one HTTP/1.1 request (headers + `content-length` body).
async fn read_request(sock: &mut TcpStream) {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = sock.read(&mut tmp).await.unwrap();
        if n == 0 {
            return;
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..end]).to_ascii_lowercase();
            let len = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if buf.len() >= end + 4 + len {
                return;
            }
        }
    }
}

/// One-shot loopback server: writes `parts` in order (flushing between
/// them to exercise chunk boundaries), then optionally stalls.
async fn serve(parts: Vec<String>, stall: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        read_request(&mut sock).await;
        for p in parts {
            if sock.write_all(p.as_bytes()).await.is_err() {
                return;
            }
            let _ = sock.flush().await;
        }
        if stall {
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
    format!("http://{addr}")
}

fn status_response(code: u16) -> Vec<String> {
    vec![format!(
        "HTTP/1.1 {code} X\r\ncontent-type: application/json\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{{}}"
    )]
}

fn anthropic(base: &str) -> AnthropicProvider {
    AnthropicProvider::with_api_key("test-key")
        .with_base_url(base)
        .with_network(true)
        .with_timeout(Duration::from_millis(500))
}

fn openai(base: &str) -> OpenAiProvider {
    OpenAiProvider::with_api_key("test-key")
        .with_base_url(base)
        .with_network(true)
        .with_timeout(Duration::from_millis(500))
}

async fn collect(p: &dyn LlmProvider, model: &str) -> Vec<AgentStreamEvent> {
    let s = p
        .stream_completion_v2(request(model))
        .await
        .expect("FR-16: stream_completion_v2 never returns Err");
    let events: Vec<_> = tokio::time::timeout(Duration::from_secs(10), s.collect())
        .await
        .expect("stream must terminate");
    // Exactly one terminal event, and it is last.
    assert_eq!(
        events.iter().filter(|e| e.is_terminal()).count(),
        1,
        "{events:?}"
    );
    assert!(events.last().unwrap().is_terminal(), "{events:?}");
    events
}

/// Returns the single `Error` payload and asserts the stream closed with
/// `Done { Error }`.
fn single_error(events: &[AgentStreamEvent]) -> StreamError {
    let errors: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            AgentStreamEvent::Error { error, recoverable } => {
                assert_eq!(*recoverable, error.recoverable);
                Some(error.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(errors.len(), 1, "{events:?}");
    assert!(matches!(
        events.last(),
        Some(AgentStreamEvent::Done {
            stop_reason: StopReason::Error,
            ..
        })
    ));
    errors[0].clone()
}

fn text(events: &[AgentStreamEvent]) -> String {
    events
        .iter()
        .filter_map(|e| match e {
            AgentStreamEvent::TextDelta { delta } => Some(delta.as_str()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------
// Happy paths
// ---------------------------------------------------------------------

#[tokio::test]
async fn it_anthropic_sse_stream_translates_to_events() {
    let body = concat!(
        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_9\",\"model\":\"claude-it\",\"usage\":{\"input_tokens\":3,\"output_tokens\":1}}}\n\n",
        "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"你好, \"}}\n\n",
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"world\"}}\n\n",
        "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}\n\n",
        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
    );
    // Split mid-frame, inside the UTF-8 encoding of '你', to exercise
    // byte-level reassembly (so raw bytes, not the `serve` helper).
    let bytes = body.as_bytes();
    let cut = body.find("你").unwrap() + 1;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let raw = bytes.to_vec();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        read_request(&mut sock).await;
        sock.write_all(SSE_HEAD.as_bytes()).await.unwrap();
        sock.write_all(&raw[..cut]).await.unwrap();
        sock.flush().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        sock.write_all(&raw[cut..]).await.unwrap();
    });

    let p = anthropic(&format!("http://{addr}"));
    let ev = collect(&p, "claude-it").await;
    assert!(matches!(&ev[0], AgentStreamEvent::StreamStart { model, .. } if model == "claude-it"));
    assert_eq!(text(&ev), "你好, world");
    match ev.last().unwrap() {
        AgentStreamEvent::Done { stop_reason, usage } => {
            assert_eq!(*stop_reason, StopReason::Stop);
            assert_eq!((usage.input_tokens, usage.output_tokens), (3, 5));
        }
        other => panic!("expected Done, got {other:?}"),
    }
}

#[tokio::test]
async fn it_openai_sse_stream_translates_to_events() {
    let base = serve(
        vec![
            SSE_HEAD.to_string(),
            "data: {\"id\":\"chatcmpl-9\",\"model\":\"gpt-it\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hel\"},\"finish_reason\":null}]}\n\n".to_string(),
            "data: {\"id\":\"chatcmpl-9\",\"model\":\"gpt-it\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"lo\"},\"finish_reason\":\"stop\"}]}\n\n".to_string(),
            "data: {\"id\":\"chatcmpl-9\",\"model\":\"gpt-it\",\"choices\":[],\"usage\":{\"prompt_tokens\":4,\"completion_tokens\":2}}\n\n".to_string(),
            "data: [DONE]\n\n".to_string(),
        ],
        false,
    )
    .await;
    let ev = collect(&openai(&base), "gpt-it").await;
    assert_eq!(text(&ev), "Hello");
    match ev.last().unwrap() {
        AgentStreamEvent::Done { stop_reason, usage } => {
            assert_eq!(*stop_reason, StopReason::Stop);
            assert_eq!((usage.input_tokens, usage.output_tokens), (4, 2));
        }
        other => panic!("expected Done, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// FR-18 failure classes
// ---------------------------------------------------------------------

#[tokio::test]
async fn it_fr18_http_4xx_is_non_recoverable_error_event() {
    let base = serve(status_response(400), false).await;
    let err = single_error(&collect(&anthropic(&base), "claude-it").await);
    assert_eq!(err.kind, "http_4xx");
    assert_eq!(err.http_status, Some(400));
    assert!(!err.recoverable);
    assert!(
        !err.message.contains("test-key"),
        "守门 #5: no key in events"
    );
}

#[tokio::test]
async fn it_fr18_http_429_is_recoverable_error_event() {
    let base = serve(status_response(429), false).await;
    let err = single_error(&collect(&openai(&base), "gpt-it").await);
    assert_eq!(err.kind, "http_4xx");
    assert!(err.recoverable);
}

#[tokio::test]
async fn it_fr18_http_5xx_is_recoverable_error_event() {
    for p in ["anthropic", "openai"] {
        let base = serve(status_response(503), false).await;
        let ev = if p == "anthropic" {
            collect(&anthropic(&base), "claude-it").await
        } else {
            collect(&openai(&base), "gpt-it").await
        };
        let err = single_error(&ev);
        assert_eq!(err.kind, "http_5xx", "{p}");
        assert_eq!(err.http_status, Some(503));
        assert!(err.recoverable);
    }
}

#[tokio::test]
async fn it_fr18_timeout_mid_stream_is_recoverable_error_event() {
    let base = serve(
        vec![
            SSE_HEAD.to_string(),
            "data: {\"id\":\"c\",\"model\":\"gpt-it\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"partial\"},\"finish_reason\":null}]}\n\n".to_string(),
        ],
        true,
    )
    .await;
    let ev = collect(&openai(&base), "gpt-it").await;
    assert_eq!(text(&ev), "partial", "events before the stall are kept");
    let err = single_error(&ev);
    assert_eq!(err.kind, "timeout");
    assert!(err.recoverable);
}

#[tokio::test]
async fn it_fr18_protocol_error_is_error_event() {
    let base = serve(
        vec![
            SSE_HEAD.to_string(),
            "event: message_start\ndata: {oops\n\n".to_string(),
        ],
        false,
    )
    .await;
    let err = single_error(&collect(&anthropic(&base), "claude-it").await);
    assert_eq!(err.kind, "protocol_violation");
    assert!(!err.recoverable);
}

#[tokio::test]
async fn it_fr18_connection_refused_is_recoverable_network_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let err = single_error(&collect(&anthropic(&format!("http://{addr}")), "claude-it").await);
    // Linux refuses instantly ("network"); Windows retries the SYN for ~2s,
    // so the 500ms connect timeout fires first ("timeout"). Both retryable.
    assert!(
        matches!(err.kind.as_str(), "network" | "timeout"),
        "{err:?}"
    );
    assert!(err.recoverable);
}

// ---------------------------------------------------------------------
// Pre-flight paths (no HTTP)
// ---------------------------------------------------------------------

#[tokio::test]
async fn it_no_network_mode_emits_stub_stream() {
    let ev = collect(&AnthropicProvider::new(), "claude-it").await;
    assert!(matches!(ev[0], AgentStreamEvent::StreamStart { .. }));
    assert!(text(&ev).contains("[anthropic stub: stream]"));
    assert_eq!(ev.last(), Some(&AgentStreamEvent::done_default()));
}

#[tokio::test]
async fn it_invalid_request_is_error_event_not_err() {
    let err = single_error(&collect(&OpenAiProvider::new(), "").await);
    assert_eq!(err.kind, "invalid_request");
    assert!(!err.recoverable);
}

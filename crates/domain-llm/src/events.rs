// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/events.rs` -- PI-1 + PI-2 from SRS-PI-BORROW-001.
//!
//! Adds the 12-variant [`AgentStreamEvent`] streaming protocol, the 7-value
//! [`StopReason`] enum, the 5-tuple [`Usage`] struct (replacing the lossy
//! 3-tuple `TokenUsage` for *per-call* accounting), and the 6-value
//! [`ThinkingLevel`] for per-model reasoning routing. All borrowed-shape
//! from `earendil-works/pi` v0.87.0 (per `deliverables/ULYS-175/analysis-pi-borrow-build.md`).
//!
//! **Additive (v0.0.2)**: existing `ChatChunk` / `finish_reason: String` are
//! preserved untouched for 1 version. New code should prefer
//! `LlmProvider::stream_completion_v2` (returning `AgentStreamEvent`) and
//! `ChatResponse::stop_reason`. Removal lands in the next minor.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #13 a W/T/M + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"`).
//! - `#[non_exhaustive]` on enums that may grow (per SRS-PI-BORROW-001 §4 FR-1, NFR-7).
//! - All DTOs derive `Serialize` / `Deserialize` (per 守门 #11 缺标比错标).
//! - `StreamError` carries `recoverable: bool` so the agent loop can decide
//!   whether to retry (per SRS-PI-BORROW-001 §4 FR-5).
//! - 5 域 Lead signoff (per ULYS-175 2026-09-22 01:28 JST): Ulysses = 5-domain Lead.

use serde::{Deserialize, Serialize};

// =====================================================================
// StopReason (PI-1 / FR-7, per pi-ai/src/types.ts:84)
// =====================================================================

/// **StopReason** -- 7-value enum replacing `finish_reason: String`.
///
/// Maps to Pi's `StopReason` (`Stop` / `Length` / `ToolUse` / `Error` /
/// `Aborted` / `ContentFilter` / `Other`). OpenAI/Google/Anthropic all
/// collapse into this enum; the legacy `String` field on `ChatResponse` is
/// retained as `#[deprecated]` and derived from this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StopReason {
    /// Model finished naturally (OpenAI `stop`, Anthropic `end_turn`).
    Stop,
    /// Hit `max_tokens` / context window limit (OpenAI `length`).
    Length,
    /// Model emitted a tool/function call (OpenAI `tool_calls`).
    ToolUse,
    /// Provider returned an error mid-stream; payload carried in
    /// [`AgentStreamEvent::Error`].
    Error,
    /// Caller aborted (e.g. UI navigation, agent policy violation).
    Aborted,
    /// Provider-side safety filter triggered (OpenAI `content_filter`).
    ContentFilter,
    /// Anything else (reserved for future provider-specific reasons).
    Other,
}

impl StopReason {
    /// Stable wire-format string form (lowercase, matches Pi + OpenAI).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Length => "length",
            Self::ToolUse => "tool_use",
            Self::Error => "error",
            Self::Aborted => "aborted",
            Self::ContentFilter => "content_filter",
            Self::Other => "other",
        }
    }

    /// Reverse of [`StopReason::as_str`] -- case-insensitive; unknown
    /// strings fall through to [`StopReason::Other`].
    pub fn parse_loose(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "stop" | "end_turn" | "stop_reason_end_turn" => Self::Stop,
            "length" | "max_tokens" => Self::Length,
            "tool_use" | "tool_calls" | "tool_choice" => Self::ToolUse,
            "error" => Self::Error,
            "aborted" | "cancelled" | "canceled" => Self::Aborted,
            "content_filter" | "safety" => Self::ContentFilter,
            _ => Self::Other,
        }
    }
}

impl Default for StopReason {
    fn default() -> Self {
        Self::Stop
    }
}

// =====================================================================
// ThinkingLevel (PI-2 / FR-10, per pi-ai/src/types.ts:419)
// =====================================================================

/// **ThinkingLevel** -- per-model reasoning depth (PI-2 / FR-10).
///
/// Models that do not support thinking treat every value as `Off`. The
/// provider dispatcher (`DispatchProvider`) maps each level to the
/// model-specific knob (Anthropic `thinking.budget_tokens`, OpenAI
/// `reasoning_effort`, Google `thinking_budget`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ThinkingLevel {
    /// No extended reasoning; cheapest + fastest.
    Off,
    /// Minimal -- e.g. Anthropic 1024 tokens, OpenAI `low`.
    Minimal,
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort (default for capable models).
    Medium,
    /// High reasoning effort (Anthropic `high`, OpenAI `high`).
    High,
    /// Extra-high -- beyond provider presets; provider may cap.
    XHigh,
}

impl ThinkingLevel {
    /// Stable wire-format string form.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
        }
    }
}

impl Default for ThinkingLevel {
    fn default() -> Self {
        Self::Off
    }
}

// =====================================================================
// Usage (PI-2 / FR-8, per pi-ai/src/types.ts:396-417)
// =====================================================================

/// **Usage** -- per-completion token + cost accounting (5 fields).
///
/// Distinct from [`crate::metering::TokenUsage`] (which is a per-event
/// metering row keyed by event id). `Usage` is the *result* carried in the
/// terminal `AgentStreamEvent::Done` and on `ChatResponse::usage`. The two
/// co-exist for one version; v0.1+ may unify.
///
/// `cache_read_tokens` / `cache_write_tokens` are `Option` because not every
/// provider surfaces them. `cost_usd` follows
/// "provider value first, static price-list fallback" (per SRS-PI-BORROW-001
/// FR-12 / PI-2 §4).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    /// Input tokens billed by the provider.
    pub input_tokens: u64,
    /// Output tokens generated by the model.
    pub output_tokens: u64,
    /// Tokens served from provider cache (Anthropic cache hit, OpenAI
    /// `cached_tokens`). `None` if the provider does not report this.
    pub cache_read_tokens: Option<u64>,
    /// Tokens written into provider cache (Anthropic 5m/1h cache write).
    /// `None` if not reported.
    pub cache_write_tokens: Option<u64>,
    /// USD cost for this completion. Provider-reported when available,
    /// else derived from `MODEL_PRICE_TABLE` (per FR-12).
    pub cost_usd: f64,
}

impl Usage {
    /// Build a `Usage` from the two required counts; cache + cost default
    /// to `None` / `0.0`.
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cache_read_tokens: None,
            cache_write_tokens: None,
            cost_usd: 0.0,
        }
    }

    /// Convenience: derive cache fields + cost in one go.
    pub fn with_cache_and_cost(
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
        cost_usd: f64,
    ) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cache_read_tokens: Some(cache_read_tokens),
            cache_write_tokens: Some(cache_write_tokens),
            cost_usd,
        }
    }

    /// Convenience: total tokens billed, cache-inclusive.
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_add(self.cache_read_tokens.unwrap_or(0))
            .saturating_add(self.cache_write_tokens.unwrap_or(0))
    }
}

impl Default for Usage {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

// =====================================================================
// StreamError (PI-3 / FR-5, per pi-agent-core/src/types.ts:19-37)
// =====================================================================

/// **StreamError** -- payload for [`AgentStreamEvent::Error`].
///
/// `recoverable: bool` lets the agent loop decide whether to retry
/// (transient 429 / 5xx / timeout) or surface the failure to the caller
/// (4xx other than 429, SDK panic, protocol violation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamError {
    /// Stable error category (e.g. `"http_4xx"`, `"http_5xx"`, `"timeout"`,
    /// `"sdk_panic"`, `"protocol_violation"`, `"aborted"`).
    pub kind: String,
    /// Human-readable detail (provider message, HTTP body excerpt).
    /// MUST NOT contain secrets / billing tokens (per 守门 #5).
    pub message: String,
    /// HTTP status code if the failure was HTTP-driven.
    pub http_status: Option<u16>,
    /// Whether the agent loop should retry.
    pub recoverable: bool,
}

impl StreamError {
    /// Convenience constructor.
    pub fn new(kind: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            http_status: None,
            recoverable,
        }
    }

    /// Convenience constructor carrying an HTTP status code.
    pub fn with_status(mut self, http_status: u16) -> Self {
        self.http_status = Some(http_status);
        self
    }
}

// =====================================================================
// AgentStreamEvent (PI-1 / FR-1, per pi-ai/src/types.ts:652-668)
// =====================================================================

/// **AgentStreamEvent** -- 12-variant streaming protocol (PI-1 / FR-1).
///
/// Replaces `ChatChunk` (kept for 1 version per FR-2). Stream-of-events
/// is the wire shape Pi ships; we adopt it because the no-throw contract
/// (PI-3) requires encoding failure inline rather than throwing.
///
/// The 12 variants mirror Pi's `AssistantMessageEvent` (12 variants):
/// `Start`, `TextDelta`, `ThinkingDelta`, `ToolCallStart`,
/// `ToolCallDelta`, `ToolCallEnd`, `Done`, `Error`, `Aborted`,
/// `AnnotationDelta`, `PathUpdate`, `MetaUpdate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentStreamEvent {
    /// Stream begins; carries the response id and model (mirrors Pi's
    /// `Start { id, model, role }`).
    StreamStart {
        /// Provider-issued response id.
        id: uuid::Uuid,
        /// Model name producing the reply.
        model: String,
    },
    /// Incremental text fragment (UTF-8). Concatenation of successive
    /// `TextDelta` events yields the assistant's final text.
    TextDelta {
        /// Fragment -- may be empty for non-text events.
        delta: String,
    },
    /// Incremental reasoning / chain-of-thought fragment (model's
    /// internal monologue; UI may render or hide).
    ThinkingDelta {
        /// Fragment of the reasoning text.
        delta: String,
    },
    /// Beginning of a tool/function call (OpenAI `tool_calls[i].id`).
    ToolCallStart {
        /// Provider-issued tool-call id.
        id: String,
        /// Tool function name (e.g. `read_file`, `bash`).
        name: String,
    },
    /// Incremental JSON-arguments fragment for a tool call.
    ToolCallDelta {
        /// Tool-call id (matches [`AgentStreamEvent::ToolCallStart`]).
        id: String,
        /// Partial arguments JSON.
        args_delta: String,
    },
    /// Tool call finalized; the agent loop can dispatch the call now.
    ToolCallEnd {
        /// Tool-call id (matches [`AgentStreamEvent::ToolCallStart`]).
        id: String,
    },
    /// Terminal success event. After `Done` the stream MUST NOT emit
    /// further events.
    Done {
        /// Why the stream ended.
        stop_reason: StopReason,
        /// Token + cost accounting for the whole completion.
        usage: Usage,
    },
    /// Stream produced an error mid-flight (per PI-3 no-throw contract).
    /// After `Error`, the agent loop decides: if `recoverable`, retry;
    /// else surface to the caller. May be followed by `Done` to formally
    /// close the stream.
    Error {
        /// Error payload.
        error: StreamError,
        /// Whether the agent loop may retry.
        recoverable: bool,
    },
    /// Stream aborted by the caller (user navigation, policy violation,
    /// agent-shutdown signal). Final event.
    Aborted,
    /// UI annotation hint (e.g. "code block", "citation", "language id").
    /// UI-only; the assistant text is not affected.
    AnnotationDelta {
        /// Stable annotation kind (e.g. `"code_block"`, `"citation"`).
        kind: String,
        /// JSON payload describing the annotation.
        payload: serde_json::Value,
    },
    /// Worktree / file path hint (per pi-coding-agent path tracking).
    /// UI-only.
    PathUpdate {
        /// Affected path.
        path: String,
        /// Action hint: `"read"`, `"write"`, `"edit"`, `"delete"`.
        action: String,
    },
    /// Free-form metadata (provider name, finish-reason raw string,
    /// provider request id). Carries arbitrary key/value pairs.
    MetaUpdate {
        /// Key (e.g. `"provider_request_id"`, `"x-ratelimit-remaining"`).
        key: String,
        /// Value (string form; structured data lives in `payload`).
        value: String,
        /// Optional structured payload.
        payload: Option<serde_json::Value>,
    },
}

impl AgentStreamEvent {
    /// Helper: is this the terminal event (`Done` or `Aborted`)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done { .. } | Self::Aborted)
    }

    /// Helper: convenience constructor for `TextDelta { delta }`.
    pub fn text_delta(delta: impl Into<String>) -> Self {
        Self::TextDelta {
            delta: delta.into(),
        }
    }

    /// Helper: convenience constructor for `Done { Stop, Usage::new(0,0) }`.
    pub fn done_default() -> Self {
        Self::Done {
            stop_reason: StopReason::Stop,
            usage: Usage::default(),
        }
    }
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 -- >= 1 unit test per type)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_reason_round_trips_via_as_str() {
        for r in [
            StopReason::Stop,
            StopReason::Length,
            StopReason::ToolUse,
            StopReason::Error,
            StopReason::Aborted,
            StopReason::ContentFilter,
            StopReason::Other,
        ] {
            assert_eq!(StopReason::parse_loose(r.as_str()), r);
        }
    }

    #[test]
    fn stop_reason_parse_loose_handles_openai_and_anthropic_aliases() {
        assert_eq!(StopReason::parse_loose("end_turn"), StopReason::Stop);
        assert_eq!(StopReason::parse_loose("tool_calls"), StopReason::ToolUse);
        assert_eq!(StopReason::parse_loose("cancelled"), StopReason::Aborted);
        assert_eq!(StopReason::parse_loose("safety"), StopReason::ContentFilter);
        assert_eq!(StopReason::parse_loose("garbage_value"), StopReason::Other);
    }

    #[test]
    fn stop_reason_default_is_stop() {
        assert_eq!(StopReason::default(), StopReason::Stop);
    }

    #[test]
    fn thinking_level_round_trips_via_as_str() {
        for lvl in [
            ThinkingLevel::Off,
            ThinkingLevel::Minimal,
            ThinkingLevel::Low,
            ThinkingLevel::Medium,
            ThinkingLevel::High,
            ThinkingLevel::XHigh,
        ] {
            assert_eq!(lvl.as_str().parse::<String>().map(|_| lvl), Ok(lvl));
        }
    }

    #[test]
    fn usage_new_sets_cache_to_none_and_cost_to_zero() {
        let u = Usage::new(100, 50);
        assert_eq!(u.input_tokens, 100);
        assert_eq!(u.output_tokens, 50);
        assert_eq!(u.cache_read_tokens, None);
        assert_eq!(u.cache_write_tokens, None);
        assert_eq!(u.cost_usd, 0.0);
    }

    #[test]
    fn usage_with_cache_and_cost_populates_all_fields() {
        let u = Usage::with_cache_and_cost(100, 50, 20, 10, 0.0042);
        assert_eq!(u.input_tokens, 100);
        assert_eq!(u.output_tokens, 50);
        assert_eq!(u.cache_read_tokens, Some(20));
        assert_eq!(u.cache_write_tokens, Some(10));
        assert!((u.cost_usd - 0.0042).abs() < 1e-9);
    }

    #[test]
    fn usage_total_tokens_sums_all_four_when_cache_present() {
        let u = Usage::with_cache_and_cost(100, 50, 20, 10, 0.0);
        assert_eq!(u.total_tokens(), 180);
    }

    #[test]
    fn usage_total_tokens_skips_none_cache_fields() {
        let u = Usage::new(100, 50);
        assert_eq!(u.total_tokens(), 150);
    }

    #[test]
    fn stream_error_new_sets_fields_and_with_status_attaches_http() {
        let e = StreamError::new("http_5xx", "upstream timeout", false).with_status(503);
        assert_eq!(e.kind, "http_5xx");
        assert_eq!(e.message, "upstream timeout");
        assert!(!e.recoverable);
        assert_eq!(e.http_status, Some(503));
    }

    #[test]
    fn agent_stream_event_text_delta_helper_works() {
        let e = AgentStreamEvent::text_delta("hello ");
        match e {
            AgentStreamEvent::TextDelta { delta } => assert_eq!(delta, "hello "),
            _ => panic!("expected TextDelta"),
        }
    }

    #[test]
    fn agent_stream_event_done_default_is_stop_with_zero_usage() {
        let e = AgentStreamEvent::done_default();
        assert!(e.is_terminal());
        match e {
            AgentStreamEvent::Done { stop_reason, usage } => {
                assert_eq!(stop_reason, StopReason::Stop);
                assert_eq!(usage.input_tokens, 0);
                assert_eq!(usage.output_tokens, 0);
            }
            _ => panic!("expected Done"),
        }
    }

    #[test]
    fn agent_stream_event_done_is_terminal_aborted_is_terminal() {
        assert!(AgentStreamEvent::Done {
            stop_reason: StopReason::Stop,
            usage: Usage::default(),
        }
        .is_terminal());
        assert!(AgentStreamEvent::Aborted.is_terminal());
        assert!(!AgentStreamEvent::text_delta("x").is_terminal());
    }

    #[test]
    fn agent_stream_event_serde_round_trip_text_delta() {
        let e = AgentStreamEvent::text_delta("abc");
        let json = serde_json::to_string(&e).unwrap();
        let back: AgentStreamEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn agent_stream_event_serde_round_trip_done_with_full_usage() {
        let e = AgentStreamEvent::Done {
            stop_reason: StopReason::ToolUse,
            usage: Usage::with_cache_and_cost(100, 50, 20, 10, 0.0042),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AgentStreamEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn agent_stream_event_serde_round_trip_tool_call_lifecycle() {
        let start = AgentStreamEvent::ToolCallStart {
            id: "tc_1".to_string(),
            name: "read_file".to_string(),
        };
        let delta = AgentStreamEvent::ToolCallDelta {
            id: "tc_1".to_string(),
            args_delta: "{\"path\":\"/".to_string(),
        };
        let end = AgentStreamEvent::ToolCallEnd {
            id: "tc_1".to_string(),
        };
        for ev in [start, delta, end] {
            let json = serde_json::to_string(&ev).unwrap();
            let back: AgentStreamEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(ev, back);
        }
    }
}
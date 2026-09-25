// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/chat.rs` — W1 (ULYS-98-W1) Chat request/response/chunk
//! DTOs + `ChatRole` enum + `Message` (per `docs/briefs/ulys-98-star-cursor-min-v1.md`
//! §"Sub-task 1.4 backend LlmProvider trait 扩展 chat_completion + stream_completion stub").
//!
//! **v0.0.1 stub (per ULYS-98-W1)**: types defined, no real provider wiring. W2
//! will plug `AnthropicProvider` + `OpenAIProvider` into `LlmProvider::chat_completion`
//! / `stream_completion`. Trait signatures already require these types so callers
//! can build request payloads today.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - DTOs derive `Serialize` / `Deserialize` for HTTP / JSON-RPC transport
//!   (per 守门 #11 缺标比错标 — explicit derives, not relying on defaults).
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// ChatRole (per OpenAI Chat Completions API spec, 4 roles)
// =====================================================================

/// **ChatRole** -- speaker role inside a `ChatRequest`.
///
/// Mirrors OpenAI Chat Completions API role enum (4 variants). Custom roles
/// (e.g. "tool", "developer") are deferred to W2 once a real provider is wired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// System prompt (instructions to the model, typically prepended).
    System,
    /// End-user message.
    User,
    /// Assistant (model) reply.
    Assistant,
    /// Tool / function result (deferred to W2 — schema reserved, not yet emitted).
    Tool,
}

impl ChatRole {
    /// Stable string form (matches the `serde(rename_all = "lowercase")` wire format).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

// =====================================================================
// ChatMessage (single message in a chat conversation)
// =====================================================================

/// **ChatMessage** -- one turn in a chat conversation.
///
/// `content` is plain UTF-8 text for v0.0.1 (no multimodal / tool-call
/// payloads yet). Provider-specific extensions (Anthropic `cache_control`,
/// OpenAI `name`, tool calls) are deferred to W2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    /// Speaker role.
    pub role: ChatRole,
    /// UTF-8 text content. v0.0.1: empty string rejected by `validate()`.
    pub content: String,
}

impl ChatMessage {
    /// Construct a `system` prompt message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    /// Construct a `user` message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    /// Construct an `assistant` message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }

    /// Cheap validation: non-empty content, role present (always present by
    /// construction). Returns a static message suitable for an `InvalidOperation`
    /// error variant.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.content.trim().is_empty() {
            return Err("chat message content must be non-empty");
        }
        Ok(())
    }
}

// =====================================================================
// ChatRequest (input to LlmProvider::chat_completion / stream_completion)
// =====================================================================

/// **ChatRequest** -- input to a chat-completion call.
///
/// Model name is required (no implicit default). Sampling knobs are minimal
/// for v0.0.1; richer controls (top_p, stop sequences, response_format) are
/// deferred to W2.
///
/// **v0.0.2 (PI-2 / FR-11)**: `thinking_level` selects per-model reasoning
/// depth (per Pi's `ThinkingLevel`). Providers that don't support
/// extended thinking treat any non-`Off` value as `Off`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    /// Model identifier (e.g. "claude-3-5-sonnet", "gpt-4o-mini").
    /// Empty string rejected.
    pub model: String,
    /// Conversation history. Must be non-empty (at least one user message).
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 - 2.0). `None` lets the provider pick a default.
    pub temperature: Option<f32>,
    /// Maximum output tokens (provider-specific upper bound enforced).
    pub max_tokens: Option<u32>,
    /// Optional caller-supplied request id for idempotency / tracing.
    pub request_id: Option<Uuid>,
    /// **v0.0.2 (PI-2 / FR-11)**: reasoning depth hint. `None` = provider
    /// default (usually `Off` for non-thinking models, `Medium` for
    /// thinking models).
    pub thinking_level: Option<crate::events::ThinkingLevel>,
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            messages: Vec::new(),
            temperature: None,
            max_tokens: None,
            request_id: None,
            thinking_level: None,
        }
    }
}

impl ChatRequest {
    /// Cheap structural validation: model + non-empty messages.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.model.trim().is_empty() {
            return Err("chat request model must be non-empty");
        }
        if self.messages.is_empty() {
            return Err("chat request must have at least one message");
        }
        for (idx, msg) in self.messages.iter().enumerate() {
            msg.validate().map_err(|_| -> &'static str {
                // Re-borrow as a static string by leaking — only on error path,
                // and only the first violation.
                Box::leak(
                    format!("chat message at index {idx} failed validation")
                        .into_boxed_str(),
                )
            })?;
        }
        Ok(())
    }
}

// =====================================================================
// ChatResponse (output of LlmProvider::chat_completion)
// =====================================================================

/// **ChatResponse** -- non-streaming chat completion result.
///
/// **v0.0.2 (PI-2 / FR-9)**: `finish_reason: String` is **deprecated**;
/// use `stop_reason: StopReason` instead. The string field is retained for
/// 1 version for backwards compatibility (per SRS-PI-BORROW-001 NFR-4).
/// `usage: Usage` carries the full 5-tuple token + cost accounting
/// (per FR-8).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatResponse {
    /// Provider-issued response id (echo of `ChatRequest::request_id` if the
    /// provider does not assign its own; for v0.0.1 stub: same UUID).
    pub id: Uuid,
    /// Model name that produced the reply (echo of `ChatRequest::model`).
    pub model: String,
    /// Single assistant message (non-streaming replies are atomic).
    pub message: ChatMessage,
    /// **Deprecated v0.0.2 (PI-2 / FR-9)**: use `stop_reason` instead.
    /// Kept for 1 version for downstream compile-compatibility (per
    /// SRS-PI-BORROW-001 NFR-4). When present it is the string form of
    /// `stop_reason`.
    #[deprecated(note = "use `stop_reason: StopReason` instead (per PI-2 / FR-9)")]
    pub finish_reason: String,
    /// **v0.0.2 (PI-2 / FR-7)**: 7-value enum replacing the legacy
    /// `finish_reason: String` field.
    pub stop_reason: crate::events::StopReason,
    /// **v0.0.2 (PI-2 / FR-8)**: 5-tuple token + cost accounting. Always
    /// present for non-streaming replies; `Usage::default()` for the stub
    /// path.
    pub usage: crate::events::Usage,
    /// Server-side timestamp at reply finalization.
    pub created_at: DateTime<Utc>,
}

impl ChatResponse {
    /// Construct a stub response carrying the given assistant `content`.
    ///
    /// Used by W1 trait stub implementations to fabricate a deterministic
    /// reply without invoking an actual LLM.
    #[allow(deprecated)] // stub path keeps the legacy string for compat
    pub fn stub_assistant(model: impl Into<String>, content: impl Into<String>) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            model: model.into(),
            message: ChatMessage::assistant(content),
            finish_reason: "stop".to_string(),
            stop_reason: crate::events::StopReason::Stop,
            usage: crate::events::Usage::default(),
            created_at: Utc::now(),
        }
    }
}

// =====================================================================
// ChatChunk (one element of a streaming completion)
// =====================================================================

/// **ChatChunk** -- one delta inside a streaming completion.
///
/// `delta` is either an incremental content fragment (`assistant` role) or,
/// for the terminal chunk, an empty `content` + populated `finish_reason`.
/// v0.0.1 stub providers may emit a single chunk; W2 Anthropic / OpenAI
/// providers will emit one chunk per SSE event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatChunk {
    /// Stream id (echo of `ChatRequest::request_id`, or provider-issued).
    pub id: Uuid,
    /// Model name (echo of `ChatRequest::model`).
    pub model: String,
    /// Delta role (always `Assistant` for v0.0.1; multi-role streams deferred to W2).
    pub role: ChatRole,
    /// Incremental content fragment. May be empty on the terminal chunk.
    pub delta: String,
    /// Stop reason. `None` for non-terminal chunks; `Some("stop" | "length")` on terminal.
    pub finish_reason: Option<String>,
}

impl ChatChunk {
    /// Construct a single stub chunk with the given content and `finish_reason = "stop"`.
    pub fn stub_terminal(model: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            model: model.into(),
            role: ChatRole::Assistant,
            delta: content.into(),
            finish_reason: Some("stop".to_string()),
        }
    }
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 — >= 1 unit test per module)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_role_as_str_matches_serde_wire_format() {
        assert_eq!(ChatRole::System.as_str(), "system");
        assert_eq!(ChatRole::User.as_str(), "user");
        assert_eq!(ChatRole::Assistant.as_str(), "assistant");
        assert_eq!(ChatRole::Tool.as_str(), "tool");
    }

    #[test]
    fn chat_message_constructors_set_expected_role() {
        let sys = ChatMessage::system("be terse");
        assert_eq!(sys.role, ChatRole::System);
        assert_eq!(sys.content, "be terse");

        let usr = ChatMessage::user("hi");
        assert_eq!(usr.role, ChatRole::User);

        let ast = ChatMessage::assistant("hello");
        assert_eq!(ast.role, ChatRole::Assistant);
    }

    #[test]
    fn chat_message_validate_rejects_empty_content() {
        let m = ChatMessage::user("   ");
        assert!(m.validate().is_err());

        let m = ChatMessage::user("hi");
        assert!(m.validate().is_ok());
    }

    #[test]
    fn chat_request_validate_rejects_empty_model_and_messages() {
        let r = ChatRequest {
            model: "".to_string(),
            ..Default::default()
        };
        assert!(r.validate().is_err());

        let r = ChatRequest {
            model: "gpt-test".to_string(),
            messages: vec![],
            ..Default::default()
        };
        assert!(r.validate().is_err());
    }

    #[test]
    fn chat_request_validate_accepts_minimal_payload() {
        let r = ChatRequest {
            model: "gpt-test".to_string(),
            messages: vec![ChatMessage::user("hi")],
            request_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        assert!(r.validate().is_ok());
    }

    #[test]
    fn chat_response_stub_assistant_sets_finish_reason_stop() {
        #[allow(deprecated)]
        let r = ChatResponse::stub_assistant("gpt-test", "hello back");
        assert_eq!(r.model, "gpt-test");
        assert_eq!(r.message.role, ChatRole::Assistant);
        assert_eq!(r.message.content, "hello back");
        #[allow(deprecated)]
        {
            assert_eq!(r.finish_reason, "stop");
        }
        assert_eq!(r.stop_reason, crate::events::StopReason::Stop);
        // id must be a non-nil UUID
        assert_ne!(r.id, Uuid::nil());
    }

    #[test]
    fn chat_chunk_stub_terminal_sets_finish_reason_stop() {
        let c = ChatChunk::stub_terminal("gpt-test", "hello back");
        assert_eq!(c.model, "gpt-test");
        assert_eq!(c.role, ChatRole::Assistant);
        assert_eq!(c.delta, "hello back");
        assert_eq!(c.finish_reason.as_deref(), Some("stop"));
        assert_ne!(c.id, Uuid::nil());
    }
}

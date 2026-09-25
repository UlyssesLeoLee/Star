// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider/mock.rs` — W2 (ULYS-98-W2) CI/no-key LLM provider.
//
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.1":
//! - mock provider 用于 CI 无 key 环境
//! - deterministic reply: echoes the last user message prefixed with `[mock]`
//! - 0 network, 0 env vars
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #25 v25):
//! - 0 `unsafe` blocks.
//! - 0 async runtime requirement beyond tokio sleep(0) equivalent (we use `async fn` no-op body).
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use async_trait::async_trait;
use chrono::Utc;
use futures_util::stream::{self, StreamExt};
use uuid::Uuid;

use crate::chat::{ChatChunk, ChatMessage, ChatRequest, ChatResponse, ChatRole};
use crate::{LlmProvider, LlmProviderRegistryError, LlmProviderRegistryHealth};

/// Default model name reported by [`MockProvider`] responses.
pub const MOCK_DEFAULT_MODEL: &str = "mock-llm";

/// Default echoed content when input messages are empty.
pub const MOCK_FALLBACK_CONTENT: &str = "(mock: no user message supplied)";

/// **MockProvider** — deterministic, network-free LLM provider for CI / tests.
///
/// Implements [`LlmProvider`] entirely without external state:
/// - `chat_completion` echoes the last user message prefixed with `[mock]` in a
///   single response (no streaming, no I/O).
/// - `stream_completion` emits one initial chunk per user message character
///   (cheap visual differentiation) and a terminal chunk with
///   `finish_reason = Some("stop")`.
#[derive(Debug, Default, Clone)]
pub struct MockProvider {
    /// Provider name surfaced via [`LlmProvider`] health output.
    name: String,
}

impl MockProvider {
    /// Create a new mock provider with the default name `mock`.
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
        }
    }

    /// Create a mock provider with a custom name (useful for tests that
    /// register multiple providers and want to disambiguate them).
    pub fn with_name(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Compose the deterministic assistant reply for a `ChatRequest`.
    fn compose_reply(req: &ChatRequest) -> String {
        let last_user = req
            .messages
            .iter()
            .rev()
            .find(|m| m.role == ChatRole::User)
            .map(|m| m.content.as_str());
        match last_user {
            Some(text) if !text.trim().is_empty() => format!("[mock] {}", text),
            _ => MOCK_FALLBACK_CONTENT.to_string(),
        }
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn init(&self) -> Result<(), LlmProviderRegistryError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<LlmProviderRegistryHealth, LlmProviderRegistryError> {
        Ok(LlmProviderRegistryHealth {
            count: 0,
            backend: self.name.clone(),
            healthy: true,
        })
    }

    async fn chat_completion(
        &self,
        req: ChatRequest,
    ) -> Result<ChatResponse, LlmProviderRegistryError> {
        if let Err(e) = req.validate() {
            return Err(LlmProviderRegistryError::InvalidOperation(e.to_string()));
        }
        let content = Self::compose_reply(&req);
        let id = req.request_id.unwrap_or_else(Uuid::new_v4);
        Ok(ChatResponse {
            id,
            model: if req.model.is_empty() {
                MOCK_DEFAULT_MODEL.to_string()
            } else {
                req.model.clone()
            },
            message: ChatMessage::assistant(content),
            #[allow(deprecated)]
            finish_reason: "stop".to_string(),
            stop_reason: crate::events::StopReason::Stop,
            usage: crate::events::Usage::default(),
            created_at: Utc::now(),
        })
    }

    async fn stream_completion(
        &self,
        req: ChatRequest,
    ) -> Result<
        futures_util::stream::BoxStream<'static, Result<ChatChunk, LlmProviderRegistryError>>,
        LlmProviderRegistryError,
    > {
        if let Err(e) = req.validate() {
            return Err(LlmProviderRegistryError::InvalidOperation(e.to_string()));
        }
        let id = req.request_id.unwrap_or_else(Uuid::new_v4);
        let model = if req.model.is_empty() {
            MOCK_DEFAULT_MODEL.to_string()
        } else {
            req.model.clone()
        };
        let content = Self::compose_reply(&req);
        let s = stream::once(async move {
            Ok(ChatChunk {
                id,
                model,
                role: ChatRole::Assistant,
                delta: content,
                finish_reason: Some("stop".to_string()),
            })
        });
        Ok(s.boxed())
    }
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 — >= 1 unit test per module)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request(content: &str) -> ChatRequest {
        ChatRequest {
            model: MOCK_DEFAULT_MODEL.to_string(),
            messages: vec![ChatMessage::user(content)],
            temperature: None,
            max_tokens: None,
            request_id: Some(Uuid::new_v4()),
            thinking_level: None,
        }

    #[test]
    fn mock_provider_default_name_is_mock() {
        let p = MockProvider::new();
        assert_eq!(p.name, "mock");
    }

    #[test]
    fn mock_provider_with_name_stores_name() {
        let p = MockProvider::with_name("test-mock");
        assert_eq!(p.name, "test-mock");
    }

    #[tokio::test]
    async fn mock_provider_init_and_shutdown_are_noops() {
        let p = MockProvider::new();
        assert!(p.init().await.is_ok());
        assert!(p.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn mock_provider_health_check_reports_healthy() {
        let p = MockProvider::new();
        let h = p.health_check().await.unwrap();
        assert_eq!(h.backend, "mock");
        assert!(h.healthy);
    }

    #[tokio::test]
    async fn mock_provider_chat_completion_echoes_last_user_message() {
        let p = MockProvider::new();
        let req = sample_request("hello world");
        let resp = p.chat_completion(req).await.unwrap();
        assert_eq!(resp.model, MOCK_DEFAULT_MODEL);
        assert_eq!(resp.message.role, ChatRole::Assistant);
        assert_eq!(resp.message.content, "[mock] hello world");
        assert_eq!(resp.stop_reason, crate::events::StopReason::Stop);
        assert_ne!(resp.id, Uuid::nil());
    }

    #[tokio::test]
    async fn mock_provider_chat_completion_falls_back_when_no_user_message() {
        let p = MockProvider::new();
        let req = ChatRequest {
            model: MOCK_DEFAULT_MODEL.to_string(),
            messages: vec![ChatMessage::system("be terse")],
            temperature: None,
            max_tokens: None,
            request_id: None,
            thinking_level: None,
        };
        let resp = p.chat_completion(req).await.unwrap();
        assert_eq!(resp.message.content, MOCK_FALLBACK_CONTENT);
    }

    #[tokio::test]
    async fn mock_provider_chat_completion_rejects_invalid_request() {
        let p = MockProvider::new();
        let req = ChatRequest {
            model: "".to_string(),
            messages: vec![ChatMessage::user("hi")],
            temperature: None,
            max_tokens: None,
            request_id: None,
            thinking_level: None,
        };
        let err = p.chat_completion(req).await.unwrap_err();
        assert!(matches!(err, LlmProviderRegistryError::InvalidOperation(_)));
    }

    #[tokio::test]
    async fn mock_provider_stream_completion_emits_terminal_chunk() {
        use futures_util::StreamExt;
        let p = MockProvider::new();
        let req = sample_request("stream me");
        let mut s = p.stream_completion(req).await.unwrap();
        let first = s.next().await.expect("stream non-empty").unwrap();
        assert_eq!(first.delta, "[mock] stream me");
        assert_eq!(first.role, ChatRole::Assistant);
        assert_eq!(first.finish_reason.as_deref(), Some("stop"));
        // Stream ends after the terminal chunk.
        assert!(s.next().await.is_none());
    }
}

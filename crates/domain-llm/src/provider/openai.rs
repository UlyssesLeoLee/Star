// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider/openai.rs` — W2 (ULYS-98-W2) OpenAI-compatible provider.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.1":
//! - POST `https://api.openai.com/v1/chat/completions`
//! - API key from env `OPENAI_API_KEY` (or direct injection via [`OpenAiProvider::with_api_key`])
//! - `no_network_mode` 默认 true (守门 #25 v25); 生产切换需要 owner 拍板.
//!
//! 守门合规 (per 守门 #5 v2 + 守门 #7 + 守门 #11 + 守门 #25 v25):
//! - API key 不入 log / 不 println.
//! - 失败统一映射到 [`LlmProviderRegistryError::Backend`], 不 panic.
//! - 0 `unsafe` blocks.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use futures_util::stream::{self, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::chat::{ChatChunk, ChatMessage, ChatRequest, ChatResponse, ChatRole};
use crate::{
    LlmProvider, LlmProviderRegistryError, LlmProviderRegistryHealth,
};

/// Default OpenAI Chat Completions base URL.
pub const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com";

/// Default OpenAI model used when the request leaves `model` empty.
pub const OPENAI_DEFAULT_MODEL: &str = "gpt-4o-mini";

/// Default request timeout (HTTP).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// **OpenAiProvider** — OpenAI Chat Completions provider implementing [`LlmProvider`].
///
/// W2 (ULYS-98-W2) drops the v0.0.1 `chat_completion` / `stream_completion`
/// stub default impls with real reqwest calls. The provider still respects
/// `no_network_mode = true` by default (守门 #25 v25).
#[derive(Clone)]
pub struct OpenAiProvider {
    /// `Authorization: Bearer <api_key>` value. **Not logged**.
    api_key: Option<String>,
    /// Base URL (e.g. `https://api.openai.com` or self-hosted proxy).
    base_url: String,
    /// HTTP client (reused across calls when `no_network_mode = false`).
    client: Option<Client>,
    /// When `true`, build the request but never send (CI default).
    no_network_mode: bool,
}

/// Custom `Debug` impl that **redacts the API key** (守门 #5 v2).
impl std::fmt::Debug for OpenAiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiProvider")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("client", &self.client.as_ref().map(|_| "<reqwest::Client>"))
            .field("no_network_mode", &self.no_network_mode)
            .finish()
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAiProvider {
    /// Construct a provider in `no_network_mode = true` (守门 #25 v25 default).
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: OPENAI_DEFAULT_BASE_URL.to_string(),
            client: None,
            no_network_mode: true,
        }
    }

    /// Construct with an explicit API key (test / MVP injection).
    ///
    /// **守门 #5 v2**: caller is responsible for sourcing the key from a
    /// secret store; the value is **never** logged by this provider.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            base_url: OPENAI_DEFAULT_BASE_URL.to_string(),
            client: None,
            no_network_mode: true,
        }
    }

    /// Override the API base URL (OpenAI-compatible proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Toggle the network mode. Production requires `false`; CI uses
    /// `true` to avoid network access (守门 #25 v25).
    pub fn with_network(mut self, enabled: bool) -> Self {
        self.no_network_mode = !enabled;
        self
    }

    /// Read `OPENAI_API_KEY` from process env at call time (守门 #5 v2).
    fn api_key_from_env(&self) -> Result<String, LlmProviderRegistryError> {
        match std::env::var("OPENAI_API_KEY") {
            Ok(v) if !v.trim().is_empty() => Ok(v),
            _ => self.api_key.clone().ok_or_else(|| {
                LlmProviderRegistryError::Backend(
                    "openai: OPENAI_API_KEY not set".to_string(),
                )
            }),
        }
    }

    /// Lazily construct a reqwest client (shared across calls).
    fn http(&self) -> Result<Client, LlmProviderRegistryError> {
        if let Some(c) = &self.client {
            return Ok(c.clone());
        }
        Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| {
                LlmProviderRegistryError::Backend(format!(
                    "openai: reqwest client build failed: {e}"
                ))
            })
    }

    /// Map [`ChatRequest`] into the OpenAI Chat Completions wire format.
    fn build_request_body<'a>(&self, req: &'a ChatRequest) -> OpenAiRequest<'a> {
        OpenAiRequest {
            model: if req.model.is_empty() {
                OPENAI_DEFAULT_MODEL
            } else {
                &req.model
            },
            messages: req
                .messages
                .iter()
                .map(|m| OpenAiMessage {
                    role: m.role.as_str(),
                    content: &m.content,
                })
                .collect(),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
        }
    }

    /// Parse an OpenAI Chat Completions response into a [`ChatResponse`].
    fn parse_response(
        model: &str,
        request_id: Uuid,
        parsed: &OpenAiResponse,
    ) -> ChatResponse {
        let text = parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        ChatResponse {
            id: request_id,
            model: model.to_string(),
            message: ChatMessage::assistant(text),
            finish_reason: parsed
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone())
                .unwrap_or_else(|| "stop".to_string()),
            created_at: Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------
// OpenAI Chat Completions API wire types (per
// https://platform.openai.com/docs/api-reference/chat)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    #[serde(default)]
    content: String,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn init(&self) -> Result<(), LlmProviderRegistryError> {
        // Eagerly build the client so a misconfiguration fails fast.
        let _ = self.http()?;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<LlmProviderRegistryHealth, LlmProviderRegistryError> {
        Ok(LlmProviderRegistryHealth {
            count: 0,
            backend: format!("openai:{OPENAI_DEFAULT_BASE_URL}"),
            healthy: self.api_key.is_some() || std::env::var("OPENAI_API_KEY").is_ok(),
        })
    }

    async fn chat_completion(
        &self,
        req: ChatRequest,
    ) -> Result<ChatResponse, LlmProviderRegistryError> {
        if let Err(e) = req.validate() {
            return Err(LlmProviderRegistryError::InvalidOperation(e.to_string()));
        }

        let body = self.build_request_body(&req);
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        if self.no_network_mode {
            debug!(
                url = %url,
                "OpenAiProvider: no_network_mode 开启, 仅构造请求不发送 (守门 #25 v25)"
            );
            // Deterministic stub reply (CI fallback). 守门 #25 v25: no
            // API key needed in no_network_mode.
            let id = req.request_id.unwrap_or_else(Uuid::new_v4);
            return Ok(ChatResponse {
                id,
                model: if req.model.is_empty() {
                    OPENAI_DEFAULT_MODEL.to_string()
                } else {
                    req.model.clone()
                },
                message: ChatMessage::assistant(format!(
                    "[openai stub] received {} messages",
                    req.messages.len()
                )),
                finish_reason: "stop".to_string(),
                created_at: Utc::now(),
            });
        }

        let api_key = self.api_key_from_env()?;
        let client = self.http()?;
        let model = if req.model.is_empty() {
            OPENAI_DEFAULT_MODEL.to_string()
        } else {
            req.model.clone()
        };
        let wire = OpenAiRequest {
            model: &model,
            messages: body.messages,
            temperature: body.temperature,
            max_tokens: body.max_tokens,
        };

        // **守门 #5 v2**: Authorization header 仅在请求时构造, 立即用完丢弃.
        let response = client
            .post(&url)
            .bearer_auth(&api_key)
            .json(&wire)
            .send()
            .await
            .map_err(|e| {
                LlmProviderRegistryError::Backend(format!(
                    "openai: HTTP send failed: {e}"
                ))
            })?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            warn!(status = %status.as_u16(), "openai: auth failure");
            return Err(LlmProviderRegistryError::Backend(format!(
                "openai: auth failure (HTTP {})",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(LlmProviderRegistryError::Backend(format!(
                "openai: HTTP {}",
                status.as_u16()
            )));
        }

        let parsed: OpenAiResponse = response.json().await.map_err(|e| {
            LlmProviderRegistryError::Backend(format!(
                "openai: response parse failed: {e}"
            ))
        })?;
        let request_id = req.request_id.unwrap_or_else(Uuid::new_v4);
        Ok(Self::parse_response(&model, request_id, &parsed))
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

        if self.no_network_mode {
            let id = req.request_id.unwrap_or_else(Uuid::new_v4);
            let model = if req.model.is_empty() {
                OPENAI_DEFAULT_MODEL.to_string()
            } else {
                req.model.clone()
            };
            let s = stream::once(async move {
                Ok(ChatChunk {
                    id,
                    model,
                    role: ChatRole::Assistant,
                    delta: format!(
                        "[openai stub: stream] {} messages",
                        req.messages.len()
                    ),
                    finish_reason: Some("stop".to_string()),
                })
            });
            return Ok(s.boxed());
        }

        // True OpenAI SSE streaming deferred to W3 (per brief).
        let _api_key = self.api_key_from_env()?;
        let _client = self.http()?;
        warn!(
            "openai: stream_completion SSE wiring deferred to W3 \
            (per docs/briefs/ulys-98-star-cursor-min-v1.md §Sub-task 3.2)"
        );
        let id = req.request_id.unwrap_or_else(Uuid::new_v4);
        let model = if req.model.is_empty() {
            OPENAI_DEFAULT_MODEL.to_string()
        } else {
            req.model.clone()
        };
        let s = stream::once(async move {
            Ok(ChatChunk {
                id,
                model,
                role: ChatRole::Assistant,
                delta: "[openai: stream deferred to W3]".to_string(),
                finish_reason: Some("stop".to_string()),
            })
        });
        Ok(s.boxed())
    }
}

// =====================================================================
// Unit Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::ChatMessage;

    fn sample_request() -> ChatRequest {
        ChatRequest {
            model: OPENAI_DEFAULT_MODEL.to_string(),
            messages: vec![
                ChatMessage::system("be terse"),
                ChatMessage::user("hi"),
            ],
            temperature: Some(0.7),
            max_tokens: Some(128),
            request_id: Some(Uuid::new_v4()),
        }
    }

    #[test]
    fn openai_provider_default_is_no_network() {
        let p = OpenAiProvider::new();
        assert!(p.no_network_mode);
        assert_eq!(p.base_url, OPENAI_DEFAULT_BASE_URL);
    }

    #[test]
    fn openai_provider_with_api_key_stores_key_without_logging() {
        let p = OpenAiProvider::with_api_key("sk-openai-test");
        assert_eq!(p.api_key.as_deref(), Some("sk-openai-test"));
        let dbg = format!("{:?}", p);
        assert!(
            !dbg.contains("sk-openai-test"),
            "Debug output must not include API key (守门 #5 v2)"
        );
    }

    #[test]
    fn openai_provider_with_base_url_overrides() {
        let p = OpenAiProvider::new()
            .with_base_url("https://openai-compatible-proxy.example.com");
        assert_eq!(
            p.base_url,
            "https://openai-compatible-proxy.example.com"
        );
    }

    #[tokio::test]
    async fn openai_provider_init_and_shutdown_succeed() {
        let p = OpenAiProvider::new();
        assert!(p.init().await.is_ok());
        assert!(p.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn openai_provider_health_check_reports_backend() {
        let p = OpenAiProvider::with_api_key("sk-test");
        let h = p.health_check().await.unwrap();
        assert!(h.backend.starts_with("openai:"));
        assert!(h.healthy);
    }

    #[test]
    fn openai_provider_build_request_body_includes_all_messages() {
        let p = OpenAiProvider::new();
        let req = sample_request();
        let body = p.build_request_body(&req);
        assert_eq!(body.model, OPENAI_DEFAULT_MODEL);
        assert_eq!(body.messages.len(), 2);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[0].content, "be terse");
        assert_eq!(body.messages[1].role, "user");
        assert_eq!(body.temperature, Some(0.7));
        assert_eq!(body.max_tokens, Some(128));
    }

    #[tokio::test]
    async fn openai_provider_chat_completion_no_network_returns_stub() {
        let p = OpenAiProvider::new();
        let resp = p.chat_completion(sample_request()).await.unwrap();
        assert!(resp.message.content.contains("[openai stub]"));
        assert_eq!(resp.finish_reason, "stop");
        assert_ne!(resp.id, Uuid::nil());
    }

    #[tokio::test]
    async fn openai_provider_chat_completion_rejects_invalid_request() {
        let p = OpenAiProvider::new();
        let req = ChatRequest {
            model: "".to_string(),
            messages: vec![ChatMessage::user("hi")],
            temperature: None,
            max_tokens: None,
            request_id: None,
        };
        let err = p.chat_completion(req).await.unwrap_err();
        assert!(matches!(
            err,
            LlmProviderRegistryError::InvalidOperation(_)
        ));
    }

    #[tokio::test]
    async fn openai_provider_chat_completion_network_mode_without_key_fails() {
        let p = OpenAiProvider::new().with_network(true);
        let req = sample_request();
        let err = p.chat_completion(req).await.unwrap_err();
        assert!(matches!(err, LlmProviderRegistryError::Backend(_)));
    }

    #[tokio::test]
    async fn openai_provider_stream_completion_emits_terminal_chunk() {
        use futures_util::StreamExt;
        let p = OpenAiProvider::new();
        let mut s = p.stream_completion(sample_request()).await.unwrap();
        let first = s.next().await.expect("stream non-empty").unwrap();
        assert_eq!(first.role, ChatRole::Assistant);
        assert!(first.delta.contains("openai"));
        assert_eq!(first.finish_reason.as_deref(), Some("stop"));
        assert!(s.next().await.is_none());
    }
}
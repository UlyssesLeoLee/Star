// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider/anthropic.rs` — W2 (ULYS-98-W2) Anthropic Claude provider.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.1":
//! - POST `https://api.anthropic.com/v1/messages`
//! - API key from env `ANTHROPIC_API_KEY` (or direct injection via [`AnthropicProvider::with_api_key`])
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
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::chat::{ChatChunk, ChatMessage, ChatRequest, ChatResponse, ChatRole};
use crate::{LlmProvider, LlmProviderRegistryError, LlmProviderRegistryHealth};

/// Default Anthropic Messages API base URL.
pub const ANTHROPIC_DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Default Anthropic model used when the request leaves `model` empty.
pub const ANTHROPIC_DEFAULT_MODEL: &str = "claude-3-5-sonnet-20241022";

/// Default request timeout (HTTP).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// **AnthropicProvider** — Claude provider implementing [`LlmProvider`].
///
/// W2 (ULYS-98-W2) drops the v0.0.1 `chat_completion` / `stream_completion`
/// stub default impls with real reqwest calls. The provider still respects
/// `no_network_mode = true` by default (守门 #25 v25), so CI runs without
/// network and without an API key pass deterministically.
#[derive(Clone)]
pub struct AnthropicProvider {
    /// `x-api-key` header value. **Not logged**.
    api_key: Option<String>,
    /// Base URL (e.g. `https://api.anthropic.com`).
    base_url: String,
    /// HTTP client (reused across calls when `no_network_mode = false`).
    client: Option<Client>,
    /// When `true`, build the request but never send (CI default).
    no_network_mode: bool,
}

/// Custom `Debug` impl that **redacts the API key** (守门 #5 v2).
impl std::fmt::Debug for AnthropicProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnthropicProvider")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("client", &self.client.as_ref().map(|_| "<reqwest::Client>"))
            .field("no_network_mode", &self.no_network_mode)
            .finish()
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicProvider {
    /// Construct a provider in `no_network_mode = true` (守门 #25 v25 default).
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: ANTHROPIC_DEFAULT_BASE_URL.to_string(),
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
            base_url: ANTHROPIC_DEFAULT_BASE_URL.to_string(),
            client: None,
            no_network_mode: true,
        }
    }

    /// Override the API base URL (self-hosted proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Toggle the network mode. **Production requires `false`**; CI uses
    /// `true` to avoid network access (守门 #25 v25).
    pub fn with_network(mut self, enabled: bool) -> Self {
        self.no_network_mode = !enabled;
        self
    }

    /// Read `ANTHROPIC_API_KEY` from process env at call time (守门 #5 v2).
    fn api_key_from_env(&self) -> Result<String, LlmProviderRegistryError> {
        match std::env::var("ANTHROPIC_API_KEY") {
            Ok(v) if !v.trim().is_empty() => Ok(v),
            _ => self.api_key.clone().ok_or_else(|| {
                LlmProviderRegistryError::Backend(
                    "anthropic: ANTHROPIC_API_KEY not set".to_string(),
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
                    "anthropic: reqwest client build failed: {e}"
                ))
            })
    }

    /// Map Anthropic Messages API request from internal [`ChatRequest`].
    fn build_request_body(&self, req: &ChatRequest) -> Value {
        // Anthropic splits "system" messages from the user/assistant
        // array. Extract system messages separately (joined by '\n').
        let mut system_buf = String::new();
        let mut messages = Vec::with_capacity(req.messages.len());
        for m in &req.messages {
            match m.role {
                ChatRole::System => {
                    if !system_buf.is_empty() {
                        system_buf.push('\n');
                    }
                    system_buf.push_str(&m.content);
                }
                ChatRole::User | ChatRole::Assistant => {
                    messages.push(serde_json::json!({
                        "role": m.role.as_str(),
                        "content": m.content,
                    }));
                }
                ChatRole::Tool => {
                    // Tool role deferred — schema reserved but not emitted.
                    debug!("anthropic: Tool role reserved (not emitted) — W2 stub");
                }
            }
        }

        let mut body = serde_json::json!({
            "model": if req.model.is_empty() { ANTHROPIC_DEFAULT_MODEL } else { &req.model },
            "max_tokens": req.max_tokens.unwrap_or(1024),
            "messages": messages,
        });
        if !system_buf.is_empty() {
            body.as_object_mut()
                .unwrap()
                .insert("system".to_string(), serde_json::Value::String(system_buf));
        }
        if let Some(t) = req.temperature {
            body.as_object_mut()
                .unwrap()
                .insert("temperature".to_string(), serde_json::json!(t));
        }
        body
    }

    /// Parse an Anthropic Messages response into a [`ChatResponse`].
    fn parse_response(model: &str, request_id: Uuid, parsed: &AnthropicResponse) -> ChatResponse {
        let text = parsed
            .content
            .iter()
            .filter_map(|b| match b {
                AnthropicContentBlock::Text { text } => Some(text.as_str()),
                AnthropicContentBlock::Unknown => None,
            })
            .collect::<Vec<_>>()
            .join("");
        #[allow(deprecated)] // legacy wire compat (PI-2 / FR-9)
        let finish_reason = parsed
            .stop_reason
            .clone()
            .unwrap_or_else(|| "stop".to_string());
        ChatResponse {
            id: request_id,
            model: model.to_string(),
            message: ChatMessage::assistant(text),
            #[allow(deprecated)]
            finish_reason: finish_reason.clone(),
            stop_reason: crate::events::StopReason::parse_loose(&finish_reason),
            usage: crate::events::Usage::default(),
            created_at: Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------
// Anthropic Messages API wire types (per
// https://docs.anthropic.com/en/api/messages)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text {
        text: String,
    },
    #[serde(other)]
    Unknown,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
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
            backend: format!("anthropic:{ANTHROPIC_DEFAULT_BASE_URL}"),
            healthy: self.api_key.is_some() || std::env::var("ANTHROPIC_API_KEY").is_ok(),
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
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));

        if self.no_network_mode {
            debug!(
                url = %url,
                "AnthropicProvider: no_network_mode 开启, 仅构造请求不发送 (守门 #25 v25)"
            );
            // Deterministic stub reply (CI fallback) — uses request_id if
            // present so tests can match. 守门 #25 v25: no API key needed
            // in no_network_mode.
            let id = req.request_id.unwrap_or_else(Uuid::new_v4);
            return Ok(ChatResponse {
                id,
                model: if req.model.is_empty() {
                    ANTHROPIC_DEFAULT_MODEL.to_string()
                } else {
                    req.model.clone()
                },
                message: ChatMessage::assistant(format!(
                    "[anthropic stub] received {} messages",
                    req.messages.len()
                )),
                #[allow(deprecated)]
                finish_reason: "stop".to_string(),
                stop_reason: crate::events::StopReason::Stop,
                usage: crate::events::Usage::default(),
                created_at: Utc::now(),
            });
        }

        let api_key = self.api_key_from_env()?;
        let client = self.http()?;
        let model = if req.model.is_empty() {
            ANTHROPIC_DEFAULT_MODEL.to_string()
        } else {
            req.model.clone()
        };
        let wire = AnthropicRequest {
            model: &model,
            max_tokens: req.max_tokens.unwrap_or(1024),
            messages: body
                .get("messages")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
            system: body
                .get("system")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            temperature: req.temperature,
        };

        // **守门 #5 v2**: Authorization header 仅在请求时构造, 立即用完丢弃.
        let response = client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&wire)
            .send()
            .await
            .map_err(|e| {
                LlmProviderRegistryError::Backend(format!("anthropic: HTTP send failed: {e}"))
            })?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            // 守门 #5 v2: 不回显 response body, 只回 status
            warn!(status = %status.as_u16(), "anthropic: auth failure");
            return Err(LlmProviderRegistryError::Backend(format!(
                "anthropic: auth failure (HTTP {})",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(LlmProviderRegistryError::Backend(format!(
                "anthropic: HTTP {}",
                status.as_u16()
            )));
        }

        let parsed: AnthropicResponse = response.json().await.map_err(|e| {
            LlmProviderRegistryError::Backend(format!("anthropic: response parse failed: {e}"))
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

        // For W2, true Anthropic SSE streaming is **deferred** to a
        // future iteration: the upstream schema returns a stream of
        // `content_block_delta` events, which would require a dedicated
        // event parser. W2 ships a single-shot stub chunk so the API
        // contract holds; W3 (per brief) adds the full SSE plumbing.
        if self.no_network_mode {
            let id = req.request_id.unwrap_or_else(Uuid::new_v4);
            let model = if req.model.is_empty() {
                ANTHROPIC_DEFAULT_MODEL.to_string()
            } else {
                req.model.clone()
            };
            let s = stream::once(async move {
                Ok(ChatChunk {
                    id,
                    model,
                    role: ChatRole::Assistant,
                    delta: format!("[anthropic stub: stream] {} messages", req.messages.len()),
                    finish_reason: Some("stop".to_string()),
                })
            });
            return Ok(s.boxed());
        }

        // Validate API key + client eagerly; we do not yet wire the SSE
        // stream (deferred to W3 per brief).  Surface the key check so
        // misconfigured callers fail fast instead of silently returning
        // a stream that never yields.
        let _api_key = self.api_key_from_env()?;
        let _client = self.http()?;
        warn!(
            "anthropic: stream_completion SSE wiring deferred to W3 \
            (per docs/briefs/ulys-98-star-cursor-min-v1.md §Sub-task 3.2)"
        );
        let id = req.request_id.unwrap_or_else(Uuid::new_v4);
        let model = if req.model.is_empty() {
            ANTHROPIC_DEFAULT_MODEL.to_string()
        } else {
            req.model.clone()
        };
        let s = stream::once(async move {
            Ok(ChatChunk {
                id,
                model,
                role: ChatRole::Assistant,
                delta: "[anthropic: stream deferred to W3]".to_string(),
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
            model: ANTHROPIC_DEFAULT_MODEL.to_string(),
            messages: vec![ChatMessage::system("be terse"), ChatMessage::user("hi")],
            temperature: Some(0.5),
            max_tokens: Some(256),
            request_id: Some(Uuid::new_v4()),
            thinking_level: None,
        }
    }

    #[test]
    fn anthropic_provider_default_is_no_network() {
        let p = AnthropicProvider::new();
        assert!(p.no_network_mode);
        assert_eq!(p.base_url, ANTHROPIC_DEFAULT_BASE_URL);
    }

    #[test]
    fn anthropic_provider_with_api_key_stores_key_without_logging() {
        let p = AnthropicProvider::with_api_key("sk-ant-test-key");
        assert_eq!(p.api_key.as_deref(), Some("sk-ant-test-key"));
        assert!(p.no_network_mode);
        // Format string must not include the key (defence in depth).
        let dbg = format!("{:?}", p);
        assert!(
            !dbg.contains("sk-ant-test-key"),
            "Debug output must not include API key (守门 #5 v2)"
        );
    }

    #[test]
    fn anthropic_provider_with_base_url_overrides() {
        let p = AnthropicProvider::new().with_base_url("https://proxy.example.com");
        assert_eq!(p.base_url, "https://proxy.example.com");
    }

    #[test]
    fn anthropic_provider_with_network_toggles_mode() {
        let p = AnthropicProvider::new().with_network(true);
        assert!(!p.no_network_mode);
        let p = AnthropicProvider::new().with_network(false);
        assert!(p.no_network_mode);
    }

    #[tokio::test]
    async fn anthropic_provider_init_and_shutdown_succeed() {
        let p = AnthropicProvider::new();
        assert!(p.init().await.is_ok());
        assert!(p.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn anthropic_provider_health_check_reports_backend() {
        let p = AnthropicProvider::with_api_key("sk-test");
        let h = p.health_check().await.unwrap();
        assert!(h.backend.starts_with("anthropic:"));
        assert!(h.healthy);
    }

    #[test]
    fn anthropic_provider_build_request_body_splits_system_messages() {
        let p = AnthropicProvider::new();
        let req = sample_request();
        let body = p.build_request_body(&req);
        assert_eq!(body["model"], ANTHROPIC_DEFAULT_MODEL);
        assert_eq!(body["max_tokens"], 256);
        assert_eq!(body["system"], "be terse");
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1, "system message must not be in messages[]");
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "hi");
        assert_eq!(body["temperature"], 0.5);
    }

    #[tokio::test]
    async fn anthropic_provider_chat_completion_no_network_returns_stub() {
        let p = AnthropicProvider::new();
        let resp = p.chat_completion(sample_request()).await.unwrap();
        assert!(resp.message.content.contains("[anthropic stub]"));
        assert_eq!(resp.finish_reason, "stop");
        assert_ne!(resp.id, Uuid::nil());
    }

    #[tokio::test]
    async fn anthropic_provider_chat_completion_rejects_invalid_request() {
        let p = AnthropicProvider::new();
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
    async fn anthropic_provider_chat_completion_network_mode_without_key_fails() {
        // Disable network (still require key) — env var absent, no injected key.
        let p = AnthropicProvider::new().with_network(true);
        let req = sample_request();
        let err = p.chat_completion(req).await.unwrap_err();
        assert!(matches!(err, LlmProviderRegistryError::Backend(_)));
    }

    #[tokio::test]
    async fn anthropic_provider_stream_completion_emits_terminal_chunk() {
        use futures_util::StreamExt;
        let p = AnthropicProvider::new();
        let mut s = p.stream_completion(sample_request()).await.unwrap();
        let first = s.next().await.expect("stream non-empty").unwrap();
        assert_eq!(first.role, ChatRole::Assistant);
        assert!(first.delta.contains("anthropic"));
        assert_eq!(first.finish_reason.as_deref(), Some("stop"));
        assert!(s.next().await.is_none());
    }
}

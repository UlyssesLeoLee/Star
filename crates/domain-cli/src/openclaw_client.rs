//! Star OpenClaw HTTP API Client (B.1 实装, per 2026-08-30 07:25 JST wt-b1-openclaw-http)
//!
//! 职责:
//!   1. **OpenClawClient**: 同步 OpenAI 兼容 HTTP 客户端 (per openclaw.dev API spec)
//!   2. **GenerateRequest / GenerateResponse**: 请求/响应 schema (per OpenClaw API)
//!   3. **Mock mode**: 走 B.5 mock endpoint (per 29692a7 mock 备选路径解锁), 凭证到位后 1 commit 替换 base_url + key
//!
//! 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//!   1. 当前只支持 /v1/chat/completions 单 endpoint, 不支持 /v1/embeddings / /v1/models
//!   2. 流式响应 (SSE) 未实装, Phase 2 接
//!   3. 真实 OpenClaw endpoint 未接, 当前用 mock base_url (per B.5 mock 备选)
//!   4. 不接 KMS, API key 明文在 OpenClawClient struct
//!
//! 不做 (per 守门):
//!   - 不动 network 层 (per Phase D)
//!   - 不写 UI (per B.9 API 监控审计)
//!   - 不接 Hermes (per B.2 独立子项)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenClaw API 错误类型 (from B.7 quota::ApiError 兼容, 简化版)
#[derive(Debug, thiserror::Error)]
pub enum OpenClawError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid API key (empty)")]
    InvalidKey,
    #[error("API returned non-2xx: status={0} body={1}")]
    NonSuccess(u16, String),
    #[error("API response parse failed: {0}")]
    Parse(String),
}

/// OpenClaw 客户端配置
#[derive(Debug, Clone)]
pub struct OpenClawConfig {
    /// base URL (mock 模式用 wiremock / ms, 真实模式用 https://api.openclaw.dev/v1)
    pub base_url: String,
    /// API key (env: OPENCLAW_API_KEY)
    pub api_key: String,
    /// 请求 timeout
    pub timeout: Duration,
    /// mock 模式开关 (per B.5 mock 备选, 默认 true 直到真实凭证到位)
    pub mock_mode: bool,
}

impl OpenClawConfig {
    /// 新建默认配置 (mock 模式)
    pub fn new_mock() -> Self {
        Self {
            base_url: "http://localhost:8080/v1".into(),
            api_key: "mock-key".into(),
            timeout: Duration::from_secs(30),
            mock_mode: true,
        }
    }

    /// 新建真实模式配置
    pub fn new_real(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Result<Self, OpenClawError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(OpenClawError::InvalidKey);
        }
        Ok(Self {
            base_url: base_url.into(),
            api_key,
            timeout: Duration::from_secs(30),
            mock_mode: false,
        })
    }
}

/// /v1/chat/completions 请求 (OpenAI 兼容)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub content: String,
}

/// /v1/chat/completions 响应 (OpenAI 兼容)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenClaw HTTP 客户端
pub struct OpenClawClient {
    config: OpenClawConfig,
    http: reqwest::Client,
}

impl OpenClawClient {
    /// 新建客户端 (per config)
    pub fn new(config: OpenClawConfig) -> Result<Self, OpenClawError> {
        if config.api_key.is_empty() {
            return Err(OpenClawError::InvalidKey);
        }
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { config, http })
    }

    /// 当前 config (ref)
    pub fn config(&self) -> &OpenClawConfig {
        &self.config
    }

    /// /v1/chat/completions 单次生成
    ///
    /// mock 模式: 不发 HTTP, 直接返回 mock response (per B.5 mock 备选)
    /// 真实模式: 发 HTTP POST 到 base_url + /chat/completions
    pub async fn generate(&self, req: &GenerateRequest) -> Result<GenerateResponse, OpenClawError> {
        if self.config.mock_mode {
            return Ok(Self::mock_response(req));
        }

        let url = format!("{}/chat/completions", self.config.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OpenClawError::NonSuccess(status.as_u16(), body));
        }

        let parsed: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| OpenClawError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    /// Mock response (per B.5 mock 备选, 不发 HTTP)
    fn mock_response(req: &GenerateRequest) -> GenerateResponse {
        let last_user_msg = req
            .messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let prompt_tokens = req
            .messages
            .iter()
            .map(|m| m.content.split_whitespace().count() as u32)
            .sum();
        let completion_content = format!(
            "[mock-openclaw] model={} echo: {}",
            req.model, last_user_msg
        );
        let completion_tokens = completion_content.split_whitespace().count() as u32;
        GenerateResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            model: req.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".into(),
                    content: completion_content,
                },
                finish_reason: Some("stop".into()),
            }],
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_new_mock() {
        let cfg = OpenClawConfig::new_mock();
        assert!(cfg.mock_mode);
        assert!(!cfg.base_url.is_empty());
    }

    #[test]
    fn config_new_real_rejects_empty_key() {
        let result = OpenClawConfig::new_real("https://api.openclaw.dev/v1", "");
        assert!(matches!(result, Err(OpenClawError::InvalidKey)));
    }

    #[test]
    fn client_rejects_empty_key() {
        let cfg = OpenClawConfig {
            base_url: "x".into(),
            api_key: "".into(),
            timeout: Duration::from_secs(1),
            mock_mode: true,
        };
        let result = OpenClawClient::new(cfg);
        assert!(matches!(result, Err(OpenClawError::InvalidKey)));
    }

    #[test]
    fn mock_generate_response() {
        let cfg = OpenClawConfig::new_mock();
        let client = OpenClawClient::new(cfg).unwrap();
        let req = GenerateRequest {
            model: "gpt-4".into(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: "You are helpful.".into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: "Hello world".into(),
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(100),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let resp = rt.block_on(client.generate(&req)).unwrap();
        assert!(resp.id.starts_with("mock-"));
        assert_eq!(resp.model, "gpt-4");
        assert_eq!(resp.choices.len(), 1);
        assert!(resp.choices[0].message.content.contains("Hello world"));
        assert!(resp.usage.total_tokens > 0);
    }

    #[test]
    fn request_serialize_keeps_optional_fields_as_null() {
        let req = GenerateRequest {
            model: "gpt-4".into(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        // Option<T> with #[serde(default)] serializes None as `null` in JSON (per OpenAI API spec)
        assert!(json.contains("\"temperature\":null"));
        assert!(json.contains("\"max_tokens\":null"));
    }
}

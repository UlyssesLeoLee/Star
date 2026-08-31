//! Star Hermes HTTP API Client (B.6 实装, per 2026-08-30 07:28 JST wt-b6-hermes-mock)
//!
//! 职责:
//!   1. **HermesClient**: 同步 OpenAI 兼容 HTTP 客户端 (per hermes.dev API spec, 跟 B.1 OpenClaw 几乎相同)
//!   2. **GenerateRequest / GenerateResponse**: 复用 B.1 OpenClaw schema (OpenAI 兼容)
//!   3. **Mock mode**: 走 B.6 mock endpoint (per 29692a7 mock 备选路径解锁), 凭证到位后 1 commit 替换 base_url + key
//!
//! 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//!   1. 当前只支持 /v1/chat/completions 单 endpoint
//!   2. 流式响应 (SSE) 未实装
//!   3. 真实 Hermes endpoint 未接, 当前用 mock base_url
//!   4. 不接 KMS, API key 明文在 HermesClient struct
//!   5. 跟 B.1 OpenClaw 重复代码多, Phase 2 抽出 HttpClient trait 共享
//!
//! 不做 (per 守门):
//!   - 不动 network 层
//!   - 不写 UI (per B.9 API 监控审计)
//!   - 不接 OpenClaw (per B.1 独立子项)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Hermes API 错误类型
#[derive(Debug, thiserror::Error)]
pub enum HermesError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid API key (empty)")]
    InvalidKey,
    #[error("API returned non-2xx: status={0} body={1}")]
    NonSuccess(u16, String),
    #[error("API response parse failed: {0}")]
    Parse(String),
}

/// Hermes 客户端配置
#[derive(Debug, Clone)]
pub struct HermesConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,
    pub mock_mode: bool,
}

impl HermesConfig {
    pub fn new_mock() -> Self {
        Self {
            base_url: "http://localhost:8081/v1".into(),
            api_key: "mock-key".into(),
            timeout: Duration::from_secs(30),
            mock_mode: true,
        }
    }

    pub fn new_real(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Result<Self, HermesError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(HermesError::InvalidKey);
        }
        Ok(Self {
            base_url: base_url.into(),
            api_key,
            timeout: Duration::from_secs(30),
            mock_mode: false,
        })
    }
}

/// /v1/chat/completions 请求 (OpenAI 兼容, 跟 B.1 OpenClaw 同 schema)
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
    pub role: String,
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

/// Hermes HTTP 客户端
pub struct HermesClient {
    config: HermesConfig,
    http: reqwest::Client,
}

impl HermesClient {
    pub fn new(config: HermesConfig) -> Result<Self, HermesError> {
        if config.api_key.is_empty() {
            return Err(HermesError::InvalidKey);
        }
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { config, http })
    }

    pub fn config(&self) -> &HermesConfig {
        &self.config
    }

    /// /v1/chat/completions 单次生成
    ///
    /// mock 模式: 不发 HTTP, 直接返回 mock response
    /// 真实模式: 发 HTTP POST 到 base_url + /chat/completions
    pub async fn generate(&self, req: &GenerateRequest) -> Result<GenerateResponse, HermesError> {
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
            return Err(HermesError::NonSuccess(status.as_u16(), body));
        }

        let parsed: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| HermesError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    /// Mock response (per B.6 mock 备选, 不发 HTTP, 跟 B.1 OpenClaw 同模式)
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
        let completion_content =
            format!("[mock-hermes] model={} echo: {}", req.model, last_user_msg);
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
        let cfg = HermesConfig::new_mock();
        assert!(cfg.mock_mode);
        assert!(!cfg.base_url.is_empty());
        assert_eq!(cfg.base_url, "http://localhost:8081/v1");
    }

    #[test]
    fn config_new_real_rejects_empty_key() {
        let result = HermesConfig::new_real("https://api.hermes.dev/v1", "");
        assert!(matches!(result, Err(HermesError::InvalidKey)));
    }

    #[test]
    fn client_rejects_empty_key() {
        let cfg = HermesConfig {
            base_url: "x".into(),
            api_key: "".into(),
            timeout: Duration::from_secs(1),
            mock_mode: true,
        };
        let result = HermesClient::new(cfg);
        assert!(matches!(result, Err(HermesError::InvalidKey)));
    }

    #[test]
    fn mock_generate_response_uses_hermes_marker() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        let req = GenerateRequest {
            model: "hermes-2".into(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: "You are helpful.".into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: "Hello Hermes".into(),
                },
            ],
            temperature: None,
            max_tokens: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let resp = rt.block_on(client.generate(&req)).unwrap();
        assert!(resp.id.starts_with("mock-"));
        assert_eq!(resp.model, "hermes-2");
        assert_eq!(resp.choices.len(), 1);
        // 跟 B.1 OpenClaw 区分: 标记 mock-hermes
        assert!(resp.choices[0].message.content.contains("[mock-hermes]"));
        assert!(resp.choices[0].message.content.contains("Hello Hermes"));
    }

    #[test]
    fn request_serialize_keeps_optional_fields_as_null() {
        let req = GenerateRequest {
            model: "hermes-2".into(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"temperature\":null"));
        assert!(json.contains("\"max_tokens\":null"));
    }
}

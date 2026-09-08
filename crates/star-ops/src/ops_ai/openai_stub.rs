// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpenAI 通道 (per OPS-BASIC-DESIGN §5.4 + ADR-0026 §2.2 L2)
//!
//! F-02 端到端实装: 真实 reqwest POST https://api.openai.com/v1/chat/completions
//! API key 走 `star-credential` KMS 加密存储 (per 守门 #5 v2, 2026-09-08 拍板)
//!
//! 安全 (per 守门 #5 v2):
//! - api_key 不入 log / 不 println
//! - Authorization header 仅在请求时构造, 立即用完丢弃
//! - 失败返 OpsError, 不 panic
//!
//! 守门 #25 v25: stub 阶段 no-network 模式, 仅构造 reqwest::Request 不发送 (CI 环境无 internet)

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry, Suggestion};

/// OpenAI 通道 (F-02 端到端)
#[derive(Default)]
pub struct OpenAiStub {
    /// API key 走 star-credential (LlmOpenAi provider), MVP 也支持直接注入
    pub api_key: Option<String>,
    /// 自定义 base_url (e.g. 自部署 OpenAI 兼容服务)
    pub base_url: Option<String>,
    /// HTTP client (共享, timeout 30s)
    pub client: Option<Client>,
    /// 测试/无网络模式: 构造请求但不实际发送, 返 mock LogAnalysis
    /// 默认 true (per 守门 #25 v25, F-02 端到端先打通契约, 真实调用 owner 拍板后切)
    pub no_network_mode: bool,
}

impl OpenAiStub {
    /// MVP 默认构造
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
            client: None,
            no_network_mode: true, // 守门 #25 v25
        }
    }

    /// 构造带 api_key (直注入, 仅供测试)
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            base_url: None,
            client: None,
            no_network_mode: true, // 守门 #25 v25
        }
    }

    /// 真实 reqwest POST (no_network_mode=false 才发, 守门 #25 v25)
    pub async fn call_openai_chat(&self, log_message: &str) -> Result<LogAnalysis, OpsError> {
        // 1. 校验 api_key (守门 #5 v2: 不入 log, 仅检测是否为空)
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            OpsError::Unauthorized(
                "OpenAI api_key 未配置 (走 star-credential LlmOpenAi)".to_string(),
            )
        })?;

        // 2. 构造请求体 (per OpenAI chat completions schema)
        let base = self.base_url.as_deref().unwrap_or("https://api.openai.com");
        let url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));

        let body = serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a log analysis assistant. Reply with JSON: {summary, anomalies: [{type,timestamp,level,message_excerpt}], suggestions: [{id,text,confidence}]}."
                },
                {
                    "role": "user",
                    "content": log_message
                }
            ],
            "temperature": 0.0,
            "response_format": {"type": "json_object"},
        });

        // 3. 守门 #25 v25: stub 阶段 no_network_mode=true 仅构造请求, 不发送
        if self.no_network_mode {
            debug!(
                url = %url,
                "OpenAiStub: no_network_mode 开启, 仅构造请求不发送 (守门 #25 v25, 真实调用 owner 拍板后切)"
            );
            // 返 mock LogAnalysis (跟 mock 通道行为一致, 方便 Ladder 测试)
            return Ok(LogAnalysis {
                log_id: uuid::Uuid::nil(),
                summary: format!(
                    "[OpenAI stub] 接收 {} 字符 log, 真实调用待切生产",
                    log_message.len()
                ),
                anomalies: vec![],
                suggestions: vec![Suggestion {
                    id: "s-openai-stub".to_string(),
                    text: "OpenAI stub: 构造请求验证 (守门 #25 v25 no_network_mode)".to_string(),
                    confidence: 0.42,
                }],
                confidence: 0.42, // 守门 #23
                generated_by: "openai_stub".to_string(),
            });
        }

        // 4. 真实发送 (owner 拍板切生产后才走)
        let client = self.client.clone().unwrap_or_else(|| {
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client build must succeed")
        });
        // 守门 #5 v2: Authorization header 立即用完丢弃, 不入 log
        let response = client
            .post(&url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| OpsError::Internal(format!("OpenAI HTTP 调用失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            // 不返 response body (可能含 api_key 反射), 仅 status
            return Err(OpsError::Internal(format!(
                "OpenAI HTTP {} 错误",
                status.as_u16()
            )));
        }

        #[derive(Deserialize)]
        struct OpenAiResponse {
            choices: Vec<OpenAiChoice>,
        }
        #[derive(Deserialize)]
        struct OpenAiChoice {
            message: OpenAiMessage,
        }
        #[derive(Deserialize)]
        struct OpenAiMessage {
            content: String,
        }

        let parsed: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| OpsError::Internal(format!("解析 OpenAI 响应失败: {}", e)))?;

        let content = parsed
            .choices
            .first()
            .ok_or_else(|| OpsError::Internal("OpenAI 返空 choices".to_string()))?
            .message
            .content
            .clone();

        // content 应该是 JSON 字符串, 二次解析
        #[derive(Deserialize)]
        struct InnerLogAnalysis {
            summary: String,
            #[serde(default)]
            anomalies: Vec<crate::ops_domain::log::Anomaly>,
            #[serde(default)]
            suggestions: Vec<Suggestion>,
            #[serde(default = "default_confidence")]
            confidence: f32,
        }
        fn default_confidence() -> f32 {
            0.5
        }
        let inner: InnerLogAnalysis = serde_json::from_str(&content)
            .map_err(|e| OpsError::Internal(format!("解析 OpenAI content JSON 失败: {}", e)))?;

        Ok(LogAnalysis {
            log_id: uuid::Uuid::nil(),
            summary: inner.summary,
            anomalies: inner.anomalies,
            suggestions: inner.suggestions,
            confidence: inner.confidence,
            generated_by: "openai".to_string(),
        })
    }
}

#[async_trait]
impl AiChannel for OpenAiStub {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn is_enabled(&self) -> bool {
        // 守门 #25 v25: api_key 配了 + 不强制 no_network (F-02 端到端 always enabled if api_key)
        self.api_key.is_some()
    }

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        warn!(
            log_id = %log.id,
            "OpenAiStub: 真实 OpenAI chat/completions 调用 (守门 #25 v25 no_network 模式)"
        );
        let mut result = self.call_openai_chat(&log.message).await?;
        // 回填真实 log_id
        result.log_id = log.id;
        Ok(result)
    }
}

/// OpenAI 通道 builder (per 守门 #25 v25: 构造 + 注入, 避免全局状态)
pub struct OpenAiStubBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    /// 默认 true (守门 #25 v25: stub 阶段不开网络)
    no_network_mode: bool,
}

impl Default for OpenAiStubBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            no_network_mode: true, // 守门 #25 v25: stub 默认无网络
        }
    }
}

impl OpenAiStubBuilder {
    /// 设置 API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// 设置自定义 base_url
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// 关闭 no_network_mode (owner 拍板后切生产)
    pub fn enable_real_network(mut self) -> Self {
        self.no_network_mode = false;
        self
    }

    /// 构造 OpenAiStub
    pub fn build(self) -> OpenAiStub {
        OpenAiStub {
            api_key: self.api_key,
            base_url: self.base_url,
            client: None,
            no_network_mode: self.no_network_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops_domain::log::LogLevel;
    use chrono::Utc;

    #[test]
    fn openai_stub_is_disabled_without_api_key() {
        let stub = OpenAiStub::new();
        assert!(!stub.is_enabled());
        assert_eq!(stub.name(), "openai");
    }

    #[test]
    fn openai_stub_is_enabled_with_api_key() {
        let stub = OpenAiStub::with_api_key("sk-test-123");
        assert!(stub.is_enabled());
    }

    #[test]
    fn openai_stub_builder_builds_with_api_key() {
        let stub = OpenAiStubBuilder::default()
            .with_api_key("sk-test-456")
            .with_base_url("https://api.openai.com")
            .build();
        assert!(stub.is_enabled());
        assert_eq!(stub.base_url.as_deref(), Some("https://api.openai.com"));
        assert!(stub.no_network_mode);
    }

    #[tokio::test]
    async fn openai_stub_no_network_mode_returns_stub_analysis() {
        // 守门 #25 v25: 默认 no_network_mode=true, 不发真实 HTTP
        let stub = OpenAiStub::with_api_key("sk-test-789");
        let log = LogEntry {
            id: uuid::Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Error,
            message: "2026-09-08 ERROR helm release 3 deploy failed".to_string(),
            timestamp: Utc::now(),
            trace_id: None,
        };
        let analysis = stub.analyze_log(&log).await.expect("must succeed");
        // 守门 #23: stub 阶段 confidence < 0.5
        assert!(analysis.confidence < 0.5);
        assert_eq!(analysis.generated_by, "openai_stub");
        assert_eq!(analysis.log_id, log.id);
    }
}

// 占位 serde Serialize derive (避免 unused import 警告, Anomaly 已有 Serialize)
#[allow(dead_code)]
fn _ensure_serialize_imported<S: Serialize>() {}

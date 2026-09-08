// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anthropic 通道 (per OPS-BASIC-DESIGN §5.4 + ADR-0026 §2.2 L3)
//!
//! F-02 端到端实装: 真实 reqwest POST https://api.anthropic.com/v1/messages
//! API key 走 `star-credential` KMS 加密存储 (per 守门 #5 v2, 2026-09-08 拍板)
//!
//! 安全 (per 守门 #5 v2):
//! - api_key 不入 log / 不 println
//! - x-api-key header 仅在请求时构造, 立即用完丢弃
//! - 失败返 OpsError, 不 panic
//!
//! 守门 #25 v25: stub 阶段 no-network 模式, 仅构造 reqwest::Request 不发送 (CI 环境无 internet)

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry, Suggestion};

/// Anthropic 通道 (F-02 端到端)
#[derive(Default)]
pub struct AnthropicStub {
    /// API key 走 star-credential (LlmAnthropic provider), MVP 也支持直接注入
    pub api_key: Option<String>,
    /// 自定义 base_url (e.g. 自部署 Anthropic 兼容服务)
    pub base_url: Option<String>,
    /// HTTP client (共享, timeout 30s)
    pub client: Option<Client>,
    /// 测试/无网络模式: 构造请求但不实际发送, 返 mock LogAnalysis
    /// 默认 true (per 守门 #25 v25, F-02 端到端先打通契约, 真实调用 owner 拍板后切)
    pub no_network_mode: bool,
}

impl AnthropicStub {
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
    pub async fn call_anthropic_messages(
        &self,
        log_message: &str,
    ) -> Result<LogAnalysis, OpsError> {
        // 1. 校验 api_key (守门 #5 v2: 不入 log, 仅检测是否为空)
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            OpsError::Unauthorized(
                "Anthropic api_key 未配置 (走 star-credential LlmAnthropic)".to_string(),
            )
        })?;

        // 2. 构造请求体 (per Anthropic messages schema)
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com");
        let url = format!("{}/v1/messages", base.trim_end_matches('/'));

        let body = serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "system": "You are a log analysis assistant. Reply with JSON: {summary, anomalies: [{type,timestamp,level,message_excerpt}], suggestions: [{id,text,confidence}]}.",
            "messages": [
                {
                    "role": "user",
                    "content": log_message
                }
            ]
        });

        // 3. 守门 #25 v25: stub 阶段 no_network_mode=true 仅构造请求, 不发送
        if self.no_network_mode {
            debug!(
                url = %url,
                "AnthropicStub: no_network_mode 开启, 仅构造请求不发送 (守门 #25 v25, 真实调用 owner 拍板后切)"
            );
            return Ok(LogAnalysis {
                log_id: uuid::Uuid::nil(),
                summary: format!(
                    "[Anthropic stub] 接收 {} 字符 log, 真实调用待切生产",
                    log_message.len()
                ),
                anomalies: vec![],
                suggestions: vec![Suggestion {
                    id: "s-anthropic-stub".to_string(),
                    text: "Anthropic stub: 构造请求验证 (守门 #25 v25 no_network_mode)".to_string(),
                    confidence: 0.42,
                }],
                confidence: 0.42, // 守门 #23
                generated_by: "anthropic_stub".to_string(),
            });
        }

        // 4. 真实发送 (owner 拍板切生产后才走)
        let client = self.client.clone().unwrap_or_else(|| {
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client build must succeed")
        });
        // 守门 #5 v2: x-api-key header 立即用完丢弃, 不入 log
        // Anthropic 还需要 anthropic-version header
        let response = client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpsError::Internal(format!("Anthropic HTTP 调用失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(OpsError::Internal(format!(
                "Anthropic HTTP {} 错误",
                status.as_u16()
            )));
        }

        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<AnthropicContent>,
        }
        #[derive(Deserialize)]
        struct AnthropicContent {
            #[serde(rename = "type")]
            kind: String,
            #[serde(default)]
            text: Option<String>,
        }

        let parsed: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| OpsError::Internal(format!("解析 Anthropic 响应失败: {}", e)))?;

        let content_text = parsed
            .content
            .iter()
            .find(|c| c.kind == "text")
            .and_then(|c| c.text.clone())
            .ok_or_else(|| OpsError::Internal("Anthropic 返空 text content".to_string()))?;

        // content_text 应该是 JSON 字符串, 二次解析
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
        let inner: InnerLogAnalysis = serde_json::from_str(&content_text)
            .map_err(|e| OpsError::Internal(format!("解析 Anthropic content JSON 失败: {}", e)))?;

        Ok(LogAnalysis {
            log_id: uuid::Uuid::nil(),
            summary: inner.summary,
            anomalies: inner.anomalies,
            suggestions: inner.suggestions,
            confidence: inner.confidence,
            generated_by: "anthropic".to_string(),
        })
    }
}

#[async_trait]
impl AiChannel for AnthropicStub {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn is_enabled(&self) -> bool {
        // 守门 #25 v25: api_key 配了 + 不强制 no_network (F-02 端到端 always enabled if api_key)
        self.api_key.is_some()
    }

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        warn!(
            log_id = %log.id,
            "AnthropicStub: 真实 Anthropic messages 调用 (守门 #25 v25 no_network 模式)"
        );
        let mut result = self.call_anthropic_messages(&log.message).await?;
        result.log_id = log.id;
        Ok(result)
    }
}

/// Anthropic 通道 builder (per 守门 #25 v25: 构造 + 注入, 避免全局状态)
pub struct AnthropicStubBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    /// 默认 true (守门 #25 v25: stub 阶段不开网络)
    no_network_mode: bool,
}

impl Default for AnthropicStubBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            no_network_mode: true, // 守门 #25 v25: stub 默认无网络
        }
    }
}

impl AnthropicStubBuilder {
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

    /// 构造 AnthropicStub
    pub fn build(self) -> AnthropicStub {
        AnthropicStub {
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
    fn anthropic_stub_is_disabled_without_api_key() {
        let stub = AnthropicStub::new();
        assert!(!stub.is_enabled());
        assert_eq!(stub.name(), "anthropic");
    }

    #[test]
    fn anthropic_stub_is_enabled_with_api_key() {
        let stub = AnthropicStub::with_api_key("sk-ant-test-123");
        assert!(stub.is_enabled());
    }

    #[test]
    fn anthropic_stub_builder_builds_with_api_key() {
        let stub = AnthropicStubBuilder::default()
            .with_api_key("sk-ant-test-456")
            .with_base_url("https://api.anthropic.com")
            .build();
        assert!(stub.is_enabled());
        assert_eq!(stub.base_url.as_deref(), Some("https://api.anthropic.com"));
        assert!(stub.no_network_mode);
    }

    #[tokio::test]
    async fn anthropic_stub_no_network_mode_returns_stub_analysis() {
        // 守门 #25 v25: 默认 no_network_mode=true, 不发真实 HTTP
        let stub = AnthropicStub::with_api_key("sk-ant-test-789");
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
        assert_eq!(analysis.generated_by, "anthropic_stub");
        assert_eq!(analysis.log_id, log.id);
    }
}

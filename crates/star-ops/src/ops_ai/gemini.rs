// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gemini 通道 (per OPS-BASIC-DESIGN §5.4 + ADR-0026 §2.2 L2)
//!
//! v0.36 端到端实装: 真实 reqwest POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
//! API key 走 `star-credential` KMS 加密存储 (per 守门 #5 v2, 2026-09-08 拍板)
//!
//! 守门 #25 v25: stub 阶段 no-network 模式, 仅构造 reqwest::Request 不发送 (CI 环境无 internet)

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry, Suggestion};

/// Gemini 通道 (v0.36 端到端, Gemini 优先 per 9/9 17:09 JST 用户拍板)
#[derive(Default)]
pub struct GeminiChannel {
    /// API key 走 star-credential (LlmGemini provider)
    pub api_key: Option<String>,
    /// 自定义 base_url (e.g. 自部署 Gemini 兼容服务)
    pub base_url: Option<String>,
    /// HTTP client (共享, timeout 30s)
    pub client: Option<Client>,
    /// 测试/无网络模式: 构造请求但不实际发送
    pub no_network_mode: bool,
}

impl GeminiChannel {
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
            no_network_mode: true,
        }
    }

    /// 真实 reqwest POST (no_network_mode=false 才发, 守门 #25 v25)
    pub async fn call_gemini_generate(&self, log_message: &str) -> Result<LogAnalysis, OpsError> {
        // 1. 校验 api_key (守门 #5 v2: 不入 log, 仅检测是否为空)
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            OpsError::Unauthorized(
                "Gemini api_key 未配置 (走 star-credential LlmGemini)".to_string(),
            )
        })?;

        // 2. 构造 URL (Gemini generateContent 端点)
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com");
        let url = format!(
            "{}/v1beta/models/gemini-1.5-flash:generateContent",
            base.trim_end_matches('/')
        );

        // 3. 构造请求体 (per Gemini generateContent schema)
        let body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{
                    "text": format!(
                        "You are a log analysis assistant. Reply with JSON: {{summary, anomalies: [{{type,timestamp,level,message_excerpt}}], suggestions: [{{id,text,confidence}}]}}.\n\nLog:\n{}",
                        log_message
                    )
                }]
            }],
            "generationConfig": {
                "temperature": 0.0,
                "responseMimeType": "application/json"
            }
        });

        // 4. 守门 #25 v25: stub 阶段 no_network_mode=true 仅构造请求, 不发送
        if self.no_network_mode {
            debug!(
                url = %url,
                "GeminiChannel: no_network_mode 开启, 仅构造请求不发送 (守门 #25 v25)"
            );
            return Ok(LogAnalysis {
                log_id: Uuid::nil(),
                summary: format!(
                    "[Gemini stub] 接收 {} 字符 log, 真实调用待切生产",
                    log_message.len()
                ),
                anomalies: vec![],
                suggestions: vec![Suggestion {
                    id: "s-gemini-stub".to_string(),
                    text: "Gemini stub: 构造请求验证 (守门 #25 v25 no_network_mode)".to_string(),
                    confidence: 0.42,
                }],
                confidence: 0.42, // 守门 #23
                generated_by: "gemini_stub".to_string(),
            });
        }

        // 5. 真实发送 (owner 拍板切生产后才走)
        let client = self.client.clone().unwrap_or_else(|| {
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client build must succeed")
        });
        // 守门 #5 v2: API key 走 query param, 立即用完丢弃, 不入 log
        let response = client
            .post(&url)
            .query(&[("key", api_key.as_str())])
            .json(&body)
            .send()
            .await
            .map_err(|e| OpsError::Internal(format!("Gemini HTTP 调用失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(OpsError::Internal(format!(
                "Gemini HTTP {} 错误",
                status.as_u16()
            )));
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<GeminiCandidate>,
        }
        #[derive(Deserialize)]
        struct GeminiCandidate {
            content: GeminiContent,
        }
        #[derive(Deserialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }
        #[derive(Deserialize)]
        struct GeminiPart {
            #[serde(default)]
            text: Option<String>,
        }

        let parsed: GeminiResponse = response
            .json()
            .await
            .map_err(|e| OpsError::Internal(format!("解析 Gemini 响应失败: {}", e)))?;

        let content_text = parsed
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.clone())
            .ok_or_else(|| OpsError::Internal("Gemini 返空 content".to_string()))?;

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
            .map_err(|e| OpsError::Internal(format!("解析 Gemini content JSON 失败: {}", e)))?;

        Ok(LogAnalysis {
            log_id: Uuid::nil(),
            summary: inner.summary,
            anomalies: inner.anomalies,
            suggestions: inner.suggestions,
            confidence: inner.confidence,
            generated_by: "gemini".to_string(),
        })
    }
}

#[async_trait]
impl AiChannel for GeminiChannel {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        warn!(
            log_id = %log.id,
            "GeminiChannel: 真实 Gemini generateContent 调用 (守门 #25 v25 no_network 模式)"
        );
        let mut result = self.call_gemini_generate(&log.message).await?;
        result.log_id = log.id;
        Ok(result)
    }
}

/// Gemini 通道 builder
pub struct GeminiChannelBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    no_network_mode: bool,
}

impl Default for GeminiChannelBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            no_network_mode: true,
        }
    }
}

impl GeminiChannelBuilder {
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn enable_real_network(mut self) -> Self {
        self.no_network_mode = false;
        self
    }

    pub fn build(self) -> GeminiChannel {
        GeminiChannel {
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
    fn gemini_channel_is_disabled_without_api_key() {
        let ch = GeminiChannel::new();
        assert!(!ch.is_enabled());
        assert_eq!(ch.name(), "gemini");
    }

    #[test]
    fn gemini_channel_is_enabled_with_api_key() {
        let ch = GeminiChannel::with_api_key("gemini-test-123");
        assert!(ch.is_enabled());
    }

    #[test]
    fn gemini_channel_builder_builds_with_api_key() {
        let ch = GeminiChannelBuilder::default()
            .with_api_key("gemini-test-456")
            .with_base_url("https://generativelanguage.googleapis.com")
            .build();
        assert!(ch.is_enabled());
        assert_eq!(
            ch.base_url.as_deref(),
            Some("https://generativelanguage.googleapis.com")
        );
        assert!(ch.no_network_mode);
    }

    #[tokio::test]
    async fn gemini_channel_no_network_mode_returns_stub_analysis() {
        let ch = GeminiChannel::with_api_key("gemini-test-789");
        let log = LogEntry {
            id: Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Error,
            message: "2026-09-09 ERROR gemini test failure".to_string(),
            timestamp: Utc::now(),
            trace_id: None,
        };
        let analysis = ch.analyze_log(&log).await.expect("must succeed");
        assert!(analysis.confidence < 0.5, "守门 #23: stub confidence < 0.5");
        assert_eq!(analysis.generated_by, "gemini_stub");
        assert_eq!(analysis.log_id, log.id);
    }

    /// 派生 #22: Gemini 缺 api_key 返 Unauthorized (per DDS-001 §2.2)
    #[tokio::test]
    async fn gemini_channel_returns_401_without_api_key() {
        let ch = GeminiChannel::new();
        let result = ch.call_gemini_generate("test log").await;
        assert!(result.is_err(), "缺 api_key 必返 Err");
        let err = result.expect_err("Err");
        assert!(
            matches!(err, OpsError::Unauthorized(_)),
            "缺 api_key 必返 Unauthorized (401 派生规)"
        );
    }
}

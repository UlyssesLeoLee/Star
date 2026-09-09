// SPDX-License-Identifier: MIT OR Apache-2.0
//! MiniMax 通道 (per OPS-BASIC-DESIGN §5.4 + ADR-0026 §2.2 L4)
//!
//! v0.36 端到端实装: 真实 reqwest POST https://api.minimax.chat/v1/chat/completions
//! (OpenAI 兼容 schema, MiniMax API 设计跟 OpenAI 兼容)
//! API key 走 `star-credential` KMS 加密存储 (per 守门 #5 v2)
//!
//! 守门 #25 v25: stub 阶段 no-network 模式, 仅构造 reqwest::Request 不发送
//!
//! 跨 session 续: 真实 API endpoint / auth 模式 owner 拍板后校准
//! (per 9/9 17:09 JST 用户发令"minimax也接", MVP 假定 OpenAI 兼容 + api.minimax.chat)

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry, Suggestion};

/// MiniMax 通道 (v0.36 端到端, per 9/9 17:09 JST 用户拍板"minimax也接")
#[derive(Default)]
pub struct MiniMaxChannel {
    /// API key 走 star-credential (LlmMiniMax provider)
    pub api_key: Option<String>,
    /// 自定义 base_url (e.g. 自部署 MiniMax 兼容服务)
    pub base_url: Option<String>,
    /// HTTP client (共享, timeout 30s)
    pub client: Option<Client>,
    /// 测试/无网络模式: 构造请求但不实际发送
    pub no_network_mode: bool,
}

impl MiniMaxChannel {
    /// MVP 默认构造
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
            client: None,
            no_network_mode: true,
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

    /// 真实 reqwest POST (no_network_mode=false 才发)
    pub async fn call_minimax_chat(&self, log_message: &str) -> Result<LogAnalysis, OpsError> {
        // 1. 校验 api_key
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            OpsError::Unauthorized(
                "MiniMax api_key 未配置 (走 star-credential LlmMiniMax)".to_string(),
            )
        })?;

        // 2. 构造 URL (MiniMax OpenAI 兼容 chat completions)
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("https://api.minimax.chat");
        let url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));

        // 3. 构造请求体 (OpenAI 兼容 schema)
        let body = serde_json::json!({
            "model": "minimax-default",
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
            "response_format": {"type": "json_object"}
        });

        // 4. 守门 #25 v25: stub 阶段 no_network_mode=true
        if self.no_network_mode {
            debug!(
                url = %url,
                "MiniMaxChannel: no_network_mode 开启, 仅构造请求不发送 (守门 #25 v25)"
            );
            return Ok(LogAnalysis {
                log_id: Uuid::nil(),
                summary: format!(
                    "[MiniMax stub] 接收 {} 字符 log, 真实调用待切生产",
                    log_message.len()
                ),
                anomalies: vec![],
                suggestions: vec![Suggestion {
                    id: "s-minimax-stub".to_string(),
                    text: "MiniMax stub: 构造请求验证 (守门 #25 v25 no_network_mode)".to_string(),
                    confidence: 0.42,
                }],
                confidence: 0.42,
                generated_by: "minimax_stub".to_string(),
            });
        }

        // 5. 真实发送
        let client = self.client.clone().unwrap_or_else(|| {
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client build must succeed")
        });
        // 守门 #5 v2: Authorization header 立即用完丢弃
        let response = client
            .post(&url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| OpsError::Internal(format!("MiniMax HTTP 调用失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(OpsError::Internal(format!(
                "MiniMax HTTP {} 错误",
                status.as_u16()
            )));
        }

        #[derive(Deserialize)]
        struct MiniMaxResponse {
            choices: Vec<MiniMaxChoice>,
        }
        #[derive(Deserialize)]
        struct MiniMaxChoice {
            message: MiniMaxMessage,
        }
        #[derive(Deserialize)]
        struct MiniMaxMessage {
            content: String,
        }

        let parsed: MiniMaxResponse = response
            .json()
            .await
            .map_err(|e| OpsError::Internal(format!("解析 MiniMax 响应失败: {}", e)))?;

        let content = parsed
            .choices
            .first()
            .ok_or_else(|| OpsError::Internal("MiniMax 返空 choices".to_string()))?
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
            .map_err(|e| OpsError::Internal(format!("解析 MiniMax content JSON 失败: {}", e)))?;

        Ok(LogAnalysis {
            log_id: Uuid::nil(),
            summary: inner.summary,
            anomalies: inner.anomalies,
            suggestions: inner.suggestions,
            confidence: inner.confidence,
            generated_by: "minimax".to_string(),
        })
    }
}

#[async_trait]
impl AiChannel for MiniMaxChannel {
    fn name(&self) -> &'static str {
        "minimax"
    }

    fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        warn!(
            log_id = %log.id,
            "MiniMaxChannel: 真实 MiniMax chat/completions 调用 (守门 #25 v25 no_network 模式)"
        );
        let mut result = self.call_minimax_chat(&log.message).await?;
        result.log_id = log.id;
        Ok(result)
    }
}

/// MiniMax 通道 builder
pub struct MiniMaxChannelBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    no_network_mode: bool,
}

impl Default for MiniMaxChannelBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            no_network_mode: true,
        }
    }
}

impl MiniMaxChannelBuilder {
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

    pub fn build(self) -> MiniMaxChannel {
        MiniMaxChannel {
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
    fn minimax_channel_is_disabled_without_api_key() {
        let ch = MiniMaxChannel::new();
        assert!(!ch.is_enabled());
        assert_eq!(ch.name(), "minimax");
    }

    #[test]
    fn minimax_channel_is_enabled_with_api_key() {
        let ch = MiniMaxChannel::with_api_key("minimax-test-123");
        assert!(ch.is_enabled());
    }

    #[test]
    fn minimax_channel_builder_builds_with_api_key() {
        let ch = MiniMaxChannelBuilder::default()
            .with_api_key("minimax-test-456")
            .with_base_url("https://api.minimax.chat")
            .build();
        assert!(ch.is_enabled());
        assert_eq!(ch.base_url.as_deref(), Some("https://api.minimax.chat"));
        assert!(ch.no_network_mode);
    }

    #[tokio::test]
    async fn minimax_channel_no_network_mode_returns_stub_analysis() {
        let ch = MiniMaxChannel::with_api_key("minimax-test-789");
        let log = LogEntry {
            id: Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Error,
            message: "2026-09-09 ERROR minimax test failure".to_string(),
            timestamp: Utc::now(),
            trace_id: None,
        };
        let analysis = ch.analyze_log(&log).await.expect("must succeed");
        assert!(analysis.confidence < 0.5);
        assert_eq!(analysis.generated_by, "minimax_stub");
        assert_eq!(analysis.log_id, log.id);
    }

    /// 派生 #23: MiniMax 缺 api_key 返 Unauthorized
    #[tokio::test]
    async fn minimax_channel_returns_401_without_api_key() {
        let ch = MiniMaxChannel::new();
        let result = ch.call_minimax_chat("test log").await;
        assert!(result.is_err());
        let err = result.expect_err("Err");
        assert!(matches!(err, OpsError::Unauthorized(_)));
    }
}

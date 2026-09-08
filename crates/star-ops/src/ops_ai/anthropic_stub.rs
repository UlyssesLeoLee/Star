// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anthropic 通道 stub (per OPS-BASIC-DESIGN §5.4 + ADR-0026 §2.2 L3)

use async_trait::async_trait;
use tracing::warn;

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry};

/// Anthropic 通道 stub
#[derive(Default)]
pub struct AnthropicStub {
    /// MVP 永远 None, 实装阶段从 star-credential 读取
    pub api_key: Option<String>,
}

#[async_trait]
impl AiChannel for AnthropicStub {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    async fn analyze_log(&self, _log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        warn!("AnthropicStub: 真实 API 调用 [M] 子项实装, 当前返 NOT_IMPLEMENTED");
        Err(OpsError::not_implemented("Anthropic 真实 API 调用"))
    }
}

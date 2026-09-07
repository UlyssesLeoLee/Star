// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpenAI 通道 stub (per OPS-BASIC-DESIGN §5.4 + ADR-0026 §2.2 L2)
//!
//! MVP-骨架: 仅暴露 trait impl, 真实 API 调用 `// TODO`
//! API key 走 star-credential crate 加密存储 (守门 #5 v2, MVP 不启用)

use async_trait::async_trait;
use tracing::warn;

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry};

/// OpenAI 通道 stub
#[derive(Default)]
pub struct OpenAiStub {
    /// MVP 永远 None, 实装阶段从 star-credential 读取
    pub api_key: Option<String>,
}

#[async_trait]
impl AiChannel for OpenAiStub {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    async fn analyze_log(&self, _log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        warn!("OpenAiStub: 真实 API 调用 [M] 子项实装, 当前返 NOT_IMPLEMENTED");
        Err(OpsError::not_implemented("OpenAI 真实 API 调用"))
    }
}

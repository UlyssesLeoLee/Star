// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` AI 通道 (Hybrid per ADR-0026 §2.2)
//!
//! per `docs/requirements/SRS-STAR-OPS-001.md` §5
//! per `docs/basic-design/OPS-BASIC-DESIGN-001.md` §5
//!
//! MVP-骨架:
//! - `mock` 通道: subprocess 调 `scripts/automation/ai_log_mock.py` (守门 #23 + 守门 #24 v2)
//! - `openai_stub`: trait impl, 真实调用 `// TODO`, 返 NOT_IMPLEMENTED
//! - `anthropic_stub`: 同上
//! - `ladder`: 4 级回退 (per ADR-0026 §2.2)

pub mod anthropic_stub;
pub mod ladder;
pub mod mock;
pub mod openai_stub;

use async_trait::async_trait;

use crate::error::OpsError;
use crate::ops_domain::log::{LogAnalysis, LogEntry};

/// AI 通道 trait (per OPS-BASIC-DESIGN §5.1)
#[async_trait]
pub trait AiChannel: Send + Sync {
    /// 通道名 (用于 meta.generated_by)
    fn name(&self) -> &'static str;

    /// 是否启用 (false 时被 ladder 跳过)
    fn is_enabled(&self) -> bool;

    /// 分析 log, 返回 LogAnalysis
    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError>;
}

/// 构建默认 Ladder (mock 永远首位, 其他通道按配置启用)
pub fn default_ladder() -> ladder::Ladder {
    ladder::Ladder::new(vec![
        Box::new(mock::MockChannel),
        Box::new(openai_stub::OpenAiStub::default()),
        Box::new(anthropic_stub::AnthropicStub::default()),
    ])
}

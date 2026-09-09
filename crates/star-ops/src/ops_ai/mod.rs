// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` AI 通道 (Hybrid per ADR-0026 §2.2)
//!
//! per `docs/requirements/SRS-STAR-OPS-001.md` §5
//! per `docs/basic-design/OPS-BASIC-DESIGN-001.md` §5
//!
//! v0.36 Hybrid 4 通道 + mock 兜底 (per 9/9 17:09 JST 用户拍板"gemini优先 其他也都接 minimax也接"):
//! - `gemini` (优先): Gemini 1.5 Flash (per 9/9 17:09 JST 拍板)
//! - `openai_stub`: OpenAI gpt-4o-mini
//! - `anthropic_stub`: Anthropic claude-3-5-sonnet
//! - `minimax`: MiniMax (per 9/9 17:09 JST 拍板"minimax也接", OpenAI 兼容)
//! - `mock` (兜底): subprocess 调 `scripts/automation/ai_log_mock.py` (守门 #23 + 守门 #24 v2)
//! - `ladder`: 5 级 Fallback (per ADR-0026 §2.2 + Gemini 优先)

pub mod anthropic_stub;
pub mod gemini;
pub mod ladder;
pub mod minimax;
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

/// 构造默认 Ladder (per ADR-0026 §2.2 + 9/9 17:09 JST 用户拍板"gemini优先 其他也都接 minimax也接")
/// 优先级顺序: Gemini > OpenAI > Anthropic > MiniMax > mock (兜底, 永远可用)
/// mock 永远末位, 保证 Fallback Ladder 总是有兜底
pub fn default_ladder() -> ladder::Ladder {
    ladder::Ladder::new(vec![
        Box::new(gemini::GeminiChannel::default()),
        Box::new(openai_stub::OpenAiStub::default()),
        Box::new(anthropic_stub::AnthropicStub::default()),
        Box::new(minimax::MiniMaxChannel::default()),
        Box::new(mock::MockChannel),
    ])
}

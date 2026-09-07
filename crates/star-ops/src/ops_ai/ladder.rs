// SPDX-License-Identifier: MIT OR Apache-2.0
//! AI 通道 Fallback Ladder (per ADR-0026 §2.2 + OPS-BASIC-DESIGN §5.2)
//!
//! L1: mock (默认) → L2: OpenAI → L3: Anthropic → L4: mock (兜底, 永远可用)
//!
//! 守门 #23: confidence < 0.5 必标 "需人工 review" (在 analyze_log 内统一处理)

use tracing::{info, warn};

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{LogAnalysis, LogEntry};

/// Ladder 持有按优先级排序的通道列表
pub struct Ladder {
    channels: Vec<Box<dyn AiChannel>>,
}

impl Ladder {
    /// 构造 Ladder, 通道按 L1→L2→L3→L4 顺序
    pub fn new(channels: Vec<Box<dyn AiChannel>>) -> Self {
        Self { channels }
    }

    /// 走 Ladder 分析 log, 第一个成功通道返回结果
    pub async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        let mut last_err: Option<OpsError> = None;

        for ch in &self.channels {
            if !ch.is_enabled() {
                info!(channel = ch.name(), "通道未启用, 跳过");
                continue;
            }

            info!(channel = ch.name(), "尝试 AI 通道");
            match ch.analyze_log(log).await {
                Ok(analysis) => {
                    // 守门 #23: confidence < 0.5 必标 "需人工 review"
                    if analysis.confidence < 0.5 {
                        warn!(
                            channel = ch.name(),
                            confidence = analysis.confidence,
                            "AI 分析 confidence < 0.5, 提示用户手动 review (守门 #23)"
                        );
                    }
                    return Ok(analysis);
                }
                Err(e) => {
                    warn!(channel = ch.name(), err = %e, "AI 通道失败, 回退");
                    if matches!(e, OpsError::Internal(_)) {
                        // retriable error, 继续下一个通道
                        last_err = Some(e);
                        continue;
                    } else {
                        // 非 retriable (如 NOT_IMPLEMENTED), 直接返
                        return Err(e);
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(OpsError::no_channel_available))
    }
}

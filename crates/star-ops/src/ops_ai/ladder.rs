// SPDX-License-Identifier: MIT OR Apache-2.0
//! AI 通道 Fallback Ladder (per ADR-0026 §2.2 + OPS-BASIC-DESIGN §5.2)
//!
//! L1: mock (默认) → L2: OpenAI → L3: Anthropic → L4: mock (兜底, 永远可用)
//!
//! 守门 #23: confidence < 0.5 必标 "需人工 review" (在 analyze_log 内统一处理)
//! 守门 #6 v2: retriable 规则 — Internal + RateLimited 都 retriable, 走下一通道
//!            (per OPS-DETAILED §6.2 ladder retriable 派生)

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

    /// 判断错误是否 retriable (per 守门 #6 v2)
    /// - Internal: retriable (per error.rs 6-field, retriable=true)
    /// - RateLimited: retriable (per error.rs 6-field, retriable=true; 60 req/min 等)
    /// - NotImplemented / Unauthorized / BadRequest: 非 retriable
    pub fn is_retriable(err: &OpsError) -> bool {
        matches!(err, OpsError::Internal(_) | OpsError::RateLimited(_))
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
                    warn!(channel = ch.name(), err = %e, retriable = Self::is_retriable(&e), "AI 通道失败, 回退");
                    if Self::is_retriable(&e) {
                        // 守门 #6 v2: Internal + RateLimited 都 retriable, 继续下一通道
                        last_err = Some(e);
                        continue;
                    } else {
                        // 非 retriable (NotImplemented / Unauthorized / BadRequest), 直接返
                        return Err(e);
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(OpsError::no_channel_available))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_retriable_internal() {
        let err = OpsError::Internal("network".to_string());
        assert!(Ladder::is_retriable(&err));
    }

    #[test]
    fn is_retriable_rate_limited() {
        // 守门 #6 v2: RateLimited 改 retriable (派生)
        let err = OpsError::RateLimited("60 req/min exceeded".to_string());
        assert!(
            Ladder::is_retriable(&err),
            "RateLimited 必 retriable (per 守门 #6 v2)"
        );
    }

    #[test]
    fn is_not_retriable_not_implemented() {
        let err = OpsError::NotImplemented("test".to_string());
        assert!(!Ladder::is_retriable(&err));
    }

    #[test]
    fn is_not_retriable_unauthorized() {
        let err = OpsError::Unauthorized("test".to_string());
        assert!(!Ladder::is_retriable(&err));
    }

    #[test]
    fn is_not_retriable_bad_request() {
        let err = OpsError::BadRequest("test".to_string());
        assert!(!Ladder::is_retriable(&err));
    }
}

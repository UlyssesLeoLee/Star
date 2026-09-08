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

    // ============ UT-IT-51 §2.3 Phase 5 ops_ai 派生缺口 (per brief §2.1) ============

    /// 派生 #17: Ladder 3 次重试边界 (per ADR-0026 §2.2 Ladder 派生规)
    /// 守门 #6 v2: Internal + RateLimited 都 retriable
    /// MVP 阶段: Ladder 走完全部 channels (无显式 retry count 限制)
    /// 派生测: 验证 3 个 retriable 错误连续时, Ladder 走完全部 channels 后返最后错误
    #[tokio::test]
    async fn ladder_retries_3_times_then_gives_up() {
        use crate::ops_ai::AiChannel;
        use crate::ops_domain::log::{LogAnalysis, LogEntry, LogLevel};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        // 构造 3 个永远返 Internal (retriable) 的 mock 通道
        struct RetriableChannel {
            name: &'static str,
            call_count: Arc<AtomicUsize>,
        }
        #[async_trait::async_trait]
        impl AiChannel for RetriableChannel {
            fn name(&self) -> &'static str {
                self.name
            }
            fn is_enabled(&self) -> bool {
                true
            }
            async fn analyze_log(&self, _log: &LogEntry) -> Result<LogAnalysis, OpsError> {
                self.call_count.fetch_add(1, Ordering::SeqCst);
                Err(OpsError::Internal(format!("{} 失败", self.name)))
            }
        }

        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));
        let counter3 = Arc::new(AtomicUsize::new(0));
        let ladder = Ladder::new(vec![
            Box::new(RetriableChannel {
                name: "channel1",
                call_count: counter1.clone(),
            }),
            Box::new(RetriableChannel {
                name: "channel2",
                call_count: counter2.clone(),
            }),
            Box::new(RetriableChannel {
                name: "channel3",
                call_count: counter3.clone(),
            }),
        ]);

        let log = LogEntry {
            id: uuid::Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Error,
            message: "test failure".to_string(),
            timestamp: chrono::Utc::now(),
            trace_id: None,
        };

        let result = ladder.analyze_log(&log).await;
        // 派生规: 3 个 retriable 错误 → Ladder 走完 → 返最后一个 Internal 错误
        assert!(result.is_err(), "3 个 retriable 错误必走完全部 channels");
        // 派生文档: 守門 #11 缺标比错标 — [M] 阶段加显式 retry count 限制 (e.g. 3 次)
        // MVP 阶段 Ladder 走完全部 channels, 不限制重试次数
        assert_eq!(counter1.load(Ordering::SeqCst), 1, "channel1 调 1 次");
        assert_eq!(counter2.load(Ordering::SeqCst), 1, "channel2 调 1 次");
        assert_eq!(counter3.load(Ordering::SeqCst), 1, "channel3 调 1 次");
    }

    /// 派生 #18: Ladder 跳过 disabled 通道 (per ADR-0026 §2.2 Ladder 派生规)
    /// MVP 阶段: Ladder.analyze_log 跳 is_enabled()=false 的通道
    /// 派生测: 验证 Ladder 跳过 disabled 通道, 走 enabled 通道
    #[tokio::test]
    async fn ladder_skips_disabled_channels() {
        use crate::ops_ai::AiChannel;
        use crate::ops_domain::log::{LogAnalysis, LogEntry, LogLevel};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct ToggleableChannel {
            name: &'static str,
            enabled: bool,
            call_count: Arc<AtomicUsize>,
        }
        #[async_trait::async_trait]
        impl AiChannel for ToggleableChannel {
            fn name(&self) -> &'static str {
                self.name
            }
            fn is_enabled(&self) -> bool {
                self.enabled
            }
            async fn analyze_log(&self, _log: &LogEntry) -> Result<LogAnalysis, OpsError> {
                self.call_count.fetch_add(1, Ordering::SeqCst);
                Ok(LogAnalysis {
                    log_id: uuid::Uuid::nil(),
                    summary: format!("{} 成功", self.name),
                    anomalies: vec![],
                    suggestions: vec![],
                    confidence: 0.42,
                    generated_by: self.name.to_string(),
                })
            }
        }

        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));
        let ladder = Ladder::new(vec![
            Box::new(ToggleableChannel {
                name: "disabled",
                enabled: false,
                call_count: counter1.clone(),
            }),
            Box::new(ToggleableChannel {
                name: "enabled",
                enabled: true,
                call_count: counter2.clone(),
            }),
        ]);

        let log = LogEntry {
            id: uuid::Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test".to_string(),
            timestamp: chrono::Utc::now(),
            trace_id: None,
        };

        let result = ladder.analyze_log(&log).await.expect("Ladder 必返 Ok");
        // 验证: 跳过 disabled 通道, 走 enabled 通道
        assert_eq!(counter1.load(Ordering::SeqCst), 0, "disabled 通道必不被调");
        assert_eq!(counter2.load(Ordering::SeqCst), 1, "enabled 通道必被调");
        assert_eq!(result.generated_by, "enabled", "必返 enabled 通道结果");
    }
}

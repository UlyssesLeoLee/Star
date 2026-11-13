/*
@cypher schema=1 source_sha256=7dabd393f6ea4032f8c029104599531a172c5e2a2d0717f3ce277a1e930ab55a
MERGE (self:File {path:"crates/star-agent-runtime/src/backpressure.rs"})
@endcypher
*/
//! Backpressure + Queue Overflow (per SRS §46-§47 + 03-DD §6.3)。
//!
//! 队列必须 bounded (per SRS §47); 溢出策略必须可配置:
//! Reject / Delay / DropLow / Persist / Throttle。禁止 OOM (per SRS §47)。

use std::fmt;
use std::time::Duration;

/// 溢出策略 (per 03-DD §6.3, per SRS §47 可配置)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// 拒绝入队 (QueueFull)。
    Reject,
    /// 延迟重试 (delay_ms)。
    Delay {
        /// 延迟毫秒。
        ms: u64,
    },
    /// 丢最低优先级。
    DropLow,
    /// 移入 DLQ。
    Persist,
    /// 429 retry_after (Fail Safe 默认, per SRS §101)。
    Throttle {
        /// 重试间隔秒。
        retry_after_secs: u64,
    },
}

impl Default for OverflowPolicy {
    fn default() -> Self {
        OverflowPolicy::Throttle {
            retry_after_secs: 60,
        }
    }
}

/// Backpressure 错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureError {
    /// 队列满 + Reject (per 03-DD §6.3 Error::QueueFull)。
    QueueFull {
        /// 当前大小。
        current: usize,
        /// 上限。
        max: usize,
    },
    /// 队列满 + Throttle (per 03-DD §6.3 Throttle { retry_after })。
    Throttled {
        /// 重试间隔。
        retry_after: Duration,
    },
    /// max_size 必须 >= 1 (禁止无界队列)。
    ZeroMaxSize,
}

impl fmt::Display for BackpressureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackpressureError::QueueFull { current, max } => {
                write!(f, "queue full ({current}/{max})")
            }
            BackpressureError::Throttled { retry_after } => {
                write!(f, "Throttle: retry_after={retry_after:?}")
            }
            BackpressureError::ZeroMaxSize => write!(f, "max_size 必须 >= 1 (禁止无界队列)"),
        }
    }
}

impl std::error::Error for BackpressureError {}

/// await_slot 判定 (per 03-DD §6.3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureDecision {
    /// 入队。
    Proceed,
    /// 延迟后重试。
    Delay(Duration),
    /// 丢最低优先级 count 个后入队。
    DropLow(usize),
    /// 移入 DLQ 后入队。
    Persist,
}

/// Bounded Queue + Overflow Policy (per 03-DD §6.3)。
#[derive(Debug, Clone)]
pub struct Backpressure {
    max_size: usize,
    policy: OverflowPolicy,
}

impl Backpressure {
    /// 新建 (默认 Throttle 60s, per SRS §101 Fail Safe)。
    pub fn new(max_size: usize) -> Result<Self, BackpressureError> {
        Self::with_policy(max_size, OverflowPolicy::default())
    }

    /// 显式策略。
    pub fn with_policy(max_size: usize, policy: OverflowPolicy) -> Result<Self, BackpressureError> {
        if max_size == 0 {
            return Err(BackpressureError::ZeroMaxSize);
        }
        Ok(Self { max_size, policy })
    }

    /// 队列上限。
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// 未满 → true (per 03-DD §6.3 await_slot 快路径)。
    pub fn has_slot(&self, current_size: usize) -> bool {
        current_size < self.max_size
    }

    /// 判定 (per 03-DD §6.3 await_slot)。
    pub fn await_slot(
        &self,
        current_size: usize,
    ) -> Result<BackpressureDecision, BackpressureError> {
        if self.has_slot(current_size) {
            return Ok(BackpressureDecision::Proceed);
        }
        match self.policy {
            OverflowPolicy::Reject => Err(BackpressureError::QueueFull {
                current: current_size,
                max: self.max_size,
            }),
            OverflowPolicy::Delay { ms } => {
                Ok(BackpressureDecision::Delay(Duration::from_millis(ms)))
            }
            OverflowPolicy::DropLow => Ok(BackpressureDecision::DropLow(1)),
            OverflowPolicy::Persist => Ok(BackpressureDecision::Persist),
            OverflowPolicy::Throttle { retry_after_secs } => Err(BackpressureError::Throttled {
                retry_after: Duration::from_secs(retry_after_secs),
            }),
        }
    }
}

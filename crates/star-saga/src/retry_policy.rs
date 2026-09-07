//! crates/star-saga/src/retry_policy.rs
//!
//! Retry policy for cross-domain Saga calls (per Phase D.2, Mavis 临时代签 match 域 Lead per 守门 #3 v2)
//! per `OPT-NEXT-01-phase-d.md` §3 D.2
//!
//! ## 职责
//!
//! 跨域调用失败重试策略: at-least-once + 3 retry + 指数退避
//! - at-least-once: 调用失败立即重试, 最多 3 次
//! - exponential backoff: 100ms / 200ms / 400ms (per ADR-0029 universal-submit)
//! - 跟 `compensation_strategy.rs` `CompensationMode::AtLeastOnce` 模式对齐 (per INV-CS-03)
//!
//! ## 关键不变量
//!
//! - INV-RP-01: 重试次数 1-5, 超过返回 `RetryPolicyError::RetryExhausted` (per 守门 #11 缺标比错标)
//! - INV-RP-02: 退避时间 50-1000ms, 默认 100ms base, 2x 指数 (per ADR-0029)
//! - INV-RP-03: 重试跟 `IdempotencyStore` 配合, 同 idempotency_key 多次调用 dedup (per INV-IDS-01)
//! - INV-RP-04: match 域 Lead 拍板 (Mavis 临时代签 per 守门 #3 v2 反转 9/3 11:35 JST):
//!   - at-least-once 模式 (per `compensation_strategy.rs` INV-CS-03)
//!   - 3 次重试上限
//!   - 100ms base + 2x 指数退避
//!
//! Lead 责任: match 域 Lead (Mavis 临时代签中, per 守门 #14 + 守门 #3 v2)

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RetryPolicyError {
    #[error("retry exhausted after {0} attempts")]
    RetryExhausted(u32),
    #[error("invalid retry policy: {0}")]
    Invalid(String),
}

/// Retry policy (per 守门 #14 match 域 Lead 拍板: Mavis 临时代签, per 守门 #3 v2 反转)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数 (含首次执行, 1 = 不重试, 5 = 最多 4 次重试)
    pub max_retries: u32,
    /// 首次重试延迟 (ms)
    pub initial_backoff_ms: u64,
    /// 退避乘数 (per INV-RP-02 默认 2)
    pub backoff_multiplier: u32,
    /// 退避上限 (ms, 防 race condition 失控)
    pub max_backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        // INV-RP-04: match 域 Lead 拍板 (Mavis 临时代签) - 3 次重试 + 100ms base + 2x 指数
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            backoff_multiplier: 2,
            max_backoff_ms: 1000,
        }
    }
}

impl RetryPolicy {
    /// 构造自定义 retry policy
    ///
    /// # Errors
    /// - `max_retries` 必须 >= 1
    /// - `initial_backoff_ms` 必须 50-1000ms
    /// - `backoff_multiplier` 必须 >= 1
    pub fn new(
        max_retries: u32,
        initial_backoff_ms: u64,
        backoff_multiplier: u32,
        max_backoff_ms: u64,
    ) -> Result<Self, RetryPolicyError> {
        // INV-RP-01
        if max_retries < 1 || max_retries > 5 {
            return Err(RetryPolicyError::Invalid(format!(
                "max_retries must be 1-5, got {}",
                max_retries
            )));
        }
        // INV-RP-02
        if initial_backoff_ms < 50 || initial_backoff_ms > 1000 {
            return Err(RetryPolicyError::Invalid(format!(
                "initial_backoff_ms must be 50-1000, got {}",
                initial_backoff_ms
            )));
        }
        if backoff_multiplier < 1 {
            return Err(RetryPolicyError::Invalid(format!(
                "backoff_multiplier must be >= 1, got {}",
                backoff_multiplier
            )));
        }
        if max_backoff_ms < initial_backoff_ms {
            return Err(RetryPolicyError::Invalid(format!(
                "max_backoff_ms ({}) must be >= initial_backoff_ms ({})",
                max_backoff_ms, initial_backoff_ms
            )));
        }
        Ok(Self {
            max_retries,
            initial_backoff_ms,
            backoff_multiplier,
            max_backoff_ms,
        })
    }

    /// 计算第 N 次重试的退避时间 (0-indexed, 0 = 第一次重试)
    /// 公式: `min(initial * multiplier^n, max_backoff)`
    pub fn backoff_for(&self, retry_index: u32) -> Duration {
        let base = self.initial_backoff_ms as u128;
        let mult = self.backoff_multiplier as u128;
        let backoff = base.saturating_mul(mult.saturating_pow(retry_index));
        let capped = backoff.min(self.max_backoff_ms as u128);
        Duration::from_millis(capped as u64)
    }

    /// 异步 sleep 当前 retry_index 对应的退避时间
    pub async fn sleep_backoff(&self, retry_index: u32) {
        tokio::time::sleep(self.backoff_for(retry_index)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_3_retries_100ms_2x() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_retries, 3);
        assert_eq!(p.initial_backoff_ms, 100);
        assert_eq!(p.backoff_multiplier, 2);
        assert_eq!(p.max_backoff_ms, 1000);
    }

    #[test]
    fn backoff_grows_exponentially() {
        let p = RetryPolicy::default();
        assert_eq!(p.backoff_for(0), Duration::from_millis(100));
        assert_eq!(p.backoff_for(1), Duration::from_millis(200));
        assert_eq!(p.backoff_for(2), Duration::from_millis(400));
    }

    #[test]
    fn backoff_capped_at_max() {
        let p = RetryPolicy::default();
        // 100 * 2^10 = 102400ms > 1000ms cap
        assert_eq!(p.backoff_for(10), Duration::from_millis(1000));
    }

    #[test]
    fn custom_policy_validated() {
        assert!(RetryPolicy::new(3, 100, 2, 1000).is_ok());
        // max_retries invalid
        assert!(RetryPolicy::new(0, 100, 2, 1000).is_err());
        assert!(RetryPolicy::new(6, 100, 2, 1000).is_err());
        // initial_backoff invalid
        assert!(RetryPolicy::new(3, 49, 2, 1000).is_err());
        assert!(RetryPolicy::new(3, 1001, 2, 1000).is_err());
        // backoff_multiplier invalid
        assert!(RetryPolicy::new(3, 100, 0, 1000).is_err());
        // max < initial
        assert!(RetryPolicy::new(3, 500, 2, 100).is_err());
    }
}

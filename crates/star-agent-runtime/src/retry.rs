//! 有限重试 + 指数退避 (per SRS §51 REQ-RETRY-001 + 03-DD §7)。
//!
//! 错误分类 (per 03-DD §7.1): Retryable → max_retry + backoff (1s, 2s, 4s);
//! Non-Retryable / Fatal → 立即失败。禁止无限 retry (per SRS §51)。
//! 幂等性 (per SRS §52): 重试带 operation_id / correlation_id, 走 idempotency key。

use std::fmt;
use std::time::Duration;

/// 重试耗尽 (per 03-DD §7.2 RetryError::Exhausted)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryError<E> {
    /// 已尝试次数。
    pub attempts: u32,
    /// 最后一次错误。
    pub last_error: E,
}

impl<E: fmt::Debug> fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "retry exhausted after {} attempts: {:?}",
            self.attempts, self.last_error
        )
    }
}

impl<E: fmt::Debug> std::error::Error for RetryError<E> {}

/// RetryPolicy (per 03-DD §7.2): max_retry + ExponentialBackoff。
#[derive(Debug, Clone, PartialEq)]
pub struct RetryPolicy {
    max_retry: u32,
    base_delay: Duration,
    backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retry: 3,
            base_delay: Duration::from_secs(1),
            backoff_factor: 2.0,
        }
    }
}

impl RetryPolicy {
    /// 新建。max_retry = 0 表示不重试 (禁止无限, per SRS §51)。
    pub fn new(max_retry: u32, base_delay: Duration, backoff_factor: f64) -> Self {
        Self {
            max_retry,
            base_delay,
            backoff_factor,
        }
    }

    /// 指数退避: base * factor^(attempt-1) → 1s, 2s, 4s (per 03-DD §7.1)。
    pub fn delay(&self, attempt: u32) -> Duration {
        let factor = self.backoff_factor.powi(attempt.saturating_sub(1) as i32);
        self.base_delay.mul_f64(factor)
    }

    /// 执行 op, 按 retryable 分类重试 (per 03-DD §7.2)。
    ///
    /// `sleep` 可注入 (测试用 no-op; 生产用 std::thread::sleep)。
    pub fn run<T, E, F>(
        &self,
        mut op: F,
        retryable: impl Fn(&E) -> bool,
        mut sleep: impl FnMut(Duration),
    ) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Result<T, E>,
    {
        let mut attempt: u32 = 0;
        loop {
            match op() {
                Ok(value) => return Ok(value),
                Err(err) => {
                    if attempt < self.max_retry && retryable(&err) {
                        attempt += 1;
                        sleep(self.delay(attempt));
                        continue;
                    }
                    return Err(RetryError {
                        attempts: attempt,
                        last_error: err,
                    });
                }
            }
        }
    }
}

/// 简易错误类别 (per 03-DD §7.1), 供调用方做 retryable 判定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// 可重试 (Network timeout / LLM rate limit / Tool 临时失败)。
    Retryable,
    /// 不可重试 (Invalid input / Permission denied)。
    NonRetryable,
    /// 致命 (OOM / 不变式违反), 立即失败 + 报警。
    Fatal,
}

/// 按 ErrorKind 判定 retryable (per 03-DD §7.1)。
pub fn retryable_by_kind(kind: &ErrorKind) -> bool {
    matches!(kind, ErrorKind::Retryable)
}

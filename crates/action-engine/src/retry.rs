//! `retry.rs` — Retry 策略 (per DD §39 + INV-WC-09)
//!
//! 3 档策略 (per ULYS-57.3 T8 acceptance: `action_retry_strategy_3_levels`):
//!
//! | Level | 错误类型               | 退避策略          | 最大次数 |
//! |-------|-----------------------|-------------------|---------|
//! | 1     | Network Error         | 指数退避 (1s, 2s, 4s) | 3     |
//! | 2     | Graph Connection     | 指数退避          | 3        |
//! | 3     | LLM Timeout          | 立即重试 + 降级模板 | 2      |
//!
//! 不重试: Git Conflict / Permission Denied / Validation / Lock Denied (per DD §39)
//!
//! 实际执行通过 `retry_with_backoff` 闭包执行, 调用方提供操作。

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// 重试决策
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryDecision {
    /// 重试
    Retry {
        /// 退避时间 (ms)
        delay_ms: u64,
    },
    /// 不重试 (致命错误)
    NoRetry,
    /// 已达最大次数, 上抛
    Exhausted,
}

/// 重试策略 (3 档, per ULYS-57.3 验收)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryPolicy {
    /// Network 错误 — 指数退避 1s, 2s, 4s, 最多 3 次
    Network {
        /// 当前 attempt (0-indexed)
        attempt: u32,
    },
    /// Graph 连接错误 — 指数退避, 最多 3 次
    GraphConnection {
        /// 当前 attempt (0-indexed)
        attempt: u32,
    },
    /// LLM 超时 — 立即重试 + 降级模板, 最多 2 次
    LlmTimeout {
        /// 当前 attempt (0-indexed)
        attempt: u32,
    },
}

impl RetryPolicy {
    /// 返回 3 档策略 (per 守门)
    pub const LEVELS: [RetryPolicy; 3] = [
        RetryPolicy::Network { attempt: 0 },
        RetryPolicy::GraphConnection { attempt: 0 },
        RetryPolicy::LlmTimeout { attempt: 0 },
    ];

    /// 3 档总数 (守门用)
    pub const LEVEL_COUNT: usize = 3;

    /// 每档最大重试次数 (per DD §39)
    pub fn max_attempts(&self) -> u32 {
        match self {
            RetryPolicy::Network { .. } | RetryPolicy::GraphConnection { .. } => 3,
            RetryPolicy::LlmTimeout { .. } => 2,
        }
    }

    /// 计算下一次退避时间
    pub fn next_delay(&self) -> Option<Duration> {
        let attempt = match *self {
            RetryPolicy::Network { attempt }
            | RetryPolicy::GraphConnection { attempt }
            | RetryPolicy::LlmTimeout { attempt } => attempt,
        };
        if attempt >= self.max_attempts() {
            return None;
        }
        let delay_ms = match self {
            // 指数退避 1s, 2s, 4s
            RetryPolicy::Network { attempt } | RetryPolicy::GraphConnection { attempt } => {
                1000u64 * (1u64 << attempt)
            }
            // LLM 立即重试 (per DD §39 — 1 次立即重试 + 降级到模板)
            RetryPolicy::LlmTimeout { attempt } => {
                if *attempt == 0 {
                    0
                } else {
                    500
                }
            }
        };
        Some(Duration::from_millis(delay_ms))
    }

    /// 推进 attempt +1
    pub fn next(&self) -> Self {
        match *self {
            RetryPolicy::Network { attempt } => RetryPolicy::Network {
                attempt: attempt + 1,
            },
            RetryPolicy::GraphConnection { attempt } => RetryPolicy::GraphConnection {
                attempt: attempt + 1,
            },
            RetryPolicy::LlmTimeout { attempt } => RetryPolicy::LlmTimeout {
                attempt: attempt + 1,
            },
        }
    }
}

/// 决策下一步: 根据当前 policy 决定 Retry/NoRetry/Exhausted
pub fn decide(policy: &RetryPolicy) -> RetryDecision {
    let attempt = match *policy {
        RetryPolicy::Network { attempt }
        | RetryPolicy::GraphConnection { attempt }
        | RetryPolicy::LlmTimeout { attempt } => attempt,
    };
    if attempt >= policy.max_attempts() {
        RetryDecision::Exhausted
    } else {
        match policy.next_delay() {
            Some(d) => RetryDecision::Retry {
                delay_ms: d.as_millis() as u64,
            },
            None => RetryDecision::Exhausted,
        }
    }
}

/// 执行 + 自动重试 (per DD §39)
///
/// `op` 返回 `Ok(T)` 视为成功; 返回 `Err(retryable)` 才重试。
/// 不可重试错误直接返回 `Err(err)`。
pub fn retry_with_backoff<T, F>(policy: RetryPolicy, mut op: F) -> Result<T, String>
where
    F: FnMut() -> Result<T, RetryError>,
{
    let mut cur = policy;
    loop {
        match op() {
            Ok(v) => return Ok(v),
            Err(RetryError::Fatal(msg)) => return Err(msg),
            Err(RetryError::Retryable(_)) => match decide(&cur) {
                RetryDecision::Retry { delay_ms } => {
                    if delay_ms > 0 {
                        std::thread::sleep(Duration::from_millis(delay_ms));
                    }
                    cur = cur.next();
                }
                RetryDecision::NoRetry => return Err("no retry allowed".to_string()),
                RetryDecision::Exhausted => {
                    return Err(format!(
                        "retry exhausted after {} attempts",
                        cur.max_attempts()
                    ));
                }
            },
        }
    }
}

/// 可重试错误 vs 致命错误
#[derive(Debug, Clone)]
pub enum RetryError {
    /// 可重试
    Retryable(String),
    /// 致命 (per DD §39: Git Conflict / Permission / Validation / Lock 不重试)
    Fatal(String),
}

impl std::fmt::Display for RetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::Retryable(s) | RetryError::Fatal(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_retry_strategy_3_levels() {
        // 守门 UT-6: Retry 3 档策略
        assert_eq!(RetryPolicy::LEVEL_COUNT, 3);
        assert_eq!(RetryPolicy::LEVELS.len(), 3);

        // Network: 1s, 2s, 4s
        let p = RetryPolicy::Network { attempt: 0 };
        assert_eq!(p.next_delay(), Some(Duration::from_millis(1000)));
        assert_eq!(p.next_delay(), Some(Duration::from_millis(1000))); // attempt 0 时固定 1s
        let p2 = RetryPolicy::Network { attempt: 1 };
        assert_eq!(p2.next_delay(), Some(Duration::from_millis(2000)));
        let p3 = RetryPolicy::Network { attempt: 2 };
        assert_eq!(p3.next_delay(), Some(Duration::from_millis(4000)));
        assert_eq!(p3.max_attempts(), 3);

        // GraphConnection: 同 Network (指数退避, 最多 3)
        let p = RetryPolicy::GraphConnection { attempt: 1 };
        assert_eq!(p.next_delay(), Some(Duration::from_millis(2000)));
        assert_eq!(p.max_attempts(), 3);

        // LlmTimeout: 立即 + 500ms (per DD §39)
        let p = RetryPolicy::LlmTimeout { attempt: 0 };
        assert_eq!(p.next_delay(), Some(Duration::from_millis(0))); // 立即
        let p2 = RetryPolicy::LlmTimeout { attempt: 1 };
        assert_eq!(p2.next_delay(), Some(Duration::from_millis(500)));
        assert_eq!(p.max_attempts(), 2);

        // Exhausted: attempt >= max_attempts 返回 None
        let p = RetryPolicy::Network { attempt: 3 };
        assert!(p.next_delay().is_none());
        assert_eq!(decide(&p), RetryDecision::Exhausted);
    }

    #[test]
    fn retry_with_backoff_succeeds_after_failures() {
        let mut count = 0u32;
        let result: Result<u32, String> =
            retry_with_backoff(RetryPolicy::LlmTimeout { attempt: 0 }, || {
                count += 1;
                if count < 3 {
                    Err(RetryError::Retryable("transient".into()))
                } else {
                    Ok(42)
                }
            });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(count, 3);
    }

    #[test]
    fn retry_with_backoff_exhausts_after_max() {
        let mut count = 0u32;
        let result: Result<u32, String> =
            retry_with_backoff(RetryPolicy::Network { attempt: 0 }, || {
                count += 1;
                Err(RetryError::Retryable("always fail".into()))
            });
        assert!(result.is_err());
        assert_eq!(count, 4); // 1 initial + 3 retries
    }

    #[test]
    fn retry_with_backoff_does_not_retry_fatal() {
        let mut count = 0u32;
        let result: Result<u32, String> =
            retry_with_backoff(RetryPolicy::Network { attempt: 0 }, || {
                count += 1;
                Err(RetryError::Fatal("git conflict".into()))
            });
        assert!(result.is_err());
        assert_eq!(count, 1);
    }
}

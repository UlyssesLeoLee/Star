//! Star CLI API Quota / Rate Limit / Retry (B.7 实装, per 2026-08-30 07:13 JST wt-b7-api-quota)
//!
//! 职责:
//!   1. **QuotaGuard**: API 配额追踪 (per minute / per hour / per day 上限)
//!   2. **RateLimiter**: 限流 (token bucket, 1 token / 1 req)
//!   3. **retry_with_backoff**: 指数退避 + 抖动 (per ADR-0029 Universal Submit)
//!   4. **ApiError**: 区分 transient (429 / 503 / timeout) vs permanent (401 / 403 / 404) 错误
//!
//! 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//!   1. QuotaGuard 当前内存计数, 跨进程不持久化 (Phase 2 接 Redis / 持久层)
//!   2. RateLimiter token bucket 简化版, 不支持 burst 调节 (Phase 2 接 leaky bucket)
//!   3. retry_with_backoff 不支持 idempotency key 投递 (P3-D 阶段接)
//!   4. 不接 KMS (per E.4 KMS 集成凭证)
//!
//! 不做 (per 守门):
//!   - 不动 network 层 (per Phase D)
//!   - 不动 domain-local-runtime http_client (per B.1/B.2)
//!   - 不写 UI (per B.9 API 监控审计)

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// API 错误类型 (区分 transient / permanent)
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApiError {
    /// HTTP 429 Too Many Requests (transient, 触发退避)
    #[error("rate limited (retry after {retry_after_secs}s)")]
    RateLimited {
        retry_after_secs: u64,
    },
    /// HTTP 503 Service Unavailable (transient)
    #[error("service unavailable")]
    ServiceUnavailable,
    /// 连接超时 (transient)
    #[error("timeout after {0}ms")]
    Timeout(u64),
    /// HTTP 401 Unauthorized (permanent, 不重试)
    #[error("unauthorized: check API key")]
    Unauthorized,
    /// HTTP 403 Forbidden (permanent)
    #[error("forbidden: insufficient scope")]
    Forbidden,
    /// HTTP 404 Not Found (permanent)
    #[error("not found: {0}")]
    NotFound(String),
    /// 配额超限 (transient, 等下个窗口)
    #[error("quota exceeded for {scope}")]
    QuotaExceeded {
        scope: String,
    },
    /// 其他未知错误
    #[error("unknown error: {0}")]
    Other(String),
}

impl ApiError {
    /// 是否可重试 (transient 错误)
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            ApiError::RateLimited { .. }
                | ApiError::ServiceUnavailable
                | ApiError::Timeout(_)
                | ApiError::QuotaExceeded { .. }
        )
    }
}

/// 配额追踪 (per scope: per minute / per hour / per day)
#[derive(Debug, Clone)]
pub struct QuotaGuard {
    scope: String,
    limit: u32,
    window: Duration,
    used: u32,
    window_start: Instant,
}

impl QuotaGuard {
    /// 新建配额守卫
    ///
    /// # Examples
    ///
    /// ```
    /// use domain_cli::quota::QuotaGuard;
    /// use std::time::Duration;
    ///
    /// let guard = QuotaGuard::new("openclaw", 60, Duration::from_secs(60));
    /// assert_eq!(guard.remaining(), 60);
    /// ```
    pub fn new(scope: impl Into<String>, limit: u32, window: Duration) -> Self {
        Self {
            scope: scope.into(),
            limit,
            window,
            used: 0,
            window_start: Instant::now(),
        }
    }

    /// 当前剩余配额
    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }

    /// 配额 scope 名称
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// 检查并消耗 1 单位配额
    pub fn try_consume(&mut self) -> Result<(), ApiError> {
        // 窗口已过 → 重置
        if self.window_start.elapsed() >= self.window {
            self.used = 0;
            self.window_start = Instant::now();
        }
        if self.used >= self.limit {
            return Err(ApiError::QuotaExceeded {
                scope: self.scope.clone(),
            });
        }
        self.used += 1;
        Ok(())
    }
}

/// 限流器 (token bucket 简化版, 1 token / req)
#[derive(Debug)]
pub struct RateLimiter {
    /// 间隔 (Duration between requests)
    interval: Duration,
    last_request: Option<Instant>,
}

impl RateLimiter {
    /// 新建限流器 (per `interval` 至少 1 req)
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_request: None,
        }
    }

    /// 检查并放行 (若距上次 req < interval, 返回 RateLimited)
    pub fn try_acquire(&mut self) -> Result<(), ApiError> {
        let now = Instant::now();
        if let Some(last) = self.last_request {
            let elapsed = now.duration_since(last);
            if elapsed < self.interval {
                let retry_after = self.interval - elapsed;
                return Err(ApiError::RateLimited {
                    retry_after_secs: retry_after.as_secs(),
                });
            }
        }
        self.last_request = Some(now);
        Ok(())
    }
}

/// 指数退避配置
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// 初始延迟 (ms)
    pub initial_delay_ms: u64,
    /// 最大延迟 (ms)
    pub max_delay_ms: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 抖动因子 (0.0-1.0, 0 = 无抖动, 1 = 0%-100% 随机)
    pub jitter_factor: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            max_retries: 5,
            jitter_factor: 0.3,
        }
    }
}

/// 指数退避 + 抖动 retry (per ADR-0029 Universal Submit 12 步)
pub fn retry_with_backoff<F, T>(
    config: &BackoffConfig,
    mut op: F,
) -> Result<T, ApiError>
where
    F: FnMut() -> Result<T, ApiError>,
{
    let mut attempt = 0u32;
    loop {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if !e.is_transient() => return Err(e),
            Err(_) if attempt >= config.max_retries => {
                return Err(ApiError::Other(format!(
                    "max retries ({}) exceeded",
                    config.max_retries
                )))
            }
            Err(_) => {
                // 计算 delay: initial * 2^attempt, 截断到 max_delay, 加 jitter
                let base_delay = config.initial_delay_ms.saturating_mul(1u64 << attempt.min(20));
                let delay = base_delay.min(config.max_delay_ms);
                let jitter = (delay as f64 * config.jitter_factor) as u64;
                let actual_delay = delay.saturating_sub(jitter / 2)
                    + (jitter % 2);
                std::thread::sleep(Duration::from_millis(actual_delay));
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn quota_guard_consume() {
        let mut guard = QuotaGuard::new("test", 3, Duration::from_secs(60));
        assert_eq!(guard.remaining(), 3);
        guard.try_consume().unwrap();
        guard.try_consume().unwrap();
        guard.try_consume().unwrap();
        assert_eq!(guard.remaining(), 0);
        let err = guard.try_consume().unwrap_err();
        assert!(matches!(err, ApiError::QuotaExceeded { .. }));
    }

    #[test]
    fn rate_limiter_blocks_too_fast() {
        let mut limiter = RateLimiter::new(Duration::from_millis(100));
        limiter.try_acquire().unwrap();
        // 立刻第二次 → 限流
        let err = limiter.try_acquire().unwrap_err();
        assert!(matches!(err, ApiError::RateLimited { .. }));
    }

    #[test]
    fn retry_skips_permanent() {
        let config = BackoffConfig::default();
        let result: Result<u32, ApiError> = retry_with_backoff(&config, || Err(ApiError::Unauthorized));
        // permanent 错误, 不重试, 立刻返回
        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }

    #[test]
    fn retry_eventually_succeeds() {
        let config = BackoffConfig {
            initial_delay_ms: 1,
            max_delay_ms: 10,
            max_retries: 5,
            jitter_factor: 0.0,
        };
        let counter = Arc::new(AtomicU32::new(0));
        let counter2 = counter.clone();
        let result: Result<u32, ApiError> = retry_with_backoff(&config, move || {
            let n = counter2.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err(ApiError::ServiceUnavailable)
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert!(counter.load(Ordering::SeqCst) >= 3);
    }

    #[test]
    fn api_error_classify() {
        assert!(ApiError::RateLimited { retry_after_secs: 1 }.is_transient());
        assert!(ApiError::ServiceUnavailable.is_transient());
        assert!(ApiError::Timeout(5000).is_transient());
        assert!(ApiError::QuotaExceeded { scope: "x".into() }.is_transient());
        assert!(!ApiError::Unauthorized.is_transient());
        assert!(!ApiError::Forbidden.is_transient());
        assert!(!ApiError::NotFound("foo".into()).is_transient());
    }
}

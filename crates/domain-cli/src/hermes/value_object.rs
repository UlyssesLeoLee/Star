//! Hermes 值对象 (B.2 4 层精简: value_object 层)
//!
//! 含:
//!   1. **HermesConfig**: 客户端配置 (base_url / api_key / timeout / mode / retry_policy)
//!   2. **HermesMode**: 客户端模式 (Mock / Real)
//!   3. **RetryPolicy**: 重试策略 (NoRetry / FixedDelay / ExponentialBackoff)
//!
//! 与 B.1 OpenClawConfig 的差异:
//!   - B.1 retry: 一次性 (per PHASE-P3-B1-IMPL-REPORT §3 #5 已知缺口)
//!   - B.2 retry: 3 变体 RetryPolicy enum (B.2 新增, B.7 retry_with_backoff 集成在 phase 2)
//!   - B.1 mode: bool (mock_mode)
//!   - B.2 mode: HermesMode enum (更明确, 配 Default::default() = Mock)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Hermes 客户端模式 (B.2: enum 区分 vs B.1 bool 区分, 配 Default = Mock)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HermesMode {
    /// Mock 模式: 不发 HTTP, 调用 mock_response_*
    #[default]
    Mock,
    /// 真实模式: 发 HTTP POST / GET / DELETE, 凭证必填
    Real,
}

/// 重试策略 (B.2: 3 变体, 不带抖动, 抖动在 B.7 phase 2 整合)
///
/// 与 B.7 quota::BackoffConfig 的关系:
///   - RetryPolicy 简化版: 仅 max_attempts + delay 策略
///   - BackoffConfig 完整版: max_attempts + initial_delay + multiplier + max_delay + jitter
///   - phase 2 HermesClient::submit/cancel 套 retry_with_backoff (B.7) wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RetryPolicy {
    /// 不重试 (1 次尝试)
    NoRetry,
    /// 固定间隔 (max_attempts, delay_ms)
    FixedDelay { max_attempts: u32, delay_ms: u64 },
    /// 指数退避 (max_attempts, initial_delay_ms, multiplier)
    /// 实际 delay = initial_delay_ms * multiplier^(attempt-1), 不带 max_delay cap
    ExponentialBackoff {
        max_attempts: u32,
        initial_delay_ms: u64,
        multiplier: u32,
    },
}

impl Default for RetryPolicy {
    fn default() -> Self {
        // 跟 B.1 mock 模式保持一致: 不重试 (1 次尝试)
        RetryPolicy::NoRetry
    }
}

impl RetryPolicy {
    /// 实际重试次数 (0 = NoRetry 即 1 次尝试)
    pub fn max_attempts(&self) -> u32 {
        match self {
            RetryPolicy::NoRetry => 1,
            RetryPolicy::FixedDelay { max_attempts, .. } => *max_attempts,
            RetryPolicy::ExponentialBackoff { max_attempts, .. } => *max_attempts,
        }
    }

    /// 第 N 次重试的 delay (per attempt index 0-based; 0 = 首次不延迟, 1+ = 重试延迟)
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        match self {
            RetryPolicy::NoRetry => None,
            RetryPolicy::FixedDelay { delay_ms, .. } => Some(Duration::from_millis(*delay_ms)),
            RetryPolicy::ExponentialBackoff {
                initial_delay_ms,
                multiplier,
                ..
            } => {
                if attempt == 0 {
                    return None;
                }
                let mul = u64::from(multiplier.pow(attempt - 1));
                Some(Duration::from_millis(initial_delay_ms * mul))
            }
        }
    }
}

/// Hermes 客户端配置
#[derive(Debug, Clone)]
pub struct HermesConfig {
    /// base URL (mock 模式用 wiremock / ms, 真实模式用 https://api.hermes.dev/v1)
    pub base_url: String,
    /// API key (env: HERMES_API_KEY)
    pub api_key: String,
    /// 请求 timeout
    pub timeout: Duration,
    /// 客户端模式 (Mock / Real)
    pub mode: HermesMode,
    /// 重试策略 (per B.7 retry_with_backoff phase 2 整合)
    pub retry_policy: RetryPolicy,
}

impl HermesConfig {
    /// 新建默认 mock 配置 (per 29692a7 mock 备选路径解锁, 默认 mock 直到真实凭证到位)
    pub fn new_mock() -> Self {
        Self {
            base_url: "http://localhost:8082/v1".into(),
            api_key: "mock-key".into(),
            timeout: Duration::from_secs(30),
            mode: HermesMode::Mock,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// 新建真实模式配置 (凭证必填, base_url 必填)
    pub fn new_real(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Result<Self, super::error::HermesError> {
        use super::error::HermesError;
        let base_url = base_url.into();
        let api_key = api_key.into();
        if base_url.is_empty() {
            return Err(HermesError::Auth("base_url is empty".into()));
        }
        if api_key.is_empty() {
            return Err(HermesError::Auth("api_key is empty".into()));
        }
        Ok(Self {
            base_url,
            api_key,
            timeout: Duration::from_secs(30),
            mode: HermesMode::Real,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// 链式 set: base_url
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// 链式 set: timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 链式 set: retry_policy
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// 链式 set: mode (Mock / Real)
    pub fn with_mode(mut self, mode: HermesMode) -> Self {
        self.mode = mode;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermes_mode_default_is_mock() {
        assert_eq!(HermesMode::default(), HermesMode::Mock);
    }

    #[test]
    fn retry_policy_default_is_no_retry() {
        let p = RetryPolicy::default();
        assert_eq!(p, RetryPolicy::NoRetry);
        assert_eq!(p.max_attempts(), 1);
        assert_eq!(p.delay_for_attempt(0), None);
    }

    #[test]
    fn retry_policy_fixed_delay() {
        let p = RetryPolicy::FixedDelay {
            max_attempts: 3,
            delay_ms: 200,
        };
        assert_eq!(p.max_attempts(), 3);
        assert_eq!(p.delay_for_attempt(0), Some(Duration::from_millis(200)));
        assert_eq!(p.delay_for_attempt(1), Some(Duration::from_millis(200)));
    }

    #[test]
    fn retry_policy_exponential_backoff() {
        let p = RetryPolicy::ExponentialBackoff {
            max_attempts: 4,
            initial_delay_ms: 100,
            multiplier: 2,
        };
        assert_eq!(p.max_attempts(), 4);
        // attempt 0: 首次, 不延迟
        assert_eq!(p.delay_for_attempt(0), None);
        // attempt 1: 100 * 2^0 = 100ms
        assert_eq!(p.delay_for_attempt(1), Some(Duration::from_millis(100)));
        // attempt 2: 100 * 2^1 = 200ms
        assert_eq!(p.delay_for_attempt(2), Some(Duration::from_millis(200)));
        // attempt 3: 100 * 2^2 = 400ms
        assert_eq!(p.delay_for_attempt(3), Some(Duration::from_millis(400)));
    }

    #[test]
    fn hermes_config_new_mock_defaults() {
        let cfg = HermesConfig::new_mock();
        assert_eq!(cfg.mode, HermesMode::Mock);
        assert!(!cfg.base_url.is_empty());
        assert!(!cfg.api_key.is_empty());
        assert_eq!(cfg.base_url, "http://localhost:8082/v1");
        assert_eq!(cfg.api_key, "mock-key");
        assert_eq!(cfg.timeout, Duration::from_secs(30));
    }

    #[test]
    fn hermes_config_new_real_rejects_empty_key() {
        let r = HermesConfig::new_real("https://api.hermes.dev/v1", "");
        assert!(matches!(r, Err(super::super::error::HermesError::Auth(_))));
    }

    #[test]
    fn hermes_config_new_real_rejects_empty_url() {
        let r = HermesConfig::new_real("", "test-key");
        assert!(matches!(r, Err(super::super::error::HermesError::Auth(_))));
    }

    #[test]
    fn hermes_config_new_real_accepts_valid() {
        let r = HermesConfig::new_real("https://api.hermes.dev/v1", "test-key");
        assert!(r.is_ok());
        let cfg = r.unwrap();
        assert_eq!(cfg.mode, HermesMode::Real);
        assert_eq!(cfg.base_url, "https://api.hermes.dev/v1");
        assert_eq!(cfg.api_key, "test-key");
    }

    #[test]
    fn hermes_config_chain_setters() {
        let cfg = HermesConfig::new_mock()
            .with_base_url("http://custom:9090/v2")
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(RetryPolicy::FixedDelay {
                max_attempts: 5,
                delay_ms: 500,
            });
        assert_eq!(cfg.base_url, "http://custom:9090/v2");
        assert_eq!(cfg.timeout, Duration::from_secs(60));
        assert_eq!(cfg.retry_policy.max_attempts(), 5);
    }

    #[test]
    fn retry_policy_serde_roundtrip() {
        let p = RetryPolicy::ExponentialBackoff {
            max_attempts: 3,
            initial_delay_ms: 100,
            multiplier: 2,
        };
        let json = serde_json::to_string(&p).unwrap();
        // serde tag = "type" + flatten 字段
        assert!(json.contains("\"type\":\"exponential_backoff\""));
        assert!(json.contains("\"max_attempts\":3"));
        let back: RetryPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}

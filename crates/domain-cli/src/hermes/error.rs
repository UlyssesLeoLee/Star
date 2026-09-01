//! Hermes 错误层 (B.2 4 层精简: error 层)
//!
//! 含 **HermesError** enum: 4 变体
//!   1. **Http**: reqwest 错误 (network / DNS / TLS / etc) — transient, 可重试
//!   2. **Auth**: 认证失败 (401/403) 或凭证配置错误 (empty key/url) — permanent, 不可重试
//!   3. **ServerError**: API 返回 5xx — transient, 可重试
//!   4. **Parse**: 响应 parse 失败 (JSON schema 不匹配) — permanent, 不可重试
//!
//! 与 B.1 OpenClawError 的差异:
//!   - B.1: Http / InvalidKey / NonSuccess(u16, String) / Parse (4 变体, NonSuccess 包含 4xx+5xx)
//!   - B.2: Http / Auth / ServerError / Parse (4 变体, **Auth 与 ServerError 拆分**)
//!   - B.2 新增 is_transient() 方法 (per B.7 retry_with_backoff phase 2 整合)
//!   - B.1: NonSuccess 是 1 变体, 4xx 和 5xx 不区分; B.2: 4xx 走 Auth, 5xx 走 ServerError
//!
//! 与 B.6 hermes_client.rs HermesError 的差异:
//!   - B.6: 跟 B.1 同结构 (Http / InvalidKey / NonSuccess / Parse)
//!   - B.2: Auth / ServerError 拆分, **per B.2 task queue 5 endpoint 设计**

use std::time::Duration;

/// Hermes API 错误 (B.2 4 变体, 区分 transient / permanent)
#[derive(Debug, thiserror::Error)]
pub enum HermesError {
    /// HTTP 请求失败 (network / DNS / TLS / timeout)
    /// **transient** — 可重试 (per B.7 retry_with_backoff)
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// 认证失败 (HTTP 401/403) 或凭证配置错误 (empty key/url)
    /// **permanent** — 不可重试
    #[error("auth failed: {0}")]
    Auth(String),

    /// API 返回 5xx (server error)
    /// **transient** — 可重试
    #[error("server error: status={0} body={1}")]
    ServerError(u16, String),

    /// 响应 parse 失败 (JSON schema 不匹配)
    /// **permanent** — 不可重试
    #[error("response parse failed: {0}")]
    Parse(String),
}

impl HermesError {
    /// 是否可重试 (per B.7 retry_with_backoff phase 2 整合)
    ///
    /// transient → Http / ServerError (网络/服务端临时错误)
    /// permanent → Auth / Parse (认证/格式错误, 重试无意义)
    pub fn is_transient(&self) -> bool {
        match self {
            HermesError::Http(_) => true,
            HermesError::ServerError(_, _) => true,
            HermesError::Auth(_) => false,
            HermesError::Parse(_) => false,
        }
    }

    /// HTTP 状态码 (仅 ServerError 有, 其他 None)
    pub fn status_code(&self) -> Option<u16> {
        match self {
            HermesError::ServerError(s, _) => Some(*s),
            _ => None,
        }
    }

    /// 简短描述 (per logs / metrics)
    pub fn short(&self) -> &'static str {
        match self {
            HermesError::Http(_) => "http",
            HermesError::Auth(_) => "auth",
            HermesError::ServerError(_, _) => "server",
            HermesError::Parse(_) => "parse",
        }
    }
}

/// 辅助: 把 HTTP status code 映射到 HermesError
///
/// per Hermes API spec:
///   - 2xx: 调用方处理成功响应
///   - 401/403: Auth
///   - 5xx: ServerError
///   - 其他 4xx: Auth (per B.2 简化, 4xx 都当 auth/客户端错误, 不细分)
pub fn classify_status(status: u16, body: String) -> HermesError {
    if status == 401 || status == 403 {
        HermesError::Auth(format!("status={} body={}", status, body))
    } else if (500..600).contains(&status) {
        HermesError::ServerError(status, body)
    } else if (400..500).contains(&status) {
        // 4xx (除 401/403): 当 Auth (per B.2 简化, 不细分 404/422/etc)
        HermesError::Auth(format!("client error status={} body={}", status, body))
    } else {
        // 1xx/3xx: 不应出现在 HTTP API, 当 Parse 错误
        HermesError::Parse(format!("unexpected status={} body={}", status, body))
    }
}

/// 重试 helper: 计算下一次重试的 delay (per RetryPolicy)
///
/// 返回 Some(duration) 表示需要重试, None 表示停止 (no more attempts 或 permanent error)
pub fn next_retry_delay(
    attempt: u32,
    policy: &super::value_object::RetryPolicy,
) -> Option<Duration> {
    if attempt == 0 {
        return None;
    }
    let max = policy.max_attempts();
    if attempt >= max {
        return None;
    }
    policy.delay_for_attempt(attempt)
}

#[cfg(test)]
mod tests {
    use super::super::value_object::RetryPolicy;
    use super::*;

    #[test]
    fn hermes_error_is_transient_classification() {
        // transient
        assert!(HermesError::ServerError(500, "internal".into()).is_transient());
        assert!(HermesError::ServerError(502, "bad gateway".into()).is_transient());
        assert!(HermesError::ServerError(503, "unavailable".into()).is_transient());
        // permanent
        assert!(!HermesError::Auth("401 unauthorized".into()).is_transient());
        assert!(!HermesError::Parse("invalid json".into()).is_transient());
    }

    #[test]
    fn hermes_error_status_code() {
        assert_eq!(
            HermesError::ServerError(500, "x".into()).status_code(),
            Some(500)
        );
        assert_eq!(
            HermesError::ServerError(503, "x".into()).status_code(),
            Some(503)
        );
        assert_eq!(HermesError::Auth("401".into()).status_code(), None);
        assert_eq!(HermesError::Parse("x".into()).status_code(), None);
    }

    #[test]
    fn hermes_error_short_label() {
        assert_eq!(HermesError::ServerError(500, "x".into()).short(), "server");
        assert_eq!(HermesError::Auth("x".into()).short(), "auth");
        assert_eq!(HermesError::Parse("x".into()).short(), "parse");
    }

    #[test]
    fn classify_status_401_403_is_auth() {
        let e = classify_status(401, "unauthorized".into());
        assert!(matches!(e, HermesError::Auth(_)));

        let e = classify_status(403, "forbidden".into());
        assert!(matches!(e, HermesError::Auth(_)));
    }

    #[test]
    fn classify_status_5xx_is_server_error() {
        for status in [500u16, 502, 503, 504] {
            let e = classify_status(status, "internal".into());
            assert!(matches!(e, HermesError::ServerError(s, _) if s == status));
        }
    }

    #[test]
    fn classify_status_4xx_non_401_403_is_auth() {
        // 简化版: 4xx 都当 Auth, 不细分
        for status in [400u16, 404, 422, 429] {
            let e = classify_status(status, "client error".into());
            assert!(matches!(e, HermesError::Auth(_)));
        }
    }

    #[test]
    fn classify_status_1xx_3xx_is_parse() {
        let e = classify_status(100, "continue".into());
        assert!(matches!(e, HermesError::Parse(_)));
        let e = classify_status(301, "moved".into());
        assert!(matches!(e, HermesError::Parse(_)));
    }

    #[test]
    fn next_retry_delay_no_retry() {
        let p = RetryPolicy::NoRetry;
        assert_eq!(next_retry_delay(1, &p), None);
    }

    #[test]
    fn next_retry_delay_fixed() {
        let p = RetryPolicy::FixedDelay {
            max_attempts: 3,
            delay_ms: 100,
        };
        // attempt 0: 不重试
        assert_eq!(next_retry_delay(0, &p), None);
        // attempt 1: 100ms
        assert_eq!(next_retry_delay(1, &p), Some(Duration::from_millis(100)));
        // attempt 2: 100ms
        assert_eq!(next_retry_delay(2, &p), Some(Duration::from_millis(100)));
        // attempt 3: >= max_attempts, None
        assert_eq!(next_retry_delay(3, &p), None);
    }

    #[test]
    fn next_retry_delay_exponential() {
        let p = RetryPolicy::ExponentialBackoff {
            max_attempts: 3,
            initial_delay_ms: 100,
            multiplier: 2,
        };
        assert_eq!(next_retry_delay(0, &p), None);
        assert_eq!(next_retry_delay(1, &p), Some(Duration::from_millis(100)));
        assert_eq!(next_retry_delay(2, &p), Some(Duration::from_millis(200)));
        assert_eq!(next_retry_delay(3, &p), None);
    }
}

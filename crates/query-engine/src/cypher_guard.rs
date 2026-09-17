//! `cypher_guard.rs` — Cypher 注入防护 (per NFR-SEC-002 + 守门 #10)
//!
//! 6 关键字 + op + file path 全部走 allowlist sanitization,
//! 生成的 Cypher 用 `$param` 参数化, 不拼接字符串.
//!
//! 守门:
//! - #10 Cypher 注入防护
//!
//! 主要入口:
//! - `sanitize_cypher_param` — 单一参数净化 (字符级)
//! - `is_safe_cypher_param` — 检查是否安全 (字符级)

use crate::error::QueryError;

/// 净化 cypher 参数 — 不安全字符返回 error
///
/// # Errors
///
/// - `CYPHER_INJECTION_DETECTED` — 含不允许字符或 `..`
pub fn sanitize_cypher_param(value: &str) -> Result<String, QueryError> {
    if value.is_empty() {
        return Err(QueryError::cypher_injection("empty cypher parameter"));
    }
    if value.contains(';') || value.contains('$') || value.contains('{') || value.contains('}') {
        return Err(QueryError::cypher_injection(format!(
            "parameter contains Cypher meta characters: '{value}'"
        )));
    }
    if value.contains("..") {
        return Err(QueryError::cypher_injection(format!(
            "parameter contains '..': '{value}'"
        )));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/' | ',' | ' '))
    {
        return Err(QueryError::cypher_injection(format!(
            "parameter contains disallowed characters: '{value}'"
        )));
    }
    Ok(value.to_string())
}

/// 检查字符串是否可作为安全 cypher 参数
pub fn is_safe_cypher_param(value: &str) -> bool {
    sanitize_cypher_param(value).is_ok()
}

/// Cypher guard 错误 (alias for backward compat / 类型导出)
pub type CypherGuardError = QueryError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_cypher_injection_safe() {
        // 守门 UT-3 (T11): Cypher 注入防护

        // 安全参数
        assert!(is_safe_cypher_param("conflict"));
        assert!(is_safe_cypher_param("codex"));
        assert!(is_safe_cypher_param("auth.rs"));
        assert!(is_safe_cypher_param("path/to/file_name-1.txt"));

        // 注入尝试 — 必须拒绝
        assert!(!is_safe_cypher_param("'; DROP TABLE users;--"));
        assert!(!is_safe_cypher_param("a] MATCH (n) DETACH DELETE n //"));
        assert!(!is_safe_cypher_param("$malicious"));
        assert!(!is_safe_cypher_param("a{b}"));
        assert!(!is_safe_cypher_param("../etc/passwd"));
        assert!(!is_safe_cypher_param("with space and !@#"));
        assert!(!is_safe_cypher_param(""));
    }

    #[test]
    fn sanitize_returns_specific_error() {
        let err = sanitize_cypher_param("a; b").expect_err("should fail");
        assert_eq!(err.code, "CYPHER_INJECTION_DETECTED");
    }

    #[test]
    fn sanitize_accepts_valid_values() {
        let v = sanitize_cypher_param("conflict,unmerged").expect("safe");
        assert_eq!(v, "conflict,unmerged");
    }
}

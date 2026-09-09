// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 Bearer auth middleware helpers (per WBS v0.43 §14.12 IV phase 2)
//!
//! v0.43 简化: 不实装 FromRequestParts extractor (需要 axum::async_trait, 跨 session 续)
//! 仅提供 BearerError + require_scope/require_role helpers + 测试
//!
//! 跨 session 续: AuthenticatedUser extractor + 5 域 RBAC 实证
//!
//! 守门 #5 v2: token 不入 log
//! 守门 #14 v3: Mavis 永久代签

use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthUser;

/// Bearer auth 错误 (per RFC 6750 §3.1)
#[derive(Debug)]
pub enum BearerError {
    /// 缺 Authorization 头
    MissingHeader,
    /// Authorization 头格式错 (不是 "Bearer ...")
    InvalidFormat,
    /// Token 验签失败
    InvalidToken(String),
    /// Token 已过期
    ExpiredToken,
    /// 5 域 RBAC 拒绝 (per 5 域 Lead 拍板, Mavis 临时代签)
    InsufficientScope(String),
}

impl BearerError {
    /// 6-field code (跟 v0.30 错误模型一致)
    pub fn code(&self) -> &'static str {
        match self {
            BearerError::MissingHeader => "BEARER_MISSING_HEADER",
            BearerError::InvalidFormat => "BEARER_INVALID_FORMAT",
            BearerError::InvalidToken(_) => "BEARER_INVALID_TOKEN",
            BearerError::ExpiredToken => "BEARER_TOKEN_EXPIRED",
            BearerError::InsufficientScope(_) => "BEARER_INSUFFICIENT_SCOPE",
        }
    }

    /// retriable (per 守门 #6 v2)
    pub fn is_retriable(&self) -> bool {
        matches!(self, BearerError::ExpiredToken)
    }
}

/// BearerToken 持有者 (helper struct, 跨 session 续 axum extractor)
pub struct BearerToken(pub String);

/// AuthenticatedUser 持有者 (placeholder, 跨 session 续 axum extractor)
pub struct AuthenticatedUser(pub AuthUser);

/// Require scope helper (5 域 RBAC, per 守门 #14 v3 5 域 Lead 拍板 跨 session 续)
pub fn require_scope(user: &AuthUser, required: &str) -> Result<(), BearerError> {
    if user.scope.split_whitespace().any(|s| s == required) {
        Ok(())
    } else {
        Err(BearerError::InsufficientScope(required.to_string()))
    }
}

/// Require role helper (5 域 Lead 角色, per 守门 #3 v2 + 9/3 11:35 JST Mavis 临时代签)
pub fn require_role(user: &AuthUser, role: &str) -> Result<(), BearerError> {
    if user.roles.iter().any(|r| r == role) {
        Ok(())
    } else {
        Err(BearerError::InsufficientScope(format!("role:{}", role)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_scope_grants_matching_scope() {
        let user = AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec!["user".to_string()],
            scope: "read write".to_string(),
        };
        assert!(require_scope(&user, "read").is_ok());
        assert!(require_scope(&user, "write").is_ok());
        assert!(require_scope(&user, "admin").is_err());
    }

    #[test]
    fn require_role_grants_matching_role() {
        let user = AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec!["player".to_string(), "ops".to_string()],
            scope: "".to_string(),
        };
        assert!(require_role(&user, "player").is_ok());
        assert!(require_role(&user, "ops").is_ok());
        assert!(require_role(&user, "admin").is_err());
    }

    #[test]
    fn require_scope_with_empty_scope_denies_all() {
        let user = AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec![],
            scope: "".to_string(),
        };
        assert!(require_scope(&user, "read").is_err());
    }

    #[test]
    fn bearer_error_codes_are_distinct() {
        assert_eq!(BearerError::MissingHeader.code(), "BEARER_MISSING_HEADER");
        assert_eq!(BearerError::InvalidFormat.code(), "BEARER_INVALID_FORMAT");
        assert_eq!(
            BearerError::InvalidToken("x".to_string()).code(),
            "BEARER_INVALID_TOKEN"
        );
        assert_eq!(BearerError::ExpiredToken.code(), "BEARER_TOKEN_EXPIRED");
        assert_eq!(
            BearerError::InsufficientScope("x".to_string()).code(),
            "BEARER_INSUFFICIENT_SCOPE"
        );
    }

    #[test]
    fn bearer_error_retriable() {
        assert!(BearerError::ExpiredToken.is_retriable());
        assert!(!BearerError::MissingHeader.is_retriable());
        assert!(!BearerError::InvalidFormat.is_retriable());
        assert!(!BearerError::InvalidToken("x".to_string()).is_retriable());
        assert!(!BearerError::InsufficientScope("x".to_string()).is_retriable());
    }
}

// 占位符避免 unused warning
const _PLACEHOLDER: () = ();

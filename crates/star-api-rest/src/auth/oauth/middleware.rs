// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 Bearer auth middleware helpers (per WBS v0.43 §14.12 IV phase 2)
//!
//! v0.47 完整化 (per brief §3.1):
//! - AuthenticatedUser extractor (axum::async_trait + FromRequestParts)
//! - BearerError 5-variant (per RFC 6750 §3.1)
//! - require_scope / require_role 5 域 RBAC helper
//!
//! 守门 #5 v2: token 不入 log
//! 守门 #6 v2: BearerError::ExpiredToken retriable
//! 守门 #14 v3: Mavis 永久代签

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::sync::Arc;

use crate::auth::{verify_token, AuthUser, JwtConfig};

/// Bearer auth 错误 (per RFC 6750 §3.1, 6-variant)
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
    /// 服务端内部错误 (e.g. JWT config 缺, DB 查 access_token 失败)
    /// per 守门 #5 v2: 错误详情不入 log response, 仅 error code
    ServerError(String),
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
            BearerError::ServerError(_) => "BEARER_SERVER_ERROR",
        }
    }

    /// retriable (per 守门 #6 v2)
    pub fn is_retriable(&self) -> bool {
        matches!(self, BearerError::ExpiredToken)
    }

    /// WWW-Authenticate header value (per RFC 6750 §3)
    pub fn www_authenticate(&self) -> String {
        match self {
            BearerError::MissingHeader => format!(
                r#"Bearer realm="star-api-rest", error="invalid_request""#
            ),
            BearerError::InvalidFormat => format!(
                r#"Bearer realm="star-api-rest", error="invalid_request""#
            ),
            BearerError::InvalidToken(_) => format!(
                r#"Bearer realm="star-api-rest", error="invalid_token""#
            ),
            BearerError::ExpiredToken => format!(
                r#"Bearer realm="star-api-rest", error="invalid_token", error_description="token expired""#
            ),
            BearerError::InsufficientScope(scope) => format!(
                r#"Bearer realm="star-api-rest", error="insufficient_scope", scope="{}""#,
                scope
            ),
            BearerError::ServerError(_) => format!(
                r#"Bearer realm="star-api-rest", error="server_error""#
            ),
        }
    }
}

impl IntoResponse for BearerError {
    fn into_response(self) -> Response {
        let status = match self {
            BearerError::MissingHeader | BearerError::InvalidFormat => StatusCode::UNAUTHORIZED,
            BearerError::InvalidToken(_) | BearerError::ExpiredToken => StatusCode::UNAUTHORIZED,
            BearerError::InsufficientScope(_) => StatusCode::FORBIDDEN,
            BearerError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = json!({
            "error": self.code(),
            "error_description": match &self {
                BearerError::MissingHeader => "missing Authorization header".to_string(),
                BearerError::InvalidFormat => "Authorization header must be 'Bearer <token>'".to_string(),
                BearerError::InvalidToken(d) => format!("invalid token: {}", d),
                BearerError::ExpiredToken => "token expired".to_string(),
                BearerError::InsufficientScope(s) => format!("insufficient scope: {}", s),
                BearerError::ServerError(_) => "internal server error".to_string(),
            },
        });
        let www_auth = self.www_authenticate();
        (
            status,
            [(header::WWW_AUTHENTICATE, www_auth)],
            Json(body),
        )
            .into_response()
    }
}

/// BearerToken 持有者 (helper struct, 给 axum extractor 用)
pub struct BearerToken(pub String);

/// AuthenticatedUser (axum extractor, 给 handler 注入 AuthUser)
///
/// 用法 (per brief §3.1):
/// ```ignore
/// async fn handler(user: AuthenticatedUser) -> impl IntoResponse {
///     let user_id = user.0.user_id;
///     ...
/// }
/// ```
pub struct AuthenticatedUser(pub AuthUser);

/// Axum extractor 状态: 需要 JwtConfig
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<JwtConfig>: axum::extract::FromRef<S>,
{
    type Rejection = BearerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. 提取 Authorization 头
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(BearerError::MissingHeader)?
            .to_str()
            .map_err(|_| BearerError::InvalidFormat)?;

        // 2. 解析 "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(BearerError::InvalidFormat)?
            .trim();
        if token.is_empty() {
            return Err(BearerError::InvalidFormat);
        }

        // 3. 提取 JwtConfig from state
        let jwt_config = Arc::<JwtConfig>::from_ref(state);

        // 4. 验签
        let claims = match verify_token(jwt_config.as_ref(), token) {
            Ok(c) => c,
            Err(crate::auth::JwtError::Expired) => return Err(BearerError::ExpiredToken),
            Err(crate::auth::JwtError::Missing) => return Err(BearerError::MissingHeader),
            Err(e) => return Err(BearerError::InvalidToken(format!("{:?}", e.code()))),
        };

        Ok(AuthenticatedUser(claims.to_auth_user()))
    }
}

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
    use uuid::Uuid;

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
        assert_eq!(
            BearerError::ServerError("x".to_string()).code(),
            "BEARER_SERVER_ERROR"
        );
    }

    #[test]
    fn bearer_error_retriable() {
        assert!(BearerError::ExpiredToken.is_retriable());
        assert!(!BearerError::MissingHeader.is_retriable());
        assert!(!BearerError::InvalidFormat.is_retriable());
        assert!(!BearerError::InvalidToken("x".to_string()).is_retriable());
        assert!(!BearerError::InsufficientScope("x".to_string()).is_retriable());
        assert!(!BearerError::ServerError("x".to_string()).is_retriable());
    }

    #[test]
    fn www_authenticate_header_per_rfc6750() {
        // per RFC 6750 §3 WWW-Authenticate 格式
        assert!(BearerError::MissingHeader
            .www_authenticate()
            .contains("Bearer"));
        assert!(BearerError::ExpiredToken
            .www_authenticate()
            .contains("invalid_token"));
        assert!(BearerError::InsufficientScope("admin".to_string())
            .www_authenticate()
            .contains("admin"));
    }

    #[test]
    fn bearer_error_into_response_status_codes() {
        // per RFC 6750 status mapping
        use axum::response::IntoResponse;
        let resp = BearerError::MissingHeader.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let resp = BearerError::InvalidFormat.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let resp = BearerError::InvalidToken("x".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let resp = BearerError::ExpiredToken.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let resp = BearerError::InsufficientScope("admin".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let resp = BearerError::ServerError("x".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

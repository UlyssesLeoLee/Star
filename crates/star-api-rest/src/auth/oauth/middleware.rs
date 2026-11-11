// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=467068513f0fc3e94d943ea0a02d3ab69042f743bc84307bbd1374b4a58bbd6d
MERGE (self:File {path:"crates/star-api-rest/src/auth/oauth/middleware.rs"})
MERGE (extractor:Type {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedDeviceContext",kind:"struct"})
MERGE (fromParts:Symbol {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedDeviceContext.from_request_parts",kind:"method"})
MERGE (authenticatedUser:Type {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedUser",kind:"struct"})
MERGE (userFromParts:Symbol {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedUser.from_request_parts",kind:"method"})
MERGE (deviceGuard:Symbol {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::requires_active_device_context",kind:"function"})
MERGE (claims:Type {id:"crates/star-api-rest/src/auth/mod.rs::Claims",kind:"struct"})
MERGE (device:Type {id:"crates/star-api-rest/src/auth/mod.rs::DeviceTokenContext",kind:"struct"})
MERGE (verify:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::verify_token",kind:"function"})
MERGE (claimsContext:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::Claims.device_context",kind:"method"})
MERGE (claimsUser:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::Claims.to_auth_user",kind:"method"})
MERGE (bindingRepo:Type {id:"star_pg_adapter::repository::DeviceBindingRepository",kind:"trait"})
MERGE (isActive:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceBindingRepository.is_active",kind:"method"})
MERGE (self)-[:DEFINES]->(extractor)
MERGE (extractor)-[:HAS_METHOD]->(fromParts)
MERGE (self)-[:DEFINES]->(authenticatedUser)
MERGE (authenticatedUser)-[:HAS_METHOD]->(userFromParts)
MERGE (fromParts)-[:CALLS]->(verify)
MERGE (fromParts)-[:CALLS]->(claimsContext)
MERGE (fromParts)-[:CALLS]->(claimsUser)
MERGE (extractor)-[:USES_TYPE]->(device)
MERGE (userFromParts)-[:CALLS]->(verify)
MERGE (userFromParts)-[:CALLS]->(claimsContext)
MERGE (userFromParts)-[:CALLS]->(deviceGuard)
MERGE (userFromParts)-[:CALLS]->(isActive)
MERGE (userFromParts)-[:USES_TYPE]->(bindingRepo)
@endcypher
*/
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
    http::{Method, StatusCode, header, request::Parts},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::sync::Arc;

use crate::auth::{AuthUser, DeviceTokenContext, JwtConfig, verify_token};
use star_pg_adapter::repository::DeviceBindingRepository;

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
            BearerError::MissingHeader => {
                format!(r#"Bearer realm="star-api-rest", error="invalid_request""#)
            }
            BearerError::InvalidFormat => {
                format!(r#"Bearer realm="star-api-rest", error="invalid_request""#)
            }
            BearerError::InvalidToken(_) => {
                format!(r#"Bearer realm="star-api-rest", error="invalid_token""#)
            }
            BearerError::ExpiredToken => format!(
                r#"Bearer realm="star-api-rest", error="invalid_token", error_description="token expired""#
            ),
            BearerError::InsufficientScope(scope) => format!(
                r#"Bearer realm="star-api-rest", error="insufficient_scope", scope="{}""#,
                scope
            ),
            BearerError::ServerError(_) => {
                format!(r#"Bearer realm="star-api-rest", error="server_error""#)
            }
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
        (status, [(header::WWW_AUTHENTICATE, www_auth)], Json(body)).into_response()
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

/// Issuer-authenticated actor and device generation from one verified JWT.
/// This extractor does not claim the generation is still current; callers that
/// authorize shared collaboration must re-check the identity owner before use.
#[derive(Debug, Clone)]
pub struct AuthenticatedDeviceContext {
    pub actor: AuthUser,
    pub device: DeviceTokenContext,
}

impl<S> FromRequestParts<S> for AuthenticatedDeviceContext
where
    S: Send + Sync,
    Arc<JwtConfig>: axum::extract::FromRef<S>,
{
    type Rejection = BearerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(BearerError::MissingHeader)?
            .to_str()
            .map_err(|_| BearerError::InvalidFormat)?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(BearerError::InvalidFormat)?
            .trim();
        if token.is_empty() {
            return Err(BearerError::InvalidFormat);
        }

        let jwt_config = Arc::<JwtConfig>::from_ref(state);
        let claims = match verify_token(jwt_config.as_ref(), token) {
            Ok(claims) => claims,
            Err(crate::auth::JwtError::Expired) => return Err(BearerError::ExpiredToken),
            Err(crate::auth::JwtError::Missing) => return Err(BearerError::MissingHeader),
            Err(error) => return Err(BearerError::InvalidToken(error.code().to_string())),
        };
        if claims.sub != claims.user_id || claims.sub.is_nil() || claims.tenant_id.is_nil() {
            return Err(BearerError::InvalidToken(
                "subject claims mismatch".to_string(),
            ));
        }
        let device = claims
            .device_context()
            .ok_or_else(|| BearerError::InvalidToken("device context required".to_string()))?;

        Ok(Self {
            actor: claims.to_auth_user(),
            device,
        })
    }
}

/// Axum extractor 状态: 需要 JwtConfig
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<JwtConfig>: axum::extract::FromRef<S>,
    Arc<dyn DeviceBindingRepository>: axum::extract::FromRef<S>,
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

        if claims.sub != claims.user_id || claims.sub.is_nil() || claims.tenant_id.is_nil() {
            return Err(BearerError::InvalidToken(
                "subject claims mismatch".to_string(),
            ));
        }

        let device_context = claims.device_context();
        if requires_active_device_context(&parts.method, parts.uri.path())
            && device_context.is_none()
        {
            return Err(BearerError::InvalidToken(
                "device context required for shared mutation".to_string(),
            ));
        }

        if let Some(device) = device_context {
            let device_binding_repo = Arc::<dyn DeviceBindingRepository>::from_ref(state);
            let active = device_binding_repo
                .is_active(
                    claims.tenant_id,
                    claims.user_id,
                    device.device_id,
                    device.generation,
                )
                .await
                .map_err(|_| BearerError::ServerError("device binding unavailable".to_string()))?;
            if !active {
                return Err(BearerError::InvalidToken(
                    "device binding revoked".to_string(),
                ));
            }
        }

        Ok(AuthenticatedUser(claims.to_auth_user()))
    }
}

/// Mutating shared API requests require a currently active device identity.
/// Device challenge creation and proof completion are exempted so a valid
/// account session can bootstrap its first device binding.
fn requires_active_device_context(method: &Method, path: &str) -> bool {
    let safe_method = method == Method::GET || method == Method::HEAD || method == Method::OPTIONS;
    let device_bootstrap = method == Method::POST
        && matches!(
            path,
            "/oauth/device/challenges" | "/oauth/device/enrollments"
        );
    !safe_method && !device_bootstrap
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
            email: None,
        };
        assert!(require_scope(&user, "read").is_ok());
        assert!(require_scope(&user, "write").is_ok());
        assert!(require_scope(&user, "admin").is_err());
    }

    #[test]
    fn legacy_sessions_cannot_mutate_shared_routes_but_can_bootstrap_device_identity() {
        assert!(!requires_active_device_context(
            &Method::GET,
            "/api/v1/group/tasks"
        ));
        assert!(requires_active_device_context(
            &Method::POST,
            "/api/v1/group/tasks"
        ));
        assert!(requires_active_device_context(
            &Method::PATCH,
            "/api/v1/group/tasks/wi-1"
        ));
        assert!(requires_active_device_context(
            &Method::DELETE,
            "/api/v1/group/tasks/wi-1"
        ));
        assert!(!requires_active_device_context(
            &Method::POST,
            "/oauth/device/challenges"
        ));
        assert!(!requires_active_device_context(
            &Method::POST,
            "/oauth/device/enrollments"
        ));
        assert!(requires_active_device_context(
            &Method::POST,
            "/oauth/device/00000000-0000-0000-0000-000000000001/revoke"
        ));
    }

    #[test]
    fn require_role_grants_matching_role() {
        let user = AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec!["player".to_string(), "ops".to_string()],
            scope: "".to_string(),
            email: None,
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
            email: None,
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
        assert!(
            BearerError::MissingHeader
                .www_authenticate()
                .contains("Bearer")
        );
        assert!(
            BearerError::ExpiredToken
                .www_authenticate()
                .contains("invalid_token")
        );
        assert!(
            BearerError::InsufficientScope("admin".to_string())
                .www_authenticate()
                .contains("admin")
        );
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

// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=aa38b418d8549c35fa815d51de0bb4dc3b5d862419f0156e661fd24f48fd14b3
MERGE (self:File {path:"crates/star-api-rest/src/auth/mod.rs"})
MERGE (claims:Type {id:"crates/star-api-rest/src/auth/mod.rs::Claims",kind:"struct"})
MERGE (device:Type {id:"crates/star-api-rest/src/auth/mod.rs::DeviceTokenContext",kind:"struct"})
MERGE (issue:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token_with_device_context",kind:"function"})
MERGE (context:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::Claims.device_context",kind:"method"})
MERGE (self)-[:DEFINES]->(claims)
MERGE (self)-[:DEFINES]->(device)
MERGE (self)-[:DEFINES]->(issue)
MERGE (self)-[:DEFINES]->(context)
MERGE (issue)-[:USES_TYPE]->(claims)
MERGE (issue)-[:USES_TYPE]->(device)
MERGE (context)-[:RETURNS_TYPE]->(device)
@endcypher
*/
//! `star-api-rest::auth` — OAuth2 + JWT 鉴权 middleware (per §14.12 IV)
//!
//! v0.41 端到端实装 (per 2026-09-09 19:20 JST 用户拍板"OAuth2 + JWT (推荐)"):
//! - RS256 签名 (per OAuth2 + JWT 主流选型, 跟 ADR-0030 Lease + Heartbeat 互补)
//! - Claims 12 字段 (sub / iss / aud / exp / iat / nbf + tenant_id / user_id / roles / scope / jti)
//! - 守门 #5 v2: PRIVATE_KEY / PUBLIC_KEY 走 env var, 不入 log / println
//! - 守门 #6 v2: TokenExpired 视为 retriable (refresh 流程)
//! - Axum middleware 走 `Authorization: Bearer <jwt>` 解析, 注入 `AuthUser` 到 request extension
//!
//! 跨 session 续: OAuth2 Authorization Code Flow + PKCE + refresh token rotation
//! 估 3-4M tokens, 1-2 session 续

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// v0.43 §14.12 IV OAuth2 server (per 2026-09-09 20:20 JST 用户拍板 both flows)
pub mod oauth;

/// JWT 错误 (4-variant, 跟 v0.30 6-field 错误模型一致)
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("missing JWT env config: {0}")]
    Config(String),
    #[error("token expired")]
    Expired,
    #[error("invalid token: {0}")]
    Invalid(String),
    #[error("missing Authorization header")]
    Missing,
}

impl JwtError {
    pub fn code(&self) -> &'static str {
        match self {
            JwtError::Config(_) => "JWT_CONFIG_MISSING",
            JwtError::Expired => "JWT_TOKEN_EXPIRED",
            JwtError::Invalid(_) => "JWT_TOKEN_INVALID",
            JwtError::Missing => "JWT_HEADER_MISSING",
        }
    }

    /// 守门 #6 v2: Expired 视为 retriable (refresh 流程)
    pub fn is_retriable(&self) -> bool {
        matches!(self, JwtError::Expired)
    }
}

/// JWT Claims 12 字段 (per OAuth2 JWT standard + §13 multi-tenant)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// subject (用户 ID, UUID)
    pub sub: Uuid,
    /// issuer (per OAuth2 standard, e.g. "https://api.star.local")
    pub iss: String,
    /// audience (per OAuth2 standard, e.g. "star-api-rest")
    pub aud: String,
    /// expiration (Unix timestamp, per OAuth2 standard)
    pub exp: i64,
    /// issued at
    pub iat: i64,
    /// not before
    pub nbf: i64,
    /// JWT ID (unique per token, 用于黑名单 / 撤销)
    pub jti: Uuid,
    /// 租户 ID (per §13 multi-tenant, RLS 13 類必携)
    pub tenant_id: Uuid,
    /// 用户 ID (跟 sub 重复但显式, 方便 axum Extractor 解析)
    pub user_id: Uuid,
    /// 角色 (per 守门 #3 + ADR-0030)
    pub roles: Vec<String>,
    /// OAuth2 scope (e.g. "work-items:read work-items:write")
    pub scope: String,
    /// Server-verified device identity; absent for legacy and unbound sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<Uuid>,
    /// Monotonic device generation issued by the identity owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_generation: Option<i64>,
}

/// Device identity copied only from a successfully verified issuer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceTokenContext {
    pub device_id: Uuid,
    pub generation: i64,
}

/// JWT Config (RS256 keys 走 env var, 守门 #5 v2)
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// 私钥 (PEM format, 仅 issuer 持有, 守门 #5 v2 不入 log)
    pub private_key_pem: String,
    /// 公钥 (PEM format, 给 verifier 用)
    pub public_key_pem: String,
    /// issuer URL (e.g. "https://api.star.local")
    pub issuer: String,
    /// audience (e.g. "star-api-rest")
    pub audience: String,
    /// token 有效期 (默认 1 hour = 3600s)
    pub ttl_seconds: i64,
}

impl JwtConfig {
    /// 从 env var 构造 (守门 #5 v2: 不打印 key 内容)
    pub fn from_env() -> Result<Self, JwtError> {
        let private_key_pem = std::env::var("JWT_PRIVATE_KEY_PEM")
            .map_err(|_| JwtError::Config("JWT_PRIVATE_KEY_PEM env var not set".to_string()))?;
        let public_key_pem = std::env::var("JWT_PUBLIC_KEY_PEM")
            .map_err(|_| JwtError::Config("JWT_PUBLIC_KEY_PEM env var not set".to_string()))?;
        let issuer =
            std::env::var("JWT_ISSUER").unwrap_or_else(|_| "https://api.star.local".to_string());
        let audience =
            std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "star-api-rest".to_string());
        Ok(Self {
            private_key_pem,
            public_key_pem,
            issuer,
            audience,
            ttl_seconds: 3600,
        })
    }
}

impl JwtConfig {
    /// 从服务端 verifier 环境变量构造，不要求 REST API 持有 JWT 私钥。
    pub fn verifier_from_env() -> Result<Self, JwtError> {
        let public_key_pem = std::env::var("JWT_PUBLIC_KEY_PEM")
            .map_err(|_| JwtError::Config("JWT_PUBLIC_KEY_PEM env var not set".to_string()))?;
        let issuer =
            std::env::var("JWT_ISSUER").unwrap_or_else(|_| "https://api.star.local".to_string());
        let audience =
            std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "star-api-rest".to_string());
        Ok(Self {
            private_key_pem: String::new(),
            public_key_pem,
            issuer,
            audience,
            ttl_seconds: 3600,
        })
    }
}
/// 颁发 JWT (用私钥签)
pub fn issue_token(
    config: &JwtConfig,
    sub: Uuid,
    tenant_id: Uuid,
    roles: Vec<String>,
    scope: String,
) -> Result<(String, Claims), JwtError> {
    issue_token_with_device_context(config, sub, tenant_id, roles, scope, None)
}

/// Issue a JWT optionally bound to a device generation from the identity owner.
pub fn issue_token_with_device_context(
    config: &JwtConfig,
    sub: Uuid,
    tenant_id: Uuid,
    roles: Vec<String>,
    scope: String,
    device_context: Option<DeviceTokenContext>,
) -> Result<(String, Claims), JwtError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(config.ttl_seconds);
    let claims = Claims {
        sub,
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
        jti: Uuid::new_v4(),
        tenant_id,
        user_id: sub,
        roles,
        scope,
        device_id: device_context.map(|context| context.device_id),
        device_generation: device_context.map(|context| context.generation),
    };
    let header = Header::new(jsonwebtoken::Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(config.private_key_pem.as_bytes())
        .map_err(|e| JwtError::Config(format!("private key parse: {}", e)))?;
    let token = encode(&header, &claims, &encoding_key)
        .map_err(|e| JwtError::Invalid(format!("encode: {}", e)))?;
    Ok((token, claims))
}

/// 验证 JWT (用公钥验签)
pub fn verify_token(config: &JwtConfig, token: &str) -> Result<Claims, JwtError> {
    let decoding_key = DecodingKey::from_rsa_pem(config.public_key_pem.as_bytes())
        .map_err(|e| JwtError::Config(format!("public key parse: {}", e)))?;
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_issuer(&[config.issuer.clone()]);
    validation.set_audience(&[config.audience.clone()]);
    let token_data =
        decode::<Claims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
            _ => JwtError::Invalid(format!("decode: {}", e)),
        })?;
    Ok(token_data.claims)
}

/// Authenticated user (从 JWT claims 提取, 注入 axum request)
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub roles: Vec<String>,
    pub scope: String,
}

impl From<Claims> for AuthUser {
    fn from(c: Claims) -> Self {
        Self {
            user_id: c.user_id,
            tenant_id: c.tenant_id,
            roles: c.roles,
            scope: c.scope,
        }
    }
}

impl Claims {
    /// 转换为 AuthUser (给 axum Extractor 用)
    pub fn to_auth_user(&self) -> AuthUser {
        AuthUser {
            user_id: self.user_id,
            tenant_id: self.tenant_id,
            roles: self.roles.clone(),
            scope: self.scope.clone(),
        }
    }

    /// Return a well-formed device identity only when both signed claims exist.
    pub fn device_context(&self) -> Option<DeviceTokenContext> {
        match (self.device_id, self.device_generation) {
            (Some(device_id), Some(generation)) if !device_id.is_nil() && generation > 0 => {
                Some(DeviceTokenContext {
                    device_id,
                    generation,
                })
            }
            _ => None,
        }
    }

    /// 过期时间 (human-readable)
    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(Utc::now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

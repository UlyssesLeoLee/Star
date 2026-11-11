// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=2026-11-11-google-login-v1
MERGE (self:File {path:"crates/star-api-rest/src/auth/google.rs"})
MERGE (state:Type {id:"crates/star-api-rest/src/auth/google.rs::GoogleAuthState",kind:"struct"})
MERGE (login:Symbol {id:"crates/star-api-rest/src/auth/google.rs::login_handler",kind:"function"})
MERGE (me:Symbol {id:"crates/star-api-rest/src/auth/google.rs::me_handler",kind:"function"})
MERGE (admin:Symbol {id:"crates/star-api-rest/src/auth/google.rs::admin_status_handler",kind:"function"})
MERGE (verify:Symbol {id:"crates/star-api-rest/src/auth/google.rs::verify_google_id_token_with_jwks",kind:"function"})
MERGE (fetch:Symbol {id:"crates/star-api-rest/src/auth/google.rs::GoogleAuthState.jwks",kind:"method"})
MERGE (roles:Symbol {id:"crates/star-api-rest/src/auth/google.rs::roles_for_email",kind:"function"})
MERGE (extractor:Type {id:"crates/star-api-rest/src/auth/google.rs::GoogleUser",kind:"struct"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(login)
MERGE (self)-[:DEFINES]->(me)
MERGE (self)-[:DEFINES]->(admin)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(roles)
MERGE (self)-[:DEFINES]->(extractor)
MERGE (login)-[:CALLS]->(fetch)
MERGE (login)-[:CALLS]->(verify)
MERGE (login)-[:CALLS]->(roles)
MERGE (login)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_provider_token",kind:"function"})
MERGE (me)-[:USES_TYPE]->(extractor)
MERGE (admin)-[:USES_TYPE]->(extractor)
MERGE (admin)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::require_role",kind:"function"})
MERGE (verify)-[:READS]->(:ExternalService {id:"google.openid.jwks",kind:"oidc_jwks"})
MERGE (self)-[:IMPORTS]->(:File {path:"crates/star-api-rest/src/auth/mod.rs"})
@endcypher
*/
//! Google OIDC 登录 + 管理员后台 (per 2026-11-11 用户发令)
//!
//! - `POST /api/v1/auth/google` — 用 Google ID token 换 Star JWT
//!   (RS256 验签 Google 公钥 JWKS, `email_verified` 必须为 true)。
//! - `GET  /api/v1/auth/me`    — 当前 Star JWT 身份回显。
//! - `GET  /api/v1/admin/status` — 管理员后台状态 (role=admin 才可读)。
//!
//! 管理员判定: `STAR_ADMIN_EMAIL` (默认 `lidian727@gmail.com`) 与 Google
//! 已验证邮箱完全相等时, roles = ["admin","user"]; 其余 Google 用户为 ["user"]。
//! 配置: `GOOGLE_CLIENT_ID` (Google OAuth web client id, 未配置则登录端点 503),
//! `GOOGLE_JWKS_URL` (默认 https://www.googleapis.com/oauth2/v3/certs)。
//! 守门 #5: token / email 不入 log。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use thiserror::Error;
use uuid::Uuid;

use crate::auth::{AuthUser, JwtConfig, JwtError, issue_provider_token, verify_token};

/// Deterministic placeholder tenant for Google logins until tenant onboarding
/// exists (per WBS §23.24: tenant mapping is a follow-up lane).
const GOOGLE_TENANT_PLACEHOLDER: &str = "star.google.tenant.placeholder";

fn deterministic_uuid(name: &str) -> Uuid {
    let digest = sha2::Sha256::digest(name.as_bytes());
    Uuid::from_slice(&digest[..16]).expect("sha256 prefix is 16 bytes")
}

fn google_tenant_id() -> Uuid {
    deterministic_uuid(GOOGLE_TENANT_PLACEHOLDER)
}

fn google_subject_uuid(google_sub: &str) -> Uuid {
    deterministic_uuid(&format!("google:{}", google_sub))
}

const DEFAULT_GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const DEFAULT_ADMIN_EMAIL: &str = "lidian727@gmail.com";
const JWKS_CACHE_TTL: Duration = Duration::from_secs(600);

/// Google 登录服务状态 (axum state)。
#[derive(Clone)]
pub struct GoogleAuthState {
    pub jwt: Arc<JwtConfig>,
    /// Sole administrator email (role=admin gate, per 用户拍板 2026-11-11)。
    pub admin_email: String,
    /// Google OAuth web client id; `None` ⇒ login endpoint fails closed (503).
    pub google_client_id: Option<String>,
    jwks_url: String,
    http: reqwest::Client,
    jwks_cache: Arc<Mutex<Option<(Instant, GoogleJwks)>>>,
}

impl GoogleAuthState {
    /// 从 env 构造。`jwt` 必须持有签发私钥 (JWT_PRIVATE_KEY_PEM)。
    pub fn from_env(jwt: Arc<JwtConfig>) -> Self {
        Self {
            jwt,
            admin_email: std::env::var("STAR_ADMIN_EMAIL")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_ADMIN_EMAIL.to_string()),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            jwks_url: std::env::var("GOOGLE_JWKS_URL")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_GOOGLE_JWKS_URL.to_string()),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client build"),
            jwks_cache: Arc::new(Mutex::new(None)),
        }
    }

    /// 拉取 Google JWKS (带 10 分钟缓存)。
    async fn jwks(&self) -> Result<GoogleJwks, GoogleAuthError> {
        if let Some((fetched_at, cached)) = self.jwks_cache.lock().map(|c| c.clone()).ok().flatten()
        {
            if fetched_at.elapsed() < JWKS_CACHE_TTL {
                return Ok(cached);
            }
        }
        let jwks: GoogleJwks = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|_| GoogleAuthError::Unavailable("google jwks fetch failed".to_string()))?
            .error_for_status()
            .map_err(|_| GoogleAuthError::Unavailable("google jwks bad status".to_string()))?
            .json()
            .await
            .map_err(|_| GoogleAuthError::Unavailable("google jwks parse failed".to_string()))?;
        let _ = self
            .jwks_cache
            .lock()
            .map(|mut cache| *cache = Some((Instant::now(), jwks.clone())));
        Ok(jwks)
    }
}

impl axum::extract::FromRef<GoogleAuthState> for Arc<JwtConfig> {
    fn from_ref(state: &GoogleAuthState) -> Self {
        Arc::clone(&state.jwt)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleJwks {
    keys: Vec<GoogleJwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleJwk {
    kty: String,
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleIdTokenPayload {
    iss: String,
    sub: String,
    email: String,
    email_verified: bool,
    #[serde(default)]
    name: Option<String>,
    aud: String,
    #[serde(default)]
    exp: i64,
    #[serde(default)]
    iat: i64,
}

#[derive(Debug, Error)]
pub enum GoogleAuthError {
    #[error("google login not configured: {0}")]
    Config(String),
    #[error("google verification unavailable: {0}")]
    Unavailable(String),
    #[error("invalid google id token: {0}")]
    InvalidToken(String),
    #[error("star jwt signing unavailable: {0}")]
    Signing(String),
}

impl GoogleAuthError {
    pub fn code(&self) -> &'static str {
        match self {
            GoogleAuthError::Config(_) => "GOOGLE_AUTH_NOT_CONFIGURED",
            GoogleAuthError::Unavailable(_) => "GOOGLE_AUTH_UNAVAILABLE",
            GoogleAuthError::InvalidToken(_) => "GOOGLE_TOKEN_INVALID",
            GoogleAuthError::Signing(_) => "JWT_SIGNING_UNAVAILABLE",
        }
    }
}

impl IntoResponse for GoogleAuthError {
    fn into_response(self) -> Response {
        let status = match self {
            GoogleAuthError::Config(_) => StatusCode::SERVICE_UNAVAILABLE,
            GoogleAuthError::Unavailable(_) => StatusCode::BAD_GATEWAY,
            GoogleAuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            GoogleAuthError::Signing(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, Json(json!({ "error": self.code() }))).into_response()
    }
}

/// 纯函数: 用给定 JWKS 验 Google ID token (audience 校验在 configured 时强制)。
fn verify_google_id_token_with_jwks(
    jwks: &GoogleJwks,
    client_id: Option<&str>,
    token: &str,
) -> Result<GoogleIdTokenPayload, GoogleAuthError> {
    let header = decode_header(token)
        .map_err(|_| GoogleAuthError::InvalidToken("unreadable header".to_string()))?;
    let kid = header
        .kid
        .ok_or_else(|| GoogleAuthError::InvalidToken("missing kid".to_string()))?;
    let jwk = jwks
        .keys
        .iter()
        .find(|key| key.kid == kid && key.kty == "RSA")
        .ok_or_else(|| GoogleAuthError::InvalidToken("unknown kid".to_string()))?;
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|_| GoogleAuthError::InvalidToken("bad rsa components".to_string()))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[
        "https://accounts.google.com",
        "accounts.google.com",
    ]);
    if let Some(audience) = client_id {
        validation.set_audience(&[audience]);
    }

    let payload = decode::<GoogleIdTokenPayload>(token, &decoding_key, &validation)
        .map_err(|_| GoogleAuthError::InvalidToken("signature or claims invalid".to_string()))?
        .claims;
    if !payload.email_verified {
        return Err(GoogleAuthError::InvalidToken(
            "email not verified".to_string(),
        ));
    }
    if payload.sub.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(GoogleAuthError::InvalidToken(
            "empty subject or email".to_string(),
        ));
    }
    Ok(payload)
}

/// 角色映射: 管理员邮箱完全匹配 ⇒ admin + user; 否则 user。
fn roles_for_email(email: &str, admin_email: &str) -> Vec<String> {
    if email.eq_ignore_ascii_case(admin_email) {
        vec!["admin".to_string(), "user".to_string()]
    } else {
        vec!["user".to_string()]
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    pub id_token: String,
}

#[derive(Debug, Serialize)]
pub struct GoogleLoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: GoogleUserInfo,
}

#[derive(Debug, Serialize)]
pub struct GoogleUserInfo {
    pub email: String,
    pub name: Option<String>,
    pub is_admin: bool,
    pub roles: Vec<String>,
}

/// `POST /api/v1/auth/google` — Google ID token ⇒ Star JWT。
pub async fn login_handler(
    State(state): State<GoogleAuthState>,
    Json(request): Json<GoogleLoginRequest>,
) -> Result<Json<GoogleLoginResponse>, GoogleAuthError> {
    let client_id = state
        .google_client_id
        .clone()
        .ok_or_else(|| GoogleAuthError::Config("GOOGLE_CLIENT_ID not set".to_string()))?;
    let jwks = state.jwks().await?;
    let payload = verify_google_id_token_with_jwks(&jwks, Some(&client_id), &request.id_token)?;

    let roles = roles_for_email(&payload.email, &state.admin_email);
    let is_admin = roles.iter().any(|role| role == "admin");
    let scope = "worktree:read worktree:write".to_string();
    let (access_token, _claims) = issue_provider_token(
        &state.jwt,
        google_subject_uuid(&payload.sub),
        google_tenant_id(),
        roles.clone(),
        scope,
        payload.email.clone(),
    )
    .map_err(|error| match error {
        JwtError::Config(detail) => GoogleAuthError::Signing(detail),
        other => GoogleAuthError::Signing(other.code().to_string()),
    })?;

    Ok(Json(GoogleLoginResponse {
        access_token,
        token_type: "Bearer",
        expires_in: state.jwt.ttl_seconds,
        user: GoogleUserInfo {
            email: payload.email,
            name: payload.name,
            is_admin,
            roles,
        },
    }))
}

/// 轻量 extractor: 验证 Star JWT 并注入 AuthUser (Google 会话无 device context)。
pub struct GoogleUser(pub AuthUser);

impl<S> FromRequestParts<S> for GoogleUser
where
    S: Send + Sync,
    Arc<JwtConfig>: FromRef<S>,
{
    type Rejection = BearerReject;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(BearerReject::MissingHeader)?
            .to_str()
            .map_err(|_| BearerReject::InvalidFormat)?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(BearerReject::InvalidFormat)?
            .trim();
        if token.is_empty() {
            return Err(BearerReject::InvalidFormat);
        }
        let jwt_config = Arc::<JwtConfig>::from_ref(state);
        let claims = verify_token(jwt_config.as_ref(), token).map_err(|error| match error {
            JwtError::Expired => BearerReject::Expired,
            _ => BearerReject::Invalid,
        })?;
        if claims.sub != claims.user_id || claims.sub.is_nil() || claims.tenant_id.is_nil() {
            return Err(BearerReject::Invalid);
        }
        Ok(GoogleUser(claims.to_auth_user()))
    }
}

#[derive(Debug, Error)]
pub enum BearerReject {
    #[error("missing Authorization header")]
    MissingHeader,
    #[error("invalid Authorization header format")]
    InvalidFormat,
    #[error("token expired")]
    Expired,
    #[error("invalid token")]
    Invalid,
    #[error("insufficient role")]
    InsufficientRole,
}

impl IntoResponse for BearerReject {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            BearerReject::MissingHeader => (StatusCode::UNAUTHORIZED, "BEARER_MISSING_HEADER"),
            BearerReject::InvalidFormat => (StatusCode::UNAUTHORIZED, "BEARER_INVALID_FORMAT"),
            BearerReject::Expired => (StatusCode::UNAUTHORIZED, "BEARER_TOKEN_EXPIRED"),
            BearerReject::Invalid => (StatusCode::UNAUTHORIZED, "BEARER_INVALID_TOKEN"),
            BearerReject::InsufficientRole => (StatusCode::FORBIDDEN, "BEARER_INSUFFICIENT_ROLE"),
        };
        (status, Json(json!({ "error": code }))).into_response()
    }
}

/// `GET /api/v1/auth/me` — 当前身份回显。
pub async fn me_handler(GoogleUser(user): GoogleUser) -> Json<serde_json::Value> {
    Json(json!({
        "user_id": user.user_id,
        "tenant_id": user.tenant_id,
        "email": user.email,
        "roles": user.roles,
        "is_admin": user.roles.iter().any(|role| role == "admin"),
    }))
}

/// `GET /api/v1/admin/status` — 仅 role=admin。
pub async fn admin_status_handler(
    State(state): State<GoogleAuthState>,
    GoogleUser(user): GoogleUser,
) -> Result<Json<serde_json::Value>, BearerReject> {
    crate::auth::oauth::middleware::require_role(&user, "admin")
        .map_err(|_| BearerReject::InsufficientRole)?;
    Ok(Json(json!({
        "server_time": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "admin": {
            "email": state.admin_email,
            "google_client_configured": state.google_client_id.is_some(),
        },
        "auth": {
            "provider": "google-oidc",
            "issuer": state.jwt.issuer,
            "audience": state.jwt.audience,
            "token_ttl_seconds": state.jwt.ttl_seconds,
        },
    })))
}

pub fn router(state: GoogleAuthState) -> Router {
    Router::new()
        .route("/api/v1/auth/google", post(login_handler))
        .route("/api/v1/auth/me", get(me_handler))
        .route("/api/v1/admin/status", get(admin_status_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header};
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};
    use rsa::traits::PublicKeyParts;
    use rsa::{RsaPrivateKey, RsaPublicKey};

    fn test_rsa_keypair() -> (RsaPrivateKey, RsaPublicKey) {
        let mut rng = rand_core::OsRng;
        let private = RsaPrivateKey::new(&mut rng, 2048).expect("rsa keygen");
        let public = RsaPublicKey::from(&private);
        (private, public)
    }

    fn jwks_for(public: &RsaPublicKey, kid: &str) -> GoogleJwks {
        let n = URL_SAFE_NO_PAD.encode(public.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public.e().to_bytes_be());
        GoogleJwks {
            keys: vec![GoogleJwk {
                kty: "RSA".to_string(),
                kid: kid.to_string(),
                n,
                e,
            }],
        }
    }

    fn sign_test_id_token(
        private: &RsaPrivateKey,
        kid: &str,
        payload: &serde_json::Value,
    ) -> String {
        let pem = private
            .to_pkcs8_pem(LineEnding::LF)
            .expect("pkcs8 pem");
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        jsonwebtoken::encode(
            &header,
            &serde_json::to_value(payload).expect("payload"),
            &EncodingKey::from_rsa_pem(pem.as_bytes()).expect("encoding key"),
        )
        .expect("encode")
    }

    fn base_payload(aud: &str) -> serde_json::Value {
        serde_json::json!({
            "iss": "https://accounts.google.com",
            "sub": "google-subject-123",
            "email": "lidian727@gmail.com",
            "email_verified": true,
            "name": "Lidian",
            "aud": aud,
            "exp": chrono::Utc::now().timestamp() + 3600,
            "iat": chrono::Utc::now().timestamp(),
        })
    }

    #[test]
    fn valid_google_token_verifies_and_returns_identity() {
        let (private, public) = test_rsa_keypair();
        let jwks = jwks_for(&public, "kid-1");
        let token = sign_test_id_token(&private, "kid-1", &base_payload("test-client.apps.googleusercontent.com"));
        let payload = verify_google_id_token_with_jwks(
            &jwks,
            Some("test-client.apps.googleusercontent.com"),
            &token,
        )
        .expect("valid token");
        assert_eq!(payload.email, "lidian727@gmail.com");
        assert_eq!(payload.sub, "google-subject-123");
    }

    #[test]
    fn wrong_audience_is_rejected() {
        let (private, public) = test_rsa_keypair();
        let jwks = jwks_for(&public, "kid-1");
        let token = sign_test_id_token(&private, "kid-1", &base_payload("other-client"));
        assert!(verify_google_id_token_with_jwks(
            &jwks,
            Some("test-client.apps.googleusercontent.com"),
            &token,
        )
        .is_err());
    }

    #[test]
    fn unverified_email_is_rejected() {
        let (private, public) = test_rsa_keypair();
        let jwks = jwks_for(&public, "kid-1");
        let mut payload = base_payload("test-client.apps.googleusercontent.com");
        payload["email_verified"] = serde_json::json!(false);
        let token = sign_test_id_token(&private, "kid-1", &payload);
        assert!(verify_google_id_token_with_jwks(
            &jwks,
            Some("test-client.apps.googleusercontent.com"),
            &token,
        )
        .is_err());
    }

    #[test]
    fn unknown_kid_is_rejected() {
        let (private, public) = test_rsa_keypair();
        let jwks = jwks_for(&public, "kid-1");
        let token = sign_test_id_token(&private, "kid-2", &base_payload("test-client.apps.googleusercontent.com"));
        assert!(verify_google_id_token_with_jwks(
            &jwks,
            Some("test-client.apps.googleusercontent.com"),
            &token,
        )
        .is_err());
    }

    #[test]
    fn admin_email_maps_to_admin_role() {
        let roles = roles_for_email("lidian727@gmail.com", "lidian727@gmail.com");
        assert!(roles.contains(&"admin".to_string()));
        assert!(roles.contains(&"user".to_string()));

        let roles = roles_for_email("lidian727@gmail.com", "lidian727@GMAIL.com");
        assert!(roles.contains(&"admin".to_string()));

        let roles = roles_for_email("someone@gmail.com", "lidian727@gmail.com");
        assert!(!roles.contains(&"admin".to_string()));
        assert_eq!(roles, vec!["user".to_string()]);
    }

    #[test]
    fn google_subject_is_stable_across_tokens() {
        let first = google_subject_uuid("google-subject-123");
        let second = google_subject_uuid("google-subject-123");
        let other = google_subject_uuid("google-subject-456");
        assert_eq!(first, second);
        assert_ne!(first, other);
    }

    #[test]
    fn bearer_missing_header_rejects() {
        assert!(matches!(BearerReject::MissingHeader, BearerReject::MissingHeader));
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 HTTP handlers (per WBS v0.43 §14.12 IV OAuth2 server phase 2)
//!
//! Endpoints (per RFC 6749 + RFC 7662 + RFC 7009 + RFC 7636):
//! - GET  /oauth/authorize       - Authorization Code flow (stub for v0.43, 真实跨 session 续)
//! - POST /oauth/token           - Code exchange + Client Credentials grant
//! - GET  /.well-known/jwks.json - Public key (JWKS)
//! - POST /oauth/introspect      - Token introspection (RFC 7662)
//! - POST /oauth/revoke          - Token revocation (RFC 7009)
//!
//! 守门 #5 v2: tokens 不入 log
//! 守门 #6 v2: invalid_grant / invalid_client 走 4xx (不 retriable)
//! 守门 #14 v3: Mavis 永久代签

use axum::{
    extract::{Form, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
};
use base64::Engine;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use star_pg_adapter::repository::{
    OAuthAccessToken, OAuthAccessTokenRepository, OAuthAuthorizationCode,
    OAuthAuthorizationCodeRepository, OAuthClientRepository, OAuthRefreshToken,
    OAuthRefreshTokenRepository, PgOAuthAccessTokenRepository, PgOAuthAuthorizationCodeRepository,
    PgOAuthClientRepository, PgOAuthRefreshTokenRepository,
};
use std::sync::Arc;
use uuid::Uuid;

use super::keypair::OAuthKeyManager;
use super::pkce;
use crate::auth::{issue_token, JwtConfig};

/// OAuth2 state (axum extension)
#[derive(Clone)]
pub struct OAuth2State {
    pub key_manager: Arc<OAuthKeyManager>,
    pub jwt_config: Arc<JwtConfig>,
    pub client_repo: Arc<PgOAuthClientRepository>,
    pub auth_code_repo: Arc<PgOAuthAuthorizationCodeRepository>,
    pub access_token_repo: Arc<PgOAuthAccessTokenRepository>,
    pub refresh_token_repo: Arc<PgOAuthRefreshTokenRepository>,
}

// ============================================
// /oauth/authorize (Authorization Code flow, 简化版)
// ============================================

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub response_type: String,        // "code"
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: Option<String>, // "S256" (default) or "plain"
}

/// GET /oauth/authorize - v0.43 简化版: 不实现 user 登录/consent, 直接返回 auth code
/// 真实 user login flow 跨 session 续
pub async fn authorize_handler(
    State(state): State<OAuth2State>,
    Query(params): Query<AuthorizeQuery>,
) -> Result<impl IntoResponse, OAuthHandlerError> {
    // 验证 response_type
    if params.response_type != "code" {
        return Err(OAuthHandlerError::InvalidRequest(
            "response_type must be 'code'".to_string(),
        ));
    }

    // 查找 client
    let client = state
        .client_repo
        .find_by_client_id(Uuid::nil(), &params.client_id)  // 简化: tenant_id=nil
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("find client: {}", e)))?
        .ok_or_else(|| OAuthHandlerError::InvalidClient(format!("client_id {}", params.client_id)))?;

    // 验证 redirect_uri (简化: 单 URI)
    if !client.redirect_uris.contains(&params.redirect_uri) {
        return Err(OAuthHandlerError::InvalidRequest(format!(
            "redirect_uri {} not in registered list",
            params.redirect_uri
        )));
    }

    // 生成 auth code
    let code = generate_random_token(32);
    let code_hash = sha256_hex(&code);
    let user_id = Uuid::nil(); // v0.43 简化: 真实 user_id 跨 session 续
    let code_method = params
        .code_challenge_method
        .as_deref()
        .unwrap_or("S256")
        .to_string();

    let auth_code = OAuthAuthorizationCode {
        id: Uuid::new_v4(),
        tenant_id: client.tenant_id,
        code_hash: code_hash.clone(),
        client_id: client.client_id.clone(),
        user_id,
        redirect_uri: params.redirect_uri.clone(),
        scope: params.scope.clone().unwrap_or_default(),
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: code_method,
        expires_at: Utc::now() + Duration::minutes(10),
        consumed_at: None,
        created_at: Utc::now(),
        source_module: "star_api_rest::auth::oauth".to_string(),
        source_kind: "audit".to_string(),
        role_id: None,
        permission_id: None,
        policy_id: None,
        workspace_id: None,
        project_id: None,
        work_item_id: None,
        agent_id: None,
        session_id: None,
        trace_id: None,
    };

    state
        .auth_code_repo
        .insert(&auth_code)
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("insert auth code: {}", e)))?;

    // 返回 redirect
    let mut redirect_url = format!("{}?code={}", params.redirect_uri, code);
    if let Some(s) = &params.state {
        redirect_url.push_str(&format!("&state={}", s));
    }

    Ok(Redirect::to(&redirect_url))
}

// ============================================
// /oauth/token
// ============================================

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,           // "authorization_code" / "client_credentials" / "refresh_token"
    pub code: Option<String>,         // for authorization_code
    pub redirect_uri: Option<String>, // for authorization_code
    pub code_verifier: Option<String>, // PKCE
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scope: Option<String>,        // for client_credentials
    pub refresh_token: Option<String>, // for refresh_token
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,           // "Bearer"
    pub expires_in: i64,              // seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BasicAuth {
    pub username: String,  // client_id
    pub password: String,  // client_secret
}

/// POST /oauth/token
pub async fn token_handler(
    State(state): State<OAuth2State>,
    Form(req): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, OAuthHandlerError> {
    match req.grant_type.as_str() {
        "authorization_code" => handle_authorization_code(state, req).await,
        "client_credentials" => handle_client_credentials(state, req).await,
        "refresh_token" => handle_refresh_token(state, req).await,
        other => Err(OAuthHandlerError::UnsupportedGrantType(other.to_string())),
    }
}

async fn handle_authorization_code(
    state: OAuth2State,
    req: TokenRequest,
) -> Result<Json<TokenResponse>, OAuthHandlerError> {
    let code = req
        .code
        .as_ref()
        .ok_or_else(|| OAuthHandlerError::InvalidRequest("code required".to_string()))?;
    let code_verifier = req
        .code_verifier
        .as_ref()
        .ok_or_else(|| OAuthHandlerError::InvalidRequest("code_verifier required".to_string()))?;
    let client_id = req
        .client_id
        .as_ref()
        .ok_or_else(|| OAuthHandlerError::InvalidClient("client_id required".to_string()))?;

    // 查找 auth code (by hash)
    let code_hash = sha256_hex(code);
    let auth_code = state
        .auth_code_repo
        .find_by_code_hash(Uuid::nil(), &code_hash)
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("find auth code: {}", e)))?
        .ok_or_else(|| OAuthHandlerError::InvalidGrant("code not found or expired".to_string()))?;

    // 验证 client
    if auth_code.client_id != *client_id {
        return Err(OAuthHandlerError::InvalidClient(format!(
            "client_id {} mismatch",
            client_id
        )));
    }

    // 验证 PKCE
    pkce::verify_pkce(
        code_verifier,
        &auth_code.code_challenge,
        &auth_code.code_challenge_method,
    )
    .map_err(|e| OAuthHandlerError::InvalidGrant(format!("PKCE: {}", e)))?;

    // 标记 code consumed
    state
        .auth_code_repo
        .mark_consumed(Uuid::nil(), &code_hash, Utc::now())
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("mark consumed: {}", e)))?;

    // 颁发 access + refresh token
    issue_tokens(
        &state,
        &auth_code.client_id,
        Some(auth_code.user_id),
        auth_code.tenant_id,
        &auth_code.scope,
        "authorization_code",
    )
    .await
}

async fn handle_client_credentials(
    state: OAuth2State,
    req: TokenRequest,
) -> Result<Json<TokenResponse>, OAuthHandlerError> {
    let client_id = req
        .client_id
        .as_ref()
        .ok_or_else(|| OAuthHandlerError::InvalidClient("client_id required".to_string()))?;

    // 查找 client
    let client = state
        .client_repo
        .find_by_client_id(Uuid::nil(), client_id)
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("find client: {}", e)))?
        .ok_or_else(|| OAuthHandlerError::InvalidClient(format!("client_id {}", client_id)))?;

    if client.client_type != "confidential" {
        return Err(OAuthHandlerError::UnauthorizedClient(
            "client_credentials requires confidential client".to_string(),
        ));
    }

    // 验证 client_secret (v0.43 简化: hash 比对 placeholder, 真实 bcrypt/argon2 跨 session 续)
    let _secret = req
        .client_secret
        .as_ref()
        .ok_or_else(|| OAuthHandlerError::InvalidClient("client_secret required".to_string()))?;

    // 颁发 access token (no refresh token for client_credentials, per RFC 6749 §4.4.3)
    let scope = req.scope.unwrap_or_default();
    issue_tokens(
        &state,
        &client.client_id,
        None, // client_credentials 无 user
        client.tenant_id,
        &scope,
        "client_credentials",
    )
    .await
    .map(|mut resp| {
        resp.refresh_token = None;
        resp
    })
}

async fn handle_refresh_token(
    state: OAuth2State,
    req: TokenRequest,
) -> Result<Json<TokenResponse>, OAuthHandlerError> {
    let refresh_token = req
        .refresh_token
        .as_ref()
        .ok_or_else(|| OAuthHandlerError::InvalidRequest("refresh_token required".to_string()))?;
    let token_hash = sha256_hex(refresh_token);

    let rt = state
        .refresh_token_repo
        .find_by_token_hash(Uuid::nil(), &token_hash)
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("find refresh: {}", e)))?
        .ok_or_else(|| OAuthHandlerError::InvalidGrant("refresh_token invalid".to_string()))?;

    // 撤销旧 refresh token (rotation)
    state
        .refresh_token_repo
        .revoke(Uuid::nil(), &token_hash, Utc::now())
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("revoke old refresh: {}", e)))?;

    // 颁发新 token 对
    issue_tokens(
        &state,
        &rt.client_id,
        Some(rt.user_id),
        rt.tenant_id,
        &rt.scope,
        "refresh_token",
    )
    .await
}

/// 颁发 access + refresh token (共享 helper)
async fn issue_tokens(
    state: &OAuth2State,
    client_id: &str,
    user_id: Option<Uuid>,
    tenant_id: Uuid,
    scope: &str,
    grant_type: &str,
) -> Result<Json<TokenResponse>, OAuthHandlerError> {
    // sub: user_id (auth code / refresh) or client_id-hash-as-uuid (client_credentials)
    // v0.43 简化: client_credentials 走 Uuid::nil() 作 placeholder
    let sub_uuid = user_id.unwrap_or_else(Uuid::nil);
    let user_uuid = user_id.unwrap_or_else(Uuid::nil);

    // 签 JWT (走 JwtConfig 直接, 不通过 key_manager)
    let (access_token, _claims) = issue_token(
        &state.jwt_config,
        sub_uuid,
        tenant_id,
        vec!["user".to_string()],
        scope.to_string(),
    )
    .map_err(|e| OAuthHandlerError::Internal(format!("issue_token: {}", e)))?;

    // claims 解出来, 取 jti
    let claims = crate::auth::verify_token(&state.jwt_config, &access_token)
        .map_err(|e| OAuthHandlerError::Internal(format!("verify (decode jti): {:?}", e)))?;

    // 存 access_token (token_hash = sha256(jti))
    let access_exp = Utc::now() + Duration::hours(1);
    let refresh_exp = Utc::now() + Duration::days(30);
    let access_token_id = Uuid::new_v4();
    let access_token_row = OAuthAccessToken {
        id: access_token_id,
        tenant_id,
        token_hash: sha256_hex(&claims.jti.to_string()),
        client_id: client_id.to_string(),
        user_id: if user_uuid.is_nil() { None } else { Some(user_uuid) },
        grant_type: grant_type.to_string(),
        scope: scope.to_string(),
        expires_at: access_exp,
        revoked_at: None,
        created_at: Utc::now(),
        source_module: "star_api_rest::auth::oauth".to_string(),
        source_kind: "audit".to_string(),
        role_id: None,
        permission_id: None,
        policy_id: None,
        workspace_id: None,
        project_id: None,
        work_item_id: None,
        agent_id: None,
        session_id: None,
        trace_id: None,
    };
    state
        .access_token_repo
        .insert(&access_token_row)
        .await
        .map_err(|e| OAuthHandlerError::Internal(format!("insert access_token: {}", e)))?;

    // refresh token 仅 auth code / refresh token grant 颁发
    let refresh_token = if grant_type == "authorization_code" || grant_type == "refresh_token" {
        let rt_value = generate_random_token(48);
        let rt_hash = sha256_hex(&rt_value);
        let refresh_token_id = Uuid::new_v4();
        let rt_row = OAuthRefreshToken {
            id: refresh_token_id,
            tenant_id,
            token_hash: rt_hash,
            client_id: client_id.to_string(),
            user_id: user_uuid,
            access_token_id,
            scope: scope.to_string(),
            expires_at: refresh_exp,
            revoked_at: None,
            created_at: Utc::now(),
            source_module: "star_api_rest::auth::oauth".to_string(),
            source_kind: "audit".to_string(),
            role_id: None,
            permission_id: None,
            policy_id: None,
            workspace_id: None,
            project_id: None,
            work_item_id: None,
            agent_id: None,
            session_id: None,
            trace_id: None,
        };
        state
            .refresh_token_repo
            .insert(&rt_row)
            .await
            .map_err(|e| OAuthHandlerError::Internal(format!("insert refresh: {}", e)))?;
        Some(rt_value)
    } else {
        None
    };

    let _ = sub_uuid; // 避免 unused warning

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token,
        scope: if scope.is_empty() {
            None
        } else {
            Some(scope.to_string())
        },
    }))
}

// ============================================
// /.well-known/jwks.json
// ============================================

/// GET /.well-known/jwks.json
pub async fn jwks_handler(
    State(state): State<OAuth2State>,
) -> Result<Json<serde_json::Value>, OAuthHandlerError> {
    let kp = state.key_manager.get().await;
    let jwks = kp
        .to_jwks()
        .map_err(|e| OAuthHandlerError::Internal(format!("jwks: {}", e)))?;
    Ok(Json(serde_json::to_value(jwks).unwrap()))
}

// ============================================
// /oauth/introspect (RFC 7662)
// ============================================

#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
    #[serde(default)]
    pub token_type_hint: Option<String>, // "access_token" or "refresh_token"
}

#[derive(Debug, Serialize)]
pub struct IntrospectResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

pub async fn introspect_handler(
    State(_state): State<OAuth2State>,
    Form(req): Form<IntrospectRequest>,
) -> Result<Json<IntrospectResponse>, OAuthHandlerError> {
    // v0.43 简化: 不实际 introspect DB, 仅 decode JWT 验证
    // 真实 DB lookup 跨 session 续
    let _ = req;
    Err(OAuthHandlerError::NotImplemented(
        "introspect 跨 session 续 (decode + DB lookup)".to_string(),
    ))
}

// ============================================
// /oauth/revoke (RFC 7009)
// ============================================

#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    pub token: String,
    #[serde(default)]
    pub token_type_hint: Option<String>,
}

pub async fn revoke_handler(
    State(state): State<OAuth2State>,
    Form(req): Form<RevokeRequest>,
) -> Result<StatusCode, OAuthHandlerError> {
    let token_hash = sha256_hex(&req.token);

    // 尝试撤销 access token
    let _ = state
        .access_token_repo
        .revoke(Uuid::nil(), &token_hash, Utc::now())
        .await;

    // 尝试撤销 refresh token
    let _ = state
        .refresh_token_repo
        .revoke(Uuid::nil(), &token_hash, Utc::now())
        .await;

    Ok(StatusCode::OK)
}

// ============================================
// 辅助函数
// ============================================

fn generate_random_token(byte_len: usize) -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; byte_len];
    rng.fill(&mut bytes).expect("rng");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

// ============================================
// Error type
// ============================================

#[derive(Debug)]
pub enum OAuthHandlerError {
    InvalidRequest(String),
    InvalidClient(String),
    InvalidGrant(String),
    UnauthorizedClient(String),
    UnsupportedGrantType(String),
    NotImplemented(String),
    Internal(String),
}

impl IntoResponse for OAuthHandlerError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, description) = match self {
            OAuthHandlerError::InvalidRequest(d) => {
                (StatusCode::BAD_REQUEST, "invalid_request", d)
            }
            OAuthHandlerError::InvalidClient(d) => {
                (StatusCode::UNAUTHORIZED, "invalid_client", d)
            }
            OAuthHandlerError::InvalidGrant(d) => {
                (StatusCode::BAD_REQUEST, "invalid_grant", d)
            }
            OAuthHandlerError::UnauthorizedClient(d) => {
                (StatusCode::BAD_REQUEST, "unauthorized_client", d)
            }
            OAuthHandlerError::UnsupportedGrantType(d) => {
                (StatusCode::BAD_REQUEST, "unsupported_grant_type", d)
            }
            OAuthHandlerError::NotImplemented(d) => {
                (StatusCode::NOT_IMPLEMENTED, "not_implemented", d)
            }
            OAuthHandlerError::Internal(d) => {
                // 守门 #5 v2: 不打印 internal error 内容到 response
                tracing::error!("OAuth2 internal error: {}", d);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "internal error".to_string(),
                )
            }
        };
        let body = json!({
            "error": error,
            "error_description": description,
        });
        (status, Json(body)).into_response()
    }
}

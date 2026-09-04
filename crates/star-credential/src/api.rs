//! crates/star-credential/src/api.rs
//!
//! V2-2 凭证管理 REST API (axum 0.8)
//! per 用户 UI 自填 → 后端 API 接收 → 加密存储 (per 守门 #5 + 守门 #14 + 守门 #DB-13)
//!
//! 4 endpoint:
//! - GET  /api/v2/credentials?provider=openclaw   (列出, 不含密文)
//! - POST /api/v2/credentials                   (创建, 接收明文)
//! - POST /api/v2/credentials/{id}/rotate        (轮换)
//! - POST /api/v2/credentials/{id}/revoke        (撤销)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    CredentialError, CredentialManager, CredentialMetadata, CredentialPlaintext, CredentialRecord,
    Provider,
};

/// AppState (依赖注入: CredentialManager + 当前 tenant_id + 当前 user_id)
#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<CredentialManager>,
    pub current_tenant_id: String, // 实际从 JWT/session 提取, PoC 用 header
    pub current_user_id: String,
}

impl AppState {
    pub fn new(manager: Arc<CredentialManager>, tenant_id: String, user_id: String) -> Self {
        Self { manager, current_tenant_id: tenant_id, current_user_id: user_id }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v2/credentials", get(list_credentials).post(create_credential))
        .route("/api/v2/credentials/:id/rotate", post(rotate_credential))
        .route("/api/v2/credentials/:id/revoke", post(revoke_credential))
}

// === Request / Response DTOs ===

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCredentialRequest {
    pub provider: String,
    pub display_name: String,
    pub description: String,
    pub secret: String,
    pub base_url: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CredentialView {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub status: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub deprecated_at_ms: Option<u64>,
    pub revoked_at_ms: Option<u64>,
}

impl From<CredentialRecord> for CredentialView {
    fn from(r: CredentialRecord) -> Self {
        Self {
            id: r.id,
            provider: r.provider.as_str().to_string(),
            display_name: r.metadata.display_name,
            status: status_to_str(r.status).to_string(),
            created_at_ms: r.created_at_ms,
            updated_at_ms: r.updated_at_ms,
            deprecated_at_ms: r.deprecated_at_ms,
            revoked_at_ms: r.revoked_at_ms,
        }
    }
}

fn status_to_str(s: crate::CredentialStatus) -> &'static str {
    match s {
        crate::CredentialStatus::Active => "active",
        crate::CredentialStatus::Deprecated => "deprecated",
        crate::CredentialStatus::Revoked => "revoked",
    }
}

fn parse_provider(s: &str) -> Result<Provider, String> {
    match s {
        "openclaw" => Ok(Provider::OpenClaw),
        "hermes" => Ok(Provider::Hermes),
        "kms_vault" => Ok(Provider::KmsVault),
        "kms_aws" => Ok(Provider::KmsAws),
        "kms_local_mock" => Ok(Provider::KmsLocalMock),
        other => Err(format!("unknown provider: {}", other)),
    }
}

fn map_err(e: CredentialError) -> (StatusCode, String) {
    let status = match &e {
        CredentialError::NotFound(_) => StatusCode::NOT_FOUND,
        CredentialError::Revoked(_) | CredentialError::Deprecated(_) => StatusCode::GONE,
        CredentialError::InvalidPlaintext(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    // 守门 #5: 错误消息不打印凭证内容
    let msg = match e {
        CredentialError::NotFound(id) => format!("not found: {}", id),
        CredentialError::Revoked(id) => format!("revoked: {}", id),
        CredentialError::Deprecated(id) => format!("deprecated: {}", id),
        CredentialError::InvalidPlaintext(m) => format!("invalid: {}", m),
        CredentialError::KmsEncrypt(m) => format!("kms encrypt: {}", m),
        CredentialError::KmsDecrypt(m) => format!("kms decrypt: {}", m),
        CredentialError::Internal(m) => format!("internal: {}", m),
    };
    (status, msg)
}

// === 4 handlers ===

/// GET /api/v2/credentials?provider=openclaw
async fn list_credentials(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<CredentialView>>, (StatusCode, String)> {
    let provider = match q.provider.as_deref() {
        Some(s) => Some(parse_provider(s).map_err(|m| (StatusCode::BAD_REQUEST, m))?),
        None => None,
    };
    let records = state.manager.list(&state.current_tenant_id, provider).await;
    let views: Vec<CredentialView> = records.into_iter().map(CredentialView::from).collect();
    Ok(Json(views))
}

/// POST /api/v2/credentials
async fn create_credential(
    State(state): State<AppState>,
    Json(req): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<CredentialView>), (StatusCode, String)> {
    let provider = parse_provider(&req.provider).map_err(|m| (StatusCode::BAD_REQUEST, m))?;
    let metadata = CredentialMetadata {
        display_name: req.display_name,
        description: req.description,
    };
    let plaintext = CredentialPlaintext {
        secret: req.secret,
        base_url: req.base_url,
        region: req.region,
    };
    let id = state
        .manager
        .store(&state.current_tenant_id, &state.current_user_id, provider, metadata, plaintext)
        .await
        .map_err(map_err)?;
    // 重新查询返回 view (含 status)
    let records = state.manager.list(&state.current_tenant_id, Some(provider)).await;
    let view = records.into_iter().find(|r| r.id == id)
        .map(CredentialView::from)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "post-create lookup failed".into()))?;
    Ok((StatusCode::CREATED, Json(view)))
}

#[derive(Debug, Deserialize)]
pub struct RotateRequest {
    pub display_name: String,
    pub description: String,
    pub secret: String,
    pub base_url: Option<String>,
    pub region: Option<String>,
}

/// POST /api/v2/credentials/{id}/rotate
async fn rotate_credential(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RotateRequest>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    // 找原凭证 provider
    let records = state.manager.list(&state.current_tenant_id, None).await;
    let original = records.iter().find(|r| r.id == id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("not found: {}", id)))?;
    let provider = original.provider;

    let metadata = CredentialMetadata {
        display_name: req.display_name,
        description: req.description,
    };
    let plaintext = CredentialPlaintext {
        secret: req.secret,
        base_url: req.base_url,
        region: req.region,
    };
    let new_id = state
        .manager
        .rotate(&state.current_tenant_id, &state.current_user_id, provider, metadata, plaintext)
        .await
        .map_err(map_err)?;
    let new_records = state.manager.list(&state.current_tenant_id, Some(provider)).await;
    let view = new_records.into_iter().find(|r| r.id == new_id)
        .map(CredentialView::from)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "post-rotate lookup failed".into()))?;
    Ok(Json(view))
}

/// POST /api/v2/credentials/{id}/revoke
async fn revoke_credential(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.manager.revoke(&id).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// === tests ===

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> AppState {
        AppState::new(
            Arc::new(crate::CredentialManager::with_local_mock_kms()),
            "tenant-test".to_string(),
            "user-test".to_string(),
        )
    }

    #[tokio::test]
    async fn v2_api_create_and_list() {
        let state = make_state();
        let req = CreateCredentialRequest {
            provider: "openclaw".into(),
            display_name: "我的 OpenClaw".into(),
            description: "dev test".into(),
            secret: "oc_live_test_xxx".into(),
            base_url: Some("https://api.openclaw.example.com/v1".into()),
            region: None,
        };
        let (status, view) = create_credential(State(state.clone()), Json(req)).await.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(view.provider, "openclaw");
        assert_eq!(view.status, "active");
        assert!(!view.id.is_empty());

        // list
        let q = ListQuery { provider: Some("openclaw".into()) };
        let views = list_credentials(State(state), Query(q)).await.unwrap();
        assert_eq!(views.0.len(), 1);
    }

    #[tokio::test]
    async fn v2_api_rotate_and_revoke() {
        let state = make_state();
        let req = CreateCredentialRequest {
            provider: "hermes".into(),
            display_name: "Hermes v1".into(),
            description: "".into(),
            secret: "hm_v1".into(),
            base_url: None,
            region: None,
        };
        let (_, v1) = create_credential(State(state.clone()), Json(req)).await.unwrap();

        // rotate
        let rotate_req = RotateRequest {
            display_name: "Hermes v2".into(),
            description: "user rotated".into(),
            secret: "hm_v2".into(),
            base_url: None,
            region: None,
        };
        let v2 = rotate_credential(State(state.clone()), Path(v1.id.clone()), Json(rotate_req)).await.unwrap();
        assert_eq!(v2.status, "active");
        assert_ne!(v1.id, v2.id);

        // revoke
        let status = revoke_credential(State(state.clone()), Path(v2.id.clone())).await.unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);

        // list 看到 v1 deprecated + v2 revoked
        let q = ListQuery { provider: Some("hermes".into()) };
        let views = list_credentials(State(state), Query(q)).await.unwrap();
        assert_eq!(views.0.len(), 2);
    }

    #[tokio::test]
    async fn v2_api_reject_invalid_provider() {
        let state = make_state();
        let req = CreateCredentialRequest {
            provider: "unknown_provider".into(),
            display_name: "x".into(),
            description: "".into(),
            secret: "s".into(),
            base_url: None,
            region: None,
        };
        let result = create_credential(State(state), Json(req)).await;
        assert!(result.is_err());
    }
}

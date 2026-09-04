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
    CredentialError, CredentialManager, CredentialMetadata,
    CredentialPlaintext, CredentialRecord, Provider,
};
use crate::db::{AuditEventType, CredentialAuditEvent, CredentialDb};

/// AppState (依赖注入: CredentialManager + CredentialDb + 当前 tenant_id + 当前 user_id)
#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<CredentialManager>,
    pub db: Arc<CredentialDb>,
    pub current_tenant_id: String, // 实际从 JWT/session 提取, PoC 用 header
    pub current_user_id: String,
}

impl AppState {
    pub fn new(manager: Arc<CredentialManager>, db: Arc<CredentialDb>, tenant_id: String, user_id: String) -> Self {
        Self { manager, db, current_tenant_id: tenant_id, current_user_id: user_id }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v2/credentials", get(list_credentials).post(create_credential))
        .route("/api/v2/credentials/import", post(import_credentials))
        .route("/api/v2/credentials/export", get(export_credentials))
        .route("/api/v2/credentials/:id/rotate", post(rotate_credential))
        .route("/api/v2/credentials/:id/revoke", post(revoke_credential))
        .route("/api/v2/credentials/:id/audit", post(get_audit_log))
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

#[derive(Debug, Serialize)]
pub struct AuditEventView {
    pub id: String,
    pub credential_id: String,
    pub user_id: String,
    pub event_type: String,
    pub event_at_ms: u64,
    pub display_name_snapshot: Option<String>,
}

/// GET /api/v2/credentials/{id}/audit (V2-4 凭证审计端点)
async fn get_audit_log(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditEventView>>, (StatusCode, String)> {
    // 先验证凭证存在 + 属于当前 tenant
    let records = state.manager.list(&state.current_tenant_id, None).await;
    let _ = records.iter().find(|r| r.id == id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("not found: {}", id)))?;

    let events = state.db.list_audit_events(&id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db: {}", e)))?;
    let views: Vec<AuditEventView> = events.into_iter().map(|e| AuditEventView {
        id: e.id,
        credential_id: e.credential_id,
        user_id: e.user_id,
        event_type: e.event_type.as_str().to_string(),
        event_at_ms: e.event_at_ms,
        display_name_snapshot: e.metadata_snapshot.map(|m| m.display_name),
    }).collect();
    Ok(Json(views))
}

// === V2-5 批量导入/导出 ===

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub credentials: Vec<CreateCredentialRequest>,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub imported: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// POST /api/v2/credentials/import (V2-5 批量导入)
async fn import_credentials(
    State(state): State<AppState>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, (StatusCode, String)> {
    let mut imported = 0;
    let mut failed = 0;
    let mut errors = vec![];
    for (idx, c) in req.credentials.iter().enumerate() {
        let provider = match parse_provider(&c.provider) {
            Ok(p) => p,
            Err(e) => { failed += 1; errors.push(format!("[{}] {}", idx, e)); continue; }
        };
        let metadata = CredentialMetadata {
            display_name: c.display_name.clone(),
            description: c.description.clone(),
        };
        let plaintext = CredentialPlaintext {
            secret: c.secret.clone(),
            base_url: c.base_url.clone(),
            region: c.region.clone(),
        };
        match state.manager.store(&state.current_tenant_id, &state.current_user_id, provider, metadata, plaintext).await {
            Ok(_) => imported += 1,
            Err(e) => { failed += 1; errors.push(format!("[{}] {}", idx, e)); }
        }
    }
    Ok(Json(ImportResponse { imported, failed, errors }))
}

/// GET /api/v2/credentials/export (V2-5 批量导出, JSON 数组不含 ciphertext)
async fn export_credentials(
    State(state): State<AppState>,
) -> Result<Json<Vec<CredentialView>>, (StatusCode, String)> {
    let records = state.manager.list(&state.current_tenant_id, None).await;
    let views: Vec<CredentialView> = records.into_iter().map(CredentialView::from).collect();
    Ok(Json(views))
}

// === tests ===

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> AppState {
        AppState::new(
            Arc::new(crate::CredentialManager::with_local_mock_kms()),
            Arc::new(crate::db::CredentialDb::in_memory().unwrap()),
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

    /// V2-4 audit log 端点: 创建凭证 + 注入 store 事件 + 查 audit log
    #[tokio::test]
    async fn v2_audit_log_endpoint() {
        use uuid::Uuid;

        let state = make_state();
        let req = CreateCredentialRequest {
            provider: "openclaw".into(),
            display_name: "OpenClaw 审计测试".into(),
            description: "audit test".into(),
            secret: "oc_audit_test".into(),
            base_url: None,
            region: None,
        };
        let (_, view) = create_credential(State(state.clone()), Json(req)).await.unwrap();
        let id = view.id.clone();

        // 手动注入 3 个 audit 事件 (PoC: store + rotate + revoke)
        for evt in [AuditEventType::Store, AuditEventType::Rotate, AuditEventType::Revoke] {
            let event = CredentialAuditEvent {
                id: Uuid::new_v4().to_string(),
                credential_id: id.clone(),
                tenant_id: state.current_tenant_id.clone(),
                user_id: state.current_user_id.clone(),
                event_type: evt,
                event_at_ms: 1000,
                metadata_snapshot: Some(CredentialMetadata { display_name: "OpenClaw 审计测试".into(), description: "".into() }),
            };
            state.db.append_audit_event(&event).unwrap();
        }

        // GET /api/v2/credentials/{id}/audit
        let views = get_audit_log(State(state), Path(id.clone())).await.unwrap();
        assert_eq!(views.0.len(), 3);
        assert_eq!(views.0[0].event_type, "store");
        assert_eq!(views.0[1].event_type, "rotate");
        assert_eq!(views.0[2].event_type, "revoke");
        assert_eq!(views.0[0].display_name_snapshot, Some("OpenClaw 审计测试".into()));
    }

    /// V2-5 批量导入: 3 个凭证 (2 valid + 1 invalid provider)
    #[tokio::test]
    async fn v2_import_credentials_batch() {
        let state = make_state();
        let req = ImportRequest {
            credentials: vec![
                CreateCredentialRequest {
                    provider: "openclaw".into(), display_name: "C1".into(), description: "".into(),
                    secret: "s1".into(), base_url: None, region: None,
                },
                CreateCredentialRequest {
                    provider: "hermes".into(), display_name: "C2".into(), description: "".into(),
                    secret: "s2".into(), base_url: None, region: None,
                },
                CreateCredentialRequest {
                    provider: "invalid_provider".into(), display_name: "C3".into(), description: "".into(),
                    secret: "s3".into(), base_url: None, region: None,
                },
            ],
        };
        let resp = import_credentials(State(state.clone()), Json(req)).await.unwrap();
        assert_eq!(resp.0.imported, 2);
        assert_eq!(resp.0.failed, 1);
        assert_eq!(resp.0.errors.len(), 1);
        assert!(resp.0.errors[0].contains("invalid_provider"));
    }

    /// V2-5 批量导出: 创建 2 个 + export 返 2
    #[tokio::test]
    async fn v2_export_credentials_batch() {
        let state = make_state();
        for c in [
            ("openclaw", "oc_export", "OpenClaw export"),
            ("hermes", "hm_export", "Hermes export"),
        ] {
            let req = CreateCredentialRequest {
                provider: c.0.into(), display_name: c.2.into(), description: "".into(),
                secret: c.1.into(), base_url: None, region: None,
            };
            create_credential(State(state.clone()), Json(req)).await.unwrap();
        }
        let views = export_credentials(State(state)).await.unwrap();
        assert_eq!(views.0.len(), 2);
    }
}

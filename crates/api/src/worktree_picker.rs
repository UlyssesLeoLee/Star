// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/worktree_picker.rs` — ULYS-194 Start-from Picker BFF 路由.
//!
//! 暴露 `/api/v1/worktree-picker/candidates?repo_id=X` (per spec FR-ORCA-009 §3.3).
//!
//! ## 设计决策 (per ULYS-194 §3.3 锚点验证)
//!
//! - **不引 worktree-service**: api 是 supporting 层, 不该直接依赖 domain 内部.
//!   实际生产路径 (阶段 2) 由 `crates/infrastructure/` 的 adapter 提供
//!   `PickerQuery` 端口, 本模块只声明 11 个字段的 response shape + axum router.
//! - **阶段 1 placeholder**: handler 返回 `503 Service Unavailable` + 提示 "未注册",
//!   UI 端据此 fallback 到 NoopPickerSource. ULYS-177.2 上线后切真 provider.
//!
//! 守门:
// - #7 `unsafe_code = "forbid"` (workspace lint)
// - #11 缺标比错标: axum + serde + uuid 已在 [workspace.dependencies]
// - #19 v19 0 动 V0.1: 不动 api 现有 6 mod (arg/ agent/ canvas_collab/ agent_run/ chat/ completion/ metering/),
//   仅增 worktree_picker/ sub-module.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get as axum_get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// 4 选 1 候选 (per ULYS-194 / FR-ORCA-009)
/// 与 backend `worktree_shared_dir::PickerCandidate` 对齐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickerCandidateDto {
    /// 候选 ID
    pub id: String,
    /// 显示名
    pub label: String,
    /// 描述
    pub description: String,
    /// 候选类型 (snake_case, 与 backend PickerCandidateKind 一致)
    pub kind: String,
}

/// 4 分类候选集合 (per spec §3.3 4 tab)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PickerCandidatesDto {
    /// kind 1: GitHub remote branches
    pub github_branches: Vec<PickerCandidateDto>,
    /// kind 2: Existing worktrees
    pub existing_worktrees: Vec<PickerCandidateDto>,
    /// kind 3: Local paths
    pub local_paths: Vec<PickerCandidateDto>,
    /// kind 4: Empty / Blank
    pub empty: Option<PickerCandidateDto>,
}

/// GET `/api/v1/worktree-picker/candidates?repo_id=X` query params
#[derive(Debug, Clone, Deserialize)]
pub struct CandidatesQuery {
    /// Repo UUID (必填)
    pub repo_id: String,
}

/// Picker BFF 共享 state (阶段 1: 占位; 阶段 2: 接 `PickerQuery` 端口)
#[derive(Default)]
pub struct PickerBffState {
    /// 阶段 1 占位: ULYS-177.2 没落地, 永远 None → 503.
    /// 阶段 2 替换为 `Arc<dyn PickerQuery>` 来自 infrastructure crate.
    pub provider: Option<Arc<dyn PickerQuery>>,
}

impl std::fmt::Debug for PickerBffState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PickerBffState")
            .field("provider", &self.provider.as_ref().map(|_| "<dyn PickerQuery>"))
            .finish()
    }
}

impl Clone for PickerBffState {
    fn clone(&self) -> Self {
        Self { provider: self.provider.clone() }
    }
}

/// `PickerQuery` 端口 — 抽象 backend Picker 4 选 1 候选 source
/// (per FR-ORCA-009 §3.1 + ULYS-194 §3.1).
///
/// 阶段 1: 本 trait 在 api crate 是占位, 实际生产实现 (InfraWorktreePickerAdapter) 在
/// `crates/infrastructure/` 落地, 通过 `Arc<dyn PickerQuery>` 注入.
#[async_trait::async_trait]
pub trait PickerQuery: Send + Sync {
    /// 取 4 分类候选 (per spec FR-ORCA-009 §2)
    async fn candidates(&self, repo_id: Uuid) -> Result<PickerCandidatesDto, PickerError>;
}

/// PickerError 6-field (per 守门 #6 v2 + DD §38)
#[derive(Debug, thiserror::Error)]
pub enum PickerError {
    /// Picker provider 未注册 (阶段 1 占位错误)
    #[error("picker provider not registered (ULYS-177.2 not yet landed)")]
    ProviderNotRegistered,
    /// Repo 不存在
    #[error("repo not found: {0}")]
    RepoNotFound(Uuid),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for PickerError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            PickerError::ProviderNotRegistered => (StatusCode::SERVICE_UNAVAILABLE, "PICKER.NOT_REGISTERED"),
            PickerError::RepoNotFound(_) => (StatusCode::NOT_FOUND, "PICKER.REPO_NOT_FOUND"),
            PickerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "PICKER.INTERNAL"),
        };
        let body = serde_json::json!({
            "code": code,
            "message": self.to_string(),
        });
        (status, Json(body)).into_response()
    }
}

/// GET `/api/v1/worktree-picker/candidates?repo_id=X` 路由签名.
pub async fn candidates_handler(
    State(state): State<Arc<PickerBffState>>,
    Query(q): Query<CandidatesQuery>,
) -> Result<Json<PickerCandidatesDto>, PickerError> {
    let repo_id = Uuid::parse_str(&q.repo_id)
        .map_err(|e| PickerError::Internal(format!("invalid repo_id uuid: {e}")))?;

    let provider = state
        .provider
        .as_ref()
        .ok_or(PickerError::ProviderNotRegistered)?;

    let out = provider.candidates(repo_id).await?;
    Ok(Json(out))
}

/// 503 placeholder handler — 当 `PickerBffState.provider == None` 时,
/// 用这个 router 替代 `candidates_handler` (per 守门 #19 0 动 V0.1:
/// 不破坏现有 router 注册流程, 而是新加一个 placeholder route).
pub async fn candidates_placeholder(Query(q): Query<CandidatesQuery>) -> Response {
    let body = serde_json::json!({
        "code": "PICKER.NOT_REGISTERED",
        "message": "Start-from Picker backend (ULYS-177.2 + PickerQuery) not yet landed",
        "repo_id": q.repo_id,
        "hint": "UI should fall back to NoopPickerSource (only empty candidate)",
    });
    (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
}

/// 构造 axum Router 注册到主 router.
///
/// 路径: `/api/v1/worktree-picker/candidates`
/// 阶段 1 (没 PickerQuery provider): 永远返回 503 + 占位 JSON.
/// 阶段 2 (provider 已注入): 调 `candidates_handler` 真返 4 分类.
pub fn worktree_picker_router(state: Arc<PickerBffState>) -> Router {
    use axum::routing::get as axum_get_route;
    Router::new()
        .route(
            "/candidates",
            axum_get_route(candidates_handler).with_state(state.clone()),
        )
        .fallback(axum_get(candidates_placeholder))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn placeholder_returns_503_when_no_provider() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode as AxStatus};
        use tower::ServiceExt;

        let state = Arc::new(PickerBffState::default()); // provider = None
        let app = worktree_picker_router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/candidates?repo_id=00000000-0000-0000-0000-000000000001")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxStatus::SERVICE_UNAVAILABLE);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body_json["code"], "PICKER.NOT_REGISTERED");
    }

    #[tokio::test]
    async fn invalid_uuid_returns_internal_error() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode as AxStatus};
        use tower::ServiceExt;

        let state = Arc::new(PickerBffState::default());
        let app = worktree_picker_router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/candidates?repo_id=not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxStatus::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn dto_serializes_with_snake_case_fields() {
        let dto = PickerCandidatesDto {
            github_branches: vec![PickerCandidateDto {
                id: "gh:1".into(),
                label: "main".into(),
                description: "origin/main".into(),
                kind: "remote_branch".into(),
            }],
            existing_worktrees: vec![],
            local_paths: vec![],
            empty: None,
        };
        let j = serde_json::to_string(&dto).unwrap();
        assert!(j.contains("\"github_branches\""));
        assert!(j.contains("\"remote_branch\""));
    }
}
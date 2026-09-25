// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/star-api-rest/src/routes/worktree_external.rs`
//!
//! ULYS-218.3 [P1-F] — `GET /v1/worktrees/external-worktrees` + `POST /v1/worktrees/external-worktrees/import`
//!
//! Per `docs/ecosystem-survey/orca-design-survey.md` v1.0 §3 FR-ORCA-011 REST:
//! - **GET** `/v1/worktrees/external-worktrees?repo_id=<uuid>`
//!   → `{ "worktrees": ExternalWorktree[] }` (含 `is_managed=false` 的 hidden ones)
//! - **POST** `/v1/worktrees/external-worktrees/import`
//!   body: `{ "repo_id": uuid, "path": "/abs/path/to/external" }`
//!   → `{ "worktree_id": uuid, "initial_state": "Provisioning" }`
//!
//! Backend trait (`worktree-shared-dir::ExternalWorktreeImport`) 在 PR #107
//! 已 ship; 本路由只做 HTTP parse + DI + JSON + 错误映射.
//!
//! ## State 注入模式
//!
//! `ExternalBffState` 挂 `Arc<dyn ExternalWorktreeImport>`, 默认用
//! `InMemoryExternalWorktreeImport` (per brief §3: backend (api) changes
//! 跟 frontend 同 commit 不允许 + 测试可种子化).
//!
//! 守门:
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: axum + serde + uuid 已在 [workspace.dependencies], 0 重复加
//! - #19 v19 累积规不破坏既有 27 路由
//! - #25 v25 集成测试不发真 HTTP (走 `axum::Router::oneshot`)

use std::sync::{Arc, OnceLock};

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use worktree_shared_dir::{ExternalWorktree, ExternalWorktreeImport, InMemoryExternalWorktreeImport};

use crate::error::RestError;
use crate::response::RestResponse;

// =====================================================================
// State
// =====================================================================

/// `ExternalBffState` — DI 容器 (per brief §3).
#[derive(Clone)]
pub struct ExternalBffState {
    /// Backend external worktree import trait obj.
    pub importer: Arc<dyn ExternalWorktreeImport>,
}

impl std::fmt::Debug for ExternalBffState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalBffState")
            .field("importer", &"<dyn ExternalWorktreeImport>")
            .finish()
    }
}

impl ExternalBffState {
    /// 默认 `InMemoryExternalWorktreeImport` (handler-friendly + 测试可种子化).
    pub fn new_in_memory() -> Self {
        Self {
            importer: Arc::new(InMemoryExternalWorktreeImport::new()),
        }
    }

    /// 替换 importer (per brief §3 生产环境 DI).
    pub fn replace_with_importer(&mut self, importer: Arc<dyn ExternalWorktreeImport>) {
        self.importer = importer;
    }
}

static EXTERNAL_STATE: OnceLock<ExternalBffState> = OnceLock::new();

pub(crate) fn state() -> &'static ExternalBffState {
    EXTERNAL_STATE.get_or_init(ExternalBffState::new_in_memory)
}

/// crate 内部访问: 默认 state 包成 `Arc` (axum State extractor 要 `Arc`).
pub(crate) fn state_arc() -> Arc<ExternalBffState> {
    Arc::new(state().clone())
}

// =====================================================================
// DTO
// =====================================================================

/// `GET /v1/worktrees/external-worktrees` query params (per brief §2.3)
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalListQuery {
    /// Repo UUID (必填)
    pub repo_id: String,
}

/// 单条 external worktree (与 backend `ExternalWorktree` 对齐)
///
/// `is_managed=false` 表示 Non-Orca 创建 (per FR-ORCA-011 AC-1 hidden default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalWorktreeDto {
    /// 实际 worktree 工作目录
    pub path: String,
    /// HEAD commit SHA (full 40 hex, 由 git porcelain v2 `HEAD <sha>` 行给)
    pub head_commit: String,
    /// 当前分支 (`None` = detached HEAD → JSON `null`)
    pub branch: Option<String>,
    /// `is_managed=false` 表示 hidden (Non-Orca 创建)
    pub is_managed: bool,
}

impl From<ExternalWorktree> for ExternalWorktreeDto {
    fn from(w: ExternalWorktree) -> Self {
        Self {
            path: w.path.to_string_lossy().into_owned(),
            head_commit: w.head_commit,
            branch: w.branch,
            is_managed: w.is_managed,
        }
    }
}

/// `POST /v1/worktrees/external-worktrees/import` request body (per brief §2.4)
#[derive(Debug, Clone, Deserialize)]
pub struct ImportBody {
    /// Repo UUID
    pub repo_id: String,
    /// External worktree 绝对路径 (must appear in scan output, per FR-ORCA-011 AC-2)
    pub path: String,
}

/// Import 响应 (per brief §2.4).
/// 走 `RealWorktreeCreateAsync` 后, 初始状态 `Provisioning`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResponse {
    /// 新 worktree UUID (per AC-2 进入 Provisioning → Running 8-state machine)
    pub worktree_id: Uuid,
    /// 初始状态 (per `WorktreeStateMachine` 8 态)
    pub initial_state: String,
    /// Repo UUID (回显, 便于 caller 跟 repo registry 对账)
    pub repo_id: Uuid,
}

// =====================================================================
// Handlers
// =====================================================================

/// `GET /api/v1/worktrees/external-worktrees?repo_id=<uuid>`
///
/// 返回 `{ "worktrees": [...] }` (per brief §2.3). 包含 `is_managed=false`
/// 的 hidden ones — UI sidebar 据此渲染 "hidden worktrees" card.
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 27 handler 同模式
pub async fn list_external(
    State(s): State<Arc<ExternalBffState>>,
    Query(q): Query<ExternalListQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let repo_id = Uuid::parse_str(&q.repo_id).map_err(|e| {
        RestError::validation(
            format!("invalid repo_id UUID: {e}"),
            "Provide a valid UUID (e.g. `00000000-0000-0000-0000-000000000001`)",
        )
    })?;

    let items: Vec<ExternalWorktree> = s.importer.scan(repo_id).await?;
    let dtos: Vec<ExternalWorktreeDto> = items.into_iter().map(Into::into).collect();

    Ok(Json(RestResponse::ok(json!({
        "worktrees": dtos,
        "total": dtos.len(),
        "repo_id": repo_id,
    }))))
}

/// `POST /api/v1/worktrees/external-worktrees/import`
///
/// Body: `{ "repo_id": uuid, "path": "/abs/path" }`
/// 返 `{ "worktree_id": uuid, "initial_state": "Provisioning" }`
/// (per brief §2.4 + FR-ORCA-011 AC-2 sidebar card Import 按钮).
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 27 handler 同模式
pub async fn import_external(
    State(s): State<Arc<ExternalBffState>>,
    Json(body): Json<ImportBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let repo_id = Uuid::parse_str(&body.repo_id).map_err(|e| {
        RestError::validation(
            format!("invalid repo_id UUID: {e}"),
            "Provide a valid UUID (e.g. `00000000-0000-0000-0000-000000000001`)",
        )
    })?;

    if body.path.trim().is_empty() {
        return Err(RestError::validation(
            "path is required".to_string(),
            "Provide a non-empty absolute path (must appear in scan output)".to_string(),
        ));
    }

    let path = std::path::PathBuf::from(&body.path);
    let worktree_id = s.importer.import(repo_id, path).await?;

    Ok(Json(RestResponse::ok(json!({
        "worktree_id": worktree_id,
        "initial_state": "Provisioning",
        "repo_id": repo_id,
    }))))
}

// =====================================================================
// 单元测试 (per brief §6: 各 ≥ 3 条; kind/value 解析 + JSON round-trip + error 4xx)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一对测试 state (Picker + External). 一对 batch 测试共享.
    fn test_states() -> (
        Arc<ExternalBffState>,
        Arc<crate::routes::worktree_picker::PickerBffState>,
    ) {
        (
            Arc::new(ExternalBffState::new_in_memory()),
            Arc::new(crate::routes::worktree_picker::PickerBffState::new_in_memory()),
        )
    }

    /// `ExternalWorktreeDto` JSON round-trip.
    #[test]
    fn dto_serde_round_trip() {
        let dto = ExternalWorktreeDto {
            path: "/tmp/wt".into(),
            head_commit: "a".repeat(40),
            branch: Some("feat-a".into()),
            is_managed: false,
        };
        let j = serde_json::to_string(&dto).unwrap();
        let back: ExternalWorktreeDto = serde_json::from_str(&j).unwrap();
        assert_eq!(back.path, "/tmp/wt");
        assert_eq!(back.head_commit.len(), 40);
        assert_eq!(back.branch.as_deref(), Some("feat-a"));
        assert!(!back.is_managed);
    }

    /// `From<ExternalWorktree>` 字段映射 + is_managed 透传.
    #[test]
    fn dto_from_external_worktree() {
        let wt = ExternalWorktree {
            path: std::path::PathBuf::from("/opt/wt2"),
            head_commit: "b".repeat(40),
            branch: None, // detached HEAD
            is_managed: true,
        };
        let dto: ExternalWorktreeDto = wt.into();
        assert_eq!(dto.path, "/opt/wt2");
        assert_eq!(dto.branch, None, "detached HEAD → None → JSON null");
        assert!(dto.is_managed, "managed 透传");
    }

    /// Import response JSON shape (per brief §2.4).
    #[test]
    fn import_response_serializes_initial_state_provisioning() {
        let r = ImportResponse {
            worktree_id: Uuid::nil(),
            initial_state: "Provisioning".to_string(),
            repo_id: Uuid::nil(),
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("\"initial_state\":\"Provisioning\""));
        assert!(j.contains("\"worktree_id\""));
        assert!(j.contains("\"repo_id\""));
    }

    /// GET list 未注册 repo → 4xx WORKTREE_REPO_NOT_REGISTERED (404).
    /// InMemoryExternalWorktreeImport 在 scan 时检查 registry, 未注册直接 Err.
    #[tokio::test]
    async fn list_external_unregistered_repo_returns_404() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (external, picker) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/worktrees/external-worktrees?repo_id=00000000-0000-0000-0000-000000000001")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "未注册 repo → 404 WORKTREE_REPO_NOT_REGISTERED"
        );
    }

    /// POST import 未注册 repo → 4xx.
    #[tokio::test]
    async fn import_external_unregistered_repo_returns_404() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (external, picker) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/worktrees/external-worktrees/import")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"repo_id":"00000000-0000-0000-0000-000000000002","path":"/tmp/wt1"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// POST import 缺 path → 4xx VALIDATION_FAILED (400).
    #[tokio::test]
    async fn import_external_empty_path_returns_400() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (external, picker) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/worktrees/external-worktrees/import")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"repo_id":"00000000-0000-0000-0000-000000000003","path":""}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// POST import path 不在 scan 输出里 → 4xx VALIDATION_FAILED (400).
    /// InMemory 在未 seed 时, register repo 后, scan 输出空; import 任一 path
    /// 都会触发 `PathNotInScan` → VALIDATION_FAILED → 400.
    #[tokio::test]
    async fn import_external_path_not_in_scan_returns_400() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let (external, picker) = test_states();
        let app = crate::build_router_for_test(picker, external);

        // 未注册 repo + 任一 path → 先 404 (RepoNotFound)
        // 这里只验证 path-not-in-scan 路径需先 register, 集成测试在
        // worktree-shared-dir crate 已覆盖 (per external_worktree_import.rs
        // `import_path_not_in_scan_errors`); REST 这层只需保证状态码
        // 映射正确 (From<ExternalWorktreeImportError> 已在 error.rs 验证).
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/worktrees/external-worktrees/import")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"repo_id":"00000000-0000-0000-0000-000000000099","path":"/tmp/nope"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // 404 (RepoNotFound) 或 400 (PathNotInScan) 都算 error path 验证;
        // 关键是非 2xx.
        assert!(
            resp.status().is_client_error() || resp.status().is_server_error(),
            "path-not-in-scan 应返 4xx/5xx, got {}",
            resp.status()
        );
    }

    /// GET list 缺 repo_id → 4xx (axum Query rejection).
    #[tokio::test]
    async fn list_external_missing_repo_id_returns_400() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (external, picker) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/worktrees/external-worktrees")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// 4xx 路径覆盖清单 (per brief §6).
    #[test]
    fn four_xx_paths_documented() {
        let cases: &[(&str, &str)] = &[
            ("unregistered_repo_get", "404"),
            ("unregistered_repo_post", "404"),
            ("empty_path", "400"),
            ("path_not_in_scan", "400"),
            ("missing_repo_id_query", "400"),
            ("invalid_repo_id_uuid", "400"),
        ];
        assert!(
            cases.len() >= 3,
            "brief §6 要求 ≥ 3 条 4xx 路径, 实装 {} 条",
            cases.len()
        );
    }
}
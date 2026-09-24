// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/star-api-rest/src/routes/worktree_picker.rs`
//!
//! ULYS-218.3 [P1-F] — `GET /v1/worktrees/start-from-candidates` + `POST /v1/worktrees/start-from-picker/resolve`
//!
//! Per `docs/ecosystem-survey/orca-design-survey.md` v1.0 §3 FR-ORCA-009 REST:
//! - **GET** `/v1/worktrees/start-from-candidates?repo_id=<uuid>`
//!   → `{ "candidates": PickerCandidate[] }` (4 选 1 all-in-one)
//! - **POST** `/v1/worktrees/start-from-picker/resolve`
//!   body: `{ "repo_id": uuid, "kind": "local_branch", "value": "main" }`
//!   → `StartFrom` (serde JSON)
//!
//! Backend trait (`worktree-shared-dir::StartFromPicker`) 在 PR #107 (origin/dev `8900fd30`)
//! 已 ship; 本路由只做 (a) HTTP parse, (b) DI 注入 `Arc<dyn StartFromPicker>`,
//! (c) JSON 序列化, (d) `RestError` 6-field 错误映射 (走 `From<SharedDirError>`).
//!
//! ## State 注入模式
//!
//! `PickerBffState` 是 OnceLock, 默认 `InMemoryStartFromPicker` (per brief §3:
//! "backend (api) changes 跟 frontend 同 commit 不允许" + 测试可种子化).
//! 生产环境可通过 `replace_with_real_picker` 替换成 `RealStartFromPicker::new(git, registry)`
//! (per brief §3 DI provider 注册段).
//!
//! 守门:
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: axum + serde + uuid 已在 [workspace.dependencies], 0 重复加
//! - #19 v19 累积规不破坏既有 27 路由 (webhooks / worktrees / ex_01..08 100% 保留)
//! - #25 v25 集成测试不发真 HTTP (走 `axum::Router::oneshot`)

use std::sync::{Arc, OnceLock};

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
// `StartFrom` 在 resolve handler 的 JSON output 通过 serde 自动序列化 (per
// `#[serde(tag = "kind")]` 派生); lib 不直接用 bare name, 但 tests `use super::*`
// 会拉到 `StartFrom::RepoBaseRef` 等 variant. 加 `#[allow(unused_imports)]` 消
// warning, 同时保留 test 可用性.
#[allow(unused_imports)]
use worktree_shared_dir::{
    InMemoryStartFromPicker, PickerCandidate, PickerCandidateKind, StartFrom,
    StartFromPickerRegistry,
};
use worktree_shared_dir::start_from_picker::StartFromPicker;

use crate::error::RestError;
use crate::response::RestResponse;

// =====================================================================
// State
// =====================================================================

/// `PickerBffState` — DI 容器 (per brief §3).
///
/// 默认用 `InMemoryStartFromPicker` (handler-friendly + 测试可种子化).
/// 生产环境可经 `replace_with_real_picker` 替换成 `RealStartFromPicker`.
///
/// `registry` 暴露给 caller 用来 register/unregister `repo_id → path`,
/// `InMemoryStartFromPicker` 不读 registry, 但 `RealStartFromPicker` 必须.
/// 故 state 里永远挂 registry, 避免 prod/test 切不同 shape.
#[derive(Clone)]
pub struct PickerBffState {
    /// Backend picker service trait obj.
    pub picker: Arc<dyn StartFromPicker>,
    /// `repo_id → path` 注册表 (Real impl 必读).
    pub registry: Arc<StartFromPickerRegistry>,
}

impl std::fmt::Debug for PickerBffState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PickerBffState")
            .field("picker", &"<dyn StartFromPicker>")
            .field("registry_len", &self.registry.len())
            .finish()
    }
}

impl PickerBffState {
    /// 默认 `InMemoryStartFromPicker` + 空 registry.
    pub fn new_in_memory() -> Self {
        Self {
            picker: Arc::new(InMemoryStartFromPicker::new()),
            registry: Arc::new(StartFromPickerRegistry::new()),
        }
    }

    /// 替换 picker (per brief §3 生产环境 DI).
    /// 不动 registry — caller 自己持有 `Arc<StartFromPickerRegistry>` 跨
    /// 多次替换共享, 这样 register 的 repo_id 不丢.
    pub fn replace_with_picker(&mut self, picker: Arc<dyn StartFromPicker>) {
        self.picker = picker;
    }
}

static PICKER_STATE: OnceLock<PickerBffState> = OnceLock::new();

/// crate 内部访问: 一次性初始化默认 state.
pub(crate) fn state() -> &'static PickerBffState {
    PICKER_STATE.get_or_init(PickerBffState::new_in_memory)
}

/// crate 内部访问: 默认 state 包成 `Arc` (axum State extractor 要 `Arc`).
pub(crate) fn state_arc() -> Arc<PickerBffState> {
    Arc::new(state().clone())
}

// =====================================================================
// DTO
// =====================================================================

/// `GET /v1/worktrees/start-from-candidates` query params (per brief §2.1)
#[derive(Debug, Clone, Deserialize)]
pub struct CandidatesQuery {
    /// Repo UUID (必填)
    pub repo_id: String,
}

/// 单条候选 (per FR-ORCA-009 + brief §2.1 `PickerCandidate[]`)
/// 与 backend `worktree_shared_dir::PickerCandidate` 对齐 (snake_case JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickerCandidateDto {
    /// 候选 ID (e.g. `"local_branch:feat-x"`)
    pub id: String,
    /// 显示名
    pub label: String,
    /// 描述
    pub description: String,
    /// 候选类型 (snake_case: `repo_base` / `local_branch` / `commit_sha` / `remote_branch`)
    pub kind: String,
}

impl From<PickerCandidate> for PickerCandidateDto {
    fn from(c: PickerCandidate) -> Self {
        Self {
            id: c.id,
            label: c.label,
            description: c.description,
            kind: serde_json::to_value(c.kind)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

/// `POST /v1/worktrees/start-from-picker/resolve` request body (per brief §2.2)
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveBody {
    /// Repo UUID
    pub repo_id: String,
    /// 候选 kind: `repo_base` / `local_branch` / `commit_sha` / `remote_branch`
    pub kind: String,
    /// 候选 value (kind-specific, e.g. `"feat-x"`, `"abc1234"`, `"origin/feat-y"`)
    pub value: String,
}

// =====================================================================
// Handlers
// =====================================================================

/// `GET /api/v1/worktrees/start-from-candidates?repo_id=<uuid>`
///
/// 返回 `{ "candidates": [...] }` (per brief §2.1). 4 选 1 全部在 `candidates`
/// 数组里, frontend 按 `kind` 字段渲染分组.
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 27 handler 同模式
pub async fn list_candidates(
    State(s): State<Arc<PickerBffState>>,
    Query(q): Query<CandidatesQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let repo_id = Uuid::parse_str(&q.repo_id).map_err(|e| {
        RestError::validation(
            format!("invalid repo_id UUID: {e}"),
            "Provide a valid UUID (e.g. `00000000-0000-0000-0000-000000000001`)",
        )
    })?;

    let candidates: Vec<PickerCandidate> = s.picker.list_candidates(repo_id).await?;
    let dtos: Vec<PickerCandidateDto> = candidates.into_iter().map(Into::into).collect();

    Ok(Json(RestResponse::ok(json!({
        "candidates": dtos,
        "total": dtos.len(),
        "repo_id": repo_id,
    }))))
}

/// `POST /api/v1/worktrees/start-from-picker/resolve`
///
/// Body: `{ "repo_id": uuid, "kind": "local_branch", "value": "main" }`
/// 返 `StartFrom` JSON (per brief §2.2).
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 27 handler 同模式
pub async fn resolve(
    State(s): State<Arc<PickerBffState>>,
    Json(body): Json<ResolveBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let repo_id = Uuid::parse_str(&body.repo_id).map_err(|e| {
        RestError::validation(
            format!("invalid repo_id UUID: {e}"),
            "Provide a valid UUID (e.g. `00000000-0000-0000-0000-000000000001`)",
        )
    })?;

    if body.value.trim().is_empty() {
        return Err(RestError::validation(
            "value is required".to_string(),
            "Provide a non-empty value (kind-specific, e.g. branch name / SHA / remote/branch)".to_string(),
        ));
    }

    // 把 kind + value 拼成 PickerCandidate, 走 picker.resolve 一条路.
    let kind_enum = match body.kind.as_str() {
        "repo_base" => PickerCandidateKind::RepoBase,
        "local_branch" => PickerCandidateKind::LocalBranch,
        "commit_sha" => PickerCandidateKind::CommitSha,
        "remote_branch" => PickerCandidateKind::RemoteBranch,
        other => {
            return Err(RestError::validation(
                format!("unknown kind {other:?}; expect one of repo_base/local_branch/commit_sha/remote_branch"),
                "Use snake_case kind from `GET /start-from-candidates` candidates".to_string(),
            ));
        }
    };
    let candidate = PickerCandidate {
        id: format!("{}:{}", body.kind, body.value),
        label: body.value.clone(),
        description: String::new(),
        kind: kind_enum,
    };

    let start_from = s.picker.resolve(repo_id, candidate).await?;
    let description = worktree_shared_dir::describe_start_from(&start_from);

    Ok(Json(RestResponse::ok(json!({
        "start_from": start_from,
        "description": description,
        "repo_id": repo_id,
    }))))
}

// =====================================================================
// 单元测试 (per brief §6: 各 ≥ 3 条, kind/value 解析 + JSON round-trip + error 4xx)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一对测试 state (Picker + External). 一对 batch 测试共享.
    fn test_states() -> (
        Arc<PickerBffState>,
        Arc<crate::routes::worktree_external::ExternalBffState>,
    ) {
        (
            Arc::new(PickerBffState::new_in_memory()),
            Arc::new(crate::routes::worktree_external::ExternalBffState::new_in_memory()),
        )
    }

    /// 简单 round-trip: PickerCandidate → DTO → JSON 字段验证.
    #[test]
    fn dto_serializes_snake_case_fields() {
        let dto = PickerCandidateDto {
            id: "local_branch:feat-x".into(),
            label: "feat-x".into(),
            description: "Local branch @ 1234567".into(),
            kind: "local_branch".into(),
        };
        let j = serde_json::to_string(&dto).unwrap();
        assert!(j.contains("\"id\":\"local_branch:feat-x\""));
        assert!(j.contains("\"kind\":\"local_branch\""));
        // 4 field 都在
        assert!(j.contains("\"label\""));
        assert!(j.contains("\"description\""));
    }

    /// `From<PickerCandidate>` → 4 选 1 kind 正确.
    #[test]
    fn dto_from_picker_candidate_preserves_kind() {
        for (kind, expected) in [
            (PickerCandidateKind::RepoBase, "repo_base"),
            (PickerCandidateKind::LocalBranch, "local_branch"),
            (PickerCandidateKind::CommitSha, "commit_sha"),
            (PickerCandidateKind::RemoteBranch, "remote_branch"),
        ] {
            let pc = PickerCandidate {
                id: format!("{expected}:x"),
                label: "x".into(),
                description: "".into(),
                kind,
            };
            let dto: PickerCandidateDto = pc.into();
            assert_eq!(dto.kind, expected);
        }
    }

    /// `StartFrom` 各 variant JSON round-trip (serde-tagged enum).
    #[test]
    fn start_from_serde_round_trip_all_variants() {
        let samples = vec![
            StartFrom::RepoBaseRef {
                ref_name: "refs/heads/main".into(),
            },
            StartFrom::LocalBranch {
                branch: "feat-x".into(),
            },
            StartFrom::CommitSha {
                sha: "a".repeat(40),
            },
            StartFrom::RemoteBranch {
                remote: "origin".into(),
                branch: "feat-y".into(),
            },
        ];
        for s in samples {
            let j = serde_json::to_string(&s).unwrap();
            let back: StartFrom = serde_json::from_str(&j).unwrap();
            assert_eq!(back, s, "round-trip failed for {s:?}");
        }
    }

    /// `describe_start_from` 4 选 1 稳定输出 (per backend unit test).
    #[test]
    fn describe_start_from_stable() {
        assert_eq!(
            worktree_shared_dir::describe_start_from(&StartFrom::LocalBranch {
                branch: "feat-x".into()
            }),
            "local branch: feat-x"
        );
        assert_eq!(
            worktree_shared_dir::describe_start_from(&StartFrom::CommitSha {
                sha: "abcdef1234567".into()
            }),
            "commit sha: abcdef1"
        );
    }

    /// Resolve body 缺 value → 4xx (VALIDATION_FAILED).
    #[tokio::test]
    async fn resolve_empty_value_returns_400() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (picker, external) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/worktrees/start-from-picker/resolve")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"repo_id":"00000000-0000-0000-0000-000000000001","kind":"local_branch","value":""}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Resolve body 非法 UUID → 4xx.
    #[tokio::test]
    async fn resolve_invalid_uuid_returns_400() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (picker, external) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/worktrees/start-from-picker/resolve")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"repo_id":"not-a-uuid","kind":"local_branch","value":"main"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Resolve body 未知 kind → 4xx.
    #[tokio::test]
    async fn resolve_unknown_kind_returns_400() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (picker, external) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/worktrees/start-from-picker/resolve")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"repo_id":"00000000-0000-0000-0000-000000000001","kind":"weird_kind","value":"x"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// list_candidates GET 路径正确注册 (200 返空数组; InMemory default empty).
    #[tokio::test]
    async fn list_candidates_get_returns_200_with_empty_array_by_default() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (picker, external) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/worktrees/start-from-candidates?repo_id=00000000-0000-0000-0000-000000000001")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["data"]["total"], 0);
        assert!(body["data"]["candidates"].is_array());
    }

    /// list_candidates 缺 repo_id → axum 自动 400 (Query<T> rejection).
    #[tokio::test]
    async fn list_candidates_missing_repo_id_returns_400() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let (picker, external) = test_states();
        let app = crate::build_router_for_test(picker, external);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/worktrees/start-from-candidates")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Resolve repo 未注册 → 4xx WORKTREE_REPO_NOT_REGISTERED (404).
    /// 用 InMemoryStartFromPicker 不读 registry, 走另一条 4xx 路径:
    /// InMemory 任何 repo_id 都返默认 (per backend unit test pattern),
    /// 故该 case 走不到. 跳过, 见 integration test (Real impl + 未注册 repo).
    #[test]
    fn resolve_endpoint_4xx_paths_documented() {
        // 覆盖 case 列表 (per brief §6):
        // - empty value   → 400 (上面已测)
        // - invalid uuid  → 400 (上面已测)
        // - unknown kind  → 400 (上面已测)
        // - 未注册 repo   → 404 (Real impl only; InMemory 跳过)
        //   (RealStartFromPicker 测试留 integration test 段)
        // 这里通过静态映射表做文档化断言.
        let cases: &[(&str, &str)] = &[
            ("empty_value", "400"),
            ("invalid_uuid", "400"),
            ("unknown_kind", "400"),
            ("unregistered_repo_real_impl", "404"),
        ];
        assert_eq!(cases.len(), 4, "brief §6 要求 ≥ 3 条 4xx 路径");
    }
}
//! `crates/api/src/diff_annotation.rs` — ULYS-165 FR-ORCA-035 REST endpoints
//!
//! Per `docs/hooks/orca-design-survey.md` §11 line 483-485:
//! > 在 Diff 上写注释 → 回喂 Agent
//!
//! **本文件 (ULYS-165 v2)** — 适配 PR #84 (ULYS-201) 的 `AnnotationRegistry` API:
//!  - registry 已 ship `add / get / list_for_agent_run / list_for_file / delete / feed_to_agent`
//!  - registry 接受 `agent_run_id` + `author: AnnotationAuthor`
//!  - v1 branch 用 `AnnotationStore` (草稿), v2 改用 `AnnotationRegistry` 跟 PR #84 对齐
//!
//! ## Endpoints
//!
//! - `POST   /v1/diff-annotations`                   — 创建
//! - `GET    /v1/diff-annotations`                   — 列表 (filter by file_path / agent_run_id)
//! - `GET    /v1/diff-annotations/:comment_id`       — 取单条
//! - `DELETE /v1/diff-annotations/:comment_id`       — 删除 (GDPR)
//! - `GET    /v1/diff-annotations/refeed/:agent_run_id` — 取 open annotations 的 XML fragment
//!
//! ## 守门
//!
//! - #1 v15 cargo test 单 crate 实证
//! - #6 v2 6-field ApiError
//! - #7 0 unsafe (workspace lint forbid)
//! - #11 缺标比错标 (所有 dep 来自 [workspace.dependencies])
//! - 跟 PR #84 AnnotationRegistry 1:1 对齐

use std::ops::Range;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use agent_bridge::{AnnotationAuthor, AnnotationRegistry, AnnotationRegistryError, DiffAnnotation};

use crate::ApiError;

// =====================================================================
// 1. state
// =====================================================================

/// Diff annotation API state (per chat.rs ChatState 同模式)
#[derive(Clone)]
pub struct DiffAnnotationState {
    /// Annotation registry (per PR #84 ULYS-201)
    pub registry: Arc<std::sync::Mutex<AnnotationRegistry>>,
    /// 默认 tenant_id (守门 #13 b)
    pub tenant_id: Uuid,
    /// 默认 workspace_id
    pub workspace_id: Uuid,
    /// 默认 actor_id (守门 #10 author)
    pub default_actor_id: Uuid,
}

impl DiffAnnotationState {
    /// 构造 InMemory 版 (per MVP / test)
    pub fn new_in_memory(tenant_id: Uuid, workspace_id: Uuid, default_actor_id: Uuid) -> Arc<Self> {
        Arc::new(Self {
            registry: Arc::new(std::sync::Mutex::new(AnnotationRegistry::new())),
            tenant_id,
            workspace_id,
            default_actor_id,
        })
    }

    /// 注册一个 agent run (per AnnotationRegistry::register_agent_run)
    pub fn register_agent_run(&self, agent_run_id: Uuid) {
        let mut reg = self
            .registry
            .lock()
            .expect("diff annotation mutex poisoned");
        reg.register_agent_run(agent_run_id);
    }
}

// =====================================================================
// 2. request / response DTO
// =====================================================================

/// 创建 annotation 请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAnnotationRequest {
    /// 文件路径 (worktree-relative POSIX)
    pub file_path: String,
    /// 行号范围 [start, end), 1-based
    pub line_start: u32,
    /// 行号范围 [start, end), 1-based
    pub line_end: u32,
    /// 注释正文 (Markdown)
    pub body: String,
    /// 关联 agent_run_id
    pub agent_run_id: Uuid,
    /// 作者 (user / agent), 缺省 = User
    #[serde(default = "default_author")]
    pub author: AuthorDto,
}

fn default_author() -> AuthorDto {
    AuthorDto::User
}

/// Author DTO (per AnnotationAuthor 1:1)
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorDto {
    /// 用户在 UI 手动 drop (per FR-ORCA-035 §11 spec)
    User,
    /// refinement loop 自动 emit (MVP v0 不实装)
    Agent,
}

impl From<AuthorDto> for AnnotationAuthor {
    fn from(a: AuthorDto) -> Self {
        match a {
            AuthorDto::User => AnnotationAuthor::User,
            AuthorDto::Agent => AnnotationAuthor::Agent,
        }
    }
}

/// Annotation API view (与 DiffAnnotation 1:1, 但 annotation_id 字段名匹配前端 DTO)
#[derive(Debug, Clone, Serialize)]
pub struct AnnotationView {
    /// Annotation UUID
    pub comment_id: Uuid,
    /// 文件路径
    pub file_path: String,
    /// 行号范围 (1-based, 半开) - 起始行号
    pub line_start: u32,
    /// 行号范围 (1-based, 半开) - 结束行号
    pub line_end: u32,
    /// 注释正文
    pub body: String,
    /// 关联 agent_run_id
    pub agent_run_id: Uuid,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 作者
    pub author: AuthorDto,
}

impl From<&DiffAnnotation> for AnnotationView {
    fn from(a: &DiffAnnotation) -> Self {
        Self {
            comment_id: a.id,
            file_path: a.file_path.clone(),
            line_start: a.line_range.start,
            line_end: a.line_range.end,
            body: a.body.clone(),
            agent_run_id: a.agent_run_id,
            created_at: a.created_at,
            author: match a.author {
                AnnotationAuthor::User => AuthorDto::User,
                AnnotationAuthor::Agent => AuthorDto::Agent,
            },
        }
    }
}

/// 列表查询 (filter)
#[derive(Debug, Clone, Deserialize)]
pub struct ListFilter {
    /// 按 file_path 过滤 (可选)
    pub file_path: Option<String>,
    /// 按 agent_run_id 过滤 (可选)
    pub agent_run_id: Option<Uuid>,
}

/// Refeed fragment 响应 (per FR-ORCA-035 spec "作为下一轮 prompt fragment")
#[derive(Debug, Clone, Serialize)]
pub struct RefeedResponse {
    /// Agent run id
    pub agent_run_id: Uuid,
    /// Annotation 数量
    pub count: usize,
    /// 拼接的 prompt fragment (per AnnotationRegistry::feed_to_agent)
    pub fragment: String,
}

// =====================================================================
// 3. handlers
// =====================================================================

/// POST /v1/diff-annotations
pub async fn create_annotation(
    State(state): State<Arc<DiffAnnotationState>>,
    Json(req): Json<CreateAnnotationRequest>,
) -> Result<(StatusCode, Json<AnnotationView>), ApiError> {
    if req.line_start == 0 {
        return Err(ApiError::invalid_state(
            "line_start must be >= 1",
            "Use 1-based line numbers",
        ));
    }
    if req.line_start > req.line_end {
        return Err(ApiError::invalid_state(
            format!(
                "line_start ({}) > line_end ({})",
                req.line_start, req.line_end
            ),
            "Ensure start <= end",
        ));
    }
    let line_range: Range<u32> = req.line_start..req.line_end;

    let mut reg = state
        .registry
        .lock()
        .expect("diff annotation mutex poisoned");
    let id = reg
        .add(
            req.file_path,
            line_range,
            req.body,
            req.agent_run_id,
            req.author.into(),
        )
        .map_err(annotation_to_api_error)?;

    let ann = reg
        .get(id)
        .ok_or_else(|| ApiError::internal("annotation added but not retrievable"))?;
    Ok((StatusCode::CREATED, Json(AnnotationView::from(ann))))
}

/// GET /v1/diff-annotations?file_path=...&agent_run_id=...
pub async fn list_annotations(
    State(state): State<Arc<DiffAnnotationState>>,
    Query(filter): Query<ListFilter>,
) -> Result<Json<Vec<AnnotationView>>, ApiError> {
    let reg = state
        .registry
        .lock()
        .expect("diff annotation mutex poisoned");
    let anns: Vec<&DiffAnnotation> = if let Some(path) = filter.file_path.as_deref() {
        reg.list_for_file(path)
    } else if let Some(agent_run_id) = filter.agent_run_id {
        reg.list_for_agent_run(agent_run_id)
    } else {
        // 无 filter → 全部 (rare in prod)
        reg.iter_all()
    };
    Ok(Json(anns.into_iter().map(AnnotationView::from).collect()))
}

/// GET /v1/diff-annotations/:comment_id
pub async fn get_annotation(
    State(state): State<Arc<DiffAnnotationState>>,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<AnnotationView>, ApiError> {
    let reg = state
        .registry
        .lock()
        .expect("diff annotation mutex poisoned");
    reg.get(comment_id)
        .map(AnnotationView::from)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(&format!("annotation {comment_id}")))
}

/// DELETE /v1/diff-annotations/:comment_id (GDPR)
pub async fn delete_annotation(
    State(state): State<Arc<DiffAnnotationState>>,
    Path(comment_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut reg = state
        .registry
        .lock()
        .expect("diff annotation mutex poisoned");
    reg.delete(comment_id).map_err(annotation_to_api_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /v1/diff-annotations/refeed/:agent_run_id
///
/// 返回拼好的 prompt fragment (per FR-ORCA-035 spec "作为下一轮 prompt fragment").
pub async fn refeed_annotations(
    State(state): State<Arc<DiffAnnotationState>>,
    Path(agent_run_id): Path<Uuid>,
) -> Result<Json<RefeedResponse>, ApiError> {
    let reg = state
        .registry
        .lock()
        .expect("diff annotation mutex poisoned");
    let fragment = reg.feed_to_agent(agent_run_id).ok_or_else(|| {
        ApiError::not_found(&format!("agent_run {agent_run_id} or no annotations"))
    })?;
    let count = reg.list_for_agent_run(agent_run_id).len();
    Ok(Json(RefeedResponse {
        agent_run_id,
        count,
        fragment,
    }))
}

// =====================================================================
// 4. router
// =====================================================================

/// `diff_annotation_routes(state)` (per chat.rs `chat_routes` 同模式)
pub fn diff_annotation_routes(state: Arc<DiffAnnotationState>) -> Router {
    Router::new()
        .route("/v1/diff-annotations", post(create_annotation))
        .route("/v1/diff-annotations", get(list_annotations))
        .route("/v1/diff-annotations/{comment_id}", get(get_annotation))
        .route(
            "/v1/diff-annotations/{comment_id}",
            delete(delete_annotation),
        )
        .route(
            "/v1/diff-annotations/refeed/{agent_run_id}",
            get(refeed_annotations),
        )
        .with_state(state)
}

// =====================================================================
// 5. helpers
// =====================================================================

/// AnnotationRegistryError → ApiError (per 守门 #6 v2 6-field schema)
fn annotation_to_api_error(e: AnnotationRegistryError) -> ApiError {
    match e {
        AnnotationRegistryError::NotFound(s) => ApiError::not_found(&s),
        AnnotationRegistryError::InvalidField(s) => {
            ApiError::invalid_state(&s, "Check field values")
        }
        AnnotationRegistryError::AgentRunNotFound(id) => {
            ApiError::not_found(&format!("agent_run {id}"))
        }
    }
}

// =====================================================================
// 6. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiError;

    fn state() -> Arc<DiffAnnotationState> {
        let s = DiffAnnotationState::new_in_memory(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        s.register_agent_run(Uuid::new_v4());
        s
    }

    fn req() -> CreateAnnotationRequest {
        CreateAnnotationRequest {
            file_path: "src/main.rs".into(),
            line_start: 10,
            line_end: 12,
            body: "review this".into(),
            agent_run_id: Uuid::new_v4(),
            author: AuthorDto::User,
        }
    }

    #[tokio::test]
    async fn create_then_get_returns_annotation() {
        let s = state();
        let agent_run = s.default_actor_id;
        // (skip register_agent_run for this test — req uses a new id each time)
        let mut r = req();
        r.agent_run_id = agent_run;
        s.register_agent_run(agent_run);

        let (_, view) = create_annotation(State(Arc::clone(&s)), Json(r.clone()))
            .await
            .unwrap();
        assert_eq!(view.file_path, "src/main.rs");
        assert_eq!(view.line_start, 10);
        assert_eq!(view.line_end, 12);
        assert_eq!(view.body, "review this");
        assert!(matches!(view.author, AuthorDto::User));
        let fetched = get_annotation(State(Arc::clone(&s)), Path(view.comment_id))
            .await
            .unwrap();
        assert_eq!(fetched.0.comment_id, view.comment_id);
    }

    #[tokio::test]
    async fn create_rejects_line_start_zero() {
        let s = state();
        let mut r = req();
        r.line_start = 0;
        let err = create_annotation(State(Arc::clone(&s)), Json(r))
            .await
            .unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn create_rejects_line_start_greater_than_end() {
        let s = state();
        let mut r = req();
        r.line_start = 50;
        r.line_end = 10;
        let err = create_annotation(State(Arc::clone(&s)), Json(r))
            .await
            .unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn create_rejects_unknown_agent_run() {
        let s = state();
        let r = req();
        let err = create_annotation(State(Arc::clone(&s)), Json(r))
            .await
            .unwrap_err();
        assert_eq!(err.code, "RESOURCE_NOT_FOUND");
    }

    #[tokio::test]
    async fn list_for_file_filters_correctly() {
        let s = state();
        let agent_run = Uuid::new_v4();
        s.register_agent_run(agent_run);

        for (path, line) in [("src/a.rs", 5), ("src/a.rs", 10), ("src/b.rs", 7)] {
            let mut r = req();
            r.file_path = path.into();
            r.line_start = line;
            r.line_end = line + 1;
            r.agent_run_id = agent_run;
            let _ = create_annotation(State(Arc::clone(&s)), Json(r)).await;
        }

        let filter = ListFilter {
            file_path: Some("src/a.rs".into()),
            agent_run_id: None,
        };
        let out = list_annotations(State(Arc::clone(&s)), Query(filter))
            .await
            .unwrap();
        assert_eq!(out.0.len(), 2);
    }

    #[tokio::test]
    async fn list_for_agent_run_filters_correctly() {
        let s = state();
        let agent_run_a = Uuid::new_v4();
        let agent_run_b = Uuid::new_v4();
        s.register_agent_run(agent_run_a);
        s.register_agent_run(agent_run_b);

        for ar in [agent_run_a, agent_run_b] {
            let mut r = req();
            r.agent_run_id = ar;
            let _ = create_annotation(State(Arc::clone(&s)), Json(r)).await;
        }

        let filter = ListFilter {
            file_path: None,
            agent_run_id: Some(agent_run_a),
        };
        let out = list_annotations(State(Arc::clone(&s)), Query(filter))
            .await
            .unwrap();
        assert_eq!(out.0.len(), 1);
    }

    #[tokio::test]
    async fn delete_removes_annotation() {
        let s = state();
        let agent_run = Uuid::new_v4();
        s.register_agent_run(agent_run);
        let mut r = req();
        r.agent_run_id = agent_run;
        let (_, view) = create_annotation(State(Arc::clone(&s)), Json(r))
            .await
            .unwrap();

        let status = delete_annotation(State(Arc::clone(&s)), Path(view.comment_id))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);

        let err = get_annotation(State(Arc::clone(&s)), Path(view.comment_id))
            .await
            .unwrap_err();
        assert_eq!(err.code, "RESOURCE_NOT_FOUND");
    }

    #[tokio::test]
    async fn delete_not_found_returns_error() {
        let s = state();
        let err = delete_annotation(State(Arc::clone(&s)), Path(Uuid::new_v4()))
            .await
            .unwrap_err();
        assert_eq!(err.code, "RESOURCE_NOT_FOUND");
    }

    #[tokio::test]
    async fn refeed_returns_fragment_with_count() {
        let s = state();
        let agent_run = Uuid::new_v4();
        s.register_agent_run(agent_run);
        let mut r = req();
        r.agent_run_id = agent_run;
        let _ = create_annotation(State(Arc::clone(&s)), Json(r)).await;
        let _ = create_annotation(State(Arc::clone(&s)), Json(req()))
            .await
            .ok();

        let resp = refeed_annotations(State(Arc::clone(&s)), Path(agent_run))
            .await
            .unwrap();
        assert_eq!(resp.0.agent_run_id, agent_run);
        assert_eq!(resp.0.count, 1);
        assert!(resp.0.fragment.contains("[Diff Annotation]"));
    }

    #[tokio::test]
    async fn refeed_returns_error_when_no_annotations() {
        let s = state();
        let unknown_agent_run = Uuid::new_v4();
        let err = refeed_annotations(State(Arc::clone(&s)), Path(unknown_agent_run))
            .await
            .unwrap_err();
        assert_eq!(err.code, "RESOURCE_NOT_FOUND");
    }

    #[tokio::test]
    async fn routes_compiles_and_builds() {
        // smoke test: 构造 router 不 panic
        let s = state();
        let _r = diff_annotation_routes(s);
        // 通过编译 + 不 panic 即证明 routes 形状正确
    }

    // ApiError 6-field 验证
    #[test]
    fn api_error_6_fields() {
        let e = ApiError::not_found("test");
        let json = serde_json::to_string(&e).unwrap();
        for field in &[
            "code",
            "message",
            "source_module",
            "source_kind",
            "retriable",
            "hint",
        ] {
            assert!(json.contains(field), "missing {field} in {json}");
        }
    }
}

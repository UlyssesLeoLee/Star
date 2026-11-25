// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/src/routes/defects.rs"})
MERGE (create:Symbol {id:"crates/star-api-rest/src/routes/defects.rs::create",kind:"function"})
MERGE (get:Symbol {id:"crates/star-api-rest/src/routes/defects.rs::get_by_id",kind:"function"})
MERGE (list:Symbol {id:"crates/star-api-rest/src/routes/defects.rs::list",kind:"function"})
MERGE (transition:Symbol {id:"crates/star-api-rest/src/routes/defects.rs::transition",kind:"function"})
MERGE (verify:Symbol {id:"crates/star-api-rest/src/routes/defects.rs::verify",kind:"function"})
MERGE (get_verifs:Symbol {id:"crates/star-api-rest/src/routes/defects.rs::list_verifications",kind:"function"})
MERGE (self)-[:DEFINES]->(create)
MERGE (self)-[:DEFINES]->(get)
MERGE (self)-[:DEFINES]->(list)
MERGE (self)-[:DEFINES]->(transition)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(get_verifs)
*/
//! 缺陷管理 REST 路由 (per Phase C: defects.rs 接入 star-api-rest)
//!
//! 范围: 缺陷创建/查询/状态流转/独立验证，复用 WorkItem 的 Bug 类型稳定身份。
//! 授权: 基于 ActorContext 的 tenant/Project/Lane/Worktree 归属与能力校验。
//! 幂等: 所有写入命令携带 idempotency_key，命中返回缓存回执。
//! 当前实现: PG 适配器 + InMemory 降级（生产未接入 defects.rs 路由前为 501）。

use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use domain_defect::{
    ActorContext, CreateDefectCommand, Defect, DefectError, DefectId,
    DefectListItem, DefectCommandPort, DefectQueryPort, DefectPort, DefectSeverity, DefectStatus, DefectVerification,
    IdempotencyKey, InMemoryDefectService, PgDefectService, SubmitVerificationCommand,
    TransitionDefectCommand, VerificationId,
};
use serde::Deserialize;
use serde::Serialize;

use std::sync::Arc;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::RestError;
use crate::response::RestResponse;
use serde_json::json;
use sqlx::PgPool;

/// 缺陷服务（生产 PG，开发/测试 InMemory）
#[derive(Clone)]
pub struct DefectService {
    pg: Option<Arc<PgDefectService>>,
    memory: Arc<InMemoryDefectService>,
}

impl DefectService {
    pub fn new(pg_pool: Option<sqlx::PgPool>) -> Self {
        let pg = pg_pool.map(|pool| Arc::new(PgDefectService::new(pool)));
        Self {
            pg,
            memory: Arc::new(InMemoryDefectService::new()),
        }
    }
}

/// Implement DefectCommandPort for DefectService (delegating to inner service)
#[async_trait::async_trait]
impl domain_defect::DefectCommandPort for DefectService {
    async fn create_defect(
        &self,
        cmd: domain_defect::CreateDefectCommand,
        actor: &domain_defect::ActorContext,
    ) -> Result<domain_defect::Defect, domain_defect::DefectError> {
        if let Some(pg) = &self.pg {
            pg.create_defect(cmd, actor).await
        } else {
            self.memory.create_defect(cmd, actor).await
        }
    }

    async fn transition_status(
        &self,
        defect_id: domain_defect::DefectId,
        cmd: domain_defect::TransitionDefectCommand,
        actor: &domain_defect::ActorContext,
    ) -> Result<domain_defect::Defect, domain_defect::DefectError> {
        if let Some(pg) = &self.pg {
            pg.transition_status(defect_id, cmd, actor).await
        } else {
            self.memory.transition_status(defect_id, cmd, actor).await
        }
    }

    async fn submit_verification(
        &self,
        defect_id: domain_defect::DefectId,
        cmd: domain_defect::SubmitVerificationCommand,
        actor: &domain_defect::ActorContext,
    ) -> Result<(domain_defect::Defect, domain_defect::DefectVerification), domain_defect::DefectError> {
        if let Some(pg) = &self.pg {
            pg.submit_verification(defect_id, cmd, actor).await
        } else {
            self.memory.submit_verification(defect_id, cmd, actor).await
        }
    }
}

/// Implement DefectQueryPort for DefectService (delegating to inner service)
#[async_trait::async_trait]
impl domain_defect::DefectQueryPort for DefectService {
    async fn get_defect(
        &self,
        defect_id: domain_defect::DefectId,
        actor: &domain_defect::ActorContext,
    ) -> Result<domain_defect::Defect, domain_defect::DefectError> {
        if let Some(pg) = &self.pg {
            pg.get_defect(defect_id, actor).await
        } else {
            self.memory.get_defect(defect_id, actor).await
        }
    }

    async fn list_defects(
        &self,
        project_id: uuid::Uuid,
        lane_id: Option<uuid::Uuid>,
        actor: &domain_defect::ActorContext,
    ) -> Result<Vec<domain_defect::DefectListItem>, domain_defect::DefectError> {
        if let Some(pg) = &self.pg {
            pg.list_defects(project_id, lane_id, actor).await
        } else {
            self.memory.list_defects(project_id, lane_id, actor).await
        }
    }

    async fn list_verifications(
        &self,
        defect_id: domain_defect::DefectId,
        actor: &domain_defect::ActorContext,
    ) -> Result<Vec<domain_defect::DefectVerification>, domain_defect::DefectError> {
        if let Some(pg) = &self.pg {
            pg.list_verifications(defect_id, actor).await
        } else {
            self.memory.list_verifications(defect_id, actor).await
        }
    }
}

/// 全局服务（生产注入 pg_pool；未配置 DATABASE_URL 时降级到 InMemory）
static DEFECT_SERVICE: std::sync::OnceLock<DefectService> = std::sync::OnceLock::new();

/// 生产 PG 连接池来源。
///
/// - `STAR_DEFECT_DATABASE_URL` 优先（缺陷域专用，便于灰度与隔离库）
/// - 回退 `DATABASE_URL`（与 `routes/im.rs` / `routes/notifications.rs` 同一口径）
/// - 两者皆缺 → `None` → 显式降级到 InMemory，并在 `/defects` 响应的
///   `degraded` 字段标注降级原因（per G-16 "保留显式降级开关与降级原因标注"）。
///
/// **不因配置缺失而 panic 或静默放行**：拿不到池时服务仍可用，但状态可被查询到。
pub fn defect_pg_pool() -> Option<PgPool> {
    static POOL: std::sync::OnceLock<Option<PgPool>> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        let url = std::env::var("STAR_DEFECT_DATABASE_URL")
            .ok()
            .or_else(|| std::env::var("DATABASE_URL").ok())?;
        // `connect_lazy` 不做 I/O，可在 `OnceLock::get_or_init` 闭包内同步调用
        // （闭包不能 async）。连接失败在首次查询时暴露，不在此处 panic。
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(8)
            .connect_lazy(&url)
        {
            Ok(pool) => Some(pool),
            Err(e) => {
                tracing::error!(
                    feature = "DefectService",
                    error = %e,
                    "缺陷域 PG 连接池构造失败 — 降级到 InMemory（生产不可用）"
                );
                None
            }
        }
    })
    .clone()
}

/// 服务构造：优先真实 PG；无池时降级 InMemory 并记录降级原因。
fn service() -> &'static DefectService {
    DEFECT_SERVICE.get_or_init(|| {
        let pool = defect_pg_pool();
        if pool.is_none() {
            tracing::warn!(
                feature = "DefectService",
                "缺陷域未配置 STAR_DEFECT_DATABASE_URL / DATABASE_URL — 使用 InMemory 降级（数据不持久、无真实授权隔离）"
            );
        }
        DefectService::new(pool)
    })
}

/// 服务是否处于显式降级状态（InMemory）。
///
/// 前端与运维据此把"未接入"如实呈现，而不是当作生产事实（per G-16 / FR-DI-006）。
pub fn is_degraded() -> bool {
    service().pg.is_none()
}

/// 降级原因（人类可读），供 `/defects` 响应标注。
pub fn degradation_reason() -> Option<&'static str> {
    if is_degraded() {
        Some(
            "缺陷域未连接 PostgreSQL（STAR_DEFECT_DATABASE_URL / DATABASE_URL 未配置或连接失败），             当前为 InMemory 降级：数据不持久、无真实跨租户隔离。",
        )
    } else {
        None
    }
}

/// 已验证会话身份（fail-closed）。
///
/// - 生产：由 `verify_jwt_and_inject_actor` middleware 验签 JWT 后注入
///   `AuthUser` request extension；此处只读取**已验证**的 extension。
/// - 缺 extension → 401，**不做任何 header 兜底**（不读 `X-User-Id`，
///   不生成随机 UUID）——这是 G-15 的核心要求。
pub struct VerifiedDefectActor(pub AuthUser);

impl<S> FromRequestParts<S> for VerifiedDefectActor
where
    S: Send + Sync,
{
    type Rejection = NotAuthenticated;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(VerifiedDefectActor)
            .ok_or(NotAuthenticated)
    }
}

/// 未认证拒绝（extractor rejection）：缺已验证会话 → 401。
pub struct NotAuthenticated;

impl IntoResponse for NotAuthenticated {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "AUTH_REQUIRED",
                "error_description": "verified session required (JWT-authenticated actor missing on request)",
            })),
        )
            .into_response()
    }
}

/// 已验证会话 → 领域 `ActorContext`。
///
/// 身份、租户、角色、Project 归属**全部来自已验签的 claims**：
/// - `user_id` / `tenant_id`：claims（nil 一律拒绝，INV-ACT-01）
/// - `roles`：claims 角色列表（空则不赋予任何写能力）
/// - `project_ids`：claims 的 issuer-verified project memberships
///   （空集 → 非 tenant_admin 的 Project 访问一律被拒，per `domain-project` 语义）
fn actor_from_session(user: &AuthUser) -> Result<ActorContext, RestError> {
    if user.user_id.is_nil() || user.tenant_id.is_nil() {
        return Err(RestError::from(DefectError::PermissionDenied(
            "会话缺少有效身份（user_id / tenant_id 为 nil）".into(),
        )));
    }
    let mut actor = ActorContext::new(user.user_id, user.tenant_id);
    // ActorContext::new 默认带 "developer"；此处按 claims 重置为真实角色集合，
    // 避免"未声明角色却拥有 developer 权限"。
    actor.roles = user.roles.clone();
    if actor.roles.is_empty() {
        actor.roles = vec!["viewer".to_string()];
    }
    for project_id in &user.projects {
        actor = actor.with_project(*project_id);
    }
    Ok(actor)
}

/// 创建缺陷
pub async fn create_defect(
    State(_svc): State<()>,
    VerifiedDefectActor(session): VerifiedDefectActor,
    Json(cmd): Json<CreateDefectCommand>,
) -> Result<Json<RestResponse<Defect>>, RestError> {
    let actor = actor_from_session(&session)?;
    let svc = service();
    let defect = svc.create_defect(cmd, &actor).await?;
    Ok(Json(RestResponse::ok(defect)))
}

/// 获取缺陷详情
pub async fn get_defect(
    State(_svc): State<()>,
    VerifiedDefectActor(session): VerifiedDefectActor,
    Path(id): Path<Uuid>,
) -> Result<Json<RestResponse<Defect>>, RestError> {
    let actor = actor_from_session(&session)?;
    let svc = service();
    let defect = svc.get_defect(DefectId::from(id), &actor).await?;
    Ok(Json(RestResponse::ok(defect)))
}

/// 列表查询参数
#[derive(Deserialize)]
pub struct ListDefectsQuery {
    pub project_id: Uuid,
    pub lane_id: Option<Uuid>,
    /// 页码（1 起）；缺省 1
    pub page: Option<usize>,
    /// 每页条数；缺省 50，上限 200
    pub page_size: Option<usize>,
}

impl ListDefectsQuery {
    /// 归一化后的页码（1 起，最小 1）
    pub fn page(&self) -> usize {
        self.page.unwrap_or(1).max(1)
    }
    /// 归一化后的每页条数（默认 50，上限 200）
    pub fn page_size(&self) -> usize {
        self.page_size.unwrap_or(50).clamp(1, 200)
    }
    /// 该页的起始偏移
    pub fn offset(&self) -> usize {
        (self.page() - 1) * self.page_size()
    }
}

/// 缺陷列表响应（含分页元信息与降级标注）。
///
/// `total` 为**过滤后**的授权范围内总数（不是当前页条数），
/// `degraded` / `degradation_reason` 如实标注 InMemory 降级状态，
/// 使前端能区分"真实事实"与"降级结果"（per G-16 / FR-DI-006）。
#[derive(Serialize)]
pub struct ListDefectsResponse {
    pub items: Vec<DefectListItem>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degradation_reason: Option<&'static str>,
}

/// 缺陷列表
pub async fn list_defects(
    State(_svc): State<()>,
    VerifiedDefectActor(session): VerifiedDefectActor,
    Query(query): Query<ListDefectsQuery>,
) -> Result<Json<RestResponse<ListDefectsResponse>>, RestError> {
    let actor = actor_from_session(&session)?;
    let svc = service();
    let all = svc.list_defects(query.project_id, query.lane_id, &actor).await?;
    let total = all.len();
    let items: Vec<DefectListItem> = all
        .into_iter()
        .skip(query.offset())
        .take(query.page_size())
        .collect();
    Ok(Json(RestResponse::ok(ListDefectsResponse {
        items,
        total,
        page: query.page(),
        page_size: query.page_size(),
        degraded: is_degraded(),
        degradation_reason: degradation_reason(),
    })))
}

/// 状态流转
pub async fn transition_defect(
    State(_svc): State<()>,
    VerifiedDefectActor(session): VerifiedDefectActor,
    Path(id): Path<Uuid>,
    Json(cmd): Json<TransitionDefectCommand>,
) -> Result<Json<RestResponse<Defect>>, RestError> {
    let actor = actor_from_session(&session)?;
    let svc = service();
    let defect = svc.transition_status(DefectId::from(id), cmd, &actor).await?;
    Ok(Json(RestResponse::ok(defect)))
}

/// 验证提交
pub async fn submit_verification(
    State(_svc): State<()>,
    VerifiedDefectActor(session): VerifiedDefectActor,
    Path(id): Path<Uuid>,
    Json(cmd): Json<SubmitVerificationCommand>,
) -> Result<Json<RestResponse<(Defect, DefectVerification)>>, RestError> {
    let actor = actor_from_session(&session)?;
    let svc = service();
    let (defect, verification) = svc.submit_verification(DefectId::from(id), cmd, &actor).await?;
    Ok(Json(RestResponse::ok((defect, verification))))
}

/// 验证记录列表
pub async fn list_verifications(
    State(_svc): State<()>,
    VerifiedDefectActor(session): VerifiedDefectActor,
    Path(id): Path<Uuid>,
) -> Result<Json<RestResponse<Vec<DefectVerification>>>, RestError> {
    let actor = actor_from_session(&session)?;
    let svc = service();
    let items = svc.list_verifications(DefectId::from(id), &actor).await?;
    Ok(Json(RestResponse::ok(items)))
}

/// 路由注册（生产：JWT 验签 + fail-closed 身份注入）
///
/// - `verify_jwt_and_inject_actor` 与 `routes/notifications.rs` / `routes/im.rs`
///   **同一套生产鉴权**：验签失败 → 401，claims 不一致 → 401。
/// - handler 通过 `VerifiedDefectActor` extractor 读取已验证 extension；
///   缺 extension → 401（不做 header 兜底，per G-15）。
pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/defects", axum::routing::post(create_defect))
        .route("/defects", axum::routing::get(list_defects))
        .route("/defects/{id}", axum::routing::get(get_defect))
        .route("/defects/{id}/transition", axum::routing::patch(transition_defect))
        .route("/defects/{id}/verify", axum::routing::post(submit_verification))
        .route("/defects/{id}/verifications", axum::routing::get(list_verifications))
        .layer(axum::middleware::from_fn_with_state(
            crate::routes::notifications::prod_jwt_config(),
            crate::routes::notifications::verify_jwt_and_inject_actor,
        ))
        .layer(axum::middleware::from_fn(crate::middleware::rate_limit::rate_limit_layer_stub))
        .layer(axum::middleware::from_fn(crate::middleware::audit::audit_layer_stub))
}

/// 测试用路由：不做 JWT 验签，但**仍要求** request extension 内存在
/// 已验证 `AuthUser`（测试经 `.extension(AuthUser{..})` 注入）。
///
/// 与生产 `router()` 的差别仅在中间件；handler 的 fail-closed 行为一致。
pub fn router_with_verified_state() -> axum::Router {
    axum::Router::new()
        .route("/defects", axum::routing::post(create_defect))
        .route("/defects", axum::routing::get(list_defects))
        .route("/defects/{id}", axum::routing::get(get_defect))
        .route("/defects/{id}/transition", axum::routing::patch(transition_defect))
        .route("/defects/{id}/verify", axum::routing::post(submit_verification))
        .route("/defects/{id}/verifications", axum::routing::get(list_verifications))
        .layer(axum::middleware::from_fn(crate::middleware::rate_limit::rate_limit_layer_stub))
        .layer(axum::middleware::from_fn(crate::middleware::audit::audit_layer_stub))
}

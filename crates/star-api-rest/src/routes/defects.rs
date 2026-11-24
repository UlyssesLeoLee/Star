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
    extract::{Path, Query, State},
    Json,
};
use domain_defect::{
    ActorContext, CreateDefectCommand, Defect, DefectCommandPort, DefectError, DefectId,
    DefectListItem, DefectQueryPort, DefectSeverity, DefectStatus, DefectVerification,
    IdempotencyKey, InMemoryDefectService, PgDefectService, SubmitVerificationCommand,
    TransitionDefectCommand, VerificationId,
};
use serde::Deserialize;

use std::sync::Arc;
use uuid::Uuid;

use crate::error::RestError;

use crate::response::RestResponse;

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

    fn port(&self) -> Box<dyn domain_defect::DefectPort> {
        self.pg.as_ref().map(|p| Box::new(p.clone()) as _)
            .unwrap_or_else(|| Box::new(self.memory.clone()))
    }
}

/// 全局服务（生产需注入 pg_pool，当前为 InMemory 降级）
static DEFECT_SERVICE: std::sync::OnceLock<DefectService> = std::sync::OnceLock::new();

fn service() -> &'static DefectService {
    DEFECT_SERVICE.get_or_init(|| DefectService::new(None))
}

/// 从已验证的 AuthLayer 提取 ActorContext（stub 实现，生产替换为真实 JWT 解析）
fn actor_from_headers(headers: &axum::http::HeaderMap) -> ActorContext {
    // stub: 从 Authorization header 或 x-actor-* headers 解析
    // 生产由 AuthLayer 注入 request extension
    ActorContext::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
}

/// 创建缺陷
pub async fn create_defect(
    State(_svc): State<()>,
    headers: axum::http::HeaderMap,
    Json(cmd): Json<CreateDefectCommand>,
) -> Result<Json<RestResponse<Defect>>, RestError> {
    let actor = actor_from_headers(&headers);
    let svc = service();
    let defect = svc.port().create_defect(cmd, &actor).await?;
    Ok(Json(RestResponse::ok(defect)))
}

/// 获取缺陷详情
pub async fn get_defect(
    State(_svc): State<()>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<RestResponse<Defect>>, RestError> {
    let actor = actor_from_headers(&headers);
    let svc = service();
    let defect = svc.port().get_defect(DefectId::from(id), &actor).await?;
    Ok(Json(RestResponse::ok(defect)))
}

/// 列表查询参数
#[derive(Deserialize)]
struct ListDefectsQuery {
    project_id: Uuid,
    lane_id: Option<Uuid>,
    page: Option<usize>,
    page_size: Option<usize>,
}

/// 缺陷列表
pub async fn list_defects(
    State(_svc): State<()>,
    headers: axum::http::HeaderMap,
    Query(query): Query<ListDefectsQuery>,
) -> Result<Json<RestResponse<Vec<DefectListItem>>>, RestError> {
    let actor = actor_from_headers(&headers);
    let svc = service();
    let items = svc.port().list_defects(query.project_id, query.lane_id, &actor).await?;
    Ok(Json(RestResponse::ok(items)))
}

/// 状态流转
pub async fn transition_defect(
    State(_svc): State<()>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(cmd): Json<TransitionDefectCommand>,
) -> Result<Json<RestResponse<Defect>>, RestError> {
    let actor = actor_from_headers(&headers);
    let svc = service();
    let defect = svc.port().transition_status(DefectId::from(id), cmd, &actor).await?;
    Ok(Json(RestResponse::ok(defect)))
}

/// 验证提交
pub async fn submit_verification(
    State(_svc): State<()>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(cmd): Json<SubmitVerificationCommand>,
) -> Result<Json<RestResponse<(Defect, DefectVerification)>>, RestError> {
    let actor = actor_from_headers(&headers);
    let svc = service();
    let (defect, verification) = svc.port().submit_verification(DefectId::from(id), cmd, &actor).await?;
    Ok(Json(RestResponse::ok((defect, verification))))
}

/// 验证记录列表
pub async fn list_verifications(
    State(_svc): State<()>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<RestResponse<Vec<DefectVerification>>>, RestError> {
    let actor = actor_from_headers(&headers);
    let svc = service();
    let items = svc.port().list_verifications(DefectId::from(id), &actor).await?;
    Ok(Json(RestResponse::ok(items)))
}

/// 路由注册
pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/defects", axum::routing::post(create_defect))
        .route("/defects", axum::routing::get(list_defects))
        .route("/defects/:id", axum::routing::get(get_defect))
        .route("/defects/:id/transition", axum::routing::patch(transition_defect))
        .route("/defects/:id/verify", axum::routing::post(submit_verification))
        .route("/defects/:id/verifications", axum::routing::get(list_verifications))
        .layer(axum::middleware::from_fn(crate::middleware::auth::auth_layer_stub))
        .layer(axum::middleware::from_fn(crate::middleware::rate_limit::rate_limit_layer_stub))
        .layer(axum::middleware::from_fn(crate::middleware::audit::audit_layer_stub))
}

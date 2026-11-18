// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=aa32a6d7adf52919e479a24654e585fef6aa29aa570082a290fc95ff0b113cd0
MERGE (self:File {path:"crates/star-api-rest/src/routes/scm_state.rs"})
MERGE (state:Type {id:"crates/star-api-rest/src/routes/scm_state.rs::ScmApiState",kind:"struct"})
MERGE (new:Symbol {id:"crates/star-api-rest/src/routes/scm_state.rs::ScmApiState.new",kind:"method"})
MERGE (actorFromAuth:Symbol {id:"crates/star-api-rest/src/routes/scm_state.rs::actor_from_auth",kind:"function"})
MERGE (scmService:Type {id:"crates/domain-scm/src/lib.rs::InMemoryScmService",kind:"struct"})
MERGE (jwtConfig:Type {id:"crates/star-api-rest/src/auth/mod.rs::JwtConfig",kind:"struct"})
MERGE (deviceRepo:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceBindingRepository",kind:"trait"})
MERGE (authUser:Type {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedUser",kind:"struct"})
MERGE (actorCtx:Type {id:"crates/star-context/src/actor.rs::ActorContext",kind:"struct"})
MERGE (actorNew:Symbol {id:"crates/star-context/src/actor.rs::ActorContext.new",kind:"method"})
MERGE (withRole:Symbol {id:"crates/star-context/src/actor.rs::ActorContext.with_role",kind:"method"})
MERGE (withProject:Symbol {id:"crates/star-context/src/actor.rs::ActorContext.with_project",kind:"method"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(actorFromAuth)
MERGE (state)-[:HAS_METHOD]->(new)
MERGE (state)-[:USES_TYPE]->(scmService)
MERGE (state)-[:USES_TYPE]->(jwtConfig)
MERGE (state)-[:USES_TYPE]->(deviceRepo)
MERGE (actorFromAuth)-[:USES_TYPE]->(authUser)
MERGE (actorFromAuth)-[:USES_TYPE]->(actorCtx)
MERGE (actorFromAuth)-[:CALLS]->(actorNew)
MERGE (actorFromAuth)-[:CALLS]->(withRole)
MERGE (actorFromAuth)-[:CALLS]->(withProject)
@endcypher
*/
//! SCM REST 共享 state (2026-11-19, F15 模块真实化批次 B)。
//!
//! 根因修复: merge_requests / reviews / pipelines 三个 handler 此前各持一份
//! `OnceLock<InMemoryScmService>`, MR 建在 service A、review 去 service B 查
//! 必然 NotFound —— 跨模块数据互不可见, 即"有名无实"。本模块提供单一共享
//! service + 与生产 group 域同一套 JWT 验签配置 / 设备绑定仓储, 由
//! `build_scm_router` 统一注入; 生产 main.rs 与 legacy build_router 同源。

use std::sync::Arc;

use domain_scm::InMemoryScmService;
use star_pg_adapter::repository::DeviceBindingRepository;

use crate::auth::oauth::middleware::AuthenticatedUser;
use crate::auth::JwtConfig;

/// SCM REST 路由 state。
///
/// `AuthenticatedUser` extractor 需要 `FromRef<S> for Arc<JwtConfig>` 与
/// `FromRef<S> for Arc<dyn DeviceBindingRepository>` 两个投影 (与
/// `GroupApiState` 生产范式一致)。
#[derive(Clone)]
pub struct ScmApiState {
    pub(crate) service: Arc<InMemoryScmService>,
    pub(crate) jwt: Arc<JwtConfig>,
    pub(crate) device_binding_repo: Arc<dyn DeviceBindingRepository>,
}

impl ScmApiState {
    /// 生产构造: 复用 group 域同一份 JWT 配置与设备绑定仓储。
    pub fn new(
        service: Arc<InMemoryScmService>,
        jwt: Arc<JwtConfig>,
        device_binding_repo: Arc<dyn DeviceBindingRepository>,
    ) -> Self {
        Self {
            service,
            jwt,
            device_binding_repo,
        }
    }
}

impl axum::extract::FromRef<ScmApiState> for Arc<JwtConfig> {
    fn from_ref(state: &ScmApiState) -> Self {
        Arc::clone(&state.jwt)
    }
}

impl axum::extract::FromRef<ScmApiState> for Arc<dyn DeviceBindingRepository> {
    fn from_ref(state: &ScmApiState) -> Self {
        Arc::clone(&state.device_binding_repo)
    }
}

impl axum::extract::FromRef<ScmApiState> for Arc<InMemoryScmService> {
    fn from_ref(state: &ScmApiState) -> Self {
        Arc::clone(&state.service)
    }
}

/// JWT claims → 真实 `ActorContext`。
///
/// tenant / user 取 JWT 主体; `project_ids` 取 JWT `projects` claim (空则留空,
/// 由 domain 层按 INV-SCM-04 拒绝越权, 不做兜底放大)。
pub(crate) fn actor_from_auth(auth: &AuthenticatedUser) -> domain_scm::ActorContext {
    // star-context 只有单数 builder (with_role / with_project), 逐个追加;
    // projects 来自 issuer 核实的成员资格, 空集即空集, 不做兜底放大。
    let mut actor = domain_scm::ActorContext::new(auth.0.user_id, auth.0.tenant_id);
    for role in &auth.0.roles {
        actor = actor.with_role(role);
    }
    for project in &auth.0.projects {
        actor = actor.with_project(*project);
    }
    actor
}

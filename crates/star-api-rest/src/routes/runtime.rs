/*
@cypher schema=1 source_sha256=1b72bf47beadf61be9e79162b5885e47c414cd4b9b79578baffd59343ffb8415
MERGE (self:File {path:"crates/star-api-rest/src/routes/runtime.rs"})
MERGE (svc:Type {id:"crates/domain-local-runtime/src/lib.rs::InMemoryRuntimeService",kind:"struct"})
MERGE (runtime:Type {id:"crates/domain-local-runtime/src/lib.rs::LocalRuntime",kind:"struct"})
MERGE (hb:Type {id:"crates/domain-local-runtime/src/lib.rs::RuntimeHeartbeat",kind:"struct"})
MERGE (mount:Type {id:"crates/domain-local-runtime/src/lib.rs::WorktreeMount",kind:"struct"})
MERGE (cmdport:Symbol {id:"crates/domain-local-runtime/src/lib.rs::RuntimeCommandPort",kind:"trait"})
MERGE (qport:Symbol {id:"crates/domain-local-runtime/src/lib.rs::RuntimeQueryPort",kind:"trait"})
MERGE (self)-[:USES]->(svc)
MERGE (self)-[:USES]->(runtime)
MERGE (self)-[:USES]->(hb)
MERGE (self)-[:USES]->(mount)
MERGE (self)-[:USES]->(cmdport)
MERGE (self)-[:USES]->(qport)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// runtime.rs — Local Runtime 端点 (domain-local-runtime 内存态门面, 2026-11-16 新增)
//
//   POST /api/v1/runtime/register               — 注册 Local Runtime, 返回 runtime_id
//   POST /api/v1/runtime/{runtime_id}/heartbeat — 心跳, 返回 accepted
//   POST /api/v1/runtime/{runtime_id}/mount     — 挂载 Worktree, 返回 mounted
//   GET  /api/v1/runtime?user_id=               — 列出运行时 (user_id 可选)
//   GET  /api/v1/runtime/{runtime_id}/heartbeats?limit= — 最近心跳列表 (倒序)
//
// 请求体字段 = domain-local-runtime 命令 DTO 的真实字段 (RegisterRuntimeCommand /
// HeartbeatCommand / MountWorktreeCommand, per lib.rs §命令/查询 DTO), 不自行发明:
// - register: tenant_id / user_id / device_id / version / capabilities / metadata
//   (hostname 约定放 metadata["host_name"]; 命令本身无 worktree_id 字段, 挂载走 mount 端点)
// - heartbeat: tenant_id / load_average / memory_used_bytes (runtime_id 来自路径)
// - mount: tenant_id / worktree_id / local_path (runtime_id 来自路径)
//
// 内存态 State (Arc<Mutex<HashMap>>) — 持久化 PostgreSQL 接入后替换:
// - 写命令走 RuntimeCommandPort (register/heartbeat/mount, service 内部自持状态)
// - runtimes 索引 (GET 列表 + tenant 回查) 由 SQL 查询替换
// - 心跳流读取走 RuntimeQueryPort::get_heartbeats (service 内部 Append-only 流)
//
// 鉴权: 中间件实装前, ActorContext 由请求体 tenant/user 派生 (is_local_runtime=true);
// 实装后从 JWT claim 注入 (同 routes/memory.rs tenant 占位模式, per 守门 #13 a)
// =====================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use domain_local_runtime::{
    ActorContext, DeviceId, GetHeartbeatsQuery, HeartbeatCommand, InMemoryRuntimeService,
    LocalRuntime, LocalRuntimeId, MountWorktreeCommand, RegisterRuntimeCommand, RuntimeCommandPort,
    RuntimeError, RuntimeQueryPort, TenantId, UserId, WorktreeId,
};

use crate::error::RestError;
use crate::response::RestResponse;

// ---------------------------------------------------------------------
// 共享状态 (内存态, 持久化 PostgreSQL 接入后替换)
// ---------------------------------------------------------------------

/// Runtime 端点共享状态。
///
/// 内存态 (`Arc<Mutex<HashMap>>`) — 持久化 PostgreSQL 接入后替换:
/// - `service`: 领域命令/查询入口 (`InMemoryRuntimeService` → PgPool + repository)
/// - `runtimes`: runtime_id → LocalRuntime 快照索引, GET 列表 + tenant 回查用
///   (service 无 list_all 公开 API, 列表端点依赖此索引; 持久化后由 SQL 查询替换)
pub struct RuntimeApiState {
    /// 领域服务 (RuntimeCommandPort + RuntimeQueryPort 内存实现)
    pub service: Arc<InMemoryRuntimeService>,
    /// runtime_id → LocalRuntime 快照索引 (PostgreSQL 接入后由 SQL 查询替换)
    pub runtimes: Arc<Mutex<HashMap<Uuid, LocalRuntime>>>,
}

/// 测试/多实例隔离用: 每次调用返回全新状态
pub fn default_state() -> Arc<RuntimeApiState> {
    Arc::new(RuntimeApiState {
        service: Arc::new(InMemoryRuntimeService::new()),
        runtimes: Arc::new(Mutex::new(HashMap::new())),
    })
}

/// 生产 build_router 用共享状态 (OnceLock)
fn state_arc() -> Arc<RuntimeApiState> {
    static STATE: OnceLock<Arc<RuntimeApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(default_state))
}

/// 生产嵌套路由 (共享 OnceLock 状态)
pub fn router() -> Router {
    router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (集成测试隔离用, per 守门 #25 v25)
pub fn router_with_state(state: Arc<RuntimeApiState>) -> Router {
    Router::new()
        .route("/", get(list_runtimes))
        .route("/register", post(register_runtime))
        .route("/{runtime_id}/heartbeat", post(heartbeat))
        .route("/{runtime_id}/mount", post(mount_worktree))
        .route("/{runtime_id}/heartbeats", get(list_heartbeats))
        .with_state(state)
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 未知 runtime_id → RUNTIME_NOT_FOUND (404)
fn runtime_not_found(runtime_id: Uuid) -> RestError {
    RuntimeError::NotFound(format!("runtime:{runtime_id}")).into()
}

/// 由索引中的 runtime owner 派生 ActorContext (内存态占位):
/// 鉴权中间件实装后从 JWT claim 注入 (per 守门 #13 a RLS 13 类)。
///
/// owner 的 tenant_id / user_id 在 register 时已校验非 nil, 此处构造不会
/// 触发 `ActorContext::new` 的 INV-ACT-01 panic。
fn owner_actor(owner: &LocalRuntime) -> ActorContext {
    ActorContext::new(owner.user_id.0, owner.tenant_id.0).as_local_runtime()
}

// ---------------------------------------------------------------------
// request / query DTO (字段与 domain-local-runtime 命令 DTO 一一对应)
// ---------------------------------------------------------------------

/// `POST /api/v1/runtime/register` body — 对应 `RegisterRuntimeCommand` 真实字段
#[derive(Debug, Deserialize)]
pub struct RegisterRuntimeRequest {
    /// 租户 ID (INV-RT-01 必带)
    pub tenant_id: Uuid,
    /// 用户 ID (INV-RT-05: 1 Runtime → 1 User)
    pub user_id: Uuid,
    /// 设备 ID ((tenant, user, device) 三元组唯一)
    pub device_id: Uuid,
    /// 运行时版本, 如 "1.0.0"
    pub version: String,
    /// 能力清单 (git / docker / rust / ...)
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// 元数据 (hostname 约定放 `metadata["host_name"]`, os / arch 同放此表)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// `POST /api/v1/runtime/{runtime_id}/heartbeat` body — 对应 `HeartbeatCommand` 真实字段
/// (runtime_id 来自路径; 无 status 字段 — 状态由领域逻辑从心跳推导, per INV-RT-04)
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// load average (1-min, 可选)
    pub load_average: Option<f32>,
    /// 已用内存字节数 (可选)
    pub memory_used_bytes: Option<u64>,
}

/// `POST /api/v1/runtime/{runtime_id}/mount` body — 对应 `MountWorktreeCommand` 真实字段
/// (runtime_id 来自路径)
#[derive(Debug, Deserialize)]
pub struct MountWorktreeRequest {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 挂载的 Worktree ID (INV-RT-02 必带)
    pub worktree_id: Uuid,
    /// 平台相关本地路径 — 仅作引用, 平台不可信 (INV-RT-03 / INV-WT-04)
    pub local_path: String,
}

/// `GET /api/v1/runtime` query (user_id 可选)
#[derive(Debug, Deserialize)]
pub struct ListRuntimesParams {
    /// 按用户过滤 (可选; 缺省列出全部)
    pub user_id: Option<Uuid>,
}

/// `GET /api/v1/runtime/{runtime_id}/heartbeats` query
#[derive(Debug, Deserialize)]
pub struct ListHeartbeatsParams {
    /// 返回条数上限 (可选, 默认全量, 按 received_at 倒序)
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------
// handlers
// ---------------------------------------------------------------------

/// `POST /api/v1/runtime/register` — 注册 Local Runtime (调 register 命令), 返回 runtime_id
pub async fn register_runtime(
    State(state): State<Arc<RuntimeApiState>>,
    Json(req): Json<RegisterRuntimeRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if req.tenant_id.is_nil() {
        return Err(RestError::validation(
            "tenant_id 不能为 nil (INV-RT-01)",
            "register body 必带非 nil tenant_id",
        ));
    }
    if req.user_id.is_nil() {
        return Err(RestError::validation(
            "user_id 不能为 nil (INV-RT-05)",
            "register body 必带非 nil user_id",
        ));
    }
    if req.device_id.is_nil() {
        return Err(RestError::validation(
            "device_id 不能为 nil",
            "(tenant, user, device) 三元组唯一, device_id 必带非 nil",
        ));
    }
    // 内存态占位 actor: 鉴权中间件实装后从 JWT claim 注入
    let actor = ActorContext::new(req.user_id, req.tenant_id).as_local_runtime();
    let runtime = state
        .service
        .register_runtime(
            RegisterRuntimeCommand {
                tenant_id: TenantId(req.tenant_id),
                user_id: UserId(req.user_id),
                device_id: DeviceId(req.device_id),
                version: req.version.clone(),
                capabilities: req.capabilities.clone(),
                metadata: req.metadata.clone(),
            },
            &actor,
        )
        .await?;
    // 索引镜像 (持久化 PostgreSQL 接入后由 SQL 查询替换)
    state
        .runtimes
        .lock()
        .expect("runtimes index poisoned")
        .insert(runtime.id.as_uuid(), runtime.clone());
    Ok(Json(RestResponse::ok(json!({
        "runtime_id": runtime.id.as_uuid(),
        "status": runtime.status.as_str(),
        "runtime": runtime,
    }))))
}

/// `POST /api/v1/runtime/{runtime_id}/heartbeat` — 心跳 (调 heartbeat 命令), 返回 accepted
pub async fn heartbeat(
    State(state): State<Arc<RuntimeApiState>>,
    Path(runtime_id): Path<Uuid>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if req.tenant_id.is_nil() {
        return Err(RestError::validation(
            "tenant_id 不能为 nil (INV-RT-01)",
            "heartbeat body 必带非 nil tenant_id",
        ));
    }
    let owner = {
        let index = state.runtimes.lock().expect("runtimes index poisoned");
        index
            .get(&runtime_id)
            .cloned()
            .ok_or_else(|| runtime_not_found(runtime_id))?
    };
    let actor = owner_actor(&owner);
    let updated = state
        .service
        .heartbeat(
            HeartbeatCommand {
                tenant_id: TenantId(req.tenant_id),
                runtime_id: LocalRuntimeId(runtime_id),
                load_average: req.load_average,
                memory_used_bytes: req.memory_used_bytes,
            },
            &actor,
        )
        .await?;
    // 索引镜像更新 (status / last_heartbeat)
    state
        .runtimes
        .lock()
        .expect("runtimes index poisoned")
        .insert(runtime_id, updated.clone());
    Ok(Json(RestResponse::ok(json!({
        "accepted": true,
        "runtime_id": runtime_id,
        "status": updated.status.as_str(),
        "last_heartbeat": updated.last_heartbeat,
    }))))
}

/// `POST /api/v1/runtime/{runtime_id}/mount` — 挂载 Worktree (调 mount 命令), 返回 mounted
pub async fn mount_worktree(
    State(state): State<Arc<RuntimeApiState>>,
    Path(runtime_id): Path<Uuid>,
    Json(req): Json<MountWorktreeRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if req.tenant_id.is_nil() {
        return Err(RestError::validation(
            "tenant_id 不能为 nil (INV-RT-01)",
            "mount body 必带非 nil tenant_id",
        ));
    }
    if req.worktree_id.is_nil() {
        return Err(RestError::validation(
            "worktree_id 不能为 nil (INV-RT-02)",
            "mount body 必带非 nil worktree_id",
        ));
    }
    let owner = {
        let index = state.runtimes.lock().expect("runtimes index poisoned");
        index
            .get(&runtime_id)
            .cloned()
            .ok_or_else(|| runtime_not_found(runtime_id))?
    };
    let actor = owner_actor(&owner);
    let mount = state
        .service
        .mount_worktree(
            MountWorktreeCommand {
                tenant_id: TenantId(req.tenant_id),
                runtime_id: LocalRuntimeId(runtime_id),
                worktree_id: WorktreeId(req.worktree_id),
                local_path: req.local_path.clone(),
            },
            &actor,
        )
        .await?;
    Ok(Json(RestResponse::ok(json!({
        "mounted": true,
        "runtime_id": runtime_id,
        "mount_id": mount.id.as_uuid(),
        "worktree_id": mount.worktree_id.as_uuid(),
        "mount": mount,
    }))))
}

/// `GET /api/v1/runtime?user_id=` — 列出运行时 (user_id 可选; 无则全量)
pub async fn list_runtimes(
    State(state): State<Arc<RuntimeApiState>>,
    Query(params): Query<ListRuntimesParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let index = state.runtimes.lock().expect("runtimes index poisoned");
    let mut runtimes: Vec<LocalRuntime> = index
        .values()
        .filter(|r| match params.user_id {
            Some(user_id) => r.user_id.0 == user_id,
            None => true,
        })
        .cloned()
        .collect();
    // HashMap 迭代序不定, 按 runtime id 排序保证输出稳定
    runtimes.sort_by_key(|r| r.id);
    let total = runtimes.len();
    Ok(Json(RestResponse::ok(json!({
        "total": total,
        "runtimes": runtimes,
    }))))
}

/// `GET /api/v1/runtime/{runtime_id}/heartbeats?limit=` — 最近心跳列表 (倒序, Append-only 流)
pub async fn list_heartbeats(
    State(state): State<Arc<RuntimeApiState>>,
    Path(runtime_id): Path<Uuid>,
    Query(params): Query<ListHeartbeatsParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let owner = {
        let index = state.runtimes.lock().expect("runtimes index poisoned");
        index
            .get(&runtime_id)
            .cloned()
            .ok_or_else(|| runtime_not_found(runtime_id))?
    };
    let actor = owner_actor(&owner);
    let heartbeats = state
        .service
        .get_heartbeats(
            GetHeartbeatsQuery {
                tenant_id: owner.tenant_id,
                runtime_id: LocalRuntimeId(runtime_id),
                limit: params.limit,
            },
            &actor,
        )
        .await?;
    let total = heartbeats.len();
    Ok(Json(RestResponse::ok(json!({
        "runtime_id": runtime_id,
        "total": total,
        "heartbeats": heartbeats,
    }))))
}

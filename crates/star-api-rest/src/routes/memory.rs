/*
@cypher schema=1 source_sha256=5fe5be0bb4133bd184c936abc072fed4f0af456d1940999ef28581778d7ed647
MERGE (self:File {path:"crates/star-api-rest/src/routes/memory.rs"})
MERGE (store:Type {id:"crates/star-memory/src/lib.rs::MemoryStore",kind:"struct"})
MERGE (record:Type {id:"crates/star-memory/src/lib.rs::MemoryRecord",kind:"struct"})
MERGE (mtrait:Symbol {id:"crates/star-memory/src/lib.rs::Memory",kind:"trait"})
MERGE (self)-[:USES]->(store)
MERGE (self)-[:USES]->(record)
MERGE (self)-[:USES]->(mtrait)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// memory.rs — Agent 记忆端点 (star-memory 内存态门面, per Phase G.6)
// 2026-11-15 新增
//
//   GET    /api/v1/memory                     — 列出全部记忆记录 (MemoryStore 状态)
//   POST   /api/v1/memory                     — 写入 MemoryRecord {agent_id, layer, content}, 返回 record id
//   GET    /api/v1/memory/agent/{agent_id}    — 列出该 agent 全部记录 (短 + 长层)
//   DELETE /api/v1/memory/record/{record_id}  — 删除记录 (short_term 物理删除; long_term 禁止)
//
// 内存态 State: store 为 Arc<Mutex<MemoryStore>> — 持久化 PostgreSQL 接入后替换
// (Phase II 落库: store → PgPool + repository; known_agents 索引 → SQL 查询)
// =====================================================================

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use star_memory::{
    LongTermMemory, Memory, MemoryError, MemoryLayer, MemoryRecord, MemoryStore, ShortTermMemory,
};

use crate::error::RestError;
use crate::response::RestResponse;

// ---------------------------------------------------------------------
// 共享状态 (内存态, 持久化 PostgreSQL 接入后替换)
// ---------------------------------------------------------------------

/// Memory 端点共享状态。
///
/// `store` 为 `Arc<Mutex<MemoryStore>>` 内存态 — 持久化 PostgreSQL 接入后替换
/// 为 PgPool + repository (per G.6 Phase II)。
pub struct MemoryApiState {
    /// 内存 MemoryStore (PostgreSQL 接入后替换)
    pub store: Arc<Mutex<MemoryStore>>,
    /// 已知 agent_id 索引: star-memory 无 list_all 公开 API, 列表端点依赖此索引
    /// (PostgreSQL 接入后由 SQL 查询替换)
    pub known_agents: Mutex<HashSet<Uuid>>,
}

impl MemoryApiState {
    /// 新建空状态 (short_term 容量 64/agent, long_term 64 MiB)
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(MemoryStore::new(64, 64 * 1024 * 1024))),
            known_agents: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for MemoryApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试/多实例隔离用: 每次调用返回全新状态
pub fn default_state() -> Arc<MemoryApiState> {
    Arc::new(MemoryApiState::new())
}

/// 生产 build_router 用共享状态 (OnceLock)
fn state_arc() -> Arc<MemoryApiState> {
    static STATE: OnceLock<Arc<MemoryApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(default_state))
}

/// 生产嵌套路由 (共享 OnceLock 状态)
pub fn router() -> Router {
    router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (集成测试隔离用, per 守门 #25 v25)
pub fn router_with_state(state: Arc<MemoryApiState>) -> Router {
    Router::new()
        .route("/", get(list_records).post(create_record))
        .route("/agent/{agent_id}", get(list_agent_records))
        .route("/record/{record_id}", delete(delete_record))
        .with_state(state)
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 当前 epoch ms
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 锁定 store 取短/长期层 Arc 后立即释放 guard。
///
/// std::sync::MutexGuard 不 Send, 不可跨 await 持有 (axum handler 需 Send future);
/// star-memory 两层内部自持 tokio::sync::RwLock, 克隆 Arc 后独立使用即安全。
fn store_layers(state: &Arc<MemoryApiState>) -> (Arc<ShortTermMemory>, Arc<LongTermMemory>) {
    let guard = state.store.lock().expect("memory store poisoned");
    (Arc::clone(&guard.short_term), Arc::clone(&guard.long_term))
}

/// MemoryRecord → JSON (附 layer 标注)
fn record_json(layer: &str, r: &MemoryRecord) -> Value {
    json!({
        "layer": layer,
        "record_id": r.record_id,
        "tenant_id": r.tenant_id,
        "agent_id": r.agent_id,
        "kind": r.kind,
        "content": r.content,
        "created_at_ms": r.created_at_ms,
        "weight": r.weight,
    })
}

// ---------------------------------------------------------------------
// handlers
// ---------------------------------------------------------------------

fn default_kind() -> String {
    "context".to_string()
}

fn default_weight() -> f32 {
    0.5
}

/// POST body: MemoryRecord 写入请求 (per 任务: {agent_id, layer, content})
#[derive(Debug, Deserialize)]
pub struct CreateMemoryRequest {
    /// 归属 agent ID (v4 UUID)
    pub agent_id: Uuid,
    /// 记忆层: "short_term" | "long_term"
    pub layer: String,
    /// 记忆内容 (任意 JSON)
    pub content: Value,
    /// 记忆类型 (可选, 默认 "context")
    #[serde(default = "default_kind")]
    pub kind: String,
    /// 权重 0.0-1.0 (可选, 默认 0.5)
    #[serde(default = "default_weight")]
    pub weight: f32,
}

/// `POST /api/v1/memory` — 写入 MemoryRecord, 返回 record id + 层
pub async fn create_record(
    State(state): State<Arc<MemoryApiState>>,
    Json(req): Json<CreateMemoryRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let layer = match req.layer.as_str() {
        "short_term" => MemoryLayer::ShortTerm,
        "long_term" => MemoryLayer::LongTerm,
        other => {
            return Err(RestError::validation(
                format!("invalid memory layer: {other}"),
                "layer must be 'short_term' or 'long_term'",
            ))
        }
    };
    let record = MemoryRecord {
        record_id: Uuid::new_v4(),
        // 内存态 tenant 占位: 鉴权中间件实装后从 ActorContext 注入 (per 守门 #13 a RLS 13 类)
        tenant_id: Uuid::nil(),
        agent_id: req.agent_id,
        kind: req.kind,
        content: req.content,
        created_at_ms: now_ms(),
        weight: req.weight,
    };
    let (short, long) = store_layers(&state);
    let id = match layer {
        MemoryLayer::ShortTerm => short.write(record).await?,
        MemoryLayer::LongTerm => long.write(record).await?,
    };
    // 列表端点索引 (PostgreSQL 接入后由 SQL 查询替换)
    state
        .known_agents
        .lock()
        .expect("known_agents poisoned")
        .insert(req.agent_id);
    Ok(Json(RestResponse::ok(json!({
        "record_id": id,
        "layer": req.layer,
        "agent_id": req.agent_id,
    }))))
}

/// `GET /api/v1/memory` — 列出全部记忆记录 (MemoryStore 状态)
pub async fn list_records(
    State(state): State<Arc<MemoryApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let (short, long) = store_layers(&state);
    let agents: Vec<Uuid> = state
        .known_agents
        .lock()
        .expect("known_agents poisoned")
        .iter()
        .copied()
        .collect();
    let mut records: Vec<Value> = Vec::new();
    for agent in agents {
        for rec in short.list_by_agent(agent, u32::MAX).await? {
            records.push(record_json("short_term", &rec));
        }
        for rec in long.list_by_agent(agent, u32::MAX).await? {
            records.push(record_json("long_term", &rec));
        }
    }
    let total = records.len();
    Ok(Json(RestResponse::ok(json!({
        "total": total,
        "records": records,
    }))))
}

/// `GET /api/v1/memory/agent/{agent_id}` — 列出该 agent 全部记录 (短 + 长层)
pub async fn list_agent_records(
    State(state): State<Arc<MemoryApiState>>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let (short, long) = store_layers(&state);
    let mut records: Vec<Value> = Vec::new();
    for rec in short.list_by_agent(agent_id, u32::MAX).await? {
        records.push(record_json("short_term", &rec));
    }
    for rec in long.list_by_agent(agent_id, u32::MAX).await? {
        records.push(record_json("long_term", &rec));
    }
    let total = records.len();
    Ok(Json(RestResponse::ok(json!({
        "agent_id": agent_id,
        "total": total,
        "records": records,
    }))))
}

/// `DELETE /api/v1/memory/record/{record_id}` — 删除记录
///
/// short_term (W 派生): 物理删除 OK。
/// long_term (T 派生): 物理删除禁止 (per 守门 #13 b append-only) → 403 POLICY_DENIED。
pub async fn delete_record(
    State(state): State<Arc<MemoryApiState>>,
    Path(record_id): Path<Uuid>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let (short, long) = store_layers(&state);
    match short.read(record_id).await {
        Ok(_) => {
            short.delete(record_id).await?;
            Ok(Json(RestResponse::ok(json!({
                "deleted": record_id,
                "layer": "short_term",
            }))))
        }
        Err(MemoryError::NotFound(_)) => match long.read(record_id).await {
            Ok(_) => Err(RestError {
                code: "POLICY_DENIED".to_string(),
                message: format!(
                    "long_term record {record_id} 物理删除禁止 (per 守门 #13 b T 派生 append-only)"
                ),
                source_module: "star-memory".to_string(),
                source_kind: "Policy".to_string(),
                retriable: false,
                hint: "长期记忆走归档/审计流程, 不做物理删除".to_string(),
            }),
            Err(MemoryError::NotFound(_)) => Err(MemoryError::NotFound(record_id).into()),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}

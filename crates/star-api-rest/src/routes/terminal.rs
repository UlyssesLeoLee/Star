/*
@cypher schema=1 source_sha256=f1645faf6788a078e688a804bf56edb4ca4c6878ecf1f2b53924a387d34e8550
MERGE (self:File {path:"crates/star-api-rest/src/routes/terminal.rs"})
MERGE (persist:Type {id:"crates/terminal-stack/src/persistence.rs::TerminalStackPersistence",kind:"struct"})
MERGE (line:Type {id:"crates/terminal-stack/src/scrollback_buffer.rs::ScrollbackLine",kind:"struct"})
MERGE (source:Type {id:"crates/terminal-stack/src/scrollback_buffer.rs::ScrollbackSource",kind:"enum"})
MERGE (tree:Type {id:"crates/terminal-stack/src/split_pane.rs::SplitTree",kind:"struct"})
MERGE (kind:Type {id:"crates/terminal-stack/src/split_pane.rs::SplitTreeKind",kind:"enum"})
MERGE (create:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::create_pane",kind:"function"})
MERGE (getpane:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::get_pane",kind:"function"})
MERGE (append:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::append_line",kind:"function"})
MERGE (scrollback:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::get_scrollback",kind:"function"})
MERGE (self)-[:USES_TYPE]->(persist)
MERGE (self)-[:USES_TYPE]->(line)
MERGE (self)-[:USES_TYPE]->(source)
MERGE (self)-[:USES_TYPE]->(tree)
MERGE (self)-[:USES_TYPE]->(kind)
MERGE (self)-[:DEFINES]->(create)
MERGE (self)-[:DEFINES]->(getpane)
MERGE (self)-[:DEFINES]->(append)
MERGE (self)-[:DEFINES]->(scrollback)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// terminal.rs — 终端栈 HTTP 端点 (terminal-stack 孤儿 crate 接线, 2026-11-16 新增)
//
//   POST /api/v1/terminal/pane                       — 创建 pane (TerminalStackPersistence::in_memory), 返回 pane_id + created_at
//   POST /api/v1/terminal/pane/{pane_id}/line        — 追加一行 scrollback (append_scrollback_line), 返回 seq
//   GET  /api/v1/terminal/pane/{pane_id}/scrollback  — 拉滚动缓冲 (?since_seq 可选 → load_scrollback_since_seq), 返回 lines [{seq,text}] + max_seq
//   GET  /api/v1/terminal/pane/{pane_id}             — pane 元数据 (load_pane + API 层 meta 索引)
//
// 不存在 pane → 404 RESOURCE_NOT_FOUND (per error.rs IntoResponse 映射)。
//
// State: Arc<Mutex<HashMap<Uuid, TerminalStackPersistence>>> — **每 pane 一个独立的
// in_memory SQLite 实例** (Connection::open_in_memory), 隔离各 pane scrollback/树数据,
// 与 "每 pane 独立会话" 语义一致。内存态服务重启即失; 生产落盘 (open + WAL)
// 或 PostgreSQL 接入后替换 (per routes/memory.rs 同款演进注释)。
//
// 注册: parent 统一注册 (routes/mod.rs `pub mod terminal;` + lib.rs
// `.nest("/terminal", routes::terminal::router())`) — 本文件不自行注册。
// 因此本文件避免 `crate::` 路径, 改用 `star_api_rest::` 库名路径, 保证在被
// parent 收入 lib 前后均可编译 (注册前由 tests/terminal_endpoints.rs 经
// `#[path]` 直接引入编译, per 任务约束 "不要碰 routes/mod.rs")。
// =====================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use terminal_stack::persistence::TerminalStackPersistence;
use terminal_stack::{ScrollbackLine, ScrollbackSource, SplitTree, SplitTreeKind};

use crate::error::RestError;
use crate::response::RestResponse;

// ---------------------------------------------------------------------
// 共享状态 (内存态; 落盘/PostgreSQL 接入后替换)
// ---------------------------------------------------------------------

/// Pane 元数据 (terminal-stack 持久化层不存 worktree_id/title/created_at, API 层自持索引)
#[derive(Debug, Clone)]
pub struct PaneMeta {
    /// 所属 worktree id (per CreatePaneRequest)
    pub worktree_id: Uuid,
    /// 前端 pane 标题
    pub title: String,
    /// 创建时间 (epoch ms)
    pub created_at_ms: i64,
}

/// Terminal 端点共享状态。
///
/// `panes` 为 `Arc<Mutex<HashMap<Uuid, TerminalStackPersistence>>>`:
/// **每 pane 一个独立 in_memory 持久化实例** (SQLite `Connection::open_in_memory`),
/// 而非全局单库 — 各 pane 的 scrollback 行与 SplitTree 互不干扰, 且 pane 生命周期
/// 即 SQLite 连接生命周期。服务重启即失 (内存态), 生产落盘接入后替换
/// (per 守门 #11 缺标比错标 + routes/memory.rs 同款模式)。
pub struct TerminalApiState {
    /// pane_id → 该 pane 专属 in_memory TerminalStackPersistence
    pub panes: Arc<Mutex<HashMap<Uuid, TerminalStackPersistence>>>,
    /// pane_id → 元数据索引 (worktree_id / title / created_at_ms; 落盘接入后由 pane 表替代)
    pub meta: Mutex<HashMap<Uuid, PaneMeta>>,
}

impl TerminalApiState {
    /// 新建空状态 (无 pane)
    pub fn new() -> Self {
        Self {
            panes: Arc::new(Mutex::new(HashMap::new())),
            meta: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for TerminalApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试/多实例隔离用: 每次调用返回全新状态
pub fn default_state() -> Arc<TerminalApiState> {
    Arc::new(TerminalApiState::new())
}

/// 生产 build_router 用共享状态 (OnceLock)
fn state_arc() -> Arc<TerminalApiState> {
    static STATE: OnceLock<Arc<TerminalApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(default_state))
}

/// 生产嵌套路由 (共享 OnceLock 状态) — parent 以 `.nest("/terminal", router())` 挂载
pub fn router() -> Router {
    router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (集成测试隔离用, per 守门 #25 v25)
pub fn router_with_state(state: Arc<TerminalApiState>) -> Router {
    Router::new()
        .route("/pane", post(create_pane))
        .route("/pane/{pane_id}", get(get_pane))
        .route("/pane/{pane_id}/line", post(append_line))
        .route("/pane/{pane_id}/scrollback", get(get_scrollback))
        .with_state(state)
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 当前 epoch ms
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 不存在 pane → 404 RESOURCE_NOT_FOUND (per error.rs 状态映射)
fn pane_not_found(pane_id: Uuid) -> RestError {
    RestError {
        code: "RESOURCE_NOT_FOUND".to_string(),
        message: format!("terminal pane not found: {pane_id}"),
        source_module: "terminal-stack".to_string(),
        source_kind: "NotFound".to_string(),
        retriable: false,
        hint: "Create the pane first via POST /api/v1/terminal/pane".to_string(),
    }
}

/// ScrollbackSource → wire 字符串 (per persistence.rs source_as_str 同款)
fn source_as_str(source: &ScrollbackSource) -> &'static str {
    match source {
        ScrollbackSource::Stdout => "stdout",
        ScrollbackSource::Stderr => "stderr",
        ScrollbackSource::AgentInjected => "agent_injected",
    }
}

/// wire 字符串 → ScrollbackSource (未知 → None, caller 转 400)
fn parse_source(s: &str) -> Option<ScrollbackSource> {
    match s {
        "stdout" => Some(ScrollbackSource::Stdout),
        "stderr" => Some(ScrollbackSource::Stderr),
        "agent_injected" => Some(ScrollbackSource::AgentInjected),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// request shapes
// ---------------------------------------------------------------------

/// POST /api/v1/terminal/pane body: 创建 pane 请求
#[derive(Debug, Deserialize)]
pub struct CreatePaneRequest {
    /// 归属 worktree id (v4 UUID)
    pub worktree_id: Uuid,
    /// 前端 pane 标题
    pub title: String,
}

/// POST /api/v1/terminal/pane/{pane_id}/line body: 追加 scrollback 行
#[derive(Debug, Deserialize)]
pub struct AppendLineRequest {
    /// 行文本 (必须非空, per scrollback_buffer 非空 invariant)
    pub line: String,
    /// 来源 (可选, 默认 "stdout"; 合法值: "stdout" | "stderr" | "agent_injected")
    pub source: Option<String>,
}

/// GET /api/v1/terminal/pane/{pane_id}/scrollback query
#[derive(Debug, Deserialize)]
pub struct ScrollbackQuery {
    /// 增量起点 (可选): 只返回 seq > since_seq 的行; 缺省 = 拉全量
    pub since_seq: Option<i64>,
}

// ---------------------------------------------------------------------
// handlers
// ---------------------------------------------------------------------

/// `POST /api/v1/terminal/pane` — 创建 pane (in_memory 持久化), 返回 pane_id + created_at
///
/// 创建即预持久化单 pane SplitTree (pane_count=1, depth=0, kind=single),
/// 使 load_pane 立即可读; worktree_id/title/created_at 存 API 层 meta 索引。
pub async fn create_pane(
    State(state): State<Arc<TerminalApiState>>,
    Json(req): Json<CreatePaneRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let pane_id = Uuid::new_v4();
    let created_at_ms = now_ms();
    // 每 pane 一个独立 in_memory SQLite 实例 (per 任务约束, 见 TerminalApiState 注释)
    let persistence = TerminalStackPersistence::in_memory()?;
    persistence.persist_pane(pane_id, &SplitTree::new_single(pane_id))?;
    state
        .panes
        .lock()
        .expect("terminal panes poisoned")
        .insert(pane_id, persistence);
    state.meta.lock().expect("terminal meta poisoned").insert(
        pane_id,
        PaneMeta {
            worktree_id: req.worktree_id,
            title: req.title.clone(),
            created_at_ms,
        },
    );
    Ok(Json(RestResponse::ok(json!({
        "pane_id": pane_id,
        "worktree_id": req.worktree_id,
        "title": req.title,
        "created_at_ms": created_at_ms,
    }))))
}

/// `GET /api/v1/terminal/pane/{pane_id}` — pane 元数据 (meta 索引 + load_pane 树统计)
///
/// 不存在 pane → 404 RESOURCE_NOT_FOUND。
pub async fn get_pane(
    State(state): State<Arc<TerminalApiState>>,
    Path(pane_id): Path<Uuid>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let meta = state
        .meta
        .lock()
        .expect("terminal meta poisoned")
        .get(&pane_id)
        .cloned()
        .ok_or_else(|| pane_not_found(pane_id))?;
    let guard = state.panes.lock().expect("terminal panes poisoned");
    let persistence = guard.get(&pane_id).ok_or_else(|| pane_not_found(pane_id))?;
    // 创建时已 persist_pane, None 仅在状态不一致时出现 → 兜底为 single tree
    let tree = persistence
        .load_pane(pane_id)?
        .unwrap_or_else(|| SplitTree::new_single(pane_id));
    let kind = match tree.kind() {
        SplitTreeKind::Single => "single",
        SplitTreeKind::Split => "split",
    };
    Ok(Json(RestResponse::ok(json!({
        "pane_id": pane_id,
        "worktree_id": meta.worktree_id,
        "title": meta.title,
        "created_at_ms": meta.created_at_ms,
        "pane_count": tree.pane_count(),
        "depth": tree.depth(),
        "kind": kind,
    }))))
}

/// `POST /api/v1/terminal/pane/{pane_id}/line` — 追加一行 scrollback, 返回 seq
///
/// - 空行 → 400 VALIDATION_FAILED (per scrollback_buffer 非空 invariant)
/// - 未知 source → 400 VALIDATION_FAILED
/// - 不存在 pane → 404 RESOURCE_NOT_FOUND
pub async fn append_line(
    State(state): State<Arc<TerminalApiState>>,
    Path(pane_id): Path<Uuid>,
    Json(req): Json<AppendLineRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let text = req.line;
    if text.is_empty() {
        return Err(RestError::validation(
            "terminal line must not be empty",
            "send at least 1 byte of scrollback text",
        ));
    }
    let source = match parse_source(req.source.as_deref().unwrap_or("stdout")) {
        Some(s) => s,
        None => {
            return Err(RestError::validation(
                format!(
                    "invalid terminal line source: {}",
                    req.source.as_deref().unwrap_or_default()
                ),
                "source must be 'stdout' | 'stderr' | 'agent_injected'",
            ))
        }
    };
    let line = ScrollbackLine {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        text: text.clone(),
        source,
        byte_len: text.len() as u32,
    };
    let guard = state.panes.lock().expect("terminal panes poisoned");
    let persistence = guard.get(&pane_id).ok_or_else(|| pane_not_found(pane_id))?;
    let seq = persistence.append_scrollback_line(pane_id, &line)?;
    Ok(Json(RestResponse::ok(json!({
        "pane_id": pane_id,
        "seq": seq,
    }))))
}

/// `GET /api/v1/terminal/pane/{pane_id}/scrollback` — 拉滚动缓冲 (支持 ?since_seq 增量)
///
/// - 无 since_seq → `load_scrollback` 全量
/// - 有 since_seq → `load_scrollback_since_seq` (seq > since_seq)
///
/// 返回 `lines: [{seq, text, source, timestamp_ms}]` + `max_seq`。
///
/// seq 推导: persistence 层 seq_no 由 `COALESCE(MAX(seq_no),0)+1` 分配, per pane
/// 连续无空洞 (1..max_seq); load 按 seq_no ASC 返回。因此以
/// `capacity = max_seq - since_seq` (精确行数) 加载, drop-oldest 不触发,
/// 第 i 行 seq = base_seq + i + 1 (base_seq = 0 全量 / since_seq 增量) 成立。
pub async fn get_scrollback(
    State(state): State<Arc<TerminalApiState>>,
    Path(pane_id): Path<Uuid>,
    Query(query): Query<ScrollbackQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let since_seq = query.since_seq.unwrap_or(0);
    let guard = state.panes.lock().expect("terminal panes poisoned");
    let persistence = guard.get(&pane_id).ok_or_else(|| pane_not_found(pane_id))?;
    let max_seq = persistence.max_seq(pane_id)?;
    // 精确容量 = 实际行数, 避免 drop-oldest 截断破坏 seq 推导 (勿传 usize::MAX:
    // ScrollbackBuffer::new 会按容量预分配, 超大会 OOM)
    let (base_seq, capacity) = if query.since_seq.is_some() {
        (since_seq, (max_seq - since_seq).max(0) as usize)
    } else {
        (0, max_seq.max(0) as usize)
    };
    let buffer = if query.since_seq.is_some() {
        persistence.load_scrollback_since_seq(pane_id, since_seq, capacity)?
    } else {
        persistence.load_scrollback(pane_id, capacity)?
    };
    let lines: Vec<Value> = buffer
        .lines()
        .iter()
        .enumerate()
        .map(|(i, l)| {
            json!({
                "seq": base_seq + (i as i64) + 1,
                "text": l.text,
                "source": source_as_str(&l.source),
                "timestamp_ms": l.timestamp.timestamp_millis(),
            })
        })
        .collect();
    Ok(Json(RestResponse::ok(json!({
        "pane_id": pane_id,
        "since_seq": since_seq,
        "max_seq": max_seq,
        "lines": lines,
    }))))
}

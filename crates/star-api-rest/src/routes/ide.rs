/*
@cypher schema=1 source_sha256=47ac08357fca767cf08631814dfdcb9045fcb3aa49fc503c769223a8e178913d
MERGE (self:File {path:"crates/star-api-rest/src/routes/ide.rs"})
MERGE (docstore:Type {id:"crates/ide-kernel/src/doc_store.rs::DocStore",kind:"struct"})
MERGE (docview:Type {id:"crates/ide-kernel/src/doc_store.rs::DocView",kind:"struct"})
MERGE (kernel:Type {id:"crates/ide-kernel/src/lib.rs::KERNEL_VERSION",kind:"const"})
MERGE (open:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::open_doc",kind:"function"})
MERGE (active:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::get_active",kind:"function"})
MERGE (insert:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::insert",kind:"function"})
MERGE (sync:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::sync_text",kind:"function"})
MERGE (outline:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::outline",kind:"function"})
MERGE (scmlog:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::scm_log",kind:"function"})
MERGE (scmstatus:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::scm_status",kind:"function"})
MERGE (self)-[:USES_TYPE]->(docstore)
MERGE (self)-[:USES_TYPE]->(docview)
MERGE (self)-[:USES_TYPE]->(kernel)
MERGE (self)-[:DEFINES]->(open)
MERGE (self)-[:DEFINES]->(active)
MERGE (self)-[:DEFINES]->(insert)
MERGE (self)-[:DEFINES]->(sync)
MERGE (self)-[:DEFINES]->(outline)
MERGE (self)-[:DEFINES]->(scmlog)
MERGE (self)-[:DEFINES]->(scmstatus)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// ide.rs — IDE1.0 编辑内核 HTTP 端点 (ide-kernel 移植 crate 接线, 2026-11-17 新增)
//
// 内核来源: E:\IDE1.0 crates/ide-kernel (+editcore 基础模块), 权利人授权 MIT 移植
//   (docs/architecture/2026-11-14-ops-console/REUSE-MAP-COLLAB-SANDBOX-001.md §4 v0.2)。
// 端点语义镜像 IDE1.0 ide-shell-web 的 /api/doc/* (会话级编辑: 服务端管
// 光标/undo/选区/多光标), 外加 /outline (语义节点树) 与 /scm/* (Patch 提交记录,
// 只读 git)。响应形状转成 Star 统一 RestResponse 包装。
//
// 状态: Arc<Mutex<DocStore>> — 单进程内核会话 (内存态, 服务重启即失;
// 多租户隔离与落盘留后续阶段)。
//
// 安全边界:
// - 磁盘读 (doc/open 无 text 时 / outline 无 text 时) 仅在 `IDE_PROJECT_ROOT`
//   环境变量显式设置时启用, 且拒绝路径逃逸 (.. / 绝对路径 / 符号链接不作解析,
//   仅前缀检查); 未设置时一律要求请求带 text。
// - /scm/* 只跑只读 git (log / status), 不接受 stage/commit 写操作 (写路径走
//   Star 的 domain-scm / star-git-core)。
//
// 注册: parent 统一注册 (routes/mod.rs `pub mod ide;` + lib.rs
// `.nest("/ide", routes::ide::router())`) — 本文件不自行注册。
// 因此本文件避免 `crate::` 路径, 改用 `star_api_rest::` 库名路径, 保证在被
// parent 收入 lib 前后均可编译 (注册前由 tests/ide_endpoints.rs 经
// `#[path]` 直接引入编译, per routes/terminal.rs 同款约定)。
// =====================================================================

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use ide_kernel::doc_store::DocStore;
use ide_kernel::{DocView, OutlineEntry};

use crate::error::RestError;
use crate::response::RestResponse;

/// 打开文档时从磁盘读文件的上限 (与 IDE1.0 的 MAX_BODY_BYTES 一个量级)。
const MAX_FILE_READ_BYTES: u64 = 4 * 1024 * 1024;
/// scm/log 默认条数上限。
const SCM_LOG_DEFAULT: usize = 20;
/// scm/log 允许的最大条数 (防超大响应)。
const SCM_LOG_MAX: usize = 100;

// ---------------------------------------------------------------------
// 共享状态 (内存态; 落盘/多租户隔离后替换)
// ---------------------------------------------------------------------

/// IDE 内核端点共享状态。
///
/// `docs` 为 `Mutex<DocStore>`: 进程级单会话编辑内核 (多标签、光标、undo、
/// 多光标全部在服务端)。服务重启即失; 生产落盘 / 按租户分片留后续阶段
/// (per 守门 #11 缺标比错标 + routes/terminal.rs 同款演进注释)。
pub struct IdeApiState {
    /// 内核文档会话 (DocStore 不是 Sync 安全类型, 用 Mutex 包住)。
    pub docs: Mutex<DocStore>,
}

impl IdeApiState {
    /// 新建空状态 (无打开文档)。
    pub fn new() -> Self {
        Self {
            docs: Mutex::new(DocStore::new()),
        }
    }
}

impl Default for IdeApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试/多实例隔离用: 每次调用返回全新状态。
pub fn default_state() -> Arc<IdeApiState> {
    Arc::new(IdeApiState::new())
}

/// 生产 build_router 用共享状态 (OnceLock)。
fn state_arc() -> Arc<IdeApiState> {
    static STATE: OnceLock<Arc<IdeApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(default_state))
}

/// 生产嵌套路由 (共享 OnceLock 状态) — parent 以 `.nest("/ide", router())` 挂载。
pub fn router() -> Router {
    router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (集成测试隔离用, per 守门 #25 v25)。
pub fn router_with_state(state: Arc<IdeApiState>) -> Router {
    Router::new()
        .route("/version", get(version))
        // ---- 文档会话 (镜像 IDE1.0 /api/doc/*) ----
        .route("/doc/open", post(open_doc))
        .route("/doc/active", get(get_active))
        .route("/doc/tabs", get(get_tabs))
        .route("/doc/activate", post(activate_doc))
        .route("/doc/close", post(close_doc))
        .route("/doc/new_tab", post(new_tab))
        // ---- 编辑原语 ----
        .route("/doc/insert", post(insert))
        .route("/doc/type", post(type_char))
        .route("/doc/newline", post(newline))
        .route("/doc/backspace", post(backspace))
        .route("/doc/delete", post(delete_forward))
        .route("/doc/delete_range", post(delete_range))
        .route("/doc/caret", post(set_caret))
        .route("/doc/select_all", post(select_all))
        .route("/doc/clear_selection", post(clear_selection))
        .route("/doc/undo", post(undo))
        .route("/doc/redo", post(redo))
        // ---- 查找替换 ----
        .route("/doc/find", post(find))
        .route("/doc/find_all", post(find_all))
        .route("/doc/replace_all", post(replace_all))
        // ---- 全文本同步 (前端 textarea) + 格式 ----
        .route("/doc/sync", post(sync_text))
        .route("/doc/mark_saved", post(mark_saved))
        .route("/doc/export", get(export_doc))
        // ---- 语义节点树 ----
        .route("/outline", post(outline))
        // ---- Patch 提交记录 (只读 git) ----
        .route("/scm/log", get(scm_log))
        .route("/scm/status", get(scm_status))
        .with_state(state)
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 无活动文档 → 404 RESOURCE_NOT_FOUND (per error.rs 状态映射)。
fn no_active_doc() -> RestError {
    RestError {
        code: "RESOURCE_NOT_FOUND".to_string(),
        message: "no active document in ide-kernel session".to_string(),
        source_module: "ide-kernel".to_string(),
        source_kind: "NotFound".to_string(),
        retriable: false,
        hint: "Open a document first via POST /api/v1/ide/doc/open".to_string(),
    }
}

/// `IDE_PROJECT_ROOT` 环境变量 (显式设置才允许磁盘读)。
fn project_root() -> Option<PathBuf> {
    std::env::var("IDE_PROJECT_ROOT").ok().map(PathBuf::from)
}

/// 在 project_root 内安全拼接相对路径; 拒绝绝对路径与 `..` 逃逸。
///
/// 不解析符号链接 (软链可能指向 root 外, 调用方应按需再校验; 当前只做
/// 词法前缀检查, 与 IDE1.0 path_safety 的沙箱意图一致但更保守)。
/// Joins a path under `root`, rejecting absolute paths and `..` escapes.
///
/// Public so the integration test can assert the containment guarantee
/// directly — this is the boundary that keeps the read-only file bridge
/// from walking outside the configured roots.
pub fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return None;
    }
    for seg in rel_path.components() {
        if matches!(
            seg,
            std::path::Component::ParentDir | std::path::Component::Prefix(_)
        ) {
            return None;
        }
    }
    Some(root.join(rel_path))
}

/// 沙箱内读 UTF-8 文件 (上限 MAX_FILE_READ_BYTES); 任何失败返回 None。
fn read_file_bounded(root: &Path, rel: &str) -> Option<String> {
    let full = safe_join(root, rel)?;
    let meta = std::fs::metadata(&full).ok()?;
    if !meta.is_file() || meta.len() > MAX_FILE_READ_BYTES {
        return None;
    }
    let bytes = std::fs::read(&full).ok()?;
    String::from_utf8(bytes).ok()
}

/// 打开文档的内容解析: text 显式给 → 用 text; 否则 IDE_PROJECT_ROOT 磁盘读;
/// 都没有 → 空字符串 (打开空文档, 语义与 IDE1.0 /api/doc/open 一致)。
fn resolve_open_text(path: Option<&str>, text: Option<&str>) -> String {
    if let Some(t) = text {
        return t.to_string();
    }
    if let (Some(root), Some(p)) = (project_root(), path) {
        if let Some(content) = read_file_bounded(&root, p) {
            return content;
        }
    }
    String::new()
}

// ---------------------------------------------------------------------
// wire 响应形状
// ---------------------------------------------------------------------

/// DocView → wire JSON (`RestResponse::ok` 的 data 内)。
fn view_json(view: DocView) -> Value {
    json!({
        "id": view.id,
        "path": view.path,
        "text": view.text,
        "line_count": view.line_count,
        "caret_row": view.caret_row,
        "caret_col": view.caret_col,
        "line_ending": view.line_ending,
        "encoding": view.encoding,
        "dirty": view.dirty,
        "has_selection": view.has_selection,
        "selected_text": view.selected_text,
        "cursor_count": view.cursor_count,
        "can_undo": view.can_undo,
        "can_redo": view.can_redo,
    })
}

/// OutlineEntry → wire JSON。
fn outline_entry_json(entry: &OutlineEntry) -> Value {
    json!({
        "name": entry.name,
        "kind": entry.kind,
        "line": entry.line,
        "depth": entry.depth,
    })
}

// ---------------------------------------------------------------------
// request shapes
// ---------------------------------------------------------------------

/// POST /api/v1/ide/doc/open body。
#[derive(Debug, Deserialize)]
pub struct OpenDocRequest {
    /// 文档绑定路径 (可为空 = 未命名文档)。
    #[serde(default)]
    pub path: Option<String>,
    /// 显式内容; 缺省时若 IDE_PROJECT_ROOT 已设置则按 path 从磁盘读。
    #[serde(default)]
    pub text: Option<String>,
}

/// 携带 tab id 的请求 (activate / close)。
#[derive(Debug, Deserialize)]
pub struct IdRequest {
    /// 文档标签 id。
    pub id: u64,
}

/// 携带 (row, col) 位置的请求 (insert 位置 / caret 设置)。
#[derive(Debug, Deserialize)]
pub struct PosRequest {
    /// 0-based 行。
    #[serde(default)]
    pub row: i32,
    /// 0-based 列 (char 语义)。
    #[serde(default)]
    pub col: i32,
}

/// POST /api/v1/ide/doc/insert body。
#[derive(Debug, Deserialize)]
pub struct InsertRequest {
    /// 0-based 行。
    #[serde(default)]
    pub row: i32,
    /// 0-based 列 (char 语义)。
    #[serde(default)]
    pub col: i32,
    /// 插入文本。
    pub text: String,
}

/// POST /api/v1/ide/doc/type body。
#[derive(Debug, Deserialize)]
pub struct TypeCharRequest {
    /// 0-based 行。
    #[serde(default)]
    pub row: i32,
    /// 0-based 列 (char 语义)。
    #[serde(default)]
    pub col: i32,
    /// 单个字符 (取首字符)。
    pub char: String,
}

/// POST /api/v1/ide/doc/delete_range body。
#[derive(Debug, Deserialize)]
pub struct DeleteRangeRequest {
    /// 起点行。
    #[serde(default)]
    pub row: i32,
    /// 起点列。
    #[serde(default)]
    pub col: i32,
    /// 终点行 (含)。
    #[serde(default)]
    pub end_row: i32,
    /// 终点列 (含)。
    #[serde(default)]
    pub end_col: i32,
}

/// POST /api/v1/ide/doc/find body。
#[derive(Debug, Deserialize)]
pub struct FindRequest {
    /// 查找模式 (字面量)。
    pub pattern: String,
    /// 大小写敏感 (缺省 false)。
    #[serde(default)]
    pub case_sensitive: bool,
    /// 起始行 (缺省 0)。
    #[serde(default)]
    pub row: i32,
    /// 起始列 (缺省 0)。
    #[serde(default)]
    pub col: i32,
}

/// POST /api/v1/ide/doc/find_all body。
#[derive(Debug, Deserialize)]
pub struct FindAllRequest {
    /// 查找模式 (字面量)。
    pub pattern: String,
    /// 大小写敏感 (缺省 false)。
    #[serde(default)]
    pub case_sensitive: bool,
    /// 结果上限 (缺省 1000)。
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// POST /api/v1/ide/doc/replace_all body。
#[derive(Debug, Deserialize)]
pub struct ReplaceAllRequest {
    /// 查找模式 (字面量)。
    pub pattern: String,
    /// 替换文本。
    pub replacement: String,
    /// 大小写敏感 (缺省 false)。
    #[serde(default)]
    pub case_sensitive: bool,
    /// 替换上限 (缺省 1000)。
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// POST /api/v1/ide/doc/sync body (全文本同步)。
#[derive(Debug, Deserialize)]
pub struct SyncTextRequest {
    /// 文档全文 (替换活动文档内容, 重置 undo 栈)。
    pub text: String,
}

/// POST /api/v1/ide/outline body。
#[derive(Debug, Deserialize)]
pub struct OutlineRequest {
    /// 文件路径 (语言检测用)。
    pub path: String,
    /// 显式正文; 缺省时若 IDE_PROJECT_ROOT 已设置则按 path 从磁盘读。
    #[serde(default)]
    pub text: Option<String>,
}

/// GET /api/v1/ide/scm/log 与 /scm/status 共用 query。
#[derive(Debug, Deserialize)]
pub struct ScmQuery {
    /// 返回条数 (log 用, 1..=100, 缺省 20)。
    #[serde(default)]
    pub n: Option<usize>,
    /// 仓库根 (缺省 IDE_PROJECT_ROOT 或服务器 cwd)。
    #[serde(default)]
    pub root: Option<String>,
}

fn default_limit() -> usize {
    1000
}

// ---------------------------------------------------------------------
// handlers — 内核版本 / 文档会话
// ---------------------------------------------------------------------

/// `GET /api/v1/ide/version` — 内核版本 (前端连通性探测 / 协议兼容确认)。
pub async fn version() -> Json<RestResponse<Value>> {
    Json(RestResponse::ok(json!({
        "kernel_version": ide_kernel::KERNEL_VERSION,
        "service": "ide-kernel",
    })))
}

/// `POST /api/v1/ide/doc/open` — 打开文本为新标签, 返回 DocView。
pub async fn open_doc(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<OpenDocRequest>,
) -> Json<RestResponse<Value>> {
    let text = resolve_open_text(req.path.as_deref(), req.text.as_deref());
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.open(req.path.as_deref(), &text);
    Json(RestResponse::ok(json!({ "view": view_json(view) })))
}

/// `GET /api/v1/ide/doc/active` — 当前活动文档视图 (无文档时 view: null)。
pub async fn get_active(State(state): State<Arc<IdeApiState>>) -> Json<RestResponse<Value>> {
    let docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.active().map(view_json);
    Json(RestResponse::ok(json!({ "view": view })))
}

/// `GET /api/v1/ide/doc/tabs` — 全部标签视图。
pub async fn get_tabs(State(state): State<Arc<IdeApiState>>) -> Json<RestResponse<Value>> {
    let docs = state.docs.lock().expect("ide docs poisoned");
    let tabs: Vec<Value> = docs.tabs().into_iter().map(view_json).collect();
    Json(RestResponse::ok(json!({ "tabs": tabs })))
}

/// `POST /api/v1/ide/doc/activate` — 切换活动标签。
pub async fn activate_doc(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<IdRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.activate(req.id).ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/close` — 关闭标签, 返回新的活动视图 (可为 null)。
pub async fn close_doc(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<IdRequest>,
) -> Json<RestResponse<Value>> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.close(req.id).map(view_json);
    Json(RestResponse::ok(json!({ "view": view })))
}

/// `POST /api/v1/ide/doc/new_tab` — 新建空白文档并激活。
pub async fn new_tab(State(state): State<Arc<IdeApiState>>) -> Json<RestResponse<Value>> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.new_tab().map(view_json);
    Json(RestResponse::ok(json!({ "view": view })))
}

// ---------------------------------------------------------------------
// handlers — 编辑原语 (无活动文档 → 404)
// ---------------------------------------------------------------------

/// `POST /api/v1/ide/doc/insert` — 在 (row, col) 插入文本。
pub async fn insert(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs
        .insert(req.row, req.col, &req.text)
        .ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/type` — 在 (row, col) 输入一个字符 (有选区则替换)。
pub async fn type_char(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<TypeCharRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let ch = req.char.chars().next().unwrap_or(' ');
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs
        .type_char(req.row, req.col, ch)
        .ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/newline` — 插入换行 (保留上一行缩进)。
pub async fn newline(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.insert_newline().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/backspace` — 退格。
pub async fn backspace(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.backspace().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/delete` — 向前删除。
pub async fn delete_forward(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.delete_forward().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/delete_range` — 删除范围。
pub async fn delete_range(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<DeleteRangeRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs
        .delete_range(req.row, req.col, req.end_row, req.end_col)
        .ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/caret` — 设置光标 (清选区)。
pub async fn set_caret(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<PosRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.set_caret(req.row, req.col).ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/select_all` — 全选。
pub async fn select_all(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.select_all().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/clear_selection` — 清空选区。
pub async fn clear_selection(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.clear_selection().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/undo` — 撤销一步 (一次按键 = 一个 undo 单元)。
pub async fn undo(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.undo().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/redo` — 重做一步。
pub async fn redo(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.redo().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

// ---------------------------------------------------------------------
// handlers — 查找替换
// ---------------------------------------------------------------------

/// `POST /api/v1/ide/doc/find` — 从 (row, col) 起找第一处匹配 (绕回文档开头)。
pub async fn find(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<FindRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let hit = docs
        .find(&req.pattern, req.case_sensitive, req.row, req.col)
        .ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({
        "hit": {
            "start_row": hit.start_row,
            "start_col": hit.start_col,
            "end_row": hit.end_row,
            "end_col": hit.end_col,
        }
    }))))
}

/// `POST /api/v1/ide/doc/find_all` — 全部匹配 (上限 limit)。
pub async fn find_all(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<FindAllRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let hits = docs
        .find_all(&req.pattern, req.case_sensitive, req.limit)
        .ok_or_else(no_active_doc)?;
    let hits: Vec<Value> = hits
        .into_iter()
        .map(|h| {
            json!({
                "start_row": h.start_row,
                "start_col": h.start_col,
                "end_row": h.end_row,
                "end_col": h.end_col,
            })
        })
        .collect();
    Ok(Json(RestResponse::ok(json!({ "hits": hits }))))
}

/// `POST /api/v1/ide/doc/replace_all` — 全部替换 (倒序执行, 防错位)。
pub async fn replace_all(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<ReplaceAllRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let stat = docs
        .replace_all(
            &req.pattern,
            &req.replacement,
            req.case_sensitive,
            req.limit,
        )
        .ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({
        "replaced": stat.replaced,
        "truncated": stat.truncated,
    }))))
}

// ---------------------------------------------------------------------
// handlers — 全文本同步 / 保存
// ---------------------------------------------------------------------

/// `POST /api/v1/ide/doc/sync` — 全文本同步 (替换活动文档, 重置 undo 栈)。
///
/// 前端 textarea 整段编辑走这里; 逐键编辑请走 /doc/type 等原语以保留 undo。
pub async fn sync_text(
    State(state): State<Arc<IdeApiState>>,
    Json(req): Json<SyncTextRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.sync_text(&req.text).ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `POST /api/v1/ide/doc/mark_saved` — 标记已保存 (清 dirty, 保留 undo)。
pub async fn mark_saved(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut docs = state.docs.lock().expect("ide docs poisoned");
    let view = docs.mark_saved().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(json!({ "view": view_json(view) }))))
}

/// `GET /api/v1/ide/doc/export` — 导出活动文档 (按声明行尾符, 无文档时 404)。
pub async fn export_doc(
    State(state): State<Arc<IdeApiState>>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let docs = state.docs.lock().expect("ide docs poisoned");
    let (path, text) = docs.export().ok_or_else(no_active_doc)?;
    Ok(Json(RestResponse::ok(
        json!({ "path": path, "text": text }),
    )))
}

// ---------------------------------------------------------------------
// handlers — 语义节点树 (outline)
// ---------------------------------------------------------------------

/// `POST /api/v1/ide/outline` — 单文件语义节点树。
///
/// 语言检测 + outline 解析走 ide-kernel 移植的 type_config/types/outline_langs
/// (Rust / Python / JS / TS / Go / ... 20+ 语言)。text 缺省时仅在
/// IDE_PROJECT_ROOT 已设置时读盘, 否则返回空 entries + error 说明 (诚实降级,
/// 前端据此回退"本地推导预览"标注)。
pub async fn outline(Json(req): Json<OutlineRequest>) -> Json<RestResponse<Value>> {
    let content: Option<String> = match req.text {
        Some(t) => Some(t),
        None => match project_root() {
            Some(root) => read_file_bounded(&root, &req.path),
            None => None,
        },
    };
    let (language, entries, error) = match content {
        Some(text) => {
            let registry = ide_kernel::type_config::TypeRegistry::default();
            match registry.detect(&req.path) {
                Some(t) => {
                    let entries = ide_kernel::type_config::TypeConfig::outline_dispatch(&t.name, &text);
                    (t.name.to_string(), entries, None)
                }
                None => (String::new(), Vec::new(), None),
            }
        }
        None => (
            String::new(),
            Vec::new(),
            Some("no text provided and IDE_PROJECT_ROOT not configured — pass `text` in request body".to_string()),
        ),
    };
    // outline 是无状态纯计算 (IDE1.0 ide-shell-web /api/outline 同款);
    // 不持有内核会话锁, 避免编辑路径与解析路径互相阻塞。
    let entries: Vec<Value> = entries.iter().map(outline_entry_json).collect();
    Json(RestResponse::ok(json!({
        "path": req.path,
        "language": language,
        "entries": entries,
        "error": error,
    })))
}

// ---------------------------------------------------------------------
// handlers — Patch 提交记录 (只读 git)
// ---------------------------------------------------------------------

/// git 仓库根解析: 显式 root → IDE_PROJECT_ROOT → 服务器 cwd (向上找 .git)。
fn resolve_repo_root(explicit: Option<&str>) -> Option<PathBuf> {
    let mut start = explicit
        .map(PathBuf::from)
        .or_else(project_root)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    loop {
        if start.join(".git").exists() {
            return Some(start);
        }
        if !start.pop() {
            return None;
        }
    }
}

/// 只读 git 命令 (log / status 专用, 白名单在调用方)。
fn run_git_readonly(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git not found: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// `GET /api/v1/ide/scm/log?n=&root=` — 最近 n 条提交 (Patch 提交记录)。
///
/// 只读 `git log`; 失败不报 500 而是返回 error 字段 (与 IDE1.0 scm.rs 同款诚实
/// 降级: 非 git 目录 / git 未安装时前端可显示原因而不是空白)。
pub async fn scm_log(Query(query): Query<ScmQuery>) -> Json<RestResponse<Value>> {
    let n = query.n.unwrap_or(SCM_LOG_DEFAULT).clamp(1, SCM_LOG_MAX);
    let Some(root) = resolve_repo_root(query.root.as_deref()) else {
        return Json(RestResponse::ok(json!({
            "commits": [],
            "error": "not a git repository (set IDE_PROJECT_ROOT or pass ?root=)",
        })));
    };
    let fmt = "%H%x00%h%x00%an%x00%ai%x00%s";
    let raw = match run_git_readonly(
        &root,
        &["log", &format!("-{n}"), &format!("--format={fmt}")],
    ) {
        Ok(r) => r,
        Err(e) => {
            return Json(RestResponse::ok(json!({ "commits": [], "error": e })));
        }
    };
    let mut commits: Vec<Value> = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split('\0').collect();
        if parts.len() >= 5 {
            commits.push(json!({
                "hash": parts[0],
                "short": parts[1],
                "author": parts[2],
                "time": parts[3],
                "message": parts[4],
            }));
        }
    }
    Json(RestResponse::ok(
        json!({ "commits": commits, "error": Value::Null }),
    ))
}

/// `GET /api/v1/ide/scm/status?root=` — 工作区状态 (只读 `git status --porcelain`)。
pub async fn scm_status(Query(query): Query<ScmQuery>) -> Json<RestResponse<Value>> {
    let Some(root) = resolve_repo_root(query.root.as_deref()) else {
        return Json(RestResponse::ok(json!({
            "branch": "",
            "files": [],
            "error": "not a git repository (set IDE_PROJECT_ROOT or pass ?root=)",
        })));
    };
    let raw = match run_git_readonly(&root, &["status", "--porcelain"]) {
        Ok(r) => r,
        Err(e) => {
            return Json(RestResponse::ok(
                json!({ "branch": "", "files": [], "error": e }),
            ));
        }
    };
    let mut files: Vec<Value> = Vec::new();
    for line in raw.lines() {
        if line.len() < 3 {
            continue;
        }
        let code = &line[0..2];
        let path = line[3..].trim().trim_matches('"');
        let (x, y) = (
            code.chars().next().unwrap_or(' '),
            code.chars().nth(1).unwrap_or(' '),
        );
        if x == '?' && y == '?' {
            files.push(json!({ "path": path, "status": "untracked", "staged": false }));
            continue;
        }
        let status = match (x, y) {
            ('A', _) => "added",
            ('M', _) | (_, 'M') => "modified",
            ('D', _) | (_, 'D') => "deleted",
            ('R', _) => "renamed",
            ('C', _) => "copied",
            _ => "unknown",
        };
        files.push(json!({ "path": path, "status": status, "staged": x != ' ' && x != '?' }));
    }
    let branch =
        run_git_readonly(&root, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    Json(RestResponse::ok(json!({
        "branch": branch.trim(),
        "files": files,
        "error": Value::Null,
    })))
}

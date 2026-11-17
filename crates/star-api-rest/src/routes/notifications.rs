/*
@cypher schema=1 source_sha256=5ad76351978074c5dd36978a82cf8f0c693fe1d5edbd06077e6e67f567701dc5
MERGE (self:File {path:"crates/star-api-rest/src/routes/notifications.rs"})
MERGE (state:Type {id:"crates/star-api-rest/src/routes/notifications.rs::NotificationsApiState",kind:"struct"})
MERGE (notif:Type {id:"crates/star-api-rest/src/routes/notifications.rs::Notification",kind:"struct"})
MERGE (comment:Type {id:"crates/star-api-rest/src/routes/notifications.rs::Comment",kind:"struct"})
MERGE (nkind:Type {id:"crates/star-api-rest/src/routes/notifications.rs::NotificationKind",kind:"enum"})
MERGE (ttype:Type {id:"crates/star-api-rest/src/routes/notifications.rs::TargetType",kind:"enum"})
MERGE (listNotifs:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_notifications",kind:"function"})
MERGE (unread:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::unread_count",kind:"function"})
MERGE (createNotif:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_notification",kind:"function"})
MERGE (patchRead:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::patch_notification_read",kind:"function"})
MERGE (markRead:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::mark_notification_read",kind:"function"})
MERGE (listComments:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_comments",kind:"function"})
MERGE (createComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_comment",kind:"function"})
MERGE (deleteComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::delete_comment",kind:"function"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(notif)
MERGE (self)-[:DEFINES]->(comment)
MERGE (self)-[:DEFINES]->(nkind)
MERGE (self)-[:DEFINES]->(ttype)
MERGE (self)-[:DEFINES]->(listNotifs)
MERGE (self)-[:DEFINES]->(unread)
MERGE (self)-[:DEFINES]->(createNotif)
MERGE (self)-[:DEFINES]->(patchRead)
MERGE (self)-[:DEFINES]->(markRead)
MERGE (self)-[:DEFINES]->(listComments)
MERGE (self)-[:DEFINES]->(createComment)
MERGE (self)-[:DEFINES]->(deleteComment)
MERGE (state)-[:HAS]->(notif)
MERGE (state)-[:HAS]->(comment)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// notifications.rs — 通知 + 评论 owner API 端点 (2026-11-17 新增, 供前端 inbox 页接真实后端)
//
//   GET    /api/v1/notifications          — 列表 (?tenant_id/&kind/&unread_only/&limit/&offset, 含 total)
//   GET    /api/v1/notifications/unread-count — 未读计数 (?tenant_id 可选) → { count }
//   POST   /api/v1/notifications          — 创建 (kind/subject/body/tenant_id 必填, target_type/target_id 成对)
//   PATCH  /api/v1/notifications/{id}     — 更新 read 状态 (body: { read: bool })
//   POST   /api/v1/notifications/{id}/read — 标记已读 (幂等)
//   GET    /api/v1/comments               — 评论列表 (?target_type/&target_id 过滤, 时间正序)
//   POST   /api/v1/comments               — 新增评论 (作者取 X-User-Id 请求头)
//   DELETE /api/v1/comments/{id}          — 删除评论 (只能删自己的, 否则 403 POLICY_DENIED)
//
// 域模型逐字段对齐任务 spec (notification / comment), 自建内存态存储
// (Arc<Mutex<HashMap>>) — 不跨仓依赖 IM1.0 crates/im-core (不在本 workspace,
// path 依赖会带来 workspace 外编译成本, per 任务约束), 也不复用 domain-comment
// (CommentError From 映射已在 error.rs, 但本文件自包含以免父代理注册顺序耦合)。
//
// 已知简化 (诚实标注, per 守门 #5):
// - 内存态: 服务重启即失 — 持久化接入点: notifications 表 + comments 表
//   (PostgreSQL, kind/target_type 存 lowercase 字符串), 接入后替换本文件 store
// - 通知无 per-user 隔离: 列表/未读计数按 tenant_id 过滤 (任务 spec 的 query
//   面), 个人收件人维度 (recipient_user_id) 留持久化时按需扩展
// - DELETE /comments 的"自己的"判定: author_id == X-User-Id 请求头
//   (中间件 stub 实装前由请求头派生, 同 routes/im.rs; 实装后换 ActorContext)
// - 评论创建不联动生成 comment-kind 通知 (任务 spec 未要求; 若 inbox 页需要
//   评论→通知联动, 在 create_comment 成功路径追加 insert 即可)
//
// 鉴权: 评论作者/删除权限取 `X-User-Id` 请求头 (同 routes/runtime.rs
// 「ActorContext 由请求体派生」的占位模式); 中间件实装后从 ActorContext/JWT
// claim 注入替换。
//
// 注册: parent 统一注册 (routes/mod.rs `pub mod notifications;` + lib.rs
// `.nest("/notifications", routes::notifications::router())` +
// `.nest("/comments", routes::notifications::comments_router())`) — 本文件
// 不自行注册。本文件以 `crate::` 路径引用 error/response 模块 (同
// routes/im.rs); 集成测试 tests/notifications_endpoints.rs 经 lib crate
// (`star_api_rest::routes::notifications`) 引用本文件公开的
// default_state/router_with_state/comments_router_with_state 构造隔离
// router (per 守门 #25 v25, 不经 OnceLock 生产共享 state)。
// =====================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::RestError;
use crate::response::RestResponse;

/// 通知列表默认单页条数 (任务 spec 未定, 取 20)
const DEFAULT_LIMIT: i64 = 20;
/// 通知列表单页上限 (per routes/im.rs MAX_PAGE 同量级约定)
const MAX_LIMIT: i64 = 200;

// ---------------------------------------------------------------------
// 领域类型 (任务 spec 逐字段对齐)
// ---------------------------------------------------------------------

/// 通知类型 (任务 spec kind 枚举, serde snake_case 对齐)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    /// 评审请求 (review_request)
    ReviewRequest,
    /// 评论 (comment)
    Comment,
    /// 提及 (@mention)
    Mention,
    /// 系统通知 (system)
    System,
}

/// 通知目标类型 (任务 spec target_type 枚举, serde snake_case 对齐)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    /// 工作项 (work_item)
    WorkItem,
    /// 合并请求 (merge_request)
    MergeRequest,
    /// Agent (agent)
    Agent,
}

/// 通知 (任务 spec notification 逐字段: id/tenant_id/kind/subject/body/
/// subject_url/actor_id/actor_name/target_type/target_id/read/created_at)
#[derive(Debug, Clone)]
pub struct Notification {
    /// 通知 id (UUID v4)
    pub id: String,
    /// 所属租户 id
    pub tenant_id: String,
    /// 通知类型 (review_request | comment | mention | system)
    pub kind: NotificationKind,
    /// 标题
    pub subject: String,
    /// 正文
    pub body: String,
    /// 跳转目标 URL (可选)
    pub subject_url: Option<String>,
    /// 触发者用户 id (可选)
    pub actor_id: Option<String>,
    /// 触发者显示名 (可选)
    pub actor_name: Option<String>,
    /// 目标类型 (work_item | merge_request | agent, 与 target_id 成对)
    pub target_type: Option<String>,
    /// 目标 id (与 target_type 成对)
    pub target_id: Option<String>,
    /// 是否已读
    pub read: bool,
    /// 创建时间 RFC 3339
    pub created_at: String,
}

/// 评论 (任务 spec comment 逐字段: id/tenant_id/target_type/target_id/
/// author_id/author_name/body/created_at)
#[derive(Debug, Clone)]
pub struct Comment {
    /// 评论 id (UUID v4)
    pub id: String,
    /// 所属租户 id
    pub tenant_id: String,
    /// 目标类型 (自由字符串; 通知的 TargetType 枚举仅约束 notification)
    pub target_type: String,
    /// 目标 id
    pub target_id: String,
    /// 作者用户 id (创建时取 X-User-Id)
    pub author_id: String,
    /// 作者显示名 (缺省回落为 author_id)
    pub author_name: String,
    /// 评论正文
    pub body: String,
    /// 创建时间 RFC 3339
    pub created_at: String,
}

// ---------------------------------------------------------------------
// 共享状态 (内存态; 持久化 PostgreSQL 接入后替换)
// ---------------------------------------------------------------------

/// 通知 + 评论端点共享状态。
///
/// 内存态 `Arc<Mutex<HashMap>>` — 服务重启即失 (per 任务约束 + routes/im.rs
/// 同款演进注释)。持久化接入点:
/// - `notifications` → notifications 表
/// - `comments` → comments 表
pub struct NotificationsApiState {
    /// notification_id → Notification
    pub notifications: Arc<Mutex<HashMap<String, Notification>>>,
    /// comment_id → Comment
    pub comments: Arc<Mutex<HashMap<String, Comment>>>,
}

impl NotificationsApiState {
    /// 新建空状态 (无通知/无评论)
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(HashMap::new())),
            comments: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for NotificationsApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试/多实例隔离用: 每次调用返回全新状态
pub fn default_state() -> Arc<NotificationsApiState> {
    Arc::new(NotificationsApiState::new())
}

/// 生产嵌套路由用共享状态 (OnceLock, per routes/im.rs 模式)
fn state_arc() -> Arc<NotificationsApiState> {
    static STATE: OnceLock<Arc<NotificationsApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(default_state))
}

/// 生产嵌套路由 (通知) — parent 以 `.nest("/notifications", router())` 挂载
pub fn router() -> Router {
    router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (通知; 集成测试隔离用, per 守门 #25 v25)
pub fn router_with_state(state: Arc<NotificationsApiState>) -> Router {
    Router::new()
        .route("/", get(list_notifications))
        .route("/unread-count", get(unread_count))
        .route("/", post(create_notification))
        .route("/{id}", patch(patch_notification_read))
        .route("/{id}/read", post(mark_notification_read))
        .with_state(state)
}

/// 生产嵌套路由 (评论) — parent 以 `.nest("/comments", comments_router())` 挂载
pub fn comments_router() -> Router {
    comments_router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (评论; 集成测试隔离用)
pub fn comments_router_with_state(state: Arc<NotificationsApiState>) -> Router {
    Router::new()
        .route("/", get(list_comments))
        .route("/", post(create_comment))
        .route("/{id}", delete(delete_comment))
        .with_state(state)
}

// ---------------------------------------------------------------------
// 错误 helpers (本文件内构造 6-field RestError, 不动 error.rs)
// ---------------------------------------------------------------------

/// 400 VALIDATION_FAILED
fn notif_validation(message: impl Into<String>, hint: impl Into<String>) -> RestError {
    RestError {
        code: "VALIDATION_FAILED".to_string(),
        message: message.into(),
        source_module: "star-api-rest".to_string(),
        source_kind: "Validation".to_string(),
        retriable: false,
        hint: hint.into(),
    }
}

/// 404 RESOURCE_NOT_FOUND (通知不存在)
fn notif_not_found(notification_id: &str) -> RestError {
    RestError {
        code: "RESOURCE_NOT_FOUND".to_string(),
        message: format!("notification not found: {notification_id}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "NotFound".to_string(),
        retriable: false,
        hint: "Create the notification first via POST /api/v1/notifications".to_string(),
    }
}

/// 404 RESOURCE_NOT_FOUND (评论不存在)
fn comment_not_found(comment_id: &str) -> RestError {
    RestError {
        code: "RESOURCE_NOT_FOUND".to_string(),
        message: format!("comment not found: {comment_id}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "NotFound".to_string(),
        retriable: false,
        hint: "Create the comment first via POST /api/v1/comments".to_string(),
    }
}

/// 403 POLICY_DENIED (非评论作者删除)
fn not_comment_owner(user_id: &str, comment_id: &str) -> RestError {
    RestError {
        code: "POLICY_DENIED".to_string(),
        message: format!("user {user_id} cannot delete comment {comment_id}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "Policy".to_string(),
        retriable: false,
        hint: "Only the comment author can delete their own comment".to_string(),
    }
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 当前用户: `X-User-Id` 请求头。
///
/// 中间件 stub 实装前由请求头派生 (同 routes/im.rs current_user); 实装后从
/// ActorContext/JWT claim 注入替换。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
fn current_user(headers: &HeaderMap) -> Result<String, RestError> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            notif_validation(
                "missing or empty X-User-Id header",
                "Set the X-User-Id header; auth middleware will inject it from ActorContext later",
            )
        })
}

/// 必填字符串字段守卫: 缺省/空白 → 400
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
fn required_field(name: &str, value: Option<String>) -> Result<String, RestError> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(notif_validation(
            format!("{name} is required and must not be empty"),
            format!("Provide a non-empty {name}"),
        )),
    }
}

/// kind 字符串 → NotificationKind (非法 → 400)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub fn parse_notification_kind(s: &str) -> Result<NotificationKind, RestError> {
    match s {
        "review_request" => Ok(NotificationKind::ReviewRequest),
        "comment" => Ok(NotificationKind::Comment),
        "mention" => Ok(NotificationKind::Mention),
        "system" => Ok(NotificationKind::System),
        _ => Err(notif_validation(
            format!("invalid notification kind: {s}"),
            "Use one of: review_request, comment, mention, system",
        )),
    }
}

/// target_type 字符串 → TargetType (非法 → 400)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub fn parse_target_type(s: &str) -> Result<TargetType, RestError> {
    match s {
        "work_item" => Ok(TargetType::WorkItem),
        "merge_request" => Ok(TargetType::MergeRequest),
        "agent" => Ok(TargetType::Agent),
        _ => Err(notif_validation(
            format!("invalid target_type: {s}"),
            "Use one of: work_item, merge_request, agent",
        )),
    }
}

/// 通知 → wire JSON (snake_case, per Star 仓惯例)
fn notification_to_value(n: &Notification) -> Value {
    json!({
        "id": n.id,
        "tenant_id": n.tenant_id,
        "kind": n.kind,
        "subject": n.subject,
        "body": n.body,
        "subject_url": n.subject_url,
        "actor_id": n.actor_id,
        "actor_name": n.actor_name,
        "target_type": n.target_type,
        "target_id": n.target_id,
        "read": n.read,
        "created_at": n.created_at,
    })
}

/// 评论 → wire JSON (snake_case)
fn comment_to_value(c: &Comment) -> Value {
    json!({
        "id": c.id,
        "tenant_id": c.tenant_id,
        "target_type": c.target_type,
        "target_id": c.target_id,
        "author_id": c.author_id,
        "author_name": c.author_name,
        "body": c.body,
        "created_at": c.created_at,
    })
}

// ---------------------------------------------------------------------
// request shapes
// ---------------------------------------------------------------------

/// GET /api/v1/notifications query
#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    /// 租户过滤 (可选)
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// kind 过滤 (可选; 非法值 → 400)
    #[serde(default)]
    pub kind: Option<String>,
    /// 仅未读 (可选, 缺省 false)
    #[serde(default)]
    pub unread_only: Option<bool>,
    /// 单页条数 (可选, 缺省 20, clamp 1..=200)
    #[serde(default)]
    pub limit: Option<i64>,
    /// 偏移 (可选, 缺省 0, 负值按 0)
    #[serde(default)]
    pub offset: Option<i64>,
}

/// GET /api/v1/notifications/unread-count query
#[derive(Debug, Deserialize)]
pub struct UnreadCountQuery {
    /// 租户过滤 (可选)
    #[serde(default)]
    pub tenant_id: Option<String>,
}

/// POST /api/v1/notifications body
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    /// 租户 id — 必填
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// 通知类型 (review_request | comment | mention | system) — 必填
    #[serde(default)]
    pub kind: Option<String>,
    /// 标题 — 必填
    #[serde(default)]
    pub subject: Option<String>,
    /// 正文 — 必填
    #[serde(default)]
    pub body: Option<String>,
    /// 跳转目标 URL (可选)
    #[serde(default)]
    pub subject_url: Option<String>,
    /// 触发者用户 id (可选)
    #[serde(default)]
    pub actor_id: Option<String>,
    /// 触发者显示名 (可选)
    #[serde(default)]
    pub actor_name: Option<String>,
    /// 目标类型 (可选, 与 target_id 成对出现)
    #[serde(default)]
    pub target_type: Option<String>,
    /// 目标 id (可选, 与 target_type 成对出现)
    #[serde(default)]
    pub target_id: Option<String>,
}

/// PATCH /api/v1/notifications/{id} body
#[derive(Debug, Deserialize)]
pub struct UpdateNotificationReadRequest {
    /// 目标已读状态
    pub read: bool,
}

/// GET /api/v1/comments query
#[derive(Debug, Deserialize)]
pub struct ListCommentsQuery {
    /// 目标类型过滤 (可选, 自由字符串精确匹配)
    #[serde(default)]
    pub target_type: Option<String>,
    /// 目标 id 过滤 (可选, 精确匹配)
    #[serde(default)]
    pub target_id: Option<String>,
}

/// POST /api/v1/comments body
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    /// 租户 id — 必填
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// 目标类型 (自由字符串) — 必填
    #[serde(default)]
    pub target_type: Option<String>,
    /// 目标 id — 必填
    #[serde(default)]
    pub target_id: Option<String>,
    /// 评论正文 — 必填
    #[serde(default)]
    pub body: Option<String>,
    /// 作者显示名 (可选, 缺省回落为 X-User-Id)
    #[serde(default)]
    pub author_name: Option<String>,
}

// ---------------------------------------------------------------------
// handlers
// ---------------------------------------------------------------------

/// `GET /api/v1/notifications` — 通知列表
///
/// 过滤: tenant_id (精确) / kind (非法值 → 400) / unread_only (true 只回未读)。
/// 分页: limit (缺省 20, clamp 1..=200) + offset (缺省 0)。排序:
/// created_at DESC (新在前), 同刻按 id DESC 稳定。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_notifications(
    State(state): State<Arc<NotificationsApiState>>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let kind_filter = match &query.kind {
        Some(k) => Some(parse_notification_kind(k)?),
        None => None,
    };
    let unread_only = query.unread_only.unwrap_or(false);
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);

    let store = state.notifications.lock().expect("notifications poisoned");
    let mut matched: Vec<&Notification> = store
        .values()
        .filter(|n| {
            query.tenant_id.as_ref().is_none_or(|t| &n.tenant_id == t)
                && kind_filter.is_none_or(|k| n.kind == k)
                && (!unread_only || !n.read)
        })
        .collect();
    matched.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));

    let total = matched.len() as i64;
    let page: Vec<Value> = matched
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(notification_to_value)
        .collect();

    Ok(Json(RestResponse::ok(json!({
        "notifications": page,
        "total": total,
        "limit": limit,
        "offset": offset,
    }))))
}

/// `GET /api/v1/notifications/unread-count` — 未读计数
///
/// 返回 `{ count }`; ?tenant_id 可选过滤。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn unread_count(
    State(state): State<Arc<NotificationsApiState>>,
    Query(query): Query<UnreadCountQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let store = state.notifications.lock().expect("notifications poisoned");
    let count = store
        .values()
        .filter(|n| !n.read)
        .filter(|n| query.tenant_id.as_ref().is_none_or(|t| &n.tenant_id == t))
        .count();

    Ok(Json(RestResponse::ok(json!({ "count": count }))))
}

/// `POST /api/v1/notifications` — 创建通知
///
/// 校验: tenant_id/kind/subject/body 必填非空; kind ∈ 4 枚举; target_type ∈
/// 3 枚举且与 target_id 成对出现 (只给其一 → 400)。新建恒 read=false。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_notification(
    State(state): State<Arc<NotificationsApiState>>,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let tenant_id = required_field("tenant_id", req.tenant_id)?;
    let kind = match req.kind {
        Some(k) => parse_notification_kind(&k)?,
        None => {
            return Err(notif_validation(
                "kind is required",
                "Use one of: review_request, comment, mention, system",
            ))
        }
    };
    let subject = required_field("subject", req.subject)?;
    let body = required_field("body", req.body)?;
    // target_type 存在时先做枚举校验, 落库保留原字符串 (已校验为合法 lowercase)
    let target_type = match req.target_type {
        Some(t) => {
            parse_target_type(&t)?;
            Some(t)
        }
        None => None,
    };
    if req.target_id.is_some() != target_type.is_some() {
        return Err(notif_validation(
            "target_type and target_id must be provided together",
            "Provide both target_type and target_id, or neither",
        ));
    }

    let notif = Notification {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id,
        kind,
        subject,
        body,
        subject_url: req.subject_url.filter(|s| !s.is_empty()),
        actor_id: req.actor_id.filter(|s| !s.is_empty()),
        actor_name: req.actor_name.filter(|s| !s.is_empty()),
        target_type,
        target_id: req.target_id.filter(|s| !s.is_empty()),
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .notifications
        .lock()
        .expect("notifications poisoned")
        .insert(notif.id.clone(), notif.clone());

    Ok(Json(RestResponse::ok(json!({
        "notification": notification_to_value(&notif),
    }))))
}

/// `PATCH /api/v1/notifications/{id}` — 更新 read 状态 (body: { read: bool })
///
/// 不存在 → 404。响应回显更新后的通知。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn patch_notification_read(
    State(state): State<Arc<NotificationsApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateNotificationReadRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut store = state.notifications.lock().expect("notifications poisoned");
    let notif = store.get_mut(&id).ok_or_else(|| notif_not_found(&id))?;
    notif.read = req.read;
    let value = notification_to_value(notif);

    Ok(Json(RestResponse::ok(json!({ "notification": value }))))
}

/// `POST /api/v1/notifications/{id}/read` — 标记已读 (幂等)
///
/// 不存在 → 404; 已读再标 → 仍 200 + read:true 回显。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn mark_notification_read(
    State(state): State<Arc<NotificationsApiState>>,
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut store = state.notifications.lock().expect("notifications poisoned");
    let notif = store.get_mut(&id).ok_or_else(|| notif_not_found(&id))?;
    notif.read = true;
    let value = notification_to_value(notif);

    Ok(Json(RestResponse::ok(json!({ "notification": value }))))
}

/// `GET /api/v1/comments` — 评论列表
///
/// ?target_type/&target_id 可选精确过滤; 时间正序 (线程阅读顺序), 同刻按 id
/// 稳定排序; 响应含 total。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_comments(
    State(state): State<Arc<NotificationsApiState>>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let store = state.comments.lock().expect("comments poisoned");
    let mut matched: Vec<&Comment> = store
        .values()
        .filter(|c| {
            query
                .target_type
                .as_ref()
                .is_none_or(|t| &c.target_type == t)
                && query.target_id.as_ref().is_none_or(|t| &c.target_id == t)
        })
        .collect();
    matched.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));

    let total = matched.len();
    let comments: Vec<Value> = matched.into_iter().map(comment_to_value).collect();

    Ok(Json(RestResponse::ok(json!({
        "comments": comments,
        "total": total,
    }))))
}

/// `POST /api/v1/comments` — 新增评论
///
/// 作者取 X-User-Id 请求头 (缺 → 400); author_name 缺省回落为 author_id;
/// target_type 为自由字符串 (不套 notification 的 TargetType 枚举, 任务 spec
/// 未枚举评论目标)。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_comment(
    State(state): State<Arc<NotificationsApiState>>,
    headers: HeaderMap,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let author_id = current_user(&headers)?;
    let tenant_id = required_field("tenant_id", req.tenant_id)?;
    let target_type = required_field("target_type", req.target_type)?;
    let target_id = required_field("target_id", req.target_id)?;
    let body = required_field("body", req.body)?;
    let author_name = match req.author_name {
        Some(n) if !n.is_empty() => n,
        _ => author_id.clone(),
    };

    let comment = Comment {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id,
        target_type,
        target_id,
        author_id,
        author_name,
        body,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .comments
        .lock()
        .expect("comments poisoned")
        .insert(comment.id.clone(), comment.clone());

    Ok(Json(RestResponse::ok(json!({
        "comment": comment_to_value(&comment),
    }))))
}

/// `DELETE /api/v1/comments/{id}` — 删除评论 (只能删自己的)
///
/// 守卫: 不存在 → 404; X-User-Id ≠ author_id → 403 POLICY_DENIED; 成功
/// 返回 `{ deleted: true, comment_id }`。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn delete_comment(
    State(state): State<Arc<NotificationsApiState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    let mut store = state.comments.lock().expect("comments poisoned");
    let comment = store.get(&id).ok_or_else(|| comment_not_found(&id))?;
    if comment.author_id != user_id {
        return Err(not_comment_owner(&user_id, &id));
    }
    store.remove(&id);

    Ok(Json(RestResponse::ok(json!({
        "deleted": true,
        "comment_id": id,
    }))))
}

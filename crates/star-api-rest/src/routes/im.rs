/*
@cypher schema=1 source_sha256=fe11beb53646a8497028abc6109b88ae2f1617f6a6033bffb7ebb52f211cd7bb
MERGE (self:File {path:"crates/star-api-rest/src/routes/im.rs"})
MERGE (state:Type {id:"crates/star-api-rest/src/routes/im.rs::ImApiState",kind:"struct"})
MERGE (conv:Type {id:"crates/star-api-rest/src/routes/im.rs::Conversation",kind:"struct"})
MERGE (member:Type {id:"crates/star-api-rest/src/routes/im.rs::Member",kind:"struct"})
MERGE (msg:Type {id:"crates/star-api-rest/src/routes/im.rs::Message",kind:"struct"})
MERGE (kind:Type {id:"crates/star-api-rest/src/routes/im.rs::ConversationKind",kind:"enum"})
MERGE (role:Type {id:"crates/star-api-rest/src/routes/im.rs::MemberRole",kind:"enum"})
MERGE (mstate:Type {id:"crates/star-api-rest/src/routes/im.rs::MessageState",kind:"enum"})
MERGE (createConv:Symbol {id:"crates/star-api-rest/src/routes/im.rs::create_conversation",kind:"function"})
MERGE (listConvs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_conversations",kind:"function"})
MERGE (sendMsg:Symbol {id:"crates/star-api-rest/src/routes/im.rs::send_message",kind:"function"})
MERGE (listMsgs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_messages",kind:"function"})
MERGE (markRead:Symbol {id:"crates/star-api-rest/src/routes/im.rs::mark_read",kind:"function"})
MERGE (presence:Symbol {id:"crates/star-api-rest/src/routes/im.rs::presence",kind:"function"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(conv)
MERGE (self)-[:DEFINES]->(member)
MERGE (self)-[:DEFINES]->(msg)
MERGE (self)-[:DEFINES]->(kind)
MERGE (self)-[:DEFINES]->(role)
MERGE (self)-[:DEFINES]->(mstate)
MERGE (self)-[:DEFINES]->(createConv)
MERGE (self)-[:DEFINES]->(listConvs)
MERGE (self)-[:DEFINES]->(sendMsg)
MERGE (self)-[:DEFINES]->(listMsgs)
MERGE (self)-[:DEFINES]->(markRead)
MERGE (self)-[:DEFINES]->(presence)
MERGE (conv)-[:HAS_MEMBER]->(member)
MERGE (conv)-[:HAS_MEMBER]->(msg)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// im.rs — IM 即时通信端点 (用户间消息, 2026-11-16 新增)
//
//   POST /api/v1/im/conversations                        — 创建 DM/群聊, 返回 conversation_id + kind + members
//   GET  /api/v1/im/conversations                        — 列出当前用户的会话 + 未读数
//   POST /api/v1/im/conversations/{conversation_id}/messages — 发消息 (kind:'text'), 返回 sequence + state:'sent'
//   GET  /api/v1/im/conversations/{conversation_id}/messages — 按 sequence 分页拉取 (?after_sequence 可选)
//   POST /api/v1/im/conversations/{conversation_id}/read  — 标记已读 (body 含 sequence)
//   GET  /api/v1/im/presence                              — 在线成员列表 (无真实连接 → 空列表 + connected:false)
//
// IM1.0 语义对齐, 独立实现避免跨仓依赖 — 域模型逐字段对齐 D:/IM1.0 的
// crates/im-core (conversation/message 仓储接口) 语义, 不引入 IM1.0 crate
// (跨仓 path 依赖会带来 workspace 外编译成本/依赖环风险, per 任务约束):
// - ConversationKind {dm, group}     ← IM1.0 ConversationKind (serde lowercase)
// - MemberRole {owner, admin, member} ← IM1.0 MemberRole
// - ConversationMember.last_read_sequence ← IM1.0 ConversationMember.last_read_sequence (i64)
// - Message {sequence, kind, content, reply_to, state} ← IM1.0 Message (repository.rs)
// - MessageState {sent, delivered, read, recalled, deleted} ← IM1.0 MessageState;
//   本实现新消息恒为 sent (IM1.0 MessageService 落库 state=MessageState::Sent)
// - sequence 按会话单调自增 ← IM1.0 SequenceAllocator (行锁原子自增语义)
// - DM 幂等 (同对用户返回既有会话) ← IM1.0 ConversationService::create_dm + dm_pairs
// - mark_read: 负值拒绝 + 上界夹紧 + 单调守卫 ← IM1.0 ConversationService::mark_read
// - presence 占位 ← IM1.0 crates/im-presence 同为占位 (V1 Valkey 在线/typing/last_seen)
//
// 内存态存储 (Arc<Mutex<HashMap>>) — 持久化/WS 网关后续接入:
// - conversations/messages 双 map → PostgreSQL (conversations + conversation_members +
//   messages 表, 序列走 per-conversation 行锁) 接入后替换
// - 未读数计算 → messages 表 count(sequence > last_read_sequence AND sender <> me)
// - presence → im-gateway WS 网关 (Valkey 在线状态 + presence_update/typing 帧,
//   per IM1.0 im-protocol::ws_frames ServerFrame 变体) 接入后由网关实装
//
// 鉴权: 中间件 stub 实装前, 当前用户取 `X-User-Id` 请求头 (同 routes/runtime.rs
// 「ActorContext 由请求体派生」的占位模式); 中间件实装后从 ActorContext/JWT claim
// 注入替换。
//
// 注册: parent 统一注册 (routes/mod.rs `pub mod im;` + lib.rs
// `.nest("/im", routes::im::router())`) — 本文件不自行注册。
// 因此本文件避免 `crate::` 路径, 改用 `star_api_rest::` 库名路径, 保证在被
// parent 收入 lib 前后均可编译 (注册前由 tests/im_endpoints.rs 经
// `#[path]` 直接引入编译, per 任务约束 "不要碰 routes/mod.rs 与 lib.rs")。
// =====================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::RestError;
use crate::response::RestResponse;

/// 消息正文最大字节数 (IM1.0 SendMessageCommand.max_size_bytes 默认 65536)
const MAX_CONTENT_BYTES: usize = 65_536;
/// 群聊成员上限 (IM1.0 ConversationService::create_group max 500)
const MAX_GROUP_MEMBERS: usize = 500;
/// 消息分页单页上限 (IM1.0 MessageService::MAX_PAGE = 200)
const MAX_PAGE: i32 = 200;

// ---------------------------------------------------------------------
// 领域类型 (IM1.0 语义对齐, 独立实现)
// ---------------------------------------------------------------------

/// 会话类型 (IM1.0 `ConversationKind` 的 dm/group 子集, serde lowercase 对齐)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversationKind {
    /// 单聊 (IM1.0 `ConversationKind::Dm`)
    Dm,
    /// 群聊 (IM1.0 `ConversationKind::Group`)
    Group,
}

/// 成员角色 (IM1.0 `MemberRole` 全量对齐)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    /// 群主 (IM1.0 `MemberRole::Owner`, create_group 创建者)
    Owner,
    /// 管理员 (IM1.0 `MemberRole::Admin`; 最小实现暂不使用)
    Admin,
    /// 普通成员 (IM1.0 `MemberRole::Member`)
    Member,
}

/// 消息投递状态 (IM1.0 `MessageState` 全量对齐; 本实现新消息恒为 Sent)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageState {
    /// 已发送 (IM1.0 `MessageState::Sent`; 本实现落库态)
    Sent,
    /// 已投递 (IM1.0 `MessageState::Delivered`; 预留, WS 网关实装后使用)
    Delivered,
    /// 已读 (IM1.0 `MessageState::Read`; 预留)
    Read,
    /// 已撤回 (IM1.0 `MessageState::Recalled`; 预留)
    Recalled,
    /// 已删除 (IM1.0 `MessageState::Deleted`; 预留)
    Deleted,
}

/// 会话成员 (IM1.0 `ConversationMember`: user_id + role + last_read_sequence)
#[derive(Debug, Clone)]
pub struct Member {
    /// 成员用户 id (IM1.0 `ConversationMember.user_id`)
    pub user_id: String,
    /// 成员角色 (IM1.0 `ConversationMember.role`)
    pub role: MemberRole,
    /// 已读指针 (IM1.0 `last_read_sequence`, 语义: 该成员已读到的最大 sequence)
    pub last_read_sequence: i64,
}

/// 会话 (IM1.0 `Conversation` + `conversation_sequences` 合并:
/// id / kind / metadata(title) / created_at + 成员列表 + 已分配 sequence 水位)
#[derive(Debug, Clone)]
pub struct Conversation {
    /// 会话 id (IM1.0 `Conversation.id`)
    pub id: String,
    /// 会话类型 (IM1.0 `Conversation.kind`)
    pub kind: ConversationKind,
    /// 群聊标题 (IM1.0 `Conversation.metadata`; DM 恒 None)
    pub title: Option<String>,
    /// 成员列表 (IM1.0 `list_members`)
    pub members: Vec<Member>,
    /// 已分配 sequence 水位 (IM1.0 `SequenceAllocator` 行锁自增语义;
    /// 下一个待分配 = 该值 + 1, 即 IM1.0 `max_allocated_sequence` = 本字段)
    pub next_sequence: i64,
    /// 创建时间 RFC 3339 (IM1.0 `Conversation.created_at`)
    pub created_at: String,
}

/// 消息 (IM1.0 `Message` 逐字段对齐; id 为消息 UUID, sequence 为会话内单调序号)
#[derive(Debug, Clone)]
pub struct Message {
    /// 消息 id (IM1.0 `Message.id`)
    pub id: String,
    /// 所属会话 id (IM1.0 `Message.conversation_id`)
    pub conversation_id: String,
    /// 会话内单调序号 (IM1.0 `Message.sequence`, SequenceAllocator 分配)
    pub sequence: i64,
    /// 发送者用户 id (IM1.0 `Message.sender_id`)
    pub sender_id: String,
    /// 消息类型 (IM1.0 `Message.kind`, 最小实现仅 "text")
    pub kind: String,
    /// 消息正文 JSON (IM1.0 `Message.content`)
    pub content: Value,
    /// IM1.0 `Message.reply_to: Option<MessageId>` — 被回复消息的 id
    pub reply_to: Option<String>,
    /// 投递状态 (IM1.0 `Message.state`)
    pub state: MessageState,
    /// 创建时间 RFC 3339 (IM1.0 `Message.created_at`)
    pub created_at: String,
}

// ---------------------------------------------------------------------
// 共享状态 (内存态; 持久化 PostgreSQL 接入后替换)
// ---------------------------------------------------------------------

/// IM 端点共享状态。
///
/// 内存态 `Arc<Mutex<HashMap>>` — 服务重启即失 (per 任务约束 + routes/memory.rs
/// 同款演进注释)。持久化接入点:
/// - `conversations` → conversations + conversation_members 表
/// - `messages` → messages 表 (sequence 走 per-conversation 行锁, per IM1.0)
/// - `dm_index` → dm_pairs 表 (IM1.0 `link_dm_pair` / `find_dm` 幂等短路)
pub struct ImApiState {
    /// conversation_id → Conversation
    pub conversations: Arc<Mutex<HashMap<String, Conversation>>>,
    /// conversation_id → Vec`<Message>` (按 sequence 升序追加)
    pub messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    /// DM 幂等索引: "min(user)|max(user)" → conversation_id (IM1.0 dm_pairs 语义)
    pub dm_index: Arc<Mutex<HashMap<String, String>>>,
}

impl ImApiState {
    /// 新建空状态 (无会话/无消息)
    pub fn new() -> Self {
        Self {
            conversations: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            dm_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for ImApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试/多实例隔离用: 每次调用返回全新状态
pub fn default_state() -> Arc<ImApiState> {
    Arc::new(ImApiState::new())
}

/// 生产嵌套路由用共享状态 (OnceLock, per routes/terminal.rs 模式)
fn state_arc() -> Arc<ImApiState> {
    static STATE: OnceLock<Arc<ImApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(default_state))
}

/// 生产嵌套路由 — parent 以 `.nest("/im", router())` 挂载
pub fn router() -> Router {
    router_with_state(state_arc())
}

/// caller-supplied state 嵌套路由 (集成测试隔离用, per 守门 #25 v25)
pub fn router_with_state(state: Arc<ImApiState>) -> Router {
    Router::new()
        .route("/conversations", post(create_conversation))
        .route("/conversations", get(list_conversations))
        .route(
            "/conversations/{conversation_id}/messages",
            post(send_message),
        )
        .route(
            "/conversations/{conversation_id}/messages",
            get(list_messages),
        )
        .route("/conversations/{conversation_id}/read", post(mark_read))
        .route("/presence", get(presence))
        .with_state(state)
}

// ---------------------------------------------------------------------
// 错误 helpers (本文件内构造 6-field RestError, 不动 error.rs)
// ---------------------------------------------------------------------

/// 400 VALIDATION_FAILED
fn im_validation(message: impl Into<String>, hint: impl Into<String>) -> RestError {
    RestError {
        code: "VALIDATION_FAILED".to_string(),
        message: message.into(),
        source_module: "star-api-rest".to_string(),
        source_kind: "Validation".to_string(),
        retriable: false,
        hint: hint.into(),
    }
}

/// 404 RESOURCE_NOT_FOUND (会话不存在)
fn conv_not_found(conversation_id: &str) -> RestError {
    RestError {
        code: "RESOURCE_NOT_FOUND".to_string(),
        message: format!("im conversation not found: {conversation_id}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "NotFound".to_string(),
        retriable: false,
        hint: "Create the conversation first via POST /api/v1/im/conversations".to_string(),
    }
}

/// 403 POLICY_DENIED (非会话成员)
fn not_member(user_id: &str, conversation_id: &str) -> RestError {
    RestError {
        code: "POLICY_DENIED".to_string(),
        message: format!("user {user_id} is not a member of conversation {conversation_id}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "Policy".to_string(),
        retriable: false,
        hint: "Join the conversation before sending messages or marking read".to_string(),
    }
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 当前用户: `X-User-Id` 请求头。
///
/// 中间件 stub 实装前由请求头派生 (同 routes/runtime.rs ActorContext 占位模式);
/// 实装后从 ActorContext/JWT claim 注入替换。
fn current_user(headers: &HeaderMap) -> Result<String, RestError> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            im_validation(
                "missing or empty X-User-Id header",
                "Set the X-User-Id header; auth middleware will inject it from ActorContext later",
            )
        })
}

/// DM 幂等键: 规范化 (min, max) 对 (IM1.0 `create_dm` 的 user_a < user_b 规范化)
fn dm_pair_key(a: &str, b: &str) -> String {
    if a < b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

/// 去重成员 id (保持首次出现顺序)
fn dedupe_members(ids: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    ids.iter()
        .filter(|id| seen.insert(id.as_str()))
        .cloned()
        .collect()
}

/// 消息 → wire JSON (message_id 命名 per Star 仓 `<resource>_id` 惯例;
/// IM1.0 对应 `Message.id`)
fn message_to_value(m: &Message) -> Value {
    json!({
        "message_id": m.id,
        "conversation_id": m.conversation_id,
        "sequence": m.sequence,
        "sender_id": m.sender_id,
        "kind": m.kind,
        "content": m.content,
        "reply_to": m.reply_to,
        "state": m.state,
        "created_at": m.created_at,
    })
}

// ---------------------------------------------------------------------
// request shapes
// ---------------------------------------------------------------------

/// POST /api/v1/im/conversations body
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    /// 成员 id 数组 (含当前用户; 2 人 → DM, ≥3 人 → 群聊)。
    /// `memberIds` 为任务原文拼写的兼容别名, 规范 wire 字段为 snake_case。
    #[serde(alias = "memberIds")]
    pub member_ids: Vec<String>,
    /// 群聊标题 (可选; DM 忽略, per IM1.0 create_dm metadata 为空对象)
    #[serde(default)]
    pub title: Option<String>,
}

/// POST /api/v1/im/conversations/{id}/messages body
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    /// 消息类型 (最小实现仅支持 "text"; IM1.0 按 kind 做 content schema 校验)
    pub kind: String,
    /// 消息正文 (JSON object, per IM1.0 send_message 第 2a 步非空 object 校验)
    pub content: Value,
    /// 被回复消息 id (IM1.0 `reply_to: Option<MessageId>`)
    #[serde(default)]
    pub reply_to: Option<String>,
}

/// GET /api/v1/im/conversations/{id}/messages query
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    /// 只返回 sequence > after_sequence 的消息 (IM1.0 `list_after_sequence`);
    /// 缺省 0 = 从头拉取
    #[serde(default)]
    pub after_sequence: Option<i64>,
    /// 单页条数 (1..=200, per IM1.0 MessageService::MAX_PAGE)
    #[serde(default)]
    pub limit: Option<i32>,
}

/// POST /api/v1/im/conversations/{id}/read body
#[derive(Debug, Deserialize)]
pub struct MarkReadRequest {
    /// 已读到的 sequence (IM1.0 mark_read: 负值拒绝, 上界夹紧到已分配最大值)
    pub sequence: i64,
}

// ---------------------------------------------------------------------
// handlers
// ---------------------------------------------------------------------

/// `POST /api/v1/im/conversations` — 创建 DM/群聊
///
/// 规则 (IM1.0 语义对齐):
/// - 去重后 2 人 → DM, 幂等: 同对用户已存在 DM 则返回既有会话
///   (IM1.0 `create_dm` 的 find_dm 短路 + dm_pairs 登记)
/// - 去重后 ≥3 人 → 群聊, 当前用户为 owner, 其余 member (IM1.0 `create_group`)
/// - 1 人 → 400 (IM1.0 "cannot create DM with self" / "group needs at least 2")
/// - 超 500 人 → 400 (IM1.0 "group too large (max 500)")
pub async fn create_conversation(
    State(state): State<Arc<ImApiState>>,
    headers: HeaderMap,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    if req.member_ids.is_empty() {
        return Err(im_validation(
            "member_ids must not be empty",
            "Provide at least the current user id in member_ids",
        ));
    }

    // 当前用户 + 请求成员去重 (当前用户必须是成员之一, per IM1.0 creator 语义)
    let mut all = vec![user_id.clone()];
    all.extend(req.member_ids.iter().cloned());
    let members = dedupe_members(&all);
    if !members.contains(&user_id) {
        return Err(im_validation(
            "current user must be in member_ids",
            "Include the X-User-Id value in member_ids",
        ));
    }
    if members.len() > MAX_GROUP_MEMBERS {
        return Err(im_validation(
            format!("too many members (max {MAX_GROUP_MEMBERS})"),
            "Reduce member count",
        ));
    }

    let kind = match members.len() {
        2 => ConversationKind::Dm,
        n if n >= 3 => ConversationKind::Group,
        _ => {
            return Err(im_validation(
                "conversation needs at least 2 distinct members",
                "DM requires 2 distinct users; group requires at least 3",
            ));
        }
    };

    // DM 幂等短路 (IM1.0 find_dm): 同对用户返回既有会话
    if kind == ConversationKind::Dm {
        let key = dm_pair_key(&members[0], &members[1]);
        let dm_index = state.dm_index.lock().expect("im dm_index poisoned");
        if let Some(existing_id) = dm_index.get(&key) {
            let convs = state
                .conversations
                .lock()
                .expect("im conversations poisoned");
            if let Some(existing) = convs.get(existing_id) {
                return Ok(Json(RestResponse::ok(json!({
                    "conversation_id": existing.id,
                    "kind": existing.kind,
                    "title": existing.title,
                    "members": existing.members.iter().map(|m| json!({
                        "user_id": m.user_id,
                        "role": m.role,
                    })).collect::<Vec<_>>(),
                    "created_at": existing.created_at,
                }))));
            }
        }
    }

    let conv = Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        kind,
        // DM 忽略 title (IM1.0 create_dm metadata 为空对象)
        title: if kind == ConversationKind::Dm {
            None
        } else {
            req.title.clone()
        },
        members: members
            .iter()
            .map(|id| Member {
                user_id: id.clone(),
                role: if *id == user_id {
                    MemberRole::Owner
                } else {
                    MemberRole::Member
                },
                last_read_sequence: 0,
            })
            .collect(),
        next_sequence: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    {
        let mut convs = state
            .conversations
            .lock()
            .expect("im conversations poisoned");
        convs.insert(conv.id.clone(), conv.clone());
    }
    if kind == ConversationKind::Dm {
        let key = dm_pair_key(&members[0], &members[1]);
        state
            .dm_index
            .lock()
            .expect("im dm_index poisoned")
            .insert(key, conv.id.clone());
    }

    Ok(Json(RestResponse::ok(json!({
        "conversation_id": conv.id,
        "kind": conv.kind,
        "title": conv.title,
        "members": conv.members.iter().map(|m| json!({
            "user_id": m.user_id,
            "role": m.role,
        })).collect::<Vec<_>>(),
        "created_at": conv.created_at,
    }))))
}

/// `GET /api/v1/im/conversations` — 列出当前用户的会话 + 未读数
///
/// 未读数 = 该会话内 sequence > 我的 last_read_sequence 且 sender != 我的消息数
/// (IM1.0 `ConversationMember.last_read_sequence` 派生; IM1.0 REST 无现成未读
/// 字段, 此为任务要求的补充展示)。
pub async fn list_conversations(
    State(state): State<Arc<ImApiState>>,
    headers: HeaderMap,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;

    let convs = state
        .conversations
        .lock()
        .expect("im conversations poisoned");
    let msgs = state.messages.lock().expect("im messages poisoned");

    let mut mine: Vec<Value> = convs
        .values()
        .filter(|c| c.members.iter().any(|m| m.user_id == user_id))
        .map(|c| {
            let my_last_read = c
                .members
                .iter()
                .find(|m| m.user_id == user_id)
                .map(|m| m.last_read_sequence)
                .unwrap_or(0);
            let all = msgs.get(&c.id).cloned().unwrap_or_default();
            let unread = all
                .iter()
                .filter(|m| m.sequence > my_last_read && m.sender_id != user_id)
                .count() as i64;
            let last_sequence = all.last().map(|m| m.sequence).unwrap_or(0);
            json!({
                "conversation_id": c.id,
                "kind": c.kind,
                "title": c.title,
                "members": c.members.iter().map(|m| json!({
                    "user_id": m.user_id,
                    "role": m.role,
                })).collect::<Vec<_>>(),
                "unread": unread,
                "last_sequence": last_sequence,
                "created_at": c.created_at,
            })
        })
        .collect();

    // 按 created_at 倒序 (IM1.0 list_for_user: created_at DESC, id DESC)
    mine.sort_by(|a, b| {
        b.get("created_at")
            .and_then(|v| v.as_str())
            .cmp(&a.get("created_at").and_then(|v| v.as_str()))
    });

    Ok(Json(RestResponse::ok(json!({ "conversations": mine }))))
}

/// `POST /api/v1/im/conversations/{conversation_id}/messages` — 发消息
///
/// 步骤 (IM1.0 MessageService 5 步对齐, 幂等键/事件发布留给持久化+WS 网关):
/// 1. 会话成员校验 (403, IM1.0 2d)
/// 2. kind + content 校验 (仅 text; object + ≤65536 字节, IM1.0 2a/2b/2c)
/// 3. reply_to 存在性校验 (400; IM1.0 在协议层校验, 本实现前置到此处)
/// 4. 会话内 sequence 单调自增 (IM1.0 SequenceAllocator 语义)
/// 5. 落内存 + 返回 Message (state = sent, IM1.0 落库 state)
pub async fn send_message(
    State(state): State<Arc<ImApiState>>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;

    if req.kind != "text" {
        return Err(im_validation(
            format!("unsupported message kind: {}", req.kind),
            "Only kind 'text' is supported in this minimal implementation",
        ));
    }
    if !matches!(req.content, Value::Object(_)) {
        return Err(im_validation(
            "content must be a JSON object",
            "Send content as a JSON object, e.g. {\"text\": \"hi\"}",
        ));
    }
    let serialized = serde_json::to_string(&req.content).unwrap_or_default();
    if serialized.len() > MAX_CONTENT_BYTES {
        return Err(im_validation(
            format!(
                "content too large ({} bytes > {MAX_CONTENT_BYTES})",
                serialized.len()
            ),
            "Reduce message size",
        ));
    }

    // sequence 分配 + 成员校验 (同一把锁内, 对应 IM1.0 事务内取 sequence)
    let sequence = {
        let mut convs = state
            .conversations
            .lock()
            .expect("im conversations poisoned");
        let conv = convs
            .get_mut(&conversation_id)
            .ok_or_else(|| conv_not_found(&conversation_id))?;
        if !conv.members.iter().any(|m| m.user_id == user_id) {
            return Err(not_member(&user_id, &conversation_id));
        }
        conv.next_sequence += 1;
        conv.next_sequence
    };

    // reply_to 存在性校验 (IM1.0 协议层校验, 此处最小实现前置)
    if let Some(reply_id) = &req.reply_to {
        let msgs = state.messages.lock().expect("im messages poisoned");
        let exists = msgs
            .get(&conversation_id)
            .is_some_and(|v| v.iter().any(|m| m.id == *reply_id));
        drop(msgs);
        if !exists {
            return Err(im_validation(
                format!("reply_to message not found: {reply_id}"),
                "reply_to must reference an existing message in the same conversation",
            ));
        }
    }

    let msg = Message {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: conversation_id.clone(),
        sequence,
        sender_id: user_id,
        kind: req.kind,
        content: req.content,
        reply_to: req.reply_to,
        state: MessageState::Sent,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .messages
        .lock()
        .expect("im messages poisoned")
        .entry(conversation_id)
        .or_default()
        .push(msg.clone());

    Ok(Json(RestResponse::ok(message_to_value(&msg))))
}

/// `GET /api/v1/im/conversations/{conversation_id}/messages` — 按 sequence 分页拉取
///
/// `after_sequence` 缺省 0; limit clamp 1..=200 (IM1.0 `list_messages` +
/// `MAX_PAGE`); `has_more` 用同一上限计算 (IM1.0 handler has_more 教训:
/// 上限必须在 service 与 handler 间一致, 否则客户端静默停止翻页)。
pub async fn list_messages(
    State(state): State<Arc<ImApiState>>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    let after = query.after_sequence.unwrap_or(0);
    let limit = query.limit.unwrap_or(MAX_PAGE).clamp(1, MAX_PAGE);

    {
        let convs = state
            .conversations
            .lock()
            .expect("im conversations poisoned");
        let conv = convs
            .get(&conversation_id)
            .ok_or_else(|| conv_not_found(&conversation_id))?;
        if !conv.members.iter().any(|m| m.user_id == user_id) {
            return Err(not_member(&user_id, &conversation_id));
        }
    }

    let msgs = state.messages.lock().expect("im messages poisoned");
    let all = msgs.get(&conversation_id).cloned().unwrap_or_default();
    let filtered: Vec<&Message> = all
        .iter()
        .filter(|m| m.sequence > after)
        .take((limit + 1) as usize)
        .collect();
    let has_more = filtered.len() > limit as usize;
    let page: Vec<Value> = filtered
        .iter()
        .take(limit as usize)
        .map(|m| message_to_value(m))
        .collect();
    let next_after_sequence = page
        .last()
        .map(|v| v["sequence"].as_i64().unwrap_or(after))
        .unwrap_or(after);

    Ok(Json(RestResponse::ok(json!({
        "messages": page,
        "has_more": has_more,
        "next_after_sequence": next_after_sequence,
    }))))
}

/// `POST /api/v1/im/conversations/{conversation_id}/read` — 标记已读
///
/// 守卫 (IM1.0 `ConversationService::mark_read` 全量对齐):
/// - 负值 → 400
/// - 非成员 → 403 (IM1.0 "不能让任意用户给任意会话写 last_read_sequence")
/// - 上界夹紧: effective = min(sequence, 已分配最大 sequence)
///   (IM1.0 2026-10-08 夹紧: 传 i64::MAX 会永久顶死读指针)
/// - 单调守卫: 仅在 effective > 当前读指针时推进, 响应回显服务端实际值
pub async fn mark_read(
    State(state): State<Arc<ImApiState>>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<MarkReadRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    if req.sequence < 0 {
        return Err(im_validation(
            "sequence must be >= 0",
            "Mark-read sequence cannot be negative",
        ));
    }

    let (effective, advanced) = {
        let mut convs = state
            .conversations
            .lock()
            .expect("im conversations poisoned");
        let conv = convs
            .get_mut(&conversation_id)
            .ok_or_else(|| conv_not_found(&conversation_id))?;
        if !conv.members.iter().any(|m| m.user_id == user_id) {
            return Err(not_member(&user_id, &conversation_id));
        }
        // 已分配最大 sequence (IM1.0 `max_allocated_sequence` = 已分配水位)
        let max_allocated = conv.next_sequence;
        let clamped = req.sequence.min(max_allocated);
        let member = conv
            .members
            .iter_mut()
            .find(|m| m.user_id == user_id)
            .expect("member checked above");
        let advanced = clamped > member.last_read_sequence;
        if advanced {
            member.last_read_sequence = clamped;
        }
        // 响应回显**操作后**服务端实际值 (IM1.0 mark_read handler 回报实际值
        // 语义): 未推进的重复上报也回显存量指针, 不回显被夹紧的入参。
        (member.last_read_sequence, advanced)
    };

    let msgs = state.messages.lock().expect("im messages poisoned");
    let unread = msgs
        .get(&conversation_id)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter(|m| m.sequence > effective && m.sender_id != user_id)
        .count() as i64;
    drop(msgs);

    Ok(Json(RestResponse::ok(json!({
        "conversation_id": conversation_id,
        "user_id": user_id,
        "last_read_sequence": effective,
        "advanced": advanced,
        "unread": unread,
    }))))
}

/// `GET /api/v1/im/presence` — 在线成员列表
///
/// IM1.0 `crates/im-presence` 同为占位 (V1 接入 Valkey 维护在线状态 / typing /
/// last_seen)。本实现无真实连接 (WS 网关未接入), 恒返回空成员列表 +
/// `connected:false`; 网关实装后由 presence_update/typing 帧驱动
/// (per IM1.0 im-protocol::ws_frames ServerFrame 变体)。
pub async fn presence() -> Json<RestResponse<Value>> {
    Json(RestResponse::ok(json!({
        "connected": false,
        "members": [],
    })))
}

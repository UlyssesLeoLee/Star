/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/src/routes/im.rs"})
MERGE (state:Type {id:"crates/star-api-rest/src/routes/im.rs::ImApiState",kind:"struct"})
MERGE (conv:Type {id:"crates/star-api-rest/src/routes/im.rs::Conversation",kind:"struct"})
MERGE (member:Type {id:"crates/star-api-rest/src/routes/im.rs::Member",kind:"struct"})
MERGE (msg:Type {id:"crates/star-api-rest/src/routes/im.rs::Message",kind:"struct"})
MERGE (kind:Type {id:"crates/star-api-rest/src/routes/im.rs::ConversationKind",kind:"enum"})
MERGE (role:Type {id:"crates/star-api-rest/src/routes/im.rs::MemberRole",kind:"enum"})
MERGE (mstate:Type {id:"crates/star-api-rest/src/routes/im.rs::MessageState",kind:"enum"})
MERGE (storeTrait:Type {id:"crates/star-api-rest/src/routes/im.rs::ImStore",kind:"trait"})
MERGE (inMemStore:Type {id:"crates/star-api-rest/src/routes/im.rs::InMemoryImStore",kind:"struct"})
MERGE (pgStore:Type {id:"crates/star-api-rest/src/routes/im.rs::PgImStore",kind:"struct"})
MERGE (gwTrait:Type {id:"crates/star-api-rest/src/routes/im.rs::ImGateway",kind:"trait"})
MERGE (unconnectedGw:Type {id:"crates/star-api-rest/src/routes/im.rs::UnconnectedImGateway",kind:"struct"})
MERGE (createConv:Symbol {id:"crates/star-api-rest/src/routes/im.rs::create_conversation",kind:"function"})
MERGE (listConvs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_conversations",kind:"function"})
MERGE (sendMsg:Symbol {id:"crates/star-api-rest/src/routes/im.rs::send_message",kind:"function"})
MERGE (listMsgs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_messages",kind:"function"})
MERGE (markRead:Symbol {id:"crates/star-api-rest/src/routes/im.rs::mark_read",kind:"function"})
MERGE (presence:Symbol {id:"crates/star-api-rest/src/routes/im.rs::presence",kind:"function"})
MERGE (vCreateConv:Symbol {id:"crates/star-api-rest/src/routes/im.rs::create_conversation_verified",kind:"function"})
MERGE (vListConvs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_conversations_verified",kind:"function"})
MERGE (vSendMsg:Symbol {id:"crates/star-api-rest/src/routes/im.rs::send_message_verified",kind:"function"})
MERGE (vListMsgs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_messages_verified",kind:"function"})
MERGE (vMarkRead:Symbol {id:"crates/star-api-rest/src/routes/im.rs::mark_read_verified",kind:"function"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(conv)
MERGE (self)-[:DEFINES]->(member)
MERGE (self)-[:DEFINES]->(msg)
MERGE (self)-[:DEFINES]->(kind)
MERGE (self)-[:DEFINES]->(role)
MERGE (self)-[:DEFINES]->(mstate)
MERGE (self)-[:DEFINES]->(storeTrait)
MERGE (self)-[:DEFINES]->(inMemStore)
MERGE (self)-[:DEFINES]->(pgStore)
MERGE (self)-[:DEFINES]->(gwTrait)
MERGE (self)-[:DEFINES]->(unconnectedGw)
MERGE (self)-[:DEFINES]->(createConv)
MERGE (self)-[:DEFINES]->(listConvs)
MERGE (self)-[:DEFINES]->(sendMsg)
MERGE (self)-[:DEFINES]->(listMsgs)
MERGE (self)-[:DEFINES]->(markRead)
MERGE (self)-[:DEFINES]->(presence)
MERGE (self)-[:DEFINES]->(vCreateConv)
MERGE (self)-[:DEFINES]->(vListConvs)
MERGE (self)-[:DEFINES]->(vSendMsg)
MERGE (self)-[:DEFINES]->(vListMsgs)
MERGE (self)-[:DEFINES]->(vMarkRead)
MERGE (conv)-[:HAS_MEMBER]->(member)
MERGE (conv)-[:HAS_MEMBER]->(msg)
MERGE (state)-[:USES_TYPE]->(storeTrait)
MERGE (state)-[:USES_TYPE]->(gwTrait)
MERGE (inMemStore)-[:IMPLEMENTS]->(storeTrait)
MERGE (pgStore)-[:IMPLEMENTS]->(storeTrait)
MERGE (unconnectedGw)-[:IMPLEMENTS]->(gwTrait)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// im.rs — IM 即时通信端点 (用户间消息)
//
// 2026-11-20 P0 批次: 生产鉴权 fail-closed + 持久化 port + 会话成员隔离。
//
// 路径面 (与 2026-11-16 版完全一致, wire 契约不变):
//   POST /api/v1/im/conversations                        — 创建 DM/群聊
//   GET  /api/v1/im/conversations                        — 列出当前用户的会话 + 未读数
//   POST /api/v1/im/conversations/{conversation_id}/messages — 发消息
//   GET  /api/v1/im/conversations/{conversation_id}/messages — 按 sequence 分页拉取
//   POST /api/v1/im/conversations/{conversation_id}/read  — 标记已读 (body 含 sequence)
//   GET  /api/v1/im/presence                              — 在线成员列表 (无真实连接 → 空 + connected:false)
//
// IM1.0 语义对齐 (同 2026-11-16 版): ConversationKind/MemberRole/
// last_read_sequence/MessageState/sequence 单调自增/DM 幂等/mark_read
// 夹紧与单调守卫/presence 占位。独立实现避免跨仓依赖 (per 任务约束)。
//
// 本批次变更:
// 1. 生产鉴权 fail-closed: `router()` = verified 路由 + JWT 验签 middleware
//    (复用 routes/notifications.rs 的 `verify_jwt_and_inject_actor` +
//    `prod_jwt_config` — group/SCM 域同一套 JwtConfig/verify_token);
//    无/无效 token → 401。verified handler 身份取已验证会话
//    (`VerifiedActor` extractor), 缺失 → 401 — 不读 X-User-Id。
// 2. 会话成员隔离 + 租户隔离: Conversation 新增 tenant_id 维度;
//    verified 读写限 tenant == actor.tenant 且 actor ∈ members, 否则 403。
// 3. 持久化: 读写走 `ImStore` port trait; `InMemoryImStore` (测试/无 DB)
//    与 `PgImStore` (sqlx PostgreSQL, DDL 见 `IM_PG_DDL`: conversations /
//    members / messages / dm_pairs / sequences, sequence 走行锁 upsert)
//    双实现; 生产 `router()` 读 DATABASE_URL (有 → PG, 无 → 内存 + warn)。
// 4. IM 网关: 定义 `ImGateway` port + `UnconnectedImGateway` (诚实标注
//    「网关未接入」— push 恒返回 NotConnected, 不假发送); 发消息成功落库
//    后 best-effort push (失败仅 tracing::warn, 不影响消息持久化)。
//
// 已知简化 (诚实标注):
// - 跨租户成员校验: member_ids 里的其他用户租户归属无法核验 (无 user
//   directory 依赖), 会话租户按创建者会话确定 — 父代理待办。
// - presence/WS 推送: 网关未接入 (IM1.0 crates/im-presence 同为占位)。
// - 消息 → 网关的事件派发走 Outbox 的改造留网关实装时 (当前无投递目标)。
//
// 注册: parent 统一注册 (routes/mod.rs `pub mod im;` + lib.rs
// `.nest("/im", routes::im::router())`) — 本文件不自行注册。
// 本文件以 `crate::` 路径引用 error/response 模块。
// =====================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
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

use super::notifications::{prod_jwt_config, verify_jwt_and_inject_actor, VerifiedActor};

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
    /// 所属租户 id (P0 租户隔离; legacy 构造为空串)
    pub tenant_id: String,
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
// IM 网关 port (诚实标注: 未接入)
// ---------------------------------------------------------------------

/// IM 网关错误 (port 边界)
#[derive(Debug, thiserror::Error)]
pub enum ImGatewayError {
    /// 网关未接入 (无真实投递目标, 不假发送)
    #[error("im gateway not connected: {0}")]
    NotConnected(String),
    /// 网关瞬时失败 (可重试)
    #[error("im gateway transient failure: {0}")]
    Transient(String),
}

/// IM 网关投递端口 (WS/外部 IM 推送)。
///
/// 仓库内无现成 feishu/lark/IM gateway adapter (platform-messaging-* 为
/// BI JetStream 事件平台, 与 IM 无关, per 本 lane 侦察); 本 trait 为
/// 「网关未接入」诚实标注的最小端口。网关实装后由父代理注入真实现。
#[async_trait]
pub trait ImGateway: Send + Sync {
    /// 推送消息到会话的在线成员 (best-effort; 失败不影响消息持久化)
    async fn push_message(&self, message: &Message) -> Result<(), ImGatewayError>;
}

/// 未接入网关: push 恒返回 NotConnected (不假发送, per 守门 #2)
pub struct UnconnectedImGateway;

#[async_trait]
impl ImGateway for UnconnectedImGateway {
    async fn push_message(&self, _message: &Message) -> Result<(), ImGatewayError> {
        Err(ImGatewayError::NotConnected(
            "no IM gateway wired; messages are persisted and readable via pull".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------
// IM 存储 port trait (会话/成员/消息/序列/DM 索引)
// ---------------------------------------------------------------------

/// IM 存储层错误 (port 边界)
#[derive(Debug, thiserror::Error)]
pub enum ImStoreError {
    /// 瞬时不可用
    #[error("im store unavailable: {0}")]
    Unavailable(String),
    /// 序列化失败
    #[error("serialization: {0}")]
    Serialization(String),
}

/// IM 持久化端口。
///
/// 内存实现 (`InMemoryImStore`) 与 PostgreSQL 实现 (`PgImStore`) 共享本
/// trait; 数据不落在 handler 内存 (重启可恢复由 PG 实现保证)。
#[async_trait]
pub trait ImStore: Send + Sync {
    /// 插入新会话 (含成员 + 序列水位 0)
    async fn insert_conversation(&self, c: Conversation) -> Result<(), ImStoreError>;
    /// 按 id 取会话 (不存在 → Ok(None))
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, ImStoreError>;
    /// 列出某用户参与的会话 (tenant 过滤可选)
    async fn list_conversations_for_user(
        &self,
        tenant_id: Option<&str>,
        user_id: &str,
    ) -> Result<Vec<Conversation>, ImStoreError>;
    /// 会话内序列分配 (单调自增; 不存在会话 → Ok(None))
    async fn allocate_sequence(&self, conversation_id: &str) -> Result<Option<i64>, ImStoreError>;
    /// 推进成员已读指针 (单调守卫: 只进不退); 返回更新后的已读值
    async fn advance_member_read(
        &self,
        conversation_id: &str,
        user_id: &str,
        last_read_sequence: i64,
    ) -> Result<Option<i64>, ImStoreError>;
    /// 追加消息
    async fn append_message(&self, m: Message) -> Result<(), ImStoreError>;
    /// 列出会话全部消息 (sequence 升序)
    async fn list_messages(&self, conversation_id: &str) -> Result<Vec<Message>, ImStoreError>;
    /// DM 幂等索引查询 (dm_key → conversation_id)
    async fn find_dm(&self, dm_key: &str) -> Result<Option<String>, ImStoreError>;
    /// DM 幂等索引登记
    async fn link_dm(&self, dm_key: &str, conversation_id: &str) -> Result<(), ImStoreError>;
}

// ---------------------------------------------------------------------
// InMemoryImStore (测试/无 DB 环境)
// ---------------------------------------------------------------------

/// 内存实现 (port trait 双实现之一; 数据在 store 而非 handler)
pub struct InMemoryImStore {
    conversations: Mutex<HashMap<String, Conversation>>,
    messages: Mutex<HashMap<String, Vec<Message>>>,
    dm_index: Mutex<HashMap<String, String>>,
}

impl InMemoryImStore {
    /// 新建空 store
    pub fn new() -> Self {
        Self {
            conversations: Mutex::new(HashMap::new()),
            messages: Mutex::new(HashMap::new()),
            dm_index: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryImStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImStore for InMemoryImStore {
    async fn insert_conversation(&self, c: Conversation) -> Result<(), ImStoreError> {
        self.conversations
            .lock()
            .expect("im conversations poisoned")
            .insert(c.id.clone(), c);
        Ok(())
    }

    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, ImStoreError> {
        Ok(self
            .conversations
            .lock()
            .expect("im conversations poisoned")
            .get(id)
            .cloned())
    }

    async fn list_conversations_for_user(
        &self,
        tenant_id: Option<&str>,
        user_id: &str,
    ) -> Result<Vec<Conversation>, ImStoreError> {
        Ok(self
            .conversations
            .lock()
            .expect("im conversations poisoned")
            .values()
            .filter(|c| tenant_id.is_none_or(|t| c.tenant_id == t))
            .filter(|c| c.members.iter().any(|m| m.user_id == user_id))
            .cloned()
            .collect())
    }

    async fn allocate_sequence(&self, conversation_id: &str) -> Result<Option<i64>, ImStoreError> {
        let mut convs = self
            .conversations
            .lock()
            .expect("im conversations poisoned");
        let Some(conv) = convs.get_mut(conversation_id) else {
            return Ok(None);
        };
        conv.next_sequence += 1;
        Ok(Some(conv.next_sequence))
    }

    async fn advance_member_read(
        &self,
        conversation_id: &str,
        user_id: &str,
        last_read_sequence: i64,
    ) -> Result<Option<i64>, ImStoreError> {
        let mut convs = self
            .conversations
            .lock()
            .expect("im conversations poisoned");
        let Some(conv) = convs.get_mut(conversation_id) else {
            return Ok(None);
        };
        let Some(member) = conv.members.iter_mut().find(|m| m.user_id == user_id) else {
            return Ok(None);
        };
        member.last_read_sequence = member.last_read_sequence.max(last_read_sequence);
        Ok(Some(member.last_read_sequence))
    }

    async fn append_message(&self, m: Message) -> Result<(), ImStoreError> {
        self.messages
            .lock()
            .expect("im messages poisoned")
            .entry(m.conversation_id.clone())
            .or_default()
            .push(m);
        Ok(())
    }

    async fn list_messages(&self, conversation_id: &str) -> Result<Vec<Message>, ImStoreError> {
        Ok(self
            .messages
            .lock()
            .expect("im messages poisoned")
            .get(conversation_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_dm(&self, dm_key: &str) -> Result<Option<String>, ImStoreError> {
        Ok(self
            .dm_index
            .lock()
            .expect("im dm_index poisoned")
            .get(dm_key)
            .cloned())
    }

    async fn link_dm(&self, dm_key: &str, conversation_id: &str) -> Result<(), ImStoreError> {
        self.dm_index
            .lock()
            .expect("im dm_index poisoned")
            .insert(dm_key.to_string(), conversation_id.to_string());
        Ok(())
    }
}

// ---------------------------------------------------------------------
// PgImStore (sqlx PostgreSQL; 真实持久化实现, 验收测试标 #[ignore])
// ---------------------------------------------------------------------

/// PostgreSQL IM DDL (每行一条语句, 与仓库 migration runner 纪律一致;
/// 父代理合并时落地到 db/migrations/ 正式迁移文件)。
pub const IM_PG_DDL: &str = "\
CREATE TABLE IF NOT EXISTS p0_im_conversations (id TEXT PRIMARY KEY, tenant_id TEXT NOT NULL, kind TEXT NOT NULL, title TEXT, created_at TEXT NOT NULL)
CREATE TABLE IF NOT EXISTS p0_im_members (conversation_id TEXT NOT NULL, user_id TEXT NOT NULL, role TEXT NOT NULL, last_read_sequence BIGINT NOT NULL DEFAULT 0, PRIMARY KEY (conversation_id, user_id))
CREATE TABLE IF NOT EXISTS p0_im_messages (id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL, sequence BIGINT NOT NULL, sender_id TEXT NOT NULL, kind TEXT NOT NULL, content TEXT NOT NULL, reply_to TEXT, state TEXT NOT NULL, created_at TEXT NOT NULL, UNIQUE (conversation_id, sequence))
CREATE TABLE IF NOT EXISTS p0_im_dm_pairs (dm_key TEXT PRIMARY KEY, conversation_id TEXT NOT NULL, user_a TEXT NOT NULL, user_b TEXT NOT NULL)
CREATE TABLE IF NOT EXISTS p0_im_sequences (conversation_id TEXT PRIMARY KEY, next_sequence BIGINT NOT NULL DEFAULT 0)";

/// PostgreSQL 实现 (`ImStore` port 的真实持久化适配; 需要真实 PG,
/// 验收测试 `#[ignore]` + `STAR_P0_NOTIFY_TEST_DATABASE_URL` 跑法)。
pub struct PgImStore {
    pool: sqlx::PgPool,
}

impl PgImStore {
    /// 以既有 pool 构造 (生产由父代理注入)
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取底层 pool (验收测试建表用)
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    /// 取会话 + 成员 + 序列水位 (不存在 → None)
    async fn load_conversation(&self, id: &str) -> Result<Option<Conversation>, ImStoreError> {
        let row = sqlx::query_as::<_, ConversationRow>(
            "SELECT c.id, c.tenant_id, c.kind, c.title, c.created_at, \
             COALESCE(s.next_sequence, 0) AS next_sequence \
             FROM p0_im_conversations c \
             LEFT JOIN p0_im_sequences s ON s.conversation_id = c.id \
             WHERE c.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        let Some(row) = row else {
            return Ok(None);
        };
        let members = self.load_members(id).await?;
        Ok(Some(Conversation {
            id: row.id,
            tenant_id: row.tenant_id,
            kind: parse_conv_kind_loose(&row.kind),
            title: row.title,
            members,
            next_sequence: row.next_sequence,
            created_at: row.created_at,
        }))
    }

    /// 取会话成员列表
    async fn load_members(&self, id: &str) -> Result<Vec<Member>, ImStoreError> {
        let rows = sqlx::query_as::<_, MemberRow>(
            "SELECT user_id, role, last_read_sequence FROM p0_im_members \
             WHERE conversation_id = $1 ORDER BY user_id ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|m| Member {
                user_id: m.user_id,
                role: parse_role_loose(&m.role),
                last_read_sequence: m.last_read_sequence,
            })
            .collect())
    }
}

/// PG 会话行 (含序列水位)
#[derive(Debug, sqlx::FromRow)]
struct ConversationRow {
    id: String,
    tenant_id: String,
    kind: String,
    title: Option<String>,
    created_at: String,
    next_sequence: i64,
}

/// PG 成员行
#[derive(Debug, sqlx::FromRow)]
struct MemberRow {
    user_id: String,
    role: String,
    last_read_sequence: i64,
}

/// PG 消息行
#[derive(Debug, sqlx::FromRow)]
struct MessageRow {
    id: String,
    conversation_id: String,
    sequence: i64,
    sender_id: String,
    kind: String,
    content: String,
    reply_to: Option<String>,
    state: String,
    created_at: String,
}

impl MessageRow {
    /// Row → 领域 Message
    fn into_message(self) -> Result<Message, ImStoreError> {
        let content: Value = serde_json::from_str(&self.content)
            .map_err(|e| ImStoreError::Serialization(e.to_string()))?;
        let state = match self.state.as_str() {
            "delivered" => MessageState::Delivered,
            "read" => MessageState::Read,
            "recalled" => MessageState::Recalled,
            "deleted" => MessageState::Deleted,
            _ => MessageState::Sent,
        };
        Ok(Message {
            id: self.id,
            conversation_id: self.conversation_id,
            sequence: self.sequence,
            sender_id: self.sender_id,
            kind: self.kind,
            content,
            reply_to: self.reply_to,
            state,
            created_at: self.created_at,
        })
    }
}

/// member role 字符串 → MemberRole (宽松)
fn parse_role_loose(s: &str) -> MemberRole {
    match s {
        "owner" => MemberRole::Owner,
        "admin" => MemberRole::Admin,
        _ => MemberRole::Member,
    }
}

/// kind 字符串 → ConversationKind (宽松)
fn parse_conv_kind_loose(s: &str) -> ConversationKind {
    match s {
        "group" => ConversationKind::Group,
        _ => ConversationKind::Dm,
    }
}

/// kind → 存储字符串
fn serde_conv_kind_str(kind: &ConversationKind) -> &'static str {
    match kind {
        ConversationKind::Dm => "dm",
        ConversationKind::Group => "group",
    }
}

/// role → 存储字符串
fn serde_role_str(role: &MemberRole) -> &'static str {
    match role {
        MemberRole::Owner => "owner",
        MemberRole::Admin => "admin",
        MemberRole::Member => "member",
    }
}

/// state → 存储字符串
fn serde_state_str(state: &MessageState) -> &'static str {
    match state {
        MessageState::Sent => "sent",
        MessageState::Delivered => "delivered",
        MessageState::Read => "read",
        MessageState::Recalled => "recalled",
        MessageState::Deleted => "deleted",
    }
}

#[async_trait]
impl ImStore for PgImStore {
    async fn insert_conversation(&self, c: Conversation) -> Result<(), ImStoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        sqlx::query(
            "INSERT INTO p0_im_conversations (id, tenant_id, kind, title, created_at) \
             VALUES ($1,$2,$3,$4,$5) ON CONFLICT (id) DO NOTHING",
        )
        .bind(&c.id)
        .bind(&c.tenant_id)
        .bind(serde_conv_kind_str(&c.kind))
        .bind(&c.title)
        .bind(&c.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        for member in &c.members {
            sqlx::query(
                "INSERT INTO p0_im_members (conversation_id, user_id, role, last_read_sequence) \
                 VALUES ($1,$2,$3,$4) ON CONFLICT (conversation_id, user_id) DO NOTHING",
            )
            .bind(&c.id)
            .bind(&member.user_id)
            .bind(serde_role_str(&member.role))
            .bind(member.last_read_sequence)
            .execute(&mut *tx)
            .await
            .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        }
        sqlx::query(
            "INSERT INTO p0_im_sequences (conversation_id, next_sequence) \
             VALUES ($1, $2) ON CONFLICT (conversation_id) DO NOTHING",
        )
        .bind(&c.id)
        .bind(c.next_sequence)
        .execute(&mut *tx)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| ImStoreError::Unavailable(e.to_string()))
    }

    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, ImStoreError> {
        self.load_conversation(id).await
    }

    async fn list_conversations_for_user(
        &self,
        tenant_id: Option<&str>,
        user_id: &str,
    ) -> Result<Vec<Conversation>, ImStoreError> {
        let rows = sqlx::query_as::<_, ConversationRow>(
            "SELECT c.id, c.tenant_id, c.kind, c.title, c.created_at, \
             COALESCE(s.next_sequence, 0) AS next_sequence \
             FROM p0_im_conversations c \
             JOIN p0_im_members m ON m.conversation_id = c.id \
             LEFT JOIN p0_im_sequences s ON s.conversation_id = c.id \
             WHERE m.user_id = $1 AND ($2::text IS NULL OR c.tenant_id = $2) \
             ORDER BY c.created_at DESC, c.id DESC",
        )
        .bind(user_id)
        .bind(tenant_id.map(|s| s.to_string()))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        let mut conversations = Vec::with_capacity(rows.len());
        for row in rows {
            let members = self.load_members(&row.id).await?;
            conversations.push(Conversation {
                id: row.id,
                tenant_id: row.tenant_id,
                kind: parse_conv_kind_loose(&row.kind),
                title: row.title,
                members,
                next_sequence: row.next_sequence,
                created_at: row.created_at,
            });
        }
        Ok(conversations)
    }

    async fn allocate_sequence(&self, conversation_id: &str) -> Result<Option<i64>, ImStoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        // 会话存在性 (行锁) — 不存在 → Ok(None)
        let exists: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM p0_im_conversations WHERE id = $1 FOR UPDATE")
                .bind(conversation_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        if exists.is_none() {
            tx.rollback()
                .await
                .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
            return Ok(None);
        }
        // 原子自增 (IM1.0 SequenceAllocator 行锁语义)
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO p0_im_sequences (conversation_id, next_sequence) VALUES ($1, 1) \
             ON CONFLICT (conversation_id) DO UPDATE SET next_sequence = p0_im_sequences.next_sequence + 1 \
             RETURNING next_sequence",
        )
        .bind(conversation_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        Ok(Some(row.0))
    }

    async fn advance_member_read(
        &self,
        conversation_id: &str,
        user_id: &str,
        last_read_sequence: i64,
    ) -> Result<Option<i64>, ImStoreError> {
        let row = sqlx::query_as::<_, (i64,)>(
            "UPDATE p0_im_members SET last_read_sequence = GREATEST(last_read_sequence, $3) \
             WHERE conversation_id = $1 AND user_id = $2 RETURNING last_read_sequence",
        )
        .bind(conversation_id)
        .bind(user_id)
        .bind(last_read_sequence)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        Ok(row.map(|r| r.0))
    }

    async fn append_message(&self, m: Message) -> Result<(), ImStoreError> {
        let content = serde_json::to_string(&m.content)
            .map_err(|e| ImStoreError::Serialization(e.to_string()))?;
        sqlx::query(
            "INSERT INTO p0_im_messages (id, conversation_id, sequence, sender_id, kind, content, reply_to, state, created_at) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (id) DO NOTHING",
        )
        .bind(&m.id)
        .bind(&m.conversation_id)
        .bind(m.sequence)
        .bind(&m.sender_id)
        .bind(&m.kind)
        .bind(&content)
        .bind(&m.reply_to)
        .bind(serde_state_str(&m.state))
        .bind(&m.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        Ok(())
    }

    async fn list_messages(&self, conversation_id: &str) -> Result<Vec<Message>, ImStoreError> {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT * FROM p0_im_messages WHERE conversation_id = $1 ORDER BY sequence ASC, id ASC",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        rows.into_iter()
            .map(MessageRow::into_message)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn find_dm(&self, dm_key: &str) -> Result<Option<String>, ImStoreError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT conversation_id FROM p0_im_dm_pairs WHERE dm_key = $1")
                .bind(dm_key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        Ok(row.map(|r| r.0))
    }

    async fn link_dm(&self, dm_key: &str, conversation_id: &str) -> Result<(), ImStoreError> {
        // dm_key = "<tenant>|<min>|<max>" (verified) 或 "<min>|<max>" (legacy);
        // user_a/user_b 仅为审计/可观测性冗余, 尽力解析
        let parts: Vec<&str> = dm_key.split('|').collect();
        let (a, b) = match parts.len() {
            3 => (parts[1].to_string(), parts[2].to_string()),
            2 => (parts[0].to_string(), parts[1].to_string()),
            _ => (String::new(), String::new()),
        };
        sqlx::query(
            "INSERT INTO p0_im_dm_pairs (dm_key, conversation_id, user_a, user_b) \
             VALUES ($1,$2,$3,$4) ON CONFLICT (dm_key) DO NOTHING",
        )
        .bind(dm_key)
        .bind(conversation_id)
        .bind(&a)
        .bind(&b)
        .execute(&self.pool)
        .await
        .map_err(|e| ImStoreError::Unavailable(e.to_string()))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------
// 共享状态 (存储 + 网关 port 注入)
// ---------------------------------------------------------------------

/// IM 端点共享状态。
///
/// 唯一数据字段是存储 port (`ImStore`); 网关为 port 注入 (生产默认
/// 未接入)。数据不落在 handler 内存 (重启可恢复由 PG 实现保证)。
pub struct ImApiState {
    /// 会话/成员/消息/序列/DM 索引存储
    pub store: Arc<dyn ImStore>,
    /// IM 网关 (未接入 → push 恒 NotConnected, 不假发送)
    pub gateway: Arc<dyn ImGateway>,
}

impl ImApiState {
    /// 以注入 store/gateway 新建状态
    pub fn from_parts(store: Arc<dyn ImStore>, gateway: Arc<dyn ImGateway>) -> Self {
        Self { store, gateway }
    }

    /// 新建状态 (内存 store + 未接入网关; 测试隔离用)
    pub fn new() -> Self {
        Self::from_parts(
            Arc::new(InMemoryImStore::new()),
            Arc::new(UnconnectedImGateway),
        )
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

/// 生产存储: DATABASE_URL 可用 → PG (重启可恢复); 否则内存 + warn
fn production_store() -> Arc<dyn ImStore> {
    match std::env::var("DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => match sqlx::postgres::PgPool::connect_lazy(url.trim())
        {
            Ok(pool) => {
                tracing::info!("im: PostgreSQL persistence enabled via DATABASE_URL");
                Arc::new(PgImStore::new(pool))
            }
            Err(e) => {
                tracing::warn!(error = %e, "im: DATABASE_URL invalid, falling back to in-memory (data lost on restart)");
                Arc::new(InMemoryImStore::new())
            }
        },
        _ => {
            tracing::warn!(
                "im: DATABASE_URL not set, using in-memory store (data lost on restart)"
            );
            Arc::new(InMemoryImStore::new())
        }
    }
}

/// 生产嵌套路由用共享状态 (OnceLock)
fn state_arc() -> Arc<ImApiState> {
    static STATE: OnceLock<Arc<ImApiState>> = OnceLock::new();
    Arc::clone(STATE.get_or_init(|| {
        Arc::new(ImApiState::from_parts(
            production_store(),
            Arc::new(UnconnectedImGateway),
        ))
    }))
}

// ---------------------------------------------------------------------
// routers
// ---------------------------------------------------------------------

/// 生产嵌套路由 — verified handler + JWT 验签 middleware (fail-closed;
/// 复用 routes/notifications.rs 的同一份 middleware 与 JWT 配置)。
/// parent 以 `.nest("/im", router())` 挂载 (lib.rs 已注册)。
pub fn router() -> Router {
    router_with_verified_state(state_arc()).layer(axum::middleware::from_fn_with_state(
        prod_jwt_config(),
        verify_jwt_and_inject_actor,
    ))
}

/// caller-supplied state 嵌套路由 (legacy X-User-Id 语义, 仅供旧集成
/// 测试; 生产一律走 `router()`)
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

/// caller-supplied state 嵌套路由 (verified — actor 来自已验证会话
/// extension, 缺 → 401; 租户 + 成员双重隔离)
pub fn router_with_verified_state(state: Arc<ImApiState>) -> Router {
    Router::new()
        .route("/conversations", post(create_conversation_verified))
        .route("/conversations", get(list_conversations_verified))
        .route(
            "/conversations/{conversation_id}/messages",
            post(send_message_verified),
        )
        .route(
            "/conversations/{conversation_id}/messages",
            get(list_messages_verified),
        )
        .route(
            "/conversations/{conversation_id}/read",
            post(mark_read_verified),
        )
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

/// 403 POLICY_DENIED (跨租户会话访问)
fn cross_tenant_conv(actor_tenant: &str, conv_tenant: &str) -> RestError {
    RestError {
        code: "POLICY_DENIED".to_string(),
        message: format!(
            "tenant mismatch: actor tenant {actor_tenant} cannot access conversation of tenant {conv_tenant}"
        ),
        source_module: "star-api-rest".to_string(),
        source_kind: "Policy".to_string(),
        retriable: false,
        hint: "Conversations are scoped to their tenant".to_string(),
    }
}

/// 500 INTERNAL (存储失败)
fn store_internal(err: &ImStoreError) -> RestError {
    RestError {
        code: "INTERNAL".to_string(),
        message: format!("im store failure: {err}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "Store".to_string(),
        retriable: true,
        hint: "Retry; persistence is the single source of truth".to_string(),
    }
}

// ---------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------

/// 当前用户 (legacy): `X-User-Id` 请求头 — 仅供 legacy 测试路由使用,
/// 生产 verified 路径一律从已验证会话取身份。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
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

/// DM 幂等键 (legacy): 规范化 (min, max) 对 (IM1.0 user_a < user_b)
fn dm_pair_key(a: &str, b: &str) -> String {
    if a < b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

/// DM 幂等键 (verified): 租户作用域 (min, max) 对, 防跨租户碰撞
fn dm_pair_key_scoped(tenant_id: &str, a: &str, b: &str) -> String {
    format!("{tenant_id}|{}", dm_pair_key(a, b))
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

/// 会话 → wire JSON (创建响应)
fn conversation_to_value(c: &Conversation) -> Value {
    json!({
        "conversation_id": c.id,
        "tenant_id": c.tenant_id,
        "kind": c.kind,
        "title": c.title,
        "members": c.members.iter().map(|m| json!({
            "user_id": m.user_id,
            "role": m.role,
        })).collect::<Vec<_>>(),
        "created_at": c.created_at,
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
// handlers (legacy X-User-Id 语义; 供旧集成测试, 生产不走此路径)
// ---------------------------------------------------------------------

/// `POST /api/v1/im/conversations` — 创建 DM/群聊 (legacy)
///
/// 规则 (IM1.0 语义对齐): 2 人 → DM 幂等; ≥3 人 → 群聊 (创建者 owner);
/// 1 人 → 400; 超 500 人 → 400。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_conversation(
    State(state): State<Arc<ImApiState>>,
    headers: HeaderMap,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    create_conversation_impl(&state, "", &user_id, req).await
}

/// 创建会话公共实现 (tenant_id: legacy 空串 / verified actor 租户)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
async fn create_conversation_impl(
    state: &Arc<ImApiState>,
    tenant_id: &str,
    user_id: &str,
    req: CreateConversationRequest,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if req.member_ids.is_empty() {
        return Err(im_validation(
            "member_ids must not be empty",
            "Provide at least the current user id in member_ids",
        ));
    }

    // 当前用户 + 请求成员去重 (当前用户必须是成员之一, per IM1.0 creator 语义)
    let mut all = vec![user_id.to_string()];
    all.extend(req.member_ids.iter().cloned());
    let members = dedupe_members(&all);
    if !members.contains(&user_id.to_string()) {
        return Err(im_validation(
            "current user must be in member_ids",
            "Include the current user id in member_ids",
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
        let key = if tenant_id.is_empty() {
            dm_pair_key(&members[0], &members[1])
        } else {
            dm_pair_key_scoped(tenant_id, &members[0], &members[1])
        };
        if let Some(existing_id) = state
            .store
            .find_dm(&key)
            .await
            .map_err(|e| store_internal(&e))?
        {
            if let Some(existing) = state
                .store
                .get_conversation(&existing_id)
                .await
                .map_err(|e| store_internal(&e))?
            {
                return Ok(Json(RestResponse::ok(conversation_to_value(&existing))));
            }
        }
    }

    let conv = Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id: tenant_id.to_string(),
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

    state
        .store
        .insert_conversation(conv.clone())
        .await
        .map_err(|e| store_internal(&e))?;
    if kind == ConversationKind::Dm {
        let key = if tenant_id.is_empty() {
            dm_pair_key(&members[0], &members[1])
        } else {
            dm_pair_key_scoped(tenant_id, &members[0], &members[1])
        };
        state
            .store
            .link_dm(&key, &conv.id)
            .await
            .map_err(|e| store_internal(&e))?;
    }

    Ok(Json(RestResponse::ok(conversation_to_value(&conv))))
}

/// `GET /api/v1/im/conversations` — 列出当前用户的会话 + 未读数 (legacy)
///
/// 未读数 = 该会话内 sequence > 我的 last_read_sequence 且 sender != 我的消息数
/// (IM1.0 `ConversationMember.last_read_sequence` 派生)。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_conversations(
    State(state): State<Arc<ImApiState>>,
    headers: HeaderMap,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    list_conversations_impl(&state, None, &user_id).await
}

/// 会话列表公共实现 (tenant 过滤可选; verified 强制 actor 租户)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
async fn list_conversations_impl(
    state: &Arc<ImApiState>,
    tenant_id: Option<&str>,
    user_id: &str,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let convs = state
        .store
        .list_conversations_for_user(tenant_id, user_id)
        .await
        .map_err(|e| store_internal(&e))?;

    let mut mine: Vec<Value> = Vec::with_capacity(convs.len());
    for c in convs {
        let my_last_read = c
            .members
            .iter()
            .find(|m| m.user_id == user_id)
            .map(|m| m.last_read_sequence)
            .unwrap_or(0);
        let all = state
            .store
            .list_messages(&c.id)
            .await
            .map_err(|e| store_internal(&e))?;
        let unread = all
            .iter()
            .filter(|m| m.sequence > my_last_read && m.sender_id != user_id)
            .count() as i64;
        let last_sequence = all.last().map(|m| m.sequence).unwrap_or(0);
        mine.push(json!({
            "conversation_id": c.id,
            "tenant_id": c.tenant_id,
            "kind": c.kind,
            "title": c.title,
            "members": c.members.iter().map(|m| json!({
                "user_id": m.user_id,
                "role": m.role,
            })).collect::<Vec<_>>(),
            "unread": unread,
            "last_sequence": last_sequence,
            "created_at": c.created_at,
        }));
    }

    // 按 created_at 倒序 (IM1.0 list_for_user: created_at DESC, id DESC)
    mine.sort_by(|a, b| {
        b.get("created_at")
            .and_then(|v| v.as_str())
            .cmp(&a.get("created_at").and_then(|v| v.as_str()))
    });

    Ok(Json(RestResponse::ok(json!({ "conversations": mine }))))
}

/// `POST /api/v1/im/conversations/{conversation_id}/messages` — 发消息 (legacy)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn send_message(
    State(state): State<Arc<ImApiState>>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    send_message_impl(&state, None, &user_id, &conversation_id, req).await
}

/// 发消息公共实现 (步骤 IM1.0 MessageService 5 步对齐)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
async fn send_message_impl(
    state: &Arc<ImApiState>,
    tenant_id: Option<&str>,
    user_id: &str,
    conversation_id: &str,
    req: SendMessageRequest,
) -> Result<Json<RestResponse<Value>>, RestError> {
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

    // 会话存在性 + 租户 + 成员校验 (对应 IM1.0 事务内校验)
    let conv = state
        .store
        .get_conversation(conversation_id)
        .await
        .map_err(|e| store_internal(&e))?
        .ok_or_else(|| conv_not_found(conversation_id))?;
    if let Some(t) = tenant_id {
        if conv.tenant_id != t {
            return Err(cross_tenant_conv(t, &conv.tenant_id));
        }
    }
    if !conv.members.iter().any(|m| m.user_id == user_id) {
        return Err(not_member(user_id, conversation_id));
    }

    // reply_to 存在性校验 (IM1.0 协议层校验, 此处最小实现前置)
    if let Some(reply_id) = &req.reply_to {
        let all = state
            .store
            .list_messages(conversation_id)
            .await
            .map_err(|e| store_internal(&e))?;
        if !all.iter().any(|m| m.id == *reply_id) {
            return Err(im_validation(
                format!("reply_to message not found: {reply_id}"),
                "reply_to must reference an existing message in the same conversation",
            ));
        }
    }

    // sequence 分配 (IM1.0 SequenceAllocator; 会话已在上面确认存在)
    let sequence = state
        .store
        .allocate_sequence(conversation_id)
        .await
        .map_err(|e| store_internal(&e))?
        .ok_or_else(|| conv_not_found(conversation_id))?;

    let msg = Message {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: conversation_id.to_string(),
        sequence,
        sender_id: user_id.to_string(),
        kind: req.kind,
        content: req.content,
        reply_to: req.reply_to,
        state: MessageState::Sent,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .store
        .append_message(msg.clone())
        .await
        .map_err(|e| store_internal(&e))?;

    // 网关 best-effort push (未接入 → NotConnected, 仅日志, 不假发送)
    if let Err(gw_err) = state.gateway.push_message(&msg).await {
        tracing::warn!(error = %gw_err, conversation_id = %conversation_id, "im gateway push skipped");
    }

    Ok(Json(RestResponse::ok(message_to_value(&msg))))
}

/// `GET /api/v1/im/conversations/{conversation_id}/messages` — 按 sequence 分页 (legacy)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_messages(
    State(state): State<Arc<ImApiState>>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    list_messages_impl(&state, None, &user_id, &conversation_id, query).await
}

/// 消息分页公共实现
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
async fn list_messages_impl(
    state: &Arc<ImApiState>,
    tenant_id: Option<&str>,
    user_id: &str,
    conversation_id: &str,
    query: ListMessagesQuery,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let after = query.after_sequence.unwrap_or(0);
    let limit = query.limit.unwrap_or(MAX_PAGE).clamp(1, MAX_PAGE);

    let conv = state
        .store
        .get_conversation(conversation_id)
        .await
        .map_err(|e| store_internal(&e))?
        .ok_or_else(|| conv_not_found(conversation_id))?;
    if let Some(t) = tenant_id {
        if conv.tenant_id != t {
            return Err(cross_tenant_conv(t, &conv.tenant_id));
        }
    }
    if !conv.members.iter().any(|m| m.user_id == user_id) {
        return Err(not_member(user_id, conversation_id));
    }

    let all = state
        .store
        .list_messages(conversation_id)
        .await
        .map_err(|e| store_internal(&e))?;
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

/// `POST /api/v1/im/conversations/{conversation_id}/read` — 标记已读 (legacy)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn mark_read(
    State(state): State<Arc<ImApiState>>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<MarkReadRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    mark_read_impl(&state, None, &user_id, &conversation_id, req).await
}

/// mark_read 公共实现 (IM1.0 夹紧 + 单调守卫全量对齐)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
async fn mark_read_impl(
    state: &Arc<ImApiState>,
    tenant_id: Option<&str>,
    user_id: &str,
    conversation_id: &str,
    req: MarkReadRequest,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if req.sequence < 0 {
        return Err(im_validation(
            "sequence must be >= 0",
            "Mark-read sequence cannot be negative",
        ));
    }

    let conv = state
        .store
        .get_conversation(conversation_id)
        .await
        .map_err(|e| store_internal(&e))?
        .ok_or_else(|| conv_not_found(conversation_id))?;
    if let Some(t) = tenant_id {
        if conv.tenant_id != t {
            return Err(cross_tenant_conv(t, &conv.tenant_id));
        }
    }
    let Some(my_last_read) = conv
        .members
        .iter()
        .find(|m| m.user_id == user_id)
        .map(|m| m.last_read_sequence)
    else {
        return Err(not_member(user_id, conversation_id));
    };
    // 已分配最大 sequence (IM1.0 `max_allocated_sequence` = 已分配水位)
    let max_allocated = conv.next_sequence;
    let clamped = req.sequence.min(max_allocated);
    let advanced = clamped > my_last_read;
    let effective = state
        .store
        .advance_member_read(conversation_id, user_id, clamped)
        .await
        .map_err(|e| store_internal(&e))?
        .ok_or_else(|| not_member(user_id, conversation_id))?;

    let all = state
        .store
        .list_messages(conversation_id)
        .await
        .map_err(|e| store_internal(&e))?;
    let unread = all
        .iter()
        .filter(|m| m.sequence > effective && m.sender_id != user_id)
        .count() as i64;

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
/// IM1.0 `crates/im-presence` 同为占位; 本实现无真实连接 (WS 网关未接入),
/// 恒返回空成员列表 + `connected:false` (诚实标注, 不假在线)。
pub async fn presence() -> Json<RestResponse<Value>> {
    Json(RestResponse::ok(json!({
        "connected": false,
        "members": [],
    })))
}

// ---------------------------------------------------------------------
// handlers (verified: 身份从已验证会话取, 租户 + 成员强制隔离)
// ---------------------------------------------------------------------

/// `POST /api/v1/im/conversations` (verified) — 会话租户 = actor 租户
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_conversation_verified(
    State(state): State<Arc<ImApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let tenant_id = actor.tenant_id.to_string();
    let user_id = actor.user_id.to_string();
    create_conversation_impl(&state, &tenant_id, &user_id, req).await
}

/// `GET /api/v1/im/conversations` (verified) — 只回 actor 租户的会话
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_conversations_verified(
    State(state): State<Arc<ImApiState>>,
    VerifiedActor(actor): VerifiedActor,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let tenant_id = actor.tenant_id.to_string();
    let user_id = actor.user_id.to_string();
    list_conversations_impl(&state, Some(&tenant_id), &user_id).await
}

/// `POST /api/v1/im/conversations/{id}/messages` (verified) — 成员 + 租户
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn send_message_verified(
    State(state): State<Arc<ImApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Path(conversation_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let tenant_id = actor.tenant_id.to_string();
    let user_id = actor.user_id.to_string();
    send_message_impl(&state, Some(&tenant_id), &user_id, &conversation_id, req).await
}

/// `GET /api/v1/im/conversations/{id}/messages` (verified) — 成员 + 租户
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_messages_verified(
    State(state): State<Arc<ImApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Path(conversation_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let tenant_id = actor.tenant_id.to_string();
    let user_id = actor.user_id.to_string();
    list_messages_impl(&state, Some(&tenant_id), &user_id, &conversation_id, query).await
}

/// `POST /api/v1/im/conversations/{id}/read` (verified) — 成员 + 租户
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn mark_read_verified(
    State(state): State<Arc<ImApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Path(conversation_id): Path<String>,
    Json(req): Json<MarkReadRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let tenant_id = actor.tenant_id.to_string();
    let user_id = actor.user_id.to_string();
    mark_read_impl(&state, Some(&tenant_id), &user_id, &conversation_id, req).await
}

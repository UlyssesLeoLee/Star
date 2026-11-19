/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/src/routes/notifications.rs"})
MERGE (state:Type {id:"crates/star-api-rest/src/routes/notifications.rs::NotificationsApiState",kind:"struct"})
MERGE (notif:Type {id:"crates/star-api-rest/src/routes/notifications.rs::Notification",kind:"struct"})
MERGE (comment:Type {id:"crates/star-api-rest/src/routes/notifications.rs::Comment",kind:"struct"})
MERGE (nkind:Type {id:"crates/star-api-rest/src/routes/notifications.rs::NotificationKind",kind:"enum"})
MERGE (ttype:Type {id:"crates/star-api-rest/src/routes/notifications.rs::TargetType",kind:"enum"})
MERGE (storeTrait:Type {id:"crates/star-api-rest/src/routes/notifications.rs::NotifyStore",kind:"trait"})
MERGE (inMemStore:Type {id:"crates/star-api-rest/src/routes/notifications.rs::InMemoryNotifyStore",kind:"struct"})
MERGE (pgStore:Type {id:"crates/star-api-rest/src/routes/notifications.rs::PgNotifyStore",kind:"struct"})
MERGE (relay:Type {id:"crates/star-api-rest/src/routes/notifications.rs::NotifyRelay",kind:"struct"})
MERGE (actor:Type {id:"crates/star-api-rest/src/routes/notifications.rs::VerifiedActor",kind:"struct"})
MERGE (listNotifs:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_notifications",kind:"function"})
MERGE (unread:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::unread_count",kind:"function"})
MERGE (createNotif:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_notification",kind:"function"})
MERGE (patchRead:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::patch_notification_read",kind:"function"})
MERGE (markRead:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::mark_notification_read",kind:"function"})
MERGE (listComments:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_comments",kind:"function"})
MERGE (createComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_comment",kind:"function"})
MERGE (deleteComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::delete_comment",kind:"function"})
MERGE (verifiedList:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_notifications_verified",kind:"function"})
MERGE (verifiedUnread:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::unread_count_verified",kind:"function"})
MERGE (verifiedCreate:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_notification_verified",kind:"function"})
MERGE (verifiedPatch:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::patch_notification_read_verified",kind:"function"})
MERGE (verifiedMark:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::mark_notification_read_verified",kind:"function"})
MERGE (verifiedListComments:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_comments_verified",kind:"function"})
MERGE (verifiedCreateComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_comment_verified",kind:"function"})
MERGE (verifiedDeleteComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::delete_comment_verified",kind:"function"})
MERGE (authMw:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::verify_jwt_and_inject_actor",kind:"function"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(notif)
MERGE (self)-[:DEFINES]->(comment)
MERGE (self)-[:DEFINES]->(nkind)
MERGE (self)-[:DEFINES]->(ttype)
MERGE (self)-[:DEFINES]->(storeTrait)
MERGE (self)-[:DEFINES]->(inMemStore)
MERGE (self)-[:DEFINES]->(pgStore)
MERGE (self)-[:DEFINES]->(relay)
MERGE (self)-[:DEFINES]->(actor)
MERGE (self)-[:DEFINES]->(listNotifs)
MERGE (self)-[:DEFINES]->(unread)
MERGE (self)-[:DEFINES]->(createNotif)
MERGE (self)-[:DEFINES]->(patchRead)
MERGE (self)-[:DEFINES]->(markRead)
MERGE (self)-[:DEFINES]->(listComments)
MERGE (self)-[:DEFINES]->(createComment)
MERGE (self)-[:DEFINES]->(deleteComment)
MERGE (self)-[:DEFINES]->(verifiedList)
MERGE (self)-[:DEFINES]->(verifiedUnread)
MERGE (self)-[:DEFINES]->(verifiedCreate)
MERGE (self)-[:DEFINES]->(verifiedPatch)
MERGE (self)-[:DEFINES]->(verifiedMark)
MERGE (self)-[:DEFINES]->(verifiedListComments)
MERGE (self)-[:DEFINES]->(verifiedCreateComment)
MERGE (self)-[:DEFINES]->(verifiedDeleteComment)
MERGE (self)-[:DEFINES]->(authMw)
MERGE (state)-[:HAS]->(notif)
MERGE (state)-[:HAS]->(comment)
MERGE (state)-[:USES_TYPE]->(storeTrait)
MERGE (inMemStore)-[:IMPLEMENTS]->(storeTrait)
MERGE (pgStore)-[:IMPLEMENTS]->(storeTrait)
MERGE (authMw)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::verify_token",kind:"function"})
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// notifications.rs — 通知 + 评论 owner API 端点
//
// 2026-11-20 P0 批次: 生产鉴权 fail-closed + 持久化 port + Outbox 幂等投递。
//
// 路径面 (与 2026-11-17 版完全一致, wire 契约不变):
//   GET    /api/v1/notifications              — 列表
//   GET    /api/v1/notifications/unread-count — 未读计数
//   POST   /api/v1/notifications              — 创建
//   PATCH  /api/v1/notifications/{id}         — 更新 read 状态
//   POST   /api/v1/notifications/{id}/read    — 标记已读 (幂等)
//   GET    /api/v1/comments                   — 评论列表
//   POST   /api/v1/comments                   — 新增评论
//   DELETE /api/v1/comments/{id}              — 删除评论 (仅作者)
//
// 本批次变更:
// 1. 生产鉴权 fail-closed:
//    - `router()` / `comments_router()` = verified 路由 + JWT 验签 middleware
//      (`verify_jwt_and_inject_actor`, 复用 `auth::verify_token` + sub/tenant
//      claim 一致性检查, 与 group/SCM 域同一套 JwtConfig); 无/无效 token → 401。
//    - verified handler 从已验证会话 (request extension 注入的 `AuthUser`)
//      取身份 (`VerifiedActor` extractor), 缺失即 401 — 不读 X-User-Id 请求头。
//    - 租户隔离: 通知/评论读写按 actor.tenant_id 强制过滤; 请求体/query 携带
//      的 tenant_id 与 actor 不符 → 403 POLICY_DENIED。
//    - 收件人隔离: Notification 新增 `recipient_user_id` 维度; verified 列表/
//      未读计数只回 actor 自己的 (recipient == actor) 或租户广播
//      (recipient IS NULL)。
// 2. 持久化: 全部读写走 `NotifyStore` port trait (通知/评论/outbox)。
//    - `InMemoryNotifyStore`: 测试/无 DB 环境 (Arc 共享, 可跨 router 重建)。
//    - `PgNotifyStore`: sqlx PostgreSQL 真实实现 (DDL 见 `NOTIFY_PG_DDL`,
//      表 p0_notifications / p0_comments / p0_notify_outbox, 幂等键唯一约束)。
//    - 生产 `router()` 读 DATABASE_URL: 有 → PG store (重启可恢复);
//      无 → InMemory + tracing::warn (诚实降级, 不静默)。
// 3. Outbox 幂等投递: verified 创建走 outbox 事件 (notification.created /
//    comment.created) → `NotifyRelay` 投递 → 幂等 apply (idempotency_key
//    = event_id, 唯一约束); 重试上限 5 (INV-N-06) + 复用
//    `crate::device_outbox::relay::RelayBackoffPolicy` 有界指数退避;
//    重复投递不产生重复通知。
//
// 已知简化 (诚实标注, per 守门 #5):
// - 收件人 membership 校验: recipient_user_id 只做同租户字符串存储, 不做
//   用户目录存在性核查 (本 lane 无 user directory 依赖) — 父代理待办。
// - device-binding 代际强校验 (`AuthenticatedUser` extractor 全套) 需要 PG
//   pool 注入, 生产 `router()` 无参构造暂用 JWT 验签 + claim 一致性;
//   父代理挂载时可换 `AuthenticatedUser` extractor (与 group 域同套)。
// - 评论 → 通知联动: outbox apply 支持 comment.created 事件的
//   notify_recipients 扇出 (每个收件人一条 Comment 通知, 幂等); REST
//   create_comment 尚无 watcher/mention 解析来源, 默认不填收件人。
//
// 注册: parent 统一注册 (routes/mod.rs `pub mod notifications;` + lib.rs
// `.nest("/notifications", routes::notifications::router())` +
// `.nest("/comments", routes::notifications::comments_router())`) — 本文件
// 不自行注册。本文件以 `crate::` 路径引用 error/response 模块。
// =====================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{header, request::Parts, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::oauth::middleware::BearerError;
use crate::auth::{verify_token, AuthUser, JwtConfig, JwtError};
use crate::error::RestError;
use crate::response::RestResponse;

/// 通知列表默认单页条数 (任务 spec 未定, 取 20)
const DEFAULT_LIMIT: i64 = 20;
/// 通知列表单页上限 (per routes/im.rs MAX_PAGE 同量级约定)
const MAX_LIMIT: i64 = 200;
/// Outbox 投递重试上限 (INV-N-06: 最多 5 次, 超限进 DLQ/dead)
pub const OUTBOX_MAX_RETRY: u32 = 5;
/// Outbox 单次 drain 处理的事件数上限
pub const OUTBOX_DRAIN_BATCH: usize = 100;

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

/// 通知 (任务 spec notification 逐字段 + P0 收件人维度)
///
/// `recipient_user_id`: `Some(u)` = 个人通知 (仅 u 可见); `None` = 租户广播
/// (租户内全员可见, 由系统级生产者写入, per P0 「收件人隔离」验收)。
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// 收件人用户 id (None = 租户广播; per P0 收件人隔离)
    pub recipient_user_id: Option<String>,
    /// 是否已读
    pub read: bool,
    /// 创建时间 RFC 3339
    pub created_at: String,
}

/// 评论 (任务 spec comment 逐字段: id/tenant_id/target_type/target_id/
/// author_id/author_name/body/created_at)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// 评论 id (UUID v4)
    pub id: String,
    /// 所属租户 id
    pub tenant_id: String,
    /// 目标类型 (自由字符串; 通知的 TargetType 枚举仅约束 notification)
    pub target_type: String,
    /// 目标 id
    pub target_id: String,
    /// 作者用户 id (创建时取已验证会话)
    pub author_id: String,
    /// 作者显示名 (缺省回落为 author_id)
    pub author_name: String,
    /// 评论正文
    pub body: String,
    /// 创建时间 RFC 3339
    pub created_at: String,
}

// ---------------------------------------------------------------------
// Outbox 事件 + 投递结果
// ---------------------------------------------------------------------

/// Outbox 事件 (先落 outbox 表/结构, relay 异步投递; 投递语义 at-least-once)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    /// 事件 id (幂等键)
    pub event_id: String,
    /// 事件类型 (notification.created | comment.created)
    pub event_type: String,
    /// 所属租户 id
    pub tenant_id: String,
    /// 事件负载 JSON (通知/评论实体 + 可选 notify_recipients 扇出)
    pub payload: Value,
    /// 创建时间 RFC 3339
    pub created_at: String,
    /// 已投递尝试次数
    pub attempts: u32,
    /// 下次可投递时间 (epoch millis; 失败后按退避推进)
    pub next_attempt_at_ms: i64,
    /// 是否已投递成功
    pub delivered: bool,
    /// 是否已死信 (重试超限/毒事件, 不再重试)
    pub dead: bool,
}

/// 幂等插入结果 (重复 insert 冲突时返回既有记录, 不产生第二份数据)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    /// 新插入
    Inserted,
    /// 幂等键冲突, 既有记录保留 (重复投递/重放)
    Duplicate,
}

/// 单事件 drain 结果 (供测试断言 + 观测)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayOutcome {
    /// 已应用 (新数据落地)
    Applied { event_id: String },
    /// 幂等重放 (既有记录返回, 无新数据)
    Duplicate { event_id: String },
    /// 瞬时失败, 已记录尝试并退避 (待下次 drain)
    Retried { event_id: String, attempts: u32 },
    /// 毒事件/重试超限, 已死信 (不再重试)
    DeadLettered { event_id: String },
}

// ---------------------------------------------------------------------
// 存储 port trait (通知 + 评论 + outbox; 内存/PG 同一 trait)
// ---------------------------------------------------------------------

/// 存储层错误 (port 边界错误)
#[derive(Debug, thiserror::Error)]
pub enum NotifyStoreError {
    /// 瞬时不可用 (relay 按退避重试)
    #[error("notify store unavailable: {0}")]
    Unavailable(String),
    /// 毒事件 (未知事件类型等, 直接死信不重试)
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    /// 序列化/反序列化失败
    #[error("serialization: {0}")]
    Serialization(String),
}

/// 通知/评论/outbox 持久化端口。
///
/// 内存实现 (`InMemoryNotifyStore`) 与 PostgreSQL 实现 (`PgNotifyStore`)
/// 共享本 trait; handler 层不感知具体存储, 数据不落在 handler 内存里
/// (「重启可恢复」由 PG 实现保证, 见 `#[ignore]` 验收测试)。
#[async_trait]
pub trait NotifyStore: Send + Sync {
    /// 插入通知; `idempotency_key` 唯一 (重复 → Duplicate, 不产生第二份)
    async fn insert_notification(
        &self,
        n: Notification,
        idempotency_key: Option<&str>,
    ) -> Result<InsertOutcome, NotifyStoreError>;
    /// 按 id 取通知 (不存在 → Ok(None))
    async fn get_notification(&self, id: &str) -> Result<Option<Notification>, NotifyStoreError>;
    /// 设置已读状态; 不存在 → Ok(None), 存在 → 回更新后的通知
    async fn set_notification_read(
        &self,
        id: &str,
        read: bool,
    ) -> Result<Option<Notification>, NotifyStoreError>;
    /// 列表: tenant/recipient/kind/unread_only 过滤 (None = 不过滤该维度)
    async fn list_notifications(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
        kind: Option<NotificationKind>,
        unread_only: bool,
    ) -> Result<Vec<Notification>, NotifyStoreError>;
    /// 未读计数 (同列表的过滤维度; 服务端单一真源)
    async fn unread_count(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
    ) -> Result<i64, NotifyStoreError>;
    /// 插入评论 (幂等键同上)
    async fn insert_comment(
        &self,
        c: Comment,
        idempotency_key: Option<&str>,
    ) -> Result<InsertOutcome, NotifyStoreError>;
    /// 按 id 取评论
    async fn get_comment(&self, id: &str) -> Result<Option<Comment>, NotifyStoreError>;
    /// 物理删除评论; 返回是否删除过
    async fn remove_comment(&self, id: &str) -> Result<bool, NotifyStoreError>;
    /// 评论列表 (tenant/target_type/target_id 过滤; None = 不过滤)
    async fn list_comments(
        &self,
        tenant_id: Option<&str>,
        target_type: Option<&str>,
        target_id: Option<&str>,
    ) -> Result<Vec<Comment>, NotifyStoreError>;
    /// 入队 outbox 事件 (同 event_id 重复入队被忽略, 保留首条)
    async fn enqueue_outbox(&self, ev: OutboxEvent) -> Result<(), NotifyStoreError>;
    /// 取到期未投递事件 (next_attempt_at_ms <= now_ms, 未 delivered 未 dead)
    async fn list_due_outbox(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> Result<Vec<OutboxEvent>, NotifyStoreError>;
    /// 标记投递成功
    async fn mark_outbox_delivered(&self, event_id: &str) -> Result<(), NotifyStoreError>;
    /// 记录失败尝试: attempts / next_attempt_at_ms / dead
    async fn mark_outbox_attempt(
        &self,
        event_id: &str,
        attempts: u32,
        next_attempt_at_ms: i64,
        dead: bool,
    ) -> Result<(), NotifyStoreError>;
}

// ---------------------------------------------------------------------
// InMemoryNotifyStore (测试/无 DB 环境; Arc 共享可跨 router 重建)
// ---------------------------------------------------------------------

/// 内存实现 (port trait 双实现之一; 数据在 store 而非 handler)
pub struct InMemoryNotifyStore {
    notifications: Mutex<HashMap<String, Notification>>,
    comments: Mutex<HashMap<String, Comment>>,
    outbox: Mutex<HashMap<String, OutboxEvent>>,
    idem_keys: Mutex<HashMap<String, String>>,
}

impl InMemoryNotifyStore {
    /// 新建空 store
    pub fn new() -> Self {
        Self {
            notifications: Mutex::new(HashMap::new()),
            comments: Mutex::new(HashMap::new()),
            outbox: Mutex::new(HashMap::new()),
            idem_keys: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryNotifyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotifyStore for InMemoryNotifyStore {
    async fn insert_notification(
        &self,
        n: Notification,
        idempotency_key: Option<&str>,
    ) -> Result<InsertOutcome, NotifyStoreError> {
        if let Some(key) = idempotency_key {
            let mut keys = self.idem_keys.lock().expect("notify idem poisoned");
            if keys.contains_key(key) {
                return Ok(InsertOutcome::Duplicate);
            }
            keys.insert(key.to_string(), n.id.clone());
        }
        self.notifications
            .lock()
            .expect("notifications poisoned")
            .insert(n.id.clone(), n);
        Ok(InsertOutcome::Inserted)
    }

    async fn get_notification(&self, id: &str) -> Result<Option<Notification>, NotifyStoreError> {
        Ok(self
            .notifications
            .lock()
            .expect("notifications poisoned")
            .get(id)
            .cloned())
    }

    async fn set_notification_read(
        &self,
        id: &str,
        read: bool,
    ) -> Result<Option<Notification>, NotifyStoreError> {
        let mut store = self.notifications.lock().expect("notifications poisoned");
        let Some(n) = store.get_mut(id) else {
            return Ok(None);
        };
        n.read = read;
        Ok(Some(n.clone()))
    }

    async fn list_notifications(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
        kind: Option<NotificationKind>,
        unread_only: bool,
    ) -> Result<Vec<Notification>, NotifyStoreError> {
        Ok(self
            .notifications
            .lock()
            .expect("notifications poisoned")
            .values()
            .filter(|n| tenant_id.is_none_or(|t| n.tenant_id == t))
            .filter(|n| {
                recipient_user_id.is_none_or(|r| {
                    n.recipient_user_id.as_deref() == Some(r) || n.recipient_user_id.is_none()
                })
            })
            .filter(|n| kind.is_none_or(|k| n.kind == k))
            .filter(|n| !unread_only || !n.read)
            .cloned()
            .collect())
    }

    async fn unread_count(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
    ) -> Result<i64, NotifyStoreError> {
        let store = self.notifications.lock().expect("notifications poisoned");
        Ok(store
            .values()
            .filter(|n| !n.read)
            .filter(|n| tenant_id.is_none_or(|t| n.tenant_id == t))
            .filter(|n| {
                recipient_user_id.is_none_or(|r| {
                    n.recipient_user_id.as_deref() == Some(r) || n.recipient_user_id.is_none()
                })
            })
            .count() as i64)
    }

    async fn insert_comment(
        &self,
        c: Comment,
        idempotency_key: Option<&str>,
    ) -> Result<InsertOutcome, NotifyStoreError> {
        if let Some(key) = idempotency_key {
            let mut keys = self.idem_keys.lock().expect("notify idem poisoned");
            if keys.contains_key(key) {
                return Ok(InsertOutcome::Duplicate);
            }
            keys.insert(key.to_string(), c.id.clone());
        }
        self.comments
            .lock()
            .expect("comments poisoned")
            .insert(c.id.clone(), c);
        Ok(InsertOutcome::Inserted)
    }

    async fn get_comment(&self, id: &str) -> Result<Option<Comment>, NotifyStoreError> {
        Ok(self
            .comments
            .lock()
            .expect("comments poisoned")
            .get(id)
            .cloned())
    }

    async fn remove_comment(&self, id: &str) -> Result<bool, NotifyStoreError> {
        Ok(self
            .comments
            .lock()
            .expect("comments poisoned")
            .remove(id)
            .is_some())
    }

    async fn list_comments(
        &self,
        tenant_id: Option<&str>,
        target_type: Option<&str>,
        target_id: Option<&str>,
    ) -> Result<Vec<Comment>, NotifyStoreError> {
        Ok(self
            .comments
            .lock()
            .expect("comments poisoned")
            .values()
            .filter(|c| tenant_id.is_none_or(|t| c.tenant_id == t))
            .filter(|c| target_type.is_none_or(|t| c.target_type == t))
            .filter(|c| target_id.is_none_or(|t| c.target_id == t))
            .cloned()
            .collect())
    }

    async fn enqueue_outbox(&self, ev: OutboxEvent) -> Result<(), NotifyStoreError> {
        self.outbox
            .lock()
            .expect("outbox poisoned")
            .entry(ev.event_id.clone())
            .or_insert(ev);
        Ok(())
    }

    async fn list_due_outbox(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> Result<Vec<OutboxEvent>, NotifyStoreError> {
        Ok(self
            .outbox
            .lock()
            .expect("outbox poisoned")
            .values()
            .filter(|ev| !ev.delivered && !ev.dead && ev.next_attempt_at_ms <= now_ms)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn mark_outbox_delivered(&self, event_id: &str) -> Result<(), NotifyStoreError> {
        if let Some(ev) = self
            .outbox
            .lock()
            .expect("outbox poisoned")
            .get_mut(event_id)
        {
            ev.delivered = true;
        }
        Ok(())
    }

    async fn mark_outbox_attempt(
        &self,
        event_id: &str,
        attempts: u32,
        next_attempt_at_ms: i64,
        dead: bool,
    ) -> Result<(), NotifyStoreError> {
        if let Some(ev) = self
            .outbox
            .lock()
            .expect("outbox poisoned")
            .get_mut(event_id)
        {
            ev.attempts = attempts;
            ev.next_attempt_at_ms = next_attempt_at_ms;
            ev.dead = dead;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// PgNotifyStore (sqlx PostgreSQL; 真实持久化实现, 验收测试标 #[ignore])
// ---------------------------------------------------------------------

/// PostgreSQL 通知/评论/outbox DDL (每行一条语句, 与仓库 migration runner
/// 一行一语句纪律一致; 父代理合并时落地到 db/migrations/ 正式迁移文件)。
pub const NOTIFY_PG_DDL: &str = "\
CREATE TABLE IF NOT EXISTS p0_notifications (id TEXT PRIMARY KEY, tenant_id TEXT NOT NULL, kind TEXT NOT NULL, subject TEXT NOT NULL, body TEXT NOT NULL, subject_url TEXT, actor_id TEXT, actor_name TEXT, target_type TEXT, target_id TEXT, recipient_user_id TEXT, read BOOLEAN NOT NULL DEFAULT FALSE, created_at TEXT NOT NULL, idempotency_key TEXT)
CREATE UNIQUE INDEX IF NOT EXISTS p0_notifications_idem_key_uniq ON p0_notifications (idempotency_key) WHERE idempotency_key IS NOT NULL
CREATE TABLE IF NOT EXISTS p0_comments (id TEXT PRIMARY KEY, tenant_id TEXT NOT NULL, target_type TEXT NOT NULL, target_id TEXT NOT NULL, author_id TEXT NOT NULL, author_name TEXT NOT NULL, body TEXT NOT NULL, created_at TEXT NOT NULL, idempotency_key TEXT)
CREATE UNIQUE INDEX IF NOT EXISTS p0_comments_idem_key_uniq ON p0_comments (idempotency_key) WHERE idempotency_key IS NOT NULL
CREATE TABLE IF NOT EXISTS p0_notify_outbox (event_id TEXT PRIMARY KEY, event_type TEXT NOT NULL, tenant_id TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0, next_attempt_at_ms BIGINT NOT NULL, delivered BOOLEAN NOT NULL DEFAULT FALSE, dead BOOLEAN NOT NULL DEFAULT FALSE)";

/// PostgreSQL 实现 (`NotifyStore` port 的真实持久化适配; 需要真实 PG,
/// 验收测试 `#[ignore]` + `STAR_P0_NOTIFY_TEST_DATABASE_URL` 跑法)。
pub struct PgNotifyStore {
    pool: sqlx::PgPool,
}

impl PgNotifyStore {
    /// 以既有 pool 构造 (生产由父代理注入; 本 lane 无参 router() 用
    /// DATABASE_URL 惰性构造)
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取底层 pool (验收测试建表用)
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[async_trait]
impl NotifyStore for PgNotifyStore {
    async fn insert_notification(
        &self,
        n: Notification,
        idempotency_key: Option<&str>,
    ) -> Result<InsertOutcome, NotifyStoreError> {
        let conflict = match idempotency_key {
            Some(_) => {
                " ON CONFLICT (idempotency_key) WHERE idempotency_key IS NOT NULL DO NOTHING"
            }
            None => " ON CONFLICT (id) DO NOTHING",
        };
        let sql = format!(
            "INSERT INTO p0_notifications (id, tenant_id, kind, subject, body, subject_url, actor_id, actor_name, target_type, target_id, recipient_user_id, read, created_at, idempotency_key) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14){conflict}"
        );
        let result = sqlx::query(&sql)
            .bind(&n.id)
            .bind(&n.tenant_id)
            .bind(serde_kind_str(&n.kind))
            .bind(&n.subject)
            .bind(&n.body)
            .bind(&n.subject_url)
            .bind(&n.actor_id)
            .bind(&n.actor_name)
            .bind(&n.target_type)
            .bind(&n.target_id)
            .bind(&n.recipient_user_id)
            .bind(n.read)
            .bind(&n.created_at)
            .bind(idempotency_key.map(|s| s.to_string()))
            .execute(&self.pool)
            .await
            .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        if result.rows_affected() == 0 && idempotency_key.is_some() {
            return Ok(InsertOutcome::Duplicate);
        }
        Ok(InsertOutcome::Inserted)
    }

    async fn get_notification(&self, id: &str) -> Result<Option<Notification>, NotifyStoreError> {
        let row =
            sqlx::query_as::<_, NotificationRow>("SELECT * FROM p0_notifications WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(row.map(NotificationRow::into_notification))
    }

    async fn set_notification_read(
        &self,
        id: &str,
        read: bool,
    ) -> Result<Option<Notification>, NotifyStoreError> {
        let row = sqlx::query_as::<_, NotificationRow>(
            "UPDATE p0_notifications SET read = $2 WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(read)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(row.map(NotificationRow::into_notification))
    }

    async fn list_notifications(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
        kind: Option<NotificationKind>,
        unread_only: bool,
    ) -> Result<Vec<Notification>, NotifyStoreError> {
        // 维度固定由 handler 传参, SQL 动态拼 tenant/recipient/kind 过滤
        // (全部 ? 值绑定, 无注入面)
        let mut sql = String::from(
            "SELECT * FROM p0_notifications WHERE TRUE \
             AND ($1::text IS NULL OR tenant_id = $1) \
             AND ($2::text IS NULL OR recipient_user_id = $2 OR ($2::text IS NOT NULL AND recipient_user_id IS NULL)) \
             AND ($3::text IS NULL OR kind = $3)",
        );
        if unread_only {
            sql.push_str(" AND read = FALSE");
        }
        sql.push_str(" ORDER BY created_at DESC, id DESC");
        let rows = sqlx::query_as::<_, NotificationRow>(&sql)
            .bind(tenant_id.map(|s| s.to_string()))
            .bind(recipient_user_id.map(|s| s.to_string()))
            .bind(kind.map(|k| serde_kind_str(&k).to_string()))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(NotificationRow::into_notification)
            .collect())
    }

    async fn unread_count(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
    ) -> Result<i64, NotifyStoreError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM p0_notifications WHERE read = FALSE \
             AND ($1::text IS NULL OR tenant_id = $1) \
             AND ($2::text IS NULL OR recipient_user_id = $2 OR ($2::text IS NOT NULL AND recipient_user_id IS NULL))",
        )
        .bind(tenant_id.map(|s| s.to_string()))
        .bind(recipient_user_id.map(|s| s.to_string()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(row.0)
    }

    async fn insert_comment(
        &self,
        c: Comment,
        idempotency_key: Option<&str>,
    ) -> Result<InsertOutcome, NotifyStoreError> {
        let conflict = match idempotency_key {
            Some(_) => {
                " ON CONFLICT (idempotency_key) WHERE idempotency_key IS NOT NULL DO NOTHING"
            }
            None => " ON CONFLICT (id) DO NOTHING",
        };
        let sql = format!(
            "INSERT INTO p0_comments (id, tenant_id, target_type, target_id, author_id, author_name, body, created_at, idempotency_key) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9){conflict}"
        );
        let result = sqlx::query(&sql)
            .bind(&c.id)
            .bind(&c.tenant_id)
            .bind(&c.target_type)
            .bind(&c.target_id)
            .bind(&c.author_id)
            .bind(&c.author_name)
            .bind(&c.body)
            .bind(&c.created_at)
            .bind(idempotency_key.map(|s| s.to_string()))
            .execute(&self.pool)
            .await
            .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        if result.rows_affected() == 0 && idempotency_key.is_some() {
            return Ok(InsertOutcome::Duplicate);
        }
        Ok(InsertOutcome::Inserted)
    }

    async fn get_comment(&self, id: &str) -> Result<Option<Comment>, NotifyStoreError> {
        let row = sqlx::query_as::<_, CommentRow>("SELECT * FROM p0_comments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(row.map(CommentRow::into_comment))
    }

    async fn remove_comment(&self, id: &str) -> Result<bool, NotifyStoreError> {
        let result = sqlx::query("DELETE FROM p0_comments WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_comments(
        &self,
        tenant_id: Option<&str>,
        target_type: Option<&str>,
        target_id: Option<&str>,
    ) -> Result<Vec<Comment>, NotifyStoreError> {
        let rows = sqlx::query_as::<_, CommentRow>(
            "SELECT * FROM p0_comments WHERE TRUE \
             AND ($1::text IS NULL OR tenant_id = $1) \
             AND ($2::text IS NULL OR target_type = $2) \
             AND ($3::text IS NULL OR target_id = $3) \
             ORDER BY created_at ASC, id ASC",
        )
        .bind(tenant_id.map(|s| s.to_string()))
        .bind(target_type.map(|s| s.to_string()))
        .bind(target_id.map(|s| s.to_string()))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(rows.into_iter().map(CommentRow::into_comment).collect())
    }

    async fn enqueue_outbox(&self, ev: OutboxEvent) -> Result<(), NotifyStoreError> {
        let payload = serde_json::to_string(&ev.payload)
            .map_err(|e| NotifyStoreError::Serialization(e.to_string()))?;
        // 同 event_id 重复入队忽略 (幂等投递第一道防线)
        sqlx::query(
            "INSERT INTO p0_notify_outbox (event_id, event_type, tenant_id, payload, created_at, attempts, next_attempt_at_ms, delivered, dead) \
             VALUES ($1,$2,$3,$4,$5,0,$6,FALSE,FALSE) ON CONFLICT (event_id) DO NOTHING",
        )
        .bind(&ev.event_id)
        .bind(&ev.event_type)
        .bind(&ev.tenant_id)
        .bind(&payload)
        .bind(&ev.created_at)
        .bind(ev.next_attempt_at_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(())
    }

    async fn list_due_outbox(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> Result<Vec<OutboxEvent>, NotifyStoreError> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            "SELECT * FROM p0_notify_outbox \
             WHERE delivered = FALSE AND dead = FALSE AND next_attempt_at_ms <= $1 \
             ORDER BY next_attempt_at_ms ASC, event_id ASC LIMIT $2",
        )
        .bind(now_ms)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        rows.into_iter()
            .map(OutboxRow::into_event)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn mark_outbox_delivered(&self, event_id: &str) -> Result<(), NotifyStoreError> {
        sqlx::query("UPDATE p0_notify_outbox SET delivered = TRUE WHERE event_id = $1")
            .bind(event_id)
            .execute(&self.pool)
            .await
            .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(())
    }

    async fn mark_outbox_attempt(
        &self,
        event_id: &str,
        attempts: u32,
        next_attempt_at_ms: i64,
        dead: bool,
    ) -> Result<(), NotifyStoreError> {
        sqlx::query(
            "UPDATE p0_notify_outbox SET attempts = $2, next_attempt_at_ms = $3, dead = $4 WHERE event_id = $1",
        )
        .bind(event_id)
        .bind(attempts as i32)
        .bind(next_attempt_at_ms)
        .bind(dead)
        .execute(&self.pool)
        .await
        .map_err(|e| NotifyStoreError::Unavailable(e.to_string()))?;
        Ok(())
    }
}

/// PG 行 → Notification (sqlx FromRow 需要列名与字段一一对应)
#[derive(Debug, sqlx::FromRow)]
struct NotificationRow {
    id: String,
    tenant_id: String,
    kind: String,
    subject: String,
    body: String,
    subject_url: Option<String>,
    actor_id: Option<String>,
    actor_name: Option<String>,
    target_type: Option<String>,
    target_id: Option<String>,
    recipient_user_id: Option<String>,
    #[allow(dead_code)] // read 由 into_notification 读取
    read: bool,
    created_at: String,
    #[allow(dead_code)] // idempotency_key 仅存储层使用
    idempotency_key: Option<String>,
}

impl NotificationRow {
    /// Row → 领域 Notification (kind 反序列化失败回退 System + 保留原串)
    fn into_notification(self) -> Notification {
        let kind = parse_notification_kind_loose(&self.kind);
        Notification {
            id: self.id,
            tenant_id: self.tenant_id,
            kind,
            subject: self.subject,
            body: self.body,
            subject_url: self.subject_url,
            actor_id: self.actor_id,
            actor_name: self.actor_name,
            target_type: self.target_type,
            target_id: self.target_id,
            recipient_user_id: self.recipient_user_id,
            read: self.read,
            created_at: self.created_at,
        }
    }
}

/// PG 行 → Comment
#[derive(Debug, sqlx::FromRow)]
struct CommentRow {
    id: String,
    tenant_id: String,
    target_type: String,
    target_id: String,
    author_id: String,
    author_name: String,
    body: String,
    created_at: String,
    #[allow(dead_code)] // idempotency_key 仅存储层使用
    idempotency_key: Option<String>,
}

impl CommentRow {
    /// Row → 领域 Comment
    fn into_comment(self) -> Comment {
        Comment {
            id: self.id,
            tenant_id: self.tenant_id,
            target_type: self.target_type,
            target_id: self.target_id,
            author_id: self.author_id,
            author_name: self.author_name,
            body: self.body,
            created_at: self.created_at,
        }
    }
}

/// PG 行 → OutboxEvent
#[derive(Debug, sqlx::FromRow)]
struct OutboxRow {
    event_id: String,
    event_type: String,
    tenant_id: String,
    payload: String,
    created_at: String,
    attempts: i32,
    next_attempt_at_ms: i64,
    delivered: bool,
    dead: bool,
}

impl OutboxRow {
    /// Row → 领域 OutboxEvent
    fn into_event(self) -> Result<OutboxEvent, NotifyStoreError> {
        let payload = serde_json::from_str(&self.payload)
            .map_err(|e| NotifyStoreError::Serialization(e.to_string()))?;
        Ok(OutboxEvent {
            event_id: self.event_id,
            event_type: self.event_type,
            tenant_id: self.tenant_id,
            payload,
            created_at: self.created_at,
            attempts: self.attempts.max(0) as u32,
            next_attempt_at_ms: self.next_attempt_at_ms,
            delivered: self.delivered,
            dead: self.dead,
        })
    }
}

// ---------------------------------------------------------------------
// NotifyRelay (outbox 投递: 有界退避 + 重试上限 + 幂等 apply)
// ---------------------------------------------------------------------

/// Outbox relay: 投递语义 at-least-once; 幂等 apply 保证「重复投递不产生
/// 重复通知」。退避复用 `crate::device_outbox::relay::RelayBackoffPolicy`
/// (base 1s / max 30s 有界指数退避); 重试上限 OUTBOX_MAX_RETRY (INV-N-06)。
#[derive(Debug, Clone, Copy, Default)]
pub struct NotifyRelay;

impl NotifyRelay {
    /// 新建 relay
    pub fn new() -> Self {
        Self
    }

    /// 单轮 drain: 取到期事件逐条 apply。瞬时失败记尝试+退避, 毒事件/超限死信。
    pub async fn drain(
        &self,
        store: &Arc<dyn NotifyStore>,
        now_ms: i64,
    ) -> Result<Vec<RelayOutcome>, NotifyStoreError> {
        let due = store.list_due_outbox(now_ms, OUTBOX_DRAIN_BATCH).await?;
        let mut outcomes = Vec::with_capacity(due.len());
        for ev in due {
            match self.apply_event(store, &ev).await {
                Ok(InsertOutcome::Inserted) => {
                    store.mark_outbox_delivered(&ev.event_id).await?;
                    outcomes.push(RelayOutcome::Applied {
                        event_id: ev.event_id,
                    });
                }
                Ok(InsertOutcome::Duplicate) => {
                    // 幂等重放: 既有记录保留, 无新数据, 仍 ack 投递
                    store.mark_outbox_delivered(&ev.event_id).await?;
                    outcomes.push(RelayOutcome::Duplicate {
                        event_id: ev.event_id,
                    });
                }
                Err(NotifyStoreError::InvalidEvent(_)) => {
                    store
                        .mark_outbox_attempt(
                            &ev.event_id,
                            ev.attempts.saturating_add(1),
                            now_ms,
                            true,
                        )
                        .await?;
                    outcomes.push(RelayOutcome::DeadLettered {
                        event_id: ev.event_id,
                    });
                }
                Err(_transient) => {
                    let attempts = ev.attempts.saturating_add(1);
                    let dead = attempts >= OUTBOX_MAX_RETRY;
                    let delay_ms = crate::device_outbox::relay::RelayBackoffPolicy::default()
                        .delay_for(attempts)
                        .as_millis() as i64;
                    store
                        .mark_outbox_attempt(&ev.event_id, attempts, now_ms + delay_ms, dead)
                        .await?;
                    if dead {
                        outcomes.push(RelayOutcome::DeadLettered {
                            event_id: ev.event_id,
                        });
                    } else {
                        outcomes.push(RelayOutcome::Retried {
                            event_id: ev.event_id,
                            attempts,
                        });
                    }
                }
            }
        }
        Ok(outcomes)
    }

    /// 单事件幂等 apply (idempotency_key = event_id)。
    ///
    /// 事件类型:
    /// - `notification.created`: payload = 通知实体 → 幂等插入。
    /// - `comment.created`: payload = 评论实体 + 可选 `notify_recipients`
    ///   (字符串数组) → 幂等插入评论; 每个收件人幂等派生一条 Comment 通知
    ///   (idempotency_key = `{event_id}:notify:{recipient}`, 联动扇出)。
    /// - 其它类型 → InvalidEvent (fail-closed 死信, 不静默跳过)。
    pub async fn apply_event(
        &self,
        store: &Arc<dyn NotifyStore>,
        ev: &OutboxEvent,
    ) -> Result<InsertOutcome, NotifyStoreError> {
        match ev.event_type.as_str() {
            "notification.created" => {
                let n: Notification = serde_json::from_value(ev.payload.clone())
                    .map_err(|e| NotifyStoreError::Serialization(e.to_string()))?;
                store.insert_notification(n, Some(&ev.event_id)).await
            }
            "comment.created" => {
                let mut payload = ev.payload.clone();
                let c: Comment = serde_json::from_value(payload.clone())
                    .map_err(|e| NotifyStoreError::Serialization(e.to_string()))?;
                let recipients: Vec<String> = payload
                    .as_object_mut()
                    .and_then(|obj| obj.remove("notify_recipients"))
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default();
                let outcome = store.insert_comment(c.clone(), Some(&ev.event_id)).await?;
                // 评论 → 通知联动: 每个收件人派生一条 Comment 通知 (幂等键
                // 含 event_id + recipient, 重放不产生重复通知)
                for recipient in recipients {
                    let n = Notification {
                        id: uuid::Uuid::new_v4().to_string(),
                        tenant_id: c.tenant_id.clone(),
                        kind: NotificationKind::Comment,
                        subject: "新评论".to_string(),
                        body: c.body.clone(),
                        subject_url: None,
                        actor_id: Some(c.author_id.clone()),
                        actor_name: Some(c.author_name.clone()),
                        target_type: Some(c.target_type.clone()),
                        target_id: Some(c.target_id.clone()),
                        recipient_user_id: Some(recipient.clone()),
                        read: false,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    store
                        .insert_notification(
                            n,
                            Some(&format!("{}:notify:{}", ev.event_id, recipient)),
                        )
                        .await?;
                }
                Ok(outcome)
            }
            other => Err(NotifyStoreError::InvalidEvent(format!(
                "unknown outbox event type: {other}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------
// 共享状态 (存储 port 注入; 生产/测试各自构造)
// ---------------------------------------------------------------------

/// 通知 + 评论端点共享状态。
///
/// 唯一字段是存储 port: 数据不落在 handler 内存 (重启可恢复由 PG 实现
/// 保证), legacy/verified 两套 handler 共用同一 store (服务端单一真源)。
pub struct NotificationsApiState {
    /// 通知/评论/outbox 存储
    pub store: Arc<dyn NotifyStore>,
}

impl NotificationsApiState {
    /// 以注入 store 新建状态
    pub fn from_store(store: Arc<dyn NotifyStore>) -> Self {
        Self { store }
    }

    /// 新建状态 (内存 store; 测试隔离用)
    pub fn new() -> Self {
        Self::from_store(Arc::new(InMemoryNotifyStore::new()))
    }
}

impl Default for NotificationsApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试/多实例隔离用: 每次调用返回全新状态 (内存 store)
pub fn default_state() -> Arc<NotificationsApiState> {
    Arc::new(NotificationsApiState::new())
}

/// 生产存储: DATABASE_URL 可用 → PG (重启可恢复); 否则内存 + warn
/// (诚实降级, 不静默; 父代理挂载时建议显式注入 PgPool)
fn production_store() -> Arc<dyn NotifyStore> {
    match std::env::var("DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => match sqlx::postgres::PgPool::connect_lazy(url.trim())
        {
            Ok(pool) => {
                tracing::info!("notifications: PostgreSQL persistence enabled via DATABASE_URL");
                Arc::new(PgNotifyStore::new(pool))
            }
            Err(e) => {
                tracing::warn!(error = %e, "notifications: DATABASE_URL invalid, falling back to in-memory (data lost on restart)");
                Arc::new(InMemoryNotifyStore::new())
            }
        },
        _ => {
            tracing::warn!(
                "notifications: DATABASE_URL not set, using in-memory store (data lost on restart)"
            );
            Arc::new(InMemoryNotifyStore::new())
        }
    }
}

/// 生产嵌套路由用共享状态 (OnceLock, per routes/im.rs 模式)
fn state_arc() -> Arc<NotificationsApiState> {
    static STATE: OnceLock<Arc<NotificationsApiState>> = OnceLock::new();
    Arc::clone(
        STATE.get_or_init(|| Arc::new(NotificationsApiState::from_store(production_store()))),
    )
}

// ---------------------------------------------------------------------
// 生产鉴权 (JWT 验签 middleware + verified actor extractor)
// ---------------------------------------------------------------------

/// 生产 JWT 配置: 读 verifier env; 缺失 → deny-all 空配置 (所有 token
/// 验签失败 → 401, fail-closed, 不因配置缺失而放行)。
/// pub 供 routes/im.rs 复用同一份配置 (IM/通知同一套生产鉴权)。
pub fn prod_jwt_config() -> Arc<JwtConfig> {
    static CFG: OnceLock<Arc<JwtConfig>> = OnceLock::new();
    Arc::clone(CFG.get_or_init(|| {
        Arc::new(
            JwtConfig::verifier_from_env().unwrap_or_else(|_| JwtConfig {
                private_key_pem: String::new(),
                public_key_pem: String::new(),
                issuer: String::new(),
                audience: String::new(),
                ttl_seconds: 3600,
            }),
        )
    }))
}

/// 生产 JWT 验签 middleware: `Authorization: Bearer <token>` → `verify_token`
/// (复用 group/SCM 域同一套验签) → claim 一致性检查 → 注入 `AuthUser` 到
/// request extension。缺头/格式错/无效/过期 → 401 (BearerError)。
///
/// 注: 不做 device-binding 代际核验 (`AuthenticatedUser` extractor 需要
/// PG 仓储 state); 父代理挂载时可按 group 域同套替换。任何缺失一律拒绝。
pub async fn verify_jwt_and_inject_actor(
    State(jwt): State<Arc<JwtConfig>>,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, BearerError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(BearerError::MissingHeader)?
        .to_str()
        .map_err(|_| BearerError::InvalidFormat)?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(BearerError::InvalidFormat)?
        .trim();
    if token.is_empty() {
        return Err(BearerError::InvalidFormat);
    }
    let claims = match verify_token(&jwt, token) {
        Ok(c) => c,
        Err(JwtError::Expired) => return Err(BearerError::ExpiredToken),
        Err(JwtError::Missing) => return Err(BearerError::MissingHeader),
        Err(e) => return Err(BearerError::InvalidToken(e.code().to_string())),
    };
    if claims.sub != claims.user_id || claims.sub.is_nil() || claims.tenant_id.is_nil() {
        return Err(BearerError::InvalidToken(
            "subject claims mismatch".to_string(),
        ));
    }
    req.extensions_mut().insert(claims.to_auth_user());
    Ok(next.run(req).await)
}

/// 未认证拒绝 (extractor rejection): verified handler 缺 `AuthUser`
/// extension → 401。不做任何 header 兜底 (fail-closed)。
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

/// 已验证 actor (从 request extension 取 `AuthUser`, 由生产 middleware
/// 注入; 测试经 `.extension(AuthUser{...})` 注入 — 均代表已验证会话,
/// 不读 X-User-Id)
pub struct VerifiedActor(pub AuthUser);

impl<S> FromRequestParts<S> for VerifiedActor
where
    S: Send + Sync,
{
    type Rejection = NotAuthenticated;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(VerifiedActor)
            .ok_or(NotAuthenticated)
    }
}

// ---------------------------------------------------------------------
// routers
// ---------------------------------------------------------------------

/// 生产嵌套路由 (通知) — verified handler + JWT middleware (fail-closed)。
/// parent 以 `.nest("/notifications", router())` 挂载 (lib.rs 已注册)。
pub fn router() -> Router {
    router_with_verified_state(state_arc()).layer(middleware::from_fn_with_state(
        prod_jwt_config(),
        verify_jwt_and_inject_actor,
    ))
}

/// 生产嵌套路由 (评论) — verified handler + JWT middleware。parent 以
/// `.nest("/comments", comments_router())` 挂载 (lib.rs 已注册)。
pub fn comments_router() -> Router {
    comments_router_with_verified_state(state_arc()).layer(middleware::from_fn_with_state(
        prod_jwt_config(),
        verify_jwt_and_inject_actor,
    ))
}

/// caller-supplied state 嵌套路由 (通知; legacy X-User-Id 语义, 仅供旧
/// 集成测试; 生产一律走 `router()`)
pub fn router_with_state(state: Arc<NotificationsApiState>) -> Router {
    Router::new()
        .route("/", get(list_notifications))
        .route("/unread-count", get(unread_count))
        .route("/", post(create_notification))
        .route("/{id}", patch(patch_notification_read))
        .route("/{id}/read", post(mark_notification_read))
        .with_state(state)
}

/// caller-supplied state 嵌套路由 (评论; legacy X-User-Id 语义, 仅供旧
/// 集成测试; 生产一律走 `comments_router()`)
pub fn comments_router_with_state(state: Arc<NotificationsApiState>) -> Router {
    Router::new()
        .route("/", get(list_comments))
        .route("/", post(create_comment))
        .route("/{id}", delete(delete_comment))
        .with_state(state)
}

/// caller-supplied state 嵌套路由 (通知; verified — actor 来自已验证会话
/// extension, 缺 → 401; 租户/收件人强制隔离)
pub fn router_with_verified_state(state: Arc<NotificationsApiState>) -> Router {
    Router::new()
        .route("/", get(list_notifications_verified))
        .route("/unread-count", get(unread_count_verified))
        .route("/", post(create_notification_verified))
        .route("/{id}", patch(patch_notification_read_verified))
        .route("/{id}/read", post(mark_notification_read_verified))
        .with_state(state)
}

/// caller-supplied state 嵌套路由 (评论; verified — 同上)
pub fn comments_router_with_verified_state(state: Arc<NotificationsApiState>) -> Router {
    Router::new()
        .route("/", get(list_comments_verified))
        .route("/", post(create_comment_verified))
        .route("/{id}", delete(delete_comment_verified))
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

/// 403 POLICY_DENIED (跨租户访问/写入)
fn cross_tenant_denied(actor_tenant: &str, attempted: &str) -> RestError {
    RestError {
        code: "POLICY_DENIED".to_string(),
        message: format!(
            "tenant mismatch: actor tenant {actor_tenant} cannot access tenant {attempted}"
        ),
        source_module: "star-api-rest".to_string(),
        source_kind: "Policy".to_string(),
        retriable: false,
        hint: "Use the tenant id from your verified session".to_string(),
    }
}

/// 403 POLICY_DENIED (收件人不符: 只能读/标自己的通知)
fn recipient_denied(actor_user: &str, notification_id: &str) -> RestError {
    RestError {
        code: "POLICY_DENIED".to_string(),
        message: format!(
            "user {actor_user} cannot access notification {notification_id} (recipient isolation)"
        ),
        source_module: "star-api-rest".to_string(),
        source_kind: "Policy".to_string(),
        retriable: false,
        hint: "Notifications are scoped to their recipient (or tenant broadcast)".to_string(),
    }
}

/// 500 INTERNAL (outbox 入队/投递失败, 数据已在 outbox 待重试)
fn outbox_internal(err: &NotifyStoreError) -> RestError {
    RestError {
        code: "INTERNAL".to_string(),
        message: format!("outbox delivery failed: {err}"),
        source_module: "star-api-rest".to_string(),
        source_kind: "Outbox".to_string(),
        retriable: true,
        hint: "The event is retained in the outbox and will be retried".to_string(),
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

/// kind 字符串 → NotificationKind (宽松: 未知回退 System, 供存储层回读)
fn parse_notification_kind_loose(s: &str) -> NotificationKind {
    parse_notification_kind(s).unwrap_or(NotificationKind::System)
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

/// kind → 存储字符串 (serde snake_case)
fn serde_kind_str(kind: &NotificationKind) -> &'static str {
    match kind {
        NotificationKind::ReviewRequest => "review_request",
        NotificationKind::Comment => "comment",
        NotificationKind::Mention => "mention",
        NotificationKind::System => "system",
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
        "recipient_user_id": n.recipient_user_id,
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

/// 当前 epoch millis (outbox 到期判定用)
fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
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
    /// 租户 id — legacy 必填; verified 与 actor.tenant 比对
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
    /// 收件人用户 id (可选; 缺省 = 租户广播 recipient NULL, 租户内可见)
    #[serde(default)]
    pub recipient_user_id: Option<String>,
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
    /// 租户 id — legacy 必填; verified 与 actor.tenant 比对
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
    /// 作者显示名 (可选, 缺省回落为 actor id)
    #[serde(default)]
    pub author_name: Option<String>,
}

// ---------------------------------------------------------------------
// handlers (legacy X-User-Id 语义; 供旧集成测试, 生产不走此路径)
// ---------------------------------------------------------------------

/// `GET /api/v1/notifications` — 通知列表 (legacy)
///
/// 过滤: tenant_id (精确) / kind (非法值 → 400) / unread_only。分页:
/// limit (缺省 20, clamp 1..=200) + offset (缺省 0)。排序: created_at DESC
/// (新在前), 同刻按 id DESC 稳定。
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

    let mut matched = state
        .store
        .list_notifications(query.tenant_id.as_deref(), None, kind_filter, unread_only)
        .await
        .map_err(|e| outbox_internal(&e))?;
    matched.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));

    let total = matched.len() as i64;
    let page: Vec<Value> = matched
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|n| notification_to_value(&n))
        .collect();

    Ok(Json(RestResponse::ok(json!({
        "notifications": page,
        "total": total,
        "limit": limit,
        "offset": offset,
    }))))
}

/// `GET /api/v1/notifications/unread-count` — 未读计数 (legacy)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn unread_count(
    State(state): State<Arc<NotificationsApiState>>,
    Query(query): Query<UnreadCountQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let count = state
        .store
        .unread_count(query.tenant_id.as_deref(), None)
        .await
        .map_err(|e| outbox_internal(&e))?;

    Ok(Json(RestResponse::ok(json!({ "count": count }))))
}

/// `POST /api/v1/notifications` — 创建通知 (legacy, 直接落库不走 outbox)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_notification(
    State(state): State<Arc<NotificationsApiState>>,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let (tenant_id, kind, subject, body, target_type, target_id) = validate_create_request(&req)?;

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
        target_id,
        recipient_user_id: None,
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .store
        .insert_notification(notif.clone(), None)
        .await
        .map_err(|e| outbox_internal(&e))?;

    Ok(Json(RestResponse::ok(json!({
        "notification": notification_to_value(&notif),
    }))))
}

/// 创建请求校验结果 (公共校验 helper 返回类型别名, 降 clippy type_complexity)
type CreateValidation = (
    String,
    NotificationKind,
    String,
    String,
    Option<String>,
    Option<String>,
);

/// 创建请求公共校验 → (tenant, kind, subject, body, target_type, target_id)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
fn validate_create_request(req: &CreateNotificationRequest) -> Result<CreateValidation, RestError> {
    let tenant_id = required_field("tenant_id", req.tenant_id.clone())?;
    let kind = match &req.kind {
        Some(k) => parse_notification_kind(k)?,
        None => {
            return Err(notif_validation(
                "kind is required",
                "Use one of: review_request, comment, mention, system",
            ))
        }
    };
    let subject = required_field("subject", req.subject.clone())?;
    let body = required_field("body", req.body.clone())?;
    let target_type = match &req.target_type {
        Some(t) => {
            parse_target_type(t)?;
            Some(t.clone())
        }
        None => None,
    };
    if req.target_id.is_some() != target_type.is_some() {
        return Err(notif_validation(
            "target_type and target_id must be provided together",
            "Provide both target_type and target_id, or neither",
        ));
    }
    Ok((
        tenant_id,
        kind,
        subject,
        body,
        target_type,
        req.target_id.clone().filter(|s| !s.is_empty()),
    ))
}

/// `PATCH /api/v1/notifications/{id}` — 更新 read 状态 (legacy)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn patch_notification_read(
    State(state): State<Arc<NotificationsApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateNotificationReadRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let notif = state
        .store
        .set_notification_read(&id, req.read)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| notif_not_found(&id))?;

    Ok(Json(RestResponse::ok(
        json!({ "notification": notification_to_value(&notif) }),
    )))
}

/// `POST /api/v1/notifications/{id}/read` — 标记已读 (幂等, legacy)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn mark_notification_read(
    State(state): State<Arc<NotificationsApiState>>,
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let notif = state
        .store
        .set_notification_read(&id, true)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| notif_not_found(&id))?;

    Ok(Json(RestResponse::ok(
        json!({ "notification": notification_to_value(&notif) }),
    )))
}

/// `GET /api/v1/comments` — 评论列表 (legacy, 无租户强制过滤)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_comments(
    State(state): State<Arc<NotificationsApiState>>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let mut matched = state
        .store
        .list_comments(
            None,
            query.target_type.as_deref(),
            query.target_id.as_deref(),
        )
        .await
        .map_err(|e| outbox_internal(&e))?;
    matched.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));

    let total = matched.len();
    let comments: Vec<Value> = matched.iter().map(comment_to_value).collect();

    Ok(Json(RestResponse::ok(json!({
        "comments": comments,
        "total": total,
    }))))
}

/// `POST /api/v1/comments` — 新增评论 (legacy, 作者取 X-User-Id)
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
        .store
        .insert_comment(comment.clone(), None)
        .await
        .map_err(|e| outbox_internal(&e))?;

    Ok(Json(RestResponse::ok(json!({
        "comment": comment_to_value(&comment),
    }))))
}

/// `DELETE /api/v1/comments/{id}` — 删除评论 (legacy, 只能删自己的)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn delete_comment(
    State(state): State<Arc<NotificationsApiState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let user_id = current_user(&headers)?;
    let comment = state
        .store
        .get_comment(&id)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| comment_not_found(&id))?;
    if comment.author_id != user_id {
        return Err(not_comment_owner(&user_id, &id));
    }
    state
        .store
        .remove_comment(&id)
        .await
        .map_err(|e| outbox_internal(&e))?;

    Ok(Json(RestResponse::ok(json!({
        "deleted": true,
        "comment_id": id,
    }))))
}

// ---------------------------------------------------------------------
// handlers (verified: 身份从已验证会话取, 租户/收件人强制隔离)
// ---------------------------------------------------------------------

/// `GET /api/v1/notifications` (verified) — 列表只回 actor 自己的 +
/// 租户广播; query.tenant_id 与 actor 不符 → 403。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_notifications_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    let actor_user = actor.user_id.to_string();
    if let Some(t) = query.tenant_id.as_deref() {
        if t != actor_tenant {
            return Err(cross_tenant_denied(&actor_tenant, t));
        }
    }
    let kind_filter = match &query.kind {
        Some(k) => Some(parse_notification_kind(k)?),
        None => None,
    };
    let unread_only = query.unread_only.unwrap_or(false);
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);

    let mut matched = state
        .store
        .list_notifications(
            Some(&actor_tenant),
            Some(&actor_user),
            kind_filter,
            unread_only,
        )
        .await
        .map_err(|e| outbox_internal(&e))?;
    matched.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));

    let total = matched.len() as i64;
    let page: Vec<Value> = matched
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|n| notification_to_value(&n))
        .collect();

    Ok(Json(RestResponse::ok(json!({
        "notifications": page,
        "total": total,
        "limit": limit,
        "offset": offset,
    }))))
}

/// `GET /api/v1/notifications/unread-count` (verified) — 未读只算 actor
/// 自己的 + 租户广播 (服务端单一真源, 收件箱/铃铛同一份)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn unread_count_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Query(query): Query<UnreadCountQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    let actor_user = actor.user_id.to_string();
    if let Some(t) = query.tenant_id.as_deref() {
        if t != actor_tenant {
            return Err(cross_tenant_denied(&actor_tenant, t));
        }
    }
    let count = state
        .store
        .unread_count(Some(&actor_tenant), Some(&actor_user))
        .await
        .map_err(|e| outbox_internal(&e))?;

    Ok(Json(RestResponse::ok(json!({ "count": count }))))
}

/// `POST /api/v1/notifications` (verified) — 创建走 Outbox:
/// notification.created 事件入队 → relay 投递 → 幂等落库。
/// body.tenant_id 与 actor 不符 → 403。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_notification_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    if let Some(t) = req.tenant_id.as_deref() {
        if t != actor_tenant {
            return Err(cross_tenant_denied(&actor_tenant, t));
        }
    }
    let (tenant_id, kind, subject, body, target_type, target_id) = validate_create_request(&req)?;
    if tenant_id != actor_tenant {
        return Err(cross_tenant_denied(&actor_tenant, &tenant_id));
    }

    let notif = Notification {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id: actor_tenant.clone(),
        kind,
        subject,
        body,
        subject_url: req.subject_url.filter(|s| !s.is_empty()),
        actor_id: Some(actor.user_id.to_string()),
        actor_name: req.actor_name.filter(|s| !s.is_empty()),
        target_type,
        target_id,
        recipient_user_id: req.recipient_user_id.filter(|s| !s.is_empty()),
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let event = OutboxEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        event_type: "notification.created".to_string(),
        tenant_id: actor_tenant,
        payload: serde_json::to_value(&notif).map_err(|e| {
            notif_validation(
                format!("payload serialization failed: {e}"),
                "Internal serialization error",
            )
        })?,
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: now_ms(),
        delivered: false,
        dead: false,
    };
    state
        .store
        .enqueue_outbox(event)
        .await
        .map_err(|e| outbox_internal(&e))?;
    NotifyRelay::new()
        .drain(&state.store, now_ms())
        .await
        .map_err(|e| outbox_internal(&e))?;

    let stored = state
        .store
        .get_notification(&notif.id)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| outbox_internal(&NotifyStoreError::Unavailable("apply lost".to_string())))?;

    Ok(Json(RestResponse::ok(json!({
        "notification": notification_to_value(&stored),
    }))))
}

/// `PATCH /api/v1/notifications/{id}` (verified) — 租户 + 收件人隔离:
/// 他人个人通知 → 403; 租户广播允许租户内成员标记。
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn patch_notification_read_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Path(id): Path<String>,
    Json(req): Json<UpdateNotificationReadRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    let actor_user = actor.user_id.to_string();
    let existing = state
        .store
        .get_notification(&id)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| notif_not_found(&id))?;
    if existing.tenant_id != actor_tenant {
        return Err(cross_tenant_denied(&actor_tenant, &existing.tenant_id));
    }
    if let Some(recipient) = existing.recipient_user_id.as_deref() {
        if recipient != actor_user {
            return Err(recipient_denied(&actor_user, &id));
        }
    }
    let notif = state
        .store
        .set_notification_read(&id, req.read)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| notif_not_found(&id))?;

    Ok(Json(RestResponse::ok(
        json!({ "notification": notification_to_value(&notif) }),
    )))
}

/// `POST /api/v1/notifications/{id}/read` (verified) — 幂等, 隔离同上
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn mark_notification_read_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    let actor_user = actor.user_id.to_string();
    let existing = state
        .store
        .get_notification(&id)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| notif_not_found(&id))?;
    if existing.tenant_id != actor_tenant {
        return Err(cross_tenant_denied(&actor_tenant, &existing.tenant_id));
    }
    if let Some(recipient) = existing.recipient_user_id.as_deref() {
        if recipient != actor_user {
            return Err(recipient_denied(&actor_user, &id));
        }
    }
    let notif = state
        .store
        .set_notification_read(&id, true)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| notif_not_found(&id))?;

    Ok(Json(RestResponse::ok(
        json!({ "notification": notification_to_value(&notif) }),
    )))
}

/// `GET /api/v1/comments` (verified) — 租户强制 = actor.tenant
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn list_comments_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    let mut matched = state
        .store
        .list_comments(
            Some(&actor_tenant),
            query.target_type.as_deref(),
            query.target_id.as_deref(),
        )
        .await
        .map_err(|e| outbox_internal(&e))?;
    matched.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));

    let total = matched.len();
    let comments: Vec<Value> = matched.iter().map(comment_to_value).collect();

    Ok(Json(RestResponse::ok(json!({
        "comments": comments,
        "total": total,
    }))))
}

/// `POST /api/v1/comments` (verified) — 作者 = actor; 租户强制; 走 Outbox
/// (comment.created 事件幂等落库)
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn create_comment_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    if let Some(t) = req.tenant_id.as_deref() {
        if t != actor_tenant {
            return Err(cross_tenant_denied(&actor_tenant, t));
        }
    }
    let tenant_id = required_field("tenant_id", req.tenant_id.clone())?;
    if tenant_id != actor_tenant {
        return Err(cross_tenant_denied(&actor_tenant, &tenant_id));
    }
    let target_type = required_field("target_type", req.target_type)?;
    let target_id = required_field("target_id", req.target_id)?;
    let body = required_field("body", req.body)?;
    let author_name = match req.author_name {
        Some(n) if !n.is_empty() => n,
        _ => actor.user_id.to_string(),
    };

    let comment = Comment {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id: actor_tenant.clone(),
        target_type,
        target_id,
        author_id: actor.user_id.to_string(),
        author_name,
        body,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let event = OutboxEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        event_type: "comment.created".to_string(),
        tenant_id: actor_tenant,
        payload: serde_json::to_value(&comment).map_err(|e| {
            notif_validation(
                format!("payload serialization failed: {e}"),
                "Internal serialization error",
            )
        })?,
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: now_ms(),
        delivered: false,
        dead: false,
    };
    state
        .store
        .enqueue_outbox(event)
        .await
        .map_err(|e| outbox_internal(&e))?;
    NotifyRelay::new()
        .drain(&state.store, now_ms())
        .await
        .map_err(|e| outbox_internal(&e))?;

    let stored = state
        .store
        .get_comment(&comment.id)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| outbox_internal(&NotifyStoreError::Unavailable("apply lost".to_string())))?;

    Ok(Json(RestResponse::ok(json!({
        "comment": comment_to_value(&stored),
    }))))
}

/// `DELETE /api/v1/comments/{id}` (verified) — 租户 + 作者双重校验,
/// 非作者 → 403 POLICY_DENIED
#[allow(clippy::result_large_err)] // RestError 6-field 含 String, 跟 crates/star-api-rest 现有 handler 同模式
pub async fn delete_comment_verified(
    State(state): State<Arc<NotificationsApiState>>,
    VerifiedActor(actor): VerifiedActor,
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let actor_tenant = actor.tenant_id.to_string();
    let actor_user = actor.user_id.to_string();
    let comment = state
        .store
        .get_comment(&id)
        .await
        .map_err(|e| outbox_internal(&e))?
        .ok_or_else(|| comment_not_found(&id))?;
    if comment.tenant_id != actor_tenant {
        return Err(cross_tenant_denied(&actor_tenant, &comment.tenant_id));
    }
    if comment.author_id != actor_user {
        return Err(not_comment_owner(&actor_user, &id));
    }
    state
        .store
        .remove_comment(&id)
        .await
        .map_err(|e| outbox_internal(&e))?;

    Ok(Json(RestResponse::ok(json!({
        "deleted": true,
        "comment_id": id,
    }))))
}

//! SCM 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.scm.*`
//!
//! **本 crate 事件清单**(spec §5):
//! 1. `RepositoryRegistered` — `star.events.scm.repository.registered.v1`
//! 2. `RepositoryLinked` — `star.events.scm.repository.linked.v1`
//! 3. `SyncStateChanged` — `star.events.scm.sync_state.changed.v1`
//! 4. `PullRequestLinked` — `star.events.scm.pull_request.linked.v1`
//! 5. `PullRequestStateChanged` — `star.events.scm.pull_request.state_changed.v1`
//! 6. `WebhookReceived` — `star.events.scm.webhook.received.v1`
//! 7. `BranchCreated` — `star.events.scm.branch.created.v1`
//! 8. `CommitLinked` — `star.events.scm.commit.linked.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    BranchId, CommitId, ExternalRepositoryId, PullRequestId, RepositoryId, RepositoryOwnership,
    ScmProvider, SyncStatus, TenantId, WorkItemId,
};

/// 事件通用元数据(所有 Domain Event 共享的最小字段集)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件唯一 ID(UUID v4)
    pub event_id: uuid::Uuid,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    /// 构造一个 `EventMeta`(便于测试 / 命令 impl 中调用)。
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

// =====================================================================
// 事件载荷
// =====================================================================

/// `RepositoryRegistered` 事件载荷(`register_repository` 成功)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRegistered {
    /// 事件元数据
    pub meta: EventMeta,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// Provider
    pub provider: ScmProvider,
    /// 所有权
    pub ownership: RepositoryOwnership,
    /// 外部 ID
    pub external_id: ExternalRepositoryId,
}

/// `RepositoryLinked` 事件载荷(Repository 关联到 Project)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryLinked {
    /// 事件元数据
    pub meta: EventMeta,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 关联的 Project ID
    pub project_id: uuid::Uuid,
}

/// `SyncStateChanged` 事件载荷(`update_sync_state` 成功)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStateChanged {
    /// 事件元数据
    pub meta: EventMeta,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 新同步状态
    pub sync_status: SyncStatus,
    /// 上次同步时间
    pub last_synced_at: DateTime<Utc>,
}

/// `PullRequestLinked` 事件载荷(PR 关联 WorkItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestLinked {
    /// 事件元数据
    pub meta: EventMeta,
    /// PR ID
    pub pull_request_id: PullRequestId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
}

/// `PullRequestStateChanged` 事件载荷(PR 状态机迁移)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestStateChanged {
    /// 事件元数据
    pub meta: EventMeta,
    /// PR ID
    pub pull_request_id: PullRequestId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 旧状态
    pub from_state: String,
    /// 新状态
    pub to_state: String,
}

/// `WebhookReceived` 事件载荷(Webhook 入站)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookReceived {
    /// 事件元数据
    pub meta: EventMeta,
    /// Provider
    pub provider: ScmProvider,
    /// 事件类型字符串(push / pull_request / issues / pipeline)
    pub event_type: String,
    /// Repository ID(解析后)
    pub repository_id: Option<RepositoryId>,
    /// 厂商侧事件 ID
    pub external_event_id: String,
    /// 是否命中 Idempotency
    pub idempotent_hit: bool,
}

/// `BranchCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Branch ID
    pub branch_id: BranchId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 分支名
    pub name: String,
}

/// `CommitLinked` 事件载荷(Commit 关联 WorkItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLinked {
    /// 事件元数据
    pub meta: EventMeta,
    /// Commit ID
    pub commit_id: CommitId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
}

// =====================================================================
// 枚举:全部 SCM 域事件
// =====================================================================

/// 全部 SCM 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScmEvent {
    /// Repository 注册
    RepositoryRegistered(RepositoryRegistered),
    /// Repository 关联 Project
    RepositoryLinked(RepositoryLinked),
    /// 同步状态变化
    SyncStateChanged(SyncStateChanged),
    /// PR 关联 WorkItem
    PullRequestLinked(PullRequestLinked),
    /// PR 状态变化
    PullRequestStateChanged(PullRequestStateChanged),
    /// Webhook 接收
    WebhookReceived(WebhookReceived),
    /// Branch 创建
    BranchCreated(BranchCreated),
    /// Commit 关联 WorkItem
    CommitLinked(CommitLinked),
}

impl ScmEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::RepositoryRegistered(_) => "star.events.scm.repository.registered.v1",
            Self::RepositoryLinked(_) => "star.events.scm.repository.linked.v1",
            Self::SyncStateChanged(_) => "star.events.scm.sync_state.changed.v1",
            Self::PullRequestLinked(_) => "star.events.scm.pull_request.linked.v1",
            Self::PullRequestStateChanged(_) => "star.events.scm.pull_request.state_changed.v1",
            Self::WebhookReceived(_) => "star.events.scm.webhook.received.v1",
            Self::BranchCreated(_) => "star.events.scm.branch.created.v1",
            Self::CommitLinked(_) => "star.events.scm.commit.linked.v1",
        }
    }

    /// 事件的 tenant_id(便于订阅者按租户过滤)
    pub fn tenant_id(&self) -> TenantId {
        match self {
            Self::RepositoryRegistered(e) => e.meta.tenant_id,
            Self::RepositoryLinked(e) => e.meta.tenant_id,
            Self::SyncStateChanged(e) => e.meta.tenant_id,
            Self::PullRequestLinked(e) => e.meta.tenant_id,
            Self::PullRequestStateChanged(e) => e.meta.tenant_id,
            Self::WebhookReceived(e) => e.meta.tenant_id,
            Self::BranchCreated(e) => e.meta.tenant_id,
            Self::CommitLinked(e) => e.meta.tenant_id,
        }
    }
}

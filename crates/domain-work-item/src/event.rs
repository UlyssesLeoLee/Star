//! WorkItem 域事件(Domain Events,CloudEvents 1.0)
//!
//! 来源: docs/api-design.md §5 (CloudEvents 1.0 subject 命名空间)
//! 主题前缀: `star.events.work_item.*`
//!
//! **本 crate 事件清单**(6 个,Phase 2 实现):
//! 1. `WorkItemCreated` — `star.events.work_item.work_item.created.v1`
//! 2. `WorkItemUpdated` — `star.events.work_item.work_item.updated.v1`
//! 3. `WorkItemStatusChanged` — `star.events.work_item.work_item.status_changed.v1`
//! 4. `WorkItemWorktreeLinked` — `star.events.work_item.work_item.worktree_linked.v1`
//! 5. `WorkItemDeleted` — `star.events.work_item.work_item.deleted.v1`
//! 6. `AcceptanceCriterionCovered` — `star.events.work_item.acceptance_criterion.covered.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_object::{TenantId, WorkItemId, WorkItemStatus, WorktreeId};

/// 事件通用元数据(所有 Domain Event 共享的最小字段集)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件唯一 ID(UUID v4)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: TenantId,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者(可选,系统触发的事件可空)
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    /// 构造一个 `EventMeta`(便于测试 / 命令 impl 中调用)。
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// `WorkItemCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// 新建 WorkItem ID
    pub work_item_id: WorkItemId,
    /// Project ID
    pub project_id: uuid::Uuid,
    /// 类型
    pub work_item_type: String,
    /// 业务键
    pub work_item_key: String,
}

/// `WorkItemUpdated` 事件载荷(用于任意字段更新)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemUpdated {
    /// 事件元数据
    pub meta: EventMeta,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 变更字段列表(字段名字符串,便于下游按需处理)
    pub changed_fields: Vec<String>,
}

/// `WorkItemStatusChanged` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemStatusChanged {
    /// 事件元数据
    pub meta: EventMeta,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 旧状态
    pub from_status: WorkItemStatus,
    /// 新状态
    pub to_status: WorkItemStatus,
}

/// `WorkItemWorktreeLinked` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemWorktreeLinked {
    /// 事件元数据
    pub meta: EventMeta,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 关联的 Worktree ID
    pub worktree_id: WorktreeId,
}

/// `WorkItemDeleted` 事件载荷(软删除)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemDeleted {
    /// 事件元数据
    pub meta: EventMeta,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
}

/// `AcceptanceCriterionCovered` 事件载荷(由 validation 触发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterionCovered {
    /// 事件元数据
    pub meta: EventMeta,
    /// AC ID
    pub acceptance_criterion_id: uuid::Uuid,
    /// 所属 WorkItem ID
    pub work_item_id: WorkItemId,
    /// 触发覆盖的 Validation ID
    pub validation_id: Uuid,
    /// 新覆盖状态(`COVERED` / `PARTIAL` / `DISPUTED`)
    pub coverage_status: String,
}

/// 全部 WorkItem 域事件的枚举包装(便于 `UnboundedSender<WorkItemEvent>` 发送)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkItemEvent {
    /// WorkItem 创建
    Created(WorkItemCreated),
    /// WorkItem 任意字段更新
    Updated(WorkItemUpdated),
    /// 状态迁移
    StatusChanged(WorkItemStatusChanged),
    /// Worktree 链接
    WorktreeLinked(WorkItemWorktreeLinked),
    /// 软删除
    Deleted(WorkItemDeleted),
    /// AC 覆盖
    AcceptanceCriterionCovered(AcceptanceCriterionCovered),
}

impl WorkItemEvent {
    /// 事件的 CloudEvents subject(如 `star.events.work_item.work_item.created.v1`)。
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.work_item.work_item.created.v1",
            Self::Updated(_) => "star.events.work_item.work_item.updated.v1",
            Self::StatusChanged(_) => "star.events.work_item.work_item.status_changed.v1",
            Self::WorktreeLinked(_) => "star.events.work_item.work_item.worktree_linked.v1",
            Self::Deleted(_) => "star.events.work_item.work_item.deleted.v1",
            Self::AcceptanceCriterionCovered(_) => {
                "star.events.work_item.acceptance_criterion.covered.v1"
            }
        }
    }
}

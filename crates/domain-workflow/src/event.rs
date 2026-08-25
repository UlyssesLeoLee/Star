//! Workflow 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.workflow.*`
//!
//! **本 crate 事件清单**(spec §5):
//! 1. `WorkflowCreated` — `star.events.workflow.workflow.created.v1`
//! 2. `WorkflowUpdated` — `star.events.workflow.workflow.updated.v1`
//! 3. `WorkflowDeleted` — `star.events.workflow.workflow.deleted.v1`
//! 4. `StateAdded` — `star.events.workflow.state.added.v1`
//! 5. `TransitionAdded` — `star.events.workflow.transition.added.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{ProjectId, StateId, TenantId, TransitionId, WorkflowId};

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

/// `WorkflowCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// 关联 Project(空 = system_default)
    pub project_id: Option<ProjectId>,
    /// 是否为 system_default
    pub is_system_default: bool,
}

/// `WorkflowUpdated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowUpdated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// 新版本号
    pub version: u32,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// `WorkflowDeleted` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDeleted {
    /// 事件元数据
    pub meta: EventMeta,
    /// Workflow ID
    pub workflow_id: WorkflowId,
}

/// `StateAdded` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAdded {
    /// 事件元数据
    pub meta: EventMeta,
    /// 新增 State ID
    pub state_id: StateId,
    /// 所属 Workflow
    pub workflow_id: WorkflowId,
    /// State 名称
    pub name: String,
    /// State 类别
    pub category: String,
}

/// `TransitionAdded` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionAdded {
    /// 事件元数据
    pub meta: EventMeta,
    /// 新增 Transition ID
    pub transition_id: TransitionId,
    /// 所属 Workflow
    pub workflow_id: WorkflowId,
    /// 源 State
    pub from_state_id: StateId,
    /// 目标 State
    pub to_state_id: StateId,
}

// =====================================================================
// 枚举:全部 Workflow 域事件
// =====================================================================

/// 全部 Workflow 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowEvent {
    /// Workflow 创建
    Created(WorkflowCreated),
    /// Workflow 更新
    Updated(WorkflowUpdated),
    /// Workflow 删除
    Deleted(WorkflowDeleted),
    /// State 添加
    StateAdded(StateAdded),
    /// Transition 添加
    TransitionAdded(TransitionAdded),
}

impl WorkflowEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.workflow.workflow.created.v1",
            Self::Updated(_) => "star.events.workflow.workflow.updated.v1",
            Self::Deleted(_) => "star.events.workflow.workflow.deleted.v1",
            Self::StateAdded(_) => "star.events.workflow.state.added.v1",
            Self::TransitionAdded(_) => "star.events.workflow.transition.added.v1",
        }
    }
}

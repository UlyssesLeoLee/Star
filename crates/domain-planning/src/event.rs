//! Planning 域事件(Domain Events,CloudEvents 1.0)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{ProjectId, SprintId, TenantId, WorkItemId};

/// 事件通用元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// `SprintCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintCreated {
    pub meta: EventMeta,
    pub sprint_id: SprintId,
    pub project_id: ProjectId,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

/// `SprintStarted` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintStarted {
    pub meta: EventMeta,
    pub sprint_id: SprintId,
    pub started_at: DateTime<Utc>,
    pub work_item_count: u32,
}

/// `SprintClosed` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintClosed {
    pub meta: EventMeta,
    pub sprint_id: SprintId,
    pub closed_at: DateTime<Utc>,
    pub moved_incomplete_to: String,
}

/// `BacklogReordered` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogReordered {
    pub meta: EventMeta,
    pub project_id: ProjectId,
    pub new_order: Vec<WorkItemId>,
}

/// `WorkItemAddedToSprint` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemAddedToSprint {
    pub meta: EventMeta,
    pub sprint_id: SprintId,
    pub work_item_id: WorkItemId,
}

/// 全部 Planning 域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlanningEvent {
    Created(SprintCreated),
    Started(SprintStarted),
    Closed(SprintClosed),
    BacklogReordered(BacklogReordered),
    WorkItemAdded(WorkItemAddedToSprint),
}

impl PlanningEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.planning.sprint.created.v1",
            Self::Started(_) => "star.events.planning.sprint.started.v1",
            Self::Closed(_) => "star.events.planning.sprint.closed.v1",
            Self::BacklogReordered(_) => "star.events.planning.backlog.reordered.v1",
            Self::WorkItemAdded(_) => "star.events.planning.sprint.work_item_added.v1",
        }
    }
}

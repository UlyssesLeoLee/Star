//! Relation 域事件

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{RelationId, RelationType, TenantId, WorkItemId};

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

/// `RelationCreated`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationCreated {
    pub meta: EventMeta,
    pub relation_id: RelationId,
    pub source_id: WorkItemId,
    pub target_id: WorkItemId,
    pub relation_type: RelationType,
}

/// `RelationDeleted`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationDeleted {
    pub meta: EventMeta,
    pub relation_id: RelationId,
    pub source_id: WorkItemId,
    pub target_id: WorkItemId,
}

/// `CircularDependencyDetected`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyDetected {
    pub meta: EventMeta,
    pub work_item_id: WorkItemId,
    pub cycle: Vec<WorkItemId>,
}

/// 全部 Relation 域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RelationEvent {
    Created(RelationCreated),
    Deleted(RelationDeleted),
    CircularDetected(CircularDependencyDetected),
}

impl RelationEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.relation.relation.created.v1",
            Self::Deleted(_) => "star.events.relation.relation.deleted.v1",
            Self::CircularDetected(_) => "star.events.relation.dependency.circular_detected.v1",
        }
    }
}

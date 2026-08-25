//! Workspace 域事件

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{TenantId, UserId, WorkspaceId, WorkspaceRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: Option<UserId>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCreated {
    pub meta: EventMeta,
    pub workspace_id: WorkspaceId,
    pub workspace_key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAdded {
    pub meta: EventMeta,
    pub workspace_id: WorkspaceId,
    pub user_id: UserId,
    pub role: WorkspaceRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemoved {
    pub meta: EventMeta,
    pub workspace_id: WorkspaceId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceEvent {
    Created(WorkspaceCreated),
    MemberAdded(MemberAdded),
    MemberRemoved(MemberRemoved),
}

impl WorkspaceEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.workspace.workspace.created.v1",
            Self::MemberAdded(_) => "star.events.workspace.member.added.v1",
            Self::MemberRemoved(_) => "star.events.workspace.member.removed.v1",
        }
    }
}

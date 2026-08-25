//! Permission 域事件

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{PermissionSchemeId, RoleId, TenantId};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCreated {
    pub meta: EventMeta,
    pub role_id: RoleId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionChecked {
    pub meta: EventMeta,
    pub role_id: RoleId,
    pub permission: String,
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeCreated {
    pub meta: EventMeta,
    pub scheme_id: PermissionSchemeId,
    pub project_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PermissionEvent {
    RoleCreated(RoleCreated),
    PermissionChecked(PermissionChecked),
    SchemeCreated(SchemeCreated),
}

impl PermissionEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::RoleCreated(_) => "star.events.permission.role.created.v1",
            Self::PermissionChecked(_) => "star.events.permission.permission.checked.v1",
            Self::SchemeCreated(_) => "star.events.permission.scheme.created.v1",
        }
    }
}

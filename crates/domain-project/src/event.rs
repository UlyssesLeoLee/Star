//! Project 域事件

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{ProjectId, ProjectTemplateType, TenantId};

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
            event_id: UserId.new(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreated {
    pub meta: EventMeta,
    pub project_id: ProjectId,
    pub workspace_id: uuid::Uuid,
    pub project_key: String,
    pub template_type: ProjectTemplateType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPolicyUpdated {
    pub meta: EventMeta,
    pub project_id: ProjectId,
    pub changed_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectEvent {
    Created(ProjectCreated),
    PolicyUpdated(ProjectPolicyUpdated),
}

impl ProjectEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.project.project.created.v1",
            Self::PolicyUpdated(_) => "star.events.project.policy.updated.v1",
        }
    }
}

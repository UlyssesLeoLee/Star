//! Project 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(ProjectId);
define_uuid_id!(ProjectTemplateId);
define_uuid_id!(ProjectPolicyId);
define_uuid_id!(TenantId);
define_uuid_id!(WorkspaceId);
define_uuid_id!(WorkflowId);
define_uuid_id!(PermissionSchemeId);
define_uuid_id!(NotificationSchemeId);
define_uuid_id!(AgentPolicyId);

/// **Project 模板类型**(`project.template_type`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectTemplateType {
    /// 敏捷软件开发
    SoftwareDev,
    /// 看板
    Kanban,
    /// Scrum
    Scrum,
    /// 运维
    Operations,
}

impl Default for ProjectTemplateType {
    fn default() -> Self {
        Self::SoftwareDev
    }
}

impl std::fmt::Display for ProjectTemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SoftwareDev => "SOFTWARE_DEV",
            Self::Kanban => "KANBAN",
            Self::Scrum => "SCRUM",
            Self::Operations => "OPERATIONS",
        };
        f.write_str(s)
    }
}

/// **Project 状态**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectStatus {
    /// 活跃
    Active,
    /// 归档
    Archived,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "ACTIVE",
            Self::Archived => "ARCHIVED",
        };
        f.write_str(s)
    }
}

pub mod roles {
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
    pub const VIEWER: &str = "viewer";
}

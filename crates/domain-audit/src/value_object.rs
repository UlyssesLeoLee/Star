//! Audit 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(AuditEventId);
define_uuid_id!(AIAuditMetadataId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);

/// **审计动作类型**(`audit_event.action` 字段的标准枚举)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    /// User 创建
    UserCreate,
    /// User 更新
    UserUpdate,
    /// User 删除
    UserDelete,
    /// 权限变更
    PermissionChange,
    /// 角色分配
    RoleAssign,
    /// AI Agent 执行
    AgentExecute,
    /// Context Build
    ContextBuild,
    /// Validation Run
    ValidationRun,
    /// Worktree 操作
    WorktreeOperate,
    /// Tenant 状态变更
    TenantStatusChange,
    /// 通用自定义动作
    Custom,
}

impl Default for AuditAction {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UserCreate => "USER_CREATE",
            Self::UserUpdate => "USER_UPDATE",
            Self::UserDelete => "USER_DELETE",
            Self::PermissionChange => "PERMISSION_CHANGE",
            Self::RoleAssign => "ROLE_ASSIGN",
            Self::AgentExecute => "AGENT_EXECUTE",
            Self::ContextBuild => "CONTEXT_BUILD",
            Self::ValidationRun => "VALIDATION_RUN",
            Self::WorktreeOperate => "WORKTREE_OPERATE",
            Self::TenantStatusChange => "TENANT_STATUS_CHANGE",
            Self::Custom => "CUSTOM",
        };
        f.write_str(s)
    }
}

pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const TENANT_AUDITOR: &str = "tenant_auditor";
}

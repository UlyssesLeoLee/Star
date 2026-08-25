//! Workspace 域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    TenantId, UserId, WorkspaceId, WorkspaceMemberId, WorkspaceRole,
};

/// **Workspace 聚合根**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// 主键
    pub id: WorkspaceId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 业务键(tenant 内唯一,INV-WS-01)
    pub workspace_key: String,
    /// 显示名
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本
    pub version: u32,
}

impl Workspace {
    pub const FIELD_COUNT: usize = 8;
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

/// **WorkspaceMember**(Workspace 成员)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// 主键
    pub id: WorkspaceMemberId,
    /// Workspace ID
    pub workspace_id: WorkspaceId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 用户 ID
    pub user_id: UserId,
    /// 角色
    pub role: WorkspaceRole,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
    /// 乐观锁版本
    pub version: u32,
}

impl WorkspaceMember {
    pub const FIELD_COUNT: usize = 7;
    pub fn is_admin(&self) -> bool {
        self.role == WorkspaceRole::Admin
    }
}

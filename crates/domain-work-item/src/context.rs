//! 调用方上下文(`ActorContext`)
//!
//! 来源: docs/specs/domain-work-item-spec.md §4.2(命令/查询端口签名),docs/basic-design.md §23.2
//!
//! **Phase 2 现状**:本 crate 自带最小 `ActorContext`(本 crate 内调用方使用)。
//! **Phase 3 计划**:由 `domain-identity` 颁发统一 `ActorContext` 取代本 crate 内定义,
//! 以避免 `domain-*` 之间的循环依赖(参见 `lib.rs` 上游依赖说明)。

use serde::{Deserialize, Serialize};

use crate::value_object::{ProjectId, TenantId, UserId};

/// **Actor 上下文**(命令/查询端口的 `actor` / `viewer` 参数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID(来自 `identity.user`)
    pub user_id: UserId,

    /// 当前租户 ID(13 类对象必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<String>,

    /// 当前用户已加入的 Project IDs(用于 Project Policy 校验)
    pub project_ids: Vec<ProjectId>,

    /// 当前用户角色(`tenant_admin` / `project_admin` / `developer` / `viewer`)
    ///
    /// 见 `crate::value_object::roles` 标准常量
    pub roles: Vec<String>,
}

impl ActorContext {
    /// 构造一个新的 `ActorContext`(测试 / 默认场景便捷构造)。
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            project_ids: Vec::new(),
            roles: Vec::new(),
        }
    }

    /// 是否具备指定角色。
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 是否为租户管理员。
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(crate::value_object::roles::TENANT_ADMIN)
    }

    /// 是否具备 `project_id` 的成员资格(简化版,真实校验交给 `domain-permission`)。
    pub fn is_member_of(&self, project_id: ProjectId) -> bool {
        self.is_tenant_admin() || self.project_ids.contains(&project_id)
    }

    /// 添加角色(测试 / 构造场景使用)。
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// 添加 Project 成员资格。
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        if !self.project_ids.contains(&project_id) {
            self.project_ids.push(project_id);
        }
        self
    }
}

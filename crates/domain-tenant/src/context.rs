//! 调用方上下文(`ActorContext`)
//!
//! **Phase 2 现状**:本 crate 自带最小 `ActorContext`(本 crate 内调用方使用)。
//! **Phase 3 计划**:由 `domain-identity` 颁发统一 `ActorContext` 取代本 crate 内定义,
//! 以避免 `domain-*` 之间的循环依赖(参见 `lib.rs` 上游依赖说明)。

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, TenantId, TenantPolicyId};

/// **Actor 上下文**(命令/查询端口的 `actor` / `viewer` 参数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID(强类型,Phase 3 由 `domain-identity` 颁发时统一)
    pub user_id: uuid::Uuid,
    /// 当前租户 ID
    pub tenant_id: TenantId,
    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<uuid::Uuid>,
    /// 当前用户角色(`tenant_admin` / `tenant_auditor` / `platform_operator`)
    pub roles: Vec<String>,
    /// 关联的 TenantPolicy ID(用于 Policy 写权限校验)
    pub tenant_policy_id: Option<TenantPolicyId>,
}

impl ActorContext {
    /// 构造一个新的 `ActorContext`(测试 / 默认场景便捷构造)。
    pub fn new(user_id: uuid::Uuid, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            roles: Vec::new(),
            tenant_policy_id: None,
        }
    }

    /// 是否具备指定角色。
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 是否为租户管理员。
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(roles::TENANT_ADMIN)
    }

    /// 是否为平台运营(可跨租户操作,需额外鉴权)。
    pub fn is_platform_operator(&self) -> bool {
        self.has_role(roles::PLATFORM_OPERATOR)
    }

    /// 添加角色(测试 / 构造场景使用)。
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
}

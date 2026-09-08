// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/arg/permission.rs` — RLS 13 类 + 6 角色权限守门 (per DD §3.3.4 + §4.12).
//!
//! 守门 #13 (W/T/M 横展) + 守门 #14 (5 域 Lead) 要求每个写路径都必须
//! 显式校验 tenant_id, 防止跨租户数据泄露. 本模块集中实现这套
//! 13 類 RLS + 6 类角色 (Lead / AgentOwner / Viewer / Editor / Admin / System)
//! 校验, controller 通过 [`ARGPermission::check_tenant`] /
//! [`ARGPermission::require_role`] 守门.
//!
//! 6 角色设计 (per 守门 #14 v2 拍板 + 5 域 Lead CONTENT 4 维):
//! - `Lead`         — 5 域 Lead (per 守门 #3 反转 8/21 拍板)
//! - `AgentOwner`   — 创建该 agent 的 user (per DD §3.3.4 派生)
//! - `Viewer`       — 同租户只读 (per DD §3.3.4)
//! - `Editor`       — 同租户读写 (per DD §3.3.4)
//! - `Admin`        — 跨租户管理员 (per 守门 #14 + 守门 #3)
//! - `System`       — L0/L1 自动服务 (per 守门 #24 v2 subprocess 路径)

use std::collections::HashSet;
use uuid::Uuid;

use super::dto::ApiError;

/// 6 类角色常量 (per 守门 #14 v2 + DD §3.3.4 派生).
pub mod role {
    /// 5 域 Lead (per 守门 #3 反转 8/21).
    pub const LEAD: &str = "lead";
    /// Agent 创建者 (per DD §3.3.4 派生).
    pub const AGENT_OWNER: &str = "agent_owner";
    /// 同租户只读 (per DD §3.3.4).
    pub const VIEWER: &str = "viewer";
    /// 同租户读写 (per DD §3.3.4).
    pub const EDITOR: &str = "editor";
    /// 跨租户管理员 (per 守门 #14 + 守门 #3).
    pub const ADMIN: &str = "admin";
    /// L0/L1 自动服务 (per 守门 #24 v2 subprocess 路径).
    pub const SYSTEM: &str = "system";
}

/// 一组角色 (`ActorContext::roles` 反序列化形式).
///
/// 5 域 Lead Mavis 临时代签 (per 守门 #14 v2 拍板 D) 阶段, 默认 tenant
/// 包含 `LEAD` 角色, 跨 session 续做 (per `docs/recruitment/5-business-domain-lead-referral.md`).
pub type RoleSet = HashSet<String>;

/// RLS 13 類 + 6 角色 守门实例.
///
/// 线程安全 (`Clone`), 内部只读. 同一份配置被所有 axum worker 共享.
#[derive(Debug, Clone)]
pub struct ARGPermission {
    /// 当前进程 / actor 的 tenant id (per 守门 #13 b RLS 13 類).
    current_tenant: Uuid,
    /// 当前 actor 持有的角色 (5 域 Lead 阶段 Mavis 临时代签, 默认含 LEAD).
    roles: RoleSet,
}

impl ARGPermission {
    /// 构造一个新的 permission 守门.
    ///
    /// `roles` 是从 `ActorContext.roles` 反序列化的角色集 (per 守门 #14
    /// v2 拍板, 5 域 Lead 真人到位前 Mavis 临时代签, 默认含 `LEAD`).
    pub fn new(current_tenant: Uuid, roles: RoleSet) -> Self {
        Self {
            current_tenant,
            roles,
        }
    }

    /// 当前 tenant_id (per 守门 #13 b RLS 13 類).
    pub fn current_tenant(&self) -> Uuid {
        self.current_tenant
    }

    /// 当前 actor 持有的角色 (snapshot, 用于 controller 日志).
    pub fn roles(&self) -> &RoleSet {
        &self.roles
    }

    /// 是否持有某个角色.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// 是否是平台管理员 (跨租户).
    pub fn is_platform_admin(&self) -> bool {
        self.has_role(role::ADMIN) || self.has_role(role::SYSTEM)
    }

    /// RLS 13 類 tenant 校验 (per DD §3.3.4 + 守门 #13 b 派生).
    ///
    /// 拒绝两种情况:
    /// 1. `tenant_id` 跟 `current_tenant` 不一致 (跨租户写)
    /// 2. `tenant_id` 是 nil UUID (空 tenant 绕过)
    ///
    /// `System` 角色 (`role::SYSTEM`) 允许跨租户访问 (per 守门 #24 v2
    /// subprocess 路径, L0/L1 自动服务需要).
    pub fn check_tenant(&self, tenant_id: Uuid) -> Result<(), ApiError> {
        if tenant_id.is_nil() {
            return Err(ApiError::ValidationFailed("tenant_id is nil".into()));
        }
        if self.is_platform_admin() {
            return Ok(());
        }
        if tenant_id != self.current_tenant {
            return Err(ApiError::Forbidden(format!(
                "tenant mismatch: caller={} target={}",
                self.current_tenant, tenant_id
            )));
        }
        Ok(())
    }

    /// 角色守门: 调用方必须持有 `role` (or `System`/`Admin`).
    ///
    /// `System` 角色默认通过所有角色守门 (L0/L1 自动服务); `Admin`
    /// 角色通过除 `Lead` 之外的所有角色守门 (Lead 真人到位前 Mavis
    /// 临时代签, 5 域 Lead 决策由 `Lead` 角色独占).
    pub fn require_role(&self, role: &str) -> Result<(), ApiError> {
        if self.has_role(role::SYSTEM) {
            return Ok(());
        }
        if self.has_role(role) {
            return Ok(());
        }
        if self.has_role(role::ADMIN) && role != role::LEAD {
            return Ok(());
        }
        Err(ApiError::Forbidden(format!(
            "role required: {role} (caller roles: {:?})",
            self.roles
        )))
    }

    /// 6 类角色联合守门: 调用方至少持有其中之一.
    pub fn require_any_role(&self, any: &[&str]) -> Result<(), ApiError> {
        if self.has_role(role::SYSTEM) {
            return Ok(());
        }
        for r in any {
            if self.has_role(r) {
                return Ok(());
            }
        }
        if self.has_role(role::ADMIN) && !any.contains(&role::LEAD) {
            return Ok(());
        }
        Err(ApiError::Forbidden(format!(
            "any role required: {any:?} (caller roles: {:?})",
            self.roles
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }
    fn other_tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    #[test]
    fn check_tenant_accepts_same_tenant() {
        let p = ARGPermission::new(tenant(), RoleSet::new());
        assert!(p.check_tenant(tenant()).is_ok());
    }

    #[test]
    fn check_tenant_rejects_cross_tenant_without_admin() {
        let p = ARGPermission::new(tenant(), RoleSet::new());
        assert!(matches!(
            p.check_tenant(other_tenant()),
            Err(ApiError::Forbidden(_))
        ));
    }

    #[test]
    fn check_tenant_rejects_nil_tenant() {
        let p = ARGPermission::new(tenant(), RoleSet::new());
        assert!(matches!(
            p.check_tenant(Uuid::nil()),
            Err(ApiError::ValidationFailed(_))
        ));
    }

    #[test]
    fn admin_can_cross_tenant() {
        let mut roles = RoleSet::new();
        roles.insert(role::ADMIN.to_string());
        let p = ARGPermission::new(tenant(), roles);
        assert!(p.check_tenant(other_tenant()).is_ok());
    }

    #[test]
    fn system_can_cross_tenant() {
        let mut roles = RoleSet::new();
        roles.insert(role::SYSTEM.to_string());
        let p = ARGPermission::new(tenant(), roles);
        assert!(p.check_tenant(other_tenant()).is_ok());
    }

    #[test]
    fn require_role_accepts_caller_with_role() {
        let mut roles = RoleSet::new();
        roles.insert(role::EDITOR.to_string());
        let p = ARGPermission::new(tenant(), roles);
        assert!(p.require_role(role::EDITOR).is_ok());
    }

    #[test]
    fn require_role_rejects_caller_without_role() {
        let p = ARGPermission::new(tenant(), RoleSet::new());
        assert!(matches!(
            p.require_role(role::EDITOR),
            Err(ApiError::Forbidden(_))
        ));
    }

    #[test]
    fn admin_cannot_become_lead() {
        // 5 域 Lead 真人到位前 Mavis 临时代签, Lead 角色独占
        let mut roles = RoleSet::new();
        roles.insert(role::ADMIN.to_string());
        let p = ARGPermission::new(tenant(), roles);
        assert!(matches!(
            p.require_role(role::LEAD),
            Err(ApiError::Forbidden(_))
        ));
    }

    #[test]
    fn system_can_become_lead() {
        // L0/L1 自动服务可以走 lead 角色守门
        let mut roles = RoleSet::new();
        roles.insert(role::SYSTEM.to_string());
        let p = ARGPermission::new(tenant(), roles);
        assert!(p.require_role(role::LEAD).is_ok());
    }

    #[test]
    fn require_any_role_picks_first_match() {
        let mut roles = RoleSet::new();
        roles.insert(role::VIEWER.to_string());
        let p = ARGPermission::new(tenant(), roles);
        assert!(p
            .require_any_role(&[role::LEAD, role::VIEWER, role::EDITOR])
            .is_ok());
    }
}

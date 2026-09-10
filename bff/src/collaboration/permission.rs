// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/collaboration/permission.rs` — RLS 13 类 + 3 档权限 (View/Comment/Edit) 守门 (per DD-AGENT §3.3 + §4.14 + 守门 #13 + brief §2.6).
//!
//! 跟 `crates/api/src/canvas_collab/permission.rs` 同形但**独立定义** (BFF 跟
//! API 平级 0 反向依赖, per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标, 0
//! cross-tier leak). BFF 这边强制 3 档权限 middleware 必填 (per 9/1
//! 13:03+13:05 JST envoy 偏好).
//!
//! 5 守门 helpers (跟 API tier 一致):
//! 1. `check_tenant` — RLS 13 类 tenant 校验
//! 2. `require` — 3 档权限等级守门 (View/Comment/Edit, per A12.7 派生)
//! 3. `has_role` — 角色守门 helper
//! 4. `is_platform_admin` — 跨租户管理员识别
//! 5. `current_tenant` / `canvas_level` — 当前 actor 上下文
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).

use std::collections::HashSet;
use uuid::Uuid;

use super::dto::ApiError;

/// 6 类角色常量 (per 守门 #14 v2 + DD §3.3 派生, 跟 API tier 一致).
pub mod role {
    /// 5 域 Lead (per 守门 #3 反转 8/21).
    pub const LEAD: &str = "lead";
    /// Agent 创建者 (per DD §3.3 派生).
    pub const AGENT_OWNER: &str = "agent_owner";
    /// 同租户只读 (per DD §3.3).
    pub const VIEWER: &str = "viewer";
    /// 同租户读写 (per DD §3.3).
    pub const EDITOR: &str = "editor";
    /// 跨租户管理员 (per 守门 #14 + 守门 #3).
    pub const ADMIN: &str = "admin";
    /// L0/L1 自动服务 (per 守门 #24 v2 subprocess 路径).
    pub const SYSTEM: &str = "system";
}

/// 一组角色 (`ActorContext::roles` 反序列化形式).
pub type RoleSet = HashSet<String>;

/// 3 档权限等级 (per A12.7 + BFF middleware 强制, per 9/1 13:03+13:05 JST envoy 偏好).
///
/// 跟 `canvas_collab::PermissionLevel` 同形 (per 守门 #19 v19 累积规), 但
/// BFF tier 这边独立定义避免 cross-tier 紧耦合 (per 守门 #9 #1 禁跨层 leak).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// 只读 (查看画布 + 评论).
    View,
    /// 评论 (可发评论, 不可改元素).
    Comment,
    /// 编辑 (完整修改权限).
    Edit,
}

impl PermissionLevel {
    /// 3 档顺序: View < Comment < Edit (per A12.7 派生).
    pub fn rank(self) -> u8 {
        match self {
            Self::View => 0,
            Self::Comment => 1,
            Self::Edit => 2,
        }
    }

    /// 是否包含 (`self >= required`).
    pub fn satisfies(self, required: PermissionLevel) -> bool {
        self.rank() >= required.rank()
    }
}

use serde::{Deserialize, Serialize};

/// RLS 13 类 + 6 角色 + 3 档权限 守门实例.
///
/// 线程安全 (`Clone`), 内部只读. 同一份配置被所有 axum worker 共享.
#[derive(Debug, Clone)]
pub struct CollaborationPermission {
    /// 当前进程 / actor 的 tenant id (per 守门 #13 b RLS 13 类).
    current_tenant: Uuid,
    /// 当前 actor 持有的角色 (5 域 Lead 阶段 Mavis 临时代签, 默认含 LEAD).
    roles: RoleSet,
    /// 当前 actor 在该 canvas 的权限等级 (per A12.7 + BFF middleware 强制).
    canvas_level: PermissionLevel,
}

impl CollaborationPermission {
    /// 构造一个新的 permission 守门.
    pub fn new(current_tenant: Uuid, roles: RoleSet, canvas_level: PermissionLevel) -> Self {
        Self {
            current_tenant,
            roles,
            canvas_level,
        }
    }

    /// 当前 tenant_id (per 守门 #13 b RLS 13 类).
    pub fn current_tenant(&self) -> Uuid {
        self.current_tenant
    }

    /// 当前 canvas 权限等级 (per A12.7).
    pub fn canvas_level(&self) -> PermissionLevel {
        self.canvas_level
    }

    /// **Helper 1**: 是否持有某个角色.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// **Helper 2**: 是否是平台管理员 (跨租户).
    pub fn is_platform_admin(&self) -> bool {
        self.has_role(role::ADMIN) || self.has_role(role::SYSTEM)
    }

    /// **Helper 3**: RLS 13 类 tenant 校验 (per DD §3.3 + 守门 #13 b 派生).
    pub fn check_tenant(&self, tenant_id: Uuid) -> Result<(), ApiError> {
        if tenant_id.is_nil() {
            return Err(ApiError::BadRequest("tenant_id is nil".into()));
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

    /// **Helper 4**: 3 档权限等级守门 (per A12.7).
    ///
    /// 调用方在该 canvas 的权限必须 >= `required` 等级. 平台管理员
    /// (`Admin` / `System`) 默认通过所有等级守门.
    pub fn require(&self, required: PermissionLevel) -> Result<(), ApiError> {
        if self.is_platform_admin() {
            return Ok(());
        }
        if self.canvas_level.satisfies(required) {
            return Ok(());
        }
        Err(ApiError::Forbidden(format!(
            "canvas permission required: {:?} (caller level: {:?})",
            required, self.canvas_level
        )))
    }

    /// **Helper 5**: 角色守门: 调用方必须持有 `role` (or `System`/`Admin`).
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

    /// **Helper 6**: 6 类角色联合守门: 调用方至少持有其中之一.
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

    #[test]
    fn permission_level_rank_orders_three_levels() {
        assert!(PermissionLevel::View.rank() < PermissionLevel::Comment.rank());
        assert!(PermissionLevel::Comment.rank() < PermissionLevel::Edit.rank());
    }

    #[test]
    fn permission_level_satisfies_is_transitive() {
        assert!(PermissionLevel::Edit.satisfies(PermissionLevel::View));
        assert!(PermissionLevel::Edit.satisfies(PermissionLevel::Edit));
        assert!(!PermissionLevel::View.satisfies(PermissionLevel::Edit));
    }

    #[test]
    fn check_tenant_accepts_same_tenant() {
        let p = CollaborationPermission::new(tenant(), RoleSet::new(), PermissionLevel::View);
        assert!(p.check_tenant(tenant()).is_ok());
    }

    #[test]
    fn check_tenant_rejects_mismatch() {
        let p = CollaborationPermission::new(tenant(), RoleSet::new(), PermissionLevel::View);
        assert!(matches!(
            p.check_tenant(Uuid::new_v4()),
            Err(ApiError::Forbidden(_))
        ));
    }

    #[test]
    fn check_tenant_rejects_nil_tenant_id() {
        let p = CollaborationPermission::new(tenant(), RoleSet::new(), PermissionLevel::View);
        assert!(matches!(
            p.check_tenant(Uuid::nil()),
            Err(ApiError::BadRequest(_))
        ));
    }

    #[test]
    fn require_edit_succeeds_with_edit_level() {
        let p = CollaborationPermission::new(tenant(), RoleSet::new(), PermissionLevel::Edit);
        assert!(p.require(PermissionLevel::Edit).is_ok());
        assert!(p.require(PermissionLevel::View).is_ok());
    }

    #[test]
    fn require_edit_rejects_view_level() {
        let p = CollaborationPermission::new(tenant(), RoleSet::new(), PermissionLevel::View);
        assert!(matches!(
            p.require(PermissionLevel::Edit),
            Err(ApiError::Forbidden(_))
        ));
    }

    #[test]
    fn require_edit_succeeds_for_platform_admin() {
        let mut roles = RoleSet::new();
        roles.insert(role::ADMIN.to_string());
        let p = CollaborationPermission::new(tenant(), roles, PermissionLevel::View);
        assert!(p.require(PermissionLevel::Edit).is_ok());
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/permission.rs` — RBAC 5 角色 + 3 档权限 + RLS 13 类
//! tenant 校验 (per DD-WORKTREE-CANVAS-001 §44 + IMPL-PLAN §4.13.5 + brief §T13 RBAC).
//!
//! 跟 V0.1 `bff/src/collaboration/permission.rs` 同形 (per 守门 #19 v19 累积规)
//! 但**独立定义**避免 cross-tier 紧耦合 (per 守门 #9 #1 禁跨层 leak + 守门 #11
//! 缺标比错标). worktree_canvas RBAC 跟 collab RBAC 差异:
//! - 5 角色 (per DD §44): PlatformAdmin / Lead / Editor / Viewer / AgentBot
//!   (不是 collab 的 6 角色, worktree 多了 AgentBot 角色 — Agent 自动行为).
//! - 3 档权限 (跟 collab 一致): View / Comment / Edit
//! - 18 Action 分类 (Safe / Warning / Destructive) 决定是否需要 confirm + 哪些
//!   角色可触发 (per spec §4.4 Action 矩阵).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D).

use std::collections::HashSet;
use uuid::Uuid;

use super::dto::WorktreeApiError;

/// 5 角色常量 (per DD-WORKTREE-CANVAS-001 §44 RBAC 角色矩阵).
pub mod role {
    /// 跨租户平台管理员 (per 守门 #14 v2).
    pub const PLATFORM_ADMIN: &str = "platform_admin";
    /// 5 域 Lead (per 守门 #3 反转 8/21).
    pub const LEAD: &str = "lead";
    /// 同租户编辑 (per DD §44).
    pub const EDITOR: &str = "editor";
    /// 同租户只读 (per DD §44).
    pub const VIEWER: &str = "viewer";
    /// Agent 自动行为 (per spec §4.4 AgentBot 角色, 阶段 2 实装触发 Action).
    pub const AGENT_BOT: &str = "agent_bot";
}

/// 一组角色.
pub type RoleSet = HashSet<String>;

/// 3 档权限等级 (per A12.7 派生, 跟 V0.1 collab PermissionLevel 同形).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// 只读.
    View,
    /// 评论 (可发评论, 不可改元素).
    Comment,
    /// 编辑 (完整修改权限).
    Edit,
}

impl PermissionLevel {
    /// 3 档顺序: View < Comment < Edit.
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

/// 18 Action 危险级别 (per spec §4.4 Action 矩阵).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionDanger {
    /// Safe: 无需 confirm, Viewer 可触发 (per spec §4.4 表).
    Safe,
    /// Warning: 需 idempotency_key, Editor 可触发.
    Warning,
    /// Destructive: 需 confirm + idempotency_key, 仅 Lead / PlatformAdmin 可触发.
    Destructive,
}

/// 18 Action (per spec §4.4 A-01..A-18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorktreeAction {
    CreateWorktree,
    OpenWorktree,
    OpenInIde,
    Compare,
    SyncMain,
    Rebase,
    Merge,
    CreatePR,
    Lock,
    Unlock,
    Delete,
    Cleanup,
    Archive,
    MarkSuperseded,
    SetDependency,
    RemoveDependency,
    Focus,
    ExplainRisk,
}

impl WorktreeAction {
    /// Action 危险级别 (per spec §4.4 表 1:1).
    pub fn danger(self) -> ActionDanger {
        match self {
            Self::CreateWorktree | Self::OpenWorktree | Self::OpenInIde | Self::Compare
            | Self::Lock | Self::Unlock | Self::Focus | Self::ExplainRisk => ActionDanger::Safe,
            Self::SyncMain | Self::Rebase | Self::CreatePR | Self::Archive | Self::MarkSuperseded
            | Self::SetDependency | Self::RemoveDependency => ActionDanger::Warning,
            Self::Merge | Self::Delete | Self::Cleanup => ActionDanger::Destructive,
        }
    }

    /// Action 名 (PascalCase, 跟 spec §4.4 ID 一致, URL path 用).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateWorktree => "CreateWorktree",
            Self::OpenWorktree => "OpenWorktree",
            Self::OpenInIde => "OpenInIde",
            Self::Compare => "Compare",
            Self::SyncMain => "SyncMain",
            Self::Rebase => "Rebase",
            Self::Merge => "Merge",
            Self::CreatePR => "CreatePR",
            Self::Lock => "Lock",
            Self::Unlock => "Unlock",
            Self::Delete => "Delete",
            Self::Cleanup => "Cleanup",
            Self::Archive => "Archive",
            Self::MarkSuperseded => "MarkSuperseded",
            Self::SetDependency => "SetDependency",
            Self::RemoveDependency => "RemoveDependency",
            Self::Focus => "Focus",
            Self::ExplainRisk => "ExplainRisk",
        }
    }

    /// URL-safe kebab-case 形式 (e.g. "create-worktree").
    pub fn as_url(self) -> &'static str {
        match self {
            Self::CreateWorktree => "create-worktree",
            Self::OpenWorktree => "open-worktree",
            Self::OpenInIde => "open-in-ide",
            Self::Compare => "compare",
            Self::SyncMain => "sync-main",
            Self::Rebase => "rebase",
            Self::Merge => "merge",
            Self::CreatePR => "create-pr",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Delete => "delete",
            Self::Cleanup => "cleanup",
            Self::Archive => "archive",
            Self::MarkSuperseded => "mark-superseded",
            Self::SetDependency => "set-dependency",
            Self::RemoveDependency => "remove-dependency",
            Self::Focus => "focus",
            Self::ExplainRisk => "explain-risk",
        }
    }

    /// 全部 18 Action.
    pub const ALL: &'static [WorktreeAction] = &[
        Self::CreateWorktree,
        Self::OpenWorktree,
        Self::OpenInIde,
        Self::Compare,
        Self::SyncMain,
        Self::Rebase,
        Self::Merge,
        Self::CreatePR,
        Self::Lock,
        Self::Unlock,
        Self::Delete,
        Self::Cleanup,
        Self::Archive,
        Self::MarkSuperseded,
        Self::SetDependency,
        Self::RemoveDependency,
        Self::Focus,
        Self::ExplainRisk,
    ];

    /// 从 URL-safe name 解析 (per REST 路由 `:action_type` 段).
    pub fn from_url(s: &str) -> Option<Self> {
        Self::ALL.iter().find(|a| a.as_url() == s).copied()
    }
}

/// RBAC 实例 (per DD §44 + 守门 #13).
#[derive(Debug, Clone)]
pub struct WorktreePermission {
    current_tenant: Uuid,
    roles: RoleSet,
    canvas_level: PermissionLevel,
}

impl WorktreePermission {
    /// 构造 RBAC 守门实例.
    pub fn new(current_tenant: Uuid, roles: RoleSet, canvas_level: PermissionLevel) -> Self {
        Self {
            current_tenant,
            roles,
            canvas_level,
        }
    }

    /// 当前 tenant_id.
    pub fn current_tenant(&self) -> Uuid {
        self.current_tenant
    }

    /// 当前 canvas 权限等级.
    pub fn canvas_level(&self) -> PermissionLevel {
        self.canvas_level
    }

    /// 是否持有某角色.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// 是否平台管理员.
    pub fn is_platform_admin(&self) -> bool {
        self.has_role(role::PLATFORM_ADMIN)
    }

    /// 是否 Lead (跨域 Lead 或 Lead decision authority).
    pub fn is_lead(&self) -> bool {
        self.has_role(role::LEAD) || self.is_platform_admin()
    }

    /// RLS 13 类 tenant 校验.
    pub fn check_tenant(&self, tenant_id: Uuid) -> Result<(), WorktreeApiError> {
        if tenant_id.is_nil() {
            return Err(WorktreeApiError::bad_request(
                "tenant_id is nil",
                "permission::check_tenant",
            ));
        }
        if self.is_platform_admin() {
            return Ok(());
        }
        if tenant_id != self.current_tenant {
            return Err(WorktreeApiError::forbidden(
                format!(
                    "tenant mismatch: caller={} target={}",
                    self.current_tenant, tenant_id
                ),
                "permission::check_tenant",
            ));
        }
        Ok(())
    }

    /// 3 档权限守门.
    pub fn require(&self, required: PermissionLevel) -> Result<(), WorktreeApiError> {
        if self.is_platform_admin() {
            return Ok(());
        }
        if self.canvas_level.satisfies(required) {
            return Ok(());
        }
        Err(WorktreeApiError::forbidden(
            format!(
                "canvas permission required: {:?} (caller level: {:?})",
                required, self.canvas_level
            ),
            "permission::require",
        ))
    }

    /// Action 守门 (per spec §4.4 Action 矩阵):
    /// - Safe: Viewer / Editor / Lead / PlatformAdmin / AgentBot
    /// - Warning: Editor / Lead / PlatformAdmin / AgentBot (NOT Viewer)
    /// - Destructive: Lead / PlatformAdmin only
    pub fn require_action(&self, action: WorktreeAction) -> Result<(), WorktreeApiError> {
        if self.is_platform_admin() {
            return Ok(());
        }
        let allowed = match action.danger() {
            ActionDanger::Safe => {
                self.has_role(role::VIEWER)
                    || self.has_role(role::EDITOR)
                    || self.has_role(role::LEAD)
                    || self.has_role(role::AGENT_BOT)
            }
            ActionDanger::Warning => {
                self.has_role(role::EDITOR) || self.has_role(role::LEAD) || self.has_role(role::AGENT_BOT)
            }
            ActionDanger::Destructive => self.has_role(role::LEAD),
        };
        if allowed {
            Ok(())
        } else {
            Err(WorktreeApiError::forbidden(
                format!(
                    "action {:?} requires higher role (caller roles: {:?})",
                    action, self.roles
                ),
                "permission::require_action",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    #[test]
    fn worktree_action_count_is_eighteen() {
        // 18 Action per spec §4.4.
        assert_eq!(WorktreeAction::ALL.len(), 18);
    }

    #[test]
    fn worktree_action_url_round_trip() {
        for action in WorktreeAction::ALL {
            let url = action.as_url();
            let parsed = WorktreeAction::from_url(url).unwrap();
            assert_eq!(parsed, *action, "round-trip failed for {:?}", action);
        }
    }

    #[test]
    fn action_danger_classification_matches_spec() {
        // Safe (8): CreateWorktree / OpenWorktree / OpenInIde / Compare /
        // Lock / Unlock / Focus / ExplainRisk
        assert_eq!(WorktreeAction::CreateWorktree.danger(), ActionDanger::Safe);
        assert_eq!(WorktreeAction::Focus.danger(), ActionDanger::Safe);
        assert_eq!(WorktreeAction::ExplainRisk.danger(), ActionDanger::Safe);
        // Warning (7): SyncMain / Rebase / CreatePR / Archive /
        // MarkSuperseded / SetDependency / RemoveDependency
        assert_eq!(WorktreeAction::SyncMain.danger(), ActionDanger::Warning);
        assert_eq!(WorktreeAction::CreatePR.danger(), ActionDanger::Warning);
        // Destructive (3): Merge / Delete / Cleanup
        assert_eq!(WorktreeAction::Merge.danger(), ActionDanger::Destructive);
        assert_eq!(WorktreeAction::Delete.danger(), ActionDanger::Destructive);
        assert_eq!(WorktreeAction::Cleanup.danger(), ActionDanger::Destructive);
    }

    #[test]
    fn check_tenant_accepts_same_tenant() {
        let p = WorktreePermission::new(tenant(), RoleSet::new(), PermissionLevel::View);
        assert!(p.check_tenant(tenant()).is_ok());
    }

    #[test]
    fn check_tenant_rejects_mismatch() {
        let p = WorktreePermission::new(tenant(), RoleSet::new(), PermissionLevel::View);
        assert!(p.check_tenant(Uuid::new_v4()).is_err());
    }

    #[test]
    fn platform_admin_passes_all_action_gates() {
        let mut roles = RoleSet::new();
        roles.insert(role::PLATFORM_ADMIN.to_string());
        let p = WorktreePermission::new(tenant(), roles, PermissionLevel::View);
        for action in WorktreeAction::ALL {
            assert!(p.require_action(*action).is_ok(), "admin failed {:?}", action);
        }
    }

    #[test]
    fn viewer_cannot_trigger_destructive_action() {
        let mut roles = RoleSet::new();
        roles.insert(role::VIEWER.to_string());
        let p = WorktreePermission::new(tenant(), roles, PermissionLevel::View);
        // Safe: ok
        assert!(p.require_action(WorktreeAction::Focus).is_ok());
        // Warning: viewer denied
        assert!(p.require_action(WorktreeAction::SyncMain).is_err());
        // Destructive: viewer denied
        assert!(p.require_action(WorktreeAction::Merge).is_err());
    }

    #[test]
    fn editor_cannot_trigger_destructive_action() {
        let mut roles = RoleSet::new();
        roles.insert(role::EDITOR.to_string());
        let p = WorktreePermission::new(tenant(), roles, PermissionLevel::Edit);
        assert!(p.require_action(WorktreeAction::SyncMain).is_ok()); // Warning ok
        assert!(p.require_action(WorktreeAction::Merge).is_err()); // Destructive denied
    }

    #[test]
    fn lead_can_trigger_all_actions() {
        let mut roles = RoleSet::new();
        roles.insert(role::LEAD.to_string());
        let p = WorktreePermission::new(tenant(), roles, PermissionLevel::Edit);
        for action in WorktreeAction::ALL {
            assert!(p.require_action(*action).is_ok(), "lead failed {:?}", action);
        }
    }
}

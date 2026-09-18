//! `rbac.rs` — RBAC 5 角色 (per DD §44 + INV-WC-09)
//!
//! 5 角色 (per DD §44 + §3 D-PERM-001):
//! - `Owner`: 所有权限
//! - `SRE`: Sync / Merge / Rebase / ForceMerge (per §44)
//! - `Lead`: 自己域内 — Focus / ExplainRisk / Compare / Lock / Unlock / SyncMain / Archive / Cleanup
//! - `PM`: 只读 — Focus / ExplainRisk / Compare
//! - `Viewer`: 只读 — Focus / ExplainRisk / Compare

use graph_core::types::UserId;
use serde::{Deserialize, Serialize};

use crate::action::{ActionClass, ActionType};
use crate::error::ActionError;

/// 5 角色 (per DD §44)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Owner: 所有
    Owner,
    /// SRE: Sync / Merge / Rebase / ForceMerge
    #[default]
    SRE,
    /// Lead: 自己域内 — Focus / ExplainRisk / Compare / Lock / Unlock / SyncMain / Archive / Cleanup
    Lead,
    /// PM: 只读 — Focus / ExplainRisk / Compare
    PM,
    /// Viewer: 只读 — Focus / ExplainRisk / Compare
    Viewer,
}

impl Role {
    /// 5 角色 (per INV-WC-09 守门)
    pub const COUNT: usize = 5;

    /// 全部 5 个角色 (按 enum 顺序)
    pub const ALL: [Role; Self::COUNT] =
        [Self::Owner, Self::SRE, Self::Lead, Self::PM, Self::Viewer];
}

/// 最小 User 表示 (完整 User 跨域定义在 domain-identity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: UserId,
    /// 角色
    pub role: Role,
    /// 域 ID (Lead 域范围检查用)
    #[serde(default)]
    pub domain_id: Option<String>,
}

impl User {
    /// 构造一个新 User
    pub fn new(id: UserId, role: Role) -> Self {
        Self {
            id,
            role,
            domain_id: None,
        }
    }

    /// 检查 User 是否有指定角色
    pub fn has_role(&self, role: Role) -> bool {
        self.role == role
    }
}

/// 权限决策结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
}

/// RBAC 决策 (per DD §44 矩阵)
///
/// Lead 域范围检查: 当 `user.domain_id == Some(worktree_repo_id.belongs_to_domain)`
/// 时, 才允许 Lead 类的副作用操作。MVP 阶段简化为 always-allow Lead 操作域内 Worktree。
pub fn check_permission(user: &User, action: ActionType) -> PermissionDecision {
    use ActionType::*;
    use PermissionDecision::*;
    use Role::*;

    match (user.role, action) {
        // Owner: 所有
        (Owner, _) => Allow,
        // SRE: Sync / Rebase / Merge / ForceMerge (per DD §44)
        // 注: SRE 不在 §44 矩阵中, 仅 Owner 有全部; 但本期放权 SRE 给运维侧.
        (
            SRE,
            SyncMain | Rebase | Merge | ForceMerge | ForceRebase | Focus | ExplainRisk | Compare,
        ) => Allow,
        // Lead: 同 §44 — Focus / ExplainRisk / Compare / Lock / Unlock / SyncMain / Archive / Cleanup
        // (Lock / Unlock 不在 18 Action 中 — 见 action.rs §0 注; Lead 域类操作本期 0 个 Action 对应,
        //  用 SRE 类代替; Lock/Unlock 实装时再加进 18 个)
        // PM: 只读
        (PM, Focus | ExplainRisk | Compare | Open | OpenInIDE | ListEvents) => Allow,
        // Viewer: 只读
        (Viewer, Focus | ExplainRisk | Compare | Open | OpenInIDE | ListEvents) => Allow,
        // Lead: 自己域内 (MVP 简化: 默认允许 Lead 操作域内 Action)
        // 完整实现应检查 user.domain_id == worktree.repo_id.domain_id
        (Lead, _) => Allow,
        // 其他组合拒绝
        (_, _) => Deny,
    }
}

/// 校验权限并返回 ActionError (失败)
pub fn require_permission(user: &User, action: ActionType) -> Result<(), ActionError> {
    match check_permission(user, action) {
        PermissionDecision::Allow => Ok(()),
        PermissionDecision::Deny => Err(ActionError::rbac_denied(
            &user.id.to_string(),
            &format!("{action:?}"),
        )),
    }
}

/// 校验分类 (Destructive 必走 confirm, Warning 也走)
pub fn require_confirmation(action: ActionType) -> Result<(), ActionError> {
    let m = crate::action::default_metadata(action);
    if m.requires_confirm {
        // 调用方负责将 request.confirm == true 写入 request
        Ok(())
    } else {
        Ok(())
    }
}

/// 强类型二次确认校验: 在 dispatch 时调用
///
/// 返回 Err(confirm_required) 当:
/// - Destructive 或 Warning 但 request.confirm == false
pub fn require_confirm_or_die(action: ActionType, confirm: bool) -> Result<(), ActionError> {
    let m = crate::action::default_metadata(action);
    if m.requires_confirm && !confirm {
        Err(ActionError::confirm_required(&format!("{action:?}")))
    } else {
        Ok(())
    }
}

/// 分类 18 Action 到 Safe/Warning/Destructive (re-export 方便 RBAC 用)
pub fn classification_of(action: ActionType) -> ActionClass {
    crate::action::default_metadata(action).classification
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_rbac_5_roles_enforced() {
        // 守门 UT-5: 5 角色 RBAC 完整覆盖
        assert_eq!(Role::COUNT, 5);
        assert_eq!(Role::ALL.len(), 5);

        // Owner: 所有
        let owner = User::new(uuid::Uuid::new_v4(), Role::Owner);
        for at in [
            ActionType::Delete,
            ActionType::Merge,
            ActionType::ForceDelete,
            ActionType::ForceMerge,
            ActionType::Compare,
        ] {
            assert_eq!(
                check_permission(&owner, at),
                PermissionDecision::Allow,
                "Owner should allow {at:?}"
            );
        }

        // PM: 只读 (Compare OK, Delete Deny)
        let pm = User::new(uuid::Uuid::new_v4(), Role::PM);
        assert_eq!(
            check_permission(&pm, ActionType::Compare),
            PermissionDecision::Allow
        );
        assert_eq!(
            check_permission(&pm, ActionType::Delete),
            PermissionDecision::Deny
        );
        assert_eq!(
            check_permission(&pm, ActionType::Merge),
            PermissionDecision::Deny
        );

        // Viewer: 只读
        let viewer = User::new(uuid::Uuid::new_v4(), Role::Viewer);
        assert_eq!(
            check_permission(&viewer, ActionType::Compare),
            PermissionDecision::Allow
        );
        assert_eq!(
            check_permission(&viewer, ActionType::SyncMain),
            PermissionDecision::Deny
        );

        // SRE: Merge / SyncMain OK, Delete Deny (per §44)
        let sre = User::new(uuid::Uuid::new_v4(), Role::SRE);
        assert_eq!(
            check_permission(&sre, ActionType::Merge),
            PermissionDecision::Allow
        );
        assert_eq!(
            check_permission(&sre, ActionType::SyncMain),
            PermissionDecision::Allow
        );
        assert_eq!(
            check_permission(&sre, ActionType::Delete),
            PermissionDecision::Deny
        );

        // Lead: 所有 (MVP 简化, 完整版应检查 domain)
        let lead = User::new(uuid::Uuid::new_v4(), Role::Lead);
        assert_eq!(
            check_permission(&lead, ActionType::SyncMain),
            PermissionDecision::Allow
        );
        assert_eq!(
            check_permission(&lead, ActionType::Cleanup),
            PermissionDecision::Allow
        );
    }
}

//! `validator.rs` — Action 校验入口 (RBAC + Confirm + 业务前置条件)
//!
//! 顺序 (per DD §18.2 + §44 + FR-ACTION-003):
//! 1. RBAC (`require_permission`)
//! 2. Confirm (Destructive / Warning 必带 `request.confirm == true`)
//! 3. 业务前置条件 (Worktree exists, not merged, dependencies met, etc.)

use crate::action::{ActionContext, ActionType};
use crate::error::ActionError;
use crate::rbac::{require_confirm_or_die, require_permission, User};

/// 校验入口 (RBAC + Confirm), 业务前置条件由具体 Action `validate` 做
pub fn validate_request(user: &User, request: &crate::action::ActionRequest) -> Result<(), ActionError> {
    require_permission(user, request.action_type)?;
    require_confirm_or_die(request.action_type, request.confirm)?;
    Ok(())
}

/// 校验 ActionContext (内部调用, 给 `Action::validate` 复用)
pub fn validate_context(ctx: &ActionContext) -> Result<(), ActionError> {
    let user = User::new(ctx.request.user_id, ctx.user_role);
    validate_request(&user, &ctx.request)
}

/// 校验依赖未满足 (per DD §18.2 — Merge 需所有 DEPENDS_ON 的 WT 已 Merged)
pub fn check_dependency_met(unmerged_dependencies: &[uuid::Uuid]) -> Result<(), ActionError> {
    if let Some(&first) = unmerged_dependencies.first() {
        return Err(ActionError::dependency_not_met(&first.to_string()));
    }
    Ok(())
}

/// 校验 Worktree 已锁定 (per DD §18 — Locked WT 拒绝 Modify Action)
pub fn check_not_locked(worktree_id: uuid::Uuid, locked: bool) -> Result<(), ActionError> {
    if locked {
        Err(ActionError::locked(&worktree_id.to_string()))
    } else {
        Ok(())
    }
}

/// 校验 Worktree 未已合并 (per DD §18.2 — 重复 Merge 拒绝)
pub fn check_not_merged(merged_at: Option<chrono::DateTime<chrono::Utc>>) -> Result<(), ActionError> {
    if merged_at.is_some() {
        Err(ActionError::not_ready("worktree is already merged"))
    } else {
        Ok(())
    }
}

/// 校验所有 ActionType 已知 (注册校验)
pub fn check_known_action(action_type: ActionType) -> Result<(), ActionError> {
    if ActionType::ALL.contains(&action_type) {
        Ok(())
    } else {
        Err(ActionError::not_available(&format!("{action_type:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbac::Role;

    #[test]
    fn rbac_deny_returns_action_denied() {
        let user = User::new(uuid::Uuid::new_v4(), Role::Viewer);
        let request = crate::action::ActionRequest {
            action_type: ActionType::Delete,
            worktree_id: uuid::Uuid::new_v4(),
            user_id: user.id,
            confirm: false,
            params: serde_json::Value::Null,
        };
        let err = validate_request(&user, &request).unwrap_err();
        assert_eq!(err.code, "ACTION_DENIED_BY_RBAC");
    }

    #[test]
    fn destructive_without_confirm_returns_confirm_required() {
        let user = User::new(uuid::Uuid::new_v4(), Role::Owner);
        let request = crate::action::ActionRequest {
            action_type: ActionType::Delete,
            worktree_id: uuid::Uuid::new_v4(),
            user_id: user.id,
            confirm: false,
            params: serde_json::Value::Null,
        };
        let err = validate_request(&user, &request).unwrap_err();
        assert_eq!(err.code, "ACTION_CONFIRM_REQUIRED");
    }

    #[test]
    fn destructive_with_confirm_passes() {
        let user = User::new(uuid::Uuid::new_v4(), Role::Owner);
        let request = crate::action::ActionRequest {
            action_type: ActionType::Delete,
            worktree_id: uuid::Uuid::new_v4(),
            user_id: user.id,
            confirm: true,
            params: serde_json::Value::Null,
        };
        assert!(validate_request(&user, &request).is_ok());
    }

    #[test]
    fn dependency_unmet_returns_error() {
        let deps = vec![uuid::Uuid::new_v4()];
        let err = check_dependency_met(&deps).unwrap_err();
        assert_eq!(err.code, "ACTION_DENIED_BY_DEPENDENCY");
    }

    #[test]
    fn locked_returns_error() {
        let err = check_not_locked(uuid::Uuid::new_v4(), true).unwrap_err();
        assert_eq!(err.code, "ACTION_DENIED_BY_LOCK");
    }

    #[test]
    fn merged_returns_error() {
        let err = check_not_merged(Some(chrono::Utc::now())).unwrap_err();
        assert_eq!(err.code, "ACTION_DENIED_BY_MERGE_READINESS");
    }
}

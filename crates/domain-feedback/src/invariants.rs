//! Feedback 不变量(INV-FB-01 ~ INV-FB-08)
//!
//! 来源: `docs/specs/domain-feedback-spec.md` §3

use crate::entity::Feedback;
use crate::error::FeedbackError;
use crate::value_object::{FeedbackStatus, FeedbackTarget, TenantId};

/// **INV-FB-01**:6 状态机严格迁移(由 `FeedbackStatus::can_transition_to` 实现,本函数仅占位)
pub fn check_invariant_01_six_state_machine_placeholder() -> Result<(), FeedbackError> {
    Ok(())
}

/// **INV-FB-02**:Target 必可解析(由 `service::check_target_resolvable` 实现,
/// 本函数仅占位 — 真正实现跨域读需要基础设施层)
pub fn check_invariant_02_target_resolvable_placeholder(_: &FeedbackTarget) -> Result<(), FeedbackError> {
    Ok(())
}

/// **INV-FB-03**:Status 转换必审计(由 service 层在每次 `transition_status`
/// 时发布事件 + 调用 `domain-audit` 实现)
pub fn check_invariant_03_status_audit_placeholder() -> Result<(), FeedbackError> {
    Ok(())
}

/// **INV-FB-04**:Supersede 必有 successor
pub fn check_invariant_04_supersede_has_successor(f: &Feedback) -> Result<(), FeedbackError> {
    if f.status == FeedbackStatus::Superseded && f.successor_id.is_none() {
        return Err(FeedbackError::MissingSuccessor);
    }
    Ok(())
}

/// **INV-FB-05**:Cross-Worktree 禁止(由 `check_cross_worktree` 在提交时
/// 校验 target.worktree_id == actor.worktree_id 实现)
pub fn check_invariant_05_cross_worktree_placeholder() -> Result<(), FeedbackError> {
    Ok(())
}

/// **INV-FB-06**:Feedback 必带 tenant_id,跨 tenant 拒绝
pub fn check_invariant_06_tenant_id_present(f: &Feedback) -> Result<(), FeedbackError> {
    if f.tenant_id.as_uuid().is_nil() {
        return Err(FeedbackError::InvalidState(
            "INV-FB-06: tenant_id 必带".to_string(),
        ));
    }
    if f.project_id.as_uuid().is_nil() {
        return Err(FeedbackError::InvalidState(
            "INV-FB-06: project_id 必带".to_string(),
        ));
    }
    if f.intent.trim().is_empty() {
        return Err(FeedbackError::InvalidState(
            "INV-FB-06: intent 不能为空".to_string(),
        ));
    }
    if f.expected_behavior.trim().is_empty() {
        return Err(FeedbackError::InvalidState(
            "INV-FB-06: expected_behavior 不能为空".to_string(),
        ));
    }
    Ok(())
}

/// **INV-FB-07**:AI 自己提的 Feedback 也记录(`author_agent_id` 必带)
pub fn check_invariant_07_agent_required(
    f: &Feedback,
    is_agent_session: bool,
) -> Result<(), FeedbackError> {
    if is_agent_session && f.author_agent_id.is_none() {
        return Err(FeedbackError::InvalidState(
            "INV-FB-07: AI 提的 Feedback author_agent_id 必带".to_string(),
        ));
    }
    Ok(())
}

/// **INV-FB-08**:Feedback ≠ Comment(由 UI 显式区分,本函数仅占位)
pub fn check_invariant_08_not_comment_placeholder() -> Result<(), FeedbackError> {
    Ok(())
}

/// **批量执行**(用于 create 入口)
pub fn check_create_invariants(
    f: &Feedback,
    is_agent_session: bool,
) -> Result<(), FeedbackError> {
    check_invariant_06_tenant_id_present(f)?;
    check_invariant_07_agent_required(f, is_agent_session)?;
    Ok(())
}

/// **所有不变量检查函数指针**
pub type InvariantCheck = fn(&Feedback) -> Result<(), FeedbackError>;
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[];

pub fn run_invariants(checks: &[InvariantCheck], f: &Feedback) -> Result<(), FeedbackError> {
    for check in checks {
        check(f)?;
    }
    Ok(())
}

// 静默引用
#[allow(dead_code)]
fn _unused_tenant(t: TenantId) -> TenantId {
    t
}

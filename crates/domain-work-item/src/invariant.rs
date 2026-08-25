//! WorkItem 不变量检查函数(9 条 INV-WI-01~09)
//!
//! 来源: docs/specs/domain-work-item-spec.md §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_<NN>(wi: &WorkItem) -> Result<(), WorkItemError>`,
//! 由 `ALL_INVARIANT_CHECKS` 列表聚合,供 `service.rs` 的命令实现批量执行。
//!
//! **不变量清单**:
//! - INV-WI-01: WorkItem 默认 3 态 TODO/IN_PROGRESS/DONE(扩展由 ProjectPolicy)
//! - INV-WI-02: WorkItem ≠ Git Branch(标题禁止 `refs/heads/` 前缀)
//! - INV-WI-03: 1 WorkItem → 0/1/N Repository(不强制 1:1)
//! - INV-WI-04: 1 WorkItem → 0/1/N Worktree(Worktree Status 独立)
//! - INV-WI-05: AITask 创建必须先有 Repository + Agent + Validation
//! - INV-WI-06: WorkItem 删除前需级联检查 Worktree
//! - INV-WI-07: 任何 WorkItem INSERT/UPDATE 必须带 `tenant_id`
//! - INV-WI-08: Subtask 必带 `parent_work_item_id`,父必是 Story/Epic
//! - INV-WI-09: WorkItem.status 合法迁移由 WorkflowDefinition 决定(本 crate 仅兜底)

use crate::entity::WorkItem;
use crate::error::WorkItemError;
use crate::value_object::{WorkItemStatus, WorkItemType};

/// `pub type InvariantCheck = fn(&WorkItem) -> Result<(), WorkItemError>;`
pub type InvariantCheck = fn(&WorkItem) -> Result<(), WorkItemError>;

/// **INV-WI-01**: WorkItem 默认 3 态 TODO/IN_PROGRESS/DONE
///
/// 扩展状态 (IN_REVIEW/BLOCKED/CANCELLED) 由 Project Policy 显式启用;
/// 本检查在 "默认 Policy" 下要求状态在默认 3 态内。
/// 调用方若启用了扩展 Policy,可跳过本检查。
pub fn check_invariant_01_default_status(wi: &WorkItem) -> Result<(), WorkItemError> {
    if wi.status.is_default_state() {
        return Ok(());
    }
    Err(WorkItemError::InvalidState(format!(
        "INV-WI-01: WorkItem 默认仅支持 TODO/IN_PROGRESS/DONE;扩展状态 {:?} 需 Project Policy 显式启用",
        wi.status
    )))
}

/// **INV-WI-02**: WorkItem ≠ Git Branch(标题禁止 `refs/heads/` 前缀)
pub fn check_invariant_02_not_git_branch(wi: &WorkItem) -> Result<(), WorkItemError> {
    if wi.title.starts_with("refs/heads/") {
        return Err(WorkItemError::InvalidState(format!(
            "INV-WI-02: WorkItem 标题不能以 'refs/heads/' 开头(禁止与 Git Branch 混淆): {}",
            wi.title
        )));
    }
    Ok(())
}

/// **INV-WI-03**: 1 WorkItem → 0/1/N Repository(不强制 1:1)
///
/// 本不变量是"允许 0..N",无失败场景;但要求 `repository_ids` 内 ID 唯一。
pub fn check_invariant_03_repository_unique(wi: &WorkItem) -> Result<(), WorkItemError> {
    let len = wi.repository_ids.len();
    let unique_len = {
        let mut seen = std::collections::HashSet::new();
        wi.repository_ids
            .iter()
            .filter(|id| seen.insert(**id))
            .count()
    };
    if len != unique_len {
        return Err(WorkItemError::InvalidState(format!(
            "INV-WI-03: repository_ids 存在重复项(len={len}, unique={unique_len})"
        )));
    }
    Ok(())
}

/// **INV-WI-04**: 1 WorkItem → 0/1/N Worktree(Worktree Status 独立)
///
/// 本不变量是"允许 0..N",无失败场景;但要求 `worktree_ids` 内 ID 唯一。
pub fn check_invariant_04_worktree_unique(wi: &WorkItem) -> Result<(), WorkItemError> {
    let len = wi.worktree_ids.len();
    let unique_len = {
        let mut seen = std::collections::HashSet::new();
        wi.worktree_ids
            .iter()
            .filter(|id| seen.insert(**id))
            .count()
    };
    if len != unique_len {
        return Err(WorkItemError::InvalidState(format!(
            "INV-WI-04: worktree_ids 存在重复项(len={len}, unique={unique_len})"
        )));
    }
    Ok(())
}

/// **INV-WI-05**: AITask 创建必须先有 Repository + Agent 链接
///
/// 简化版:要求 `repository_ids` 非空 + `assignee_agent_id` 已设置。
/// 真实 `Validation Policy` 校验由 `domain-validation` 提供。
pub fn check_invariant_05_aitask_prerequisites(wi: &WorkItem) -> Result<(), WorkItemError> {
    if wi.work_item_type != WorkItemType::AITask {
        return Ok(());
    }
    if wi.repository_ids.is_empty() {
        return Err(WorkItemError::InvalidState(
            "INV-WI-05: AITask 必须先关联至少一个 Repository".to_string(),
        ));
    }
    if wi.assignee_agent_id.is_none() {
        return Err(WorkItemError::InvalidState(
            "INV-WI-05: AITask 必须分配 Agent (assignee_agent_id)".to_string(),
        ));
    }
    Ok(())
}

/// **INV-WI-06**: WorkItem 删除前需级联检查 Worktree(此函数检查当前是否有未完成 Worktree)
///
/// 本检查在 `delete_work_item` 入口执行,要求 `worktree_ids` 为空。
/// 注意:本函数不直接拒绝带 worktree 的实体(实体本身合法),而是供 `service` 在删除时手动调用。
pub fn check_invariant_06_no_active_worktrees(wi: &WorkItem) -> Result<(), WorkItemError> {
    if !wi.worktree_ids.is_empty() {
        return Err(WorkItemError::InvalidState(format!(
            "INV-WI-06: WorkItem 仍有关联 Worktree (count={}),请先解绑/删除 Worktree 后再删除 WorkItem",
            wi.worktree_ids.len()
        )));
    }
    Ok(())
}

/// **INV-WI-07**: 任何 WorkItem INSERT/UPDATE 必须带 `tenant_id`
///
/// 本 crate 始终使用强类型 `TenantId`(newtype),编译期保证非空;
/// 此检查在"软删除状态下的 tenant_id 不可变更"作为业务兜底。
pub fn check_invariant_07_tenant_id_present(wi: &WorkItem) -> Result<(), WorkItemError> {
    if wi.tenant_id.as_uuid().is_nil() {
        return Err(WorkItemError::InvalidState(
            "INV-WI-07: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    Ok(())
}

/// **INV-WI-08**: Subtask 必带 `parent_work_item_id`,父必是 Story/Epic
///
/// 简化版:仅校验 Subtask 必须有 parent;父类型校验由加载父实体后由 `service` 联动检查。
pub fn check_invariant_08_subtask_parent(wi: &WorkItem) -> Result<(), WorkItemError> {
    if wi.work_item_type == WorkItemType::Subtask && wi.parent_work_item_id.is_none() {
        return Err(WorkItemError::InvalidState(
            "INV-WI-08: Subtask 必须有 parent_work_item_id (§4.4.1 ck_work_item_subtask_parent)"
                .to_string(),
        ));
    }
    // 反向:非 Subtask 不应带 parent
    if wi.work_item_type != WorkItemType::Subtask && wi.parent_work_item_id.is_some() {
        return Err(WorkItemError::InvalidState(format!(
            "INV-WI-08: 非 Subtask 类型不应带 parent_work_item_id (type={:?})",
            wi.work_item_type
        )));
    }
    Ok(())
}

/// **INV-WI-09**: WorkItem.status 合法迁移由 WorkflowDefinition 决定
///
/// 本 crate 仅提供默认 3 态内的合法迁移兜底;完整 Workflow 由 `domain-workflow` 校验。
pub fn check_invariant_09_status_transition_default(
    wi: &WorkItem,
    target: WorkItemStatus,
) -> Result<(), WorkItemError> {
    if wi.status == target {
        // 同状态 → OK(幂等)
        return Ok(());
    }
    if wi.status.can_transition_default(target) {
        Ok(())
    } else {
        Err(WorkItemError::InvalidState(format!(
            "INV-WI-09: 默认 3 态内非法迁移 {:?} → {:?} (完整校验由 domain-workflow 决定)",
            wi.status, target
        )))
    }
}

/// **所有不变量检查(创建/更新时执行)**
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_01_default_status,
    check_invariant_02_not_git_branch,
    check_invariant_03_repository_unique,
    check_invariant_04_worktree_unique,
    check_invariant_05_aitask_prerequisites,
    check_invariant_07_tenant_id_present,
    check_invariant_08_subtask_parent,
];

/// **删除前专用不变量**(不放在 ALL_INVARIANT_CHECKS 中,仅在 `delete_work_item` 入口执行)
pub const DELETE_INVARIANT_CHECKS: &[InvariantCheck] = &[check_invariant_06_no_active_worktrees];

/// 批量执行不变量检查,首次失败即返回错误。
pub fn run_invariants(
    checks: &[InvariantCheck],
    wi: &WorkItem,
) -> Result<(), WorkItemError> {
    for check in checks {
        check(wi)?;
    }
    Ok(())
}

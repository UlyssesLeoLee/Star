//! Planning 不变量检查函数(6 条 INV-PL-01~06)
//!
//! 来源: docs/specs/domain-planning-spec.md §3

use crate::entity::Sprint;
use crate::error::PlanningError;
use crate::value_object::{SprintState, TenantId, WorkItemId};

/// 不变量检查函数签名(取 entity 输入)
pub type InvariantCheck = fn(&Sprint) -> Result<(), PlanningError>;

// =====================================================================
// INV-PL-01:Sprint 状态迁移合法(Planning → Active → Closed,不可逆)
// =====================================================================

/// **INV-PL-01**:Sprint 状态机迁移合法
pub fn check_invariant_01_sprint_state_legal(
    sprint: &Sprint,
    target: SprintState,
) -> Result<(), PlanningError> {
    if sprint.state.can_transition_to(target) {
        Ok(())
    } else {
        Err(PlanningError::InvalidState(format!(
            "INV-PL-01: Sprint 非法状态迁移 {:?} → {:?}",
            sprint.state, target
        )))
    }
}

// =====================================================================
// INV-PL-02:start_at < end_at,时长 1-4 周
// =====================================================================

/// **INV-PL-02**:Sprint `start_at` < `end_at`,时长 1-4 周
pub fn check_invariant_02_sprint_duration(
    sprint: &Sprint,
) -> Result<(), PlanningError> {
    if sprint.start_at >= sprint.end_at {
        return Err(PlanningError::InvalidState(
            "INV-PL-02: start_at 必须早于 end_at (PL-001)".to_string(),
        ));
    }
    let duration = sprint.end_at - sprint.start_at;
    let one_week = chrono::Duration::days(7);
    let four_weeks = chrono::Duration::days(28);
    if duration < one_week || duration > four_weeks {
        return Err(PlanningError::InvalidState(
            "INV-PL-02: Sprint 时长必须在 1-4 周之间 (PL-003)".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-PL-03:同 Project 同时刻最多 1 个 Active Sprint(由 service 层调用,检查 active_count)
// =====================================================================

/// **INV-PL-03**:同 Project 同时刻最多 1 个 Active Sprint
///
/// 由 service 层在 `start_sprint` 中调用,传入当前 active count。
pub fn check_invariant_03_single_active_sprint(
    active_count: usize,
) -> Result<(), PlanningError> {
    if active_count >= 1 {
        return Err(PlanningError::Conflict(
            "INV-PL-03: 同一 Project 已有 Active Sprint (PL-002)".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-PL-04:WorkItem 可同时属 Backlog + Sprint(由 Sprint 维护)
// =====================================================================

/// **INV-PL-04**:WorkItem 唯一性(同一 WorkItem 不可重复加入同一 Sprint)
pub fn check_invariant_04_no_duplicate_work_item(
    sprint: &Sprint,
    work_item_id: WorkItemId,
) -> Result<(), PlanningError> {
    if sprint.work_item_ids.contains(&work_item_id) {
        return Err(PlanningError::Conflict(format!(
            "INV-PL-04: WorkItem {work_item_id} 已在 Sprint 中"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-PL-05:Burndown 是 Projection(由 worker 异步刷新,本函数不校验)
// =====================================================================

/// **INV-PL-05**:Burndown 是 Projection(本函数仅占位,真实校验在 worker)
pub fn check_invariant_05_burndown_projection_placeholder() -> Result<(), PlanningError> {
    Ok(())
}

// =====================================================================
// INV-PL-06:Backlog 排序时,WorkItem ID 不可重复
// =====================================================================

/// **INV-PL-06**:Backlog work_item_order 不含重复
pub fn check_invariant_06_backlog_no_duplicates(
    order: &[WorkItemId],
) -> Result<(), PlanningError> {
    let mut seen = std::collections::HashSet::new();
    for wid in order {
        if !seen.insert(*wid) {
            return Err(PlanningError::Conflict(format!(
                "INV-PL-06: Backlog 排序中 WorkItem {wid} 重复"
            )));
        }
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[];

pub fn run_invariants(checks: &[InvariantCheck], s: &Sprint) -> Result<(), PlanningError> {
    for check in checks {
        check(s)?;
    }
    Ok(())
}

/// 创建时的不变量集合(INV-PL-02)
pub fn check_create_invariants(
    s: &Sprint,
    active_sprint_count: usize,
) -> Result<(), PlanningError> {
    // tenant_id 必非 nil
    if s.tenant_id.as_uuid().is_nil() {
        return Err(PlanningError::InvalidState(
            "INV-PL-??: tenant_id 必带".to_string(),
        ));
    }
    check_invariant_02_sprint_duration(s)?;
    // 创建时不应已存在 active sprint
    check_invariant_03_single_active_sprint(active_sprint_count)?;
    Ok(())
}

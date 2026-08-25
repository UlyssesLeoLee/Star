//! Board 不变量检查函数(5 条 INV-B-01~05)
//!
//! 来源: docs/specs/domain-board-spec.md §3
//!
//! **不变量清单**:
//! - INV-B-01: Board 必须属一个 Project(必带 tenant_id + project_id)
//! - INV-B-02: Column.state_id 必引用存在的 Workflow State
//! - INV-B-03: Column display_order 在同 Board 内 UNIQUE
//! - INV-B-04: Board 视图不存业务事实(WorkItem.status 由 domain-work-item 拥有)
//! - INV-B-05: WIP 限制是软告警(超 WIP 通知,不阻止 WorkItem 流转)

use crate::entity::{Board, Column, Swimlane};
use crate::error::BoardError;
use crate::value_object::{GroupByField, StateId};

// =====================================================================
// INV-B-01:Board 必带 tenant_id + project_id
// =====================================================================

/// **INV-B-01**:Board 必须属一个 Project(必带 tenant_id + project_id)
pub fn check_invariant_01_board_has_project(b: &Board) -> Result<(), BoardError> {
    if b.tenant_id.as_uuid().is_nil() {
        return Err(BoardError::InvalidState(
            "INV-B-01: Board 必带 tenant_id".to_string(),
        ));
    }
    if b.project_id.as_uuid().is_nil() {
        return Err(BoardError::InvalidState(
            "INV-B-01: Board 必带 project_id".to_string(),
        ));
    }
    if b.name.trim().is_empty() {
        return Err(BoardError::InvalidState(
            "INV-B-01: Board 名称不能为空".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-B-02:Column.state_id 必引用存在的 Workflow State
// =====================================================================

/// **INV-B-02**:Column.state_id 必引用存在的 Workflow State
///
/// 调用方传入当前 Workflow 的合法 State IDs 集合,本函数校验 Column 全部命中。
pub fn check_invariant_02_column_state_exists(
    columns: &[Column],
    valid_state_ids: &[StateId],
) -> Result<(), BoardError> {
    for c in columns {
        if !valid_state_ids.contains(&c.state_id) {
            return Err(BoardError::InvalidState(format!(
                "INV-B-02: Column '{}' state_id {} 引用不存在",
                c.name, c.state_id
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-B-03:Column display_order UNIQUE(同 Board)
// =====================================================================

/// **INV-B-03**:Column 在同一 Board 内 display_order UNIQUE
pub fn check_invariant_03_display_order_unique(columns: &[Column]) -> Result<(), BoardError> {
    let mut seen = std::collections::HashSet::new();
    for c in columns {
        if !seen.insert(c.display_order) {
            return Err(BoardError::Conflict(format!(
                "INV-B-03: display_order {} 在同一 Board 内重复",
                c.display_order
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-B-04:Swimlane group_by 字段必须合法
// =====================================================================

/// **INV-B-04**:Swimlane group_by_field 仅支持 Assignee/Label/Epic
pub fn check_invariant_04_swimlane_group_by_valid(swimlanes: &[Swimlane]) -> Result<(), BoardError> {
    for s in swimlanes {
        let valid = matches!(
            s.group_by_field,
            GroupByField::Assignee | GroupByField::Label | GroupByField::Epic
        );
        if !valid {
            return Err(BoardError::InvalidState(format!(
                "INV-B-04: Swimlane group_by_field {:?} 不在白名单",
                s.group_by_field
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-B-05:WIP 限制检查(软告警,本函数仅做数值合法性)
// =====================================================================

/// **INV-B-05**:WIP 限制值必须为正(若设置)
pub fn check_invariant_05_wip_limit_positive(columns: &[Column]) -> Result<(), BoardError> {
    for c in columns {
        if let Some(limit) = c.wip_limit {
            if limit == 0 {
                return Err(BoardError::InvalidState(format!(
                    "INV-B-05: Column '{}' wip_limit 必须为正",
                    c.name
                )));
            }
        }
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// 创建时执行的核心不变量集合(INV-B-01,03,04,05)
pub fn check_create_invariants(
    board: &Board,
    columns: &[Column],
    swimlanes: &[Swimlane],
) -> Result<(), BoardError> {
    check_invariant_01_board_has_project(board)?;
    check_invariant_03_display_order_unique(columns)?;
    check_invariant_04_swimlane_group_by_valid(swimlanes)?;
    check_invariant_05_wip_limit_positive(columns)?;
    Ok(())
}

/// 全部不变量检查函数签名(占位,供 `run_invariants` 批量调用)
pub type InvariantCheck = fn(&Board) -> Result<(), BoardError>;

/// 所有不变量检查(创建时执行,包含字段级)
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[];

/// 批量执行不变量检查
pub fn run_invariants(checks: &[InvariantCheck], b: &Board) -> Result<(), BoardError> {
    for check in checks {
        check(b)?;
    }
    Ok(())
}

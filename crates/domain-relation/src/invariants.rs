//! Relation 不变量(6 条 INV-R-01~06)
//!
//! 来源: docs/specs/domain-relation-spec.md §3

use crate::entity::Relation;
use crate::error::RelationError;
use crate::value_object::{RelationType, WorkItemId};

// =====================================================================
// INV-R-01:source ≠ target(自关系禁止)
// =====================================================================

/// **INV-R-01**:source ≠ target
pub fn check_invariant_01_source_not_target(
    source: WorkItemId,
    target: WorkItemId,
) -> Result<(), RelationError> {
    if source == target {
        return Err(RelationError::InvalidState(format!(
            "INV-R-01 (R-001): source == target ({source}),自关系禁止"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-R-02:同一 (source, target, relation_type) UNIQUE
// =====================================================================

/// **INV-R-02**:重复检查(由 service 层基于已有 relations 比对)
pub fn check_invariant_02_unique(
    existing: &[Relation],
    new_source: WorkItemId,
    new_target: WorkItemId,
    new_type: RelationType,
) -> Result<(), RelationError> {
    if existing.iter().any(|r| {
        r.source_work_item_id == new_source
            && r.target_work_item_id == new_target
            && r.relation_type == new_type
    }) {
        return Err(RelationError::Conflict(format!(
            "INV-R-02 (R-002): Relation ({new_source}, {new_target}, {new_type}) 已存在"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-R-03:同 Project(由 service 层校验)
// =====================================================================

/// **INV-R-03** 占位(由 service 层比对 source/target project_id)
pub fn check_invariant_03_same_project_placeholder() -> Result<(), RelationError> {
    Ok(())
}

// =====================================================================
// INV-R-04:创建不引入循环(由 service 层运行 DFS)
// =====================================================================

/// **INV-R-04**:无循环。`has_cycle` 为 true 时报错。
pub fn check_invariant_04_no_cycle(has_cycle: bool, cycle: &[WorkItemId]) -> Result<(), RelationError> {
    if has_cycle {
        return Err(RelationError::InvalidState(format!(
            "INV-R-04 (R-004): 检测到循环依赖 cycle={:?}",
            cycle
        )));
    }
    Ok(())
}

// =====================================================================
// INV-R-05:删除不级联(INV-R-05 由 service 层保证)
// =====================================================================

/// **INV-R-05** 占位
pub fn check_invariant_05_no_cascade_placeholder() -> Result<(), RelationError> {
    Ok(())
}

// =====================================================================
// INV-R-06:relation_type 必为枚举之一(由 Rust 类型系统保证)
// =====================================================================

/// **INV-R-06** 占位(由 Rust 类型系统保证)
pub fn check_invariant_06_enum_placeholder(_: RelationType) -> Result<(), RelationError> {
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

pub type InvariantCheck = fn(&Relation) -> Result<(), RelationError>;

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[];

pub fn run_invariants(checks: &[InvariantCheck], r: &Relation) -> Result<(), RelationError> {
    for check in checks {
        check(r)?;
    }
    Ok(())
}

pub fn check_create_invariants(r: &Relation) -> Result<(), RelationError> {
    check_invariant_01_source_not_target(r.source_work_item_id, r.target_work_item_id)?;
    check_invariant_03_same_project_placeholder()?;
    check_invariant_05_no_cascade_placeholder()?;
    check_invariant_06_enum_placeholder(r.relation_type)?;
    Ok(())
}

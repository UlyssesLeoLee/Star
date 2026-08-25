//! Workflow 不变量检查函数(6 条 INV-WF-01~06)
//!
//! 来源: docs/specs/domain-workflow-spec.md §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_<NN>(...) -> Result<(), WorkflowError>`,
//! 由 `ALL_INVARIANT_CHECKS` 列表聚合,供 `service.rs` 的命令实现批量执行。
//!
//! **不变量清单**:
//! - INV-WF-01: system_default Workflow 不可被修改 / 删除(平台级只读)
//! - INV-WF-02: 每个 WorkflowDefinition 必有一个 Initial State 且唯一
//! - INV-WF-03: Transition 必须有 from / to,且 from ≠ to
//! - INV-WF-04: State 名称在同一 Workflow 内 UNIQUE
//! - INV-WF-05: 删除 WorkflowDefinition 前需级联检查 Project Policy 引用
//! - INV-WF-06: 自定义 Workflow 必须继承 system default 的全部基本 State

use crate::entity::{State, Transition, WorkflowDefinition};
use crate::error::WorkflowError;
use crate::value_object::{StateCategory, StateId};

/// 不变量检查函数签名(取 entity 输入)
pub type InvariantCheck = fn(&WorkflowDefinition) -> Result<(), WorkflowError>;
/// 不变量检查函数签名(取 State 输入)
pub type StateCheck = fn(&State) -> Result<(), WorkflowError>;
/// 不变量检查函数签名(取 Transition 输入)
pub type TransitionCheck = fn(&Transition) -> Result<(), WorkflowError>;

// =====================================================================
// INV-WF-01:system_default 只读保护
// =====================================================================

/// **INV-WF-01**:system_default Workflow 不可被修改 / 删除(平台级只读)
pub fn check_invariant_01_system_default_readonly(
    wf: &WorkflowDefinition,
) -> Result<(), WorkflowError> {
    if wf.is_system_default {
        return Err(WorkflowError::InvalidState(
            "INV-WF-01: system_default Workflow 平台级只读,不可修改".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-WF-02:必有一个且唯一 Initial State
// =====================================================================

/// **INV-WF-02**:每个 WorkflowDefinition 必有一个 Initial State 且唯一
///
/// 输入参数:`states` 是该 Workflow 下所有 State 列表。
pub fn check_invariant_02_initial_state_unique(
    states: &[State],
    claimed_initial: StateId,
) -> Result<(), WorkflowError> {
    let initial_count = states
        .iter()
        .filter(|s| s.category == StateCategory::Initial)
        .count();
    if initial_count == 0 {
        return Err(WorkflowError::InvalidState(
            "INV-WF-02: Workflow 缺少 Initial State".to_string(),
        ));
    }
    if initial_count > 1 {
        return Err(WorkflowError::Conflict(format!(
            "INV-WF-02: Workflow 含 {initial_count} 个 Initial State,仅允许 1 个"
        )));
    }
    // 验证 claimed_initial 指向的是 Initial
    if let Some(s) = states.iter().find(|s| s.id == claimed_initial) {
        if s.category != StateCategory::Initial {
            return Err(WorkflowError::InvalidState(format!(
                "INV-WF-02: initial_state_id {} 指向非 Initial 状态",
                claimed_initial
            )));
        }
    } else {
        return Err(WorkflowError::InvalidState(format!(
            "INV-WF-02: initial_state_id {claimed_initial} 不存在"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-WF-03:Transition from ≠ to
// =====================================================================

/// **INV-WF-03**:Transition 必须有 from / to,且 from ≠ to
pub fn check_invariant_03_transition_distinct(
    t: &Transition,
) -> Result<(), WorkflowError> {
    if t.from_state_id == t.to_state_id {
        return Err(WorkflowError::InvalidState(format!(
            "INV-WF-03: Transition {from} → {to} 自环,禁止",
            from = t.from_state_id,
            to = t.to_state_id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-WF-04:State 名称在同一 Workflow 内 UNIQUE
// =====================================================================

/// **INV-WF-04**:State 名称在同一 Workflow 内 UNIQUE
pub fn check_invariant_04_state_name_unique(states: &[State]) -> Result<(), WorkflowError> {
    let mut seen = std::collections::HashSet::new();
    for s in states {
        if !seen.insert(s.name.as_str()) {
            return Err(WorkflowError::Conflict(format!(
                "INV-WF-04: State 名称 '{}' 在同一 Workflow 内重复",
                s.name
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-WF-05:删除前 Project Policy 引用检查
// =====================================================================

/// **INV-WF-05**:删除 WorkflowDefinition 前需级联检查 Project Policy 引用
///
/// 调用方在删除前需传入被 Project 引用计数(由 application 层聚合,
/// 本函数只做"如果被引用则拒绝"的语义检查)。
pub fn check_invariant_05_no_project_reference(
    wf: &WorkflowDefinition,
    project_reference_count: usize,
) -> Result<(), WorkflowError> {
    if wf.is_system_default {
        // system_default 不可被删除(INV-WF-01)
        return Err(WorkflowError::InvalidState(
            "INV-WF-01+05: system_default Workflow 不可被删除".to_string(),
        ));
    }
    if project_reference_count > 0 {
        return Err(WorkflowError::Conflict(format!(
            "INV-WF-05: WorkflowDefinition 被 {project_reference_count} 个 Project Policy 引用,删除拒绝"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-WF-06:继承 system default 基本三态
// =====================================================================

/// **INV-WF-06**:自定义 Workflow 必须继承 system default 的全部基本 State(TODO/IN_PROGRESS/DONE)
pub fn check_invariant_06_inherit_default_states(states: &[State]) -> Result<(), WorkflowError> {
    let required = ["TODO", "IN_PROGRESS", "DONE"];
    let names: std::collections::HashSet<&str> = states.iter().map(|s| s.name.as_str()).collect();
    let missing: Vec<&&str> = required
        .iter()
        .filter(|r| !names.contains(**r))
        .collect();
    if !missing.is_empty() {
        let missing_list = missing
            .iter()
            .map(|s| **s)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(WorkflowError::InvalidState(format!(
            "INV-WF-06: 自定义 Workflow 缺少 system default 基本 State: {missing_list}"
        )));
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// **所有不变量检查(创建时执行)**
///
/// INV-WF-01/03 接受 `&WorkflowDefinition` / `&Transition` 单参,
/// 聚合在此不包含;各路径由 service 显式调用。
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[];

/// 批量执行不变量检查,首次失败即返回错误。
pub fn run_invariants(checks: &[InvariantCheck], wf: &WorkflowDefinition) -> Result<(), WorkflowError> {
    for check in checks {
        check(wf)?;
    }
    Ok(())
}

/// 创建时的核心不变量集合(INV-WF-01,02,04,06)
pub fn check_create_invariants(
    wf: &WorkflowDefinition,
    states: &[State],
) -> Result<(), WorkflowError> {
    check_invariant_01_system_default_readonly(wf)?;
    check_invariant_04_state_name_unique(states)?;
    check_invariant_06_inherit_default_states(states)?;
    check_invariant_02_initial_state_unique(states, wf.initial_state_id)?;
    Ok(())
}

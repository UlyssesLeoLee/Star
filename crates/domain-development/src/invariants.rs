//! Development 不变量检查函数(10 条 INV-DX-01~10)
//!
//! 来源: docs/specs/domain-development-spec.md §3 (8 条 INV-D-01~08,扩展为 10 条)
//!
//! 命名约定:为与 `domain-workflow` 的 INV-WF-NN 区分,本 crate 使用 `INV-DX-NN` 前缀。
//!
//! **不变量清单**:
//! - INV-DX-01: ChangeSet ≠ Git Diff(必须结构化)
//! - INV-DX-02: 1 ChangeSet 关联 1 Commit,1 Commit 可被 0..1 PR 引用
//! - INV-DX-03: Diff 全文不存 PostgreSQL,仅 `diff_reference` 引用 Object Storage
//! - INV-DX-04: 8 种 Risk Signal 类型(基本设计锁定)
//! - INV-DX-05: Diff / Build Log / Test Log 的 Object Storage Key 必带 tenant_id 前缀
//! - INV-DX-06: SymbolIndex 跨 Repository 不合并(独立 Project)
//! - INV-DX-07: AISelfClaim RiskSignal 必走 Validation Chain(VAL-001 强约束)
//! - INV-DX-08: Symbol-aware Context 第一阶段 File-level + Basic Symbol Detection
//! - INV-DX-09: DevelopmentExecution.worktree_ids 1..N(至少 1 个 Worktree)
//! - INV-DX-10: 已 commit 的 ChangeSet 不可修改(INV-D-02 互补)

use crate::entity::ChangeSet;
use crate::error::DevelopmentError;
use crate::value_object::{
    ExecutionState, FilePath, RiskSignalKind, TenantId,
};

// =====================================================================
// INV-DX-01:ChangeSet ≠ Git Diff
// =====================================================================

/// **INV-DX-01**:ChangeSet ≠ Git Diff(必须结构化)
///
/// 验证 ChangeSet 含至少 1 个结构化 FileChange 与 ≥1 字段(files / symbols / risk_signals)
pub fn check_invariant_01_structured_change_set(
    change_set: &ChangeSet,
) -> Result<(), DevelopmentError> {
    if change_set.files.is_empty() {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-01: ChangeSet 必须结构化(files 不能为空)".to_string(),
        ));
    }
    // 必须同时有 file / risk_signal / symbols 之一
    if change_set.risk_signals.is_empty() {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-01: ChangeSet 必含 risk_signals(8 种类型之一)".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-DX-02:1 ChangeSet 关联 1 Commit
// =====================================================================

/// **INV-DX-02**:1 ChangeSet 关联 1 Commit,1 Commit 可被 0..1 PR 引用
///
/// 输入为 ChangeSet,校验其 commit_id 非空(UUID::nil() 视为空)
pub fn check_invariant_02_one_commit_per_change_set(
    change_set: &ChangeSet,
) -> Result<(), DevelopmentError> {
    if change_set.commit_id.as_uuid().is_nil() {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-02: ChangeSet 必须关联 1 个 Commit(commit_id 必填)".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-DX-03:Diff 全文不入 PostgreSQL
// =====================================================================

/// **INV-DX-03**:Diff 全文不存 PostgreSQL,仅 `diff_reference` 引用 Object Storage
///
/// 校验:`diff_reference` 非空且不包含完整 diff body 标识
pub fn check_invariant_03_diff_reference_only(
    change_set: &ChangeSet,
) -> Result<(), DevelopmentError> {
    if change_set.diff_reference.trim().is_empty() {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-03: diff_reference 必填(Diff 全文不入 PostgreSQL)".to_string(),
        ));
    }
    // 防止整段 diff 内容误填到 reference
    if change_set.diff_reference.contains('\n') {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-03: diff_reference 必为 Object Storage Key(单行,无换行)".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-DX-04:8 种 Risk Signal 类型
// =====================================================================

/// **INV-DX-04**:8 种 Risk Signal 类型(基本设计锁定,接口稳定承诺 #4)
pub fn check_invariant_04_eight_risk_signal_kinds(
    change_set: &ChangeSet,
) -> Result<(), DevelopmentError> {
    for r in &change_set.risk_signals {
        if !r.kind.is_known() {
            return Err(DevelopmentError::InvalidRiskSignalKind(r.kind.to_string()));
        }
    }
    Ok(())
}

// =====================================================================
// INV-DX-05:Object Storage Key 必带 tenant_id 前缀
// =====================================================================

/// **INV-DX-05**:Diff / Build Log / Test Log 的 Object Storage Key 必带 tenant_id 前缀
///
/// Object Storage Key 格式:`development.diff/{tenant_id}/{change_set_id}.diff`
pub fn check_invariant_05_tenant_prefix_in_storage_key(
    diff_reference: &str,
    expected_tenant_id: TenantId,
) -> Result<(), DevelopmentError> {
    let prefix = format!("development.diff/{expected_tenant_id}/");
    if !diff_reference.starts_with(&prefix) {
        return Err(DevelopmentError::InvalidObjectStorageKey(format!(
            "INV-DX-05: Object Storage Key 必须以 '{prefix}' 开头,实际: {diff_reference}"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-DX-06:SymbolIndex 跨 Repository 不合并
// =====================================================================

/// **INV-DX-06**:SymbolIndex 跨 Repository 不合并(独立 Project)
///
/// 校验输入:SymbolIndex 的 repository_id 与 expected_repository_id 一致
pub fn check_invariant_06_symbol_index_repository_boundary(
    symbol_index_repository_id: crate::value_object::RepositoryId,
    expected_repository_id: crate::value_object::RepositoryId,
) -> Result<(), DevelopmentError> {
    if symbol_index_repository_id != expected_repository_id {
        return Err(DevelopmentError::InvalidState(format!(
            "INV-DX-06: SymbolIndex repository_id 不匹配(expected={}, actual={symbol_index_repository_id})",
            expected_repository_id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-DX-07:AISelfClaim 必走 Validation Chain
// =====================================================================

/// **INV-DX-07**:AISelfClaim RiskSignal 必走 Validation Chain(VAL-001)
///
/// 校验:`validation_passed_id` 不为 None
pub fn check_invariant_07_ai_self_claim_validation(
    kind: RiskSignalKind,
    validation_passed_id: Option<uuid::Uuid>,
) -> Result<(), DevelopmentError> {
    if kind == RiskSignalKind::AISelfClaim && validation_passed_id.is_none() {
        return Err(DevelopmentError::ValidationRequired);
    }
    Ok(())
}

// =====================================================================
// INV-DX-08:Symbol-aware Context 第一阶段 File-level + Basic Symbol Detection
// =====================================================================

/// **INV-DX-08**:Symbol-aware Context 第一阶段 File-level + Basic Symbol Detection
///
/// 校验:IndexedSymbol 必含 file_path
pub fn check_invariant_08_file_level_symbols(
    symbol: &crate::entity::IndexedSymbol,
) -> Result<(), DevelopmentError> {
    if symbol.file_path.as_str().is_empty() {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-08: IndexedSymbol 必含 file_path(第一阶段 File-level)".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-DX-09:DevelopmentExecution.worktree_ids 1..N
// =====================================================================

/// **INV-DX-09**:DevelopmentExecution.worktree_ids 1..N(至少 1 个 Worktree)
pub fn check_invariant_09_at_least_one_worktree(
    worktree_ids: &[crate::value_object::WorktreeId],
) -> Result<(), DevelopmentError> {
    if worktree_ids.is_empty() {
        return Err(DevelopmentError::InvalidState(
            "INV-DX-09: DevelopmentExecution.worktree_ids 必含至少 1 个 Worktree".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-DX-10:已 commit 的 ChangeSet 不可修改
// =====================================================================

/// **INV-DX-10**:已 commit 的 ChangeSet 不可修改(INV-D-02 互补)
pub fn check_invariant_10_change_set_not_committed(
    change_set: &ChangeSet,
) -> Result<(), DevelopmentError> {
    if change_set.is_committed {
        return Err(DevelopmentError::Conflict(format!(
            "INV-DX-10: ChangeSet {} 已 commit,不可修改",
            change_set.id
        )));
    }
    Ok(())
}

// =====================================================================
// 辅助:ExecutionState 终态校验
// =====================================================================

/// Execution 关闭时:terminal_state 必须是终态
pub fn check_terminal_state(
    terminal_state: ExecutionState,
) -> Result<(), DevelopmentError> {
    if !matches!(
        terminal_state,
        ExecutionState::Succeeded | ExecutionState::Failed | ExecutionState::Cancelled
    ) {
        return Err(DevelopmentError::InvalidState(format!(
            "Execution 关闭时 terminal_state 必为 Succeeded/Failed/Cancelled,实际: {terminal_state}"
        )));
    }
    Ok(())
}

// =====================================================================
// 批量执行(append_change_set 一并执行)
// =====================================================================

/// 全部 append_change_set 不变量检查(INV-DX-01/02/03/04/05)
pub fn check_append_change_set_invariants(
    change_set: &ChangeSet,
) -> Result<(), DevelopmentError> {
    check_invariant_01_structured_change_set(change_set)?;
    check_invariant_02_one_commit_per_change_set(change_set)?;
    check_invariant_03_diff_reference_only(change_set)?;
    check_invariant_04_eight_risk_signal_kinds(change_set)?;
    check_invariant_05_tenant_prefix_in_storage_key(
        &change_set.diff_reference,
        change_set.tenant_id,
    )?;
    Ok(())
}

/// 所有不变量检查(空数组占位,各路径由 service 显式调用)
pub const ALL_INVARIANT_CHECKS: &[fn() -> Result<(), DevelopmentError>] = &[];

// 引入以避免 unused_import 警告
#[allow(dead_code)]
const _: Option<FilePath> = None;

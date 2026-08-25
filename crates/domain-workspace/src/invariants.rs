//! Workspace 不变量

use crate::entity::Workspace;
use crate::error::WorkspaceError;
use crate::value_object::WorkspaceId;

pub type InvariantCheck = fn(&Workspace) -> Result<(), WorkspaceError>;

/// INV-WS-01:`workspace_key` 在 tenant 内唯一
pub fn check_invariant_01_workspace_key_unique(
    ws: &Workspace,
    existing: &[String],
) -> Result<(), WorkspaceError> {
    if existing.iter().any(|k| k == &ws.workspace_key) {
        return Err(WorkspaceError::Conflict(format!(
            "INV-WS-01: workspace_key '{}' 已被占用",
            ws.workspace_key
        )));
    }
    Ok(())
}

/// INV-WS-02:Workspace 必带 tenant_id
pub fn check_invariant_02_tenant_id_present(ws: &Workspace) -> Result<(), WorkspaceError> {
    if ws.tenant_id.as_uuid().is_nil() {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-02: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    Ok(())
}

/// INV-WS-03:workspace_key 格式校验(非空 + 长度)
pub fn check_invariant_03_workspace_key_format(ws: &Workspace) -> Result<(), WorkspaceError> {
    if ws.workspace_key.trim().is_empty() {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-03: workspace_key 不能为空".to_string(),
        ));
    }
    if ws.workspace_key.len() > 64 {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-03: workspace_key 长度 ≤ 64 字符".to_string(),
        ));
    }
    Ok(())
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_02_tenant_id_present,
    check_invariant_03_workspace_key_format,
];

pub fn run_invariants(checks: &[InvariantCheck], ws: &Workspace) -> Result<(), WorkspaceError> {
    for check in checks {
        check(ws)?;
    }
    Ok(())
}

// 防止 unused import
#[allow(dead_code)]
fn _unused(_w: WorkspaceId) {}

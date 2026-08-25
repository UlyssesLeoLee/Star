//! Project 不变量

use crate::entity::Project;
use crate::error::ProjectError;

pub type InvariantCheck = fn(&Project) -> Result<(), ProjectError>;

/// INV-PRJ-01:`project_key` 在 tenant 内唯一
pub fn check_invariant_01_project_key_unique(
    p: &Project,
    existing: &[String],
) -> Result<(), ProjectError> {
    if existing.iter().any(|k| k == &p.project_key) {
        return Err(ProjectError::Conflict(format!(
            "INV-PRJ-01: project_key '{}' 已被占用",
            p.project_key
        )));
    }
    Ok(())
}

/// INV-PRJ-02:Project 必带 tenant_id
pub fn check_invariant_02_tenant_id_present(p: &Project) -> Result<(), ProjectError> {
    if p.tenant_id.as_uuid().is_nil() {
        return Err(ProjectError::InvalidState(
            "INV-PRJ-02: tenant_id 必须非 nil".to_string(),
        ));
    }
    Ok(())
}

/// INV-PRJ-03:project_key 格式校验
pub fn check_invariant_03_project_key_format(p: &Project) -> Result<(), ProjectError> {
    if p.project_key.trim().is_empty() {
        return Err(ProjectError::InvalidState(
            "INV-PRJ-03: project_key 不能为空".to_string(),
        ));
    }
    if p.project_key.len() > 32 {
        return Err(ProjectError::InvalidState(
            "INV-PRJ-03: project_key 长度 ≤ 32 字符".to_string(),
        ));
    }
    Ok(())
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_02_tenant_id_present,
    check_invariant_03_project_key_format,
];

pub fn run_invariants(checks: &[InvariantCheck], p: &Project) -> Result<(), ProjectError> {
    for check in checks {
        check(p)?;
    }
    Ok(())
}

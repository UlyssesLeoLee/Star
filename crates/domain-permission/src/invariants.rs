//! Permission 不变量

use crate::entity::{Permission, PermissionScheme, Role};
use crate::error::PermissionError;

pub type InvariantCheck = fn(&Role) -> Result<(), PermissionError>;

/// INV-PERM-01:permission code 全局唯一
pub fn check_invariant_01_permission_code_unique(
    perm: &Permission,
    existing_codes: &[String],
) -> Result<(), PermissionError> {
    if existing_codes.iter().any(|c| c == &perm.code) {
        return Err(PermissionError::Conflict(format!(
            "INV-PERM-01: permission code '{}' 已存在",
            perm.code
        )));
    }
    Ok(())
}

/// INV-PERM-02:PermissionScheme 必有 owner role
pub fn check_invariant_02_scheme_has_owner(
    scheme: &PermissionScheme,
) -> Result<(), PermissionError> {
    if scheme.default_role.is_empty() {
        return Err(PermissionError::InvalidState(
            "INV-PERM-02: PermissionScheme 必须有 default_role".to_string(),
        ));
    }
    if !scheme.role_permissions.contains_key(&scheme.default_role) {
        return Err(PermissionError::InvalidState(format!(
            "INV-PERM-02: default_role '{}' 不在 role_permissions 中",
            scheme.default_role
        )));
    }
    Ok(())
}

/// INV-PERM-03:Role 必带 tenant_id
pub fn check_invariant_03_tenant_id_present(role: &Role) -> Result<(), PermissionError> {
    if role.tenant_id.as_uuid().is_nil() {
        return Err(PermissionError::InvalidState(
            "INV-PERM-03: tenant_id 必须非 nil".to_string(),
        ));
    }
    Ok(())
}

/// INV-PERM-04:Role name 非空
pub fn check_invariant_04_role_name_format(role: &Role) -> Result<(), PermissionError> {
    if role.name.trim().is_empty() {
        return Err(PermissionError::InvalidState(
            "INV-PERM-04: role name 不能为空".to_string(),
        ));
    }
    Ok(())
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_03_tenant_id_present,
    check_invariant_04_role_name_format,
];

pub fn run_invariants(checks: &[InvariantCheck], r: &Role) -> Result<(), PermissionError> {
    for c in checks {
        c(r)?;
    }
    Ok(())
}

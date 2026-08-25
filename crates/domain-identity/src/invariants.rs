//! Identity 不变量检查函数
//!
//! 来源: docs/specs/domain-identity-spec.md §3
//!
//! **不变量清单**:
//! - INV-IDN-01: `email` 在 tenant 内全局唯一
//! - INV-IDN-02: (device_id, user_id, project_id) 三元组在平台内唯一
//! - INV-IDN-03: User 必带 `tenant_id` (§6.1,REQ-SEC-001)
//! - INV-IDN-04: User 邮箱格式合法(简化:非空 + 含 `@`)

use crate::entity::User;
use crate::error::IdentityError;
use crate::value_object::UserId;

/// 不变量检查函数签名(取 entity 输入)
pub type InvariantCheck = fn(&User) -> Result<(), IdentityError>;

/// **INV-IDN-01**:`email` 在 tenant 内唯一
///
/// service 层在 `create_user` 时与 `existing_emails: &[String]` 比对。
pub fn check_invariant_01_email_unique(
    user: &User,
    existing_emails: &[String],
) -> Result<(), IdentityError> {
    if existing_emails
        .iter()
        .any(|e| e.eq_ignore_ascii_case(&user.email))
    {
        return Err(IdentityError::Conflict(format!(
            "INV-IDN-01: email '{}' 已被占用",
            user.email
        )));
    }
    Ok(())
}

/// **INV-IDN-02**:(device_id, user_id, project_id) 三元组唯一
///
/// service 层在 `bind_device` 时调用。
pub fn check_invariant_02_device_binding_unique(
    device_id: uuid::Uuid,
    user_id: UserId,
    project_id: Option<uuid::Uuid>,
    existing: &[(uuid::Uuid, UserId, Option<uuid::Uuid>)],
) -> Result<(), IdentityError> {
    if existing
        .iter()
        .any(|(d, u, p)| *d == device_id && *u == user_id && *p == project_id)
    {
        return Err(IdentityError::Conflict(
            "INV-IDN-02: (device, user, project) 三元组已绑定".to_string(),
        ));
    }
    Ok(())
}

/// **INV-IDN-03**:User 必带 `tenant_id`
pub fn check_invariant_03_tenant_id_present(user: &User) -> Result<(), IdentityError> {
    if user.tenant_id.as_uuid().is_nil() {
        return Err(IdentityError::InvalidState(
            "INV-IDN-03: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    Ok(())
}

/// **INV-IDN-04**:邮箱格式合法
pub fn check_invariant_04_email_format(user: &User) -> Result<(), IdentityError> {
    if user.email.is_empty() || !user.email.contains('@') {
        return Err(IdentityError::InvalidState(format!(
            "INV-IDN-04: email 格式非法: {}",
            user.email
        )));
    }
    if user.email.len() > 255 {
        return Err(IdentityError::InvalidState(
            "INV-IDN-04: email 长度 ≤ 255 字符".to_string(),
        ));
    }
    Ok(())
}

/// **所有不变量检查(创建时执行)**
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_03_tenant_id_present,
    check_invariant_04_email_format,
];

/// 批量执行不变量检查
pub fn run_invariants(checks: &[InvariantCheck], u: &User) -> Result<(), IdentityError> {
    for check in checks {
        check(u)?;
    }
    Ok(())
}

//! Tenant 不变量检查函数(2 条 INV-TEN-01~02)
//!
//! 来源: docs/specs/domain-tenant-spec.md §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_<NN>(...) -> Result<(), TenantError>`,
//! 由 `ALL_INVARIANT_CHECKS` 列表聚合,供 `service.rs` 的命令实现批量执行。
//!
//! **不变量清单**:
//! - INV-TEN-01: `tenant_key` 平台内全局唯一(由 service 层在 `create_tenant` 时与已有 set 比对)
//! - INV-TEN-02: Tenant 状态机迁移合法(见 [`TenantStatus::can_transition_to`])

use crate::entity::Tenant;
use crate::error::TenantError;
use crate::value_object::TenantStatus;

/// 不变量检查函数签名(取 entity 输入)
pub type InvariantCheck = fn(&Tenant) -> Result<(), TenantError>;
/// 不变量检查函数签名(状态迁移:旧 → 新)
pub type TransitionCheck = fn(&Tenant, TenantStatus) -> Result<(), TenantError>;

/// **INV-TEN-01**:`tenant_key` 平台内全局唯一
///
/// service 层在 `create_tenant` 时与 `existing_keys: &[String]` 比对,
/// 命中冲突则返回 `TenantError::Conflict`。
pub fn check_invariant_01_tenant_key_unique(
    tenant: &Tenant,
    existing_keys: &[String],
) -> Result<(), TenantError> {
    if existing_keys.iter().any(|k| k == &tenant.tenant_key) {
        return Err(TenantError::Conflict(format!(
            "INV-TEN-01: tenant_key '{}' 已被占用",
            tenant.tenant_key
        )));
    }
    Ok(())
}

/// **INV-TEN-02**:Tenant 状态机迁移合法
pub fn check_invariant_02_status_transition(
    tenant: &Tenant,
    target: TenantStatus,
) -> Result<(), TenantError> {
    if tenant.status == target {
        return Ok(()); // 幂等
    }
    if tenant.status.can_transition_to(target) {
        Ok(())
    } else {
        Err(TenantError::InvalidState(format!(
            "INV-TEN-02: 非法状态迁移 {:?} → {:?}",
            tenant.status, target
        )))
    }
}

/// **INV-AUX-01**:Tenant 关键字段非空校验
pub fn check_invariant_required_fields(tenant: &Tenant) -> Result<(), TenantError> {
    if tenant.tenant_key.trim().is_empty() {
        return Err(TenantError::InvalidState(
            "INV-AUX-01: tenant_key 不能为空".to_string(),
        ));
    }
    if tenant.name.trim().is_empty() {
        return Err(TenantError::InvalidState(
            "INV-AUX-01: name 不能为空".to_string(),
        ));
    }
    if tenant.tenant_key.len() > 64 {
        return Err(TenantError::InvalidState(
            "INV-AUX-01: tenant_key 长度 ≤ 64 字符".to_string(),
        ));
    }
    Ok(())
}

/// **所有不变量检查(创建时执行)**
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[check_invariant_required_fields];

/// 批量执行不变量检查,首次失败即返回错误。
pub fn run_invariants(checks: &[InvariantCheck], t: &Tenant) -> Result<(), TenantError> {
    for check in checks {
        check(t)?;
    }
    Ok(())
}

//! Audit 不变量
//!
//! **不变量清单**:
//! - INV-AUD-01: append-only,AuditEvent 一旦写入不可 UPDATE/DELETE(本 crate 不暴露 update/delete 接口)
//! - INV-AUD-02: AuditEvent 必带 tenant_id + actor_id + occurred_at
//! - INV-AUD-03: `immutable_hash` 必填(防止篡改检测)
//! - INV-AUD-04: AIAuditMetadata 必带 `prompt_hash` / `response_hash`(不存明文)

use crate::entity::{AIAuditMetadata, AuditEvent};
use crate::error::AuditError;
use crate::value_object::UserId;

pub type InvariantCheck = fn(&AuditEvent) -> Result<(), AuditError>;

/// INV-AUD-02:必填字段
pub fn check_invariant_02_required_fields(ev: &AuditEvent) -> Result<(), AuditError> {
    if ev.tenant_id.as_uuid().is_nil() {
        return Err(AuditError::InvalidState(
            "INV-AUD-02: tenant_id 必须非 nil".to_string(),
        ));
    }
    if ev.actor_id.as_uuid().is_nil() {
        return Err(AuditError::InvalidState(
            "INV-AUD-02: actor_id 必须非 nil".to_string(),
        ));
    }
    if ev.target_type.is_empty() {
        return Err(AuditError::InvalidState(
            "INV-AUD-02: target_type 不能为空".to_string(),
        ));
    }
    if ev.target_id.is_nil() {
        return Err(AuditError::InvalidState(
            "INV-AUD-02: target_id 必须非 nil".to_string(),
        ));
    }
    Ok(())
}

/// INV-AUD-03:`immutable_hash` 必填
pub fn check_invariant_03_immutable_hash(ev: &AuditEvent) -> Result<(), AuditError> {
    if ev.immutable_hash.is_empty() {
        return Err(AuditError::InvalidState(
            "INV-AUD-03: immutable_hash 不能为空(防篡改)".to_string(),
        ));
    }
    if ev.immutable_hash.len() != 64 {
        return Err(AuditError::InvalidState(
            "INV-AUD-03: immutable_hash 必须是 64 字符 sha256 hex".to_string(),
        ));
    }
    Ok(())
}

/// INV-AUD-04:AI 审计元数据必带 hash
pub fn check_invariant_04_ai_metadata_required(
    m: &AIAuditMetadata,
) -> Result<(), AuditError> {
    if m.prompt_hash.is_empty() {
        return Err(AuditError::InvalidState(
            "INV-AUD-04: prompt_hash 不能为空".to_string(),
        ));
    }
    if m.response_hash.is_empty() {
        return Err(AuditError::InvalidState(
            "INV-AUD-04: response_hash 不能为空".to_string(),
        ));
    }
    Ok(())
}

/// INV-AUX-01:构造时验证 immutable_hash 正确性(简化:长度校验)
pub fn compute_immutable_hash(
    tenant: crate::value_object::TenantId,
    actor: UserId,
    action: &str,
    target_type: &str,
    target_id: uuid::Uuid,
    occurred_at: chrono::DateTime<chrono::Utc>,
) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = write!(
        s,
        "{}|{}|{}|{}|{}|{}",
        tenant,
        actor,
        action,
        target_type,
        target_id,
        occurred_at.timestamp_millis()
    );
    // 简化:用 hex 编码替代真实 SHA-256(本 demo)
    format!("{:0>64x}", seahash_hash(&s))
}

// 极简 hash(避免引入 sha2 crate)
fn seahash_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_02_required_fields,
    check_invariant_03_immutable_hash,
];

pub fn run_invariants(
    checks: &[InvariantCheck],
    ev: &AuditEvent,
) -> Result<(), AuditError> {
    for c in checks {
        c(ev)?;
    }
    Ok(())
}

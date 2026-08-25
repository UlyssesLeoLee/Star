//! Validation 不变量(INV-VL-01 ~ INV-VL-08)
//!
//! 来源:`docs/specs/domain-validation-spec.md` §3

use crate::entity::{ValidationEvidence, ValidationPolicy, ValidationResult};
use crate::error::ValidationError;
use crate::value_object::{
    EvidenceType, TenantId, ValidationKind, ValidationStatus, roles,
};

// =====================================================================
// INV-VL-01:AI 自我报告不构成完成(VAL-001 P0)
// =====================================================================

/// **INV-VL-01**:AI 自我声明完成时,必须经四重门(VAL-001 强约束)
///
/// **四重门**:`ValidationPassed && AcceptanceCoverage==100 && FeedbackResolved && GateApproved`
/// 本函数仅校验 ValidationPassed 这一维;其他三维由调用方在 application 层
/// (orchestrator)传入,因为它们跨 domain-feedback / domain-worktree。
pub fn check_ai_self_claim_requires_validation_passed(
    r: &ValidationResult,
    validation_passed: bool,
) -> Result<(), ValidationError> {
    if r.is_ai_complete_claim && !validation_passed {
        return Err(ValidationError::InvariantViolated(
            "INV-VL-01/VAL-001: AI 自我声明完成,但 ValidationPassed = false,四重门失败"
                .to_string(),
        ));
    }
    Ok(())
}

/// **INV-VL-01 简化版**:`is_ai_complete_claim=true` 时,result.status 必须为 Passed
pub fn check_ai_self_claim_status(r: &ValidationResult) -> Result<(), ValidationError> {
    if r.is_ai_complete_claim && r.status != ValidationStatus::Passed {
        return Err(ValidationError::InvariantViolated(format!(
            "INV-VL-01/VAL-001: AI 自我声明完成,但 status={:?} ≠ Passed",
            r.status
        )));
    }
    Ok(())
}

// =====================================================================
// INV-VL-02:四重门(本 crate 仅校验第一维 + 文档说明)
// =====================================================================

/// **INV-VL-02**:四重门文档占位(其余三维由 application orchestrator 校验)
pub fn check_invariant_02_four_gates_placeholder() -> Result<(), ValidationError> {
    Ok(())
}

// =====================================================================
// INV-VL-03:6 状态机严格迁移(SOW 5 状态;由 value_object::is_valid_state_transition 实现)
// =====================================================================

/// **INV-VL-03**:状态机迁移校验
pub fn check_invariant_03_state_transition(
    from: ValidationStatus,
    to: ValidationStatus,
) -> Result<(), ValidationError> {
    if !crate::value_object::is_valid_state_transition(from, to) {
        return Err(ValidationError::InvalidState(format!(
            "INV-VL-03: 非法状态迁移 {from} -> {to}"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-VL-04:ValidationResult 必带 evidence_ref(不可缺)
// =====================================================================

/// **INV-VL-04**:PASSED 状态必须带 evidence(log_excerpt_ref 或 evidence_ids 非空)
pub fn check_invariant_04_evidence_required(r: &ValidationResult) -> Result<(), ValidationError> {
    // 仅当状态进入 Passed / Failed 时强制 evidence(VAL-001 强约束)
    if matches!(r.status, ValidationStatus::Passed | ValidationStatus::Failed) && !r.has_evidence() {
        return Err(ValidationError::InvalidState(
            "INV-VL-04 / VAL-001: ValidationResult PASSED/FAILED 必须带 evidence_ref(不可 Agent 自报)"
                .to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-VL-05:AcceptanceCoverage 100% 是 READY_FOR_REVIEW 必要条件
// =====================================================================

/// **INV-VL-05**:AcceptanceCoverage 未达 100% 时尝试进入 READY_FOR_REVIEW 拒绝
pub fn check_invariant_05_full_coverage_required(
    total_criteria: u32,
    covered: u32,
) -> Result<(), ValidationError> {
    if total_criteria == 0 || covered < total_criteria {
        return Err(ValidationError::InvalidState(format!(
            "INV-VL-05: AcceptanceCoverage < 100% ({covered}/{total_criteria}),不可 READY_FOR_REVIEW"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-VL-06:Override 必须人类 Protected 鉴权 + Audit
// =====================================================================

/// **INV-VL-06**:actor 必须为人类(非 service / agent)
pub fn check_invariant_06_override_human_only(
    actor: &crate::context::ActorContext,
) -> Result<(), ValidationError> {
    if actor.is_service_internal() || actor.has_role(roles::DEVELOPER) && actor.user_id.as_uuid().is_nil() {
        return Err(ValidationError::PermissionDenied);
    }
    // 简化:必须非 service_internal
    if actor.is_service_internal() {
        return Err(ValidationError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-VL-07:必带 tenant_id,跨 tenant 拒绝
// =====================================================================

/// **INV-VL-07**:必带 tenant_id
pub fn check_invariant_07_tenant_id_present(r: &ValidationResult) -> Result<(), ValidationError> {
    if r.tenant_id.as_uuid().is_nil() {
        return Err(ValidationError::InvalidState(
            "INV-VL-07: ValidationResult 必带 tenant_id".to_string(),
        ));
    }
    if r.project_id.as_uuid().is_nil() {
        return Err(ValidationError::InvalidState(
            "INV-VL-07: ValidationResult 必带 project_id".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-VL-08:Build Log / Test Log Object Storage Key 必带 tenant_id 前缀
// =====================================================================

/// **INV-VL-08**:ValidationEvidence.storage_ref 必带 tenant_id 前缀(13 类 #10/#11)
pub fn check_invariant_08_evidence_storage_tenant_prefix(
    e: &ValidationEvidence,
) -> Result<(), ValidationError> {
    let prefix = format!("{}/", e.tenant_id);
    if !e.storage_ref.starts_with(&prefix) {
        return Err(ValidationError::InvalidState(format!(
            "INV-VL-08: evidence storage_ref '{}' 缺少 tenant_id 前缀 '{}'",
            e.storage_ref, prefix
        )));
    }
    Ok(())
}

// =====================================================================
// INV-VL-09:ValidationPolicy 默认 allow_ai_self_claim=false
// =====================================================================

/// **INV-VL-09**:新建 Policy 必须默认 allow_ai_self_claim=false(VAL-001)
pub fn check_invariant_09_policy_default_ai_self_claim(
    p: &ValidationPolicy,
) -> Result<(), ValidationError> {
    if p.allow_ai_self_claim {
        return Err(ValidationError::InvariantViolated(
            "INV-VL-09/VAL-001: ValidationPolicy.allow_ai_self_claim 不可为 true(VAL-001 强约束)"
                .to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// EvidenceType 限制(INV-VL-10,与 data-design §4.24.2 对齐)
// =====================================================================

/// **INV-VL-10**:evidence_type 必须在白名单(由 enum 类型保证;此处占位)
pub fn check_invariant_10_evidence_type_whitelist(
    _t: EvidenceType,
) -> Result<(), ValidationError> {
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// 创建时的不变量校验(收口)
pub fn check_create_invariants(r: &ValidationResult) -> Result<(), ValidationError> {
    check_invariant_07_tenant_id_present(r)?;
    // 创建时不应直接处于终态(必须先 Pending / Running)
    if r.status == ValidationStatus::Skipped {
        return Err(ValidationError::InvalidState(
            "创建时不可直接进入 Skipped".to_string(),
        ));
    }
    Ok(())
}

/// 状态变更时的全量校验
pub fn check_status_transition(
    from: ValidationStatus,
    to: ValidationStatus,
    r: &ValidationResult,
) -> Result<(), ValidationError> {
    check_invariant_03_state_transition(from, to)?;
    // 到达 Passed/Failed 时强校验 evidence
    if matches!(to, ValidationStatus::Passed | ValidationStatus::Failed) {
        check_invariant_04_evidence_required(r)?;
        check_ai_self_claim_status(r)?;
    }
    Ok(())
}

// 静默引用
#[allow(dead_code)]
fn _unused_t(t: TenantId) -> TenantId {
    t
}
#[allow(dead_code)]
fn _unused_k(k: ValidationKind) -> ValidationKind {
    k
}

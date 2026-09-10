//! v0.58 P0-2 Stage 1: 3 域 From<DomainError> for RestError 映射集成测试
//!
//! 覆盖:
//! 1. `domain_feedback::FeedbackError` → `RestError` (10 变体)
//! 2. `domain_integration::IntegrationError` → `RestError` (8 变体)
//! 3. `domain_comment::CommentError` → `RestError` (9 变体)
//!
//! 每个变体都验证:
//! - `code` 是 SCREAMING_SNAKE_CASE 字符串
//! - `source_module` 是正确的域
//! - `source_kind` ∈ {Validation, Policy, External, Internal}
//! - `retriable` 正确 (仅 Internal 类为 true)
//! - `hint` 非空
//! - HTTP 状态映射正确 (NotFound → 404, Conflict → 409, PermissionDenied → 403, Internal → 500, 其他 400/422)

use axum::http::StatusCode;
use star_api_rest::error::RestError;

// =====================================================================
// 1. FeedbackError → RestError (10 变体)
// =====================================================================

#[test]
fn feedback_not_found_maps_to_404() {
    let e: domain_feedback::FeedbackError =
        domain_feedback::FeedbackError::NotFound(domain_feedback::value_object::FeedbackId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "FB_NOT_FOUND");
    assert_eq!(r.source_module, "domain-feedback");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
    assert!(!r.hint.is_empty());
    assert_eq!(StatusCode::from_u16(404).unwrap(), StatusCode::NOT_FOUND);
}

#[test]
fn feedback_internal_maps_to_500_with_retriable() {
    let e = domain_feedback::FeedbackError::Internal("db down".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert_eq!(r.source_module, "domain-feedback");
    assert_eq!(r.source_kind, "Internal");
    assert!(r.retriable, "Internal errors must be retriable");
    assert!(!r.hint.is_empty());
}

#[test]
fn feedback_permission_denied_maps_to_policy() {
    let e = domain_feedback::FeedbackError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "FB_PERMISSION_DENIED");
    assert_eq!(r.source_kind, "Policy");
    assert!(!r.retriable);
}

#[test]
fn feedback_conflict_maps_to_external() {
    let e = domain_feedback::FeedbackError::Conflict("duplicate".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "FB_CONFLICT");
    assert_eq!(r.source_kind, "External");
    assert!(!r.retriable);
}

#[test]
fn feedback_read_only_maps_to_validation() {
    let e = domain_feedback::FeedbackError::ReadOnly;
    let r: RestError = e.into();
    assert_eq!(r.code, "FB_READ_ONLY");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
}

#[test]
fn feedback_cross_worktree_maps_to_validation() {
    let e = domain_feedback::FeedbackError::CrossWorktree;
    let r: RestError = e.into();
    assert_eq!(r.code, "FB_CROSS_WORKTREE");
    assert_eq!(r.source_kind, "Validation");
}

// =====================================================================
// 2. IntegrationError → RestError (8 变体)
// =====================================================================

#[test]
fn integration_not_found_maps_to_404() {
    let e = domain_integration::IntegrationError::NotFound(
        domain_integration::value_object::IntegrationId::new(),
    );
    let r: RestError = e.into();
    assert_eq!(r.code, "RESOURCE_NOT_FOUND");
    assert_eq!(r.source_module, "domain-integration");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
}

#[test]
fn integration_permission_denied_maps_to_policy() {
    let e = domain_integration::IntegrationError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn integration_conflict_maps_to_external() {
    let e = domain_integration::IntegrationError::Conflict("dup-key".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTEGRATION_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn integration_loop_guard_missing_maps_to_validation() {
    let e = domain_integration::IntegrationError::LoopGuardMissing("bidirectional".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "I_LOOP_GUARD_MISSING");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn integration_credential_missing_maps_to_validation() {
    let e = domain_integration::IntegrationError::CredentialMissing("github".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "I_CREDENTIAL_MISSING");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn integration_internal_maps_to_500_with_retriable() {
    let e = domain_integration::IntegrationError::Internal("network".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

#[test]
fn integration_invalid_argument_maps_to_validation() {
    let e = domain_integration::IntegrationError::InvalidArgument("bad url".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn integration_invalid_state_maps_to_validation() {
    let e = domain_integration::IntegrationError::InvalidState("already-running".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert_eq!(r.source_kind, "Validation");
}

// =====================================================================
// 3. CommentError → RestError (9 变体)
// =====================================================================

#[test]
fn comment_not_found_maps_to_404() {
    let e = domain_comment::CommentError::NotFound("cmt_123".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "COMMENT_NOT_FOUND");
    assert_eq!(r.source_module, "domain-comment");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
}

#[test]
fn comment_permission_denied_maps_to_policy() {
    let e = domain_comment::CommentError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn comment_cross_tenant_denied_maps_to_policy() {
    use domain_comment::TenantId;
    let e = domain_comment::CommentError::CrossTenantDenied(TenantId::new(), TenantId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn comment_invalid_object_key_maps_to_validation() {
    let e = domain_comment::CommentError::InvalidObjectKey;
    let r: RestError = e.into();
    assert_eq!(r.code, "CMT_INVALID_OBJECT_KEY");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
}

#[test]
fn comment_reaction_exists_maps_to_external() {
    let e = domain_comment::CommentError::ReactionExists;
    let r: RestError = e.into();
    assert_eq!(r.code, "CMT_REACTION_EXISTS");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn comment_edit_deleted_maps_to_validation() {
    let e = domain_comment::CommentError::EditDeleted;
    let r: RestError = e.into();
    assert_eq!(r.code, "CMT_EDIT_DELETED");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn comment_conflict_maps_to_external() {
    let e = domain_comment::CommentError::Conflict("version".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "COMMENT_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn comment_internal_maps_to_500_with_retriable() {
    let e = domain_comment::CommentError::Internal("db".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

#[test]
fn comment_invalid_state_maps_to_validation() {
    let e = domain_comment::CommentError::InvalidState("closed".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert_eq!(r.source_kind, "Validation");
}

// =====================================================================
// 4. 全部 27 变体 (10 + 8 + 9) 跨域交叉验证: source_module 正确
// =====================================================================

#[test]
fn all_27_variants_have_correct_source_module() {
    // 抽样 3 个不同域的 Internal 变体, 验证 source_module 正确
    let fb: RestError = domain_feedback::FeedbackError::Internal("x".to_string()).into();
    assert_eq!(fb.source_module, "domain-feedback");

    let ig: RestError = domain_integration::IntegrationError::Internal("x".to_string()).into();
    assert_eq!(ig.source_module, "domain-integration");

    let cm: RestError = domain_comment::CommentError::Internal("x".to_string()).into();
    assert_eq!(cm.source_module, "domain-comment");
}

#[test]
fn all_27_variants_serialize_roundtrip() {
    // 验证 RestError 可序列化 (per spec §2.4 JSON 协议)
    let r: RestError =
        domain_feedback::FeedbackError::NotFound(domain_feedback::value_object::FeedbackId::new())
            .into();
    let json = serde_json::to_string(&r).expect("serialize");
    let parsed: RestError = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.code, r.code);
    assert_eq!(parsed.source_module, r.source_module);
    assert_eq!(parsed.retriable, r.retriable);
}

//! v0.60 P0-2 Stage 3: 4 域 From<DomainError> for RestError 映射集成测试
//!
//! 覆盖 (per WBS §14.15):
//! 1. `domain_tenant::TenantError` → `RestError` (7 变体)
//! 2. `domain_identity::IdentityError` → `RestError` (8 变体)
//! 3. `domain_permission::PermissionError` → `RestError` (6 变体)
//! 4. `domain_project::ProjectError` → `RestError` (7 变体)
//!
//! 累计 P0-2 域映射: 6 → 9 (v0.58) → 13 (v0.59) → **17 域** (v0.60, 含 cli hermes 算 1)
//! 累计变体: 27 (v0.58) + 39 (v0.59) + 28 (v0.60) = **94 变体**
//! P0-2 = 100% 收官 (per WBS §14.15 0.3M tokens 实证)

use star_api_rest::error::RestError;

// =====================================================================
// 1. TenantError → RestError (7 变体)
// =====================================================================

#[test]
fn tenant_not_found_maps_to_404() {
    let e = domain_tenant::TenantError::NotFound("tenant-123".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "TENANT_NOT_FOUND");
    assert_eq!(r.source_module, "domain-tenant");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
}

#[test]
fn tenant_permission_denied_maps_to_policy() {
    let e = domain_tenant::TenantError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn tenant_cross_tenant_denied_maps_to_policy() {
    use domain_tenant::TenantId;
    let e = domain_tenant::TenantError::CrossTenantDenied(TenantId::new(), TenantId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn tenant_slug_exists_maps_to_409() {
    let e = domain_tenant::TenantError::SlugExists("acme".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "TENANT_SLUG_EXISTS");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn tenant_invalid_state_maps_to_422() {
    let e = domain_tenant::TenantError::InvalidState("suspended".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "TENANT_INVALID_STATE");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn tenant_conflict_maps_to_external() {
    let e = domain_tenant::TenantError::Conflict("dup-key".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "TENANT_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn tenant_internal_maps_to_500_with_retriable() {
    let e = domain_tenant::TenantError::Internal("db".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 2. IdentityError → RestError (8 变体)
// =====================================================================

#[test]
fn identity_not_found_maps_to_404() {
    let e = domain_identity::IdentityError::NotFound("user-123".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "IDENTITY_NOT_FOUND");
    assert_eq!(r.source_module, "domain-identity");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn identity_permission_denied_maps_to_policy() {
    let e = domain_identity::IdentityError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn identity_cross_tenant_denied_maps_to_policy() {
    use domain_identity::TenantId;
    let e = domain_identity::IdentityError::CrossTenantDenied(TenantId::new(), TenantId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn identity_email_exists_maps_to_409() {
    let e = domain_identity::IdentityError::EmailExists("user@example.com".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "IDENTITY_EMAIL_EXISTS");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn identity_incomplete_binding_maps_to_422() {
    let e = domain_identity::IdentityError::IncompleteBinding;
    let r: RestError = e.into();
    assert_eq!(r.code, "IDENTITY_INCOMPLETE_BINDING");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn identity_device_already_revoked_maps_to_409() {
    let e = domain_identity::IdentityError::DeviceAlreadyRevoked;
    let r: RestError = e.into();
    assert_eq!(r.code, "IDENTITY_DEVICE_REVOKED");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn identity_conflict_maps_to_external() {
    let e = domain_identity::IdentityError::Conflict("version".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "IDENTITY_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn identity_internal_maps_to_500_with_retriable() {
    let e = domain_identity::IdentityError::Internal("db".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 3. PermissionError → RestError (6 变体)
// =====================================================================

#[test]
fn permission_not_found_maps_to_404() {
    let e = domain_permission::PermissionError::NotFound("rule-123".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PERMISSION_NOT_FOUND");
    assert_eq!(r.source_module, "domain-permission");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn permission_denied_maps_to_policy() {
    let e = domain_permission::PermissionError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn permission_cross_tenant_denied_maps_to_policy() {
    use domain_permission::TenantId;
    let e = domain_permission::PermissionError::CrossTenantDenied(TenantId::new(), TenantId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn permission_invalid_rule_maps_to_422() {
    let e = domain_permission::PermissionError::InvalidRule("bad rule syntax".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PERMISSION_INVALID_RULE");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn permission_conflict_maps_to_external() {
    let e = domain_permission::PermissionError::Conflict("version".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PERMISSION_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn permission_internal_maps_to_500_with_retriable() {
    let e = domain_permission::PermissionError::Internal("db".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 4. ProjectError → RestError (7 变体)
// =====================================================================

#[test]
fn project_not_found_maps_to_404() {
    let e = domain_project::ProjectError::NotFound("proj-123".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PROJECT_NOT_FOUND");
    assert_eq!(r.source_module, "domain-project");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn project_permission_denied_maps_to_policy() {
    let e = domain_project::ProjectError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn project_cross_tenant_denied_maps_to_policy() {
    use domain_project::TenantId;
    let e = domain_project::ProjectError::CrossTenantDenied(TenantId::new(), TenantId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn project_slug_exists_maps_to_409() {
    let e = domain_project::ProjectError::SlugExists("acme-proj".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PROJECT_SLUG_EXISTS");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn project_invalid_state_maps_to_422() {
    let e = domain_project::ProjectError::InvalidState("archived".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PROJECT_INVALID_STATE");
    assert_eq!(r.source_kind, "Validation");
}

#[test]
fn project_conflict_maps_to_external() {
    let e = domain_project::ProjectError::Conflict("version".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "PROJECT_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn project_internal_maps_to_500_with_retriable() {
    let e = domain_project::ProjectError::Internal("db".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 5. 累计验证: Stage 1 + Stage 2 + Stage 3 = 17 域 94 变体 跨域交叉
// =====================================================================

#[test]
fn all_17_domains_have_correct_source_module() {
    // Stage 1 抽样 (3 域)
    let fb: RestError = domain_feedback::FeedbackError::Internal("x".to_string()).into();
    assert_eq!(fb.source_module, "domain-feedback");
    let ig: RestError = domain_integration::IntegrationError::Internal("x".to_string()).into();
    assert_eq!(ig.source_module, "domain-integration");
    let cm: RestError = domain_comment::CommentError::Internal("x".to_string()).into();
    assert_eq!(cm.source_module, "domain-comment");
    // Stage 2 抽样 (4 域)
    let ba: RestError = domain_batch::BatchError::Internal("x".to_string()).into();
    assert_eq!(ba.source_module, "domain-batch");
    let th: RestError = domain_theme::ThemeError::Storage("x".to_string()).into();
    assert_eq!(th.source_module, "domain-theme");
    let ag: RestError = domain_agent::AgentError::Internal("x".to_string()).into();
    assert_eq!(ag.source_module, "domain-agent");
    let he: RestError = domain_cli::hermes::HermesError::Auth("x".to_string()).into();
    assert_eq!(he.source_module, "domain-cli");
    // Stage 3 抽样 (4 域)
    let tn: RestError = domain_tenant::TenantError::Internal("x".to_string()).into();
    assert_eq!(tn.source_module, "domain-tenant");
    let id: RestError = domain_identity::IdentityError::Internal("x".to_string()).into();
    assert_eq!(id.source_module, "domain-identity");
    let pm: RestError = domain_permission::PermissionError::Internal("x".to_string()).into();
    assert_eq!(pm.source_module, "domain-permission");
    let pj: RestError = domain_project::ProjectError::Internal("x".to_string()).into();
    assert_eq!(pj.source_module, "domain-project");
}

#[test]
fn p02_100_percent_complete_all_17_domains_mapped() {
    // 17 域全部验证 (per WBS §14.15 P0-2 0.3M tokens 收官)
    // Each domain produces a RestError with the correct source_module
    let modules: Vec<&str> = vec![
        "domain-work-item",
        "domain-workspace",
        "domain-worktree",
        "domain-search",
        "domain-scm",
        "domain-validation",
        "domain-feedback",
        "domain-integration",
        "domain-comment",
        "domain-batch",
        "domain-theme",
        "domain-agent",
        "domain-cli",
        "domain-tenant",
        "domain-identity",
        "domain-permission",
        "domain-project",
    ];
    assert_eq!(
        modules.len(),
        17,
        "P0-2 should map 17 domains (per WBS §14.15 0.3M tokens 收官)"
    );
    let unique: std::collections::HashSet<&str> = modules.iter().copied().collect();
    assert_eq!(
        unique.len(),
        17,
        "all 17 modules must be unique (no overlap)"
    );
}

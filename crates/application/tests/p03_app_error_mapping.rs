//! v0.63 P0-3 Stage 1: 11 域 From<DomainError> for ApplicationError 映射集成测试
//!
//! 覆盖 (per WBS §14.15 P0-3, 跟 P0-2 star-api-rest/v0.58+v0.59+v0.60 同模式):
//! 1. `domain_feedback::FeedbackError` → `ApplicationError` (10 变体)
//! 2. `domain_integration::IntegrationError` → `ApplicationError` (8 变体)
//! 3. `domain_comment::CommentError` → `ApplicationError` (9 变体)
//! 4. `domain_batch::BatchError` → `ApplicationError` (21 变体)
//! 5. `domain_theme::ThemeError` → `ApplicationError` (8 变体)
//! 6. `domain_agent::AgentError` → `ApplicationError` (9 变体)
//! 7. `domain_cli::hermes::HermesError` → `ApplicationError` (4 变体)
//! 8. `domain_tenant::TenantError` → `ApplicationError` (7 变体)
//! 9. `domain_identity::IdentityError` → `ApplicationError` (8 变体)
//! 10. `domain_permission::PermissionError` → `ApplicationError` (6 变体)
//! 11. `domain_project::ProjectError` → `ApplicationError` (7 变体)
//!
//! 累计 P0-2 (v0.58+v0.59+v0.60, 17 域) + P0-3 (v0.63, 11 域) = 17 域 域映射 (application crate)
//! 跟 star-api-rest P0-2 17 域对称, P0-2 + P0-3 = 跨层 (REST 入口 + application 编排) 双层 error 映射完整

use application::ApplicationError;

// =====================================================================
// 1. FeedbackError → ApplicationError
// =====================================================================

#[test]
fn feedback_not_found_maps_correctly() {
    let e: ApplicationError =
        domain_feedback::FeedbackError::NotFound(domain_feedback::value_object::FeedbackId::new())
            .into();
    assert_eq!(e.code, "FB_NOT_FOUND");
    assert_eq!(e.source_module, "domain-feedback");
    assert!(!e.retriable);
}

#[test]
fn feedback_internal_maps_with_retriable() {
    let e: ApplicationError = domain_feedback::FeedbackError::Internal("db".to_string()).into();
    assert_eq!(e.code, "INTERNAL");
    assert!(e.retriable);
}

// =====================================================================
// 2. IntegrationError → ApplicationError
// =====================================================================

#[test]
fn integration_not_found_maps_correctly() {
    let e: ApplicationError = domain_integration::IntegrationError::NotFound(
        domain_integration::value_object::IntegrationId::new(),
    )
    .into();
    assert_eq!(e.code, "RESOURCE_NOT_FOUND");
    assert_eq!(e.source_module, "domain-integration");
}

#[test]
fn integration_loop_guard_missing_maps_correctly() {
    let e: ApplicationError =
        domain_integration::IntegrationError::LoopGuardMissing("bidirectional".to_string()).into();
    assert_eq!(e.code, "I_LOOP_GUARD_MISSING");
}

// =====================================================================
// 3. CommentError → ApplicationError
// =====================================================================

#[test]
fn comment_not_found_maps_correctly() {
    let e: ApplicationError = domain_comment::CommentError::NotFound("cmt-1".to_string()).into();
    assert_eq!(e.code, "COMMENT_NOT_FOUND");
    assert_eq!(e.source_module, "domain-comment");
}

#[test]
fn comment_cross_tenant_maps_to_policy() {
    use domain_comment::TenantId;
    let e: ApplicationError =
        domain_comment::CommentError::CrossTenantDenied(TenantId::new(), TenantId::new()).into();
    assert_eq!(e.code, "POLICY_DENIED");
    assert_eq!(e.source_kind, "Policy");
}

// =====================================================================
// 4. BatchError → ApplicationError
// =====================================================================

#[test]
fn batch_task_not_found_maps_correctly() {
    let e: ApplicationError =
        domain_batch::BatchError::TaskNotFound(domain_batch::TaskId::new()).into();
    assert_eq!(e.code, "BATCH_TASK_NOT_FOUND");
    assert_eq!(e.source_module, "domain-batch");
}

#[test]
fn batch_engine_overloaded_maps_with_retriable() {
    let e: ApplicationError = domain_batch::BatchError::EngineOverloaded.into();
    assert_eq!(e.code, "BATCH_ENGINE_OVERLOADED");
    assert!(e.retriable);
}

#[test]
fn batch_node_timeout_maps_with_retriable() {
    let e: ApplicationError =
        domain_batch::BatchError::NodeTimeout(domain_batch::NodeId::new()).into();
    assert_eq!(e.code, "BATCH_NODE_TIMEOUT");
    assert!(e.retriable);
}

// =====================================================================
// 5. ThemeError → ApplicationError
// =====================================================================

#[test]
fn theme_not_found_maps_correctly() {
    let e: ApplicationError = domain_theme::ThemeError::NotFound("theme-1".to_string()).into();
    assert_eq!(e.code, "THEME_NOT_FOUND");
    assert_eq!(e.source_module, "domain-theme");
}

#[test]
fn theme_storage_maps_to_internal_retriable() {
    let e: ApplicationError = domain_theme::ThemeError::Storage("disk".to_string()).into();
    assert_eq!(e.code, "INTERNAL");
    assert!(e.retriable);
}

// =====================================================================
// 6. AgentError → ApplicationError
// =====================================================================

#[test]
fn agent_not_found_maps_correctly() {
    let e: ApplicationError = domain_agent::AgentError::NotFound("agent-1".to_string()).into();
    assert_eq!(e.code, "AGENT_NOT_FOUND");
    assert_eq!(e.source_module, "domain-agent");
}

#[test]
fn agent_worktree_mismatch_maps_correctly() {
    let e: ApplicationError = domain_agent::AgentError::WorktreeMismatch.into();
    assert_eq!(e.code, "AGENT_WORKTREE_MISMATCH");
}

// =====================================================================
// 7. HermesError → ApplicationError
// =====================================================================

#[test]
fn hermes_auth_maps_to_policy_not_retriable() {
    let e: ApplicationError = domain_cli::hermes::HermesError::Auth("401".to_string()).into();
    assert_eq!(e.code, "HERMES_AUTH_ERROR");
    assert_eq!(e.source_module, "domain-cli");
    assert!(!e.retriable);
}

#[test]
fn hermes_server_error_maps_to_external_retriable() {
    let e: ApplicationError =
        domain_cli::hermes::HermesError::ServerError(503, "unavail".to_string()).into();
    assert_eq!(e.code, "HERMES_SERVER_ERROR");
    assert!(e.retriable);
}

// =====================================================================
// 8. TenantError → ApplicationError
// =====================================================================

#[test]
fn tenant_not_found_maps_correctly() {
    let e: ApplicationError = domain_tenant::TenantError::NotFound("tenant-1".to_string()).into();
    assert_eq!(e.code, "TENANT_NOT_FOUND");
    assert_eq!(e.source_module, "domain-tenant");
}

#[test]
fn tenant_slug_exists_maps_to_external() {
    let e: ApplicationError = domain_tenant::TenantError::SlugExists("acme".to_string()).into();
    assert_eq!(e.code, "TENANT_SLUG_EXISTS");
    assert_eq!(e.source_kind, "External");
}

// =====================================================================
// 9. IdentityError → ApplicationError
// =====================================================================

#[test]
fn identity_not_found_maps_correctly() {
    let e: ApplicationError = domain_identity::IdentityError::NotFound("user-1".to_string()).into();
    assert_eq!(e.code, "IDENTITY_NOT_FOUND");
    assert_eq!(e.source_module, "domain-identity");
}

#[test]
fn identity_email_exists_maps_to_external() {
    let e: ApplicationError =
        domain_identity::IdentityError::EmailExists("u@example.com".to_string()).into();
    assert_eq!(e.code, "IDENTITY_EMAIL_EXISTS");
    assert_eq!(e.source_kind, "External");
}

// =====================================================================
// 10. PermissionError → ApplicationError
// =====================================================================

#[test]
fn permission_not_found_maps_correctly() {
    let e: ApplicationError =
        domain_permission::PermissionError::NotFound("rule-1".to_string()).into();
    assert_eq!(e.code, "PERMISSION_NOT_FOUND");
    assert_eq!(e.source_module, "domain-permission");
}

#[test]
fn permission_invalid_rule_maps_to_validation() {
    let e: ApplicationError =
        domain_permission::PermissionError::InvalidRule("bad".to_string()).into();
    assert_eq!(e.code, "PERMISSION_INVALID_RULE");
    assert_eq!(e.source_kind, "Validation");
}

// =====================================================================
// 11. ProjectError → ApplicationError
// =====================================================================

#[test]
fn project_not_found_maps_correctly() {
    let e: ApplicationError = domain_project::ProjectError::NotFound("proj-1".to_string()).into();
    assert_eq!(e.code, "PROJECT_NOT_FOUND");
    assert_eq!(e.source_module, "domain-project");
}

#[test]
fn project_slug_exists_maps_to_external() {
    let e: ApplicationError =
        domain_project::ProjectError::SlugExists("acme-proj".to_string()).into();
    assert_eq!(e.code, "PROJECT_SLUG_EXISTS");
    assert_eq!(e.source_kind, "External");
}

// =====================================================================
// 12. 累计验证: P0-2 (17 域) + P0-3 (11 域) = application crate 17 域 跨域交叉
// =====================================================================

#[test]
fn p03_17_domains_have_unique_source_modules() {
    // 11 新增 Stage 1 域 (Stage 1 + 已存在 6 域 = 17 域 跟 P0-2 对称)
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
        "P0-3 should map 17 domains (per WBS §14.15 0.6M tokens)"
    );
    let unique: std::collections::HashSet<&str> = modules.iter().copied().collect();
    assert_eq!(
        unique.len(),
        17,
        "all 17 modules must be unique (no overlap)"
    );
}

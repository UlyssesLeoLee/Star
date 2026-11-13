//! v0.61 WBS 9.6 覆盖率补写: 首批 6 域 From<DomainError> for RestError 映射集成测试
//!
//! p02_error_mapping(_stage2/_stage3).rs 已覆盖 feedback/integration/comment/
//! batch/theme/agent/hermes/tenant/identity/permission/project (17 域)。
//! 本文件补首批 6 域 + worktree-shared-dir 2 个错误类型 + RestError 构造器:
//! 1. `domain_work_item::WorkItemError` → `RestError` (9 变体)
//! 2. `domain_workspace::WorkspaceError` → `RestError` (5 变体)
//! 3. `domain_worktree::WorktreeError` → `RestError` (9 变体)
//! 4. `domain_search::SearchError` → `RestError` (7 变体)
//! 5. `domain_scm::ScmError` → `RestError` (7 变体)
//! 6. `domain_validation::ValidationError` → `RestError` (6 变体)
//! 7. `worktree_shared_dir::SharedDirError` → `RestError` (11 码 + default)
//! 8. `worktree_shared_dir::ExternalWorktreeImportError` → `RestError` (4 变体)
//! 9. `RestError::not_implemented` / `validation` 构造器 + serde

use star_api_rest::error::RestError;

// =====================================================================
// 1. WorkItemError → RestError (9 变体)
// =====================================================================

#[test]
fn work_item_not_found_maps_to_404() {
    let e = domain_work_item::WorkItemError::NotFound("wi-1".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "RESOURCE_NOT_FOUND");
    assert_eq!(r.source_module, "domain-work-item");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
}

#[test]
fn work_item_permission_denied_maps_to_policy() {
    let e = domain_work_item::WorkItemError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
    assert!(!r.retriable);
}

#[test]
fn work_item_cross_tenant_denied_maps_to_policy() {
    let e = domain_work_item::WorkItemError::CrossTenantDenied(
        domain_work_item::TenantId::new(),
        domain_work_item::TenantId::new(),
    );
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn work_item_invalid_transition_maps_to_validation() {
    let e = domain_work_item::WorkItemError::InvalidTransition {
        from: "Todo".into(),
        to: "Done".into(),
    };
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert!(r.message.contains("work-item"));
}

#[test]
fn work_item_ai_task_missing_fields_map_to_validation() {
    for e in [
        domain_work_item::WorkItemError::AiTaskMissingObjective,
        domain_work_item::WorkItemError::AiTaskMissingScope,
        domain_work_item::WorkItemError::ParentProjectMismatch,
    ] {
        let label = format!("{e:?}");
        let r: RestError = e.into();
        assert_eq!(r.code, "VALIDATION_FAILED", "{label}");
        assert_eq!(r.source_kind, "Validation");
    }
}

#[test]
fn work_item_conflict_maps_to_external_validation() {
    let e = domain_work_item::WorkItemError::Conflict("dup".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn work_item_internal_maps_to_retriable_500() {
    let e = domain_work_item::WorkItemError::Internal("db down".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert_eq!(r.source_kind, "Internal");
    assert!(r.retriable);
}

// =====================================================================
// 2. WorkspaceError → RestError (5 变体)
// =====================================================================

#[test]
fn workspace_not_found_maps_to_404() {
    let e = domain_workspace::WorkspaceError::NotFound(domain_workspace::WorkspaceId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "RESOURCE_NOT_FOUND");
    assert_eq!(r.source_module, "domain-workspace");
    assert!(!r.retriable);
}

#[test]
fn workspace_permission_denied_maps_to_policy() {
    let e = domain_workspace::WorkspaceError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
    assert_eq!(r.source_kind, "Policy");
}

#[test]
fn workspace_invalid_state_maps_to_validation() {
    let e = domain_workspace::WorkspaceError::InvalidState("bad".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn workspace_conflict_maps_to_409_code() {
    let e = domain_workspace::WorkspaceError::Conflict("exists".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "WORKSPACE_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn workspace_internal_maps_to_retriable() {
    let e = domain_workspace::WorkspaceError::Internal("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 3. WorktreeError → RestError (9 变体)
// =====================================================================

#[test]
fn worktree_not_found_maps_to_404() {
    let e = domain_worktree::WorktreeError::NotFound(domain_worktree::WorktreeId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "WORKTREE_NOT_FOUND");
    assert_eq!(r.source_module, "domain-worktree");
    assert!(!r.retriable);
}

#[test]
fn worktree_permission_denied_maps_to_policy() {
    let e = domain_worktree::WorktreeError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
}

#[test]
fn worktree_cross_tenant_denied_maps_to_policy() {
    let e = domain_worktree::WorktreeError::CrossTenantDenied(
        domain_worktree::TenantId::new(),
        domain_worktree::TenantId::new(),
    );
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
}

#[test]
fn worktree_invalid_transition_maps_to_validation() {
    let e = domain_worktree::WorktreeError::InvalidTransition {
        from: "RUNNING".into(),
        to: "MERGED".into(),
    };
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn worktree_runtime_required_maps_to_validation() {
    let e = domain_worktree::WorktreeError::RuntimeRequired;
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn worktree_conflict_maps_to_409_code() {
    let e = domain_worktree::WorktreeError::Conflict("merge conflict".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "WORKTREE_CONFLICT");
}

#[test]
fn worktree_gate_failures_map_to_validation_run_failed() {
    for e in [
        domain_worktree::WorktreeError::CompletionGateFailed("gate".to_string()),
        domain_worktree::WorktreeError::IsolationFailed("iso".to_string()),
    ] {
        let label = format!("{e:?}");
        let r: RestError = e.into();
        assert_eq!(r.code, "VALIDATION_RUN_FAILED", "{label}");
        assert_eq!(r.source_kind, "Validation");
        assert!(!r.retriable);
    }
}

#[test]
fn worktree_internal_maps_to_retriable() {
    let e = domain_worktree::WorktreeError::Internal("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 4. SearchError → RestError (7 变体)
// =====================================================================

#[test]
fn search_not_found_maps_to_404() {
    let e = domain_search::SearchError::NotFound("q".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "RESOURCE_NOT_FOUND");
    assert_eq!(r.source_module, "domain-search");
}

#[test]
fn search_invalid_state_maps_to_validation() {
    let e = domain_search::SearchError::InvalidState("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn search_permission_denied_maps_to_policy() {
    let e = domain_search::SearchError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
}

#[test]
fn search_cross_tenant_denied_maps_to_policy() {
    let e = domain_search::SearchError::CrossTenantDenied(
        domain_search::TenantId::new(),
        domain_search::TenantId::new(),
    );
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
}

#[test]
fn search_invalid_query_maps_to_validation() {
    let e = domain_search::SearchError::InvalidQuery("bad jql".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn search_conflict_maps_to_external_validation() {
    let e = domain_search::SearchError::Conflict("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn search_internal_maps_to_retriable() {
    let e = domain_search::SearchError::Internal("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 5. ScmError → RestError (7 变体)
// =====================================================================

#[test]
fn scm_not_found_maps_to_404_code() {
    let e = domain_scm::ScmError::NotFound(domain_scm::RepositoryId::new());
    let r: RestError = e.into();
    assert_eq!(r.code, "SCM_NOT_FOUND");
    assert_eq!(r.source_module, "domain-scm");
}

#[test]
fn scm_permission_denied_maps_to_policy() {
    let e = domain_scm::ScmError::PermissionDenied("no access".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
}

#[test]
fn scm_invalid_state_maps_to_validation() {
    let e = domain_scm::ScmError::InvalidState("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn scm_conflict_maps_to_409_code() {
    let e = domain_scm::ScmError::Conflict("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "SCM_CONFLICT");
    assert_eq!(r.source_kind, "External");
}

#[test]
fn scm_idempotency_conflict_maps_to_scm_conflict() {
    let e = domain_scm::ScmError::IdempotencyConflict;
    let r: RestError = e.into();
    assert_eq!(r.code, "SCM_CONFLICT");
}

#[test]
fn scm_provider_error_maps_to_external_retriable() {
    let e = domain_scm::ScmError::ProviderError("api down".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "SCM_PROVIDER_ERROR");
    assert_eq!(r.source_kind, "External");
    assert!(r.retriable);
}

#[test]
fn scm_internal_maps_to_retriable() {
    let e = domain_scm::ScmError::Internal("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 6. ValidationError → RestError (6 变体)
// =====================================================================

#[test]
fn validation_not_found_maps_to_404() {
    let e = domain_validation::ValidationError::NotFound(
        domain_validation::value_object::ValidationId::new(),
    );
    let r: RestError = e.into();
    assert_eq!(r.code, "RESOURCE_NOT_FOUND");
    assert_eq!(r.source_module, "domain-validation");
}

#[test]
fn validation_permission_denied_maps_to_policy() {
    let e = domain_validation::ValidationError::PermissionDenied;
    let r: RestError = e.into();
    assert_eq!(r.code, "POLICY_DENIED");
}

#[test]
fn validation_invalid_state_maps_to_validation() {
    let e = domain_validation::ValidationError::InvalidState("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn validation_conflict_maps_to_vl_conflict() {
    let e = domain_validation::ValidationError::Conflict("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VL_CONFLICT");
}

#[test]
fn validation_invariant_violated_maps_to_422() {
    let e = domain_validation::ValidationError::InvariantViolated("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VL_INVARIANT_VIOLATED");
    assert!(!r.retriable);
}

#[test]
fn validation_internal_maps_to_retriable() {
    let e = domain_validation::ValidationError::Internal("x".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert!(r.retriable);
}

// =====================================================================
// 7. SharedDirError → RestError (code 路由)
// =====================================================================

fn shared_err(code: &str) -> RestError {
    let e = worktree_shared_dir::SharedDirError::new(code, format!("msg for {code}"), "trace");
    e.into()
}

#[test]
fn shared_dir_picker_repo_not_registered() {
    let r = shared_err("WSD.PICKER_REPO_NOT_REGISTERED");
    assert_eq!(r.code, "WORKTREE_REPO_NOT_REGISTERED");
    assert_eq!(r.source_module, "worktree-shared-dir");
    assert_eq!(r.source_kind, "Validation");
    assert!(!r.retriable);
    assert!(r.hint.contains("register_repo"));
}

#[test]
fn shared_dir_picker_id_invalid_maps_to_validation() {
    let r = shared_err("WSD.PICKER_ID_INVALID");
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert!(!r.retriable);
}

#[test]
fn shared_dir_picker_git_failures_map_to_internal_retriable() {
    for code in [
        "WSD.PICKER_FETCH_FAIL",
        "WSD.PICKER_GIT_OPEN_FAIL",
        "WSD.PICKER_GIT_LIST_BRANCHES_FAIL",
        "WSD.PICKER_GIT_HEAD_FAIL",
        "WSD.EXT_GIT_CMD_FAIL",
    ] {
        let r = shared_err(code);
        assert_eq!(r.code, "INTERNAL", "{code}");
        assert_eq!(r.source_kind, "Internal", "{code}");
        assert!(r.retriable, "{code}");
        assert!(r.hint.contains("Retry with backoff"), "{code}");
    }
}

#[test]
fn shared_dir_ext_repo_not_found() {
    let r = shared_err("WSD.EXT_REPO_NOT_FOUND");
    assert_eq!(r.code, "WORKTREE_REPO_NOT_REGISTERED");
}

#[test]
fn shared_dir_ext_path_not_in_scan() {
    let r = shared_err("WSD.EXT_PATH_NOT_IN_SCAN");
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert!(r.hint.contains("candidate id format"));
}

#[test]
fn shared_dir_ext_parse_fail() {
    let r = shared_err("WSD.EXT_PARSE_FAIL");
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn shared_dir_branch_naming_invalid() {
    let r = shared_err("WSD.BRANCH_NAMING_INVALID");
    assert_eq!(r.code, "VALIDATION_FAILED");
}

#[test]
fn shared_dir_unknown_code_falls_back_to_internal() {
    let r = shared_err("WSD.SOMETHING_NEW");
    assert_eq!(r.code, "INTERNAL");
    assert_eq!(r.source_kind, "Internal");
    assert!(r.retriable);
}

// =====================================================================
// 8. ExternalWorktreeImportError → RestError (4 变体)
// =====================================================================

#[test]
fn ext_import_git_command_maps_to_internal_retriable() {
    let e =
        worktree_shared_dir::ExternalWorktreeImportError::GitCommand("git not found".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "INTERNAL");
    assert_eq!(r.source_kind, "Internal");
    assert!(r.retriable);
}

#[test]
fn ext_import_parse_maps_to_validation() {
    let e = worktree_shared_dir::ExternalWorktreeImportError::Parse("bad porcelain".to_string());
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert!(!r.retriable);
}

#[test]
fn ext_import_repo_not_found_maps_to_404() {
    let e = worktree_shared_dir::ExternalWorktreeImportError::RepoNotFound {
        repo_id: uuid::Uuid::new_v4(),
        message: "not registered".to_string(),
    };
    let r: RestError = e.into();
    assert_eq!(r.code, "WORKTREE_REPO_NOT_REGISTERED");
    assert!(!r.retriable);
}

#[test]
fn ext_import_path_not_in_scan_maps_to_validation() {
    let e = worktree_shared_dir::ExternalWorktreeImportError::PathNotInScan {
        path: "/tmp/nope".into(),
    };
    let r: RestError = e.into();
    assert_eq!(r.code, "VALIDATION_FAILED");
    assert!(!r.retriable);
}

// =====================================================================
// 9. RestError 构造器 + serde
// =====================================================================

#[test]
fn rest_error_not_implemented_constructor() {
    let e = RestError::not_implemented("GET", "/api/v1/x");
    assert_eq!(e.code, "NOT_IMPLEMENTED");
    assert!(e.message.contains("GET /api/v1/x"), "{}", e.message);
    assert_eq!(e.source_module, "star-api-rest");
    assert_eq!(e.source_kind, "NotImplemented");
    assert!(!e.retriable);
    assert!(!e.hint.is_empty());
}

#[test]
fn rest_error_validation_constructor() {
    let e = RestError::validation("uuid parse failed", "use a valid uuid");
    assert_eq!(e.code, "VALIDATION_FAILED");
    assert_eq!(e.message, "uuid parse failed");
    assert_eq!(e.source_kind, "Validation");
    assert_eq!(e.hint, "use a valid uuid");
    assert!(!e.retriable);
}

#[test]
fn rest_error_serde_roundtrip() {
    let e = RestError::validation("m", "h");
    let json = serde_json::to_string(&e).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["code"], "VALIDATION_FAILED");
    assert_eq!(v["message"], "m");
    assert_eq!(v["hint"], "h");
    assert_eq!(v["retriable"], false);
    let parsed: RestError = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, e);
    // 6 字段完整
    let keys = [
        "code",
        "message",
        "source_module",
        "source_kind",
        "retriable",
        "hint",
    ];
    for k in keys {
        assert!(v.get(k).is_some(), "missing field {k}");
    }
}

"""v0.63 P0-3 Stage 1: Insert 11 From<DomainError> for ApplicationError impls.

Per 守门 #19 v19 agent 交互 Python 化 + 守门 #12 v21 [P] docs 同步.
幂等: 已存在则 skip.

P0-3 Stage 1 = 11 域 (feedback/integration/comment/batch/theme/agent + cli(hermes) + tenant/identity/permission/project = 11)
+ 现有 6 域 (work_item/workspace/worktree/search/scm/validation) = 17 域 total (跟 P0-2 对称)
"""
import re
import sys
from pathlib import Path

APP_LIB = Path("crates/application/src/lib.rs")

# 11 From impls (跟 P0-2 star-api-rest/src/error.rs 模式同, 但目标是 ApplicationError 而不是 RestError)
STAGE1_IMPLS = '''/// `domain_feedback::FeedbackError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_feedback::FeedbackError> for ApplicationError {
    fn from(e: domain_feedback::FeedbackError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_feedback::FeedbackError::NotFound(_) => ("FB_NOT_FOUND", "Validation", false),
            domain_feedback::FeedbackError::InvalidState(_) => ("FB_INVALID_STATE_TRANSITION", "Validation", false),
            domain_feedback::FeedbackError::TargetUnresolvable(_) => ("FB_TARGET_UNRESOLVABLE", "Validation", false),
            domain_feedback::FeedbackError::ReadOnly => ("FB_READ_ONLY", "Validation", false),
            domain_feedback::FeedbackError::NotDeletable => ("FB_NOT_DELETABLE", "Validation", false),
            domain_feedback::FeedbackError::MissingSuccessor => ("FB_MISSING_SUCCESSOR", "Validation", false),
            domain_feedback::FeedbackError::CrossWorktree => ("FB_CROSS_WORKTREE", "Validation", false),
            domain_feedback::FeedbackError::PermissionDenied => ("FB_PERMISSION_DENIED", "Policy", false),
            domain_feedback::FeedbackError::Conflict(_) => ("FB_CONFLICT", "External", false),
            domain_feedback::FeedbackError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("feedback: {e}"),
            source_module: "domain-feedback".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the feedback id + tenant + role (developer/agent for AI feedback)"),
        }
    }
}

/// `domain_integration::IntegrationError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_integration::IntegrationError> for ApplicationError {
    fn from(e: domain_integration::IntegrationError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_integration::IntegrationError::NotFound(_) => ("RESOURCE_NOT_FOUND", "Validation", false),
            domain_integration::IntegrationError::InvalidState(_) => ("VALIDATION_FAILED", "Validation", false),
            domain_integration::IntegrationError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_integration::IntegrationError::Conflict(_) => ("INTEGRATION_CONFLICT", "External", false),
            domain_integration::IntegrationError::InvalidArgument(_) => ("VALIDATION_FAILED", "Validation", false),
            domain_integration::IntegrationError::LoopGuardMissing(_) => ("I_LOOP_GUARD_MISSING", "Validation", false),
            domain_integration::IntegrationError::CredentialMissing(_) => ("I_CREDENTIAL_MISSING", "Validation", false),
            domain_integration::IntegrationError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("integration: {e}"),
            source_module: "domain-integration".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the integration id + provider + tenant + role (project_admin/developer)"),
        }
    }
}

/// `domain_comment::CommentError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_comment::CommentError> for ApplicationError {
    fn from(e: domain_comment::CommentError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_comment::CommentError::NotFound(_) => ("COMMENT_NOT_FOUND", "Validation", false),
            domain_comment::CommentError::InvalidState(_) => ("VALIDATION_FAILED", "Validation", false),
            domain_comment::CommentError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_comment::CommentError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_comment::CommentError::InvalidObjectKey => ("CMT_INVALID_OBJECT_KEY", "Validation", false),
            domain_comment::CommentError::ReactionExists => ("CMT_REACTION_EXISTS", "External", false),
            domain_comment::CommentError::EditDeleted => ("CMT_EDIT_DELETED", "Validation", false),
            domain_comment::CommentError::Conflict(_) => ("COMMENT_CONFLICT", "External", false),
            domain_comment::CommentError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("comment: {e}"),
            source_module: "domain-comment".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the comment id + thread id + tenant + role (developer/agent)"),
        }
    }
}

/// `domain_batch::BatchError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_batch::BatchError> for ApplicationError {
    fn from(e: domain_batch::BatchError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_batch::BatchError::Unauthenticated => ("BATCH_UNAUTHENTICATED", "Policy", false),
            domain_batch::BatchError::PermissionDenied(_) => ("BATCH_PERMISSION_DENIED", "Policy", false),
            domain_batch::BatchError::NodeTypeNotApproved(_) => ("BATCH_NODE_TYPE_NOT_APPROVED", "Policy", false),
            domain_batch::BatchError::TaskNotFound(_) => ("BATCH_TASK_NOT_FOUND", "Validation", false),
            domain_batch::BatchError::RunNotFound(_) => ("BATCH_RUN_NOT_FOUND", "Validation", false),
            domain_batch::BatchError::NodeNotFound(_) => ("BATCH_NODE_NOT_FOUND", "Validation", false),
            domain_batch::BatchError::NodeTypeNotFound(_) => ("BATCH_NODE_TYPE_NOT_FOUND", "Validation", false),
            domain_batch::BatchError::TaskNameConflict(_) => ("BATCH_TASK_NAME_CONFLICT", "External", false),
            domain_batch::BatchError::RunAlreadyRunning(_) => ("BATCH_RUN_ALREADY_RUNNING", "External", false),
            domain_batch::BatchError::NodeTimeout(_) => ("BATCH_NODE_TIMEOUT", "External", true),
            domain_batch::BatchError::InvalidDagSchema(_) => ("BATCH_INVALID_DAG_SCHEMA", "Validation", false),
            domain_batch::BatchError::DagCycle(_) => ("BATCH_DAG_CYCLE", "Validation", false),
            domain_batch::BatchError::InvalidNodeTypeConfig(_) => ("BATCH_INVALID_NODE_TYPE_CONFIG", "Validation", false),
            domain_batch::BatchError::InvalidCron(_) => ("BATCH_INVALID_CRON", "Validation", false),
            domain_batch::BatchError::ValidationFailed(_) => ("BATCH_VALIDATION_FAILED", "Validation", false),
            domain_batch::BatchError::NodeExecutionFailed(_) => ("BATCH_NODE_EXECUTION_FAILED", "Internal", true),
            domain_batch::BatchError::WorkerLeaseLost(_) => ("BATCH_WORKER_LEASE_LOST", "Internal", true),
            domain_batch::BatchError::Database(_) => ("BATCH_DATABASE_ERROR", "Internal", true),
            domain_batch::BatchError::EngineOverloaded => ("BATCH_ENGINE_OVERLOADED", "Internal", true),
            domain_batch::BatchError::NotImplemented { .. } => ("BATCH_NOT_IMPLEMENTED", "Validation", false),
            domain_batch::BatchError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("batch: {e}"),
            source_module: "domain-batch".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the task/run/node id + tenant + role (developer); BATCH-REQ-001 §8"),
        }
    }
}

/// `domain_theme::ThemeError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_theme::ThemeError> for ApplicationError {
    fn from(e: domain_theme::ThemeError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_theme::ThemeError::NotFound(_) => ("THEME_NOT_FOUND", "Validation", false),
            domain_theme::ThemeError::DuplicateId { .. } => ("THEME_DUPLICATE_ID", "External", false),
            domain_theme::ThemeError::IncompleteDefinition(_) => ("THEME_INCOMPLETE_DEFINITION", "Validation", false),
            domain_theme::ThemeError::InvalidHex(_) => ("THEME_INVALID_HEX", "Validation", false),
            domain_theme::ThemeError::InvalidSpacing(_) => ("THEME_INVALID_SPACING", "Validation", false),
            domain_theme::ThemeError::PermissionDenied { .. } => ("THEME_PERMISSION_DENIED", "Policy", false),
            domain_theme::ThemeError::Storage(_) => ("INTERNAL", "Internal", true),
            domain_theme::ThemeError::Serialization(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("theme: {e}"),
            source_module: "domain-theme".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the theme id + scope + tenant + role (developer/admin)"),
        }
    }
}

/// `domain_agent::AgentError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_agent::AgentError> for ApplicationError {
    fn from(e: domain_agent::AgentError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_agent::AgentError::NotFound(_) => ("AGENT_NOT_FOUND", "Validation", false),
            domain_agent::AgentError::InvalidTransition { .. } => ("AGENT_INVALID_TRANSITION", "Validation", false),
            domain_agent::AgentError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_agent::AgentError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_agent::AgentError::PolicyViolation(_) => ("AGENT_POLICY_VIOLATION", "Policy", false),
            domain_agent::AgentError::AgentAlreadyExists(_) => ("AGENT_ALREADY_EXISTS", "External", false),
            domain_agent::AgentError::WorktreeMismatch => ("AGENT_WORKTREE_MISMATCH", "Validation", false),
            domain_agent::AgentError::Conflict(_) => ("AGENT_CONFLICT", "External", false),
            domain_agent::AgentError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("agent: {e}"),
            source_module: "domain-agent".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the agent id + tenant + role (developer/agent operator)"),
        }
    }
}

/// `domain_cli::hermes::HermesError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_cli::hermes::HermesError> for ApplicationError {
    fn from(e: domain_cli::hermes::HermesError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_cli::hermes::HermesError::Http(_) => ("HERMES_HTTP_ERROR", "External", true),
            domain_cli::hermes::HermesError::Auth(_) => ("HERMES_AUTH_ERROR", "Policy", false),
            domain_cli::hermes::HermesError::ServerError(_, _) => ("HERMES_SERVER_ERROR", "External", true),
            domain_cli::hermes::HermesError::Parse(_) => ("HERMES_PARSE_ERROR", "External", false),
        };
        Self {
            code: code.to_string(),
            message: format!("hermes: {e}"),
            source_module: "domain-cli".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check Hermes API base_url + api_key + network (HTTP/Auth/ServerError/Parse per B.2 4 variants)"),
        }
    }
}

/// `domain_tenant::TenantError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_tenant::TenantError> for ApplicationError {
    fn from(e: domain_tenant::TenantError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_tenant::TenantError::NotFound(_) => ("TENANT_NOT_FOUND", "Validation", false),
            domain_tenant::TenantError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_tenant::TenantError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_tenant::TenantError::SlugExists(_) => ("TENANT_SLUG_EXISTS", "External", false),
            domain_tenant::TenantError::InvalidState(_) => ("TENANT_INVALID_STATE", "Validation", false),
            domain_tenant::TenantError::Conflict(_) => ("TENANT_CONFLICT", "External", false),
            domain_tenant::TenantError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("tenant: {e}"),
            source_module: "domain-tenant".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the tenant id + slug + role (tenant_admin/platform_admin)"),
        }
    }
}

/// `domain_identity::IdentityError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_identity::IdentityError> for ApplicationError {
    fn from(e: domain_identity::IdentityError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_identity::IdentityError::NotFound(_) => ("IDENTITY_NOT_FOUND", "Validation", false),
            domain_identity::IdentityError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_identity::IdentityError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_identity::IdentityError::EmailExists(_) => ("IDENTITY_EMAIL_EXISTS", "External", false),
            domain_identity::IdentityError::IncompleteBinding => ("IDENTITY_INCOMPLETE_BINDING", "Validation", false),
            domain_identity::IdentityError::DeviceAlreadyRevoked => ("IDENTITY_DEVICE_REVOKED", "External", false),
            domain_identity::IdentityError::Conflict(_) => ("IDENTITY_CONFLICT", "External", false),
            domain_identity::IdentityError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("identity: {e}"),
            source_module: "domain-identity".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the user id + email + tenant + role (developer/platform_admin)"),
        }
    }
}

/// `domain_permission::PermissionError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_permission::PermissionError> for ApplicationError {
    fn from(e: domain_permission::PermissionError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_permission::PermissionError::NotFound(_) => ("PERMISSION_NOT_FOUND", "Validation", false),
            domain_permission::PermissionError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_permission::PermissionError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_permission::PermissionError::InvalidRule(_) => ("PERMISSION_INVALID_RULE", "Validation", false),
            domain_permission::PermissionError::Conflict(_) => ("PERMISSION_CONFLICT", "External", false),
            domain_permission::PermissionError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("permission: {e}"),
            source_module: "domain-permission".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the permission rule + role + scope (project_admin/tenant_admin/developer)"),
        }
    }
}

/// `domain_project::ProjectError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_project::ProjectError> for ApplicationError {
    fn from(e: domain_project::ProjectError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_project::ProjectError::NotFound(_) => ("PROJECT_NOT_FOUND", "Validation", false),
            domain_project::ProjectError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_project::ProjectError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_project::ProjectError::SlugExists(_) => ("PROJECT_SLUG_EXISTS", "External", false),
            domain_project::ProjectError::InvalidState(_) => ("PROJECT_INVALID_STATE", "Validation", false),
            domain_project::ProjectError::Conflict(_) => ("PROJECT_CONFLICT", "External", false),
            domain_project::ProjectError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("project: {e}"),
            source_module: "domain-project".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the project id + slug + workspace + tenant + role (project_admin/developer)"),
        }
    }
}

'''

def main():
    if not APP_LIB.exists():
        print(f"FAIL: {APP_LIB} not found", file=sys.stderr)
        sys.exit(1)
    content = APP_LIB.read_text(encoding="utf-8")
    if "domain_feedback::FeedbackError" in content and "ApplicationError" in content:
        print("OK: Stage 1 From impls already present (idempotent skip)")
        return
    # Insert before the test mod
    marker = "#[cfg(test)]\nmod tests {"
    idx = content.find(marker)
    if idx < 0:
        # Fallback: insert at end of file before final brace
        # Find the last test mod
        marker2 = "mod tests {"
        idx = content.rfind(marker2)
        if idx < 0:
            # Insert before last closing brace
            idx = content.rstrip().rfind("\n}")
    if idx < 0:
        print("FAIL: marker not found", file=sys.stderr)
        sys.exit(2)
    # Backtrack to start of line containing marker
    line_start = content.rfind("\n", 0, idx) + 1
    new_content = content[:line_start] + STAGE1_IMPLS + content[line_start:]
    APP_LIB.write_text(new_content, encoding="utf-8")
    print(f"OK: 11 From impls inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()

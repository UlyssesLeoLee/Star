"""v0.59 P0-2 Stage 2: Insert 4 From<DomainError> for RestError impls (batch/theme/agent/hermes)

Per 守门 #19 v19 agent 交互 Python 化 + 守门 #12 v21 [P] docs 同步.
幂等: 已存在则 skip.
"""
import re
import sys
from pathlib import Path

ERROR_RS = Path("crates/star-api-rest/src/error.rs")

# 4 From impls to insert (before the IntoResponse impl)
STAGE2_IMPLS = '''/// `domain_batch::BatchError` → `RestError` (per v0.59 P0-2 Stage 2, WBS §14.15)
///
/// BatchError 19 变体 (BA-001~016 + Unauthenticated + Internal + NotImplemented + ValidationFailed).
/// HTTP status 映射 per `BatchError::http_status()` (BATCH-REQ-001 §8):
/// - 401 Unauthenticated, 403 PermissionDenied/NodeTypeNotApproved,
/// - 404 NotFound 类, 408 NodeTimeout, 409 Conflict 类,
/// - 422 Validation 类, 500 Internal/DB/NodeExecutionFailed/WorkerLeaseLost,
/// - 503 EngineOverloaded, 501 NotImplemented
impl From<domain_batch::BatchError> for RestError {
    fn from(e: domain_batch::BatchError) -> Self {
        let (code, source_kind, retriable, http) = match &e {
            domain_batch::BatchError::Unauthenticated => {
                ("BATCH_UNAUTHENTICATED", "Policy", false, 401)
            }
            domain_batch::BatchError::PermissionDenied(_) => {
                ("BATCH_PERMISSION_DENIED", "Policy", false, 403)
            }
            domain_batch::BatchError::NodeTypeNotApproved(_) => {
                ("BATCH_NODE_TYPE_NOT_APPROVED", "Policy", false, 403)
            }
            domain_batch::BatchError::TaskNotFound(_) => {
                ("BATCH_TASK_NOT_FOUND", "Validation", false, 404)
            }
            domain_batch::BatchError::RunNotFound(_) => {
                ("BATCH_RUN_NOT_FOUND", "Validation", false, 404)
            }
            domain_batch::BatchError::NodeNotFound(_) => {
                ("BATCH_NODE_NOT_FOUND", "Validation", false, 404)
            }
            domain_batch::BatchError::NodeTypeNotFound(_) => {
                ("BATCH_NODE_TYPE_NOT_FOUND", "Validation", false, 404)
            }
            domain_batch::BatchError::TaskNameConflict(_) => {
                ("BATCH_TASK_NAME_CONFLICT", "External", false, 409)
            }
            domain_batch::BatchError::RunAlreadyRunning(_) => {
                ("BATCH_RUN_ALREADY_RUNNING", "External", false, 409)
            }
            domain_batch::BatchError::NodeTimeout(_) => {
                ("BATCH_NODE_TIMEOUT", "External", true, 408)
            }
            domain_batch::BatchError::InvalidDagSchema(_) => {
                ("BATCH_INVALID_DAG_SCHEMA", "Validation", false, 422)
            }
            domain_batch::BatchError::DagCycle(_) => {
                ("BATCH_DAG_CYCLE", "Validation", false, 422)
            }
            domain_batch::BatchError::InvalidNodeTypeConfig(_) => {
                ("BATCH_INVALID_NODE_TYPE_CONFIG", "Validation", false, 422)
            }
            domain_batch::BatchError::InvalidCron(_) => {
                ("BATCH_INVALID_CRON", "Validation", false, 422)
            }
            domain_batch::BatchError::ValidationFailed(_) => {
                ("BATCH_VALIDATION_FAILED", "Validation", false, 422)
            }
            domain_batch::BatchError::NodeExecutionFailed(_) => {
                ("BATCH_NODE_EXECUTION_FAILED", "Internal", true, 500)
            }
            domain_batch::BatchError::WorkerLeaseLost(_) => {
                ("BATCH_WORKER_LEASE_LOST", "Internal", true, 500)
            }
            domain_batch::BatchError::Database(_) => {
                ("BATCH_DATABASE_ERROR", "Internal", true, 500)
            }
            domain_batch::BatchError::EngineOverloaded => {
                ("BATCH_ENGINE_OVERLOADED", "Internal", true, 503)
            }
            domain_batch::BatchError::NotImplemented { .. } => {
                ("BATCH_NOT_IMPLEMENTED", "Validation", false, 501)
            }
            domain_batch::BatchError::Internal(_) => ("INTERNAL", "Internal", true, 500),
        };
        Self {
            code: code.to_string(),
            message: format!("batch: {e}"),
            source_module: "domain-batch".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!(
                "Check the task/run/node id + tenant + role (developer); HTTP {http} per BATCH-REQ-001 §8"
            ),
        }
    }
}

/// `domain_theme::ThemeError` → `RestError` (per v0.59 P0-2 Stage 2, WBS §14.15)
impl From<domain_theme::ThemeError> for RestError {
    fn from(e: domain_theme::ThemeError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_theme::ThemeError::NotFound(_) => ("THEME_NOT_FOUND", "Validation", false),
            domain_theme::ThemeError::DuplicateId { .. } => {
                ("THEME_DUPLICATE_ID", "External", false)
            }
            domain_theme::ThemeError::IncompleteDefinition(_) => {
                ("THEME_INCOMPLETE_DEFINITION", "Validation", false)
            }
            domain_theme::ThemeError::InvalidHex(_) => {
                ("THEME_INVALID_HEX", "Validation", false)
            }
            domain_theme::ThemeError::InvalidSpacing(_) => {
                ("THEME_INVALID_SPACING", "Validation", false)
            }
            domain_theme::ThemeError::PermissionDenied { .. } => {
                ("THEME_PERMISSION_DENIED", "Policy", false)
            }
            domain_theme::ThemeError::Storage(_) => ("INTERNAL", "Internal", true),
            domain_theme::ThemeError::Serialization(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("theme: {e}"),
            source_module: "domain-theme".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the theme id + scope + tenant + role (developer/admin)".to_string(),
        }
    }
}

/// `domain_agent::AgentError` → `RestError` (per v0.59 P0-2 Stage 2, WBS §14.15)
impl From<domain_agent::AgentError> for RestError {
    fn from(e: domain_agent::AgentError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_agent::AgentError::NotFound(_) => ("AGENT_NOT_FOUND", "Validation", false),
            domain_agent::AgentError::InvalidTransition { .. } => {
                ("AGENT_INVALID_TRANSITION", "Validation", false)
            }
            domain_agent::AgentError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_agent::AgentError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_agent::AgentError::PolicyViolation(_) => {
                ("AGENT_POLICY_VIOLATION", "Policy", false)
            }
            domain_agent::AgentError::AgentAlreadyExists(_) => {
                ("AGENT_ALREADY_EXISTS", "External", false)
            }
            domain_agent::AgentError::WorktreeMismatch => {
                ("AGENT_WORKTREE_MISMATCH", "Validation", false)
            }
            domain_agent::AgentError::Conflict(_) => ("AGENT_CONFLICT", "External", false),
            domain_agent::AgentError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("agent: {e}"),
            source_module: "domain-agent".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the agent id + tenant + role (developer/agent operator)".to_string(),
        }
    }
}

/// `domain_cli::hermes::HermesError` → `RestError` (per v0.59 P0-2 Stage 2, WBS §14.15)
///
/// HermesError 4 变体 per B.2 简化: Http (transient) / Auth (permanent) / ServerError (transient) / Parse (permanent)
impl From<domain_cli::hermes::HermesError> for RestError {
    fn from(e: domain_cli::hermes::HermesError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_cli::hermes::HermesError::Http(_) => ("HERMES_HTTP_ERROR", "External", true),
            domain_cli::hermes::HermesError::Auth(_) => ("HERMES_AUTH_ERROR", "Policy", false),
            domain_cli::hermes::HermesError::ServerError(_, _) => {
                ("HERMES_SERVER_ERROR", "External", true)
            }
            domain_cli::hermes::HermesError::Parse(_) => ("HERMES_PARSE_ERROR", "External", false),
        };
        Self {
            code: code.to_string(),
            message: format!("hermes: {e}"),
            source_module: "domain-cli".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check Hermes API base_url + api_key + network (HTTP/Auth/ServerError/Parse per B.2 4 variants)"
                .to_string(),
        }
    }
}

'''

def main():
    if not ERROR_RS.exists():
        print(f"FAIL: {ERROR_RS} not found", file=sys.stderr)
        sys.exit(1)
    content = ERROR_RS.read_text(encoding="utf-8")
    if "domain_batch::BatchError" in content:
        print("OK: Stage 2 From impls already present (idempotent skip)")
        return
    # Insert before `impl IntoResponse for RestError {`
    marker = "impl IntoResponse for RestError {"
    idx = content.find(marker)
    if idx < 0:
        print("FAIL: marker not found", file=sys.stderr)
        sys.exit(2)
    new_content = content[:idx] + STAGE2_IMPLS + content[idx:]
    # Also update the IntoResponse match arms to cover new codes
    # Replace the existing match block with extended version
    old_match_block = '''            "POLICY_DENIED" | "FB_PERMISSION_DENIED" => StatusCode::FORBIDDEN,
            "WORKTREE_CONFLICT"
            | "INTEGRATION_CONFLICT"
            | "COMMENT_CONFLICT"
            | "VL_CONFLICT"
            | "FB_CONFLICT"
            | "SCM_CONFLICT" => StatusCode::CONFLICT,
            "FB_INVALID_STATE_TRANSITION"
            | "FB_TARGET_UNRESOLVABLE"
            | "FB_READ_ONLY"
            | "FB_NOT_DELETABLE"
            | "FB_MISSING_SUCCESSOR"
            | "FB_CROSS_WORKTREE"
            | "I_LOOP_GUARD_MISSING"
            | "I_CREDENTIAL_MISSING"
            | "VL_INVARIANT_VIOLATED"
            | "SCM_NOT_FOUND"
            | "SCM_PROVIDER_ERROR" => StatusCode::UNPROCESSABLE_ENTITY,
            "INTERNAL" => StatusCode::INTERNAL_SERVER_ERROR,'''

    new_match_block = '''            "BATCH_UNAUTHENTICATED" => StatusCode::UNAUTHORIZED,
            "POLICY_DENIED" | "FB_PERMISSION_DENIED" | "AGENT_WORKTREE_MISMATCH" => {
                StatusCode::FORBIDDEN
            }
            "WORKTREE_CONFLICT"
            | "INTEGRATION_CONFLICT"
            | "COMMENT_CONFLICT"
            | "VL_CONFLICT"
            | "FB_CONFLICT"
            | "SCM_CONFLICT"
            | "BATCH_TASK_NAME_CONFLICT"
            | "BATCH_RUN_ALREADY_RUNNING"
            | "AGENT_CONFLICT"
            | "AGENT_ALREADY_EXISTS" => StatusCode::CONFLICT,
            "BATCH_NODE_TIMEOUT" => StatusCode::REQUEST_TIMEOUT,
            "BATCH_ENGINE_OVERLOADED" => StatusCode::SERVICE_UNAVAILABLE,
            "FB_INVALID_STATE_TRANSITION"
            | "FB_TARGET_UNRESOLVABLE"
            | "FB_READ_ONLY"
            | "FB_NOT_DELETABLE"
            | "FB_MISSING_SUCCESSOR"
            | "FB_CROSS_WORKTREE"
            | "I_LOOP_GUARD_MISSING"
            | "I_CREDENTIAL_MISSING"
            | "VL_INVARIANT_VIOLATED"
            | "SCM_NOT_FOUND"
            | "SCM_PROVIDER_ERROR"
            | "BATCH_INVALID_DAG_SCHEMA"
            | "BATCH_DAG_CYCLE"
            | "BATCH_INVALID_NODE_TYPE_CONFIG"
            | "BATCH_INVALID_CRON"
            | "BATCH_VALIDATION_FAILED"
            | "AGENT_INVALID_TRANSITION" => StatusCode::UNPROCESSABLE_ENTITY,
            "INTERNAL" | "BATCH_NODE_EXECUTION_FAILED" | "BATCH_WORKER_LEASE_LOST"
            | "BATCH_DATABASE_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
            "BATCH_NOT_IMPLEMENTED" => StatusCode::NOT_IMPLEMENTED,'''

    if old_match_block in new_content:
        new_content = new_content.replace(old_match_block, new_match_block)
        print("OK: IntoResponse match block updated")
    else:
        print("WARN: old match block not found (may have been changed); skipping match block update")

    ERROR_RS.write_text(new_content, encoding="utf-8")
    print(f"OK: 4 From impls inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()

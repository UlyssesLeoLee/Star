"""v0.60 P0-2 Stage 3: Insert 4 From<DomainError> for RestError impls (tenant/identity/permission/project)

Per 守门 #19 v19 agent 交互 Python 化 + 守门 #12 v21 [P] docs 同步.
幂等: 已存在则 skip.

P0-2 Stage 3 = 4 域 (tenant 7 + identity 8 + permission 6 + project 7 = 28 变体)
Stage 1 (v0.58) + Stage 2 (v0.59) + Stage 3 (v0.60) = 27 + 39 + 28 = 94 变体
P0-2 域映射 6 → 9 → 13 → **17 域** (含 cli hermes 算 1)
"""
import re
import sys
from pathlib import Path

ERROR_RS = Path("crates/star-api-rest/src/error.rs")

STAGE3_IMPLS = '''/// `domain_tenant::TenantError` → `RestError` (per v0.60 P0-2 Stage 3, WBS §14.15)
impl From<domain_tenant::TenantError> for RestError {
    fn from(e: domain_tenant::TenantError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_tenant::TenantError::NotFound(_) => ("TENANT_NOT_FOUND", "Validation", false),
            domain_tenant::TenantError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_tenant::TenantError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_tenant::TenantError::SlugExists(_) => ("TENANT_SLUG_EXISTS", "External", false),
            domain_tenant::TenantError::InvalidState(_) => {
                ("TENANT_INVALID_STATE", "Validation", false)
            }
            domain_tenant::TenantError::Conflict(_) => ("TENANT_CONFLICT", "External", false),
            domain_tenant::TenantError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("tenant: {e}"),
            source_module: "domain-tenant".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the tenant id + slug + role (tenant_admin/platform_admin)"
                .to_string(),
        }
    }
}

/// `domain_identity::IdentityError` → `RestError` (per v0.60 P0-2 Stage 3, WBS §14.15)
impl From<domain_identity::IdentityError> for RestError {
    fn from(e: domain_identity::IdentityError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_identity::IdentityError::NotFound(_) => {
                ("IDENTITY_NOT_FOUND", "Validation", false)
            }
            domain_identity::IdentityError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_identity::IdentityError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_identity::IdentityError::EmailExists(_) => {
                ("IDENTITY_EMAIL_EXISTS", "External", false)
            }
            domain_identity::IdentityError::IncompleteBinding => {
                ("IDENTITY_INCOMPLETE_BINDING", "Validation", false)
            }
            domain_identity::IdentityError::DeviceAlreadyRevoked => {
                ("IDENTITY_DEVICE_REVOKED", "External", false)
            }
            domain_identity::IdentityError::Conflict(_) => ("IDENTITY_CONFLICT", "External", false),
            domain_identity::IdentityError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("identity: {e}"),
            source_module: "domain-identity".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the user id + email + tenant + role (developer/platform_admin)"
                .to_string(),
        }
    }
}

/// `domain_permission::PermissionError` → `RestError` (per v0.60 P0-2 Stage 3, WBS §14.15)
impl From<domain_permission::PermissionError> for RestError {
    fn from(e: domain_permission::PermissionError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_permission::PermissionError::NotFound(_) => {
                ("PERMISSION_NOT_FOUND", "Validation", false)
            }
            domain_permission::PermissionError::PermissionDenied => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_permission::PermissionError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_permission::PermissionError::InvalidRule(_) => {
                ("PERMISSION_INVALID_RULE", "Validation", false)
            }
            domain_permission::PermissionError::Conflict(_) => {
                ("PERMISSION_CONFLICT", "External", false)
            }
            domain_permission::PermissionError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("permission: {e}"),
            source_module: "domain-permission".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the permission rule + role + scope (project_admin/tenant_admin/developer)"
                .to_string(),
        }
    }
}

/// `domain_project::ProjectError` → `RestError` (per v0.60 P0-2 Stage 3, WBS §14.15)
impl From<domain_project::ProjectError> for RestError {
    fn from(e: domain_project::ProjectError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_project::ProjectError::NotFound(_) => {
                ("PROJECT_NOT_FOUND", "Validation", false)
            }
            domain_project::ProjectError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_project::ProjectError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_project::ProjectError::SlugExists(_) => {
                ("PROJECT_SLUG_EXISTS", "External", false)
            }
            domain_project::ProjectError::InvalidState(_) => {
                ("PROJECT_INVALID_STATE", "Validation", false)
            }
            domain_project::ProjectError::Conflict(_) => ("PROJECT_CONFLICT", "External", false),
            domain_project::ProjectError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("project: {e}"),
            source_module: "domain-project".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the project id + slug + workspace + tenant + role (project_admin/developer)"
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
    if "domain_tenant::TenantError" in content:
        print("OK: Stage 3 From impls already present (idempotent skip)")
        return
    # Insert before `impl IntoResponse for RestError {`
    marker = "impl IntoResponse for RestError {"
    idx = content.find(marker)
    if idx < 0:
        print("FAIL: marker not found", file=sys.stderr)
        sys.exit(2)
    new_content = content[:idx] + STAGE3_IMPLS + content[idx:]
    # Also update the IntoResponse match arms to cover new codes
    old_match_block = '''            "RESOURCE_NOT_FOUND" | "WORKTREE_NOT_FOUND" | "FB_NOT_FOUND" | "COMMENT_NOT_FOUND"
            | "AGENT_NOT_FOUND" | "THEME_NOT_FOUND" | "BATCH_TASK_NOT_FOUND"
            | "BATCH_RUN_NOT_FOUND" | "BATCH_NODE_NOT_FOUND" | "BATCH_NODE_TYPE_NOT_FOUND" => {
                StatusCode::NOT_FOUND
            }'''
    new_match_block = '''            "RESOURCE_NOT_FOUND" | "WORKTREE_NOT_FOUND" | "FB_NOT_FOUND" | "COMMENT_NOT_FOUND"
            | "AGENT_NOT_FOUND" | "THEME_NOT_FOUND" | "BATCH_TASK_NOT_FOUND"
            | "BATCH_RUN_NOT_FOUND" | "BATCH_NODE_NOT_FOUND" | "BATCH_NODE_TYPE_NOT_FOUND"
            | "TENANT_NOT_FOUND" | "IDENTITY_NOT_FOUND" | "PERMISSION_NOT_FOUND"
            | "PROJECT_NOT_FOUND" => {
                StatusCode::NOT_FOUND
            }'''
    if old_match_block in new_content:
        new_content = new_content.replace(old_match_block, new_match_block)
        print("OK: IntoResponse NOT_FOUND block updated")
    else:
        print("WARN: NOT_FOUND block not found")
    # Also update 422 block
    old_422 = '''            | "BATCH_INVALID_DAG_SCHEMA"
            | "BATCH_DAG_CYCLE"
            | "BATCH_INVALID_NODE_TYPE_CONFIG"
            | "BATCH_INVALID_CRON"
            | "BATCH_VALIDATION_FAILED"
            | "AGENT_INVALID_TRANSITION" => StatusCode::UNPROCESSABLE_ENTITY,'''
    new_422 = '''            | "BATCH_INVALID_DAG_SCHEMA"
            | "BATCH_DAG_CYCLE"
            | "BATCH_INVALID_NODE_TYPE_CONFIG"
            | "BATCH_INVALID_CRON"
            | "BATCH_VALIDATION_FAILED"
            | "AGENT_INVALID_TRANSITION"
            | "TENANT_INVALID_STATE"
            | "IDENTITY_INCOMPLETE_BINDING"
            | "PERMISSION_INVALID_RULE"
            | "PROJECT_INVALID_STATE" => StatusCode::UNPROCESSABLE_ENTITY,'''
    if old_422 in new_content:
        new_content = new_content.replace(old_422, new_422)
        print("OK: IntoResponse 422 block updated")
    else:
        print("WARN: 422 block not found")
    # Update CONFLICT block
    old_conflict = '''            | "AGENT_CONFLICT"
            | "AGENT_ALREADY_EXISTS" => StatusCode::CONFLICT,'''
    new_conflict = '''            | "AGENT_CONFLICT"
            | "AGENT_ALREADY_EXISTS"
            | "TENANT_CONFLICT" | "TENANT_SLUG_EXISTS"
            | "IDENTITY_CONFLICT" | "IDENTITY_EMAIL_EXISTS" | "IDENTITY_DEVICE_REVOKED"
            | "PERMISSION_CONFLICT"
            | "PROJECT_CONFLICT" | "PROJECT_SLUG_EXISTS" => StatusCode::CONFLICT,'''
    if old_conflict in new_content:
        new_content = new_content.replace(old_conflict, new_conflict)
        print("OK: IntoResponse CONFLICT block updated")
    else:
        print("WARN: CONFLICT block not found")

    ERROR_RS.write_text(new_content, encoding="utf-8")
    print(f"OK: 4 From impls inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()

// SPDX-License-Identifier: MIT OR Apache-2.0
//! 6-field 错误模型 (per spec §2.4, 复用 MCP `agent-api/v1#Error`)
//!
//! 字段:
//! - `code` (SCREAMING_SNAKE_CASE 字符串, 24 个 per `star_mcp::error_code`)
//! - `message` (人类可读消息, 不暴露 secret / 内部 stack trace)
//! - `source_module` (e.g. `"domain-work-item"`)
//! - `source_kind` (e.g. `"NotFound"` / `"Validation"` / `"Unauthorized"`)
//! - `retriable` (bool, true → client 可重试)
//! - `hint` (可执行的修复提示)

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// 6-field REST 错误 (per spec §2.4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestError {
    /// 24 个 SCREAMING_SNAKE_CASE 错误码之一 (per `star_mcp::error_code`)
    pub code: String,
    /// 人类可读消息 (不暴露 secret / 内部 stack trace)
    pub message: String,
    /// 触发的 source module (e.g. `"domain-work-item"`)
    pub source_module: String,
    /// 触发的 source kind (e.g. `"NotFound"`)
    pub source_kind: String,
    /// client 是否可重试
    pub retriable: bool,
    /// 可执行的修复提示
    pub hint: String,
}

impl RestError {
    /// 业务端点未实装 (P2 阶段 worker 子代理实装时移除)
    pub fn not_implemented(method: &str, path: &str) -> Self {
        Self {
            code: "NOT_IMPLEMENTED".to_string(),
            message: format!("REST endpoint {method} {path} is not yet implemented (P2 phase, awaiting worker delegation per AGENTS.md §4 #20)"),
            source_module: "star-api-rest".to_string(),
            source_kind: "NotImplemented".to_string(),
            retriable: false,
            hint: "Wait for P2 phase implementation, or check spec at docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md".to_string(),
        }
    }

    /// 校验失败 (e.g. UUID 解析失败, 缺字段)
    pub fn validation(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code: "VALIDATION_FAILED".to_string(),
            message: message.into(),
            source_module: "star-api-rest".to_string(),
            source_kind: "Validation".to_string(),
            retriable: false,
            hint: hint.into(),
        }
    }
}

/// `domain_work_item::WorkItemError` → `RestError` (per star-mcp::error.rs 模式简化, source_kind = String)
impl From<domain_work_item::WorkItemError> for RestError {
    fn from(e: domain_work_item::WorkItemError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_work_item::WorkItemError::NotFound(_) => {
                ("RESOURCE_NOT_FOUND", "Validation", false)
            }
            domain_work_item::WorkItemError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_work_item::WorkItemError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_work_item::WorkItemError::InvalidTransition { .. } => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_work_item::WorkItemError::AiTaskMissingObjective
            | domain_work_item::WorkItemError::AiTaskMissingScope
            | domain_work_item::WorkItemError::ParentProjectMismatch => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_work_item::WorkItemError::Conflict(_) => {
                ("VALIDATION_FAILED", "External", false)
            }
            domain_work_item::WorkItemError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("work-item: {e}"),
            source_module: "domain-work-item".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the work-item id + tenant + role (developer/project_admin/tenant_admin)"
                .to_string(),
        }
    }
}

/// `domain_workspace::WorkspaceError` → `RestError`
impl From<domain_workspace::WorkspaceError> for RestError {
    fn from(e: domain_workspace::WorkspaceError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_workspace::WorkspaceError::NotFound(_) => {
                ("RESOURCE_NOT_FOUND", "Validation", false)
            }
            domain_workspace::WorkspaceError::PermissionDenied => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_workspace::WorkspaceError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_workspace::WorkspaceError::Conflict(_) => {
                ("WORKSPACE_CONFLICT", "External", false)
            }
            domain_workspace::WorkspaceError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("workspace: {e}"),
            source_module: "domain-workspace".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the workspace id + tenant + role (workspace_admin/tenant_admin)"
                .to_string(),
        }
    }
}

/// `domain_worktree::WorktreeError` → `RestError`
impl From<domain_worktree::WorktreeError> for RestError {
    fn from(e: domain_worktree::WorktreeError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_worktree::WorktreeError::NotFound(_) => {
                ("WORKTREE_NOT_FOUND", "Validation", false)
            }
            domain_worktree::WorktreeError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_worktree::WorktreeError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_worktree::WorktreeError::InvalidTransition { .. } => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_worktree::WorktreeError::RuntimeRequired => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_worktree::WorktreeError::Conflict(_) => ("WORKTREE_CONFLICT", "External", false),
            domain_worktree::WorktreeError::CompletionGateFailed(_)
            | domain_worktree::WorktreeError::IsolationFailed(_) => {
                ("VALIDATION_RUN_FAILED", "Validation", false)
            }
            domain_worktree::WorktreeError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("worktree: {e}"),
            source_module: "domain-worktree".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the worktree id + tenant + role (developer)".to_string(),
        }
    }
}

/// `domain_search::SearchError` → `RestError`
impl From<domain_search::SearchError> for RestError {
    fn from(e: domain_search::SearchError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_search::SearchError::NotFound(_) => ("RESOURCE_NOT_FOUND", "Validation", false),
            domain_search::SearchError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_search::SearchError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_search::SearchError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_search::SearchError::InvalidQuery(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_search::SearchError::Conflict(_) => ("VALIDATION_FAILED", "External", false),
            domain_search::SearchError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("search: {e}"),
            source_module: "domain-search".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check query + filters + tenant + role (developer/system:search-projector)"
                .to_string(),
        }
    }
}

/// `domain_scm::ScmError` → `RestError`
impl From<domain_scm::ScmError> for RestError {
    fn from(e: domain_scm::ScmError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_scm::ScmError::NotFound(_) => ("SCM_NOT_FOUND", "Validation", false),
            domain_scm::ScmError::PermissionDenied(_) => ("POLICY_DENIED", "Policy", false),
            domain_scm::ScmError::InvalidState(_) => ("VALIDATION_FAILED", "Validation", false),
            domain_scm::ScmError::Conflict(_) => ("SCM_CONFLICT", "External", false),
            domain_scm::ScmError::IdempotencyConflict => ("SCM_CONFLICT", "External", false),
            domain_scm::ScmError::ProviderError(_) => ("SCM_PROVIDER_ERROR", "External", true),
            domain_scm::ScmError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("scm: {e}"),
            source_module: "domain-scm".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the SCM resource id + tenant + role (project_admin/developer)".to_string(),
        }
    }
}

/// `domain_validation::ValidationError` → `RestError`
impl From<domain_validation::ValidationError> for RestError {
    fn from(e: domain_validation::ValidationError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_validation::ValidationError::NotFound(_) => {
                ("RESOURCE_NOT_FOUND", "Validation", false)
            }
            domain_validation::ValidationError::PermissionDenied => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_validation::ValidationError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_validation::ValidationError::Conflict(_) => ("VL_CONFLICT", "External", false),
            domain_validation::ValidationError::InvariantViolated(_) => {
                ("VL_INVARIANT_VIOLATED", "Validation", false)
            }
            domain_validation::ValidationError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("validation: {e}"),
            source_module: "domain-validation".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the validation id + tenant + role (developer/service_internal)"
                .to_string(),
        }
    }
}

/// `domain_feedback::FeedbackError` → `RestError` (per v0.58 P0-2 Stage 1, WBS §14.15)
impl From<domain_feedback::FeedbackError> for RestError {
    fn from(e: domain_feedback::FeedbackError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_feedback::FeedbackError::NotFound(_) => ("FB_NOT_FOUND", "Validation", false),
            domain_feedback::FeedbackError::InvalidState(_) => {
                ("FB_INVALID_STATE_TRANSITION", "Validation", false)
            }
            domain_feedback::FeedbackError::TargetUnresolvable(_) => {
                ("FB_TARGET_UNRESOLVABLE", "Validation", false)
            }
            domain_feedback::FeedbackError::ReadOnly => ("FB_READ_ONLY", "Validation", false),
            domain_feedback::FeedbackError::NotDeletable => {
                ("FB_NOT_DELETABLE", "Validation", false)
            }
            domain_feedback::FeedbackError::MissingSuccessor => {
                ("FB_MISSING_SUCCESSOR", "Validation", false)
            }
            domain_feedback::FeedbackError::CrossWorktree => {
                ("FB_CROSS_WORKTREE", "Validation", false)
            }
            domain_feedback::FeedbackError::PermissionDenied => {
                ("FB_PERMISSION_DENIED", "Policy", false)
            }
            domain_feedback::FeedbackError::Conflict(_) => ("FB_CONFLICT", "External", false),
            domain_feedback::FeedbackError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("feedback: {e}"),
            source_module: "domain-feedback".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the feedback id + tenant + role (developer/agent for AI feedback)"
                .to_string(),
        }
    }
}

/// `domain_integration::IntegrationError` → `RestError` (per v0.58 P0-2 Stage 1, WBS §14.15)
impl From<domain_integration::IntegrationError> for RestError {
    fn from(e: domain_integration::IntegrationError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_integration::IntegrationError::NotFound(_) => {
                ("RESOURCE_NOT_FOUND", "Validation", false)
            }
            domain_integration::IntegrationError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_integration::IntegrationError::PermissionDenied => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_integration::IntegrationError::Conflict(_) => {
                ("INTEGRATION_CONFLICT", "External", false)
            }
            domain_integration::IntegrationError::InvalidArgument(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_integration::IntegrationError::LoopGuardMissing(_) => {
                ("I_LOOP_GUARD_MISSING", "Validation", false)
            }
            domain_integration::IntegrationError::CredentialMissing(_) => {
                ("I_CREDENTIAL_MISSING", "Validation", false)
            }
            domain_integration::IntegrationError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("integration: {e}"),
            source_module: "domain-integration".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: "Check the integration id + provider + tenant + role (project_admin/developer)"
                .to_string(),
        }
    }
}

/// `domain_comment::CommentError` → `RestError` (per v0.58 P0-2 Stage 1, WBS §14.15)
impl From<domain_comment::CommentError> for RestError {
    fn from(e: domain_comment::CommentError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_comment::CommentError::NotFound(_) => ("COMMENT_NOT_FOUND", "Validation", false),
            domain_comment::CommentError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_comment::CommentError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_comment::CommentError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_comment::CommentError::InvalidObjectKey => {
                ("CMT_INVALID_OBJECT_KEY", "Validation", false)
            }
            domain_comment::CommentError::ReactionExists => {
                ("CMT_REACTION_EXISTS", "External", false)
            }
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
            hint: "Check the comment id + thread id + tenant + role (developer/agent)".to_string(),
        }
    }
}

impl IntoResponse for RestError {
    fn into_response(self) -> axum::response::Response {
        // per spec §2.4: code → HTTP status 映射
        let status = match self.code.as_str() {
            "NOT_IMPLEMENTED" => StatusCode::NOT_IMPLEMENTED,
            "VALIDATION_FAILED" | "VALIDATION_RUN_FAILED" => StatusCode::BAD_REQUEST,
            "RESOURCE_NOT_FOUND" | "WORKTREE_NOT_FOUND" | "FB_NOT_FOUND" | "COMMENT_NOT_FOUND" => {
                StatusCode::NOT_FOUND
            }
            "POLICY_DENIED" | "FB_PERMISSION_DENIED" => StatusCode::FORBIDDEN,
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
            "INTERNAL" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "error": self }))).into_response()
    }
}

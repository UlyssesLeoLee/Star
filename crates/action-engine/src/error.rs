//! `error.rs` — 6-field `ActionError` (per 守门 #6 v2 + DD-WORKTREE-CANVAS-001 §38 Error Code)
//!
//! 6 字段: code / message / source / location / context / trace_id
//! 错误码 (per DD §38): ACTION_DENIED_BY_RBAC / ACTION_DENIED_BY_MERGE_READINESS /
//! ACTION_DENIED_BY_DEPENDENCY / ACTION_DENIED_BY_LOCK / ACTION_TIMEOUT /
//! ACTION_NOT_AVAILABLE / PERMISSION_DENIED / VALIDATION_FAILED / 等

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field `ActionError` (per 守门 #6 v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionError {
    /// 错误码 (e.g. "ACTION_DENIED_BY_RBAC", "ACTION_TIMEOUT")
    pub code: String,
    /// 人类可读消息
    pub message: String,
    /// 底层错误源描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 现场位置 (file:line)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// KV context (per 守门 #13 d audit context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Trace ID
    pub trace_id: String,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (at {}, trace={})",
            self.code,
            self.message,
            self.location.as_deref().unwrap_or("?"),
            self.trace_id
        )?;
        if let Some(src) = &self.source {
            write!(f, "\n  cause: {src}")?;
        }
        Ok(())
    }
}

impl StdError for ActionError {}

impl ActionError {
    /// 构造一个新 ActionError
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source: None,
            location: None,
            context: None,
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// 设置 source 字段
    #[must_use]
    pub fn with_source(mut self, source: impl fmt::Display) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// 设置 location 字段
    #[must_use]
    pub fn at(mut self, file: &str, line: u32) -> Self {
        self.location = Some(format!("{file}:{line}"));
        self
    }

    /// 设置 context KV
    #[must_use]
    pub fn with_context(mut self, ctx: serde_json::Value) -> Self {
        self.context = Some(ctx);
        self
    }

    /// 设置 trace_id
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = trace_id.into();
        self
    }

    // ============ 工厂方法 (per DD §38) ============

    /// RBAC 拒绝
    pub fn rbac_denied(user: &str, action: &str) -> Self {
        Self::new(
            "ACTION_DENIED_BY_RBAC",
            format!("user {user} is not allowed to perform {action}"),
        )
    }

    /// Merge Readiness 不通过
    pub fn not_ready(reason: &str) -> Self {
        Self::new("ACTION_DENIED_BY_MERGE_READINESS", reason.to_string())
    }

    /// 依赖未满足
    pub fn dependency_not_met(worktree_id: &str) -> Self {
        Self::new(
            "ACTION_DENIED_BY_DEPENDENCY",
            format!("dependency not met: worktree {worktree_id} not yet merged"),
        )
    }

    /// Worktree 已锁定
    pub fn locked(worktree_id: &str) -> Self {
        Self::new(
            "ACTION_DENIED_BY_LOCK",
            format!("worktree {worktree_id} is locked"),
        )
    }

    /// 操作超时
    pub fn timeout(action: &str, ms: u64) -> Self {
        Self::new(
            "ACTION_TIMEOUT",
            format!("action {action} timed out after {ms} ms"),
        )
    }

    /// 二次确认缺失
    pub fn confirm_required(action: &str) -> Self {
        Self::new(
            "ACTION_CONFIRM_REQUIRED",
            format!("action {action} requires user confirmation"),
        )
    }

    /// 操作不可用
    pub fn not_available(action: &str) -> Self {
        Self::new(
            "ACTION_NOT_AVAILABLE",
            format!("action {action} is not available"),
        )
    }

    /// 通用权限拒绝 (per DD §38 PERMISSION_DENIED)
    pub fn permission_denied(user: &str) -> Self {
        Self::new(
            "PERMISSION_DENIED",
            format!("permission denied for user {user}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = ActionError::rbac_denied("alice", "Merge")
            .with_source("inner")
            .at("file.rs", 42)
            .with_context(serde_json::json!({"a": 1}))
            .with_trace_id("trace-1");
        assert_eq!(e.code, "ACTION_DENIED_BY_RBAC");
        assert!(e.message.contains("alice"));
        assert!(e.message.contains("Merge"));
        assert_eq!(e.source.as_deref(), Some("inner"));
        assert_eq!(e.location.as_deref(), Some("file.rs:42"));
        assert_eq!(e.trace_id, "trace-1");
        assert!(e.context.is_some());
    }

    #[test]
    fn factories_set_distinct_codes() {
        let e = ActionError::rbac_denied("u", "A");
        assert_eq!(e.code, "ACTION_DENIED_BY_RBAC");
        let e = ActionError::not_ready("no");
        assert_eq!(e.code, "ACTION_DENIED_BY_MERGE_READINESS");
        let e = ActionError::dependency_not_met("wt-1");
        assert_eq!(e.code, "ACTION_DENIED_BY_DEPENDENCY");
        let e = ActionError::locked("wt-1");
        assert_eq!(e.code, "ACTION_DENIED_BY_LOCK");
        let e = ActionError::timeout("Merge", 100);
        assert_eq!(e.code, "ACTION_TIMEOUT");
        let e = ActionError::confirm_required("Delete");
        assert_eq!(e.code, "ACTION_CONFIRM_REQUIRED");
        let e = ActionError::not_available("X");
        assert_eq!(e.code, "ACTION_NOT_AVAILABLE");
        let e = ActionError::permission_denied("u");
        assert_eq!(e.code, "PERMISSION_DENIED");
    }
}

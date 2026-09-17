//! `error.rs` — `ServiceError` 6-field (per 守门 #6 v2 + DD §12 + §38)
//!
//! 6 字段: code / message / source / location / context / trace_id
//!
//! 错误码列表 (per DD §12 + §38):
//! - WT.NOT_FOUND          : get/update/delete 找不到 Worktree
//! - WT.CREATE_FAIL        : create 失败
//! - WT.INVALID_TRANSITION : 状态机非法迁移 (per INV-WC-01)
//! - WT.LOCKED             : 操作被 lock 阻塞
//! - WT.ARCHIVED           : 已归档, 不可改
//! - WT.GIT_FAIL           : git-adapter 调用失败
//! - WT.GRAPH_FAIL         : graph-core 调用失败
//! - WT.PERMISSION_DENIED  : RLS / 多租户权限拒绝 (per INV-WC-09, NFR-SEC-001)
//! - WT.IO_FAIL            : 持久化层失败

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field ServiceError (per 守门 #6 v2 + DD §12 + §38)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceError {
    /// 错误码 (e.g. "WT.NOT_FOUND")
    pub code: String,
    /// 人类可读消息
    pub message: String,
    /// 底层错误源描述 (保留 error chain 追溯)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 现场位置 (file:line)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// KV context (per 守门 #13 d audit context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Trace ID (关联 SSE / REST 请求)
    pub trace_id: String,
}

impl fmt::Display for ServiceError {
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

impl StdError for ServiceError {}

impl ServiceError {
    /// 构造一个新 ServiceError (无 source/location)
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source: None,
            location: None,
            context: None,
            trace_id: trace_id.into(),
        }
    }

    /// 带 source 链
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 带 location (file:line, 由调用者传入)
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// 带 KV context
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    // ========== 常见快捷构造 (per DD §38) ==========

    /// WT.NOT_FOUND
    pub fn not_found(worktree_id: uuid::Uuid, trace_id: impl Into<String>) -> Self {
        Self::new(
            "WT.NOT_FOUND",
            format!("Worktree {} not found", worktree_id),
            trace_id,
        )
        .with_context(serde_json::json!({ "worktree_id": worktree_id }))
    }

    /// WT.CREATE_FAIL
    pub fn create_fail(reason: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self::new(
            "WT.CREATE_FAIL",
            format!("Create failed: {}", reason.into()),
            trace_id,
        )
    }

    /// WT.INVALID_TRANSITION
    pub fn invalid_transition(
        from: graph_core::state::HumanState,
        event: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        Self::new(
            "WT.INVALID_TRANSITION",
            format!("Invalid transition from {:?} on {}", from, event.into()),
            trace_id,
        )
        .with_context(serde_json::json!({ "from": format!("{:?}", from) }))
    }

    /// WT.LOCKED
    pub fn locked(worktree_id: uuid::Uuid, trace_id: impl Into<String>) -> Self {
        Self::new(
            "WT.LOCKED",
            format!("Worktree {} is locked", worktree_id),
            trace_id,
        )
    }

    /// WT.ARCHIVED
    pub fn archived(worktree_id: uuid::Uuid, trace_id: impl Into<String>) -> Self {
        Self::new(
            "WT.ARCHIVED",
            format!("Worktree {} is archived", worktree_id),
            trace_id,
        )
    }

    /// WT.GIT_FAIL (包装 git-adapter::GitError)
    pub fn git_fail(source: git_adapter::GitError) -> Self {
        Self::new(
            "WT.GIT_FAIL",
            source.message.clone(),
            source.trace_id.clone(),
        )
        .with_source(format!("{:?}", source))
    }

    /// WT.GRAPH_FAIL (包装 graph-core::GraphError)
    pub fn graph_fail(source: graph_core::GraphError) -> Self {
        Self::new("WT.GRAPH_FAIL", format!("{:?}", source), "n/a".to_string())
            .with_source(format!("{:?}", source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_six_fields_complete() {
        let e = ServiceError::not_found(uuid::Uuid::new_v4(), "t1");
        assert_eq!(e.code, "WT.NOT_FOUND");
        assert!(!e.message.is_empty());
        assert!(e.context.is_some());
        assert!(!e.trace_id.is_empty());
    }

    #[test]
    fn error_serialization_roundtrip() {
        let e = ServiceError::new("WT.TEST", "msg", "tr")
            .with_source("cause")
            .with_location("file.rs:42")
            .with_context(serde_json::json!({"k": "v"}));
        let json = serde_json::to_string(&e).unwrap();
        let parsed: ServiceError = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.code, "WT.TEST");
        assert_eq!(parsed.source.as_deref(), Some("cause"));
        assert_eq!(parsed.location.as_deref(), Some("file.rs:42"));
    }
}

//! `error.rs` — `RiskError` 6-field (per 守门 #6 v2 + DD §13 + §38)

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field RiskError (per 守门 #6 v2 + DD §13 + §38)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskError {
    /// 错误码 (e.g. "RISK.NOT_FOUND")
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

impl fmt::Display for RiskError {
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

impl StdError for RiskError {}

impl RiskError {
    /// 构造一个新 RiskError
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

    /// RISK.NOT_FOUND
    pub fn not_found(worktree_id: uuid::Uuid, trace_id: impl Into<String>) -> Self {
        Self::new(
            "RISK.NOT_FOUND",
            format!("No risks found for Worktree {}", worktree_id),
            trace_id,
        )
    }

    /// RISK.GIT_FAIL
    pub fn git_fail(source: git_adapter::GitError) -> Self {
        Self::new(
            "RISK.GIT_FAIL",
            source.message.clone(),
            source.trace_id.clone(),
        )
    }

    /// RISK.GRAPH_FAIL
    pub fn graph_fail(source: graph_core::GraphError) -> Self {
        Self::new(
            "RISK.GRAPH_FAIL",
            format!("{:?}", source),
            "n/a".to_string(),
        )
    }

    /// 加 source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 加 location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// 加 context
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_six_fields_complete() {
        let e = RiskError::new("RISK.TEST", "msg", "tr")
            .with_source("cause")
            .with_location("file.rs:1")
            .with_context(serde_json::json!({"k": "v"}));
        assert_eq!(e.code, "RISK.TEST");
        assert_eq!(e.source.as_deref(), Some("cause"));
        assert_eq!(e.location.as_deref(), Some("file.rs:1"));
    }
}

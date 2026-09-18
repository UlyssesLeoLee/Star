//! `error.rs` — `RelationshipError` 6-field (per 守门 #6 v2 + DD §13 + §38)

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field RelationshipError (per 守门 #6 v2 + DD §13 + §38)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipError {
    /// 错误码 (e.g. "REL.NOT_FOUND")
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

impl fmt::Display for RelationshipError {
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

impl StdError for RelationshipError {}

impl RelationshipError {
    /// 构造新 RelationshipError
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

    /// REL.NOT_FOUND
    pub fn not_found(kind: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self::new(
            "REL.NOT_FOUND",
            format!("{} edge not found", kind.into()),
            trace_id,
        )
    }

    /// REL.INVALID_KIND
    pub fn invalid_kind(kind: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self::new(
            "REL.INVALID_KIND",
            format!("Invalid edge kind {:?}", kind.into()),
            trace_id,
        )
    }

    /// REL.GRAPH_FAIL
    pub fn graph_fail(source: graph_core::GraphError) -> Self {
        Self::new("REL.GRAPH_FAIL", format!("{:?}", source), "n/a".to_string())
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

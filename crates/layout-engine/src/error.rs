//! `error.rs` — 6-field `LayoutError` (per 守门 #6 v2 + DD §38)

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field `LayoutError`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutError {
    /// 错误码
    pub code: String,
    /// 消息
    pub message: String,
    /// 底层错误源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 现场位置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// KV context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Trace ID
    pub trace_id: String,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (trace={})",
            self.code, self.message, self.trace_id
        )
    }
}

impl StdError for LayoutError {}

impl LayoutError {
    /// 构造新错误
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

    /// 设置 source
    #[must_use]
    pub fn with_source(mut self, source: impl fmt::Display) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// 设置 context
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

    /// 设置 location
    #[must_use]
    pub fn at(mut self, file: &str, line: u32) -> Self {
        self.location = Some(format!("{file}:{line}"));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = LayoutError::new("LAYOUT.EMPTY_INPUT", "no nodes")
            .with_source("inner")
            .at("file.rs", 1)
            .with_context(serde_json::json!({"count": 0}))
            .with_trace_id("trace-1");
        assert_eq!(e.code, "LAYOUT.EMPTY_INPUT");
        assert_eq!(e.source.as_deref(), Some("inner"));
    }
}

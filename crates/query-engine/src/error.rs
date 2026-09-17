//! `error.rs` — 6-field `QueryError` (per 守门 #6 v2 + DD §38)
//!
//! 错误码 (per DD §38): DSL_PARSE_ERROR / NL_TRANSLATION_FAILED /
//! CYPHER_INJECTION_DETECTED / CYPHER_SERIALIZE / LLM_TIMEOUT

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field `QueryError`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryError {
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

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (trace={})",
            self.code, self.message, self.trace_id
        )
    }
}

impl StdError for QueryError {}

impl QueryError {
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

    /// DSL parse 失败
    pub fn dsl_parse(input: &str, e: impl fmt::Display) -> Self {
        Self::new("DSL_PARSE_ERROR", format!("failed to parse DSL: {input}")).with_source(e)
    }

    /// NL 翻译失败
    pub fn nl_translation(nl: &str, e: impl fmt::Display) -> Self {
        Self::new(
            "NL_TRANSLATION_FAILED",
            format!("failed to translate NL to DSL: {nl}"),
        )
        .with_source(e)
    }

    /// Cypher 注入检测
    pub fn cypher_injection(detail: impl fmt::Display) -> Self {
        Self::new(
            "CYPHER_INJECTION_DETECTED",
            "Cypher injection attempt detected",
        )
        .with_source(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = QueryError::dsl_parse("bad dsl", "unexpected token")
            .with_context(serde_json::json!({"input": "bad"}))
            .with_trace_id("trace-1")
            .at("file.rs", 10);
        assert_eq!(e.code, "DSL_PARSE_ERROR");
        assert_eq!(e.location.as_deref(), Some("file.rs:10"));
        assert_eq!(e.trace_id, "trace-1");
    }
}

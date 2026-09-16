//! `error.rs` — `GraphError` 6-field (per 守门 #6 v2)
//!
//! 6 字段: code / message / source / location / context / trace_id
//! - code: 错误码 (e.g. "GRAPH.NODE_NOT_FOUND")
//! - message: 人类可读
//! - source: 底层错误源描述 (Display fmt)
//! - location: `file:line` (per `#[track_caller]`)
//! - context: KV context (per 守门 #13 d audit)
//! - trace_id: 关联 SSE/REST 请求的 trace id

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use std::path::Path;

/// 6-field `GraphError` (per 守门 #6 v2)
///
/// 存储描述性字段而非 trait object, 以保证 Send + Sync + 'static
/// + serde-friendly (thiserror 的 `#[source]` 在 serde 下不友好).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphError {
    /// 错误码 (e.g. "GRAPH.NODE_NOT_FOUND", "GRAPH.EDGE_NOT_FOUND", "GRAPH.NEO4J_CONN")
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

impl fmt::Display for GraphError {
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

impl StdError for GraphError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        // Source 字段是 String 描述, 无可转的 trait object; 返回 None
        None
    }
}

impl GraphError {
    /// 构造一个新 GraphError
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

    /// 工厂: NodeNotFound
    pub fn node_not_found(kind: &str, id: &str) -> Self {
        Self::new(
            "GRAPH.NODE_NOT_FOUND",
            format!("node not found: {kind}#{id}"),
        )
    }

    /// 工厂: EdgeNotFound
    pub fn edge_not_found(kind: &str, from: &str, to: &str) -> Self {
        Self::new(
            "GRAPH.EDGE_NOT_FOUND",
            format!("edge not found: {kind}({from} -> {to})"),
        )
    }

    /// 工厂: DuplicateNode
    pub fn duplicate_node(kind: &str, id: &str) -> Self {
        Self::new(
            "GRAPH.DUPLICATE_NODE",
            format!("duplicate node: {kind}#{id}"),
        )
    }

    /// 工厂: Neo4j connection
    pub fn neo4j_conn(uri: &str, e: impl fmt::Display) -> Self {
        Self::new(
            "GRAPH.NEO4J_CONN",
            format!("Neo4j connection failed: {uri}"),
        )
        .with_source(e)
    }

    /// 工厂: 序列化失败
    pub fn serialize_failed(path: &Path, e: impl fmt::Display) -> Self {
        Self::new(
            "GRAPH.SERIALIZE",
            format!("serialize failed: {}", path.display()),
        )
        .with_source(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = GraphError::new("X.Y", "msg")
            .with_source("inner")
            .at("file.rs", 42)
            .with_context(serde_json::json!({"a": 1}))
            .with_trace_id("trace-1");
        assert_eq!(e.code, "X.Y");
        assert_eq!(e.message, "msg");
        assert_eq!(e.source.as_deref(), Some("inner"));
        assert_eq!(e.location.as_deref(), Some("file.rs:42"));
        assert_eq!(e.trace_id, "trace-1");
        assert!(e.context.is_some());
    }

    #[test]
    fn factories_set_codes() {
        let e = GraphError::node_not_found("Repository", "abc");
        assert_eq!(e.code, "GRAPH.NODE_NOT_FOUND");
        let e = GraphError::edge_not_found("BASED_ON", "wt-a", "wt-b");
        assert_eq!(e.code, "GRAPH.EDGE_NOT_FOUND");
        let e = GraphError::duplicate_node("Worktree", "wt-x");
        assert_eq!(e.code, "GRAPH.DUPLICATE_NODE");
        let e = GraphError::neo4j_conn("bolt://x", "boom");
        assert_eq!(e.code, "GRAPH.NEO4J_CONN");
    }

    #[test]
    fn error_display_includes_code_and_trace() {
        let e = GraphError::new("X.Y", "msg");
        let s = format!("{e}");
        assert!(s.contains("X.Y"));
        assert!(s.contains("msg"));
        assert!(s.contains(&e.trace_id));
    }
}

//! `error.rs` — 6-field `EventBusError` (per 守门 #6 v2 + DD §38)
//!
//! 错误码 (per DD §38): EVENT_BUS_LAG / EVENT_SERIALIZE / EVENT_PUBLISH_FAILED /
//! EVENT_CONSUMER_FAILED / EVENT_DEBOUNCE_TIMEOUT

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field `EventBusError`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusError {
    /// 错误码
    pub code: String,
    /// 消息
    pub message: String,
    /// 底层错误源描述
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

impl fmt::Display for EventBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (at {}, trace={})",
            self.code,
            self.message,
            self.location.as_deref().unwrap_or("?"),
            self.trace_id
        )
    }
}

impl StdError for EventBusError {}

impl EventBusError {
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

    /// 设置 location
    #[must_use]
    pub fn at(mut self, file: &str, line: u32) -> Self {
        self.location = Some(format!("{file}:{line}"));
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

    // ============ 工厂方法 ============

    /// Event bus lag (per NFR-REL-002)
    pub fn lag(stream: &str, behind_ms: u64) -> Self {
        Self::new(
            "EVENT_BUS_LAG",
            format!("event bus lag: stream={stream}, behind={behind_ms} ms"),
        )
    }

    /// 发布失败
    pub fn publish_failed(stream: &str, e: impl fmt::Display) -> Self {
        Self::new(
            "EVENT_PUBLISH_FAILED",
            format!("failed to publish to stream {stream}"),
        )
        .with_source(e)
    }

    /// 消费者失败
    pub fn consumer_failed(consumer: &str, e: impl fmt::Display) -> Self {
        Self::new(
            "EVENT_CONSUMER_FAILED",
            format!("consumer {consumer} failed"),
        )
        .with_source(e)
    }

    /// 序列化失败
    pub fn serialize_failed(e: impl fmt::Display) -> Self {
        Self::new("EVENT_SERIALIZE", "event serialize failed").with_source(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = EventBusError::lag("canvas", 200)
            .with_source("redis")
            .at("file.rs", 1)
            .with_context(serde_json::json!({"stream": "canvas"}))
            .with_trace_id("trace-1");
        assert_eq!(e.code, "EVENT_BUS_LAG");
        assert!(e.message.contains("canvas"));
        assert!(e.message.contains("200"));
    }
}

//! `error.rs` — `ObserverError` 6-field (per 守门 #6 v2)
//!
//! 错误码 (per DD §11 + spec §4.5):
//! - OBS.WATCH_INIT_FAIL     : notify::Watcher 创建失败
//! - OBS.WATCH_PATH_INVALID  : 路径无效
//! - OBS.NOTIFY_RECV_FAIL    : notify channel 接收失败
//! - OBS.LIBGIT2_NOTIFY_FAIL : libgit2 notify callback 失败
//! - OBS.DEBOUNCE_FULL       : 防抖队列满 (per NFR-REL-001 反压)
//! - OBS.TRACKER_NOT_FOUND   : tracker 找不到对应 worktree / branch
//! - OBS.SHUTDOWN_TIMEOUT    : graceful shutdown 超时

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field ObserverError (per 守门 #6 v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverError {
    /// 错误码 (e.g. "OBS.WATCH_INIT_FAIL")
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

impl fmt::Display for ObserverError {
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

impl StdError for ObserverError {}

impl ObserverError {
    /// 构造一个新 ObserverError
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

    /// 工厂: WatchInitFail
    pub fn watch_init_fail(e: impl fmt::Display) -> Self {
        Self::new("OBS.WATCH_INIT_FAIL", "failed to init notify watcher").with_source(e)
    }

    /// 工厂: WatchPathInvalid
    pub fn watch_path_invalid(path: &str) -> Self {
        Self::new(
            "OBS.WATCH_PATH_INVALID",
            format!("watch path invalid: {path}"),
        )
    }

    /// 工厂: NotifyRecvFail
    pub fn notify_recv_fail(e: impl fmt::Display) -> Self {
        Self::new("OBS.NOTIFY_RECV_FAIL", "notify channel recv failed").with_source(e)
    }

    /// 工厂: Libgit2NotifyFail
    pub fn libgit2_notify_fail(e: impl fmt::Display) -> Self {
        Self::new("OBS.LIBGIT2_NOTIFY_FAIL", "libgit2 notify callback failed").with_source(e)
    }

    /// 工厂: DebounceFull
    pub fn debounce_full(capacity: usize) -> Self {
        Self::new(
            "OBS.DEBOUNCE_FULL",
            format!("debouncer full (capacity={capacity})"),
        )
    }

    /// 工厂: TrackerNotFound
    pub fn tracker_not_found(what: &str, id: &str) -> Self {
        Self::new(
            "OBS.TRACKER_NOT_FOUND",
            format!("tracker not found: {what}#{id}"),
        )
    }

    /// 工厂: ShutdownTimeout
    pub fn shutdown_timeout(ms: u64) -> Self {
        Self::new(
            "OBS.SHUTDOWN_TIMEOUT",
            format!("graceful shutdown timeout after {ms}ms"),
        )
    }
}

impl From<notify::Error> for ObserverError {
    fn from(e: notify::Error) -> Self {
        ObserverError::watch_init_fail(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = ObserverError::new("X.Y", "msg")
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
        let e = ObserverError::watch_init_fail("boom");
        assert_eq!(e.code, "OBS.WATCH_INIT_FAIL");
        let e = ObserverError::watch_path_invalid("/x");
        assert_eq!(e.code, "OBS.WATCH_PATH_INVALID");
        let e = ObserverError::notify_recv_fail("x");
        assert_eq!(e.code, "OBS.NOTIFY_RECV_FAIL");
        let e = ObserverError::libgit2_notify_fail("x");
        assert_eq!(e.code, "OBS.LIBGIT2_NOTIFY_FAIL");
        let e = ObserverError::debounce_full(1000);
        assert_eq!(e.code, "OBS.DEBOUNCE_FULL");
        let e = ObserverError::tracker_not_found("Worktree", "wt-x");
        assert_eq!(e.code, "OBS.TRACKER_NOT_FOUND");
        let e = ObserverError::shutdown_timeout(5000);
        assert_eq!(e.code, "OBS.SHUTDOWN_TIMEOUT");
    }
}

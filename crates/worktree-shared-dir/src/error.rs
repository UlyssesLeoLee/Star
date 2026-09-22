//! `error.rs` — `SharedDirError` 6-field (per 守门 #6 v2 + DD §38 + ULYS-104.3 FR-ORCA-007)
//!
//! 6 字段: code / message / source / location / context / trace_id
//!
//! 错误码列表 (per ULYS-104.3 §2 + 9/22 D-Boy 拍板 TBD-0044-01 C + 11:36 路径 C 自决):
//! - WSD.NOT_FOUND          : resolver 找不到 SharedDirectory
//! - WSD.SOURCE_DISABLED    : source 标记 enabled=false, 跳过
//! - WSD.SOURCE_CONFLICT    : 同 path 不同 mount_strategy 冲突
//! - WSD.INVALID_PRIORITY   : priority 不在 1-3 范围
//! - WSD.INVALID_STRATEGY   : mount_strategy 字面值无效
//! - WSD.PATH_NOT_ABSOLUTE  : path 不是绝对路径
//! - WSD.PERMISSION_DENIED  : RLS / 多租户权限拒绝 (per 守门 #13 a + INV-WC-09)
//! - WSD.CONFIG_PARSE_FAIL  : ConfigSource JSON 解析失败 (file-backed, 不依赖 CLI)
//! - WSD.CONFIG_IO_FAIL     : ConfigSource FS 读失败 (file not found 不算, 其它 IO 算)
//! - WSD.CONFIG_TYPE_FAIL   : ConfigSource 字段类型错 (期望 array, 实际其它)
//! - WSD.PG_FAIL            : WorkspaceSource 读 PG 表失败 (待 PG impl 实装)

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field SharedDirError (per 守门 #6 v2 + DD §38)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedDirError {
    /// 错误码 (e.g. "WSD.NOT_FOUND")
    pub code: String,
    /// 人类可读消息
    pub message: String,
    /// 底层错误源描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 现场位置 (file:line)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// KV context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Trace ID (关联 SSE / REST 请求)
    pub trace_id: String,
}

impl fmt::Display for SharedDirError {
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
            write!(f, " | source: {src}")?;
        }
        Ok(())
    }
}

impl StdError for SharedDirError {}

impl SharedDirError {
    /// 工厂方法 (per 守门 #6 v2)
    pub fn new(code: &str, message: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            source: None,
            location: None,
            context: None,
            trace_id: trace_id.into(),
        }
    }

    /// 带 source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 带 location (file:line)
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// 带 context (KV)
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

/// 共享 Result 类型
pub type SharedDirResult<T> = Result<T, SharedDirError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_dir_error_6_fields_per_guard_v6() {
        let err = SharedDirError::new("WSD.NOT_FOUND", "test", "trace-1")
            .with_source("underlying")
            .with_location("file:1")
            .with_context(serde_json::json!({"k": "v"}));
        assert_eq!(err.code, "WSD.NOT_FOUND");
        assert_eq!(err.message, "test");
        assert_eq!(err.source.as_deref(), Some("underlying"));
        assert_eq!(err.location.as_deref(), Some("file:1"));
        assert!(err.context.is_some());
        assert_eq!(err.trace_id, "trace-1");
    }

    #[test]
    fn shared_dir_error_display_includes_all() {
        let err = SharedDirError::new("WSD.NOT_FOUND", "msg", "t1").with_source("inner");
        let s = format!("{err}");
        assert!(s.contains("WSD.NOT_FOUND"));
        assert!(s.contains("msg"));
        assert!(s.contains("inner"));
    }
}

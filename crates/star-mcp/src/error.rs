//! `star-mcp` 错误类型
//!
//! Phase D 骨架只暴露 4 类错误(per 任务 brief 隐含:json / io / unknown tool / bad request)。

#![warn(missing_docs)]

use thiserror::Error;

/// MCP server 顶层错误
#[derive(Debug, Error)]
pub(crate) enum McpError {
    /// JSON 解析失败
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO 错误
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// 未知 tool 名
    #[error("unknown tool: {0}")]
    UnknownTool(String),

    /// 请求格式错误(缺字段等)
    #[error("bad request: {0}")]
    BadRequest(String),
}

//! `star-mcp` MCP server (Phase D.3 实装)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md`
//!
//! ## Phase D.3 实装
//!
//! - **完整** JSON-RPC 2.0 transport (per transport.rs)
//! - 3 个 MCP 标准方法: `initialize` / `tools/list` / `tools/call`
//! - 16 tool (per P1-F + submit) 通过 transport dispatch
//! - 5 个错误码: -32700 / -32600 / -32601 / -32602 / -32603
//! - 16 tool inputSchema 完整 (per transport::tools_list)
//! - **不**依赖 rmcp (per 任务 brief 极简骨架约束)
//!
//! ## 16 tools
//!
//! get_issue / search_issues / get_current_task / get_workspace / get_worktree /
//! create_worktree / search_code / get_symbol / find_references / get_code_context /
//! get_context / create_merge_request / request_review / run_validation /
//! get_pipeline_status / submit
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 0 新外部依赖
//! - RUSTFLAGS=-D warnings 必 pass

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod error;
mod tools;
mod transport;

pub(crate) use error::McpError;
pub(crate) use transport::run_session;

/// Phase D.3 MCP server main: stdin → run_session → stdout
///
/// 支持 multi-turn 通信 (EOF 后退出)
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), McpError> {
    eprintln!("star-mcp: Phase D.3 MCP server (16 tools, JSON-RPC 2.0)");
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run_session(stdin.lock(), stdout.lock())
        .await
        .map_err(McpError::Io)?;
    Ok(())
}

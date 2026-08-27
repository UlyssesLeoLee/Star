//! `star-mcp` MCP server (Phase D.5+: stdio + Streamable HTTP + Resources + Prompts)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md`
//!
//! ## Phase D.5+ 实装
//!
//! - **stdio transport** (Phase D.3): `JSON-RPC 2.0` over stdin/stdout
//! - **Streamable HTTP transport** (Phase D.5+): `JSON-RPC 2.0` over `POST /` + SSE response
//! - 7 个 MCP 标准方法: `initialize` / `tools/list` / `tools/call` /
//!   `resources/list` / `resources/read` / `prompts/list` / `prompts/get`
//! - capabilities: `tools` + `resources` + `prompts` (per 2025-06-27 spec)
//! - 16 tool (per P1-F + submit) 通过 transport dispatch
//! - 5 个错误码: -32700 / -32600 / -32601 / -32602 / -32603
//! - **不**依赖 rmcp (per 任务 brief 极简骨架约束)
//!
//! ## CLI
//!
//! ```text
//! star-mcp [--transport stdio|http] [--bind-addr <ADDR>]
//!   --transport stdio    # 默认, 向后兼容 Phase D.3
//!   --transport http     # Phase D.5+ 新增
//!   --bind-addr ADDR     # http 模式监听地址 (默认 127.0.0.1:8080)
//!                        # 可通过 STAR_MCP_BIND_ADDR 环境变量覆盖
//! ```
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
//! - Phase D.5+ 新增 axum + tokio-stream 依赖 (D.3 0 新外部依赖已不适用)
//! - RUSTFLAGS=-D warnings 必 pass

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod error;
mod prompts;
mod resources;
mod tools;
mod transport;
mod transport_http;

pub(crate) use error::McpError;
pub(crate) use transport::run_session;
pub(crate) use transport_http::run_http_server;

/// CLI 参数(per 任务 brief `--transport stdio|http`)
#[derive(Debug, Clone, PartialEq, Eq)]
enum Transport {
    /// Phase D.3 stdio JSON-RPC 2.0 (默认, 向后兼容)
    Stdio,
    /// Phase D.5+ Streamable HTTP (per 2025-06-27 MCP spec §1.2)
    Http,
}

/// Phase D.5+ MCP server main: 解析 CLI → 启动对应 transport
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), McpError> {
    // 解析 CLI args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (transport, bind_addr) = parse_args(&args);

    match transport {
        Transport::Stdio => {
            eprintln!("star-mcp: Phase D.5+ stdio transport (16 tools + Resources + Prompts, JSON-RPC 2.0)");
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            run_session(stdin.lock(), stdout.lock())
                .await
                .map_err(McpError::Io)?;
        }
        Transport::Http => {
            // 优先从环境变量读 (per 任务 brief STAR_MCP_BIND_ADDR), 然后从 CLI
            let addr = std::env::var("STAR_MCP_BIND_ADDR")
                .ok()
                .or(bind_addr)
                .unwrap_or_else(|| transport_http::DEFAULT_BIND_ADDR.to_string());
            run_http_server(&addr).await?;
        }
    }
    Ok(())
}

/// 解析 CLI 参数
///
/// 简单手写 parser(不引入 clap, per 任务 brief 极简约束)
/// 返回 (Transport, Option<bind_addr>)
fn parse_args(args: &[String]) -> (Transport, Option<String>) {
    let mut transport = Transport::Stdio;
    let mut bind_addr: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--transport" => {
                if i + 1 < args.len() {
                    match args[i + 1].as_str() {
                        "stdio" => transport = Transport::Stdio,
                        "http" => transport = Transport::Http,
                        other => {
                            eprintln!("star-mcp: unknown transport '{other}' (expected: stdio|http), falling back to stdio");
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("star-mcp: --transport requires a value (stdio|http)");
                    i += 1;
                }
            }
            "--bind-addr" => {
                if i + 1 < args.len() {
                    bind_addr = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("star-mcp: --bind-addr requires a value (e.g. 127.0.0.1:8080)");
                    i += 1;
                }
            }
            "-h" | "--help" => {
                eprintln!("star-mcp: usage: star-mcp [--transport stdio|http] [--bind-addr <ADDR>]");
                eprintln!("  --transport stdio    # default, Phase D.3 stdio JSON-RPC 2.0");
                eprintln!("  --transport http     # Phase D.5+ Streamable HTTP (POST + SSE)");
                eprintln!("  --bind-addr ADDR     # default 127.0.0.1:8080 (also: STAR_MCP_BIND_ADDR env)");
            }
            other => {
                eprintln!("star-mcp: unknown arg '{other}' (ignored)");
                i += 1;
            }
        }
    }
    (transport, bind_addr)
}

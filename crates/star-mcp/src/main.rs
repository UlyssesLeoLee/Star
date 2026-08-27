//! `star-mcp` MCP server (Phase E: stdio + Streamable HTTP + Resources + Prompts + 6-field error model
//! + Phase H: 22 domain handler 真实数据接入框架)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §1-§5
//! + `spec/agents/02-data-sources-spec.md` §2 (22 domain)
//! + `spec/mcp/02-resources-prompts-spec.md` §2 (Resources 扩展)
//! + `spec/cache/01-cache-contract-spec.md` §4 (TTL 策略)
//!
//! ## Phase E 实装
//!
//! - **stdio transport** (Phase D.3): `JSON-RPC 2.0` over stdin/stdout
//! - **Streamable HTTP transport** (Phase D.5+): `JSON-RPC 2.0` over `POST /` + SSE response
//! - 7 个 MCP 标准方法: `initialize` / `tools/list` / `tools/call` /
//!   `resources/list` / `resources/read` / `prompts/list` / `prompts/get`
//! - capabilities: `tools` + `resources` + `prompts` (per 2025-06-27 spec)
//! - 16 tool (per P1-F + submit) 通过 transport dispatch
//! - 4 个 resource (Phase E per task brief):
//!   - `workspace://current` · `worktree://{id}` · `agent://{id}/state` · `decision://{id}`
//! - 5 个 prompt (Phase E per task brief):
//!   - `submit` · `review` · `context` · `workflow` · `debug`
//! - 6-field 错误模型 (per `agent-api/v1#Error` §3.14, F-06 修复):
//!   `code` / `message` / `source_module` / `source_kind` / `retriable` / `hint`
//! - 24 个 SCREAMING_SNAKE_CASE 错误码 (per `error_code` 模块)
//! - **不**依赖 rmcp (per 任务 brief 极简骨架约束)
//!
//! ## Phase H 实装 (新增)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 (22 domain crate)
//! + `spec/mcp/02-resources-prompts-spec.md` §2 (Resources Phase H 扩展)
//! + `spec/cache/01-cache-contract-spec.md` §4 (TTL 策略: 5s/30s/60s/300s/3600s/86400s):
//!
//! - **22 domain handler** (`crates/star-mcp/src/handlers/*.rs`): 每个 handler 暴露 URI pattern
//!   (e.g. `agent://{id}` / `worktree://{id}`) + cache TTL + mock-but-functional read
//! - `Resource` trait (`crates/star-mcp/src/resources.rs`): typed `Resource<Data = X>`
//! - `DynResource` trait: type-erased (Box<dyn DynResource>) 注册到 `ResourcesHandler::domains`
//! - `KeyBuilder` (`spec/cache/01` §3 L119-126): cache key 格式化
//! - `ResourceError`: handler 内部错误 → 映射到 6-field `McpError`
//! - 全部 mock-but-functional (per AGENTS.md 缺标比错标安全守门 + Phase E mock 标记):
//!   数据标 `TODO: Phase H+ 接 crates/domain-*` 真实数据源
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
//! ## Phase E handler 集成
//!
//! - `ResourcesHandler` (unit struct) — `crates/star-mcp/src/resources.rs`
//!   - 真实数据源依赖 (Phase F) 占位字段已声明, mock 数据标 `_mock: true` + `_todo: ...`
//! - `PromptsHandler` (unit struct) — `crates/star-mcp/src/prompts.rs`
//!   - 5 个模板覆盖 spec/flows 关键路径, 模板内联 mock-but-functional 渲染
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - Phase D.5+ 新增 axum + tokio-stream 依赖 (D.3 0 新外部依赖已不适用)
//! - RUSTFLAGS=-D warnings 必 pass
//! - Phase H: 22 domain 全部 mock-but-functional, 真实数据接入标 `TODO: Phase H+`

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod error;
mod handlers;
mod prompts;
mod resources;
mod tools;
mod transport;
mod transport_http;

pub(crate) use error::McpError;
pub(crate) use prompts::PromptsHandler;
pub(crate) use resources::ResourcesHandler;
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

/// Phase E MCP server main: 解析 CLI → 实例化 handlers → 启动对应 transport
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), McpError> {
    // 解析 CLI args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (transport, bind_addr) = parse_args(&args);

    // Phase H: 22 domain handler 全部实例化 + 注册到 ResourcesHandler
    // (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2
    //   + `spec/mcp/02-resources-prompts-spec.md` §2)
    let resources_handler = ResourcesHandler::with_domains(handlers::all_domain_handlers());
    let prompts_handler = PromptsHandler::new();
    eprintln!(
        "star-mcp: Phase E + H handlers ready (resources: {} = 4 core + 22 domain, prompts: {})",
        resources_handler.list().len(),
        prompts_handler.list().len()
    );

    match transport {
        Transport::Stdio => {
            eprintln!("star-mcp: stdio transport (16 tools + 4 resources + 5 prompts + 22 domain handlers, JSON-RPC 2.0, 6-field error)");
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            run_session(stdin.lock(), stdout.lock())
                .await?;
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

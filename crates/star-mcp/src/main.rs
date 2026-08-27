//! `star-mcp` — STAR MCP server 骨架(Phase D)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md`
//!
//! ## Phase D 范围
//!
//! - 暴露 16 个 tool stub(per 任务 brief 列出)
//! - **不**实现 MCP transport — 留 TODO 给 Phase D.1
//! - 当前 entry 行为:读 stdin 一行 → 打印 `stub: receive {request}` → 退出
//!
//! ## 16 tools 清单
//!
//! 1. `get_issue`
//! 2. `search_issues`
//! 3. `get_current_task`
//! 4. `get_workspace`
//! 5. `get_worktree`
//! 6. `create_worktree`
//! 7. `search_code`
//! 8. `get_symbol`
//! 9. `find_references`
//! 10. `get_code_context`
//! 11. `get_context`
//! 12. `create_merge_request`
//! 13. `request_review`
//! 14. `run_validation`
//! 15. `get_pipeline_status`
//! 16. `submit` (per P1-F)
//!
//! ## 设计原则
//!
//! - 0 unsafe
//! - 0 新依赖(rmcp 在 Phase D.1 评估)
//! - 所有 tool stub 标记 `unimplemented!()` + `// TODO Phase D.1`

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod error;
mod tools;

pub(crate) use error::McpError;

use tools::{
    create_merge_request, create_worktree, find_references, get_code_context, get_context,
    get_current_task, get_issue, get_pipeline_status, get_symbol, get_workspace, get_worktree,
    request_review, run_validation, search_code, search_issues, submit,
};

/// 已知 16 tool 名称(per 任务 brief + `spec/mcp/01-mcp-spec.md`)
const KNOWN_TOOLS: &[&str] = &[
    "get_issue",
    "search_issues",
    "get_current_task",
    "get_workspace",
    "get_worktree",
    "create_worktree",
    "search_code",
    "get_symbol",
    "find_references",
    "get_code_context",
    "get_context",
    "create_merge_request",
    "request_review",
    "run_validation",
    "get_pipeline_status",
    "submit",
];

/// 入口 — Phase D stub 行为
///
/// ## 行为
///
/// 1. 打印 `star-mcp: stub entry, known tools = N` 到 stderr
/// 2. 读 stdin 一行
/// 3. 解析 `{tool, args}` JSON
/// 4. 根据 tool 名称 dispatch 到对应 stub
/// 5. 打印 stub 结果或 `unimplemented!` 错误
///
/// ## 注意
///
/// - 完整 MCP 2026-07-28 transport(stdio JSON-RPC 2.0)待 Phase D.1
/// - Phase D 不会在 spawn 的子进程跑通真实 MCP Inspector(per `spec/.../mcp/01-mcp-spec.md` §7)
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), McpError> {
    eprintln!("star-mcp: stub entry, known tools = {}", KNOWN_TOOLS.len());

    // 读 stdin 一行(非阻塞 — 当前实现假设 agent / Inspector 会送一行)
    let mut line = String::new();
    let n = std::io::stdin().read_line(&mut line).map_err(McpError::Io)?;
    if n == 0 {
        eprintln!("star-mcp: empty stdin, exit");
        return Ok(());
    }
    let request: serde_json::Value = serde_json::from_str(line.trim()).map_err(McpError::Json)?;
    let tool = request
        .get("tool")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| McpError::BadRequest("missing 'tool' field".to_string()))?;
    let args = request
        .get("args")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let result = dispatch(tool, args).await;
    match &result {
        Ok(value) => println!("{}", serde_json::to_string_pretty(value)?),
        Err(e) => eprintln!("error: {e}"),
    }
    result.map(|_| ())
}

/// 内部 dispatch:tool 名 → stub 函数
async fn dispatch(tool: &str, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
    match tool {
        "get_issue" => get_issue::invoke(args).await,
        "search_issues" => search_issues::invoke(args).await,
        "get_current_task" => get_current_task::invoke(args).await,
        "get_workspace" => get_workspace::invoke(args).await,
        "get_worktree" => get_worktree::invoke(args).await,
        "create_worktree" => create_worktree::invoke(args).await,
        "search_code" => search_code::invoke(args).await,
        "get_symbol" => get_symbol::invoke(args).await,
        "find_references" => find_references::invoke(args).await,
        "get_code_context" => get_code_context::invoke(args).await,
        "get_context" => get_context::invoke(args).await,
        "create_merge_request" => create_merge_request::invoke(args).await,
        "request_review" => request_review::invoke(args).await,
        "run_validation" => run_validation::invoke(args).await,
        "get_pipeline_status" => get_pipeline_status::invoke(args).await,
        "submit" => submit::invoke(args).await,
        unknown => Err(McpError::UnknownTool(unknown.to_string())),
    }
}

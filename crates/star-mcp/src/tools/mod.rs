//! 16 MCP tool stub 模块聚合
//!
//! 每个子模块都暴露一个 `pub async fn invoke(args: serde_json::Value) -> Result<serde_json::Value, McpError>`
//! 函数体 `unimplemented!()` + `// TODO Phase D.1` 标记。
//!
//! 完整 schema 引用见 `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2。

#![warn(missing_docs)]

pub(crate) mod create_merge_request;
pub(crate) mod create_worktree;
pub(crate) mod find_references;
pub(crate) mod get_code_context;
pub(crate) mod get_context;
pub(crate) mod get_current_task;
pub(crate) mod get_issue;
pub(crate) mod get_pipeline_status;
pub(crate) mod get_symbol;
pub(crate) mod get_workspace;
pub(crate) mod get_worktree;
pub(crate) mod request_review;
pub(crate) mod run_validation;
pub(crate) mod search_code;
pub(crate) mod search_issues;
pub(crate) mod submit;

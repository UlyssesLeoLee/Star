//! 16 MCP tool stub 模块聚合
//!
//! 每个 tool 都暴露 `pub(crate) async fn invoke(args: serde_json::Value) -> Result<serde_json::Value, McpError>`
//!
//! Phase D 行为:
//! - 真实 schema 返回(per `agent-api/v1`)
//! - 解析 `args` 拿必填字段(缺字段 → McpError::validation)
//! - 返回 mock 数据
//! - 不实现真实业务逻辑(Phase D.1 补齐)

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

/// Phase D mock 统一 schema 版本守门
pub(crate) const SCHEMA_VERSION: &str = "agent-api/v1";

/// 构造 mock 响应的 helper
///
/// 在 mock 数据外层加 `schema_version` + `mock: true` 标记
/// 真实 Phase D.1 实现会移除 `mock` 字段
#[allow(dead_code)] // 部分 mock 当前未触发
pub(crate) fn mock_response(tool: &str, body: serde_json::Value) -> serde_json::Value {
    let mut outer = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "mock": true,
        "tool": tool,
    });
    if let Some(obj) = body.as_object() {
        if let Some(outer_obj) = outer.as_object_mut() {
            for (k, v) in obj {
                outer_obj.insert(k.clone(), v.clone());
            }
        }
    }
    outer
}

/// 从 `args` 拿必填字符串字段
pub(crate) fn require_string(args: &serde_json::Value, field: &str) -> Result<String, String> {
    args.get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("missing required field: {field}"))
}

/// 从 `args` 拿可选字符串字段
#[allow(dead_code)]
pub(crate) fn optional_string(args: &serde_json::Value, field: &str) -> Option<String> {
    args.get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

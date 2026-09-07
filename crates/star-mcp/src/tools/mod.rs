//! 16 MCP tool stub 模块聚合
//!
//! 每个 tool 都暴露 `pub(crate) async fn invoke(args: serde_json::Value) -> Result<serde_json::Value, McpError>`
//!
//! Phase D 行为:
//! - 真实 schema 返回(per `agent-api/v1`)
//! - 解析 `args` 拿必填字段(缺字段 → McpError::validation)
//! - 返回 mock 数据
//! - 不实现真实业务逻辑(Phase D.1 补齐)

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

/// 构造真实响应的 helper (per docs/briefs/tool-p0-impl-001.md §1)
///
/// 在真实数据外层加 `schema_version` (无 `mock: true` 标记)
/// 用于 P0 工具链 (create_merge_request / create_worktree / search_issues)
/// 调真实 domain service 后构造响应
pub(crate) fn real_response(tool: &str, body: serde_json::Value) -> serde_json::Value {
    let mut outer = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
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

/// 检查 actor 是否有非 nil tenant_id
///
/// per 9/7 17:30 JST OPT-WORKER-14: 4 P1 search tool (find_references /
/// get_code_context / get_symbol / search_code) invoke 入口加 nil-actor 检查.
/// nil-tenant actor 表示调用方未提供 tenant 上下文, 应该用
/// `ActorContext::nil_actor_with_tenant(tenant_id)` 显式设置.
///
/// 不通过时返 `ACTOR_SESSION_INVALID` 错误 (跟 domain-service 行为一致, 拒绝 nil tenant).
pub(crate) fn check_actor_tenant(
    actor: &domain_search::ActorContext,
) -> Result<(), crate::error::McpError> {
    use crate::error::error_code;
    use crate::error::ErrorSourceKind;

    if actor.tenant_id.is_nil() {
        return Err(crate::error::McpError::new(
            error_code::ACTOR_SESSION_INVALID,
            "actor session has nil tenant_id; cannot proceed with cross-tenant request"
                .to_string(),
            "actor_session",
            ErrorSourceKind::UserInput,
            false,
            Some(
                "use ActorContext::nil_actor_with_tenant(tenant_id) before invoke to set explicit tenant"
                    .to_string(),
            ),
        ));
    }
    Ok(())
}

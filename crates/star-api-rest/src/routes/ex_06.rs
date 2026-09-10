// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-06 star-mcp 16 tool 幂等改造 (Rust) — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.6):
//! - `crates/star-mcp/src/middleware/idempotency.rs` — Idempotency-Key 解析
//! - 16 tool 加 Idempotency-Key 解析 + 写 idempotency_keys 表
//! - 跟 ADR-0032 MCP Transport stdio 一致
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `POST /api/v1/exclusion/tool-idem` — 16 tool 幂等性检查端点
//!
//! 守门 #5 v2: token / secret / Idempotency-Key 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #13: 写 idempotency_keys (T) 表, audit trigger 必携.

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::response::ResponseMeta;

#[derive(Debug, Deserialize)]
pub struct ToolIdemRequest {
    /// MCP tool 名称 (per 16 tool 列表)
    pub tool_name: String,
    /// 幂等键 (per RFC 8594 draft 或自有规范; e.g. "STAR-1024-attempt-1")
    pub idempotency_key: String,
    /// 5 域: player / economy / match / social / admin
    pub domain: String,
    /// 业务 hash (可选, 双键 dedup 用)
    pub business_hash: Option<String>,
}

/// `POST /api/v1/exclusion/tool-idem`
///
/// v0.53 stub: 返回 501 not_implemented + 16 tool 元数据, 真实 idempotency.rs middleware 跨 session 续.
///
/// 跨 session 续:
/// - 接入 `crates/star-mcp/src/middleware/idempotency.rs` (per 守门 #19 [M] `mcp_idem_wrapper.py`)
/// - 16 tool RACI 5 域分摊 (per §14.9 EX-06 + 5 域 Lead 拍板, 临时代签 per 守门 #3 v2)
/// - 写 idempotency_keys (T) + audit trigger (per 守门 #13 d Transaction 100% audit)
pub async fn check_tool_idempotency(Json(req): Json<ToolIdemRequest>) -> (StatusCode, Json<Value>) {
    let _ = req; // stub 阶段不实际使用
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-06 star-mcp 16 tool 幂等改造跨 session 续; 5 域 Lead 真人到位后接入 crates/star-mcp/src/middleware/idempotency.rs + 16 tool Idempotency-Key 解析 (per 守门 #19 [M] mcp_idem_wrapper.py + 守门 #13 d Transaction 100% audit)",
            "middleware": "crates/star-mcp/src/middleware/idempotency.rs",
            "tools": [
                "search_code", "get_symbol", "find_references", "get_code_context",
                "list_work_items", "get_work_item", "create_work_item", "update_work_item",
                "list_workspaces", "get_workspace", "list_worktrees", "create_worktree",
                "get_merge_request", "create_merge_request", "list_pipelines", "get_pipeline"
            ],
            "idempotency_target_table": "idempotency_keys (T) + audit trigger",
            "raci_split": "16 tool RACI 5 域分摊待 5 域 Lead 拍板",
            "meta": ResponseMeta::stub(),
        })),
    )
}

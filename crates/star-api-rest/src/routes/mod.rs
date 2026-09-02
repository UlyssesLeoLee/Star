// SPDX-License-Identifier: MIT OR Apache-2.0
//! 路由 stub (per spec §2.2 + §2.3)
//!
//! 所有业务端点当前返 `RestError::not_implemented()`,
//! P2 阶段 worker 子代理实装业务逻辑 (派前必先 brief).

pub mod code;
pub mod context;
pub mod merge_requests;
pub mod pipelines;
pub mod reviews;
pub mod submissions;
pub mod validations;
pub mod webhooks;
pub mod work_items;
pub mod workspaces;
pub mod worktrees;

use axum::Json;
use serde_json::{json, Value};

/// 健康检查端点 (per spec §2.4: 标准端点, 不计入 22 业务路由)
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "star-api-rest",
        "version": crate::VERSION,
    }))
}

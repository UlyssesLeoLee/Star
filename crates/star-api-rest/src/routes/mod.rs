// SPDX-License-Identifier: MIT OR Apache-2.0
//! 27 业务路由 (per Phase I 全部完成, per WBS §14.12.1 + 独立 WBS §1.1)
//!
//! 拆分:
//! - `work_items` (5) + `workspaces` (1) + `worktrees` (2) = Batch 1 (8) — 调 InMemory*Service
//! - `code` (4) + `context` (1) + `merge_requests` (1) + `reviews` (1) + `validations` (1) + `submissions` (1) + `pipelines` (1) = Batch 2 (10)
//! - `webhooks` (9) = Batch 3 (端点本地 + 复用 star-webhook::DeliveryStore)
//!
//! 已知简化 (per 独立 WBS §3):
//! - `code::get_symbol` / `code::find_references` 占位 (P0 helper 缺, 真实实现留 Phase II 持久化时)
//! - `submissions::submit` 走 `list_results` 真实 service 路径 (per 独立 WBS §3 #2 上游 MCP `submit.rs` 自身 step 6-12 简化 mock)
//! - `webhooks::list_deliveries` 显示 total 计数 (DeliveryStore 缺 list_all 公开 API, 真实 list 留 Phase II)
//! - `webhooks::replay_delivery` 仅标记新状态, 真实 retry 调度留 Phase II

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

/// 健康检查端点 (per spec §2.4: 标准端点, 不计入 27 业务路由)
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "star-api-rest",
        "version": crate::VERSION,
    }))
}

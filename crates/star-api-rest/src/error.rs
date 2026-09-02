// SPDX-License-Identifier: MIT OR Apache-2.0
//! 6-field 错误模型 (per spec §2.4, 复用 MCP `agent-api/v1#Error`)
//!
//! 字段:
//! - `code` (SCREAMING_SNAKE_CASE 字符串, 24 个 per `star_mcp::error_code`)
//! - `message` (人类可读消息, 不暴露 secret / 内部 stack trace)
//! - `source_module` (e.g. `"domain-work-item"`)
//! - `source_kind` (e.g. `"NotFound"` / `"Validation"` / `"Unauthorized"`)
//! - `retriable` (bool, true → client 可重试)
//! - `hint` (可执行的修复提示)

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// 6-field REST 错误 (per spec §2.4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestError {
    /// 24 个 SCREAMING_SNAKE_CASE 错误码之一 (per `star_mcp::error_code`)
    pub code: String,
    /// 人类可读消息 (不暴露 secret / 内部 stack trace)
    pub message: String,
    /// 触发的 source module (e.g. `"domain-work-item"`)
    pub source_module: String,
    /// 触发的 source kind (e.g. `"NotFound"`)
    pub source_kind: String,
    /// client 是否可重试
    pub retriable: bool,
    /// 可执行的修复提示
    pub hint: String,
}

impl RestError {
    /// 业务端点未实装 (P2 阶段 worker 子代理实装时移除)
    pub fn not_implemented(method: &str, path: &str) -> Self {
        Self {
            code: "NOT_IMPLEMENTED".to_string(),
            message: format!("REST endpoint {method} {path} is not yet implemented (P2 phase, awaiting worker delegation per AGENTS.md §4 #20)"),
            source_module: "star-api-rest".to_string(),
            source_kind: "NotImplemented".to_string(),
            retriable: false,
            hint: "Wait for P2 phase implementation, or check spec at docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md".to_string(),
        }
    }
}

impl IntoResponse for RestError {
    fn into_response(self) -> axum::response::Response {
        // per spec §2.4: 业务端点未实装 → 501
        // P2 阶段按 source_kind 映射真实 HTTP status
        let status = if self.code == "NOT_IMPLEMENTED" {
            StatusCode::NOT_IMPLEMENTED
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(serde_json::json!({ "error": self }))).into_response()
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! 5 路由 stub (per spec §2.2 `get_issue` / `search_issues` / `get_current_task` / work-items CRUD)
//!
//! handler 返回 `RestError` 直接利用 `IntoResponse` 映射 501 Not Implemented
//! (P2 阶段 worker 子代理实装时改为 `Result<RestResponse<T>, RestError>` 模式)

use crate::error::RestError;

/// GET /api/v1/work-items?q=...&status=...&page=... (per `search_issues`)
pub async fn search() -> RestError {
    RestError::not_implemented("GET", "/api/v1/work-items")
}

/// GET /api/v1/work-items/current (per `get_current_task`)
pub async fn current() -> RestError {
    RestError::not_implemented("GET", "/api/v1/work-items/current")
}

/// GET /api/v1/work-items/{id} (per `get_issue`)
pub async fn get_by_id() -> RestError {
    RestError::not_implemented("GET", "/api/v1/work-items/{id}")
}

/// POST /api/v1/work-items (REST 独有, MCP 不暴露)
pub async fn create() -> RestError {
    RestError::not_implemented("POST", "/api/v1/work-items")
}

/// PATCH /api/v1/work-items/{id} (REST 独有, MCP 不暴露)
pub async fn update() -> RestError {
    RestError::not_implemented("PATCH", "/api/v1/work-items/{id}")
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! webhook 管理 9 路由 stub (per spec §2.3)
//!
//! - 5 endpoint CRUD: list / create / get / update / delete
//! - 1 test: send test event
//! - 2 delivery 查询: list / get
//! - 1 replay: replay single delivery

use crate::error::RestError;

// ── endpoint CRUD ───────────────────────────────────────────────────

/// GET /api/v1/webhooks/endpoints
pub async fn list_endpoints() -> RestError {
    RestError::not_implemented("GET", "/api/v1/webhooks/endpoints")
}

/// POST /api/v1/webhooks/endpoints
pub async fn create_endpoint() -> RestError {
    RestError::not_implemented("POST", "/api/v1/webhooks/endpoints")
}

/// GET /api/v1/webhooks/endpoints/{id}
pub async fn get_endpoint() -> RestError {
    RestError::not_implemented("GET", "/api/v1/webhooks/endpoints/{id}")
}

/// PATCH /api/v1/webhooks/endpoints/{id}
pub async fn update_endpoint() -> RestError {
    RestError::not_implemented("PATCH", "/api/v1/webhooks/endpoints/{id}")
}

/// DELETE /api/v1/webhooks/endpoints/{id}
pub async fn delete_endpoint() -> RestError {
    RestError::not_implemented("DELETE", "/api/v1/webhooks/endpoints/{id}")
}

/// POST /api/v1/webhooks/endpoints/{id}/test (发测试事件)
pub async fn test_endpoint() -> RestError {
    RestError::not_implemented("POST", "/api/v1/webhooks/endpoints/{id}/test")
}

// ── delivery 查询 + 重放 ─────────────────────────────────────────────

/// GET /api/v1/webhooks/deliveries (W 类, retention 7d, per spec §5.1)
pub async fn list_deliveries() -> RestError {
    RestError::not_implemented("GET", "/api/v1/webhooks/deliveries")
}

/// GET /api/v1/webhooks/deliveries/{delivery_id}
pub async fn get_delivery() -> RestError {
    RestError::not_implemented("GET", "/api/v1/webhooks/deliveries/{delivery_id}")
}

/// POST /api/v1/webhooks/deliveries/{delivery_id}/replay
pub async fn replay_delivery() -> RestError {
    RestError::not_implemented("POST", "/api/v1/webhooks/deliveries/{delivery_id}/replay")
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! context 路由 stub (per spec §2.2 `get_context`)

use crate::error::RestError;

/// GET /api/v1/context?type=...&id=...
pub async fn get() -> RestError {
    RestError::not_implemented("GET", "/api/v1/context")
}

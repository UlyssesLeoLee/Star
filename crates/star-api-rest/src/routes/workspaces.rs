// SPDX-License-Identifier: MIT OR Apache-2.0
//! workspace 路由 stub (per spec §2.2 `get_workspace`)

use crate::error::RestError;

/// GET /api/v1/workspaces/{id}
pub async fn get_by_id() -> RestError {
    RestError::not_implemented("GET", "/api/v1/workspaces/{id}")
}

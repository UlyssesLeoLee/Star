// SPDX-License-Identifier: MIT OR Apache-2.0
//! pipeline 路由 stub (per spec §2.2 `get_pipeline_status`)

use crate::error::RestError;

/// GET /api/v1/pipelines/{id}
pub async fn get_status() -> RestError {
    RestError::not_implemented("GET", "/api/v1/pipelines/{id}")
}

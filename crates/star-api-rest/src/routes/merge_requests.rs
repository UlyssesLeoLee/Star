// SPDX-License-Identifier: MIT OR Apache-2.0
//! merge_request 路由 stub (per spec §2.2 `create_merge_request`)

use crate::error::RestError;

/// POST /api/v1/merge-requests
pub async fn create() -> RestError {
    RestError::not_implemented("POST", "/api/v1/merge-requests")
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! submission 路由 stub (per spec §2.2 `submit`)

use crate::error::RestError;

/// POST /api/v1/submissions
pub async fn submit() -> RestError {
    RestError::not_implemented("POST", "/api/v1/submissions")
}

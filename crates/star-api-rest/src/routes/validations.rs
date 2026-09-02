// SPDX-License-Identifier: MIT OR Apache-2.0
//! validation 路由 stub (per spec §2.2 `run_validation`)

use crate::error::RestError;

/// POST /api/v1/validations
pub async fn run() -> RestError {
    RestError::not_implemented("POST", "/api/v1/validations")
}

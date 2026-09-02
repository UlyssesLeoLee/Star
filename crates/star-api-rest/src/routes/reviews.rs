// SPDX-License-Identifier: MIT OR Apache-2.0
//! review 路由 stub (per spec §2.2 `request_review`)

use crate::error::RestError;

/// POST /api/v1/reviews
pub async fn request() -> RestError {
    RestError::not_implemented("POST", "/api/v1/reviews")
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! code 路由 stub (per spec §2.2 `search_code` / `get_symbol` / `find_references` / `get_code_context`)

use crate::error::RestError;

/// GET /api/v1/code/search?q=...&path=...&limit=...
pub async fn search() -> RestError {
    RestError::not_implemented("GET", "/api/v1/code/search")
}

/// GET /api/v1/code/symbols/{id}
pub async fn get_symbol() -> RestError {
    RestError::not_implemented("GET", "/api/v1/code/symbols/{id}")
}

/// GET /api/v1/code/symbols/{id}/references
pub async fn find_references() -> RestError {
    RestError::not_implemented("GET", "/api/v1/code/symbols/{id}/references")
}

/// GET /api/v1/code/context?file=...&line=...&window=...
pub async fn get_context() -> RestError {
    RestError::not_implemented("GET", "/api/v1/code/context")
}

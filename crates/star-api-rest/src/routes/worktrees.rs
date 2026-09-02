// SPDX-License-Identifier: MIT OR Apache-2.0
//! worktree 路由 stub (per spec §2.2 `get_worktree` / `create_worktree`)

use crate::error::RestError;

/// POST /api/v1/worktrees
pub async fn create() -> RestError {
    RestError::not_implemented("POST", "/api/v1/worktrees")
}

/// GET /api/v1/worktrees/{id}
pub async fn get_by_id() -> RestError {
    RestError::not_implemented("GET", "/api/v1/worktrees/{id}")
}

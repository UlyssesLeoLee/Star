// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error types for the ARG Data Tier.
//!
//! Per [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §9.1,
//! the ARG Data Tier exposes exactly **10 error variants**:
//!
//! 1. `MemgraphConnection`  — Bolt / network failures
//! 2. `CypherExecution`     — query / write rejected
//! 3. `AgentNotFound`
//! 4. `EdgeNotFound`
//! 5. `ValidationFailed`    — generic validation (empty name, weight OOB, etc.)
//! 6. `PermissionDenied`    — RLS 13 類 / tenant mismatch
//! 7. `TemplateAgentCountMismatch` — template instantiate with wrong N
//! 8. `TrustScoreOutOfRange` — score outside `[0.0, 1.0]`
//! 9. `OfflineQueueFull`    — sled-backed offline queue overflow
//! 10. `Other`              — catch-all

use thiserror::Error;
use uuid::Uuid;

/// Top-level error type for the ARG Data Tier.
#[derive(Debug, Error)]
pub enum ARGError {
    /// Memgraph Bolt connection failure (per DD §9.1 variant 1).
    #[error("Memgraph connection failed: {0}")]
    MemgraphConnection(String),

    /// Cypher query / write rejected by the engine (per DD §9.1 variant 2).
    #[error("Cypher execution failed: {0}")]
    CypherExecution(String),

    /// Agent lookup returned `None` for the given id (per DD §9.1 variant 3).
    #[error("Agent not found: {0}")]
    AgentNotFound(Uuid),

    /// Edge lookup returned `None` for the given id (per DD §9.1 variant 4).
    #[error("Edge not found: {0}")]
    EdgeNotFound(Uuid),

    /// Generic validation failure (empty name, self-loop, weight OOB, …).
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Permission denied (RLS 13 類 or actor mismatch).
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Template instantiate called with the wrong number of agent ids.
    #[error("Template agent count mismatch: expected {expected}, got {actual}")]
    TemplateAgentCountMismatch {
        /// Number of agents required by the template (`min_agents..=max_agents`).
        expected: usize,
        /// Number of agents supplied by the caller.
        actual: usize,
    },

    /// Trust score update would push the score outside `[0.0, 1.0]`.
    #[error("Trust score out of range: {0}")]
    TrustScoreOutOfRange(f32),

    /// Offline queue (sled-backed, in `arg-bridge`) reached its capacity.
    #[error("Offline queue full: {0} items")]
    OfflineQueueFull(usize),

    /// Catch-all for errors that don't fit one of the 9 specific buckets.
    #[error("Other: {0}")]
    Other(String),
}

impl ARGError {
    /// Short, stable error code for cross-crate / API consumers.
    pub fn code(&self) -> &'static str {
        "ARG_ERROR"
    }

    /// Whether the operation is safe to retry.
    ///
    /// Per DD §9.2, connection-level / network errors are retriable while
    /// permission / validation / not-found errors are not.
    pub fn retriable(&self) -> bool {
        matches!(self, Self::MemgraphConnection(_) | Self::CypherExecution(_))
    }
}

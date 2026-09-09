// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error types for the ARG Bridge Tier.
//!
//! Per [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §9.1
//! and [`docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md`](../../docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) §1.1, the
//! ARG Bridge Tier exposes exactly **6 error variants**:
//!
//! 1. `BoltSubscribeFailed`    — Bolt subscription 建立失败
//! 2. `LangGraphReducerFailed` — PyO3 Reducer dispatch 失败
//! 3. `FlushTimeout`           — PeriodFlushWorker 周期超时
//! 4. `OfflinePersistFailed`   — `sled` 写入失败
//! 5. `MemgraphDown`           — Memgraph 不可达, 触发 OfflineQueue
//! 6. `InternalError`          — 兜底 (catch-all)

use thiserror::Error;

/// Top-level error type for the ARG Bridge Tier.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// Bolt subscription failed (e.g. network error, auth rejection).
    #[error("Bolt subscription failed: {0}")]
    BoltSubscribeFailed(String),

    /// LangGraph Reducer dispatch failed (PyO3 call rejection).
    #[error("LangGraph Reducer dispatch failed: {0}")]
    LangGraphReducerFailed(String),

    /// Period flush worker exceeded its allotted window.
    #[error("Period flush timed out after {0:?}")]
    FlushTimeout(std::time::Duration),

    /// Offline persistence (sled) failed.
    #[error("Offline persistence failed: {0}")]
    OfflinePersistFailed(String),

    /// Memgraph connection dropped; bridge must fall back to the offline queue.
    #[error("Memgraph is down: {0}")]
    MemgraphDown(String),

    /// Catch-all for errors that don't fit one of the 5 specific buckets.
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl BridgeError {
    /// Short, stable error code for cross-crate / API consumers.
    pub fn code(&self) -> &'static str {
        "BRIDGE_ERROR"
    }

    /// Whether the operation is safe to retry.
    ///
    /// Per DD §9.2, network / Bolt subscription / Memgraph-down errors are
    /// retriable while internal / Reducer-failed errors are not.
    pub fn retriable(&self) -> bool {
        matches!(
            self,
            Self::BoltSubscribeFailed(_) | Self::MemgraphDown(_) | Self::FlushTimeout(_)
        )
    }
}

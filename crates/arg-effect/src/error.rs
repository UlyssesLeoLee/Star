// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error types for the ARG Effect Tier.
//!
//! Per [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §9.1
//! + [`docs/architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md`](../../docs/architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md) §1.1,
//! the ARG Effect Tier exposes exactly **8 error variants**:
//!
//! 1. `DispatchNoRoute`         — no outgoing / incoming / collaborating edges
//! 2. `ContextTooLarge`         — injected prompt exceeds the per-call byte cap
//! 3. `TrustScoreOutOfRange`    — score outside `[0.0, 1.0]`
//! 4. `OutputQualityLow`        — peer_review / challenge returned reject
//! 5. `AchievementNotUnlocked`  — achievement evaluator asked about a non-existent code
//! 6. `LLMServiceDown`          — LLMClient call failed (per 守门 #5 v2 + #23 mock fallback)
//! 7. `PyO3BridgeFailed`        — L0↔L1 PyO3 binding failure (per 缺口 G-3)
//! 8. `InternalError`           — catch-all

use thiserror::Error;

/// Top-level error type for the ARG Effect Tier.
#[derive(Debug, Error)]
pub enum EffectError {
    /// No route could be derived (no outgoing delegates_to / incoming stand_in_for / collaborating edges).
    #[error("DispatchNoRoute: no outgoing / incoming / collaborating edges for agent {0}")]
    DispatchNoRoute(uuid::Uuid),

    /// Injected prompt exceeds the per-call byte cap (default 32 KiB).
    #[error("ContextTooLarge: injected prompt is {size} bytes, cap is {cap}")]
    ContextTooLarge {
        /// Injected prompt size in bytes.
        size: usize,
        /// Configured cap in bytes.
        cap: usize,
    },

    /// Trust score update would push the score outside `[0.0, 1.0]`.
    #[error("TrustScoreOutOfRange: {0}")]
    TrustScoreOutOfRange(f32),

    /// Output rejected by challenge or peer review.
    #[error("OutputQualityLow: {0}")]
    OutputQualityLow(String),

    /// Achievement unlock asked for an unknown code.
    #[error("AchievementNotUnlocked: code={0}")]
    AchievementNotUnlocked(String),

    /// LLMClient call failed.
    #[error("LLMServiceDown: {0}")]
    LLMServiceDown(String),

    /// L0↔L1 PyO3 binding failure.
    #[error("PyO3BridgeFailed: {0}")]
    PyO3BridgeFailed(String),

    /// Catch-all for errors that don't fit one of the 7 specific buckets.
    #[error("InternalError: {0}")]
    InternalError(String),
}

impl EffectError {
    /// Short, stable error code for cross-crate / API consumers.
    pub fn code(&self) -> &'static str {
        "EFFECT_ERROR"
    }

    /// Whether the operation is safe to retry.
    ///
    /// Per DD §9.2, infrastructure errors (LLM / PyO3) are retriable
    /// while domain / validation errors are not.
    pub fn retriable(&self) -> bool {
        matches!(
            self,
            Self::LLMServiceDown(_) | Self::PyO3BridgeFailed(_) | Self::DispatchNoRoute(_)
        )
    }
}

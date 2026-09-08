// SPDX-License-Identifier: MIT OR Apache-2.0
//! Trust score 5 档 (per DD-AGENT-RELATIONSHIP-001 §3.3.3).
//!
//! Buckets:
//! - `Untrusted`  — `[0.0, 0.2)`
//! - `Low`        — `[0.2, 0.4)`
//! - `Medium`     — `[0.4, 0.7)`
//! - `High`       — `[0.7, 0.9)`
//! - `VeryHigh`   — `[0.9, 1.0]`
//!
//! The free function [`update_trust_score`] applies the canonical
//! `+0.01` (success) / `-0.05` (failure) nudge and clamps to `[0.0, 1.0]`.

/// 5 档 enum (per DD §3.3.3).
///
/// `Untrusted` is the lowest bucket; `VeryHigh` is the highest. Used by
/// `ARGTrustEngine::should_skip_verify` (per BD §4.3.3) to gate
/// verifier-skipping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustScoreTier {
    /// `[0.0, 0.2)`.
    Untrusted,
    /// `[0.2, 0.4)`.
    Low,
    /// `[0.4, 0.7)`.
    Medium,
    /// `[0.7, 0.9)`.
    High,
    /// `[0.9, 1.0]`.
    VeryHigh,
}

impl TrustScoreTier {
    /// Bucket a score (in `[0.0, 1.0]`) into a 5-档 enum.
    pub fn from_score(score: f32) -> Self {
        if score < 0.2 {
            Self::Untrusted
        } else if score < 0.4 {
            Self::Low
        } else if score < 0.7 {
            Self::Medium
        } else if score < 0.9 {
            Self::High
        } else {
            Self::VeryHigh
        }
    }

    /// Inclusive `(min, max)` range covered by this tier.
    pub fn to_score_range(&self) -> (f32, f32) {
        match self {
            Self::Untrusted => (0.0, 0.2),
            Self::Low => (0.2, 0.4),
            Self::Medium => (0.4, 0.7),
            Self::High => (0.7, 0.9),
            Self::VeryHigh => (0.9, 1.0),
        }
    }
}

/// Apply the canonical `+0.01` / `-0.05` nudge, clamped to `[0.0, 1.0]`.
///
/// `success = true` → trust grows by `+0.01`.
/// `success = false` → trust shrinks by `-0.05`.
pub fn update_trust_score(current: f32, success: bool) -> f32 {
    if success {
        (current + 0.01).min(1.0)
    } else {
        (current - 0.05).max(0.0)
    }
}

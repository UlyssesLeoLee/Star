// SPDX-License-Identifier: MIT OR Apache-2.0
//! 5 产出质量 aggregate metrics (per DD-AGENT-RELATIONSHIP-001 §3.1 + §6).
//!
//! Placeholders for P3-E ARG.8. Each entry is the `(code, metric)`
//! pair consumed by the `OutputEvaluator`.

/// A `(code, metric_name)` pair for one 产出 achievement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputMetric {
    /// Achievement code.
    pub code: String,
    /// Metric name (e.g. `"peer_review_score"`).
    pub metric: String,
}

/// Returns the 5 产出 metrics in canonical order.
pub fn all_output_metrics() -> Vec<OutputMetric> {
    vec![
        OutputMetric {
            code: "OUT-001-PEER-REVIEW-80".into(),
            metric: "peer_review_score".into(),
        },
        OutputMetric {
            code: "OUT-002-CHALLENGE-ACCEPT".into(),
            metric: "challenge_accept_streak".into(),
        },
        OutputMetric {
            code: "OUT-003-FAST-COLLAB".into(),
            metric: "wall_clock_savings_seconds".into(),
        },
        OutputMetric {
            code: "OUT-004-TRUST-VERIFIED".into(),
            metric: "trust_skip_verify_total".into(),
        },
        OutputMetric {
            code: "OUT-005-LEGENDARY-AGENT".into(),
            metric: "agents_at_very_high_tier".into(),
        },
    ]
}

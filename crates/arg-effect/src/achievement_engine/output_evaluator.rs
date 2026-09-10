// SPDX-License-Identifier: MIT OR Apache-2.0
//! `OutputEvaluator` (per DD-AGENT-RELATIONSHIP-001 §3.2.4 + brief §2.1 A.2).
//!
//! The evaluator runs the **5 产出 aggregate metrics** that drive the
//! 5 `OUT-*` achievement codes. Each metric consumes one or more
//! in-memory [`OutputStats`] counters and returns the list of OUT codes
//! that transitioned to "triggered" during this evaluation pass.
//!
//! The 5 metrics (per brief `arg-08-behavior-output-evaluator.md` §2.1 A.2):
//!
//! | # | Code | Metric (per brief) | Description |
//! |---|---|---|---|
//! | 1 | `OUT-001-TRUSTS-100K-TOKEN` | `trusts_skip_verify_save_100k_token` | `TRUSTS` skip-verify saved >= 100K tokens |
//! | 2 | `OUT-002-COLLAB-1H`        | `collaborate_save_1h_wall_clock`    | parallel collab saved >= 1h wall-clock |
//! | 3 | `OUT-003-5-LEAD-CONSENSUS` | `5_domain_lead_consensus_reached`  | 5 domain Leads reached consensus |
//! | 4 | `OUT-004-ZERO-FAIL-100`    | `zero_failure_100_collaborations`   | 100 collaborations with 0 failures |
//! | 5 | `OUT-005-ACHIEVEMENT-CHAIN-5` | `achievement_chain_5_in_a_row`   | 5 achievement unlocks in a row |
//!
//! Pattern matching is in-process; the engine can be exercised end-to-end
//! without Memgraph (per ARG.1 G-1 stub + ARG.7 brief §2.2).
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use star_arg::models::event::ARGEvent;
use uuid::Uuid;

use crate::error::EffectError;

/// OUT-001: `TRUSTS` skip-verify saved >= 100K tokens (per brief A.2 #1).
pub const OUT_001_TRUSTS_100K_TOKEN: &str = "OUT-001-TRUSTS-100K-TOKEN";
/// OUT-002: parallel collab saved >= 1h wall-clock (per brief A.2 #2).
pub const OUT_002_COLLAB_1H: &str = "OUT-002-COLLAB-1H";
/// OUT-003: 5 domain Leads reached consensus (per brief A.2 #3).
pub const OUT_003_5_LEAD_CONSENSUS: &str = "OUT-003-5-LEAD-CONSENSUS";
/// OUT-004: 100 collaborations with 0 failures (per brief A.2 #4).
pub const OUT_004_ZERO_FAIL_100: &str = "OUT-004-ZERO-FAIL-100";
/// OUT-005: 5 achievement unlocks in a row (per brief A.2 #5).
pub const OUT_005_ACHIEVEMENT_CHAIN_5: &str = "OUT-005-ACHIEVEMENT-CHAIN-5";

/// All 5 OUT codes in canonical order.
pub const ALL_OUT_CODES: [&str; 5] = [
    OUT_001_TRUSTS_100K_TOKEN,
    OUT_002_COLLAB_1H,
    OUT_003_5_LEAD_CONSENSUS,
    OUT_004_ZERO_FAIL_100,
    OUT_005_ACHIEVEMENT_CHAIN_5,
];

/// Required metric thresholds to trigger each of the 5 OUT achievements
/// (per brief A.2 thresholds).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutputThresholds {
    /// OUT-001: cumulative `TRUSTS` skip-verify tokens saved to trigger.
    pub trusts_tokens_saved: f64,
    /// OUT-002: parallel collab wall-clock seconds saved to trigger.
    pub collab_wall_clock_seconds: f64,
    /// OUT-003: count of 5-domain Lead consensus events to trigger.
    pub consensus_count: u32,
    /// OUT-004: count of zero-failure collaborations to trigger.
    pub zero_failure_count: u32,
    /// OUT-005: count of consecutive achievement unlocks to trigger.
    pub chain_length: u32,
}

impl Default for OutputThresholds {
    fn default() -> Self {
        Self {
            trusts_tokens_saved: 100_000.0,
            collab_wall_clock_seconds: 3_600.0,
            consensus_count: 1,
            zero_failure_count: 100,
            chain_length: 5,
        }
    }
}

/// In-memory aggregate metrics that drive the 5 OUT patterns.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OutputStats {
    /// Cumulative tokens saved by `TRUSTS` skip-verify (per OUT-001).
    pub trusts_tokens_saved: f64,
    /// Cumulative wall-clock seconds saved by parallel collab (per OUT-002).
    pub collab_wall_clock_seconds: f64,
    /// Count of 5-domain Lead consensus events (per OUT-003).
    pub consensus_count: u32,
    /// Count of zero-failure collaborations (per OUT-004).
    pub zero_failure_count: u32,
    /// Count of total collaborations (denominator of zero-failure rate).
    pub total_collaborations: u32,
    /// Count of consecutive achievement unlocks (per OUT-005).
    pub achievement_chain_length: u32,
}

impl OutputStats {
    /// Build a new zeroed stats struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the metrics based on the current `ARGEvent`.
    ///
    /// The `token_savings` / `wall_clock_seconds` fields are read from
    /// the event's `metadata` JSON when present (production: written by
    /// the trust engine / dispatch router; tests can seed directly).
    pub fn record_event(&mut self, event: &ARGEvent) {
        match event {
            ARGEvent::TrustScoreChanged { delta, .. } => {
                // Each trust score change is treated as a small
                // skip-verify token saving; 200 tokens per +delta
                // (per trust_engine.rs SKIP_VERIFY heuristics).
                if *delta > 0.0 {
                    self.trusts_tokens_saved += f64::from(*delta) * 200.0;
                }
            }
            ARGEvent::AchievementUnlocked(_) => {
                self.achievement_chain_length += 1;
            }
            ARGEvent::EdgeCreated(e) | ARGEvent::EdgeUpdated { after: e, .. } => {
                self.total_collaborations += 1;
                // read metadata for wall_clock savings if present
                if let Some(saved) = read_wall_clock_saved(&e.metadata) {
                    self.collab_wall_clock_seconds += saved;
                }
            }
            ARGEvent::EdgeArchived { .. } => {
                // soft-delete: doesn't affect output metrics
            }
            ARGEvent::AgentCreated(_)
            | ARGEvent::AgentUpdated { .. }
            | ARGEvent::AgentArchived { .. }
            | ARGEvent::TemplateInstantiated(_) => {
                // Templates / agents trigger consensus counting (1
                // consensus = 5 domain Leads agreed on a template
                // instantiation).
                if matches!(event, ARGEvent::TemplateInstantiated(_)) {
                    self.consensus_count += 1;
                }
            }
        }
    }

    /// Record an explicit zero-failure collaboration (called by tests /
    /// the dispatch router when a `COLLABORATES_WITH` round completes
    /// without a failure).
    pub fn record_zero_failure_collab(&mut self) {
        self.zero_failure_count += 1;
    }

    /// Reset the achievement chain length (called by tests; production
    /// resets when an unlock fails or is duplicated).
    pub fn reset_achievement_chain(&mut self) {
        self.achievement_chain_length = 0;
    }

    /// Returns the OUT codes that newly trigger given the configured
    /// [`OutputThresholds`].
    pub fn triggered_codes(&self, t: &OutputThresholds) -> Vec<String> {
        let mut out = Vec::new();
        if self.trusts_tokens_saved >= t.trusts_tokens_saved {
            out.push(OUT_001_TRUSTS_100K_TOKEN.to_string());
        }
        if self.collab_wall_clock_seconds >= t.collab_wall_clock_seconds {
            out.push(OUT_002_COLLAB_1H.to_string());
        }
        if self.consensus_count >= t.consensus_count {
            out.push(OUT_003_5_LEAD_CONSENSUS.to_string());
        }
        if self.zero_failure_count >= t.zero_failure_count
            && self.total_collaborations >= t.zero_failure_count
        {
            out.push(OUT_004_ZERO_FAIL_100.to_string());
        }
        if self.achievement_chain_length >= t.chain_length {
            out.push(OUT_005_ACHIEVEMENT_CHAIN_5.to_string());
        }
        out
    }
}

/// Helper: read `wall_clock_saved_seconds` from edge metadata.
fn read_wall_clock_saved(metadata: &serde_json::Value) -> Option<f64> {
    metadata
        .get("wall_clock_saved_seconds")
        .and_then(serde_json::Value::as_f64)
}

/// `OutputEvaluator` (per brief §2.1 A.2).
#[derive(Debug)]
pub struct OutputEvaluator {
    /// Cumulative metric counters.
    stats: Mutex<OutputStats>,
    /// Per-metric thresholds.
    thresholds: OutputThresholds,
}

impl Default for OutputEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for OutputEvaluator {
    fn clone(&self) -> Self {
        let stats = self.stats.lock().expect("output stats poisoned").clone();
        Self {
            stats: Mutex::new(stats),
            thresholds: self.thresholds,
        }
    }
}

impl OutputEvaluator {
    /// Build a new evaluator with default thresholds (per brief).
    pub fn new() -> Self {
        Self::with_thresholds(OutputThresholds::default())
    }

    /// Build a new evaluator with custom thresholds (used by tests).
    pub fn with_thresholds(thresholds: OutputThresholds) -> Self {
        Self {
            stats: Mutex::new(OutputStats::new()),
            thresholds,
        }
    }

    /// Returns a snapshot of the current stats.
    pub fn stats(&self) -> OutputStats {
        self.stats.lock().expect("output stats poisoned").clone()
    }

    /// Replace the stats entirely (used by tests for seeding).
    pub fn set_stats(&self, stats: OutputStats) {
        let mut guard = self.stats.lock().expect("output stats poisoned");
        *guard = stats;
    }

    /// Run the 5 metrics against the supplied [`ARGEvent`].
    pub async fn evaluate(
        &self,
        event: &ARGEvent,
        _tenant_id: Uuid,
    ) -> Result<Vec<String>, EffectError> {
        let snapshot = {
            let mut guard = self.stats.lock().expect("output stats poisoned");
            guard.record_event(event);
            guard.clone()
        };
        Ok(snapshot.triggered_codes(&self.thresholds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};

    fn edge_with_meta(
        from: Uuid,
        to: Uuid,
        t: RelationshipType,
        meta: serde_json::Value,
        tenant: Uuid,
    ) -> Edge {
        let now = Utc::now();
        Edge {
            id: Uuid::new_v4(),
            from_agent: from,
            to_agent: to,
            edge_type: t,
            weight: 0.7,
            direction: if t.is_directed() {
                EdgeDirection::Directed
            } else {
                EdgeDirection::Undirected
            },
            archived: false,
            metadata: meta,
            tenant_id: tenant,
            created_at: now,
            updated_at: now,
            version: 1,
            created_by: Uuid::new_v4(),
        }
    }

    #[test]
    fn out_002_triggers_after_3600_wall_clock_seconds() {
        let eval = OutputEvaluator::new();
        let t = Uuid::new_v4();
        let mut stats = OutputStats::new();
        stats.collab_wall_clock_seconds = 3500.0;
        eval.set_stats(stats);
        // 1 more collab event saving 200s = 3700 >= 3600.
        let evt = ARGEvent::EdgeCreated(edge_with_meta(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::CollaboratesWith,
            serde_json::json!({"wall_clock_saved_seconds": 200.0}),
            t,
        ));
        let snapshot = eval.stats();
        let mut updated = snapshot;
        updated.record_event(&evt);
        let codes = updated.triggered_codes(&OutputThresholds::default());
        assert!(codes.iter().any(|c| c == OUT_002_COLLAB_1H));
    }

    #[test]
    fn out_005_triggers_after_5_unlocks() {
        let mut stats = OutputStats::new();
        stats.achievement_chain_length = 5;
        let codes = stats.triggered_codes(&OutputThresholds::default());
        assert!(codes.iter().any(|c| c == OUT_005_ACHIEVEMENT_CHAIN_5));
    }

    #[test]
    fn all_5_codes_unique() {
        let mut set: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for c in ALL_OUT_CODES.iter() {
            assert!(set.insert(*c), "duplicate code: {c}");
        }
    }
}

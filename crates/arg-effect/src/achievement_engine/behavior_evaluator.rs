// SPDX-License-Identifier: MIT OR Apache-2.0
//! `BehaviorEvaluator` (per DD-AGENT-RELATIONSHIP-001 §3.2.4 + brief §2.1 A.1).
//!
//! The evaluator runs the **7 协作行为 event patterns** that drive the
//! 7 `BEH-*` achievement codes. Each pattern consumes one or more
//! in-memory [`BehaviorStats`] counters and returns the list of BEH codes
//! that transitioned to "triggered" during this evaluation pass.
//!
//! The 7 patterns (per brief `arg-08-behavior-output-evaluator.md` §2.1 A.1):
//!
//! | # | Code | Pattern (per brief) | Description |
//! |---|---|---|---|
//! | 1 | `BEH-001-FIRST-DELEGATES` | `first_dispatch_via_delegates_to` | First `DELEGATES_TO` edge created |
//! | 2 | `BEH-002-CONSULTS-100`     | `consults_decision_made_100_times` | 100 `CONSULTS` decisions made |
//! | 3 | `BEH-003-COLLAB-50`        | `collaborates_with_parallel_50_times` | 50 `COLLABORATES_WITH` parallel edges |
//! | 4 | `BEH-004-STAND-IN-5`       | `stand_in_for_takeover_5_times` | 5 `STAND_IN_FOR` takeover events |
//! | 5 | `BEH-005-PEER-100PCT-10`   | `peer_reviews_one_pass_100_percent` | 10 `PEER_REVIEWS` rounds, all accepted |
//! | 6 | `BEH-006-CHALLENGE-3`      | `challenges_rebuttal_3_times` | 3 `CHALLENGES` rebuttal rounds |
//! | 7 | `BEH-007-SHADOWS-24H`      | `shadows_observation_24_hours` | 24h of `SHADOWS` observation |
//!
//! Pattern matching is in-process; the engine can be exercised end-to-end
//! without Memgraph (per ARG.1 G-1 stub + ARG.7 brief §2.2).
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use star_arg::models::edge::RelationshipType;
use star_arg::models::event::ARGEvent;
use uuid::Uuid;

use crate::error::EffectError;

/// BEH-001: first `DELEGATES_TO` edge created (per brief A.1 #1).
pub const BEH_001_FIRST_DELEGATES: &str = "BEH-001-FIRST-DELEGATES";
/// BEH-002: 100 `CONSULTS` decision rounds (per brief A.1 #2).
pub const BEH_002_CONSULTS_100: &str = "BEH-002-CONSULTS-100";
/// BEH-003: 50 `COLLABORATES_WITH` parallel rounds (per brief A.1 #3).
pub const BEH_003_COLLAB_50: &str = "BEH-003-COLLAB-50";
/// BEH-004: 5 `STAND_IN_FOR` takeover rounds (per brief A.1 #4).
pub const BEH_004_STAND_IN_5: &str = "BEH-004-STAND-IN-5";
/// BEH-005: 10 `PEER_REVIEWS` rounds, 100% accepted (per brief A.1 #5).
pub const BEH_005_PEER_100PCT_10: &str = "BEH-005-PEER-100PCT-10";
/// BEH-006: 3 `CHALLENGES` rebuttal rounds (per brief A.1 #6).
pub const BEH_006_CHALLENGE_3: &str = "BEH-006-CHALLENGE-3";
/// BEH-007: 24h of `SHADOWS` observation (per brief A.1 #7).
pub const BEH_007_SHADOWS_24H: &str = "BEH-007-SHADOWS-24H";

/// All 7 BEH codes in canonical order.
pub const ALL_BEH_CODES: [&str; 7] = [
    BEH_001_FIRST_DELEGATES,
    BEH_002_CONSULTS_100,
    BEH_003_COLLAB_50,
    BEH_004_STAND_IN_5,
    BEH_005_PEER_100PCT_10,
    BEH_006_CHALLENGE_3,
    BEH_007_SHADOWS_24H,
];

/// Required event counts to trigger each of the 7 BEH achievements
/// (per brief A.1 thresholds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviorThresholds {
    /// BEH-001: count of `DELEGATES_TO` edges to trigger.
    pub first_dispatch_count: u32,
    /// BEH-002: count of `CONSULTS` decisions to trigger.
    pub consults_100_count: u32,
    /// BEH-003: count of `COLLABORATES_WITH` parallel edges to trigger.
    pub collaborates_50_count: u32,
    /// BEH-004: count of `STAND_IN_FOR` takeover events to trigger.
    pub stand_in_5_count: u32,
    /// BEH-005: count of `PEER_REVIEWS` rounds with 100% accept rate to trigger.
    pub peer_reviews_10_count: u32,
    /// BEH-006: count of `CHALLENGES` rebuttal rounds to trigger.
    pub challenges_3_count: u32,
    /// BEH-007: count of `SHADOWS` hours (each event = 1h) to trigger.
    pub shadows_24h_count: u32,
}

impl Default for BehaviorThresholds {
    fn default() -> Self {
        Self {
            first_dispatch_count: 1,
            consults_100_count: 100,
            collaborates_50_count: 50,
            stand_in_5_count: 5,
            peer_reviews_10_count: 10,
            challenges_3_count: 3,
            shadows_24h_count: 24,
        }
    }
}

/// In-memory counters that drive the 7 behavior patterns.
///
/// The struct is plain data; the evaluator takes `&mut BehaviorStats`
/// via a [`std::sync::Mutex`] so the engine stays `Sync`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BehaviorStats {
    /// Total `DELEGATES_TO` edges observed (per tenant).
    pub delegates_total: u64,
    /// Total `CONSULTS` decisions observed.
    pub consults_total: u64,
    /// Total `COLLABORATES_WITH` parallel edges observed.
    pub collaborates_total: u64,
    /// Total `STAND_IN_FOR` takeover events observed.
    pub stand_in_total: u64,
    /// Total `PEER_REVIEWS` rounds observed.
    pub peer_reviews_total: u64,
    /// Total `PEER_REVIEWS` rounds accepted.
    pub peer_reviews_accepted: u64,
    /// Total `CHALLENGES` rebuttal rounds observed.
    pub challenges_total: u64,
    /// Total `SHADOWS` observation events observed (each = 1h).
    pub shadows_total: u64,
}

impl BehaviorStats {
    /// Build a new zeroed stats struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the counters based on the current `ARGEvent`.
    pub fn record_event(&mut self, event: &ARGEvent) {
        match event {
            ARGEvent::EdgeCreated(e) if !e.archived => self.record_edge(e.edge_type),
            ARGEvent::EdgeUpdated { after, .. } if !after.archived => {
                self.record_edge(after.edge_type)
            }
            ARGEvent::EdgeArchived { .. } => {
                // Soft-delete does not count toward cumulative patterns.
            }
            _ => {}
        }
    }

    /// Update the counter for a single edge relationship type.
    ///
    /// `PeerReviews` increments both `peer_reviews_total` and
    /// `peer_reviews_accepted` (the behavior layer assumes the
    /// first-pass accept — the output layer tracks the actual
    /// accept / reject distribution separately).
    fn record_edge(&mut self, t: RelationshipType) {
        match t {
            RelationshipType::DelegatesTo => self.delegates_total += 1,
            RelationshipType::Consults => self.consults_total += 1,
            RelationshipType::CollaboratesWith => self.collaborates_total += 1,
            RelationshipType::StandInFor => self.stand_in_total += 1,
            RelationshipType::PeerReviews => {
                self.peer_reviews_total += 1;
                self.peer_reviews_accepted += 1;
            }
            RelationshipType::Challenges => self.challenges_total += 1,
            RelationshipType::Shadows => self.shadows_total += 1,
            RelationshipType::Mentors | RelationshipType::ReportsTo | RelationshipType::Trusts => {
                // No-op for BEH patterns.
            }
        }
    }

    /// Returns the BEH codes that newly trigger given the configured
    /// [`BehaviorThresholds`].
    pub fn triggered_codes(&self, t: &BehaviorThresholds) -> Vec<String> {
        let mut out = Vec::new();
        if self.delegates_total >= t.first_dispatch_count as u64 {
            out.push(BEH_001_FIRST_DELEGATES.to_string());
        }
        if self.consults_total >= t.consults_100_count as u64 {
            out.push(BEH_002_CONSULTS_100.to_string());
        }
        if self.collaborates_total >= t.collaborates_50_count as u64 {
            out.push(BEH_003_COLLAB_50.to_string());
        }
        if self.stand_in_total >= t.stand_in_5_count as u64 {
            out.push(BEH_004_STAND_IN_5.to_string());
        }
        // BEH-005 requires 10 peer-reviews with a 100% accept rate.
        if self.peer_reviews_total >= t.peer_reviews_10_count as u64
            && self.peer_reviews_total == self.peer_reviews_accepted
        {
            out.push(BEH_005_PEER_100PCT_10.to_string());
        }
        if self.challenges_total >= t.challenges_3_count as u64 {
            out.push(BEH_006_CHALLENGE_3.to_string());
        }
        if self.shadows_total >= t.shadows_24h_count as u64 {
            out.push(BEH_007_SHADOWS_24H.to_string());
        }
        out
    }
}

/// `BehaviorEvaluator` (per brief §2.1 A.1).
///
/// The evaluator owns a [`Mutex<BehaviorStats>`] so it can be shared
/// across the 3 evaluators and across the async runtime. Construction
/// is infallible; pass `BehaviorThresholds::default()` for the brief's
/// production thresholds.
#[derive(Debug)]
pub struct BehaviorEvaluator {
    /// Cumulative event counters.
    stats: Mutex<BehaviorStats>,
    /// Per-pattern trigger thresholds.
    thresholds: BehaviorThresholds,
}

impl Default for BehaviorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BehaviorEvaluator {
    fn clone(&self) -> Self {
        let stats = self.stats.lock().expect("behavior stats poisoned").clone();
        Self {
            stats: Mutex::new(stats),
            thresholds: self.thresholds,
        }
    }
}

impl BehaviorEvaluator {
    /// Build a new evaluator with default thresholds (per brief).
    pub fn new() -> Self {
        Self::with_thresholds(BehaviorThresholds::default())
    }

    /// Build a new evaluator with custom thresholds (used by tests).
    pub fn with_thresholds(thresholds: BehaviorThresholds) -> Self {
        Self {
            stats: Mutex::new(BehaviorStats::new()),
            thresholds,
        }
    }

    /// Returns a snapshot of the current stats.
    pub fn stats(&self) -> BehaviorStats {
        self.stats.lock().expect("behavior stats poisoned").clone()
    }

    /// Replace the stats entirely (used by tests for seeding).
    pub fn set_stats(&self, stats: BehaviorStats) {
        let mut guard = self.stats.lock().expect("behavior stats poisoned");
        *guard = stats;
    }

    /// Run the 7 patterns against the supplied [`ARGEvent`].
    ///
    /// The method is idempotent: once a pattern triggers, the same
    /// pattern is returned on every subsequent call (the
    /// `AchievementOps::unlock` layer enforces the real
    /// `(user, code)`-level idempotency, per DD §3.2.5).
    pub async fn evaluate(
        &self,
        event: &ARGEvent,
        _tenant_id: Uuid,
    ) -> Result<Vec<String>, EffectError> {
        // 1. Update the cumulative counters from the event.
        let snapshot = {
            let mut guard = self.stats.lock().expect("behavior stats poisoned");
            guard.record_event(event);
            guard.clone()
        };
        // 2. Return the codes that are now triggered.
        Ok(snapshot.triggered_codes(&self.thresholds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use star_arg::models::edge::{Edge, EdgeDirection};

    fn edge(from: Uuid, to: Uuid, t: RelationshipType, tenant: Uuid) -> Edge {
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
            metadata: serde_json::Value::Null,
            tenant_id: tenant,
            created_at: now,
            updated_at: now,
            version: 1,
            created_by: Uuid::new_v4(),
        }
    }

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime")
            .block_on(f)
    }

    #[test]
    fn beh_001_triggers_on_first_delegates_to_edge() {
        let eval = BehaviorEvaluator::new();
        let t = Uuid::new_v4();
        let evt = ARGEvent::EdgeCreated(edge(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::DelegatesTo,
            t,
        ));
        let code = block_on(eval.evaluate(&evt, t));
        let code = code.expect("ok");
        assert!(code.iter().any(|c| c == BEH_001_FIRST_DELEGATES));
    }

    #[test]
    fn beh_002_triggers_at_100_consults_edges() {
        let eval = BehaviorEvaluator::new();
        let t = Uuid::new_v4();
        // Pre-seed with 99 consults.
        let mut stats = BehaviorStats::new();
        stats.consults_total = 99;
        eval.set_stats(stats);
        // 100th triggers BEH-002.
        let evt = ARGEvent::EdgeCreated(edge(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::Consults,
            t,
        ));
        let code = block_on(eval.evaluate(&evt, t)).expect("ok");
        assert!(code.iter().any(|c| c == BEH_002_CONSULTS_100));
    }

    #[test]
    fn beh_005_requires_100_percent_acceptance() {
        let mut stats = BehaviorStats::new();
        stats.peer_reviews_total = 10;
        stats.peer_reviews_accepted = 9; // not 100%
        let codes = stats.triggered_codes(&BehaviorThresholds::default());
        assert!(!codes.iter().any(|c| c == BEH_005_PEER_100PCT_10));

        stats.peer_reviews_accepted = 10;
        let codes = stats.triggered_codes(&BehaviorThresholds::default());
        assert!(codes.iter().any(|c| c == BEH_005_PEER_100PCT_10));
    }

    #[test]
    fn all_7_codes_unique() {
        let mut set: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for c in ALL_BEH_CODES.iter() {
            assert!(set.insert(*c), "duplicate code: {c}");
        }
    }
}

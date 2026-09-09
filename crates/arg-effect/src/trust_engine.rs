// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ARGTrustEngine` (per DD-AGENT-RELATIONSHIP-001 §4.7).
//!
//! The engine tracks per-agent trust scores and answers the
//! `should_skip_verify` query used by the LangGraph `verify_node`
//! (per DD §4.3.3: trust_score >= 0.8 AND trusts 边 weight >= 0.7).
//!
//! The score update is the canonical `+0.01` (success) / `-0.05`
//! (failure) nudge, clamped to `[0.0, 1.0]` — same as
//! `star_arg::models::trust_score::update_trust_score`. We re-implement
//! the logic locally so the Effect Tier can operate without a live
//! `EdgeOps` / `AgentNodeOps` (per ARG.1 G-1 stub).
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::collections::HashMap;
use std::sync::Mutex;

use star_arg::models::edge::RelationshipType;
use star_arg::models::trust_score::{update_trust_score, TrustScoreTier};
use uuid::Uuid;

use crate::dispatch_router::InMemoryEdgeStore;

/// Threshold above which a trust score qualifies for `should_skip_verify`
/// (per DD §4.3.3).
pub const SKIP_VERIFY_SCORE_THRESHOLD: f32 = 0.8;

/// Minimum `weight` on a `TRUSTS` edge for `should_skip_verify` to fire
/// (per DD §4.3.3).
pub const SKIP_VERIFY_EDGE_WEIGHT_THRESHOLD: f32 = 0.7;

/// In-memory trust score store keyed by `(agent_id, tenant_id)`.
///
/// The real store lives in `star_arg::ops::agent_node` (per ARG.1
/// `AgentNodeOps`). The local copy is sufficient for the Effect Tier
/// to track nudges during a session; the period flush worker (ARG.2)
/// persists them back to Memgraph.
#[derive(Debug, Default, Clone)]
pub struct TrustStore {
    scores: HashMap<(Uuid, Uuid), f32>,
}

impl TrustStore {
    /// Build a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial score for an agent. Overwrites any existing
    /// value (used by tests and by the listener integration).
    pub fn set(&mut self, agent_id: Uuid, tenant_id: Uuid, score: f32) {
        let score = score.clamp(0.0, 1.0);
        self.scores.insert((agent_id, tenant_id), score);
    }

    /// Read the current score for an agent. Returns `None` if the
    /// agent has no recorded score (default 0.5 used by the
    /// `update_*` methods when this is the case).
    pub fn get(&self, agent_id: Uuid, tenant_id: Uuid) -> Option<f32> {
        self.scores.get(&(agent_id, tenant_id)).copied()
    }

    /// Number of `(agent, tenant)` pairs currently tracked.
    pub fn len(&self) -> usize {
        self.scores.len()
    }

    /// True iff the store has no recorded scores.
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }
}

/// `ARGTrustEngine` (per DD §4.7 + arch §3.1).
///
/// The engine takes a shared [`TrustStore`] and [`InMemoryEdgeStore`]
/// and exposes the canonical `record_success` / `record_failure` /
/// `should_skip_verify` API.
#[derive(Debug, Clone)]
pub struct ARGTrustEngine {
    /// Shared trust store.
    store: TrustStore,
    /// Shared edge store (for `TRUSTS` edge lookups in `should_skip_verify`).
    edges: std::sync::Arc<InMemoryEdgeStore>,
    /// Mutex around the trust store (for the rare concurrent-update case).
    _lock: std::sync::Arc<Mutex<()>>,
}

impl ARGTrustEngine {
    /// Build a new engine wrapping the given trust store and edge store.
    pub fn new(store: TrustStore, edges: std::sync::Arc<InMemoryEdgeStore>) -> Self {
        Self {
            store,
            edges,
            _lock: std::sync::Arc::new(Mutex::new(())),
        }
    }

    /// Apply a +0.01 nudge (per DD §4.3.3 success path).
    ///
    /// Returns the new score. If the agent has no recorded score yet,
    /// the default starting score of 0.5 is used (matching `Agent::new`).
    pub fn record_success(&mut self, agent_id: Uuid, tenant_id: Uuid) -> f32 {
        let current = self.store.get(agent_id, tenant_id).unwrap_or(0.5);
        let new_score = update_trust_score(current, true);
        self.store.set(agent_id, tenant_id, new_score);
        new_score
    }

    /// Apply a -0.05 nudge (per DD §4.3.3 failure path).
    pub fn record_failure(&mut self, agent_id: Uuid, tenant_id: Uuid) -> f32 {
        let current = self.store.get(agent_id, tenant_id).unwrap_or(0.5);
        let new_score = update_trust_score(current, false);
        self.store.set(agent_id, tenant_id, new_score);
        new_score
    }

    /// Decide whether the verify node should be skipped for the given
    /// agent (per DD §4.3.3).
    ///
    /// Returns `true` iff:
    /// 1. The agent's trust score is `>= 0.8`, AND
    /// 2. At least one incoming `TRUSTS` edge has `weight >= 0.7`.
    pub fn should_skip_verify(&self, agent_id: Uuid, tenant_id: Uuid) -> bool {
        let score = self.store.get(agent_id, tenant_id).unwrap_or(0.0);
        if score < SKIP_VERIFY_SCORE_THRESHOLD {
            return false;
        }
        let incoming = self.edges.incoming(agent_id, tenant_id);
        incoming.iter().any(|e| {
            e.edge_type == RelationshipType::Trusts && e.weight >= SKIP_VERIFY_EDGE_WEIGHT_THRESHOLD
        })
    }

    /// Current trust score (or default 0.5 if unrecorded).
    pub fn current_score(&self, agent_id: Uuid, tenant_id: Uuid) -> f32 {
        self.store.get(agent_id, tenant_id).unwrap_or(0.5)
    }

    /// Current trust tier for the agent (per `star_arg::models::trust_score::TrustScoreTier`).
    pub fn current_tier(&self, agent_id: Uuid, tenant_id: Uuid) -> TrustScoreTier {
        TrustScoreTier::from_score(self.current_score(agent_id, tenant_id))
    }

    /// Reference to the underlying trust store.
    pub fn store(&self) -> &TrustStore {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use star_arg::models::edge::{Edge, EdgeDirection};

    fn edge(from: Uuid, to: Uuid, t: RelationshipType, weight: f32, tenant: Uuid) -> Edge {
        let now = Utc::now();
        Edge {
            id: Uuid::new_v4(),
            from_agent: from,
            to_agent: to,
            edge_type: t,
            weight,
            direction: EdgeDirection::Directed,
            archived: false,
            metadata: serde_json::Value::Null,
            tenant_id: tenant,
            created_at: now,
            updated_at: now,
            version: 1,
            created_by: Uuid::new_v4(),
        }
    }

    #[test]
    fn record_success_increments_by_0_01() {
        let mut store = TrustStore::new();
        let mut engine = ARGTrustEngine::new(
            std::mem::take(&mut store),
            std::sync::Arc::new(InMemoryEdgeStore::default()),
        );
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        // first call starts from 0.5
        let s1 = engine.record_success(agent, tenant);
        assert!((s1 - 0.51).abs() < 1e-6);
        let s2 = engine.record_success(agent, tenant);
        assert!((s2 - 0.52).abs() < 1e-6);
    }

    #[test]
    fn record_failure_decrements_by_0_05() {
        let mut store = TrustStore::new();
        let mut engine = ARGTrustEngine::new(
            std::mem::take(&mut store),
            std::sync::Arc::new(InMemoryEdgeStore::default()),
        );
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let s1 = engine.record_failure(agent, tenant);
        assert!((s1 - 0.45).abs() < 1e-6);
    }

    #[test]
    fn should_skip_verify_requires_score_and_trust_edge() {
        let tenant = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let granter = Uuid::new_v4();

        let mut trust = TrustStore::new();
        trust.set(agent, tenant, 0.85);

        let mut edges = InMemoryEdgeStore::default();
        edges.add(edge(granter, agent, RelationshipType::Trusts, 0.8, tenant));

        let engine = ARGTrustEngine::new(trust, std::sync::Arc::new(edges));
        assert!(engine.should_skip_verify(agent, tenant));
    }

    #[test]
    fn should_not_skip_when_trust_edge_weight_low() {
        let tenant = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let granter = Uuid::new_v4();

        let mut trust = TrustStore::new();
        trust.set(agent, tenant, 0.85);

        let mut edges = InMemoryEdgeStore::default();
        edges.add(edge(granter, agent, RelationshipType::Trusts, 0.5, tenant));

        let engine = ARGTrustEngine::new(trust, std::sync::Arc::new(edges));
        assert!(!engine.should_skip_verify(agent, tenant));
    }

    #[test]
    fn should_not_skip_when_score_below_threshold() {
        let tenant = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let granter = Uuid::new_v4();

        let mut trust = TrustStore::new();
        trust.set(agent, tenant, 0.7);

        let mut edges = InMemoryEdgeStore::default();
        edges.add(edge(granter, agent, RelationshipType::Trusts, 0.9, tenant));

        let engine = ARGTrustEngine::new(trust, std::sync::Arc::new(edges));
        assert!(!engine.should_skip_verify(agent, tenant));
    }
}

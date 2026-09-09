// SPDX-License-Identifier: MIT OR Apache-2.0
//! `LangGraphStateUpdater` — `ARGEvent` → 5 Reducer (per DD §4.11 + §5.3).
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §4.11
//! + [`docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md`](../../docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) §4.
//!
//! The 5 Reducer functions mirror the Python Reducers defined in
//! `docs/design/DD-AGENT-RELATIONSHIP-001.md` §5.1 and applied to the
//! LangGraph `TopAgentState` (per ADR-0046 + TMO 7 nodes):
//!
//! 1. `merge_arg_agents`        — by `Agent.id`, LWW
//! 2. `merge_arg_edges`         — by `(from, to, type)`, higher `version` wins
//! 3. `merge_trust_scores`      — per-key, value = `max(existing, new)`
//! 4. `merge_dispatch`          — per-key, value = list union (dedup)
//! 5. `add`                     — for `achievements_unlocked`
//!
//! The current implementation applies the Reducers to in-process state
//! snapshots (`LangGraphStateStore`). The PyO3 binding that actually
//! mutates the Python `TopAgentState` lands in ARG.3 / P3-C W3; the
//! semantics implemented here are the same and act as the contract test
//! for that binding.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::debug;
use uuid::Uuid;

use star_arg::models::agent::{Agent, AgentStatus};
use star_arg::models::edge::{Edge, RelationshipType};

use crate::error::BridgeError;
use crate::protocol::BridgeEnvelope;

/// In-process state store that mirrors the LangGraph `TopAgentState`
/// 5 channel shape (per DD §5.1).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LangGraphStateStore {
    /// `arg_agents` — by `Agent.id`, LWW.
    pub arg_agents: HashMap<Uuid, Agent>,
    /// `arg_edges` — by `(from, to, type)`, higher `version` wins.
    pub arg_edges: HashMap<EdgeKey, Edge>,
    /// `arg_trust_scores` — per agent id, value = `max(existing, new)`.
    pub arg_trust_scores: HashMap<Uuid, f64>,
    /// `arg_dispatch_overrides` — per `from_agent_id`, list of target ids.
    pub arg_dispatch_overrides: HashMap<Uuid, Vec<Uuid>>,
    /// `arg_achievements_unlocked` — append-only.
    pub arg_achievements_unlocked: Vec<String>,
}

/// Composite key for the `arg_edges` channel (per DD §5.1 `merge_arg_edges`).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeKey {
    /// Source agent id.
    pub from: Uuid,
    /// Target agent id.
    pub to: Uuid,
    /// Relationship label (e.g. `"DELEGATES_TO"`).
    pub relationship_type: RelationshipType,
}

impl EdgeKey {
    /// Build a key from an [`Edge`].
    pub fn from_edge(edge: &Edge) -> Self {
        Self {
            from: edge.from_agent,
            to: edge.to_agent,
            relationship_type: edge.edge_type,
        }
    }
}

/// 5 Reducer kind — used as a public enumeration of the Reducers
/// implemented in DD §5.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReducerKind {
    /// LWW merge by `Agent.id` (DD §5.1 `merge_arg_agents`).
    MergeArgAgents,
    /// Merge by `(from, to, type)`, higher `version` wins (DD §5.1 `merge_arg_edges`).
    MergeArgEdges,
    /// Per-key `max(existing, new)` (DD §5.1 `merge_trust_scores`).
    MergeTrustScores,
    /// Per-key list union (dedup) (DD §5.1 `merge_dispatch`).
    MergeDispatch,
    /// Append (DD §5.1 `add` for `arg_achievements_unlocked`).
    Add,
}

impl ReducerKind {
    /// Stable string code used in metric / log labels.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MergeArgAgents => "merge_arg_agents",
            Self::MergeArgEdges => "merge_arg_edges",
            Self::MergeTrustScores => "merge_trust_scores",
            Self::MergeDispatch => "merge_dispatch",
            Self::Add => "add",
        }
    }

    /// All 5 Reducers in canonical order.
    pub fn all() -> [ReducerKind; 5] {
        [
            Self::MergeArgAgents,
            Self::MergeArgEdges,
            Self::MergeTrustScores,
            Self::MergeDispatch,
            Self::Add,
        ]
    }
}

/// `LangGraphStateUpdater` — receives [`BridgeEnvelope`]s and applies the
/// 5 Reducers to an in-process [`LangGraphStateStore`].
#[derive(Debug)]
pub struct LangGraphStateUpdater {
    /// Shared state store (mirrors the Python `TopAgentState`).
    state: Arc<Mutex<LangGraphStateStore>>,
}

impl LangGraphStateUpdater {
    /// Build a new updater wrapping a fresh empty state store.
    pub fn new() -> Self {
        Self::with_state(Arc::new(Mutex::new(LangGraphStateStore::default())))
    }

    /// Build a new updater wrapping the given state store.
    pub fn with_state(state: Arc<Mutex<LangGraphStateStore>>) -> Self {
        Self { state }
    }

    /// Apply an envelope to the in-process state store.
    pub async fn update_state(&self, envelope: &BridgeEnvelope) -> Result<(), BridgeError> {
        use crate::protocol::BridgeEnvelopeKind;
        let mut state = self.state.lock().await;
        match &envelope.kind {
            BridgeEnvelopeKind::EdgeChanged(e) => {
                debug!(
                    edge_id = %e.edge_id,
                    kind = "arg_edge_changed",
                    "LangGraphStateUpdater::update_state"
                );
                // map to MergeArgEdges conceptually; an Edge struct is
                // synthesised with the broadcast payload.
                let key = EdgeKey {
                    from: e.from_agent_id,
                    to: e.to_agent_id,
                    relationship_type: Self::label_to_type(&e.relationship_type),
                };
                state.arg_edges.entry(key).or_insert_with(|| {
                    // First time we see the edge; ignore further updates
                    // until a full Edge row arrives via EdgeCreated.
                    Edge {
                        id: e.edge_id,
                        from_agent: e.from_agent_id,
                        to_agent: e.to_agent_id,
                        edge_type: key.relationship_type,
                        weight: e.weight as f32,
                        direction: star_arg::models::edge::EdgeDirection::Directed,
                        archived: e.relationship_type == "ARCHIVED",
                        metadata: serde_json::Value::Null,
                        tenant_id: Uuid::nil(),
                        created_at: e.created_at,
                        updated_at: e.created_at,
                        version: 1,
                        created_by: Uuid::nil(),
                    }
                });
            }
            BridgeEnvelopeKind::TrustScoreUpdate(t) => {
                let entry = state
                    .arg_trust_scores
                    .entry(t.agent_id)
                    .or_insert(t.new_score);
                if t.new_score > *entry {
                    *entry = t.new_score;
                }
            }
            BridgeEnvelopeKind::DispatchRoute(r) => {
                let list = state
                    .arg_dispatch_overrides
                    .entry(r.from_agent_id)
                    .or_default();
                if !list.contains(&r.to_agent_id) {
                    list.push(r.to_agent_id);
                }
            }
            BridgeEnvelopeKind::ContextInject(c) => {
                debug!(
                    inject_id = %c.inject_id,
                    "context_inject received (no reducer for this protocol yet)"
                );
            }
            BridgeEnvelopeKind::AchievementUnlocked(a) => {
                state
                    .arg_achievements_unlocked
                    .push(a.achievement_id.clone());
            }
        }
        Ok(())
    }

    /// Snapshot the current state store (for testing / external sync).
    pub async fn snapshot(&self) -> LangGraphStateStore {
        self.state.lock().await.clone()
    }

    /// Count the number of envelopes successfully applied (for tests).
    pub async fn arg_agent_count(&self) -> usize {
        self.state.lock().await.arg_agents.len()
    }

    /// Count the number of edges in the in-process store (for tests).
    pub async fn arg_edge_count(&self) -> usize {
        self.state.lock().await.arg_edges.len()
    }

    /// Look up a stored agent by id (for tests).
    pub async fn get_agent(&self, id: Uuid) -> Option<Agent> {
        self.state.lock().await.arg_agents.get(&id).cloned()
    }

    /// Look up a stored edge by id (for tests).
    pub async fn get_edge(&self, id: Uuid) -> Option<Edge> {
        self.state
            .lock()
            .await
            .arg_edges
            .values()
            .find(|e| e.id == id)
            .cloned()
    }

    /// Upsert an [`Agent`] (used by tests + bridge boot path).
    pub async fn put_agent(&self, agent: Agent) {
        self.state.lock().await.arg_agents.insert(agent.id, agent);
    }

    /// Upsert an [`Edge`] applying the `merge_arg_edges` Reducer rules.
    pub async fn put_edge(&self, edge: Edge) {
        let key = EdgeKey::from_edge(&edge);
        let mut state = self.state.lock().await;
        match state.arg_edges.get(&key) {
            Some(existing) if existing.version >= edge.version => {
                // lower-version write, ignore
            }
            _ => {
                state.arg_edges.insert(key, edge);
            }
        }
    }

    /// Apply the `merge_arg_agents` Reducer manually (for unit tests).
    pub async fn reducer_merge_arg_agents(&self, new: Vec<Agent>) {
        let mut state = self.state.lock().await;
        for a in new {
            match state.arg_agents.get(&a.id) {
                Some(existing) if existing.version >= a.version => {}
                _ => {
                    state.arg_agents.insert(a.id, a);
                }
            }
        }
    }

    /// Apply the `merge_arg_edges` Reducer manually (for unit tests).
    pub async fn reducer_merge_arg_edges(&self, new: Vec<Edge>) {
        let mut state = self.state.lock().await;
        for e in new {
            let key = EdgeKey::from_edge(&e);
            match state.arg_edges.get(&key) {
                Some(existing) if existing.version >= e.version => {}
                _ => {
                    state.arg_edges.insert(key, e);
                }
            }
        }
    }

    /// Apply the `merge_trust_scores` Reducer manually (for unit tests).
    pub async fn reducer_merge_trust_scores(&self, new: &HashMap<Uuid, f64>) {
        let mut state = self.state.lock().await;
        for (k, v) in new {
            let entry = state.arg_trust_scores.entry(*k).or_insert(0.0);
            if *v > *entry {
                *entry = *v;
            }
        }
    }

    /// Apply the `merge_dispatch` Reducer manually (for unit tests).
    pub async fn reducer_merge_dispatch(&self, new: &HashMap<Uuid, Vec<Uuid>>) {
        let mut state = self.state.lock().await;
        for (k, v) in new {
            let entry = state.arg_dispatch_overrides.entry(*k).or_default();
            let seen: HashSet<Uuid> = entry.iter().copied().collect();
            for t in v {
                if !seen.contains(t) {
                    entry.push(*t);
                }
            }
        }
    }

    /// Apply the `add` Reducer manually (for unit tests).
    pub async fn reducer_add(&self, items: Vec<String>) {
        self.state
            .lock()
            .await
            .arg_achievements_unlocked
            .extend(items);
    }

    fn label_to_type(label: &str) -> RelationshipType {
        match label {
            "DELEGATES_TO" => RelationshipType::DelegatesTo,
            "CONSULTS" => RelationshipType::Consults,
            "COLLABORATES_WITH" => RelationshipType::CollaboratesWith,
            "REPORTS_TO" => RelationshipType::ReportsTo,
            "MENTORS" => RelationshipType::Mentors,
            "PEER_REVIEWS" => RelationshipType::PeerReviews,
            "STAND_IN_FOR" => RelationshipType::StandInFor,
            "SHADOWS" => RelationshipType::Shadows,
            "CHALLENGES" => RelationshipType::Challenges,
            "TRUSTS" => RelationshipType::Trusts,
            _ => RelationshipType::DelegatesTo,
        }
    }
}

impl Default for LangGraphStateUpdater {
    fn default() -> Self {
        Self::new()
    }
}

/// Touch a couple of model-side enums so the public surface stays
/// consistent with ARG.1 (kept for the ARG.3 PyO3 binding to reuse).
pub fn active_status() -> AgentStatus {
    AgentStatus::Active
}

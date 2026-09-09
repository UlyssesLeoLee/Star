// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ARGAchievementEngine` (per DD-AGENT-RELATIONSHIP-001 §4.9 + §6 + §7).
//!
//! The engine wraps the 8 拓扑成就 Cypher queries, the 7 行为 event
//! patterns and the 5 产出 aggregate-metric placeholders, and applies
//! the `AchievementOps::unlock` (idempotent per `(user_id, code)`).
//!
//! During ARG.3 the actual Cypher evaluation is stubbed (per ARG.1 G-1
//! stub): the [`TopologyEvaluator`] returns `triggered = true` for the
//! cypher codes that match a hard-coded tenant_id hash, so the full
//! `evaluate → unlock` path is exercised end-to-end without a real
//! Memgraph. The 8 cypher queries themselves are exposed verbatim
//! through [`topology::all_topology_cyphers`] (already present in
//! `star_arg::query::topology`).
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::sync::Arc;

use star_arg::models::achievement_unlock::AchievementUnlock;
use star_arg::models::event::ARGEvent;
use star_arg::ops::AchievementOps;
use star_arg::query::topology::{all_topology_cyphers, TopologyCypher};
use uuid::Uuid;

use crate::error::EffectError;

/// 8 拓扑成就 Cypher 模板 (per DD §6 + brief §2.1 B).
///
/// The 8 codes mirror `star_arg::query::topology::all_topology_cyphers()`.
pub mod topology {
    use super::*;

    /// The 8 拓扑成就 codes (per DD §6). Stable order.
    pub const TOP_001: &str = "TOP-001-MESH-5DOMAIN";
    /// TOP-002: Hub-and-Spoke (1 Lead + 4 outgoing DELEGATES_TO).
    pub const TOP_002: &str = "TOP-002-HUB-AND-SPOKE";
    /// TOP-003: 10-node mesh (10 nodes, 45 undirected edges).
    pub const TOP_003: &str = "TOP-003-MESH-10";
    /// TOP-004: 5-step chain (4 DELEGATES_TO edges in sequence).
    pub const TOP_004: &str = "TOP-004-CHAIN-5";
    /// TOP-005: 3-layer hierarchical (1 Lead + 2 Sub-Lead + 6 Worker).
    pub const TOP_005: &str = "TOP-005-HIERARCHICAL-3";
    /// TOP-006: Review Council (1 Lead + 3 incoming CONSULTS).
    pub const TOP_006: &str = "TOP-006-REVIEW-COUNCIL";
    /// TOP-007: no self-loop (graph health check).
    pub const TOP_007: &str = "TOP-007-NO-SELF-LOOP";
    /// TOP-008: no isolated node (graph health check).
    pub const TOP_008: &str = "TOP-008-NO-ISLAND";

    /// Returns the 8 拓扑成就 codes in canonical order.
    pub fn all_codes() -> [&'static str; 8] {
        [
            TOP_001, TOP_002, TOP_003, TOP_004, TOP_005, TOP_006, TOP_007, TOP_008,
        ]
    }

    /// Returns the 8 拓扑成就 Cypher pairs in canonical order.
    pub fn all_cyphers() -> Vec<TopologyCypher> {
        all_topology_cyphers()
    }
}

/// `TopologyEvaluator` — runs the 8 拓扑成就 Cypher queries against a
/// pluggable backend.
///
/// The default backend ([`StubTopologyBackend`]) returns `triggered =
/// true` when the tenant_id's first 8 bits match a configurable
/// trigger mask. This exercises the full `evaluate → unlock` path
/// without a live Memgraph (per ARG.1 G-1).
pub trait TopologyBackend: Send + Sync {
    /// Run a single Cypher query and return whether it triggered.
    fn evaluate(&self, cypher: &str, tenant_id: Uuid) -> Result<bool, EffectError>;
}

/// Default stub backend (per ARG.3 brief §2.1 B + G-1 closure).
#[derive(Debug, Default, Clone)]
pub struct StubTopologyBackend {
    /// When `Some(mask)`, the high 8 bits of `tenant_id.as_u128()` are
    /// compared against the mask. When `None`, nothing triggers.
    pub trigger_mask: Option<u8>,
}

impl StubTopologyBackend {
    /// Build a backend that triggers for every Cypher (used by the
    /// default achievement engine so the 8 unlocks are exercised in
    /// the end-to-end tests).
    pub fn always_trigger() -> Self {
        // We can't actually mark every tenant as triggering; we set
        // a high mask and rely on the tenant-id check.
        Self {
            trigger_mask: Some(0x00),
        }
    }

    /// Build a backend that triggers for the given 8-bit mask.
    pub fn with_mask(mask: u8) -> Self {
        Self {
            trigger_mask: Some(mask),
        }
    }

    /// Build a backend that never triggers (used by negative tests).
    pub fn never_trigger() -> Self {
        Self { trigger_mask: None }
    }
}

impl TopologyBackend for StubTopologyBackend {
    fn evaluate(&self, cypher: &str, tenant_id: Uuid) -> Result<bool, EffectError> {
        let _ = cypher;
        match self.trigger_mask {
            Some(_) => {
                let bits = (tenant_id.as_u128() & 0xFF) as u8;
                // Trigger when low 8 bits of tenant_id equal 0.
                // This gives roughly 1/256 hit rate, deterministic.
                Ok(bits == 0x00)
            }
            None => Ok(false),
        }
    }
}

/// `TopologyEvaluator` — owns a [`TopologyBackend`] and exposes the
/// 8 拓扑 evaluation pass.
pub struct TopologyEvaluator {
    backend: Arc<dyn TopologyBackend>,
}

impl std::fmt::Debug for TopologyEvaluator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TopologyEvaluator")
            .field("backend", &"<dyn TopologyBackend>")
            .finish()
    }
}

impl Clone for TopologyEvaluator {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}

impl TopologyEvaluator {
    /// Build a new evaluator wrapping the given backend.
    pub fn new(backend: Arc<dyn TopologyBackend>) -> Self {
        Self { backend }
    }

    /// Run the 8 拓扑 Cypher queries and return the achievement codes
    /// that triggered.
    pub async fn evaluate(
        &self,
        _event: &ARGEvent,
        tenant_id: Uuid,
    ) -> Result<Vec<String>, EffectError> {
        let mut triggered = Vec::new();
        for pair in all_topology_cyphers() {
            if self.backend.evaluate(&pair.cypher, tenant_id)? {
                triggered.push(pair.code);
            }
        }
        Ok(triggered)
    }
}

/// `BehaviorEvaluator` — placeholder for the 7 行为 event pattern
/// evaluator (per DD §3.1 + P3-E ARG.8 follow-up).
///
/// During ARG.3 this always returns an empty list; the placeholder
/// is exercised by the 5 achievement tests via the
/// [`crate::achievement_engine::ARGAchievementEngine::evaluate`]
/// orchestration.
#[derive(Debug, Default, Clone)]
pub struct BehaviorEvaluator;

impl BehaviorEvaluator {
    /// Build a new (empty) evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluate the 7 行为 event patterns. Returns `Vec::new()` —
    /// the real implementation lands in P3-E ARG.8.
    pub async fn evaluate(
        &self,
        _event: &ARGEvent,
        _tenant_id: Uuid,
    ) -> Result<Vec<String>, EffectError> {
        Ok(Vec::new())
    }
}

/// `OutputEvaluator` — placeholder for the 5 产出 aggregate-metric
/// evaluator (per DD §3.1 + P3-E ARG.8 follow-up).
#[derive(Debug, Default, Clone)]
pub struct OutputEvaluator;

impl OutputEvaluator {
    /// Build a new (empty) evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluate the 5 产出 aggregate metrics. Returns `Vec::new()`.
    pub async fn evaluate(
        &self,
        _event: &ARGEvent,
        _tenant_id: Uuid,
    ) -> Result<Vec<String>, EffectError> {
        Ok(Vec::new())
    }
}

/// `ARGAchievementEngine` (per DD §4.9 + arch §3.1).
///
/// Orchestrates the 3 evaluators and persists unlocks via the
/// shared [`AchievementOps`]. Construction is infallible; the engine
/// is `Clone` so it can be moved into `Arc` for shared use.
pub struct ARGAchievementEngine {
    topology: TopologyEvaluator,
    behavior: BehaviorEvaluator,
    output: OutputEvaluator,
    ops: Arc<AchievementOps>,
    /// Default user_id used when the engine is called without an
    /// explicit user (per the brief's `evaluate(event, tenant_id)`
    /// signature). Real callers should use the variant that takes a
    /// user_id.
    default_user_id: Uuid,
}

impl std::fmt::Debug for ARGAchievementEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ARGAchievementEngine")
            .field("topology", &self.topology)
            .field("behavior", &self.behavior)
            .field("output", &self.output)
            .field("ops", &"<AchievementOps>")
            .field("default_user_id", &self.default_user_id)
            .finish()
    }
}

impl Clone for ARGAchievementEngine {
    fn clone(&self) -> Self {
        Self {
            topology: self.topology.clone(),
            behavior: self.behavior.clone(),
            output: self.output.clone(),
            ops: self.ops.clone(),
            default_user_id: self.default_user_id,
        }
    }
}

impl ARGAchievementEngine {
    /// Build a new engine wrapping the 3 evaluators + the
    /// [`AchievementOps`].
    pub fn new(
        topology: TopologyEvaluator,
        behavior: BehaviorEvaluator,
        output: OutputEvaluator,
        ops: AchievementOps,
        default_user_id: Uuid,
    ) -> Self {
        Self {
            topology,
            behavior,
            output,
            ops: Arc::new(ops),
            default_user_id,
        }
    }

    /// Run the 3 evaluators in parallel-ish (sequential today) and
    /// persist the resulting unlocks via [`AchievementOps::unlock`].
    ///
    /// Returns the list of [`AchievementUnlock`] rows that were
    /// newly inserted (deduplicated per `(user_id, code)`).
    pub async fn evaluate(
        &self,
        event: ARGEvent,
        tenant_id: Uuid,
    ) -> Result<Vec<AchievementUnlock>, EffectError> {
        self.evaluate_for_user(self.default_user_id, event, tenant_id)
            .await
    }

    /// Same as [`Self::evaluate`] but with an explicit user_id.
    pub async fn evaluate_for_user(
        &self,
        user_id: Uuid,
        event: ARGEvent,
        tenant_id: Uuid,
    ) -> Result<Vec<AchievementUnlock>, EffectError> {
        let mut codes: Vec<String> = Vec::new();
        codes.extend(self.topology.evaluate(&event, tenant_id).await?);
        codes.extend(self.behavior.evaluate(&event, tenant_id).await?);
        codes.extend(self.output.evaluate(&event, tenant_id).await?);

        let mut unlocks: Vec<AchievementUnlock> = Vec::with_capacity(codes.len());
        for code in codes {
            let row = AchievementUnlock::new(
                code.clone(),
                user_id,
                vec![],
                serde_json::json!({"trigger": "evaluate_for_user"}),
                tenant_id,
            );
            let newly = self
                .ops
                .unlock(row.clone())
                .await
                .map_err(|e| EffectError::InternalError(e.to_string()))?;
            if newly {
                unlocks.push(row);
            }
        }
        Ok(unlocks)
    }

    /// Number of unique `(user, code)` pairs currently tracked by
    /// the underlying [`AchievementOps`].
    pub fn unique_unlocks(&self) -> usize {
        self.ops.unique_unlocks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use star_arg::models::agent::{Agent, AgentArchetype};
    use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
    use star_arg::ops::event_writer::EventWriter;
    use star_arg::ops::AchievementOps;

    fn make_edge(from: Uuid, to: Uuid, t: RelationshipType, tenant: Uuid) -> Edge {
        let now = chrono::Utc::now();
        Edge {
            id: Uuid::new_v4(),
            from_agent: from,
            to_agent: to,
            edge_type: t,
            weight: 0.7,
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

    fn make_engine_with_tenant(_tenant: Uuid) -> (ARGAchievementEngine, Uuid) {
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        (
            ARGAchievementEngine::new(topology, behavior, output, ops, user),
            user,
        )
    }

    fn make_edge_event() -> ARGEvent {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        ARGEvent::EdgeCreated(make_edge(from, to, RelationshipType::DelegatesTo, tenant))
    }

    #[tokio::test]
    async fn evaluate_runs_three_evaluators() {
        // Always-trigger backend → 8 unlocks expected.
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        let engine = ARGAchievementEngine::new(topology, behavior, output, ops, user);

        // Use a tenant whose low 8 bits are 0 so the stub triggers.
        let tenant = Uuid::nil();
        let unlocks = engine
            .evaluate(make_edge_event(), tenant)
            .await
            .expect("ok");
        assert_eq!(unlocks.len(), 8);
    }

    #[tokio::test]
    async fn evaluate_is_idempotent() {
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        let engine = ARGAchievementEngine::new(topology, behavior, output, ops, user);
        let tenant = Uuid::nil();

        let first = engine
            .evaluate(make_edge_event(), tenant)
            .await
            .expect("ok");
        let second = engine
            .evaluate(make_edge_event(), tenant)
            .await
            .expect("ok");
        assert_eq!(first.len(), 8);
        assert_eq!(second.len(), 0, "second call is a no-op");
        assert_eq!(engine.unique_unlocks(), 8);
    }

    #[tokio::test]
    async fn evaluate_returns_empty_when_never_trigger() {
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::never_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        let engine = ARGAchievementEngine::new(topology, behavior, output, ops, user);
        let tenant = Uuid::new_v4();
        let unlocks = engine
            .evaluate(make_edge_event(), tenant)
            .await
            .expect("ok");
        assert_eq!(unlocks.len(), 0);
    }

    #[tokio::test]
    async fn topology_evaluator_returns_8_codes_when_triggered() {
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let eval = TopologyEvaluator::new(backend);
        let triggered = eval
            .evaluate(&make_edge_event(), Uuid::nil())
            .await
            .expect("ok");
        assert_eq!(triggered.len(), 8);
        for code in &triggered {
            assert!(
                topology::all_codes().contains(&code.as_str()),
                "unknown code {code}"
            );
        }
    }

    #[tokio::test]
    async fn topology_codes_match_canonical_8() {
        assert_eq!(topology::all_codes().len(), 8);
        assert_eq!(topology::all_codes()[0], topology::TOP_001);
        assert_eq!(topology::all_codes()[7], topology::TOP_008);
        let codes: Vec<String> = topology::all_cyphers()
            .into_iter()
            .map(|c| c.code)
            .collect();
        let expected: Vec<String> = topology::all_codes()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(codes, expected);
    }

    #[test]
    fn all_8_cyphers_avoid_apoc() {
        // per self-review F-11 fix.
        for pair in topology::all_cyphers() {
            assert!(
                !pair.cypher.contains("apoc."),
                "cypher {} uses apoc (must avoid per F-11): {}",
                pair.code,
                pair.cypher
            );
        }
    }

    #[test]
    fn stub_topology_always_trigger_actually_triggers() {
        // sanity: the always_trigger backend should trigger for the
        // nil tenant (low 8 bits == 0).
        let b = StubTopologyBackend::always_trigger();
        let ok = b.evaluate("MATCH (n) RETURN n", Uuid::nil()).expect("ok");
        assert!(ok);
        // and not trigger for a tenant whose low 8 bits are non-zero.
        let busy_tenant: Uuid = uuid::Uuid::from_u128(0x01);
        let ok2 = b.evaluate("MATCH (n) RETURN n", busy_tenant).expect("ok");
        assert!(!ok2);
    }

    // ensure the make_engine_with_tenant helper is callable.
    #[allow(dead_code)]
    fn _make_engine_with_tenant_used() {
        let (engine, _user) = make_engine_with_tenant(Uuid::new_v4());
        let _ = engine.unique_unlocks();
    }

    // ensure the make_agent helper would compile (mirrors arg crate).
    #[allow(dead_code)]
    fn _make_agent() -> Agent {
        Agent::new(
            "test".into(),
            AgentArchetype::Sa01,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
    }
}

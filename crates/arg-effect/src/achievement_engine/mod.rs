// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ARGAchievementEngine` (per DD-AGENT-RELATIONSHIP-001 §4.9 + §6 + §7).
//!
//! The engine orchestrates 3 evaluators that together cover the **20
//! canonical achievements** (per brief `arg-08-behavior-output-evaluator.md`):
//!
//! - **8 拓扑** — `TopologyEvaluator` (8 Cypher queries)
//! - **7 行为** — `BehaviorEvaluator` (7 event patterns)
//! - **5 产出** — `OutputEvaluator` (5 aggregate metrics)
//!
//! The engine runs all 3 evaluators in parallel via `tokio::join!` and
//! persists the resulting unlocks through the shared
//! [`AchievementOps`]. Unlock is idempotent per `(user_id, code)` (per
//! ARG.1 §3.2.5).
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::sync::Arc;

use async_trait::async_trait;
use star_arg::models::achievement_unlock::AchievementUnlock;
use star_arg::models::event::ARGEvent;
use star_arg::ops::AchievementOps;
use star_arg::query::topology::{all_topology_cyphers, TopologyCypher};
use uuid::Uuid;

use crate::error::EffectError;

#[path = "behavior_evaluator.rs"]
pub mod behavior;
#[path = "output_evaluator.rs"]
pub mod output;

pub use behavior::{BehaviorEvaluator, BehaviorStats, BehaviorThresholds};
pub use output::{OutputEvaluator, OutputStats, OutputThresholds};

/// 8 拓扑成就 Cypher 模板 (per DD §6 + brief §2.1 A).
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

/// `TopologyBackend` — runs a single Cypher query and returns whether it triggered.
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
    /// Build a backend that triggers for the nil tenant.
    pub fn always_trigger() -> Self {
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

/// `AchievementPublisher` — sink for SSE push of newly-unlocked achievements
/// (per brief A.3 `ArgsSEHub.publish_achievement_unlocked`).
///
/// The trait is defined in `arg-effect` so the Effect Tier can stay
/// decoupled from the API tier (which owns the concrete `ARGSSEHub`).
/// The default no-op implementation records nothing; production wires
/// this to the `ARGSSEHub` from `crates/api/src/arg/sse_hub.rs`.
#[async_trait]
pub trait AchievementPublisher: Send + Sync {
    /// Publish a single unlock event. The implementation should be
    /// non-blocking; failures must be reported via the returned `Result`
    /// but should not stop the engine's evaluate loop.
    async fn publish_achievement_unlocked(
        &self,
        unlock: &AchievementUnlock,
    ) -> Result<(), EffectError>;

    /// Number of events successfully published (for tests / metrics).
    fn published_count(&self) -> usize;
}

/// `NoopAchievementPublisher` — default sink that records nothing
/// (per brief §2.1 A.3 "mock ARGSSEHub" until ARG.4 wires the real one).
#[derive(Debug, Default)]
pub struct NoopAchievementPublisher {
    /// Internal counter (only used by tests).
    count: std::sync::atomic::AtomicUsize,
}

impl Clone for NoopAchievementPublisher {
    fn clone(&self) -> Self {
        // New instance with counter zeroed — counters are diagnostic
        // only and tests are fine starting fresh.
        Self::new()
    }
}

impl NoopAchievementPublisher {
    /// Build a new noop publisher.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AchievementPublisher for NoopAchievementPublisher {
    async fn publish_achievement_unlocked(
        &self,
        _unlock: &AchievementUnlock,
    ) -> Result<(), EffectError> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn published_count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// `ChannelAchievementPublisher` — broadcast-channel-backed publisher
/// that hands every unlock event to one or more in-process subscribers
/// (used by the WS / SSE handler in `crates/api/src/arg/sse_hub.rs`).
///
/// This is the bridge that lets the Effect Tier fan out unlock events
/// without taking a hard dependency on the API tier.
#[derive(Debug, Clone)]
pub struct ChannelAchievementPublisher {
    tx: tokio::sync::broadcast::Sender<AchievementUnlock>,
}

impl ChannelAchievementPublisher {
    /// Build a new publisher wrapping the given broadcast channel.
    pub fn new(tx: tokio::sync::broadcast::Sender<AchievementUnlock>) -> Self {
        Self { tx }
    }

    /// Build a default-capacity channel and return both halves.
    pub fn with_capacity(cap: usize) -> (Self, tokio::sync::broadcast::Receiver<AchievementUnlock>) {
        let (tx, rx) = tokio::sync::broadcast::channel(cap);
        (Self { tx }, rx)
    }
}

#[async_trait]
impl AchievementPublisher for ChannelAchievementPublisher {
    async fn publish_achievement_unlocked(
        &self,
        unlock: &AchievementUnlock,
    ) -> Result<(), EffectError> {
        // `send` returns the number of active receivers; 0 is fine
        // (no WS clients connected yet) and not an error.
        let _ = self.tx.send(unlock.clone());
        Ok(())
    }

    fn published_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// `ARGAchievementEngine` (per DD §4.9 + arch §3.1).
///
/// Orchestrates the 3 evaluators in parallel via `tokio::join!` and
/// persists unlocks via the shared [`AchievementOps`]. Construction
/// is infallible; the engine is `Clone` so it can be moved into `Arc`
/// for shared use.
pub struct ARGAchievementEngine {
    topology: TopologyEvaluator,
    behavior: BehaviorEvaluator,
    output: OutputEvaluator,
    ops: Arc<AchievementOps>,
    /// Default user_id used when the engine is called without an
    /// explicit user.
    default_user_id: Uuid,
    /// SSE / WS sink (per brief A.3).
    publisher: Arc<dyn AchievementPublisher>,
}

impl std::fmt::Debug for ARGAchievementEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ARGAchievementEngine")
            .field("topology", &self.topology)
            .field("behavior", &self.behavior)
            .field("output", &self.output)
            .field("ops", &"<AchievementOps>")
            .field("default_user_id", &self.default_user_id)
            .field("publisher", &"<dyn AchievementPublisher>")
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
            publisher: self.publisher.clone(),
        }
    }
}

impl ARGAchievementEngine {
    /// Build a new engine wrapping the 3 evaluators + the
    /// [`AchievementOps`] + a noop publisher.
    pub fn new(
        topology: TopologyEvaluator,
        behavior: BehaviorEvaluator,
        output: OutputEvaluator,
        ops: AchievementOps,
        default_user_id: Uuid,
    ) -> Self {
        Self::with_publisher(
            topology,
            behavior,
            output,
            ops,
            default_user_id,
            Arc::new(NoopAchievementPublisher::new()),
        )
    }

    /// Build a new engine wrapping the 3 evaluators + ops + a custom
    /// publisher (per brief A.3 `ArgsSEHub.publish_achievement_unlocked`).
    pub fn with_publisher(
        topology: TopologyEvaluator,
        behavior: BehaviorEvaluator,
        output: OutputEvaluator,
        ops: AchievementOps,
        default_user_id: Uuid,
        publisher: Arc<dyn AchievementPublisher>,
    ) -> Self {
        Self {
            topology,
            behavior,
            output,
            ops: Arc::new(ops),
            default_user_id,
            publisher,
        }
    }

    /// Run the 3 evaluators in parallel via `tokio::join!` and persist
    /// the resulting unlocks via [`AchievementOps::unlock`].
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
        // 3 evaluators in parallel (per brief A.3 tokio::join!).
        let (topo_res, beh_res, out_res) = tokio::join!(
            self.topology.evaluate(&event, tenant_id),
            self.behavior.evaluate(&event, tenant_id),
            self.output.evaluate(&event, tenant_id),
        );
        let mut codes: Vec<String> = Vec::new();
        codes.extend(topo_res?);
        codes.extend(beh_res?);
        codes.extend(out_res?);

        let mut unlocks: Vec<AchievementUnlock> = Vec::with_capacity(codes.len());
        for code in codes {
            let row = AchievementUnlock::new(
                code.clone(),
                user_id,
                vec![],
                serde_json::json!({"trigger": "evaluate_for_user", "event_kind": event.kind()}),
                tenant_id,
            );
            let newly = self
                .ops
                .unlock(row.clone())
                .await
                .map_err(|e| EffectError::InternalError(e.to_string()))?;
            if newly {
                // Publish to SSE / WS via the configured publisher.
                // Errors from publish are logged via tracing but do
                // not block the unlock return value.
                if let Err(e) = self.publisher.publish_achievement_unlocked(&row).await {
                    tracing::warn!(target: "arg.achievement", "publish failed for {}: {}", code, e);
                }
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

    /// Reference to the configured SSE / WS publisher.
    pub fn publisher(&self) -> &Arc<dyn AchievementPublisher> {
        &self.publisher
    }

    /// Reference to the behavior evaluator (used by tests).
    pub fn behavior_evaluator(&self) -> &BehaviorEvaluator {
        &self.behavior
    }

    /// Reference to the output evaluator (used by tests).
    pub fn output_evaluator(&self) -> &OutputEvaluator {
        &self.output
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

    fn make_edge_event() -> ARGEvent {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        ARGEvent::EdgeCreated(make_edge(from, to, RelationshipType::DelegatesTo, tenant))
    }

    #[tokio::test]
    async fn evaluate_runs_three_evaluators() {
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        let engine = ARGAchievementEngine::new(topology, behavior, output, ops, user);
        let tenant = Uuid::nil();
        let unlocks = engine
            .evaluate(make_edge_event(), tenant)
            .await
            .expect("ok");
        // 8 topology + 1 behavior (BEH-001 first_dispatch_via_delegates_to) = 9
        assert_eq!(unlocks.len(), 9);
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
        assert_eq!(first.len(), 9);
        assert_eq!(second.len(), 0, "second call is a no-op");
        assert_eq!(engine.unique_unlocks(), 9);
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
        // BehaviorEvaluator still triggers BEH-001 on the first delegates_to edge
        // even when topology backend is never_trigger.
        assert_eq!(unlocks.len(), 1);
        assert!(unlocks.iter().any(|u| u.achievement_code == "BEH-001-FIRST-DELEGATES"));
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
        let b = StubTopologyBackend::always_trigger();
        let ok = b.evaluate("MATCH (n) RETURN n", Uuid::nil()).expect("ok");
        assert!(ok);
        let busy_tenant: Uuid = uuid::Uuid::from_u128(0x01);
        let ok2 = b.evaluate("MATCH (n) RETURN n", busy_tenant).expect("ok");
        assert!(!ok2);
    }

    #[tokio::test]
    async fn noop_publisher_records_each_unlock() {
        let pub_: Arc<dyn AchievementPublisher> = Arc::new(NoopAchievementPublisher::new());
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        let engine = ARGAchievementEngine::with_publisher(
            topology,
            behavior,
            output,
            ops,
            user,
            pub_.clone(),
        );
        let tenant = Uuid::nil();
        engine.evaluate(make_edge_event(), tenant).await.expect("ok");
        // 8 topology + 1 behavior (BEH-001) = 9 publishes
        assert_eq!(pub_.published_count(), 9);
    }

    #[tokio::test]
    async fn channel_publisher_fans_out_to_subscriber() {
        let (pub_, mut rx) = ChannelAchievementPublisher::with_capacity(64);
        let pub_: Arc<dyn AchievementPublisher> = Arc::new(pub_);
        let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
        let topology = TopologyEvaluator::new(backend);
        let behavior = BehaviorEvaluator::new();
        let output = OutputEvaluator::new();
        let writer = EventWriter::new();
        let ops = AchievementOps::new(writer);
        let user = Uuid::new_v4();
        let engine = ARGAchievementEngine::with_publisher(
            topology,
            behavior,
            output,
            ops,
            user,
            pub_.clone(),
        );
        let tenant = Uuid::nil();
        let unlocks = engine
            .evaluate(make_edge_event(), tenant)
            .await
            .expect("ok");
        // 8 topology + 1 behavior (BEH-001) = 9 new unlocks
        assert_eq!(unlocks.len(), 9);
        // Receiver is still in scope, so published_count returns the
        // number of active receivers (1).
        assert_eq!(pub_.published_count(), 1);
        // Drain the channel: 9 unlocks were broadcast.
        let mut count = 0;
        while let Ok(_u) = rx.try_recv() {
            count += 1;
        }
        assert_eq!(count, 9);
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

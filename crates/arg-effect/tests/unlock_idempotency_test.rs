// SPDX-License-Identifier: MIT OR Apache-2.0
//! `unlock_idempotency_test` — 3 UT (per brief §2.1 B + DD §10.1.3 IDEMP-UT-01..03).
//!
//! 3 tests confirming the (user, code) idempotency layer:
//! - IDEMP-UT-01 first unlock returns the row
//! - IDEMP-UT-02 second unlock with the same (user, code) returns no row
//! - IDEMP-UT-03 different user_id unlocks the same code twice

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use star_arg::models::achievement_unlock::AchievementUnlock;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg::models::event::ARGEvent;
use star_arg::ops::event_writer::EventWriter;
use star_arg::ops::AchievementOps;
use star_arg_effect::achievement_engine::{
    ARGAchievementEngine, AchievementPublisher, BehaviorEvaluator, OutputEvaluator,
    StubTopologyBackend, TopologyBackend, TopologyEvaluator,
};
use star_arg_effect::error::EffectError;
use uuid::Uuid;

fn edge(from: Uuid, to: Uuid, t: RelationshipType, tenant: Uuid) -> Edge {
    let now = Utc::now();
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

struct CountingPublisher {
    count: AtomicUsize,
}

impl CountingPublisher {
    fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl AchievementPublisher for CountingPublisher {
    async fn publish_achievement_unlocked(
        &self,
        _unlock: &AchievementUnlock,
    ) -> Result<(), EffectError> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn published_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

fn make_engine_with_pub(pub_: Arc<dyn AchievementPublisher>) -> ARGAchievementEngine {
    let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
    ARGAchievementEngine::with_publisher(
        TopologyEvaluator::new(backend),
        BehaviorEvaluator::new(),
        OutputEvaluator::new(),
        AchievementOps::new(EventWriter::new()),
        Uuid::new_v4(),
        pub_,
    )
}

#[tokio::test]
async fn idemp_ut_01_first_unlock_returns_row() {
    let pub_: Arc<dyn AchievementPublisher> = Arc::new(CountingPublisher::new());
    let engine = make_engine_with_pub(pub_.clone());
    let tenant = Uuid::nil();
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::DelegatesTo,
        tenant,
    ));
    let unlocks = engine.evaluate(evt, tenant).await.unwrap();
    // 8 topology + 1 behavior (BEH-001) = 9
    assert_eq!(unlocks.len(), 9);
    assert_eq!(pub_.published_count(), 9);
}

#[tokio::test]
async fn idemp_ut_02_second_unlock_same_user_code_is_noop() {
    let pub_: Arc<dyn AchievementPublisher> = Arc::new(CountingPublisher::new());
    let engine = make_engine_with_pub(pub_.clone());
    let tenant = Uuid::nil();
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::DelegatesTo,
        tenant,
    ));
    let first = engine.evaluate(evt.clone(), tenant).await.unwrap();
    let first_count = first.len();
    let second = engine.evaluate(evt, tenant).await.unwrap();
    assert_eq!(second.len(), 0, "second call must be a no-op");
    assert_eq!(pub_.published_count(), first_count);
    assert_eq!(engine.unique_unlocks(), first_count);
}

#[tokio::test]
async fn idemp_ut_03_different_user_id_unlocks_same_code_twice() {
    let pub_: Arc<dyn AchievementPublisher> = Arc::new(CountingPublisher::new());
    let backend: Arc<dyn TopologyBackend> = Arc::new(StubTopologyBackend::always_trigger());
    let engine = ARGAchievementEngine::with_publisher(
        TopologyEvaluator::new(backend),
        BehaviorEvaluator::new(),
        OutputEvaluator::new(),
        AchievementOps::new(EventWriter::new()),
        Uuid::new_v4(),
        pub_.clone(),
    );
    let tenant = Uuid::nil();
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::DelegatesTo,
        tenant,
    ));
    let first = engine.evaluate(evt.clone(), tenant).await.unwrap();
    let first_count = first.len();

    // Use a different user_id via evaluate_for_user
    let second_user = Uuid::new_v4();
    let second = engine
        .evaluate_for_user(second_user, evt, tenant)
        .await
        .unwrap();
    assert_eq!(second.len(), first_count);
    // The 8 topology codes are shared (same tenant + same code) but
    // the AchievementOps idempotency is per (user_id, code), so the
    // second user gets a fresh set of unlocks.
    assert_eq!(pub_.published_count(), first_count * 2);
    assert_eq!(engine.unique_unlocks(), first_count * 2);
}

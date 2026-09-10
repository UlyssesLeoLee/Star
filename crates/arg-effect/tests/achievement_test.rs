// SPDX-License-Identifier: MIT OR Apache-2.0
//! `achievement_test` — 5 UT (per DD §10.1.3 UT-48..UT-52).
//!
//! - UT-48 test_achievement_topology_eval — 8 拓扑成就 Cypher
//! - UT-49 test_achievement_behavior_eval — 7 行为 pattern
//! - UT-50 test_achievement_output_eval   — 5 产出指标
//! - UT-51 test_achievement_unlock_idempotent — 重复 unlock 幂等
//! - UT-52 test_achievement_3_categories — 3 维度正确分类

use std::sync::Arc;

use chrono::Utc;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg::models::event::ARGEvent;
use star_arg::ops::event_writer::EventWriter;
use star_arg::ops::AchievementOps;
use star_arg::query::{all_behavior_patterns, all_output_metrics, all_topology_cyphers};
use star_arg_effect::achievement_engine::topology;
use star_arg_effect::achievement_engine::{
    ARGAchievementEngine, BehaviorEvaluator, OutputEvaluator, StubTopologyBackend,
    TopologyEvaluator,
};
use uuid::Uuid;

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

fn make_edge_event(tenant: Uuid) -> ARGEvent {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    ARGEvent::EdgeCreated(edge(from, to, RelationshipType::DelegatesTo, 0.7, tenant))
}

#[tokio::test]
async fn ut48_achievement_topology_eval_8_cyphers() {
    let backend: Arc<dyn star_arg_effect::achievement_engine::TopologyBackend> =
        Arc::new(StubTopologyBackend::always_trigger());
    let eval = TopologyEvaluator::new(backend);
    let triggered = eval
        .evaluate(&make_edge_event(Uuid::nil()), Uuid::nil())
        .await
        .expect("ok");
    assert_eq!(triggered.len(), 8);
    // Confirm the 8 codes match the canonical order.
    let canonical: Vec<String> = topology::all_codes()
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(triggered, canonical);
}

#[test]
fn ut48b_topology_cypher_count_matches_8() {
    let pairs = all_topology_cyphers();
    assert_eq!(pairs.len(), 8);
    for pair in &pairs {
        assert!(
            !pair.cypher.contains("apoc."),
            "cypher {} uses apoc.coll (forbidden by F-11)",
            pair.code
        );
    }
}

#[tokio::test]
async fn ut49_achievement_behavior_eval_first_edge() {
    // ARG.8 (per brief `arg-08-behavior-output-evaluator.md` §2.1 A.1):
    // BehaviorEvaluator now triggers BEH-001 on the first DELEGATES_TO
    // edge (per pattern `first_dispatch_via_delegates_to`).
    let eval = BehaviorEvaluator::new();
    let triggered = eval
        .evaluate(&make_edge_event(Uuid::nil()), Uuid::nil())
        .await
        .expect("ok");
    assert!(
        triggered.iter().any(|c| c == "BEH-001-FIRST-DELEGATES"),
        "BEH-001 should trigger on first DELEGATES_TO edge, got: {triggered:?}"
    );
    // The query-template layer is in `star_arg::query::behavior`:
    // 7 行为 patterns are already enumerated.
    let patterns = all_behavior_patterns();
    assert_eq!(patterns.len(), 7);
}

#[tokio::test]
async fn ut50_achievement_output_eval_no_trigger_on_first_edge() {
    // ARG.8 (per brief `arg-08-behavior-output-evaluator.md` §2.1 A.2):
    // OutputEvaluator triggers on aggregate thresholds. A single
    // DELEGATES_TO edge doesn't push any OUT metric over its threshold.
    let eval = OutputEvaluator::new();
    let triggered = eval
        .evaluate(&make_edge_event(Uuid::nil()), Uuid::nil())
        .await
        .expect("ok");
    assert!(
        triggered.is_empty(),
        "OUT metrics should not trigger on first edge, got: {triggered:?}"
    );
    let metrics = all_output_metrics();
    assert_eq!(metrics.len(), 5);
}

#[tokio::test]
async fn ut51_achievement_unlock_idempotent() {
    let backend: Arc<dyn star_arg_effect::achievement_engine::TopologyBackend> =
        Arc::new(StubTopologyBackend::always_trigger());
    let topology = TopologyEvaluator::new(backend);
    let behavior = BehaviorEvaluator::new();
    let output = OutputEvaluator::new();
    let ops = AchievementOps::new(EventWriter::new());
    let user = Uuid::new_v4();
    let engine = ARGAchievementEngine::new(topology, behavior, output, ops, user);

    // first call → 8 topology + 1 behavior (BEH-001) = 9 unlocks
    let first = engine
        .evaluate(make_edge_event(Uuid::nil()), Uuid::nil())
        .await
        .expect("ok");
    assert_eq!(first.len(), 9);

    // second call → 0 unlocks (idempotent per (user, code))
    let second = engine
        .evaluate(make_edge_event(Uuid::nil()), Uuid::nil())
        .await
        .expect("ok");
    assert_eq!(second.len(), 0);

    assert_eq!(engine.unique_unlocks(), 9);
}

#[tokio::test]
async fn ut52_achievement_3_categories_classification() {
    use star_arg::models::achievement::{count_by_category, AchievementCategory};

    // 8 拓扑 + 7 行为 + 5 产出 = 20 achievements
    assert_eq!(count_by_category(AchievementCategory::Topology), 8);
    assert_eq!(count_by_category(AchievementCategory::Behavior), 7);
    assert_eq!(count_by_category(AchievementCategory::Output), 5);

    // And the 3 evaluators match the 3 categories 1:1.
    let topology_codes: Vec<String> = topology::all_codes()
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(topology_codes.len(), 8);
    let behavior_codes: Vec<String> = all_behavior_patterns()
        .into_iter()
        .map(|p| p.code)
        .collect();
    assert_eq!(behavior_codes.len(), 7);
    let output_codes: Vec<String> = all_output_metrics().into_iter().map(|m| m.code).collect();
    assert_eq!(output_codes.len(), 5);
}

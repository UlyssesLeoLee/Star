// SPDX-License-Identifier: MIT OR Apache-2.0
//! `behavior_test` — 7 UT (per brief §2.1 B + DD §10.1.3 BEH-UT-01..07).
//!
//! 7 tests, one per BEH pattern:
//! - BEH-UT-01 first_dispatch_via_delegates_to
//! - BEH-UT-02 consults_decision_made_100_times
//! - BEH-UT-03 collaborates_with_parallel_50_times
//! - BEH-UT-04 stand_in_for_takeover_5_times
//! - BEH-UT-05 peer_reviews_one_pass_100_percent
//! - BEH-UT-06 challenges_rebuttal_3_times
//! - BEH-UT-07 shadows_observation_24_hours

use chrono::Utc;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg::models::event::ARGEvent;
use star_arg_effect::achievement_engine::behavior::{
    BEH_001_FIRST_DELEGATES, BEH_002_CONSULTS_100, BEH_003_COLLAB_50, BEH_004_STAND_IN_5,
    BEH_005_PEER_100PCT_10, BEH_006_CHALLENGE_3, BEH_007_SHADOWS_24H, BehaviorEvaluator,
    BehaviorStats, BehaviorThresholds, ALL_BEH_CODES,
};
use uuid::Uuid;

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
fn beh_ut_01_first_dispatch_via_delegates_to() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::DelegatesTo,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    assert!(code.iter().any(|c| c == BEH_001_FIRST_DELEGATES));
}

#[test]
fn beh_ut_02_consults_decision_made_100_times() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    // Pre-seed 99 consults, then 1 more triggers BEH-002.
    let mut stats = BehaviorStats::new();
    stats.consults_total = 99;
    eval.set_stats(stats);
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
fn beh_ut_03_collaborates_with_parallel_50_times() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = BehaviorStats::new();
    stats.collaborates_total = 49;
    eval.set_stats(stats);
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::CollaboratesWith,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    assert!(code.iter().any(|c| c == BEH_003_COLLAB_50));
}

#[test]
fn beh_ut_04_stand_in_for_takeover_5_times() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = BehaviorStats::new();
    stats.stand_in_total = 5;
    eval.set_stats(stats);
    // 6th event with stand_in type.
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::StandInFor,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    assert!(code.iter().any(|c| c == BEH_004_STAND_IN_5));
}

#[test]
fn beh_ut_05_peer_reviews_one_pass_100_percent() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    // Pre-seed 9 peer-reviews, all accepted. The 10th edge event
    // will push peer_reviews_total to 10; with all 10 accepted,
    // BEH-005 (threshold=10) triggers.
    let mut stats = BehaviorStats::new();
    stats.peer_reviews_total = 9;
    stats.peer_reviews_accepted = 9;
    eval.set_stats(stats);
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::PeerReviews,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    // After 10th review, all 10 accepted → BEH-005 triggers.
    assert!(code.iter().any(|c| c == BEH_005_PEER_100PCT_10));
}

#[test]
fn beh_ut_06_challenges_rebuttal_3_times() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = BehaviorStats::new();
    stats.challenges_total = 3;
    eval.set_stats(stats);
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::Challenges,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    assert!(code.iter().any(|c| c == BEH_006_CHALLENGE_3));
}

#[test]
fn beh_ut_07_shadows_observation_24_hours() {
    let eval = BehaviorEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = BehaviorStats::new();
    stats.shadows_total = 24;
    eval.set_stats(stats);
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::Shadows,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    assert!(code.iter().any(|c| c == BEH_007_SHADOWS_24H));
}

#[test]
fn all_7_beh_codes_unique_and_canonical() {
    assert_eq!(ALL_BEH_CODES.len(), 7);
    let mut set: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for c in ALL_BEH_CODES.iter() {
        assert!(set.insert(*c), "duplicate code: {c}");
    }
    // Spot-check: first entry is BEH-001, last is BEH-007.
    assert_eq!(ALL_BEH_CODES[0], BEH_001_FIRST_DELEGATES);
    assert_eq!(ALL_BEH_CODES[6], BEH_007_SHADOWS_24H);
}

#[test]
fn thresholds_default_matches_brief() {
    let t = BehaviorThresholds::default();
    assert_eq!(t.first_dispatch_count, 1);
    assert_eq!(t.consults_100_count, 100);
    assert_eq!(t.collaborates_50_count, 50);
    assert_eq!(t.stand_in_5_count, 5);
    assert_eq!(t.peer_reviews_10_count, 10);
    assert_eq!(t.challenges_3_count, 3);
    assert_eq!(t.shadows_24h_count, 24);
}

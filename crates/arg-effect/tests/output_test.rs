// SPDX-License-Identifier: MIT OR Apache-2.0
//! `output_test` — 5 UT (per brief §2.1 B + DD §10.1.3 OUT-UT-01..05).
//!
//! 5 tests, one per OUT metric:
//! - OUT-UT-01 trusts_skip_verify_save_100k_token
//! - OUT-UT-02 collaborate_save_1h_wall_clock
//! - OUT-UT-03 5_domain_lead_consensus_reached
//! - OUT-UT-04 zero_failure_100_collaborations
//! - OUT-UT-05 achievement_chain_5_in_a_row

use chrono::Utc;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg::models::event::ARGEvent;
use star_arg_effect::achievement_engine::output::{
    ALL_OUT_CODES, OUT_001_TRUSTS_100K_TOKEN, OUT_002_COLLAB_1H, OUT_003_5_LEAD_CONSENSUS,
    OUT_004_ZERO_FAIL_100, OUT_005_ACHIEVEMENT_CHAIN_5, OutputEvaluator, OutputStats,
    OutputThresholds,
};
use uuid::Uuid;

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

fn edge(from: Uuid, to: Uuid, t: RelationshipType, tenant: Uuid) -> Edge {
    edge_with_meta(from, to, t, serde_json::Value::Null, tenant)
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
        .block_on(f)
}

#[test]
fn out_ut_01_trusts_skip_verify_save_100k_token() {
    let eval = OutputEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = OutputStats::new();
    stats.trusts_tokens_saved = 100_000.0;
    eval.set_stats(stats);
    let evt = ARGEvent::TrustScoreChanged {
        agent_id: Uuid::new_v4(),
        before: 0.5,
        after: 0.55,
        delta: 0.05,
    };
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    // 100000 + 0.05 * 200 = 100010 >= 100000
    assert!(code.iter().any(|c| c == OUT_001_TRUSTS_100K_TOKEN));
}

#[test]
fn out_ut_02_collaborate_save_1h_wall_clock() {
    let eval = OutputEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = OutputStats::new();
    stats.collab_wall_clock_seconds = 3500.0;
    eval.set_stats(stats);
    let evt = ARGEvent::EdgeCreated(edge_with_meta(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::CollaboratesWith,
        serde_json::json!({"wall_clock_saved_seconds": 200.0}),
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    // 3500 + 200 = 3700 >= 3600
    assert!(code.iter().any(|c| c == OUT_002_COLLAB_1H));
}

#[test]
fn out_ut_03_5_domain_lead_consensus_reached() {
    let eval = OutputEvaluator::new();
    let t = Uuid::new_v4();
    let mut stats = OutputStats::new();
    stats.consensus_count = 1;
    eval.set_stats(stats);
    let evt = ARGEvent::EdgeCreated(edge(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::DelegatesTo,
        t,
    ));
    let code = block_on(eval.evaluate(&evt, t)).expect("ok");
    assert!(code.iter().any(|c| c == OUT_003_5_LEAD_CONSENSUS));
}

#[test]
fn out_ut_04_zero_failure_100_collaborations() {
    let mut stats = OutputStats::new();
    stats.total_collaborations = 100;
    stats.zero_failure_count = 100;
    let codes = stats.triggered_codes(&OutputThresholds::default());
    assert!(codes.iter().any(|c| c == OUT_004_ZERO_FAIL_100));

    // 100 collab but 99 zero-failure → not triggered.
    let mut stats2 = OutputStats::new();
    stats2.total_collaborations = 100;
    stats2.zero_failure_count = 99;
    let codes2 = stats2.triggered_codes(&OutputThresholds::default());
    assert!(!codes2.iter().any(|c| c == OUT_004_ZERO_FAIL_100));
}

#[test]
fn out_ut_05_achievement_chain_5_in_a_row() {
    let mut stats = OutputStats::new();
    stats.achievement_chain_length = 5;
    let codes = stats.triggered_codes(&OutputThresholds::default());
    assert!(codes.iter().any(|c| c == OUT_005_ACHIEVEMENT_CHAIN_5));

    // Reset chain → not triggered.
    let mut stats2 = OutputStats::new();
    stats2.achievement_chain_length = 4;
    let codes2 = stats2.triggered_codes(&OutputThresholds::default());
    assert!(!codes2.iter().any(|c| c == OUT_005_ACHIEVEMENT_CHAIN_5));
}

#[test]
fn all_5_out_codes_unique_and_canonical() {
    assert_eq!(ALL_OUT_CODES.len(), 5);
    let mut set: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for c in ALL_OUT_CODES.iter() {
        assert!(set.insert(*c), "duplicate code: {c}");
    }
    assert_eq!(ALL_OUT_CODES[0], OUT_001_TRUSTS_100K_TOKEN);
    assert_eq!(ALL_OUT_CODES[4], OUT_005_ACHIEVEMENT_CHAIN_5);
}

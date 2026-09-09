// SPDX-License-Identifier: MIT OR Apache-2.0
//! `trust_test` — 4 UT (per DD §10.1.3 UT-41..UT-44).
//!
//! - UT-41 test_trust_engine_record_success — score +0.01
//! - UT-42 test_trust_engine_record_failure — score -0.05
//! - UT-43 test_trust_engine_skip_verify   — trust ≥ 0.8 + trusts 边 weight ≥ 0.7
//! - UT-44 test_trust_engine_no_skip        — trust < 0.8

use std::sync::Arc;

use chrono::Utc;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg_effect::dispatch_router::InMemoryEdgeStore;
use star_arg_effect::trust_engine::{ARGTrustEngine, TrustStore};
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

#[test]
fn ut41_trust_engine_record_success() {
    let store = TrustStore::new();
    let mut engine = ARGTrustEngine::new(TrustStore::new(), Arc::new(InMemoryEdgeStore::default()));
    let agent = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let s = engine.record_success(agent, tenant);
    // first call starts from 0.5 → +0.01 → 0.51
    assert!((s - 0.51).abs() < 1e-6, "expected 0.51, got {s}");
    let _ = store; // silence unused warning
}

#[test]
fn ut42_trust_engine_record_failure() {
    let mut engine = ARGTrustEngine::new(TrustStore::new(), Arc::new(InMemoryEdgeStore::default()));
    let agent = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let s = engine.record_failure(agent, tenant);
    // first call starts from 0.5 → -0.05 → 0.45
    assert!((s - 0.45).abs() < 1e-6, "expected 0.45, got {s}");
}

#[test]
fn ut43_trust_engine_skip_verify() {
    let tenant = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let granter = Uuid::new_v4();

    let mut trust = TrustStore::new();
    trust.set(agent, tenant, 0.85);
    let mut edges = InMemoryEdgeStore::default();
    edges.add(edge(granter, agent, RelationshipType::Trusts, 0.8, tenant));

    let engine = ARGTrustEngine::new(trust, Arc::new(edges));
    assert!(engine.should_skip_verify(agent, tenant));
}

#[test]
fn ut44_trust_engine_no_skip_when_score_low() {
    let tenant = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let granter = Uuid::new_v4();

    let mut trust = TrustStore::new();
    trust.set(agent, tenant, 0.7);
    let mut edges = InMemoryEdgeStore::default();
    edges.add(edge(granter, agent, RelationshipType::Trusts, 0.9, tenant));

    let engine = ARGTrustEngine::new(trust, Arc::new(edges));
    assert!(!engine.should_skip_verify(agent, tenant));
}

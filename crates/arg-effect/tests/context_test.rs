// SPDX-License-Identifier: MIT OR Apache-2.0
//! `context_test` — 3 UT (per DD §10.1.3 UT-38..UT-40).
//!
//! - UT-38 test_context_inject_mentors — 拉 mentor 历史
//! - UT-39 test_context_inject_shadows — 拉被观察 event
//! - UT-40 test_context_inject_empty   — 无 mentor / shadow 不报错

use std::sync::Arc;

use chrono::Utc;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg_effect::context_injector::{
    ARGContextInjector, DecisionRow, EventRow, InMemoryContextProvider,
};
use star_arg_effect::dispatch_router::InMemoryEdgeStore;
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
fn ut38_context_inject_mentors() {
    let tenant = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let mentor = Uuid::new_v4();

    let mut store = InMemoryEdgeStore::default();
    store.add(edge(mentor, agent, RelationshipType::Mentors, 0.7, tenant));

    let mut provider = InMemoryContextProvider::new();
    provider.seed_decisions(
        mentor,
        vec![DecisionRow::new("choose async runtime", Utc::now())],
    );

    let injector = ARGContextInjector::new(Arc::new(store), Arc::new(provider));
    let out = injector
        .inject_context(agent, "you are", tenant)
        .expect("ok");
    assert!(out.contains("Mentor "));
    assert!(out.contains("choose async runtime"));
    assert!(out.contains("--- ARG Context ---"));
}

#[test]
fn ut39_context_inject_shadows() {
    let tenant = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let target = Uuid::new_v4();

    let mut store = InMemoryEdgeStore::default();
    store.add(edge(target, agent, RelationshipType::Shadows, 0.6, tenant));

    let mut provider = InMemoryContextProvider::new();
    provider.seed_events(target, vec![EventRow::new("rebuild schema", Utc::now())]);

    let injector = ARGContextInjector::new(Arc::new(store), Arc::new(provider));
    let out = injector
        .inject_context(agent, "you are", tenant)
        .expect("ok");
    assert!(out.contains("Shadow "));
    assert!(out.contains("rebuild schema"));
}

#[test]
fn ut40_context_inject_empty() {
    let store = Arc::new(InMemoryEdgeStore::default());
    let provider = Arc::new(InMemoryContextProvider::new());
    let injector = ARGContextInjector::new(store, provider);
    let out = injector
        .inject_context(Uuid::new_v4(), "base", Uuid::new_v4())
        .expect("ok");
    // No edges → no context block appended; base_prompt is returned verbatim.
    assert_eq!(out, "base");
    assert!(!out.contains("--- ARG Context ---"));
}

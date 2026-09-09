// SPDX-License-Identifier: MIT OR Apache-2.0
//! `dispatch_test` — 3 UT (per DD §10.1.3 UT-35..UT-37).
//!
//! - UT-35 test_dispatch_router_delegates   — 查 outgoing delegates_to
//! - UT-36 test_dispatch_router_stand_in    — fallback 触发
//! - UT-37 test_dispatch_router_collaborators — 並行 + merge

use std::sync::Arc;

use chrono::Utc;
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg_effect::dispatch_router::ARGDispatchRouter;
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

#[test]
fn ut35_dispatch_router_delegates() {
    let tenant = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();

    let mut store = InMemoryEdgeStore::default();
    store.add(edge(a, b, RelationshipType::DelegatesTo, 0.7, tenant));
    store.add(edge(a, c, RelationshipType::DelegatesTo, 0.6, tenant));

    let router = ARGDispatchRouter::new(Arc::new(store));
    let route = router.route(a, tenant);
    assert_eq!(route.delegates.len(), 2);
    assert!(route.delegates.contains(&b));
    assert!(route.delegates.contains(&c));
    assert!(route.stand_ins.is_empty());
    assert!(route.collaborators.is_empty());
}

#[test]
fn ut36_dispatch_router_stand_in() {
    let tenant = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let backup = Uuid::new_v4();

    let mut store = InMemoryEdgeStore::default();
    store.add(edge(
        backup,
        agent,
        RelationshipType::StandInFor,
        0.8,
        tenant,
    ));

    let router = ARGDispatchRouter::new(Arc::new(store));
    let route = router.route(agent, tenant);
    assert_eq!(route.stand_ins, vec![backup]);
    assert!(route.delegates.is_empty());
}

#[test]
fn ut37_dispatch_router_collaborators() {
    let tenant = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();

    let mut store = InMemoryEdgeStore::default();
    // Two undirected edges touching `a`; the router should dedup and
    // emit a single collaborator (b).
    store.add(edge(a, b, RelationshipType::CollaboratesWith, 0.5, tenant));
    store.add(edge(b, c, RelationshipType::CollaboratesWith, 0.4, tenant));

    let router = ARGDispatchRouter::new(Arc::new(store));
    let route = router.route(a, tenant);
    assert_eq!(route.collaborators, vec![b]);
    // c is not a collaborator of a (not connected to a).
    assert!(!route.collaborators.contains(&c));
}

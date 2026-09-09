// SPDX-License-Identifier: MIT OR Apache-2.0
//! `langgraph_test` — UT-33/34 (per DD §10.1.2) — PyO3 binding + 5 Reducer.
//!
//! The real PyO3 binding lives in `crates/arg-effect` (ARG.3) and is
//! stubbed in ARG.2. The tests here validate the 5 Reducer semantics
//! against an in-process [`LangGraphStateStore`] — these are the same
//! Reducers the Python `TopAgentState` applies (per DD §5.1) and act
//! as the contract test for the upcoming PyO3 binding.

use std::collections::HashMap;

use star_arg::models::agent::{Agent, AgentArchetype};
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg_bridge::{
    protocol::{ArgDispatchRoute, ArgTrustScoreUpdate, BridgeEnvelope, BridgeEnvelopeKind},
    LangGraphStateUpdater, ReducerKind,
};
use uuid::Uuid;

fn make_agent(name: &str) -> Agent {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    Agent::new(name.into(), AgentArchetype::Sa01, tenant, author)
}

fn make_edge(from: Uuid, to: Uuid) -> Edge {
    let now = chrono::Utc::now();
    Edge {
        id: Uuid::new_v4(),
        from_agent: from,
        to_agent: to,
        edge_type: RelationshipType::DelegatesTo,
        weight: 0.5,
        direction: EdgeDirection::Directed,
        archived: false,
        metadata: serde_json::Value::Null,
        tenant_id: Uuid::new_v4(),
        created_at: now,
        updated_at: now,
        version: 1,
        created_by: Uuid::new_v4(),
    }
}

#[tokio::test]
async fn ut33_langgraph_state_update_5_reducer_dispatch() {
    // 5 Reducer kind codes must be unique (per DD §5.1).
    let kinds = ReducerKind::all();
    assert_eq!(kinds.len(), 5);
    let mut labels: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
    labels.sort();
    labels.dedup();
    assert_eq!(labels.len(), 5);

    let updater = LangGraphStateUpdater::new();
    // Dispatch a single trust-score envelope and confirm the store
    // mirrors it.
    let agent_id = Uuid::new_v4();
    let envelope = BridgeEnvelope::new(
        Uuid::new_v4(),
        BridgeEnvelopeKind::TrustScoreUpdate(ArgTrustScoreUpdate {
            agent_id,
            old_score: 0.4,
            new_score: 0.7,
            delta_reason: "success".to_string(),
            trigger_sa: "SA_03".to_string(),
            created_at: chrono::Utc::now(),
        }),
        0,
    );
    updater
        .update_state(&envelope)
        .await
        .expect("update_state ok");

    let snap = updater.snapshot().await;
    assert_eq!(snap.arg_trust_scores.get(&agent_id).copied(), Some(0.7));
}

#[tokio::test]
async fn ut34_langgraph_5_reducer_semantics() {
    let updater = LangGraphStateUpdater::new();

    // merge_arg_agents: 2 versions of the same id — higher wins.
    let a1 = make_agent("a");
    let mut a2 = a1.clone();
    a2.name = "a2".into();
    a2.version = 5;
    updater
        .reducer_merge_arg_agents(vec![a1.clone(), a2.clone()])
        .await;
    let got = updater.get_agent(a1.id).await.expect("agent present");
    assert_eq!(got.version, 5);
    assert_eq!(got.name, "a2");

    // merge_arg_edges: 2 versions — higher wins.
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let e1 = make_edge(from, to);
    let mut e2 = e1.clone();
    e2.weight = 0.9;
    e2.version = 7;
    updater.reducer_merge_arg_edges(vec![e1, e2.clone()]).await;
    let got = updater.get_edge(e2.id).await.expect("edge present");
    assert_eq!(got.version, 7);
    assert!((got.weight - 0.9).abs() < 1e-6);

    // merge_trust_scores: max value wins.
    let k1 = Uuid::new_v4();
    let mut ts = HashMap::new();
    ts.insert(k1, 0.3);
    updater.reducer_merge_trust_scores(&ts).await;
    ts.insert(k1, 0.6);
    updater.reducer_merge_trust_scores(&ts).await;
    let snap = updater.snapshot().await;
    assert_eq!(snap.arg_trust_scores.get(&k1).copied(), Some(0.6));

    // merge_dispatch: list union with dedup.
    let k = Uuid::new_v4();
    let t1 = Uuid::new_v4();
    let t2 = Uuid::new_v4();
    let mut dispatch = HashMap::new();
    dispatch.insert(k, vec![t1]);
    updater.reducer_merge_dispatch(&dispatch).await;
    dispatch.insert(k, vec![t1, t2]);
    updater.reducer_merge_dispatch(&dispatch).await;
    let snap = updater.snapshot().await;
    let list = snap.arg_dispatch_overrides.get(&k).expect("present");
    assert_eq!(list.len(), 2);
    assert!(list.contains(&t1));
    assert!(list.contains(&t2));

    // add: append-only.
    updater
        .reducer_add(vec!["ACH-001".into(), "ACH-002".into()])
        .await;
    updater.reducer_add(vec!["ACH-003".into()]).await;
    let snap = updater.snapshot().await;
    assert_eq!(snap.arg_achievements_unlocked.len(), 3);
}

#[tokio::test]
async fn ut34b_langgraph_dispatch_route_envelope() {
    // Confirm a BridgeEnvelope of kind DispatchRoute is applied to the
    // in-process store.
    let updater = LangGraphStateUpdater::new();
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let envelope = BridgeEnvelope::new(
        Uuid::new_v4(),
        BridgeEnvelopeKind::DispatchRoute(ArgDispatchRoute {
            route_id: Uuid::new_v4(),
            from_agent_id: from,
            to_agent_id: to,
            sa_type: "SA_03".into(),
            weight: 0.8,
            override_reason: "delegate".into(),
            created_at: chrono::Utc::now(),
        }),
        0,
    );
    updater.update_state(&envelope).await.expect("ok");
    let snap = updater.snapshot().await;
    let list = snap.arg_dispatch_overrides.get(&from).expect("present");
    assert!(list.contains(&to));
}

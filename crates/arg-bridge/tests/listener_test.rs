// SPDX-License-Identifier: MIT OR Apache-2.0
//! `listener_test` — 3 UT (per DD §10.1.2 UT-25..UT-27).
//!
//! Covers [`MemgraphEventListener`]:
//! - UT-25 test_memgraph_listener_connect   — Bolt subscription 建立
//! - UT-26 test_memgraph_listener_event     — edge.changed event 解析
//! - UT-27 test_memgraph_listener_reconnect — 断线重连

use std::sync::Arc;
use std::time::Duration;

use star_arg::client::MemgraphClient;
use star_arg::models::agent::{Agent, AgentArchetype};
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg::models::event::ARGEvent;
use star_arg_bridge::{BridgeEnvelope, MemgraphEventListener};
use uuid::Uuid;

fn make_client() -> Arc<MemgraphClient> {
    Arc::new(MemgraphClient::new(
        "bolt://localhost:7687".into(),
        "test".into(),
        "test".into(),
        1,
    ))
}

fn make_edge() -> Edge {
    let now = chrono::Utc::now();
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    Edge {
        id: Uuid::new_v4(),
        from_agent: from,
        to_agent: to,
        edge_type: RelationshipType::DelegatesTo,
        weight: 0.7,
        direction: EdgeDirection::Directed,
        archived: false,
        metadata: serde_json::Value::Null,
        tenant_id: tenant,
        created_at: now,
        updated_at: now,
        version: 1,
        created_by: author,
    }
}

fn make_agent() -> Agent {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    Agent::new("listener-test".into(), AgentArchetype::Sa01, tenant, author)
}

#[tokio::test]
async fn ut25_memgraph_listener_connect() {
    // G-1 stub path: the listener must come up cleanly without
    // requiring a real Bolt pool.
    let listener = MemgraphEventListener::new(make_client());
    let res = listener.start().await;
    assert!(res.is_ok(), "start() should succeed against the G-1 stub");
    assert_eq!(listener.channel_capacity(), 256);
    assert_eq!(listener.receiver_count(), 0);
}

#[tokio::test]
async fn ut26_memgraph_listener_event() {
    // Build an `EdgeCreated` event and verify the listener emits a
    // `BridgeEnvelope` with the right payload + `kind` code.
    let listener = MemgraphEventListener::new(make_client());
    let edge = make_edge();
    let event = ARGEvent::EdgeCreated(edge.clone());
    let envelope = listener
        .publish(&event)
        .expect("publish should succeed for EdgeCreated");
    assert_eq!(envelope.kind_str(), "arg_edge_changed");
    match &envelope.kind {
        star_arg_bridge::BridgeEnvelopeKind::EdgeChanged(e) => {
            assert_eq!(e.edge_id, edge.id);
            assert_eq!(e.from_agent_id, edge.from_agent);
            assert_eq!(e.to_agent_id, edge.to_agent);
            assert_eq!(e.relationship_type, "DELEGATES_TO");
            assert!((e.weight - 0.7).abs() < 1e-6);
        }
        _ => panic!("expected EdgeChanged kind"),
    }

    // The agent.flushed event currently returns an InternalError because
    // the ARG.1 → ARG.2 mapping only covers 4 of the 9 ARGEvent variants.
    let agent = make_agent();
    let other = listener.publish(&ARGEvent::AgentCreated(agent));
    assert!(other.is_err(), "unmapped ARGEvent should error out");
}

#[tokio::test]
async fn ut27_memgraph_listener_reconnect() {
    // The listener must allow multiple `start` calls and a fresh
    // subscriber to observe later envelopes (simulating a Bolt
    // reconnect).
    let listener = Arc::new(MemgraphEventListener::new(make_client()));
    listener.start().await.expect("start 1");
    listener.start().await.expect("start 2 (reconnect)");
    assert!(listener.start().await.is_ok());

    let mut rx = listener.subscribe();
    // Spawn a publisher that emits after a short delay so the receiver
    // is registered first.
    let listener_pub = Arc::clone(&listener);
    let _ = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let event = ARGEvent::EdgeCreated(make_edge());
        let _ = listener_pub.publish(&event);
    });

    let env = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("no timeout")
        .expect("receiver should observe the envelope");
    let _: BridgeEnvelope = env;
}

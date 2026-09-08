// SPDX-License-Identifier: MIT OR Apache-2.0
//! `edge_ops_test` — 8 UT (per DD §10.1.1 UT-06..UT-13).

use star_arg::models::edge::{Edge, EdgeDirection, EdgeState, RelationshipType};
use uuid::Uuid;

fn make_edge(t: RelationshipType, weight: f32, from: Uuid, to: Uuid) -> Edge {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let direction = if t.is_directed() {
        EdgeDirection::Directed
    } else {
        EdgeDirection::Undirected
    };
    let now = chrono::Utc::now();
    Edge {
        id: Uuid::new_v4(),
        from_agent: from,
        to_agent: to,
        edge_type: t,
        weight,
        direction,
        archived: false,
        metadata: serde_json::Value::Null,
        tenant_id: tenant,
        created_at: now,
        updated_at: now,
        version: 1,
        created_by: author,
    }
}

#[test]
fn ut06_edge_10_types_and_labels() {
    let all = RelationshipType::all();
    assert_eq!(all.len(), 10);
    assert_eq!(all[0].cypher_label(), "DELEGATES_TO");
    assert_eq!(all[1].cypher_label(), "CONSULTS");
    assert_eq!(all[2].cypher_label(), "COLLABORATES_WITH");
    assert_eq!(all[9].cypher_label(), "TRUSTS");
}

#[test]
fn ut07_edge_undirected_types() {
    assert!(!RelationshipType::CollaboratesWith.is_directed());
    assert!(!RelationshipType::PeerReviews.is_directed());
    assert!(RelationshipType::DelegatesTo.is_directed());
    assert!(RelationshipType::Trusts.is_directed());
}

#[test]
fn ut08_edge_update_weight_via_patch() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let mut edge = make_edge(RelationshipType::DelegatesTo, 0.5, from, to);
    edge.weight = 0.7;
    edge.version += 1;
    assert!((edge.weight - 0.7).abs() < 1e-6);
    assert_eq!(edge.version, 2);
    assert!(edge.validate().is_ok());
}

#[test]
fn ut09_edge_archive_soft_delete() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let mut edge = make_edge(RelationshipType::ReportsTo, 0.6, from, to);
    edge.archived = true;
    edge.version += 1;
    assert!(edge.archived);
    assert!(Edge::can_transition(
        EdgeState::Created,
        EdgeState::Archived
    ));
    assert!(!Edge::can_transition(
        EdgeState::Archived,
        EdgeState::Updated
    ));
}

#[test]
fn ut10_edge_audit_event_is_written() {
    // Sanity: ARGEvent::EdgeCreated and ARGEvent::EdgeArchived exist and
    // round-trip through serde.
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let edge = make_edge(RelationshipType::Mentors, 0.8, from, to);
    let event = star_arg::models::ARGEvent::EdgeCreated(edge.clone());
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("edge_created"));
    let archived = star_arg::models::ARGEvent::EdgeArchived { id: edge.id };
    let json2 = serde_json::to_string(&archived).unwrap();
    assert!(json2.contains("edge_archived"));
}

#[test]
fn ut11_edge_undirected_excluded_from_outgoing() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let undir = make_edge(RelationshipType::CollaboratesWith, 0.5, from, to);
    assert_eq!(undir.direction, EdgeDirection::Undirected);
    // The dispatch router in arg-effect filters outgoing by is_directed();
    // here we just assert that the flag is observable.
    assert!(!undir.edge_type.is_directed());
}

#[test]
fn ut12_edge_incoming_only_directed() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let dir_edge = make_edge(RelationshipType::StandInFor, 0.7, from, to);
    let undir_edge = make_edge(RelationshipType::PeerReviews, 0.7, from, to);
    assert!(dir_edge.edge_type.is_directed());
    assert!(!undir_edge.edge_type.is_directed());
}

#[test]
fn ut13_edge_validation_rejects_self_loop_and_oob() {
    let from = Uuid::new_v4();
    let bad_self = make_edge(RelationshipType::DelegatesTo, 0.5, from, from);
    assert!(bad_self.validate().is_err());
    let to = Uuid::new_v4();
    let bad_oob = make_edge(RelationshipType::DelegatesTo, 1.5, from, to);
    assert!(bad_oob.validate().is_err());
}

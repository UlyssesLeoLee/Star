// SPDX-License-Identifier: MIT OR Apache-2.0
//! `event_test` — 2 UT (ARGEvent 9 variants, per DD §3.2.5).

use star_arg::models::agent::{Agent, AgentArchetype, AgentStatus, Domain};
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg::models::event::ARGEvent;
use uuid::Uuid;

fn make_agent() -> Agent {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    Agent::new("evt".into(), AgentArchetype::Sa01, tenant, author)
}

fn make_edge() -> Edge {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let now = chrono::Utc::now();
    Edge {
        id: Uuid::new_v4(),
        from_agent: Uuid::new_v4(),
        to_agent: Uuid::new_v4(),
        edge_type: RelationshipType::DelegatesTo,
        weight: 0.5,
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

#[test]
fn ut31_arg_event_9_variants_kind_codes() {
    let agent = make_agent();
    let edge = make_edge();
    let template_instance = star_arg::models::template_instance::TemplateInstance::new(
        star_arg::models::template::TemplateId::HubAndSpoke,
        "inst".into(),
        (0..5).map(|_| Uuid::new_v4()).collect(),
        vec![],
        agent.tenant_id,
        agent.created_by,
    );
    let unlock = star_arg::models::achievement_unlock::AchievementUnlock::new(
        "TOP-001".into(),
        Uuid::new_v4(),
        vec![],
        serde_json::json!({}),
        agent.tenant_id,
    );
    let cases = vec![
        ARGEvent::AgentCreated(agent.clone()),
        ARGEvent::AgentUpdated {
            id: agent.id,
            before: agent.clone(),
            after: agent.clone(),
        },
        ARGEvent::AgentArchived { id: agent.id },
        ARGEvent::EdgeCreated(edge.clone()),
        ARGEvent::EdgeUpdated {
            id: edge.id,
            before: edge.clone(),
            after: edge.clone(),
        },
        ARGEvent::EdgeArchived { id: edge.id },
        ARGEvent::TemplateInstantiated(template_instance),
        ARGEvent::TrustScoreChanged {
            agent_id: agent.id,
            before: 0.5,
            after: 0.51,
            delta: 0.01,
        },
        ARGEvent::AchievementUnlocked(unlock),
    ];
    assert_eq!(cases.len(), 9);
    let mut kinds: Vec<String> = cases.iter().map(|e| e.kind().to_string()).collect();
    kinds.sort();
    kinds.dedup();
    assert_eq!(kinds.len(), 9);
}

#[test]
fn ut32_arg_event_serde_round_trip() {
    let agent = make_agent();
    let event = ARGEvent::AgentCreated(agent.clone());
    let json = serde_json::to_string(&event).unwrap();
    let back: ARGEvent = serde_json::from_str(&json).unwrap();
    match back {
        ARGEvent::AgentCreated(a) => {
            assert_eq!(a.id, agent.id);
            assert_eq!(a.name, agent.name);
            assert_eq!(a.archetype, agent.archetype);
            assert_eq!(a.domain, agent.domain);
            assert_eq!(a.status, AgentStatus::Active);
        }
        _ => panic!("expected AgentCreated"),
    }
    // Touch the unused import in case the compiler complains (Domain is
    // referenced by the assertion path above via `a.domain`).
    let _ = std::any::type_name::<Domain>();
}

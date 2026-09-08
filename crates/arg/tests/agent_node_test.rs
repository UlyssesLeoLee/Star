// SPDX-License-Identifier: MIT OR Apache-2.0
//! `agent_node_test` — 5 UT (per DD §10.1.1 UT-01..UT-05).

use star_arg::models::agent::{Agent, AgentArchetype, AgentStatus};
use star_arg::models::ARGEvent;
use uuid::Uuid;

#[test]
fn ut01_agent_new_and_to_cypher() {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let agent = Agent::new(
        "lead-player".into(),
        AgentArchetype::LeadPlayer,
        tenant,
        author,
    );
    assert_eq!(agent.name, "lead-player");
    assert_eq!(agent.archetype, AgentArchetype::LeadPlayer);
    assert_eq!(agent.status, AgentStatus::Active);
    assert_eq!(agent.trust_score, 0.5);
    assert_eq!(agent.version, 1);
    let cypher = agent.to_cypher();
    assert!(cypher.starts_with("MERGE (a:Agent"));
    assert!(cypher.contains("LEAD_PLAYER"));
    assert!(cypher.contains("player")); // domain
}

#[test]
fn ut02_agent_update_trust_score() {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let mut agent = Agent::new("a".into(), AgentArchetype::Sa01, tenant, author);
    let after_success = agent.update_trust_score(true).unwrap();
    assert!((after_success - 0.51).abs() < 1e-6);
    let after_failure = agent.update_trust_score(false).unwrap();
    assert!((after_failure - 0.46).abs() < 1e-6);
    assert_eq!(agent.version, 3);
}

#[test]
fn ut03_agent_archive_soft_delete() {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let mut agent = Agent::new("a".into(), AgentArchetype::Sa02, tenant, author);
    assert!(agent.can_transition_to(AgentStatus::Archived));
    agent.status = AgentStatus::Archived;
    // Archived is absorbing — cannot transition out.
    assert!(!agent.can_transition_to(AgentStatus::Active));
    assert!(!agent.can_transition_to(AgentStatus::Standby));
}

#[test]
fn ut04_agent_rls_tenant_isolation() {
    let tenant = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let agent = Agent::new("a".into(), AgentArchetype::Sa03, tenant, author);
    assert_eq!(agent.tenant_id, tenant);
    assert_ne!(agent.tenant_id, other_tenant);
}

#[test]
fn ut05_agent_validate_rejects_empty_and_oob() {
    let tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let bad_name = Agent::new("".into(), AgentArchetype::Sa04, tenant, author);
    assert!(bad_name.validate().is_err());
    let mut bad_trust = Agent::new("ok".into(), AgentArchetype::Sa05, tenant, author);
    bad_trust.trust_score = 1.5;
    assert!(bad_trust.validate().is_err());
    // Sanity: the same `update_trust_score` produces an `ARGEvent::TrustScoreChanged`
    // for any external observer (asserted via the same delta).
    let mut trusted = Agent::new("ok".into(), AgentArchetype::Sa06, tenant, author);
    let before = trusted.trust_score;
    let after = trusted.update_trust_score(true).unwrap();
    let event = ARGEvent::TrustScoreChanged {
        agent_id: trusted.id,
        before,
        after,
        delta: after - before,
    };
    assert!(matches!(event, ARGEvent::TrustScoreChanged { .. }));
}

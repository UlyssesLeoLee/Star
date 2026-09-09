// SPDX-License-Identifier: MIT OR Apache-2.0
//! `output_test` — 3 UT (per DD §10.1.3 UT-45..UT-47).
//!
//! - UT-45 test_output_challenge_round — 双向论证 accept
//! - UT-46 test_output_challenge_reject — reject 触发 escalate
//! - UT-47 test_output_peer_review      — 双向 review ≥ 0.8

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use star_arg::error::ARGError;
use star_arg::llm::{LLMClient, LLMResponse, MockLLMClient};
use star_arg::models::decision::{Decision, DecisionType, Output, Verdict};
use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
use star_arg_effect::dispatch_router::InMemoryEdgeStore;
use star_arg_effect::output_evaluator::ARGOutputEvaluator;
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

fn decision() -> Decision {
    Decision {
        decision_type: DecisionType::Architectural,
        description: "use axum 0.8".into(),
        context: json!({}),
        tenant_id: Uuid::new_v4(),
    }
}

fn output() -> Output {
    Output {
        output_type: "code.review".into(),
        content: json!({"pr": 1}),
        agent_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
    }
}

#[tokio::test]
async fn ut45_output_challenge_round_accept() {
    let tenant = Uuid::new_v4();
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let mut store = InMemoryEdgeStore::default();
    store.add(edge(from, to, RelationshipType::Challenges, 0.8, tenant));
    let llm: Arc<dyn LLMClient> = Arc::new(MockLLMClient::new());
    let eval = ARGOutputEvaluator::new(Arc::new(store), llm);
    let v = eval
        .challenge_round(from, to, decision(), tenant)
        .await
        .expect("ok");
    // Mock returns verdict=None → we default to Accept (per ARG.3 守门 #23).
    assert_eq!(v.verdict, Verdict::Accept);
    assert!(v.is_accepted());
    assert!(v.escalation.is_none());
}

#[tokio::test]
async fn ut46_output_challenge_reject_escalate() {
    let tenant = Uuid::new_v4();
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let mut store = InMemoryEdgeStore::default();
    store.add(edge(from, to, RelationshipType::Challenges, 0.5, tenant));

    // Custom mock that returns Verdict::Escalate on the second
    // (evaluate) call so we exercise the escalation-metadata path.
    struct EscalateMock;
    #[async_trait]
    impl LLMClient for EscalateMock {
        async fn call(
            &self,
            _prompt: &str,
            _input: &serde_json::Value,
        ) -> Result<LLMResponse, ARGError> {
            Ok(LLMResponse::new(
                "needs review".into(),
                Some(Verdict::Escalate),
                Some(0.42),
                0,
            ))
        }
    }

    let llm: Arc<dyn LLMClient> = Arc::new(EscalateMock);
    let eval = ARGOutputEvaluator::new(Arc::new(store), llm);
    let v = eval
        .challenge_round(from, to, decision(), tenant)
        .await
        .expect("ok");
    assert_eq!(v.verdict, Verdict::Escalate);
    assert!(!v.is_accepted());
    assert!(v.escalation.is_some());
    assert_eq!(v.escalation.as_ref().unwrap().escalation_target, from);
}

#[tokio::test]
async fn ut47_output_peer_review_above_threshold() {
    let tenant = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let mut store = InMemoryEdgeStore::default();
    store.add(edge(a, b, RelationshipType::PeerReviews, 0.5, tenant));

    // Custom mock returning score=0.95 on both calls.
    struct HighScoreMock;
    #[async_trait]
    impl LLMClient for HighScoreMock {
        async fn call(
            &self,
            _prompt: &str,
            _input: &serde_json::Value,
        ) -> Result<LLMResponse, ARGError> {
            Ok(LLMResponse::new("ok".into(), None, Some(0.95), 0))
        }
    }

    let llm: Arc<dyn LLMClient> = Arc::new(HighScoreMock);
    let eval = ARGOutputEvaluator::new(Arc::new(store), llm);
    let v = eval.peer_review(a, b, output(), tenant).await.expect("ok");
    assert!(v.accepted());
    assert!((v.score() - 0.95).abs() < 1e-6);
}

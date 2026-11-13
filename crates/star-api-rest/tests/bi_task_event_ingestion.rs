// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=01440e8234a02f2f397f771d4ea1916e19abd574e785f31cb63b8ed6efd87704
MERGE (self:File {path:"crates/star-api-rest/tests/bi_task_event_ingestion.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/tests/bi_task_event_ingestion.rs::bi_task_event_ingestion_tests",kind:"module"})
MERGE (implementation:File {path:"crates/star-api-rest/src/bi_task_event_ingestion.rs"})
MERGE (opt_in_test:Symbol {id:"crates/star-api-rest/tests/bi_task_event_ingestion.rs::readiness_reports_opt_in_disabled_state",kind:"test"})
MERGE (blocked_test:Symbol {id:"crates/star-api-rest/tests/bi_task_event_ingestion.rs::readiness_reports_incomplete_configuration_state",kind:"test"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:IMPORTS]->(implementation)
MERGE (module)-[:DEFINES]->(opt_in_test)
MERGE (module)-[:DEFINES]->(blocked_test)
MERGE (opt_in_test)-[:TESTS]->(:Endpoint {id:"/readyz/bi-task-event-ingestion",kind:"http_readiness"})
MERGE (blocked_test)-[:TESTS]->(:Endpoint {id:"/readyz/bi-task-event-ingestion",kind:"http_readiness"})
@endcypher
*/
//! Focused fixture checks for the opt-in and readiness boundary.

#[path = "../src/bi_task_event_ingestion.rs"]
#[allow(dead_code)] // This integration target exercises readiness, not the worker lifecycle.
mod bi_task_event_ingestion;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use bi_task_event_ingestion::{BiTaskEventIngestion, READINESS_PATH};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn readiness_reports_opt_in_disabled_state() {
    let ingestion = BiTaskEventIngestion::disabled("not_opted_in");
    let response = ingestion
        .readiness_router()
        .oneshot(
            Request::builder()
                .uri(READINESS_PATH)
                .body(Body::empty())
                .expect("valid readiness request"),
        )
        .await
        .expect("readiness handler responds");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("bounded readiness response");
    let payload: Value = serde_json::from_slice(&body).expect("valid readiness JSON");
    assert_eq!(payload["component"], "bi-task-event-ingestion");
    assert_eq!(payload["status"], "disabled");
    assert_eq!(payload["reason"], "not_opted_in");
}

#[tokio::test]
async fn readiness_reports_incomplete_configuration_state() {
    let ingestion = BiTaskEventIngestion::disabled("incomplete_configuration");
    let response = ingestion
        .readiness_router()
        .oneshot(
            Request::builder()
                .uri(READINESS_PATH)
                .body(Body::empty())
                .expect("valid readiness request"),
        )
        .await
        .expect("readiness handler responds");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("bounded readiness response");
    let payload: Value = serde_json::from_slice(&body).expect("valid readiness JSON");
    assert_eq!(payload["status"], "disabled");
    assert_eq!(payload["reason"], "incomplete_configuration");
}

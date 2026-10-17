// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=a11e5cde24d76aef51d9c4ee862a93a196544a79cf2bee90eeaadeda014daee6
MERGE (self:File {path:"crates/star-api-rest/src/group_api/bi_analytics.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::bi_analytics",kind:"module"})
MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::router",kind:"function"})
MERGE (handler:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::list_task_events",kind:"function"})
MERGE (query:Type {id:"star_api_rest::group_api::bi_analytics::TaskEventQuery",kind:"struct"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQueryPort",kind:"trait"})
MERGE (auth:Symbol {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::authorize_run_owned_task_scope",kind:"function"})
MERGE (timeout:ExternalService {id:"tokio::time::timeout",kind:"function"})
MERGE (handler)-[:CALLS]->(timeout)
MERGE (function:Database {id:"bi.list_task_event_page_v1",kind:"stored_function"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(handler)
MERGE (self)-[:DEFINES]->(query)
MERGE (router)-[:CALLS]->(handler)
MERGE (handler)-[:CALLS]->(auth)
MERGE (handler)-[:CALLS]->(port)
MERGE (port)-[:CALLS]->(function)
@endcypher
*/
//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"bi_analytics.rs",type:"file",language:"rust"}),
//!   (router:Function {name:"bi_analytics::router",type:"function",language:"rust"}),
//!   (handler:Function {name:"list_task_events",type:"function",language:"rust"}),
//!   (query:Class {name:"TaskEventQuery",type:"class",language:"rust"}),
//!   (port:Interface {name:"TaskEventQueryPort",type:"interface",language:"rust"}),
//!   (auth:Function {name:"authorize_run_owned_task_scope",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(router),(f)-[:CONTAINS]->(handler),(f)-[:CONTAINS]->(query),
//!   (handler)-[:CALLS]->(auth),(handler)-[:CALLS]->(port),(router)-[:CALLS]->(handler);
//!
//! This Run BI endpoint returns only bounded allowlisted event facts. Access requires current
//! Project and Run authorization; its coverage and freshness remain unknown until a delivery
//! checkpoint becomes authoritative.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};
use bi_owner_runtime::{
    TaskEventQueryCursor, TaskEventQueryPage, TaskEventQueryPageRequest, TaskEventQueryPort,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use std::time::Duration;

use crate::auth::oauth::AuthenticatedUser;

use super::{
    engineering_runs::authorize_run_owned_task_scope, require_scope, set_tenant, validate_actor,
    GroupApiError, GroupApiState,
};

const DEFAULT_PAGE_SIZE: u16 = 50;
const MAX_CURSOR_TIMESTAMP_LENGTH: usize = 64;

pub(super) fn router() -> Router<GroupApiState> {
    Router::new().route(
        "/api/v1/engineering-runs/{engineering_run_id}/bi/task-events",
        get(list_task_events),
    )
}

#[derive(Debug, Deserialize)]
struct TaskEventQuery {
    limit: Option<u16>,
    before_occurred_at: Option<String>,
    before_event_id: Option<String>,
}

impl TaskEventQuery {
    fn into_request(
        self,
        tenant_id: Uuid,
        project_id: Uuid,
        engineering_run_id: Uuid,
    ) -> Result<TaskEventQueryPageRequest, GroupApiError> {
        let before = match (self.before_occurred_at, self.before_event_id) {
            (None, None) => None,
            (Some(timestamp), Some(event_id)) => {
                if timestamp.len() > MAX_CURSOR_TIMESTAMP_LENGTH || event_id.len() > 36 {
                    return Err(GroupApiError::bad_request());
                }
                let occurred_at = DateTime::parse_from_rfc3339(&timestamp)
                    .map_err(|_| GroupApiError::bad_request())?
                    .with_timezone(&Utc);
                let event_id = Uuid::parse_str(&event_id)
                    .map_err(|_| GroupApiError::bad_request())?;
                Some(
                    TaskEventQueryCursor::new(occurred_at, event_id)
                        .map_err(|_| GroupApiError::bad_request())?,
                )
            }
            _ => return Err(GroupApiError::bad_request()),
        };

        TaskEventQueryPageRequest::new(
            tenant_id,
            project_id,
            engineering_run_id,
            self.limit.unwrap_or(DEFAULT_PAGE_SIZE),
            before,
        )
        .map_err(|_| GroupApiError::bad_request())
    }
}

async fn list_task_events(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<TaskEventQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "project:read")?;
    let run_id = Uuid::parse_str(&run_id).map_err(|_| GroupApiError::bad_request())?;

    let mut authorization_tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut authorization_tx, actor.tenant_id).await?;
    sqlx::query(
        "SELECT set_config('statement_timeout', '3000', true), set_config('lock_timeout', '1000', true)",
    )
    .execute(&mut *authorization_tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let scope = authorize_run_owned_task_scope(&mut authorization_tx, &actor, run_id).await?;
    let request = query.into_request(actor.tenant_id, scope.project_id, scope.engineering_run_id)?;

    let query_port = state
        .bi_task_event_query_port
        .as_ref()
        .ok_or_else(|| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?;
    let page = tokio::time::timeout(
        Duration::from_secs(6),
        query_port.list_task_events(request),
    )
    .await
    .map_err(|_| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?
    .map_err(|_| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?;
    authorization_tx
        .commit()
        .await
        .map_err(|_| GroupApiError::internal())?;

    Ok(super::reply(page_response(scope.project_id, run_id, page)))
}

fn page_response(project_id: Uuid, engineering_run_id: Uuid, page: TaskEventQueryPage) -> serde_json::Value {
    let next_cursor = page.next_cursor.map(|cursor| {
        json!({
            "before_occurred_at": cursor.occurred_at,
            "before_event_id": cursor.event_id,
        })
    });
    json!({
        "project_id": project_id,
        "engineering_run_id": engineering_run_id,
        "items": page.items,
        "has_more": page.has_more,
        "next_cursor": next_cursor,
        "coverage": page.coverage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TENANT_ID: Uuid = Uuid::from_u128(1);
    const PROJECT_ID: Uuid = Uuid::from_u128(2);
    const RUN_ID: Uuid = Uuid::from_u128(3);

    #[test]
    fn query_rejects_partial_and_invalid_cursors() {
        let partial = TaskEventQuery {
            limit: None,
            before_occurred_at: Some("2026-10-17T00:00:00Z".to_string()),
            before_event_id: None,
        };
        assert!(partial
            .into_request(TENANT_ID, PROJECT_ID, RUN_ID)
            .is_err());

        let invalid = TaskEventQuery {
            limit: None,
            before_occurred_at: Some("not-a-time".to_string()),
            before_event_id: Some(Uuid::new_v4().to_string()),
        };
        assert!(invalid
            .into_request(TENANT_ID, PROJECT_ID, RUN_ID)
            .is_err());
    }

    #[test]
    fn query_enforces_bounded_page_size_and_cursor_identity() {
        let oversized = TaskEventQuery {
            limit: Some(101),
            before_occurred_at: None,
            before_event_id: None,
        };
        assert!(oversized
            .into_request(TENANT_ID, PROJECT_ID, RUN_ID)
            .is_err());

        let nil_cursor = TaskEventQuery {
            limit: Some(1),
            before_occurred_at: Some("2026-10-17T00:00:00Z".to_string()),
            before_event_id: Some(Uuid::nil().to_string()),
        };
        assert!(nil_cursor
            .into_request(TENANT_ID, PROJECT_ID, RUN_ID)
            .is_err());
    }
}

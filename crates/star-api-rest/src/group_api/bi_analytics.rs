// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=4919e74d83e16ac54ded06eac2e830ed5fc91e9c4886f2d3c3ed6933fc77c7d6
MERGE (self:File {path:"crates/star-api-rest/src/group_api/bi_analytics.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::bi_analytics",kind:"module"})
MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::router",kind:"function"})
MERGE (handler:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::list_task_events",kind:"function"})
MERGE (summaryHandler:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::get_task_summary",kind:"function"})
MERGE (collect:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::collect_scoped_task_events",kind:"function"})
MERGE (aggregate:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::aggregate_task_summary",kind:"function"})
MERGE (summaryResponse:Symbol {id:"crates/star-api-rest/src/group_api/bi_analytics.rs::summary_response",kind:"function"})
MERGE (query:Type {id:"star_api_rest::group_api::bi_analytics::TaskEventQuery",kind:"struct"})
MERGE (bucket:Type {id:"star_api_rest::group_api::bi_analytics::TaskSummaryStatusBucket",kind:"struct"})
MERGE (range:Type {id:"star_api_rest::group_api::bi_analytics::TaskSummarySourceRange",kind:"struct"})
MERGE (projection:Type {id:"star_api_rest::group_api::bi_analytics::TaskSummaryProjection",kind:"struct"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQueryPort",kind:"trait"})
MERGE (auth:Symbol {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::authorize_run_owned_task_scope",kind:"function"})
MERGE (timeout:ExternalService {id:"tokio::time::timeout",kind:"function"})
MERGE (handler)-[:CALLS]->(timeout)
MERGE (summaryHandler)-[:CALLS]->(timeout)
MERGE (function:Database {id:"bi.list_task_event_page_v1",kind:"stored_function"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(handler)
MERGE (self)-[:DEFINES]->(summaryHandler)
MERGE (self)-[:DEFINES]->(collect)
MERGE (self)-[:DEFINES]->(aggregate)
MERGE (self)-[:DEFINES]->(summaryResponse)
MERGE (self)-[:DEFINES]->(query)
MERGE (self)-[:DEFINES]->(bucket)
MERGE (self)-[:DEFINES]->(range)
MERGE (self)-[:DEFINES]->(projection)
MERGE (router)-[:CALLS]->(handler)
MERGE (router)-[:CALLS]->(summaryHandler)
MERGE (handler)-[:CALLS]->(auth)
MERGE (handler)-[:CALLS]->(port)
MERGE (summaryHandler)-[:CALLS]->(auth)
MERGE (summaryHandler)-[:CALLS]->(collect)
MERGE (summaryHandler)-[:CALLS]->(aggregate)
MERGE (summaryHandler)-[:CALLS]->(summaryResponse)
MERGE (collect)-[:CALLS]->(port)
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
    Json, Router,
};
use bi_owner_runtime::{
    TaskEventQueryCoverage, TaskEventQueryCursor, TaskEventQueryItem, TaskEventQueryPage,
    TaskEventQueryPageRequest,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;
use uuid::Uuid;

use crate::auth::oauth::AuthenticatedUser;

use super::{
    engineering_runs::authorize_run_owned_task_scope, require_scope, set_tenant, validate_actor,
    GroupApiError, GroupApiState,
};

const DEFAULT_PAGE_SIZE: u16 = 50;
const MAX_CURSOR_TIMESTAMP_LENGTH: usize = 64;
/// Hard page-count bound for a full-scope Task summary walk. Beyond this bound the summary
/// refuses to fabricate a partial aggregate and reports the scope as unavailable instead.
const MAX_SUMMARY_PAGES: usize = 20_000;
/// Latest projected lifecycle status that counts toward the completed Task metric.
const COMPLETED_STATUS: &str = "completed";
/// Status label used when the source event carries no projected lifecycle status.
const UNKNOWN_STATUS: &str = "unknown";

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/bi/task-events",
            get(list_task_events),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/bi/task-summary",
            get(get_task_summary),
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
                let event_id =
                    Uuid::parse_str(&event_id).map_err(|_| GroupApiError::bad_request())?;
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
    let request =
        query.into_request(actor.tenant_id, scope.project_id, scope.engineering_run_id)?;

    let query_port = state
        .bi_task_event_query_port
        .as_ref()
        .ok_or_else(|| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?;
    let page = tokio::time::timeout(Duration::from_secs(6), query_port.list_task_events(request))
        .await
        .map_err(|_| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?
        .map_err(|_| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?;
    authorization_tx
        .commit()
        .await
        .map_err(|_| GroupApiError::internal())?;

    Ok(Json(page_response(scope.project_id, run_id, page)))
}

fn page_response(
    project_id: Uuid,
    engineering_run_id: Uuid,
    page: TaskEventQueryPage,
) -> serde_json::Value {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// One observed bucket of the latest projected lifecycle status per work item.
pub(super) struct TaskSummaryStatusBucket {
    /// Latest projected lifecycle status; `unknown` when the source event carries none.
    pub status: String,
    /// Number of work items whose latest observed event projects this status.
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Bounds of the observed event stream covered by a Run Task summary.
pub(super) struct TaskSummarySourceRange {
    /// UTC occurrence time of the earliest observed event, when any event exists.
    pub earliest_occurred_at: Option<DateTime<Utc>>,
    /// UTC occurrence time of the latest observed event, when any event exists.
    pub latest_occurred_at: Option<DateTime<Utc>>,
    /// Number of observed events in the covered range; always equals the summary total.
    pub event_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Aggregated Run-scoped Task facts derived exclusively from the Task event query source.
pub(super) struct TaskSummaryProjection {
    /// Distribution of the latest lifecycle status per work item.
    pub status_distribution: Vec<TaskSummaryStatusBucket>,
    /// Work items whose latest observed status is [`COMPLETED_STATUS`].
    pub completed_count: u64,
    /// Total number of observed events; equals the full `task-events` count for the same scope.
    pub total_count: u64,
    /// Distinct work items observed; equals the sum of the status distribution.
    pub work_item_count: u64,
    /// UTC instant at which this aggregate was computed.
    pub as_of: DateTime<Utc>,
    /// Covered occurrence range of the source events backing the aggregate.
    pub source_event_range: TaskSummarySourceRange,
}

/// Aggregate the allowlisted event facts into a Task summary.
///
/// Pages arrive in descending `occurred_at` keyset order, so the first event observed per
/// work item is that work item's most recent one. Status distribution and completion both
/// use this latest-per-work-item projection; `total_count` counts every event.
fn aggregate_task_summary(
    items: &[TaskEventQueryItem],
    as_of: DateTime<Utc>,
) -> TaskSummaryProjection {
    let mut latest_status: BTreeMap<Uuid, &str> = BTreeMap::new();
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;
    for item in items {
        latest_status
            .entry(item.work_item_id)
            .or_insert_with(|| item.lifecycle_status.as_deref().unwrap_or(UNKNOWN_STATUS));
        earliest = Some(earliest.map_or(item.occurred_at, |time| time.min(item.occurred_at)));
        latest = Some(latest.map_or(item.occurred_at, |time| time.max(item.occurred_at)));
    }
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for status in latest_status.values() {
        *counts.entry((*status).to_string()).or_default() += 1;
    }
    let completed_count = counts.get(COMPLETED_STATUS).copied().unwrap_or(0);
    let mut status_distribution: Vec<TaskSummaryStatusBucket> = counts
        .into_iter()
        .map(|(status, count)| TaskSummaryStatusBucket { status, count })
        .collect();
    status_distribution.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.status.cmp(&right.status))
    });
    TaskSummaryProjection {
        status_distribution,
        completed_count,
        total_count: items.len() as u64,
        work_item_count: latest_status.len() as u64,
        as_of,
        source_event_range: TaskSummarySourceRange {
            earliest_occurred_at: earliest,
            latest_occurred_at: latest,
            event_count: items.len() as u64,
        },
    }
}

/// Walk the entire Run-scoped Task event stream through the same owner query port that backs
/// `task-events`, honoring its keyset cursors, so aggregate and source-record counts reconcile.
///
/// Returns the observed facts plus the query coverage of the first page. Fails closed when the
/// port is unavailable, a page is inconsistent, or the scope exceeds [`MAX_SUMMARY_PAGES`].
async fn collect_scoped_task_events(
    state: &GroupApiState,
    tenant_id: Uuid,
    project_id: Uuid,
    engineering_run_id: Uuid,
) -> Result<(Vec<TaskEventQueryItem>, TaskEventQueryCoverage), GroupApiError> {
    let query_port = state
        .bi_task_event_query_port
        .as_ref()
        .ok_or_else(|| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?;
    let mut events: Vec<TaskEventQueryItem> = Vec::new();
    let mut before: Option<TaskEventQueryCursor> = None;
    let mut coverage: Option<TaskEventQueryCoverage> = None;
    let mut exhausted = false;
    for _ in 0..MAX_SUMMARY_PAGES {
        let request = TaskEventQueryPageRequest::new(
            tenant_id,
            project_id,
            engineering_run_id,
            DEFAULT_PAGE_SIZE,
            before.take(),
        )
        .map_err(|_| GroupApiError::internal())?;
        let page =
            tokio::time::timeout(Duration::from_secs(6), query_port.list_task_events(request))
                .await
                .map_err(|_| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?
                .map_err(|_| GroupApiError::feature_unavailable("bi_task_query_unavailable"))?;
        if coverage.is_none() {
            coverage = Some(page.coverage);
        }
        if page.has_more && page.next_cursor.is_none() {
            return Err(GroupApiError::internal());
        }
        if page.items.is_empty() {
            if page.has_more {
                return Err(GroupApiError::internal());
            }
            exhausted = true;
            break;
        }
        events.extend(page.items);
        if !page.has_more {
            exhausted = true;
            break;
        }
        before = page.next_cursor;
    }
    if !exhausted {
        return Err(GroupApiError::feature_unavailable(
            "bi_task_summary_scope_too_large",
        ));
    }
    let coverage = coverage.unwrap_or(TaskEventQueryCoverage {
        completeness: bi_owner_runtime::TaskEventQueryCoverageState::Unknown,
        freshness: bi_owner_runtime::TaskEventQueryCoverageState::Unknown,
        basis: "no observed event page returned coverage evidence",
    });
    Ok((events, coverage))
}

async fn get_task_summary(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
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

    let (events, coverage) = collect_scoped_task_events(
        &state,
        actor.tenant_id,
        scope.project_id,
        scope.engineering_run_id,
    )
    .await?;
    authorization_tx
        .commit()
        .await
        .map_err(|_| GroupApiError::internal())?;

    let summary = aggregate_task_summary(&events, Utc::now());
    Ok(Json(summary_response(
        scope.project_id,
        run_id,
        summary,
        coverage,
    )))
}

fn summary_response(
    project_id: Uuid,
    engineering_run_id: Uuid,
    summary: TaskSummaryProjection,
    coverage: TaskEventQueryCoverage,
) -> serde_json::Value {
    json!({
        "project_id": project_id,
        "engineering_run_id": engineering_run_id,
        "status_distribution": summary.status_distribution,
        "completed_count": summary.completed_count,
        "total_count": summary.total_count,
        "work_item_count": summary.work_item_count,
        "as_of": summary.as_of,
        "source_event_range": summary.source_event_range,
        "coverage": coverage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bi_owner_runtime::{TaskEventQueryError, TaskEventQueryPort};

    const TENANT_ID: Uuid = Uuid::from_u128(1);
    const PROJECT_ID: Uuid = Uuid::from_u128(2);
    const RUN_ID: Uuid = Uuid::from_u128(3);

    fn event(
        event_id: u128,
        work_item_id: u128,
        occurred_minute: i64,
        status: Option<&str>,
    ) -> TaskEventQueryItem {
        TaskEventQueryItem {
            event_id: Uuid::from_u128(event_id),
            work_item_id: Uuid::from_u128(work_item_id),
            aggregate_type: "work_item".to_string(),
            aggregate_version: 1,
            event_type: "lifecycle_changed".to_string(),
            schema_version: 1,
            occurred_at: Utc::now() - chrono::Duration::minutes(occurred_minute),
            lifecycle_status: status.map(str::to_string),
            review_state: Some("none".to_string()),
            from_status: None,
            to_status: None,
            owner_changed: false,
        }
    }

    fn coverage() -> bi_owner_runtime::TaskEventQueryCoverage {
        bi_owner_runtime::TaskEventQueryCoverage {
            completeness: bi_owner_runtime::TaskEventQueryCoverageState::Unknown,
            freshness: bi_owner_runtime::TaskEventQueryCoverageState::Unknown,
            basis: "unit-test fixture",
        }
    }

    /// Scripted page sequence: the collector cannot inspect the private request shape, so this
    /// fixture replays the exact page/cursor transitions a real store would produce.
    struct ScriptedTaskEventQueryPort {
        pages: std::sync::Mutex<std::collections::VecDeque<TaskEventQueryPage>>,
        call_count: std::sync::atomic::AtomicUsize,
    }

    impl ScriptedTaskEventQueryPort {
        fn new(pages: Vec<TaskEventQueryPage>) -> Self {
            Self {
                pages: std::sync::Mutex::new(pages.into()),
                call_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.call_count.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl TaskEventQueryPort for ScriptedTaskEventQueryPort {
        async fn list_task_events(
            &self,
            _request: TaskEventQueryPageRequest,
        ) -> Result<TaskEventQueryPage, TaskEventQueryError> {
            self.call_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.pages
                .lock()
                .expect("fixture lock poisoned")
                .pop_front()
                .ok_or(TaskEventQueryError::InvalidRequest)
        }
    }

    #[test]
    fn query_rejects_partial_and_invalid_cursors() {
        let partial = TaskEventQuery {
            limit: None,
            before_occurred_at: Some("2026-10-17T00:00:00Z".to_string()),
            before_event_id: None,
        };
        assert!(partial.into_request(TENANT_ID, PROJECT_ID, RUN_ID).is_err());

        let invalid = TaskEventQuery {
            limit: None,
            before_occurred_at: Some("not-a-time".to_string()),
            before_event_id: Some(Uuid::new_v4().to_string()),
        };
        assert!(invalid.into_request(TENANT_ID, PROJECT_ID, RUN_ID).is_err());
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

    #[test]
    fn aggregate_counts_every_event_and_projects_latest_status_per_work_item() {
        // Events arrive in descending occurrence order: for work item 2 the newest event
        // projects completed and an older one carries no status; work item 1's newest
        // event says in_progress (older one said pending).
        let items = vec![
            event(14, 2, 1, Some("completed")),
            event(11, 1, 1, Some("in_progress")),
            event(12, 2, 2, None),
            event(13, 1, 5, Some("pending")),
        ];
        let summary = aggregate_task_summary(&items, Utc::now());

        assert_eq!(summary.total_count, 4, "total must count every event");
        assert_eq!(summary.source_event_range.event_count, 4);
        assert_eq!(summary.work_item_count, 2);
        let buckets: BTreeMap<&str, u64> = summary
            .status_distribution
            .iter()
            .map(|bucket| (bucket.status.as_str(), bucket.count))
            .collect();
        assert_eq!(buckets.get("in_progress"), Some(&1));
        assert_eq!(buckets.get("completed"), Some(&1));
        assert_eq!(
            buckets.get("unknown"),
            None,
            "older None status must not shadow the latest"
        );
        assert_eq!(summary.completed_count, 1);
        assert!(summary.source_event_range.earliest_occurred_at.is_some());
        assert!(summary.source_event_range.latest_occurred_at.is_some());
        assert!(
            summary.source_event_range.earliest_occurred_at
                <= summary.source_event_range.latest_occurred_at
        );
    }

    #[test]
    fn aggregate_returns_zero_values_for_empty_events() {
        let summary = aggregate_task_summary(&[], Utc::now());
        assert_eq!(summary.total_count, 0);
        assert_eq!(summary.completed_count, 0);
        assert_eq!(summary.work_item_count, 0);
        assert!(summary.status_distribution.is_empty());
        assert_eq!(summary.source_event_range.event_count, 0);
        assert_eq!(summary.source_event_range.earliest_occurred_at, None);
        assert_eq!(summary.source_event_range.latest_occurred_at, None);
    }

    #[tokio::test]
    async fn collect_walks_cursor_pages_until_exhausted_without_truncation() {
        let cursor = |event_id: u128| {
            Some(
                TaskEventQueryCursor::new(
                    Utc::now() - chrono::Duration::minutes(1),
                    Uuid::from_u128(event_id),
                )
                .expect("non-nil cursor"),
            )
        };
        // 3 pages of 2 items each: first two carry cursors, the last terminates the walk.
        let port = std::sync::Arc::new(ScriptedTaskEventQueryPort::new(vec![
            TaskEventQueryPage {
                items: vec![
                    event(21, 1, 1, Some("pending")),
                    event(22, 2, 2, Some("completed")),
                ],
                has_more: true,
                next_cursor: cursor(22),
                coverage: coverage(),
            },
            TaskEventQueryPage {
                items: vec![
                    event(23, 3, 3, Some("failed")),
                    event(24, 4, 4, Some("completed")),
                ],
                has_more: true,
                next_cursor: cursor(24),
                coverage: coverage(),
            },
            TaskEventQueryPage {
                items: vec![
                    event(26, 5, 4, Some("completed")),
                    event(25, 5, 5, Some("cancelled")),
                ],
                has_more: false,
                next_cursor: None,
                coverage: coverage(),
            },
        ]));
        let state = test_state_with_port(port.clone());

        let (events, observed_coverage) =
            collect_scoped_task_events(&state, TENANT_ID, PROJECT_ID, RUN_ID)
                .await
                .expect("scripted walk must complete");

        assert_eq!(port.calls(), 3, "one port call per scripted page");
        assert_eq!(events.len(), 6, "no event may be dropped or truncated");
        assert_eq!(observed_coverage.basis, "unit-test fixture");

        let summary = aggregate_task_summary(&events, Utc::now());
        assert_eq!(summary.total_count, 6);
        assert_eq!(summary.completed_count, 3);
        // Work item 5's latest (first-seen) status is completed, not cancelled.
        assert_eq!(summary.work_item_count, 5);
    }

    fn test_state_with_port(port: std::sync::Arc<ScriptedTaskEventQueryPort>) -> GroupApiState {
        GroupApiState::new(
            std::sync::Arc::new(crate::auth::JwtConfig {
                private_key_pem: String::new(),
                public_key_pem: String::new(),
                issuer: "https://unit.test".to_string(),
                audience: "star-api-rest".to_string(),
                ttl_seconds: 3600,
            }),
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://localhost/unavailable-for-bi-unit-test")
                .expect("lazy pool"),
        )
        .with_bi_task_event_query_port(port)
    }

    #[tokio::test]
    async fn collect_accepts_empty_scope_and_rejects_inconsistent_pages() {
        let empty_port = ScriptedTaskEventQueryPort::new(vec![TaskEventQueryPage {
            items: vec![],
            has_more: false,
            next_cursor: None,
            coverage: coverage(),
        }]);
        let state = GroupApiState::new(
            std::sync::Arc::new(crate::auth::JwtConfig {
                private_key_pem: String::new(),
                public_key_pem: String::new(),
                issuer: "https://unit.test".to_string(),
                audience: "star-api-rest".to_string(),
                ttl_seconds: 3600,
            }),
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://localhost/unavailable-for-bi-unit-test")
                .expect("lazy pool"),
        )
        .with_bi_task_event_query_port(std::sync::Arc::new(empty_port));
        let (events, _) = collect_scoped_task_events(&state, TENANT_ID, PROJECT_ID, RUN_ID)
            .await
            .expect("empty scope is a valid zero-value result");
        assert!(events.is_empty());

        let stalled = ScriptedTaskEventQueryPort::new(vec![TaskEventQueryPage {
            items: vec![],
            has_more: true,
            next_cursor: Some(
                TaskEventQueryCursor::new(
                    Utc::now() - chrono::Duration::minutes(1),
                    Uuid::from_u128(99),
                )
                .expect("non-nil cursor"),
            ),
            coverage: coverage(),
        }]);
        let state = GroupApiState::new(
            std::sync::Arc::new(crate::auth::JwtConfig {
                private_key_pem: String::new(),
                public_key_pem: String::new(),
                issuer: "https://unit.test".to_string(),
                audience: "star-api-rest".to_string(),
                ttl_seconds: 3600,
            }),
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://localhost/unavailable-for-bi-unit-test")
                .expect("lazy pool"),
        )
        .with_bi_task_event_query_port(std::sync::Arc::new(stalled));
        assert!(
            collect_scoped_task_events(&state, TENANT_ID, PROJECT_ID, RUN_ID)
                .await
                .is_err(),
            "empty page claiming more data is inconsistent and must fail closed"
        );

        let missing = ScriptedTaskEventQueryPort::new(vec![TaskEventQueryPage {
            items: vec![event(31, 1, 1, Some("pending"))],
            has_more: true,
            next_cursor: None,
            coverage: coverage(),
        }]);
        let state = GroupApiState::new(
            std::sync::Arc::new(crate::auth::JwtConfig {
                private_key_pem: String::new(),
                public_key_pem: String::new(),
                issuer: "https://unit.test".to_string(),
                audience: "star-api-rest".to_string(),
                ttl_seconds: 3600,
            }),
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://localhost/unavailable-for-bi-unit-test")
                .expect("lazy pool"),
        )
        .with_bi_task_event_query_port(std::sync::Arc::new(missing));
        assert!(
            collect_scoped_task_events(&state, TENANT_ID, PROJECT_ID, RUN_ID)
                .await
                .is_err(),
            "has_more without a cursor is inconsistent and must fail closed"
        );
    }
}

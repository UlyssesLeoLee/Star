// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=1125bcec36307f099f71459e4be988f9a225bf8572afcf0858be3de15d336d6d
MERGE (self:File {path:"crates/bi-owner-runtime/src/task_event_query.rs"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQueryPort",kind:"trait"})
MERGE (store:Type {id:"bi_owner_runtime::PgTaskEventQueryStore",kind:"struct"})
MERGE (request:Type {id:"bi_owner_runtime::TaskEventQueryPageRequest",kind:"struct"})
MERGE (cursor:Type {id:"bi_owner_runtime::TaskEventQueryCursor",kind:"struct"})
MERGE (item:Type {id:"bi_owner_runtime::TaskEventQueryItem",kind:"struct"})
MERGE (page:Type {id:"bi_owner_runtime::TaskEventQueryPage",kind:"struct"})
MERGE (coverage:Type {id:"bi_owner_runtime::TaskEventQueryCoverage",kind:"struct"})
MERGE (error:Type {id:"bi_owner_runtime::TaskEventQueryError",kind:"enum"})
MERGE (new:Symbol {id:"crates/bi-owner-runtime/src/task_event_query.rs::TaskEventQueryPageRequest.new",kind:"method"})
MERGE (list:Symbol {id:"crates/bi-owner-runtime/src/task_event_query.rs::TaskEventQueryPort.list_task_events",kind:"method"})
MERGE (store_new:Symbol {id:"crates/bi-owner-runtime/src/task_event_query.rs::PgTaskEventQueryStore.new",kind:"method"})
MERGE (store_list:Symbol {id:"crates/bi-owner-runtime/src/task_event_query.rs::PgTaskEventQueryStore.list_task_events",kind:"method"})
MERGE (pool:Type {id:"sqlx::PgPool",kind:"struct"})
MERGE (function:Database {id:"bi.list_task_event_page_v1",kind:"stored_function"})
MERGE (self)-[:DEFINES]->(port)
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(request)
MERGE (self)-[:DEFINES]->(cursor)
MERGE (self)-[:DEFINES]->(item)
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:DEFINES]->(coverage)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:IMPORTS]->(pool)
MERGE (request)-[:HAS_METHOD]->(new)
MERGE (port)-[:HAS_METHOD]->(list)
MERGE (store)-[:IMPLEMENTS]->(port)
MERGE (store)-[:HAS_METHOD]->(store_new)
MERGE (store)-[:HAS_METHOD]->(store_list)
MERGE (store_new)-[:USES_TYPE]->(pool)
MERGE (store_list)-[:USES_TYPE]->(request)
MERGE (store_list)-[:CALLS]->(function)
MERGE (function)-[:READS]->(table:Table {id:"bi.task_event_fact",kind:"table"})
@endcypher
*/
//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"task_event_query.rs",type:"file",language:"rust"}),
//!   (port:Interface {name:"TaskEventQueryPort",type:"interface",language:"rust"}),
//!   (store:Class {name:"PgTaskEventQueryStore",type:"class",language:"rust"}),
//!   (request:Class {name:"TaskEventQueryPageRequest",type:"class",language:"rust"}),
//!   (cursor:Class {name:"TaskEventQueryCursor",type:"class",language:"rust"}),
//!   (item:Class {name:"TaskEventQueryItem",type:"class",language:"rust"}),
//!   (page:Class {name:"TaskEventQueryPage",type:"class",language:"rust"}),
//!   (coverage:Class {name:"TaskEventQueryCoverage",type:"class",language:"rust"}),
//!   (error:Enum {name:"TaskEventQueryError",type:"enum",language:"rust"}),
//!   (new:Function {name:"TaskEventQueryPageRequest::new",type:"function",language:"rust"}),
//!   (query:Function {name:"PgTaskEventQueryStore::list_task_events",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(port),(f)-[:CONTAINS]->(store),(f)-[:CONTAINS]->(request),
//!   (f)-[:CONTAINS]->(cursor),(f)-[:CONTAINS]->(item),(f)-[:CONTAINS]->(page),
//!   (f)-[:CONTAINS]->(coverage),(f)-[:CONTAINS]->(error),(request)-[:CONTAINS]->(new),
//!   (store)-[:CONTAINS]->(query),(store)-[:IMPLEMENTS]->(port);
//!
//! This owner-local reader calls only the fixed BI database query function and returns an
//! allowlisted event page. It does not expose payload, actor, correlation, route, or wire data.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

/// Maximum number of observed event facts returned in one page.
pub const MAX_TASK_EVENT_QUERY_PAGE_SIZE: u16 = 100;

#[derive(Debug, Error)]
/// Validation and database failures of a BI Task event query.
pub enum TaskEventQueryError {
    #[error("invalid BI Task event query")]
    /// The query scope, cursor, or page bound is invalid.
    InvalidRequest,
    #[error("BI Task event query database operation failed")]
    /// The owner-local PostgreSQL operation failed.
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Stable position in descending event time and ascending event identity order.
pub struct TaskEventQueryCursor {
    /// UTC occurrence time of the observed event.
    pub occurred_at: DateTime<Utc>,
    /// Immutable source event identity.
    pub event_id: Uuid,
}

impl TaskEventQueryCursor {
    /// Validate and construct a cursor with a nonnil event identity.
    pub fn new(occurred_at: DateTime<Utc>, event_id: Uuid) -> Result<Self, TaskEventQueryError> {
        if event_id.is_nil() {
            return Err(TaskEventQueryError::InvalidRequest);
        }
        Ok(Self {
            occurred_at,
            event_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Validated tenant, Project, Run, and page bounds for an owner query.
pub struct TaskEventQueryPageRequest {
    tenant_id: Uuid,
    project_id: Uuid,
    engineering_run_id: Uuid,
    page_size: u16,
    before: Option<TaskEventQueryCursor>,
}

impl TaskEventQueryPageRequest {
    /// Validate the scope identities and enforce the public page-size bound.
    pub fn new(
        tenant_id: Uuid,
        project_id: Uuid,
        engineering_run_id: Uuid,
        page_size: u16,
        before: Option<TaskEventQueryCursor>,
    ) -> Result<Self, TaskEventQueryError> {
        if tenant_id.is_nil()
            || project_id.is_nil()
            || engineering_run_id.is_nil()
            || !(1..=MAX_TASK_EVENT_QUERY_PAGE_SIZE).contains(&page_size)
            || before.as_ref().is_some_and(|cursor| cursor.event_id.is_nil())
        {
            return Err(TaskEventQueryError::InvalidRequest);
        }
        Ok(Self {
            tenant_id,
            project_id,
            engineering_run_id,
            page_size,
            before,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Allowlisted event fact projection without payload or actor information.
pub struct TaskEventQueryItem {
    /// Immutable source event identity.
    pub event_id: Uuid,
    /// Task or work item identity associated with the fact.
    pub work_item_id: Uuid,
    /// Versioned event aggregate category.
    pub aggregate_type: String,
    /// Source aggregate revision carried by the event.
    pub aggregate_version: i32,
    /// Allowlisted lifecycle or change event category.
    pub event_type: String,
    /// Source event schema version.
    pub schema_version: i32,
    /// UTC occurrence time of the observed event.
    pub occurred_at: DateTime<Utc>,
    /// Projected lifecycle status, when supplied by the source event.
    pub lifecycle_status: Option<String>,
    /// Projected review state, when supplied by the source event.
    pub review_state: Option<String>,
    /// Previous lifecycle status of a status-change event.
    pub from_status: Option<String>,
    /// Resulting lifecycle status of a status-change event.
    pub to_status: Option<String>,
    /// Whether the event records an ownership change.
    pub owner_changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Evidence state for delivery completeness or freshness.
pub enum TaskEventQueryCoverageState {
    /// No authoritative delivery checkpoint currently establishes this property.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Explicit evidence limits accompanying the observed event page.
pub struct TaskEventQueryCoverage {
    /// Whether authoritative evidence establishes delivery completeness.
    pub completeness: TaskEventQueryCoverageState,
    /// Whether authoritative evidence establishes delivery freshness.
    pub freshness: TaskEventQueryCoverageState,
    /// Human-readable basis for the reported evidence states.
    pub basis: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Bounded observed facts with a continuation cursor and evidence limits.
pub struct TaskEventQueryPage {
    /// Allowlisted facts in stable keyset order.
    pub items: Vec<TaskEventQueryItem>,
    /// Whether an additional matching fact was observed beyond this page.
    pub has_more: bool,
    /// Continuation position when another page is available.
    pub next_cursor: Option<TaskEventQueryCursor>,
    /// Completeness and freshness evidence for this projection.
    pub coverage: TaskEventQueryCoverage,
}

#[async_trait::async_trait]
/// Read-only owner boundary for querying bounded observed Task facts.
pub trait TaskEventQueryPort: Send + Sync {
    /// Read one page within the validated tenant, Project, and Run scope.
    async fn list_task_events(
        &self,
        request: TaskEventQueryPageRequest,
    ) -> Result<TaskEventQueryPage, TaskEventQueryError>;
}

/// PostgreSQL reader restricted to the BI owner query function.
pub struct PgTaskEventQueryStore {
    pool: PgPool,
}

impl PgTaskEventQueryStore {
    /// Bind this reader to the BI owner's query-only application role.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct TaskEventQueryRow {
    event_id: Uuid,
    work_item_id: Uuid,
    aggregate_type: String,
    aggregate_version: i32,
    event_type: String,
    schema_version: i32,
    occurred_at: DateTime<Utc>,
    lifecycle_status: Option<String>,
    review_state: Option<String>,
    from_status: Option<String>,
    to_status: Option<String>,
    owner_changed: bool,
}

#[async_trait::async_trait]
impl TaskEventQueryPort for PgTaskEventQueryStore {
    async fn list_task_events(
        &self,
        request: TaskEventQueryPageRequest,
    ) -> Result<TaskEventQueryPage, TaskEventQueryError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
            .bind(request.tenant_id.to_string())
            .execute(&mut *transaction)
            .await?;
        sqlx::query("SELECT set_config('statement_timeout', '3000', true), set_config('lock_timeout', '1000', true)")
            .execute(&mut *transaction)
            .await?;

        let before_occurred_at = request.before.as_ref().map(|cursor| cursor.occurred_at);
        let before_event_id = request.before.as_ref().map(|cursor| cursor.event_id);
        let rows = sqlx::query_as::<_, TaskEventQueryRow>(
            "SELECT event_id, work_item_id, aggregate_type, aggregate_version, event_type, \
                    schema_version, occurred_at, lifecycle_status, review_state, from_status, \
                    to_status, owner_changed \
             FROM bi.list_task_event_page_v1($1, $2, $3, $4, $5, $6)",
        )
        .bind(request.tenant_id)
        .bind(request.project_id)
        .bind(request.engineering_run_id)
        .bind(before_occurred_at)
        .bind(before_event_id)
        .bind(i32::from(request.page_size) + 1)
        .fetch_all(&mut *transaction)
        .await?;
        transaction.commit().await?;

        let has_more = rows.len() > usize::from(request.page_size);
        let items = rows
            .into_iter()
            .take(usize::from(request.page_size))
            .map(|row| TaskEventQueryItem {
                event_id: row.event_id,
                work_item_id: row.work_item_id,
                aggregate_type: row.aggregate_type,
                aggregate_version: row.aggregate_version,
                event_type: row.event_type,
                schema_version: row.schema_version,
                occurred_at: row.occurred_at,
                lifecycle_status: row.lifecycle_status,
                review_state: row.review_state,
                from_status: row.from_status,
                to_status: row.to_status,
                owner_changed: row.owner_changed,
            })
            .collect::<Vec<_>>();
        let next_cursor = if has_more {
            items.last().map(|item| TaskEventQueryCursor {
                occurred_at: item.occurred_at,
                event_id: item.event_id,
            })
        } else {
            None
        };

        Ok(TaskEventQueryPage {
            items,
            has_more,
            next_cursor,
            coverage: TaskEventQueryCoverage {
                completeness: TaskEventQueryCoverageState::Unknown,
                freshness: TaskEventQueryCoverageState::Unknown,
                basis: "observed BI facts only; no authoritative delivery checkpoint is configured",
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_rejects_nil_scope_and_out_of_bounds_page_size() {
        let tenant_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();

        assert!(TaskEventQueryPageRequest::new(
            Uuid::nil(), project_id, run_id, 10, None
        )
        .is_err());
        assert!(TaskEventQueryPageRequest::new(
            tenant_id, project_id, run_id, 0, None
        )
        .is_err());
        assert!(TaskEventQueryPageRequest::new(
            tenant_id,
            project_id,
            run_id,
            MAX_TASK_EVENT_QUERY_PAGE_SIZE + 1,
            None
        )
        .is_err());
    }

    #[test]
    fn cursor_rejects_nil_event_identity() {
        assert!(TaskEventQueryCursor::new(Utc::now(), Uuid::nil()).is_err());
        assert!(TaskEventQueryPageRequest::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            Some(TaskEventQueryCursor {
                occurred_at: Utc::now(),
                event_id: Uuid::nil(),
            }),
        )
        .is_err());
    }
}

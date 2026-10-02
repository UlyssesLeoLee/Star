//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"task_runs.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"task_runs",type:"module",language:"rust"}),
//!   (list:Function {name:"list_task_runs",type:"function",language:"rust"}),
//!   (detail:Function {name:"get_task_run_detail",type:"function",language:"rust"}),
//!   (authorize:Function {name:"authorize_task_run_scope",type:"function",language:"rust"}),
//!   (limit:Function {name:"validate_run_limit",type:"function",language:"rust"}),
//!   (cursor:Function {name:"parse_run_cursor",type:"function",language:"rust"}),
//!   (summary:Class {name:"TaskRunSummary",type:"class",language:"rust"}),
//!   (event:Class {name:"TaskRunEventProjection",type:"class",language:"rust"}),
//!   (evidence:Class {name:"TaskRunEvidenceProjection",type:"class",language:"rust"});
//! MATCH (m:Module {name:"task_runs",type:"module",language:"rust"});
//! CREATE
//!   (m)-[:CONTAINS]->(list),(m)-[:CONTAINS]->(detail),(m)-[:CONTAINS]->(authorize),
//!   (m)-[:CONTAINS]->(limit),(m)-[:CONTAINS]->(cursor),(m)-[:CONTAINS]->(summary),
//!   (m)-[:CONTAINS]->(event),(m)-[:CONTAINS]->(evidence),
//!   (list)-[:CALLS]->(authorize),(list)-[:CALLS]->(limit),(list)-[:CALLS]->(cursor),
//!   (detail)-[:CALLS]->(authorize);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"task_runs",type:"module"}),(summary:Class {name:"TaskRunSummary",type:"class"});
//! CREATE (launch_id:Variable {name:"approved_launch_profile_id",type:"variable",language:"rust"}),(launch_version:Variable {name:"approved_launch_profile_version",type:"variable",language:"rust"}),(launch_digest:Variable {name:"approved_launch_profile_digest",type:"variable",language:"rust"}),(summary_json:Function {name:"run_summary_json",type:"function",language:"rust"}),(m)-[:CONTAINS]->(summary_json),(summary)-[:USES]->(launch_id),(summary)-[:USES]->(launch_version),(summary)-[:USES]->(launch_digest),(summary_json)-[:CALLS]->(summary),(summary_json)-[:USES]->(launch_id),(summary_json)-[:USES]->(launch_version),(summary_json)-[:USES]->(launch_digest);

use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::FromRow;
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, AuthenticatedUser, GroupApiError,
    GroupApiState,
};

const DEFAULT_RUN_LIMIT: i64 = 20;
const MAX_RUN_LIMIT: i64 = 50;
const MAX_DETAIL_EVENTS: i64 = 100;
const MAX_DETAIL_EVIDENCE: i64 = 100;

#[derive(Debug, Deserialize)]
struct TaskRunListQuery {
    limit: Option<i64>,
    cursor_started_at: Option<String>,
    cursor_run_id: Option<String>,
}

#[derive(Debug, FromRow)]
struct TaskRunScope {
    project_id: Uuid,
    repository_id: Uuid,
    branch_id: Uuid,
    engineering_run_id: Uuid,
}

#[derive(Debug, FromRow)]
struct TaskRunSummary {
    run_id: Uuid,
    worktree_id: Option<Uuid>,
    repository_id: Option<Uuid>,
    work_item_id: Uuid,
    initiated_by: Uuid,
    execution_channel: String,
    run_origin: String,
    start_ref: Option<String>,
    start_commit_ref: Option<String>,
    task_contract_version: Option<i64>,
    agent_id: Option<Uuid>,
    model_version: Option<String>,
    skill_version: Option<String>,
    orchestrator_version: Option<String>,
    strategy_version: Option<String>,
    execution_profile_id: Option<Uuid>,
    execution_profile_version: Option<i64>,
    execution_profile_digest: Option<String>,
    approved_launch_profile_id: Option<Uuid>,
    approved_launch_profile_version: Option<i64>,
    approved_launch_profile_digest: Option<String>,
    spawn_fence_binding_digest: Option<String>,
    execution_state: Option<String>,
    verification_state: Option<String>,
    human_acceptance_state: Option<String>,
    started_at: DateTime<Utc>,
    correlation_id: Uuid,
}

#[derive(Debug, FromRow)]
struct TaskRunEventProjection {
    event_id: Uuid,
    event_type: String,
    execution_state: Option<String>,
    agent_declaration_state: Option<String>,
    verification_state: Option<String>,
    human_acceptance_state: Option<String>,
    integration_state: Option<String>,
    failure_category: Option<String>,
    actual_cost_amount: Option<String>,
    estimated_cost_amount: Option<String>,
    cost_unit: Option<String>,
    automation_occurrence_id: Option<Uuid>,
    loop_iteration_no: Option<i32>,
    loop_phase: Option<String>,
    loop_decision: Option<String>,
    stop_reason: Option<String>,
    hook_set_version: Option<i64>,
    hook_rule_id: Option<Uuid>,
    hook_rule_version: Option<i64>,
    hook_evaluator_version: Option<String>,
    hook_digest: Option<String>,
    hook_phase: Option<String>,
    hook_decision: Option<String>,
    hook_reason_class: Option<String>,
    hook_duration_ms: Option<i64>,
    hook_timed_out: Option<bool>,
    peak_rss_bytes: Option<i64>,
    cpu_time_ms: Option<i64>,
    child_process_high_water: Option<i32>,
    input_bytes: Option<i64>,
    output_bytes: Option<i64>,
    resource_measurement_source: Option<String>,
    resource_measurement_window_ms: Option<i64>,
    actor_id: Option<Uuid>,
    correlation_id: Uuid,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct TaskRunEvidenceProjection {
    evidence_id: Uuid,
    evidence_kind: String,
    summary: Option<String>,
    sha256_digest: Option<String>,
    media_type: Option<String>,
    byte_length: Option<i64>,
    sensitivity: String,
    actor_id: Option<Uuid>,
    correlation_id: Uuid,
    captured_at: DateTime<Utc>,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/runs",
            get(list_task_runs),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/runs/{run_id}",
            get(get_task_run_detail),
        )
}

async fn list_task_runs(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    Query(query): Query<TaskRunListQuery>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let work_item_id = parse_id(&work_item_id)?;
    let limit = validate_run_limit(query.limit)?;
    let cursor = parse_run_cursor(query.cursor_started_at, query.cursor_run_id)?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let (scope, binding) =
        authorize_task_run_scope(&mut tx, &actor, worktree_id, work_item_id).await?;
    let rows = sqlx::query_as::<_, TaskRunSummary>(
        r#"
        SELECT run_id, worktree_id, repository_id, work_item_id, initiated_by,
               execution_channel, run_origin, start_ref, start_commit_ref,
               task_contract_version, agent_id, model_version, skill_version,
               orchestrator_version, strategy_version, execution_profile_id,
               execution_profile_version, execution_profile_digest,
               approved_launch_profile_id, approved_launch_profile_version,
               rtrim(approved_launch_profile_digest::text) AS approved_launch_profile_digest,
               rtrim(spawn_fence_binding_digest::text) AS spawn_fence_binding_digest,
               (SELECT e.execution_state FROM multica.task_execution_run_event e
                WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                  AND e.execution_state IS NOT NULL
                ORDER BY e.occurred_at DESC, e.event_id DESC LIMIT 1) AS execution_state,
               (SELECT e.verification_state FROM multica.task_execution_run_event e
                WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                  AND e.verification_state IS NOT NULL
                ORDER BY e.occurred_at DESC, e.event_id DESC LIMIT 1) AS verification_state,
               (SELECT e.human_acceptance_state FROM multica.task_execution_run_event e
                WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                  AND e.human_acceptance_state IS NOT NULL
                ORDER BY e.occurred_at DESC, e.event_id DESC LIMIT 1) AS human_acceptance_state,
               started_at, correlation_id
        FROM multica.task_execution_run r
        WHERE tenant_id = $1 AND project_id = $2 AND worktree_id = $3 AND work_item_id = $4
          AND repository_id = $5 AND branch_id = $6 AND engineering_run_id = $7
          AND ($8::timestamptz IS NULL OR (started_at, run_id) < ($8, $9::uuid))
        ORDER BY started_at DESC, run_id DESC
        LIMIT $10
        "#,
    )
    .bind(actor.tenant_id)
    .bind(scope.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .bind(scope.repository_id)
    .bind(scope.branch_id)
    .bind(scope.engineering_run_id)
    .bind(cursor.as_ref().map(|(started_at, _)| *started_at))
    .bind(cursor.as_ref().map(|(_, run_id)| *run_id))
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let has_more = rows.len() > limit as usize;
    let mut rows = rows;
    rows.truncate(limit as usize);
    let next_cursor = if has_more {
        rows.last().map(|row| {
            json!({
                "cursor_started_at": row.started_at,
                "cursor_run_id": row.run_id,
            })
        })
    } else {
        None
    };
    let runs = rows.into_iter().map(run_summary_json).collect::<Vec<_>>();
    Ok(no_store(Json(json!({
        "project_id": scope.project_id,
        "repository_id": scope.repository_id,
        "worktree_id": worktree_id,
        "work_item_id": work_item_id,
        "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
        "limit": limit,
        "runs": runs,
        "next_cursor": next_cursor,
    }))))
}

async fn get_task_run_detail(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, run_id)): Path<(String, String, String)>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let work_item_id = parse_id(&work_item_id)?;
    let run_id = parse_id(&run_id)?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let (scope, binding) =
        authorize_task_run_scope(&mut tx, &actor, worktree_id, work_item_id).await?;
    let run = sqlx::query_as::<_, TaskRunSummary>(
        r#"
        SELECT run_id, worktree_id, repository_id, work_item_id, initiated_by,
               execution_channel, run_origin, start_ref, start_commit_ref,
               task_contract_version, agent_id, model_version, skill_version,
               orchestrator_version, strategy_version, execution_profile_id,
               execution_profile_version, execution_profile_digest,
               approved_launch_profile_id, approved_launch_profile_version,
               rtrim(approved_launch_profile_digest::text) AS approved_launch_profile_digest,
               rtrim(spawn_fence_binding_digest::text) AS spawn_fence_binding_digest,
               (SELECT e.execution_state FROM multica.task_execution_run_event e
                WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                  AND e.execution_state IS NOT NULL
                ORDER BY e.occurred_at DESC, e.event_id DESC LIMIT 1) AS execution_state,
               (SELECT e.verification_state FROM multica.task_execution_run_event e
                WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                  AND e.verification_state IS NOT NULL
                ORDER BY e.occurred_at DESC, e.event_id DESC LIMIT 1) AS verification_state,
               (SELECT e.human_acceptance_state FROM multica.task_execution_run_event e
                WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                  AND e.human_acceptance_state IS NOT NULL
                ORDER BY e.occurred_at DESC, e.event_id DESC LIMIT 1) AS human_acceptance_state,
               started_at, correlation_id
        FROM multica.task_execution_run r
        WHERE tenant_id = $1 AND project_id = $2 AND worktree_id = $3
          AND work_item_id = $4 AND run_id = $5 AND repository_id = $6
          AND branch_id = $7 AND engineering_run_id = $8
        "#,
    )
    .bind(actor.tenant_id)
    .bind(scope.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .bind(run_id)
    .bind(scope.repository_id)
    .bind(scope.branch_id)
    .bind(scope.engineering_run_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;

    let mut events = sqlx::query_as::<_, TaskRunEventProjection>(
        r#"
        SELECT event_id, event_type, execution_state, agent_declaration_state,
               verification_state, human_acceptance_state, integration_state,
               failure_category, actual_cost_amount::text AS actual_cost_amount,
               estimated_cost_amount::text AS estimated_cost_amount, cost_unit,
               automation_occurrence_id, loop_iteration_no, loop_phase, loop_decision,
               stop_reason, hook_set_version, hook_rule_id, hook_rule_version,
               hook_evaluator_version, hook_digest, hook_phase, hook_decision,
               hook_reason_class, hook_duration_ms, hook_timed_out, peak_rss_bytes,
               cpu_time_ms, child_process_high_water, input_bytes, output_bytes,
               resource_measurement_source, resource_measurement_window_ms, actor_id,
               correlation_id, occurred_at
        FROM multica.task_execution_run_event
        WHERE tenant_id = $1 AND project_id = $2 AND work_item_id = $3 AND run_id = $4
        ORDER BY occurred_at DESC, event_id DESC
        LIMIT $5
        "#,
    )
    .bind(actor.tenant_id)
    .bind(scope.project_id)
    .bind(work_item_id)
    .bind(run_id)
    .bind(MAX_DETAIL_EVENTS + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let events_truncated = events.len() > MAX_DETAIL_EVENTS as usize;
    events.truncate(MAX_DETAIL_EVENTS as usize);
    events.reverse();

    let mut evidence = sqlx::query_as::<_, TaskRunEvidenceProjection>(
        r#"
        SELECT evidence_id, evidence_kind, summary, sha256_digest, media_type,
               byte_length, sensitivity, actor_id, correlation_id, captured_at
        FROM multica.task_execution_evidence
        WHERE tenant_id = $1 AND project_id = $2 AND work_item_id = $3 AND run_id = $4
        ORDER BY captured_at DESC, evidence_id DESC
        LIMIT $5
        "#,
    )
    .bind(actor.tenant_id)
    .bind(scope.project_id)
    .bind(work_item_id)
    .bind(run_id)
    .bind(MAX_DETAIL_EVIDENCE + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let evidence_truncated = evidence.len() > MAX_DETAIL_EVIDENCE as usize;
    evidence.truncate(MAX_DETAIL_EVIDENCE as usize);
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    Ok(no_store(Json(json!({
        "project_id": scope.project_id,
        "worktree_id": worktree_id,
        "work_item_id": work_item_id,
        "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
        "run": run_summary_json(run),
        "events": events.into_iter().map(run_event_json).collect::<Vec<_>>(),
        "events_truncated": events_truncated,
        "evidence": evidence.into_iter().map(run_evidence_json).collect::<Vec<_>>(),
        "evidence_truncated": evidence_truncated,
    }))))
}

async fn authorize_task_run_scope(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
    work_item_id: Uuid,
) -> Result<(TaskRunScope, super::ProjectBinding), GroupApiError> {
    let scope = sqlx::query_as::<_, TaskRunScope>(
        r#"
        SELECT p.project_id, w.repo_id AS repository_id, b.branch_id, b.engineering_run_id
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        JOIN multica.engineering_run_worktree_binding b
          ON b.tenant_id = w.tenant_id AND b.project_id = p.project_id
         AND b.repository_id = w.repo_id AND b.worktree_id = w.id
         AND b.project_binding_id = p.binding_id
         AND b.valid_from <= now() AND b.valid_to IS NULL
        JOIN multica.task_metadata m
          ON m.tenant_id = b.tenant_id AND m.project_id = b.project_id
         AND m.work_item_id = $3 AND m.valid_from <= now() AND m.valid_to IS NULL
         AND m.repository_id = b.repository_id AND m.branch_id = b.branch_id
         AND m.engineering_run_id = b.engineering_run_id
        JOIN multica.work_item_worktree l
          ON l.tenant_id = m.tenant_id AND l.project_id = m.project_id
         AND l.work_item_id = m.work_item_id AND l.worktree_id = w.id
         AND l.valid_from <= now() AND l.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        FOR SHARE OF w, p, b, m, l
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .bind(work_item_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let binding = active_binding(tx, actor, scope.project_id).await?;
    Ok((scope, binding))
}

fn run_summary_json(run: TaskRunSummary) -> Value {
    json!({
        "run_id": run.run_id,
        "worktree_id": run.worktree_id,
        "repository_id": run.repository_id,
        "work_item_id": run.work_item_id,
        "initiated_by": run.initiated_by,
        "execution_channel": run.execution_channel,
        "run_origin": run.run_origin,
        "start_ref": run.start_ref,
        "start_commit_ref": run.start_commit_ref,
        "task_contract_version": run.task_contract_version,
        "agent_id": run.agent_id,
        "model_version": run.model_version,
        "skill_version": run.skill_version,
        "orchestrator_version": run.orchestrator_version,
        "strategy_version": run.strategy_version,
        "execution_profile_id": run.execution_profile_id,
        "execution_profile_version": run.execution_profile_version,
        "execution_profile_digest": run.execution_profile_digest,
        "approved_launch_profile_id": run.approved_launch_profile_id,
        "approved_launch_profile_version": run.approved_launch_profile_version,
        "approved_launch_profile_digest": run.approved_launch_profile_digest,
        "spawn_fence_binding_digest": run.spawn_fence_binding_digest,
        "execution_state": run.execution_state,
        "verification_state": run.verification_state,
        "human_acceptance_state": run.human_acceptance_state,
        "started_at": run.started_at,
        "correlation_id": run.correlation_id,
    })
}

fn run_event_json(event: TaskRunEventProjection) -> Value {
    json!({
        "event_id": event.event_id,
        "event_type": event.event_type,
        "execution_state": event.execution_state,
        "agent_declaration_state": event.agent_declaration_state,
        "verification_state": event.verification_state,
        "human_acceptance_state": event.human_acceptance_state,
        "integration_state": event.integration_state,
        "failure_category": event.failure_category,
        "actual_cost_amount": event.actual_cost_amount,
        "estimated_cost_amount": event.estimated_cost_amount,
        "cost_unit": event.cost_unit,
        "automation_occurrence_id": event.automation_occurrence_id,
        "loop_iteration_no": event.loop_iteration_no,
        "loop_phase": event.loop_phase,
        "loop_decision": event.loop_decision,
        "stop_reason": event.stop_reason,
        "hook_set_version": event.hook_set_version,
        "hook_rule_id": event.hook_rule_id,
        "hook_rule_version": event.hook_rule_version,
        "hook_evaluator_version": event.hook_evaluator_version,
        "hook_digest": event.hook_digest,
        "hook_phase": event.hook_phase,
        "hook_decision": event.hook_decision,
        "hook_reason_class": event.hook_reason_class,
        "hook_duration_ms": event.hook_duration_ms,
        "hook_timed_out": event.hook_timed_out,
        "peak_rss_bytes": event.peak_rss_bytes,
        "cpu_time_ms": event.cpu_time_ms,
        "child_process_high_water": event.child_process_high_water,
        "input_bytes": event.input_bytes,
        "output_bytes": event.output_bytes,
        "resource_measurement_source": event.resource_measurement_source,
        "resource_measurement_window_ms": event.resource_measurement_window_ms,
        "actor_id": event.actor_id,
        "correlation_id": event.correlation_id,
        "occurred_at": event.occurred_at,
    })
}

fn run_evidence_json(evidence: TaskRunEvidenceProjection) -> Value {
    json!({
        "evidence_id": evidence.evidence_id,
        "evidence_kind": evidence.evidence_kind,
        "summary": evidence.summary,
        "sha256_digest": evidence.sha256_digest,
        "media_type": evidence.media_type,
        "byte_length": evidence.byte_length,
        "sensitivity": evidence.sensitivity,
        "actor_id": evidence.actor_id,
        "correlation_id": evidence.correlation_id,
        "captured_at": evidence.captured_at,
    })
}

fn validate_run_limit(limit: Option<i64>) -> Result<i64, GroupApiError> {
    match limit.unwrap_or(DEFAULT_RUN_LIMIT) {
        1..=MAX_RUN_LIMIT => Ok(limit.unwrap_or(DEFAULT_RUN_LIMIT)),
        _ => Err(GroupApiError::bad_request()),
    }
}

fn parse_run_cursor(
    started_at: Option<String>,
    run_id: Option<String>,
) -> Result<Option<(DateTime<Utc>, Uuid)>, GroupApiError> {
    match (started_at, run_id) {
        (None, None) => Ok(None),
        (Some(started_at), Some(run_id)) => {
            let started_at = DateTime::parse_from_rfc3339(&started_at)
                .map_err(|_| GroupApiError::bad_request())?
                .with_timezone(&Utc);
            let run_id = Uuid::parse_str(&run_id).map_err(|_| GroupApiError::bad_request())?;
            if run_id.is_nil() {
                return Err(GroupApiError::bad_request());
            }
            Ok(Some((started_at, run_id)))
        }
        _ => Err(GroupApiError::bad_request()),
    }
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value).map_err(|_| GroupApiError::bad_request())
}

fn no_store(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::PRAGMA,
        axum::http::HeaderValue::from_static("no-cache"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_limit_is_bounded() {
        assert_eq!(validate_run_limit(None).unwrap(), DEFAULT_RUN_LIMIT);
        assert_eq!(validate_run_limit(Some(1)).unwrap(), 1);
        assert_eq!(
            validate_run_limit(Some(MAX_RUN_LIMIT)).unwrap(),
            MAX_RUN_LIMIT
        );
        assert!(validate_run_limit(Some(0)).is_err());
        assert!(validate_run_limit(Some(MAX_RUN_LIMIT + 1)).is_err());
    }

    #[test]
    fn run_cursor_requires_a_valid_pair() {
        let run_id = Uuid::new_v4();
        assert!(parse_run_cursor(None, None).unwrap().is_none());
        assert!(parse_run_cursor(
            Some("2026-09-30T00:00:00Z".to_owned()),
            Some(run_id.to_string()),
        )
        .unwrap()
        .is_some());
        assert!(parse_run_cursor(Some("2026-09-30T00:00:00Z".to_owned()), None).is_err());
        assert!(
            parse_run_cursor(Some("no timestamp".to_owned()), Some(run_id.to_string())).is_err()
        );
        assert!(parse_run_cursor(
            Some("2026-09-30T00:00:00Z".to_owned()),
            Some(Uuid::nil().to_string()),
        )
        .is_err());
    }

    #[tokio::test]
    async fn task_run_routes_are_mounted_and_require_authentication() {
        use axum::{body::Body, http::Request};
        use sqlx::postgres::PgPoolOptions;
        use std::sync::Arc;
        use tower::ServiceExt;

        let jwt = Arc::new(crate::auth::JwtConfig {
            private_key_pem: String::new(),
            public_key_pem: String::new(),
            issuer: "https://api.star.local".to_owned(),
            audience: "star-api-rest".to_owned(),
            ttl_seconds: 3600,
        });
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://localhost/unavailable-for-auth-route-test")
            .expect("lazy test pool configuration should parse");
        let app = super::super::build_group_router(super::super::GroupApiState::new(jwt, pool));

        for path in [
            "/api/v1/worktrees/00000000-0000-0000-0000-000000000001/work-items/00000000-0000-0000-0000-000000000002/runs",
            "/api/v1/worktrees/00000000-0000-0000-0000-000000000001/work-items/00000000-0000-0000-0000-000000000002/runs/00000000-0000-0000-0000-000000000003",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
        }
    }
}

/* CYPHER STRUCTURAL MANIFEST ADDENDUM
MATCH (m:Module {name:"task_runs",type:"module",language:"rust"}),(router:Function {name:"task_runs::router",type:"function"});
CREATE (tests:Module {name:"task_run_tests",type:"module",language:"rust"}),(limit_test:Function {name:"run_limit_is_bounded",type:"function",language:"rust"}),(cursor_test:Function {name:"run_cursor_requires_a_valid_pair",type:"function",language:"rust"}),(route_test:Function {name:"task_run_routes_are_mounted_and_require_authentication",type:"function",language:"rust"});
CREATE (m)-[:CONTAINS]->(tests),(tests)-[:CONTAINS]->(limit_test),(tests)-[:CONTAINS]->(cursor_test),(tests)-[:CONTAINS]->(route_test),(route_test)-[:CALLS]->(router);
*/

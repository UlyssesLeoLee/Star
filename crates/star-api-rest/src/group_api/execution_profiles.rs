//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"execution_profiles.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"execution_profiles",type:"module",language:"rust"}),
//!   (router:Function {name:"router",type:"function",language:"rust"}),
//!   (list:Function {name:"list_execution_profiles",type:"function",language:"rust"}),
//!   (detail:Function {name:"get_execution_profile",type:"function",language:"rust"}),
//!   (authorize:Function {name:"authorize_worktree_profile_scope",type:"function",language:"rust"}),
//!   (verify:Function {name:"verify_profile_row",type:"function",language:"rust"}),
//!   (validate_limit:Function {name:"validate_profile_list_limit",type:"function",language:"rust"}),
//!   (parse_cursor:Function {name:"parse_profile_cursor",type:"function",language:"rust"}),
//!   (no_store:Function {name:"no_store",type:"function",language:"rust"}),
//!   (no_store_response:Function {name:"no_store_response",type:"function",language:"rust"}),
//!   (list_row:Class {name:"ExecutionProfileListRow",type:"class",language:"rust"}),
//!   (detail_row:Class {name:"ExecutionProfileDetailRow",type:"class",language:"rust"}),
//!   (summary:Class {name:"ExecutionProfileSummary",type:"class",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(router),(m)-[:CONTAINS]->(list),
//!   (m)-[:CONTAINS]->(detail),(m)-[:CONTAINS]->(authorize),(m)-[:CONTAINS]->(verify),
//!   (m)-[:CONTAINS]->(validate_limit),(m)-[:CONTAINS]->(parse_cursor),(m)-[:CONTAINS]->(no_store),(m)-[:CONTAINS]->(no_store_response),
//!   (m)-[:CONTAINS]->(list_row),(m)-[:CONTAINS]->(detail_row),(m)-[:CONTAINS]->(summary),
//!   (router)-[:CALLS]->(list),(router)-[:CALLS]->(detail),
//!   (list)-[:CALLS]->(authorize),(list)-[:CALLS]->(validate_limit),(list)-[:CALLS]->(parse_cursor),
//!   (detail)-[:CALLS]->(authorize),(detail)-[:CALLS]->(verify),
//!   (verify)-[:CALLS]->(detail_row);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"execution_profiles",type:"module"}),(router:Function {name:"router",type:"function"}),(limit:Function {name:"validate_profile_list_limit",type:"function"}),(cursor:Function {name:"parse_profile_cursor",type:"function"}),(verify:Function {name:"verify_profile_row",type:"function"}),(store:Function {name:"no_store",type:"function"}),(response_middleware:Function {name:"no_store_response",type:"function"});
//! CREATE (tests:Module {name:"tests",type:"module",language:"rust"}),(fixture:Function {name:"tests::valid_profile_document",type:"function",language:"rust"}),(test_limits:Function {name:"tests::profile_list_limit_is_bounded",type:"function",language:"rust"}),(test_cursor:Function {name:"tests::profile_cursor_requires_a_uuid",type:"function",language:"rust"}),(test_verify:Function {name:"tests::profile_detail_verifies_scope_and_digest",type:"function",language:"rust"}),(test_cache:Function {name:"tests::profile_responses_are_not_cached",type:"function",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(tests),(tests)-[:CONTAINS]->(fixture),(tests)-[:CONTAINS]->(test_limits),(tests)-[:CONTAINS]->(test_cursor),(tests)-[:CONTAINS]->(test_verify),(tests)-[:CONTAINS]->(test_cache),(test_limits)-[:CALLS]->(limit),(test_cursor)-[:CALLS]->(cursor),(fixture)-[:CALLS]->(verify),(test_verify)-[:CALLS]->(fixture),(test_verify)-[:CALLS]->(verify),(test_cache)-[:CALLS]->(store),(router)-[:USES]->(response_middleware),(response_middleware)-[:CALLS]->(store);

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue},
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use domain_agent::execution_profile::{AgentExecutionProfileDocument, ExecutionProfileScope};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, AuthenticatedUser, GroupApiError,
    GroupApiState,
};

const DEFAULT_PROFILE_LIMIT: i64 = 20;
const MAX_PROFILE_LIMIT: i64 = 50;

#[derive(Debug, Deserialize)]
struct ExecutionProfileListQuery {
    limit: Option<i64>,
    cursor: Option<String>,
}

#[derive(Debug, FromRow)]
struct ExecutionProfileListRow {
    profile_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    profile_version: i64,
    schema_version: i32,
    content_digest: String,
}

#[derive(Debug, FromRow)]
struct ExecutionProfileDetailRow {
    profile_revision_id: Uuid,
    project_id: Uuid,
    profile_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    profile_version: i64,
    schema_version: i32,
    content_digest: String,
    profile_document: Value,
}

#[derive(Debug, Serialize)]
struct ExecutionProfileSummary {
    profile_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    profile_version: i64,
    schema_version: i32,
    content_digest: String,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/worktrees/{worktree_id}/execution-profiles",
            get(list_execution_profiles),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/execution-profiles/{profile_id}",
            get(get_execution_profile),
        )
        .route_layer(map_response(no_store_response))
}

async fn list_execution_profiles(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    Query(query): Query<ExecutionProfileListQuery>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "worktree:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let limit = validate_profile_list_limit(query.limit)?;
    let cursor = parse_profile_cursor(query.cursor)?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let project_id = authorize_worktree_profile_scope(&mut tx, &actor, worktree_id).await?;
    let mut rows = sqlx::query_as::<_, ExecutionProfileListRow>(
        r#"
        SELECT profile_id, scope_kind, worktree_id, profile_version, schema_version,
               rtrim(content_digest::text) AS content_digest
        FROM multica.agent_execution_profile
        WHERE tenant_id = $1 AND project_id = $2 AND valid_to IS NULL
          AND lifecycle_state = 'active'
          AND (scope_kind = 'project' OR (scope_kind = 'worktree' AND worktree_id = $3))
          AND ($4::uuid IS NULL OR profile_id > $4)
        ORDER BY profile_id
        LIMIT $5
        "#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(worktree_id)
    .bind(cursor)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let has_more = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    let next_cursor = if has_more {
        rows.last().map(|row| row.profile_id)
    } else {
        None
    };
    let profiles = rows
        .into_iter()
        .map(|row| ExecutionProfileSummary {
            profile_id: row.profile_id,
            scope_kind: row.scope_kind,
            worktree_id: row.worktree_id,
            profile_version: row.profile_version,
            schema_version: row.schema_version,
            content_digest: row.content_digest,
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({
        "profiles": profiles,
        "next_cursor": next_cursor,
    }))
    .into_response())
}

async fn get_execution_profile(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, profile_id)): Path<(String, String)>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "worktree:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let profile_id = parse_id(&profile_id)?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let project_id = authorize_worktree_profile_scope(&mut tx, &actor, worktree_id).await?;
    let row = sqlx::query_as::<_, ExecutionProfileDetailRow>(
        r#"
        SELECT profile_revision_id, project_id, profile_id, scope_kind, worktree_id,
               profile_version, schema_version,
               rtrim(content_digest::text) AS content_digest, profile_document
        FROM multica.agent_execution_profile
        WHERE tenant_id = $1 AND project_id = $2 AND profile_id = $3
          AND valid_to IS NULL AND lifecycle_state = 'active'
          AND (scope_kind = 'project' OR (scope_kind = 'worktree' AND worktree_id = $4))
        "#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(profile_id)
    .bind(worktree_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let document = verify_profile_row(&row, actor.tenant_id, project_id, worktree_id)?;
    Ok(Json(json!({
        "profile_revision_id": row.profile_revision_id,
        "profile_id": row.profile_id,
        "scope_kind": row.scope_kind,
        "worktree_id": row.worktree_id,
        "profile_version": row.profile_version,
        "schema_version": row.schema_version,
        "content_digest": row.content_digest,
        "profile_document": document,
    }))
    .into_response())
}

async fn authorize_worktree_profile_scope(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
) -> Result<Uuid, GroupApiError> {
    let project_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT p.project_id
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        FOR SHARE OF w, p
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    active_binding(tx, actor, project_id).await?;
    Ok(project_id)
}

fn verify_profile_row(
    row: &ExecutionProfileDetailRow,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
) -> Result<AgentExecutionProfileDocument, GroupApiError> {
    if row.project_id != project_id || row.profile_version <= 0 || row.schema_version <= 0 {
        return Err(GroupApiError::internal());
    }
    match (row.scope_kind.as_str(), row.worktree_id) {
        ("project", None) | ("worktree", Some(_)) => {}
        _ => return Err(GroupApiError::internal()),
    }
    let encoded =
        serde_json::to_vec(&row.profile_document).map_err(|_| GroupApiError::internal())?;
    let verified = AgentExecutionProfileDocument::decode_and_verify(&encoded)
        .map_err(|_| GroupApiError::internal())?;
    let profile_scope = &verified.document().profile.scope;
    if profile_scope.tenant_id != tenant_id
        || profile_scope.project_id != project_id
        || profile_scope.worktree_id != row.worktree_id
        || verified.document().profile.schema_version != row.schema_version as u32
        || verified.document().content_digest != row.content_digest
    {
        return Err(GroupApiError::internal());
    }
    verified
        .validate_for_scope(&ExecutionProfileScope {
            tenant_id,
            project_id,
            worktree_id: Some(worktree_id),
        })
        .map_err(|_| GroupApiError::internal())?;
    Ok(verified.document().clone())
}

fn validate_profile_list_limit(limit: Option<i64>) -> Result<i64, GroupApiError> {
    let limit = limit.unwrap_or(DEFAULT_PROFILE_LIMIT);
    if !(1..=MAX_PROFILE_LIMIT).contains(&limit) {
        return Err(GroupApiError::bad_request());
    }
    Ok(limit)
}

fn parse_profile_cursor(cursor: Option<String>) -> Result<Option<Uuid>, GroupApiError> {
    cursor
        .map(|value| Uuid::parse_str(&value).map_err(|_| GroupApiError::bad_request()))
        .transpose()
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value).map_err(|_| GroupApiError::bad_request())
}

fn no_store<T: IntoResponse>(value: T) -> Response {
    let mut response = value.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn no_store_response(response: Response) -> Response {
    no_store(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_agent::execution_profile::{
        AgentExecutionProfileDraft, ContextPolicySnapshot, GrantSnapshot, HookSetSnapshot,
        LoopBudgetSnapshot, MemoryPolicySnapshot, ProviderReference, ResourceBudgetSnapshot,
        SkillBindingSnapshot, ValidationPolicySnapshot, EXECUTION_PROFILE_SCHEMA_VERSION,
    };

    fn digest(character: char) -> String {
        std::iter::repeat(character).take(64).collect()
    }

    fn provider(provider_id: &str, capabilities: &[&str]) -> ProviderReference {
        ProviderReference {
            provider_id: provider_id.to_owned(),
            version: 1,
            implementation_digest: digest('a'),
            configuration_digest: digest('b'),
            capabilities: capabilities
                .iter()
                .map(|capability| (*capability).to_owned())
                .collect(),
        }
    }

    fn valid_profile_document(worktree_id: Option<Uuid>) -> AgentExecutionProfileDocument {
        AgentExecutionProfileDraft {
            schema_version: EXECUTION_PROFILE_SCHEMA_VERSION,
            scope: ExecutionProfileScope {
                tenant_id: Uuid::from_u128(1),
                project_id: Uuid::from_u128(2),
                worktree_id,
            },
            agent_provider: provider("agent.local", &["tool.read"]),
            memory: MemoryPolicySnapshot::Disabled,
            skills: vec![SkillBindingSnapshot {
                skill_id: "review.rust".to_owned(),
                version: 1,
                content_digest: digest('c'),
                capabilities: vec!["tool.read".to_owned()],
            }],
            context: ContextPolicySnapshot {
                assembler: provider("context.assembler", &[]),
                max_input_bytes: 32_768,
                max_input_tokens: 8_192,
                max_sources: 32,
                compaction_policy_digest: digest('d'),
                preserve_task_contract: true,
                preserve_acceptance_criteria: true,
                preserve_authorization_scope: true,
            },
            validation: ValidationPolicySnapshot {
                provider: provider("validator.rust", &[]),
                suite_digest: digest('e'),
                toolchain_digest: digest('f'),
                acceptance_criteria: vec!["ac-compile".to_owned(), "ac-tests".to_owned()],
            },
            loop_budget: LoopBudgetSnapshot {
                policy: provider("loop.engineering", &[]),
                max_iterations: 8,
                max_no_progress_iterations: 2,
                max_wall_clock_ms: 120_000,
                max_provider_calls: 24,
            },
            resource_budget: ResourceBudgetSnapshot {
                max_rss_bytes: 1_073_741_824,
                max_cpu_ms: 300_000,
                max_runtime_ms: 180_000,
                max_child_processes: 8,
                max_parallel_tools: 4,
                max_provider_calls: 32,
                max_output_bytes: 2_097_152,
                max_event_buffer_bytes: 262_144,
            },
            hook_set: HookSetSnapshot {
                hook_set_id: Uuid::from_u128(3),
                version: 4,
                effective_digest: digest('1'),
            },
            grants: GrantSnapshot {
                grant_set_id: Uuid::from_u128(5),
                version: 2,
                capabilities: vec!["tool.read".to_owned()],
                expires_at_epoch_ms: 1_900_000_000_000,
            },
            engineering_manifest: None,
        }
        .seal()
        .expect("fixture profile must satisfy the domain contract")
        .document()
        .clone()
    }

    #[test]
    fn profile_list_limit_is_bounded() {
        assert_eq!(
            validate_profile_list_limit(None).unwrap(),
            DEFAULT_PROFILE_LIMIT
        );
        assert_eq!(validate_profile_list_limit(Some(1)).unwrap(), 1);
        assert_eq!(
            validate_profile_list_limit(Some(MAX_PROFILE_LIMIT)).unwrap(),
            MAX_PROFILE_LIMIT
        );
        assert!(validate_profile_list_limit(Some(0)).is_err());
        assert!(validate_profile_list_limit(Some(MAX_PROFILE_LIMIT + 1)).is_err());
    }

    #[test]
    fn profile_cursor_requires_a_uuid() {
        let cursor = Uuid::from_u128(7);
        assert_eq!(
            parse_profile_cursor(Some(cursor.to_string())).unwrap(),
            Some(cursor)
        );
        assert_eq!(parse_profile_cursor(None).unwrap(), None);
        assert!(parse_profile_cursor(Some("not-a-uuid".to_owned())).is_err());
    }

    #[test]
    fn profile_detail_verifies_scope_and_digest() {
        let tenant_id = Uuid::from_u128(1);
        let project_id = Uuid::from_u128(2);
        let worktree_id = Uuid::from_u128(4);
        let document = valid_profile_document(None);
        let row = ExecutionProfileDetailRow {
            profile_revision_id: Uuid::from_u128(8),
            project_id,
            profile_id: Uuid::from_u128(9),
            scope_kind: "project".to_owned(),
            worktree_id: None,
            profile_version: 1,
            schema_version: document.profile.schema_version as i32,
            content_digest: document.content_digest.clone(),
            profile_document: serde_json::to_value(&document).unwrap(),
        };
        assert_eq!(
            verify_profile_row(&row, tenant_id, project_id, worktree_id).unwrap(),
            document
        );

        let other_worktree = Uuid::from_u128(5);
        let scoped = valid_profile_document(Some(other_worktree));
        let scoped_row = ExecutionProfileDetailRow {
            profile_revision_id: Uuid::from_u128(10),
            project_id,
            profile_id: Uuid::from_u128(11),
            scope_kind: "worktree".to_owned(),
            worktree_id: Some(other_worktree),
            profile_version: 1,
            schema_version: scoped.profile.schema_version as i32,
            content_digest: scoped.content_digest.clone(),
            profile_document: serde_json::to_value(&scoped).unwrap(),
        };
        assert!(verify_profile_row(&scoped_row, tenant_id, project_id, worktree_id).is_err());

        let mut tampered = row;
        tampered.content_digest = digest('0');
        assert!(verify_profile_row(&tampered, tenant_id, project_id, worktree_id).is_err());
    }

    #[tokio::test]
    async fn profile_responses_are_not_cached() {
        let response = no_store(Json(json!({ "profiles": [] })));
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            &HeaderValue::from_static("no-store")
        );

        let error_response = no_store_response(GroupApiError::not_found().into_response()).await;
        assert_eq!(
            error_response.headers().get(header::CACHE_CONTROL).unwrap(),
            &HeaderValue::from_static("no-store")
        );
    }
}

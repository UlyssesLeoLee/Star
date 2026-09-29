//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"group_apps.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"group_apps",type:"module",language:"rust"}),
//!   (provider:Interface {name:"GroupAppRegistryProvider",type:"interface",language:"rust"}),
//!   (pgprovider:Class {name:"PgGroupAppRegistryProvider",type:"class",language:"rust"}),
//!   (query:Class {name:"GroupAppRegistryQuery",type:"class",language:"rust"}),
//!   (snapshot:Class {name:"GroupAppRegistrySnapshot",type:"class",language:"rust"}),
//!   (entry:Class {name:"GroupAppNavigationEntry",type:"class",language:"rust"}),
//!   (error:Enum {name:"GroupAppRegistryError",type:"enum",language:"rust"}),
//!   (router:Function {name:"router",type:"function",language:"rust"}),
//!   (list:Function {name:"list_group_apps",type:"function",language:"rust"}),
//!   (pgnew:Function {name:"PgGroupAppRegistryProvider::new",type:"function",language:"rust"}),
//!   (authorized:Function {name:"PgGroupAppRegistryProvider::authorized_projection",type:"function",language:"rust"}),
//!   (valid:Function {name:"validate_projection",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(provider),(m)-[:CONTAINS]->(query),
//!   (m)-[:CONTAINS]->(snapshot),(m)-[:CONTAINS]->(entry),(m)-[:CONTAINS]->(error),(m)-[:CONTAINS]->(pgprovider),
//!   (m)-[:CONTAINS]->(router),(m)-[:CONTAINS]->(list),(m)-[:CONTAINS]->(valid),
//!   (pgprovider)-[:HAS_METHOD]->(pgnew),(pgprovider)-[:HAS_METHOD]->(authorized),
//!   (pgprovider)-[:IMPLEMENTS]->(provider),
//!   (router)-[:CALLS]->(list),(list)-[:USES]->(provider),(list)-[:USES]->(query),
//!   (list)-[:CALLS]->(valid),(provider)-[:RETURNS]->(snapshot),(snapshot)-[:CONTAINS]->(entry);

use std::collections::HashSet;

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header::CACHE_CONTROL},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{AuthenticatedUser, GroupApiError, GroupApiState, require_scope, validate_actor};

const MAX_GROUP_APPS: usize = 100;

#[derive(Clone, Debug)]
pub struct GroupAppRegistryQuery {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub worktree_id: Uuid,
    pub actor_id: Uuid,
    pub membership_version: i64,
    pub correlation_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct GroupAppNavigationEntry {
    pub plugin_id: String,
    pub manifest_version: String,
    pub label: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug)]
pub struct GroupAppRegistrySnapshot {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub worktree_id: Uuid,
    pub membership_version: i64,
    pub registry_version: i64,
    pub entries: Vec<GroupAppNavigationEntry>,
}

#[derive(Clone, Copy, Debug)]
pub enum GroupAppRegistryError {
    Unavailable,
    Internal,
}

/// Trusted providers return only compatible, enabled apps with current grants. Implementations
/// must recheck membership and plugin grants in the registry read transaction, not reuse the
/// earlier context resolution as a durable authorization snapshot.
#[async_trait]
pub trait GroupAppRegistryProvider: Send + Sync {
    async fn authorized_projection(
        &self,
        query: GroupAppRegistryQuery,
    ) -> Result<GroupAppRegistrySnapshot, GroupAppRegistryError>;
}

/// PostgreSQL-backed, read-only projection over the Worktree Group App Registry.
/// Lifecycle writes and manifest trust verification are deliberately outside this provider.
#[derive(Clone)]
pub struct PgGroupAppRegistryProvider {
    pool: PgPool,
    host_api_version: i32,
}

impl PgGroupAppRegistryProvider {
    /// Bump only when a breaking Group App host surface change invalidates older manifests.
    pub const HOST_API_VERSION: i32 = 1;

    pub fn new(pool: PgPool, host_api_version: i32) -> Self {
        Self {
            pool,
            host_api_version,
        }
    }
}

#[async_trait]
impl GroupAppRegistryProvider for PgGroupAppRegistryProvider {
    async fn authorized_projection(
        &self,
        query: GroupAppRegistryQuery,
    ) -> Result<GroupAppRegistrySnapshot, GroupAppRegistryError> {
        if query.tenant_id.is_nil()
            || query.project_id.is_nil()
            || query.worktree_id.is_nil()
            || query.actor_id.is_nil()
            || self.host_api_version < 1
        {
            return Err(GroupAppRegistryError::Internal);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| GroupAppRegistryError::Unavailable)?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.actor_id', $2, true)",
        )
        .bind(query.tenant_id.to_string())
        .bind(query.actor_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupAppRegistryError::Unavailable)?;

        // Lock and compare the current membership version against the GroupContext resolved
        // immediately before this provider call. A revocation or role change fails closed.
        let current_membership_version = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT version
            FROM permission.project_role_binding
            WHERE tenant_id = $1 AND project_id = $2 AND user_id = $3
              AND valid_from <= now() AND valid_to IS NULL
            FOR SHARE
            "#,
        )
        .bind(query.tenant_id)
        .bind(query.project_id)
        .bind(query.actor_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| GroupAppRegistryError::Unavailable)?
        .ok_or(GroupAppRegistryError::Unavailable)?;
        if i64::from(current_membership_version) != query.membership_version {
            return Err(GroupAppRegistryError::Unavailable);
        }

        // The authoritative relation and non-archived Worktree are re-read under row locks.
        let worktree_binding = sqlx::query_as::<_, (Uuid, bool)>(
            r#"
            SELECT p.project_id, w.archived
            FROM worktree_canvas_worktree w
            JOIN multica.worktree_project_binding p
             ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
             AND p.project_id = w.project_id AND p.valid_from <= now() AND p.valid_to IS NULL
            WHERE w.tenant_id = $1 AND w.id = $2
            FOR SHARE OF w, p
            "#,
        )
        .bind(query.tenant_id)
        .bind(query.worktree_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| GroupAppRegistryError::Unavailable)?
        .ok_or(GroupAppRegistryError::Unavailable)?;
        if worktree_binding.0 != query.project_id || worktree_binding.1 {
            return Err(GroupAppRegistryError::Unavailable);
        }

        let registry_version = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT registry_version
            FROM plugin.group_app_registry_state
            WHERE tenant_id = $1 AND project_id = $2 AND worktree_id = $3
              AND valid_from <= now() AND valid_to IS NULL
            FOR SHARE
            "#,
        )
        .bind(query.tenant_id)
        .bind(query.project_id)
        .bind(query.worktree_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| GroupAppRegistryError::Unavailable)?
        .unwrap_or(0);

        let entries = sqlx::query_as::<_, (String, String, String, i32)>(
            r#"
            SELECT b.plugin_id, m.manifest_version, m.label, b.sort_order
            FROM plugin.group_app_binding b
            JOIN plugin.group_app_manifest m
             ON m.tenant_id = b.tenant_id AND m.manifest_id = b.manifest_id
             AND m.plugin_id = b.plugin_id
             AND m.valid_from <= now()
             AND m.valid_to IS NULL AND m.verification_status = 'verified'
             AND m.host_api_min <= $5 AND m.host_api_max >= $5
            JOIN plugin.group_app_access_grant g
              ON g.tenant_id = b.tenant_id AND g.binding_id = b.binding_id
             AND g.project_id = b.project_id AND g.worktree_id = b.worktree_id
             AND g.plugin_id = b.plugin_id AND g.actor_id = $4
             AND g.capability = 'group_app:open'
             AND g.valid_from <= now() AND g.valid_to IS NULL
             AND (g.expires_at IS NULL OR g.expires_at > now())
            WHERE b.tenant_id = $1 AND b.project_id = $2 AND b.worktree_id = $3
              AND b.valid_to IS NULL AND b.lifecycle_state = 'active'
              AND b.valid_from <= now()
            ORDER BY b.sort_order, b.plugin_id
            LIMIT 101
            "#,
        )
        .bind(query.tenant_id)
        .bind(query.project_id)
        .bind(query.worktree_id)
        .bind(query.actor_id)
        .bind(self.host_api_version)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| GroupAppRegistryError::Unavailable)?;
        if entries.len() > MAX_GROUP_APPS {
            return Err(GroupAppRegistryError::Internal);
        }

        tx.commit()
            .await
            .map_err(|_| GroupAppRegistryError::Unavailable)?;

        Ok(GroupAppRegistrySnapshot {
            tenant_id: query.tenant_id,
            project_id: query.project_id,
            worktree_id: query.worktree_id,
            membership_version: query.membership_version,
            registry_version,
            entries: entries
                .into_iter()
                .map(
                    |(plugin_id, manifest_version, label, sort_order)| GroupAppNavigationEntry {
                        plugin_id,
                        manifest_version,
                        label,
                        sort_order,
                    },
                )
                .collect(),
        })
    }
}

#[derive(Serialize)]
struct GroupAppRegistryResponse {
    worktree_id: Uuid,
    registry_version: i64,
    correlation_id: Uuid,
    apps: Vec<GroupAppNavigationEntry>,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new().route(
        "/api/v1/worktrees/{worktree_id}/group-apps",
        get(list_group_apps),
    )
}

async fn list_group_apps(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree): Path<String>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "worktree:read")?;
    let worktree_id = Uuid::parse_str(&worktree).map_err(|_| GroupApiError::bad_request())?;
    let context = state
        .resolver
        .resolve_worktree_context(&actor, worktree_id)
        .await?;
    let group_context = context
        .get("group_context")
        .ok_or_else(GroupApiError::internal)?;
    let project_id = read_uuid(group_context, "project_id")?;
    let membership_version = group_context
        .get("context_version")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(GroupApiError::internal)?;
    let correlation_id = read_uuid(group_context, "correlation_id")?;
    let provider = state
        .group_app_registry
        .as_ref()
        .ok_or_else(|| GroupApiError::feature_unavailable("group_app_registry_unavailable"))?;

    let snapshot = provider
        .authorized_projection(GroupAppRegistryQuery {
            tenant_id: actor.tenant_id,
            project_id,
            worktree_id,
            actor_id: actor.user_id,
            membership_version,
            correlation_id,
        })
        .await
        .map_err(|error| match error {
            GroupAppRegistryError::Unavailable => {
                GroupApiError::feature_unavailable("group_app_registry_unavailable")
            }
            GroupAppRegistryError::Internal => GroupApiError::internal(),
        })?;

    if snapshot.tenant_id != actor.tenant_id
        || snapshot.project_id != project_id
        || snapshot.worktree_id != worktree_id
        || snapshot.membership_version != membership_version
        || snapshot.registry_version < 0
        || snapshot.entries.len() > MAX_GROUP_APPS
    {
        return Err(GroupApiError::internal());
    }
    validate_projection(&snapshot.entries)?;
    let mut apps = snapshot.entries;
    apps.sort_by(|left, right| {
        left.sort_order
            .cmp(&right.sort_order)
            .then_with(|| left.plugin_id.cmp(&right.plugin_id))
    });

    let response = Json(GroupAppRegistryResponse {
        worktree_id,
        registry_version: snapshot.registry_version,
        correlation_id,
        apps,
    });
    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((headers, response).into_response())
}

fn read_uuid(value: &serde_json::Value, field: &str) -> Result<Uuid, GroupApiError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(GroupApiError::internal)
}

fn validate_projection(entries: &[GroupAppNavigationEntry]) -> Result<(), GroupApiError> {
    let mut plugin_ids = HashSet::with_capacity(entries.len());
    for entry in entries {
        let valid_id = !entry.plugin_id.is_empty()
            && entry.plugin_id.len() <= 64
            && entry.plugin_id.bytes().enumerate().all(|(index, byte)| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || (index > 0 && matches!(byte, b'-' | b'.' | b'_'))
            })
            && entry
                .plugin_id
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric);
        if !valid_id
            || !plugin_ids.insert(entry.plugin_id.as_str())
            || entry.manifest_version.trim().is_empty()
            || entry.manifest_version.len() > 128
            || entry.label.trim().is_empty()
            || entry.label.len() > 160
            || entry.label.chars().any(char::is_control)
        {
            return Err(GroupApiError::internal());
        }
    }
    Ok(())
}

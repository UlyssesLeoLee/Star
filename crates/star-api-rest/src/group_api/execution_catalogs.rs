//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"execution_catalogs.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"execution_catalogs",type:"module",language:"rust"}),
//!   (snapshot:Class {name:"CurrentExecutionAdmissionSnapshot",type:"class",language:"rust"}),
//!   (profile_row:Class {name:"ExecutionProfileCatalogRow",type:"class",language:"rust"}),
//!   (revision_row:Class {name:"CatalogRevisionRow",type:"class",language:"rust"}),
//!   (provider_row:Class {name:"ProviderCatalogRow",type:"class",language:"rust"}),
//!   (skill_row:Class {name:"SkillCatalogRow",type:"class",language:"rust"}),
//!   (grant_row:Class {name:"GrantCatalogRow",type:"class",language:"rust"}),
//!   (capability_row:Class {name:"CatalogCapabilityRow",type:"class",language:"rust"}),
//!   (load:Function {name:"load_current_execution_admission_snapshot",type:"function",language:"rust"}),
//!   (recheck:Function {name:"recheck_current_execution_admission_snapshot",type:"function",language:"rust"}),
//!   (load_profile:Function {name:"load_verified_profile",type:"function",language:"rust"}),
//!   (load_revisions:Function {name:"load_catalog_revisions",type:"function",language:"rust"}),
//!   (load_providers:Function {name:"load_provider_entries",type:"function",language:"rust"}),
//!   (load_skills:Function {name:"load_skill_entries",type:"function",language:"rust"}),
//!   (load_grants:Function {name:"load_grant_snapshot",type:"function",language:"rust"}),
//!   (skill_budget:Function {name:"ensure_skill_capability_heap_bound",type:"function",language:"rust"}),
//!   (references:Function {name:"required_catalog_references",type:"function",language:"rust"}),
//!   (group:Function {name:"group_capabilities",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(snapshot),(m)-[:CONTAINS]->(profile_row),
//!   (m)-[:CONTAINS]->(revision_row),(m)-[:CONTAINS]->(provider_row),
//!   (m)-[:CONTAINS]->(skill_row),(m)-[:CONTAINS]->(grant_row),(m)-[:CONTAINS]->(capability_row),
//!   (m)-[:CONTAINS]->(load),(m)-[:CONTAINS]->(recheck),(m)-[:CONTAINS]->(load_profile),
//!   (m)-[:CONTAINS]->(load_revisions),(m)-[:CONTAINS]->(load_providers),
//!   (m)-[:CONTAINS]->(load_skills),(m)-[:CONTAINS]->(load_grants),(m)-[:CONTAINS]->(skill_budget),
//!   (m)-[:CONTAINS]->(references),(m)-[:CONTAINS]->(group),
//!   (load)-[:CALLS]->(load_profile),(load)-[:CALLS]->(load_revisions),
//!   (load)-[:CALLS]->(load_providers),(load)-[:CALLS]->(load_skills),
//!   (load)-[:CALLS]->(load_grants),(load)-[:CALLS]->(references),(load)-[:CALLS]->(group),
//!   (load_skills)-[:CALLS]->(skill_budget),
//!   (recheck)-[:CALLS]->(load_profile),(recheck)-[:CALLS]->(load_revisions),
//!   (recheck)-[:CALLS]->(load_grants);
//! CYPHER STRUCTURAL MANIFEST END

use std::{collections::BTreeMap, sync::Arc};

use chrono::Utc;
use domain_agent::execution_profile::{
    AgentExecutionProfileDocument, CurrentExecutionCatalogSnapshot, ExecutionCatalogRevisions,
    ExecutionProfileResolver, ExecutionProfileRevisionIdentity, ExecutionProfileScope,
    ExecutionProviderCatalogEntry, ExecutionSkillCatalogEntry, GrantSnapshot, HookSetSnapshot,
    MemoryPolicySnapshot, ProviderReference, VerifiedAgentExecutionProfile, WorktreeRunState,
    EXECUTION_CATALOG_FENCE_MAX_TTL_MS, EXECUTION_CATALOG_FENCE_MIN_REMAINING_MS,
};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::GroupApiError;

const MAX_PROFILE_PROVIDER_REFERENCES: usize = 5;
const MAX_PROFILE_SKILL_REFERENCES: usize = 128;
const MAX_CATALOG_CAPABILITIES: usize = 64;
// Provider and GrantSet capability heaps are already bounded by their 5/64 and 64/64 caps.
// Reserve 384 KiB for Skill labels, including per-label String/association overhead, before fetch.
const MAX_SKILL_CAPABILITY_HEAP_BYTES: i64 = 393_216;

/// Immutable Profile and current catalog facts shared with Runtime readiness.
#[derive(Clone, Debug)]
pub struct CurrentExecutionAdmissionSnapshot {
    /// Exact Tenant/Project/Worktree request scope.
    pub scope: ExecutionProfileScope,
    /// Current persisted Profile identity selected by the Task Card.
    pub profile_identity: ExecutionProfileRevisionIdentity,
    /// Verified current Profile document.
    pub verified_profile: Arc<VerifiedAgentExecutionProfile>,
    /// Bounded current Provider/Skill/Grant snapshot with its short revision fence.
    pub catalogs: Arc<CurrentExecutionCatalogSnapshot>,
    /// Effective current HookSet identity used to resolve the Profile.
    pub hook_set: HookSetSnapshot,
}

#[derive(Debug, FromRow)]
struct ExecutionProfileCatalogRow {
    profile_revision_id: Uuid,
    project_id: Uuid,
    profile_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    profile_version: i64,
    schema_version: i16,
    content_digest: String,
    profile_document: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct CatalogRevisionRow {
    provider_catalog_revision: i64,
    skill_catalog_revision: i64,
}

#[derive(Debug, FromRow)]
struct ProviderCatalogRow {
    entry_id: Uuid,
    provider_id: String,
    provider_version: i64,
    implementation_digest: String,
    configuration_digest: String,
    capability_count: i16,
    available: bool,
}

#[derive(Debug, FromRow)]
struct SkillCatalogRow {
    entry_id: Uuid,
    skill_id: String,
    skill_version: i64,
    content_digest: String,
    capability_count: i16,
    available: bool,
}

#[derive(Debug, FromRow)]
struct GrantCatalogRow {
    grant_set_id: Uuid,
    grant_set_version: i64,
    capability_count: i16,
    expires_at_epoch_ms: i64,
}

#[derive(Debug, FromRow)]
struct CatalogCapabilityRow {
    entry_id: Uuid,
    capability_code: String,
}

/// Load an authorized Profile and only the catalog rows that Profile references.
pub(super) async fn load_current_execution_admission_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    profile_id: Uuid,
) -> Result<Arc<CurrentExecutionAdmissionSnapshot>, GroupApiError> {
    let observed_at_ms = current_epoch_ms()?;
    let scope = ExecutionProfileScope {
        tenant_id,
        project_id,
        worktree_id: Some(worktree_id),
    };
    let (verified_profile, profile_identity) =
        load_verified_profile(tx, &scope, profile_id).await?;
    let (provider_references, skill_references) =
        required_catalog_references(verified_profile.document())?;
    let providers = load_provider_entries(tx, tenant_id, project_id, &provider_references).await?;
    let skills = load_skill_entries(tx, tenant_id, project_id, &skill_references).await?;
    let current_grants = load_grant_snapshot(tx, tenant_id, project_id).await?;
    let revisions = load_catalog_revisions(tx, tenant_id, project_id, &current_grants).await?;
    let expires_at_ms = observed_at_ms
        .checked_add(EXECUTION_CATALOG_FENCE_MAX_TTL_MS)
        .ok_or_else(GroupApiError::internal)?;
    let catalogs = Arc::new(
        CurrentExecutionCatalogSnapshot::for_profile(
            scope.clone(),
            profile_identity.clone(),
            &verified_profile,
            revisions,
            observed_at_ms,
            expires_at_ms,
            current_grants,
            Arc::from(providers),
            Arc::from(skills),
        )
        .map_err(|_| GroupApiError::conflict("execution_catalog_snapshot_invalid"))?,
    );
    let Some((_, hook_set)) = super::hook_policies::load_verified_effective_run_snapshot(
        tx,
        tenant_id,
        project_id,
        worktree_id,
    )
    .await?
    else {
        return Err(GroupApiError::feature_unavailable(
            "execution_profile_hook_set_unavailable",
        ));
    };
    if verified_profile.document().profile.hook_set != hook_set {
        return Err(GroupApiError::conflict(
            "execution_profile_hook_set_changed",
        ));
    }
    let now_ms = current_epoch_ms()?;
    ExecutionProfileResolver
        .resolve_current_snapshot(
            &verified_profile,
            &profile_identity,
            &scope,
            revisions,
            &hook_set,
            WorktreeRunState::Active,
            now_ms,
            &catalogs,
        )
        .map_err(|_| GroupApiError::conflict("execution_profile_dependencies_changed"))?;

    Ok(Arc::new(CurrentExecutionAdmissionSnapshot {
        scope,
        profile_identity,
        verified_profile: Arc::new(verified_profile),
        catalogs,
        hook_set,
    }))
}

/// Lock and compare current Profile, catalog revision, and GrantSet facts inside Run admission.
pub(super) async fn recheck_current_execution_admission_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    snapshot: &CurrentExecutionAdmissionSnapshot,
    effective_hook_set: &HookSetSnapshot,
) -> Result<(), GroupApiError> {
    let now_ms = current_epoch_ms()?;
    let (current_profile, current_identity) =
        load_verified_profile(tx, &snapshot.scope, snapshot.profile_identity.profile_id).await?;
    if current_identity != snapshot.profile_identity
        || current_profile.document().content_digest
            != snapshot.verified_profile.document().content_digest
        || effective_hook_set != &snapshot.hook_set
    {
        return Err(GroupApiError::conflict(
            "execution_admission_snapshot_changed",
        ));
    }
    let current_grant =
        load_grant_snapshot(tx, snapshot.scope.tenant_id, snapshot.scope.project_id).await?;
    let revisions = load_catalog_revisions(
        tx,
        snapshot.scope.tenant_id,
        snapshot.scope.project_id,
        &current_grant,
    )
    .await?;
    if current_grant != *snapshot.catalogs.current_grants()
        || now_ms >= current_grant.expires_at_epoch_ms
    {
        return Err(GroupApiError::conflict("execution_grant_set_changed"));
    }
    snapshot
        .catalogs
        .validate_for(
            &snapshot.scope,
            &snapshot.profile_identity,
            revisions,
            now_ms,
            EXECUTION_CATALOG_FENCE_MIN_REMAINING_MS,
        )
        .map_err(|_| GroupApiError::conflict("execution_catalog_revision_changed"))?;
    ExecutionProfileResolver
        .resolve_current_snapshot(
            &snapshot.verified_profile,
            &snapshot.profile_identity,
            &snapshot.scope,
            revisions,
            effective_hook_set,
            WorktreeRunState::Active,
            now_ms,
            &snapshot.catalogs,
        )
        .map_err(|_| GroupApiError::conflict("execution_profile_dependencies_changed"))?;
    Ok(())
}

async fn load_verified_profile(
    tx: &mut Transaction<'_, Postgres>,
    scope: &ExecutionProfileScope,
    profile_id: Uuid,
) -> Result<
    (
        VerifiedAgentExecutionProfile,
        ExecutionProfileRevisionIdentity,
    ),
    GroupApiError,
> {
    let row = sqlx::query_as::<_, ExecutionProfileCatalogRow>(
        r#"
        SELECT profile_revision_id, project_id, profile_id, scope_kind, worktree_id,
               profile_version, schema_version,
               rtrim(content_digest::text) AS content_digest, profile_document
        FROM multica.agent_execution_profile
        WHERE tenant_id = $1 AND project_id = $2 AND profile_id = $3
          AND valid_to IS NULL AND lifecycle_state = 'active'
          AND (scope_kind = 'project' OR (scope_kind = 'worktree' AND worktree_id = $4))
        FOR SHARE
        "#,
    )
    .bind(scope.tenant_id)
    .bind(scope.project_id)
    .bind(profile_id)
    .bind(scope.worktree_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::feature_unavailable("execution_profile_unavailable"))?
    .ok_or_else(|| GroupApiError::conflict("execution_profile_unavailable"))?;
    verify_profile_row(&row, scope)
}

fn verify_profile_row(
    row: &ExecutionProfileCatalogRow,
    scope: &ExecutionProfileScope,
) -> Result<
    (
        VerifiedAgentExecutionProfile,
        ExecutionProfileRevisionIdentity,
    ),
    GroupApiError,
> {
    if row.profile_revision_id.is_nil()
        || row.project_id != scope.project_id
        || row.profile_id.is_nil()
        || row.profile_version <= 0
        || row.schema_version <= 0
        || !matches!(
            (row.scope_kind.as_str(), row.worktree_id),
            ("project", None) | ("worktree", Some(_))
        )
    {
        return Err(GroupApiError::conflict("execution_profile_invalid"));
    }
    let encoded =
        serde_json::to_vec(&row.profile_document).map_err(|_| GroupApiError::internal())?;
    let verified = AgentExecutionProfileDocument::decode_and_verify(&encoded)
        .map_err(|_| GroupApiError::conflict("execution_profile_invalid"))?;
    let document = verified.document();
    if document.profile.scope.tenant_id != scope.tenant_id
        || document.profile.scope.project_id != scope.project_id
        || document.profile.scope.worktree_id != row.worktree_id
        || document.profile.schema_version != u32::try_from(row.schema_version).unwrap_or_default()
        || document.content_digest != row.content_digest
    {
        return Err(GroupApiError::conflict("execution_profile_invalid"));
    }
    verified
        .validate_for_scope(scope)
        .map_err(|_| GroupApiError::conflict("execution_profile_scope_changed"))?;
    let profile_identity = ExecutionProfileRevisionIdentity {
        profile_id: row.profile_id,
        version: u64::try_from(row.profile_version)
            .map_err(|_| GroupApiError::conflict("execution_profile_invalid"))?,
        content_digest: row.content_digest.clone(),
    };
    Ok((verified, profile_identity))
}

async fn load_catalog_revisions(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    current_grants: &GrantSnapshot,
) -> Result<ExecutionCatalogRevisions, GroupApiError> {
    let row = sqlx::query_as::<_, CatalogRevisionRow>(
        r#"SELECT provider_catalog_revision, skill_catalog_revision
           FROM multica.agent_execution_catalog_revision
           WHERE tenant_id = $1 AND project_id = $2
           FOR SHARE"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| catalog_unavailable())?
    .ok_or_else(catalog_unavailable)?;
    Ok(ExecutionCatalogRevisions {
        provider_catalog_revision: u64::try_from(row.provider_catalog_revision)
            .map_err(|_| catalog_unavailable())?,
        skill_catalog_revision: u64::try_from(row.skill_catalog_revision)
            .map_err(|_| catalog_unavailable())?,
        grant_set_id: current_grants.grant_set_id,
        grant_set_version: current_grants.version,
    })
}

async fn load_provider_entries(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    references: &[(String, u32)],
) -> Result<Vec<ExecutionProviderCatalogEntry>, GroupApiError> {
    if references.is_empty() || references.len() > MAX_PROFILE_PROVIDER_REFERENCES {
        return Err(GroupApiError::conflict("execution_profile_provider_limit"));
    }
    let ids = references
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let rows = sqlx::query_as::<_, ProviderCatalogRow>(
        r#"SELECT entry_id, provider_id, provider_version,
                  rtrim(implementation_digest::text) AS implementation_digest,
                  rtrim(configuration_digest::text) AS configuration_digest,
                  capability_count, available
           FROM multica.agent_execution_provider_catalog
           WHERE tenant_id = $1 AND project_id = $2 AND provider_id::text = ANY($3::text[])
             AND valid_to IS NULL
           ORDER BY provider_id"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(|_| catalog_unavailable())?;
    if rows.len() > references.len() {
        return Err(catalog_unavailable());
    }
    let entry_ids = rows.iter().map(|row| row.entry_id).collect::<Vec<_>>();
    let mut capabilities = load_capabilities(
        tx,
        tenant_id,
        &entry_ids,
        "agent_execution_provider_capability",
    )
    .await?;
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let version = u32::try_from(row.provider_version).map_err(|_| catalog_unavailable())?;
        let labels = take_capabilities(&mut capabilities, row.entry_id, row.capability_count)?;
        entries.push(ExecutionProviderCatalogEntry {
            provider: ProviderReference {
                provider_id: row.provider_id,
                version,
                implementation_digest: row.implementation_digest,
                configuration_digest: row.configuration_digest,
                capabilities: labels,
            },
            available: row.available,
        });
    }
    if !capabilities.is_empty() {
        return Err(catalog_unavailable());
    }
    Ok(entries)
}

async fn load_skill_entries(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    references: &[(String, u32)],
) -> Result<Vec<ExecutionSkillCatalogEntry>, GroupApiError> {
    if references.len() > MAX_PROFILE_SKILL_REFERENCES {
        return Err(GroupApiError::conflict("execution_profile_skill_limit"));
    }
    if references.is_empty() {
        return Ok(Vec::new());
    }
    let ids = references
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let rows = sqlx::query_as::<_, SkillCatalogRow>(
        r#"SELECT entry_id, skill_id, skill_version,
                  rtrim(content_digest::text) AS content_digest, capability_count, available
           FROM multica.agent_execution_skill_catalog
           WHERE tenant_id = $1 AND project_id = $2 AND skill_id::text = ANY($3::text[])
             AND valid_to IS NULL
           ORDER BY skill_id"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(|_| catalog_unavailable())?;
    if rows.len() > references.len() {
        return Err(catalog_unavailable());
    }
    let entry_ids = rows.iter().map(|row| row.entry_id).collect::<Vec<_>>();
    ensure_skill_capability_heap_bound(tx, tenant_id, &entry_ids).await?;
    let mut capabilities = load_capabilities(
        tx,
        tenant_id,
        &entry_ids,
        "agent_execution_skill_capability",
    )
    .await?;
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let version = u32::try_from(row.skill_version).map_err(|_| catalog_unavailable())?;
        let labels = take_capabilities(&mut capabilities, row.entry_id, row.capability_count)?;
        entries.push(ExecutionSkillCatalogEntry {
            skill: domain_agent::execution_profile::SkillBindingSnapshot {
                skill_id: row.skill_id,
                version,
                content_digest: row.content_digest,
                capabilities: labels,
            },
            available: row.available,
        });
    }
    if !capabilities.is_empty() {
        return Err(catalog_unavailable());
    }
    Ok(entries)
}

async fn ensure_skill_capability_heap_bound(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    entry_ids: &[Uuid],
) -> Result<(), GroupApiError> {
    if entry_ids.is_empty() {
        return Ok(());
    }
    let estimated_heap_bytes = sqlx::query_scalar::<_, i64>(
        r#"SELECT COALESCE(SUM(octet_length(capability_code)::BIGINT + 64), 0)
           FROM multica.agent_execution_skill_capability
           WHERE tenant_id = $1 AND entry_id = ANY($2::uuid[])"#,
    )
    .bind(tenant_id)
    .bind(entry_ids)
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| catalog_unavailable())?;
    if !(0..=MAX_SKILL_CAPABILITY_HEAP_BYTES).contains(&estimated_heap_bytes) {
        return Err(GroupApiError::conflict("execution_catalog_size_limit"));
    }
    Ok(())
}

async fn load_grant_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
) -> Result<GrantSnapshot, GroupApiError> {
    let row = sqlx::query_as::<_, GrantCatalogRow>(
        r#"SELECT grant_set_id, grant_set_version, capability_count, expires_at_epoch_ms
           FROM multica.agent_execution_grant_set
           WHERE tenant_id = $1 AND project_id = $2 AND valid_to IS NULL
           FOR SHARE"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| catalog_unavailable())?
    .ok_or_else(catalog_unavailable)?;
    let rows = sqlx::query_as::<_, CatalogCapabilityRow>(
        r#"SELECT entry_id, capability_code
           FROM multica.agent_execution_grant_capability
           WHERE tenant_id = $1 AND entry_id = (
                SELECT entry_id FROM multica.agent_execution_grant_set
                WHERE tenant_id = $1 AND project_id = $2 AND valid_to IS NULL
           )
           ORDER BY capability_code"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|_| catalog_unavailable())?;
    let capabilities = rows
        .into_iter()
        .map(|row| row.capability_code)
        .collect::<Vec<_>>();
    if row.capability_count < 0
        || usize::try_from(row.capability_count).ok() != Some(capabilities.len())
        || capabilities.len() > MAX_CATALOG_CAPABILITIES
    {
        return Err(catalog_unavailable());
    }
    Ok(GrantSnapshot {
        grant_set_id: row.grant_set_id,
        version: u64::try_from(row.grant_set_version).map_err(|_| catalog_unavailable())?,
        capabilities,
        expires_at_epoch_ms: u64::try_from(row.expires_at_epoch_ms)
            .map_err(|_| catalog_unavailable())?,
    })
}

async fn load_capabilities(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    entry_ids: &[Uuid],
    table: &'static str,
) -> Result<BTreeMap<Uuid, Vec<String>>, GroupApiError> {
    if entry_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let query = match table {
        "agent_execution_provider_capability" => {
            "SELECT entry_id, capability_code FROM multica.agent_execution_provider_capability WHERE tenant_id=$1 AND entry_id=ANY($2::uuid[]) ORDER BY entry_id, capability_code"
        }
        "agent_execution_skill_capability" => {
            "SELECT entry_id, capability_code FROM multica.agent_execution_skill_capability WHERE tenant_id=$1 AND entry_id=ANY($2::uuid[]) ORDER BY entry_id, capability_code"
        }
        _ => return Err(GroupApiError::internal()),
    };
    let rows = sqlx::query_as::<_, CatalogCapabilityRow>(query)
        .bind(tenant_id)
        .bind(entry_ids)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| catalog_unavailable())?;
    let mut grouped = BTreeMap::<Uuid, Vec<String>>::new();
    for row in rows {
        let entry = grouped.entry(row.entry_id).or_default();
        if entry.len() >= MAX_CATALOG_CAPABILITIES {
            return Err(catalog_unavailable());
        }
        entry.push(row.capability_code);
    }
    Ok(grouped)
}

fn take_capabilities(
    capabilities: &mut BTreeMap<Uuid, Vec<String>>,
    entry_id: Uuid,
    expected_count: i16,
) -> Result<Vec<String>, GroupApiError> {
    if expected_count < 0 {
        return Err(catalog_unavailable());
    }
    let expected_count = usize::try_from(expected_count).map_err(|_| catalog_unavailable())?;
    if expected_count > MAX_CATALOG_CAPABILITIES {
        return Err(catalog_unavailable());
    }
    let labels = capabilities.remove(&entry_id).unwrap_or_default();
    if labels.len() != expected_count {
        return Err(catalog_unavailable());
    }
    Ok(labels)
}

fn required_catalog_references(
    document: &AgentExecutionProfileDocument,
) -> Result<(Vec<(String, u32)>, Vec<(String, u32)>), GroupApiError> {
    let draft = &document.profile;
    let mut providers = vec![
        (
            draft.agent_provider.provider_id.clone(),
            draft.agent_provider.version,
        ),
        (
            draft.context.assembler.provider_id.clone(),
            draft.context.assembler.version,
        ),
        (
            draft.validation.provider.provider_id.clone(),
            draft.validation.provider.version,
        ),
        (
            draft.loop_budget.policy.provider_id.clone(),
            draft.loop_budget.policy.version,
        ),
    ];
    if let MemoryPolicySnapshot::Enabled { provider, .. } = &draft.memory {
        providers.push((provider.provider_id.clone(), provider.version));
    }
    providers.sort_unstable();
    providers.dedup();
    if providers.len() > MAX_PROFILE_PROVIDER_REFERENCES
        || providers.windows(2).any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(GroupApiError::conflict("execution_profile_provider_limit"));
    }
    let mut skills = draft
        .skills
        .iter()
        .map(|skill| (skill.skill_id.clone(), skill.version))
        .collect::<Vec<_>>();
    skills.sort_unstable();
    skills.dedup();
    if skills.len() > MAX_PROFILE_SKILL_REFERENCES
        || skills.windows(2).any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(GroupApiError::conflict("execution_profile_skill_limit"));
    }
    Ok((providers, skills))
}

fn current_epoch_ms() -> Result<u64, GroupApiError> {
    u64::try_from(Utc::now().timestamp_millis()).map_err(|_| GroupApiError::internal())
}

fn catalog_unavailable() -> GroupApiError {
    GroupApiError::feature_unavailable("execution_catalog_provider_unavailable")
}

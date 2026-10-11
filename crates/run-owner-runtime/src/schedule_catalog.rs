//! Bounded current referenced catalogs and immutable short-lived execution preflight.
/*
@cypher schema=1 source_sha256=9107ee7d2115d0495980f7a82daee1216752b796b30a9e52ab253f4084732768
MERGE (self:File {path:"crates/run-owner-runtime/src/schedule_catalog.rs"})
MERGE (load:Symbol {id:"crates/run-owner-runtime/src/schedule_catalog.rs::load_current",kind:"function"})
MERGE (snapshot:Type {id:"run_owner_runtime::ScheduleExecutionSnapshot",kind:"struct"})
MERGE (self)-[:DEFINES]->(load)
MERGE (self)-[:DEFINES]->(snapshot)
MERGE (load)-[:CALLS]->(read:Symbol {id:"db/migrations/2026-10-10-run-owner-catalog-currentness.sql::read_schedule_catalog_current",kind:"function"})
MERGE (load)-[:CALLS]->(resolve:Symbol {id:"crates/domain-agent/src/execution_profile.rs::ExecutionProfileResolver.resolve_current_snapshot",kind:"method"})
MERGE (load)-[:CALLS]->(build:Symbol {id:"crates/domain-agent/src/execution_profile.rs::CurrentExecutionCatalogSnapshot.for_profile",kind:"method"})
MERGE (self)-[:IMPORTS]->(profile_module:File {path:"crates/domain-agent/src/execution_profile.rs"})
@endcypher
*/
use std::sync::Arc;

use domain_agent::execution_profile::{
    CurrentExecutionCatalogSnapshot, ExecutionCatalogRevisions, ExecutionProfileResolver,
    ExecutionProfileRevisionIdentity, ExecutionProfileScope, ExecutionProviderCatalogEntry,
    ExecutionSkillCatalogEntry, GrantSnapshot, MemoryPolicySnapshot, ProviderReference,
    SkillBindingSnapshot, WorktreeRunState, EXECUTION_CATALOG_FENCE_MAX_TTL_MS,
    EXECUTION_CATALOG_FENCE_MIN_REMAINING_MS,
};
use domain_automation::schedule::{
    ScheduleAdmissionRejectionCodeV1, ScheduleRunAdmissionCommandV1,
};
use serde::Deserialize;
use sqlx::{Postgres, Transaction};

use crate::{ScheduleAdmissionStoreError, SchedulePolicySnapshot};

/// Current preflight snapshot; no runtime/Hook/quota permit can be constructed from it.
pub struct ScheduleExecutionSnapshot {
    policies: SchedulePolicySnapshot,
    catalogs: CurrentExecutionCatalogSnapshot,
    scope: ExecutionProfileScope,
    identity: ExecutionProfileRevisionIdentity,
    revisions: ExecutionCatalogRevisions,
}
impl ScheduleExecutionSnapshot {
    /// Borrow the current verified policy pair.
    pub fn policies(&self) -> &SchedulePolicySnapshot {
        &self.policies
    }
    /// Borrow the bounded current catalogs and their fence.
    pub fn catalogs(&self) -> &CurrentExecutionCatalogSnapshot {
        &self.catalogs
    }
    /// Recheck time-sensitive grants and the fence before later owner writes.
    /// Source locks are only authoritative while the originating transaction is alive.
    pub fn validate_lifetime(&self) -> Result<(), ScheduleAdmissionStoreError> {
        let now = epoch_ms()?;
        ExecutionProfileResolver
            .resolve_current_snapshot(
                self.policies.profile(),
                &self.identity,
                &self.scope,
                self.revisions,
                &self.policies.profile().document().profile.hook_set,
                WorktreeRunState::Active,
                now,
                &self.catalogs,
            )
            .map_err(|_| rejected())?;
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEntry {
    provider: ProviderReference,
    available: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillEntry {
    skill: SkillBindingSnapshot,
    available: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogEnvelope {
    providers: Vec<ProviderEntry>,
    skills: Vec<SkillEntry>,
    grants: GrantSnapshot,
    provider_revision: u64,
    skill_revision: u64,
}
fn rejected() -> ScheduleAdmissionStoreError {
    ScheduleAdmissionStoreError::PrerequisiteRejected(
        ScheduleAdmissionRejectionCodeV1::TargetUnavailable,
    )
}
fn epoch_ms() -> Result<u64, ScheduleAdmissionStoreError> {
    u64::try_from(chrono::Utc::now().timestamp_millis()).map_err(|_| rejected())
}
pub(crate) async fn load_current(
    tx: &mut Transaction<'_, Postgres>,
    command: &ScheduleRunAdmissionCommandV1,
    policies: SchedulePolicySnapshot,
) -> Result<ScheduleExecutionSnapshot, ScheduleAdmissionStoreError> {
    let observed = epoch_ms()?;
    let draft = &policies.profile().document().profile;
    let mut references = vec![
        &draft.agent_provider,
        &draft.context.assembler,
        &draft.validation.provider,
        &draft.loop_budget.policy,
    ];
    if let MemoryPolicySnapshot::Enabled { provider, .. } = &draft.memory {
        references.push(provider);
    }
    references.sort_by(|a, b| (&a.provider_id, a.version).cmp(&(&b.provider_id, b.version)));
    references.dedup_by(|a, b| a.provider_id == b.provider_id && a.version == b.version);
    if references.len() > 5
        || references
            .windows(2)
            .any(|v| v[0].provider_id == v[1].provider_id)
    {
        return Err(rejected());
    }
    let provider_ids = references
        .iter()
        .map(|p| p.provider_id.clone())
        .collect::<Vec<_>>();
    let skill_ids = draft
        .skills
        .iter()
        .map(|s| s.skill_id.clone())
        .collect::<Vec<_>>();
    if skill_ids.len() > 128 {
        return Err(rejected());
    }
    let t = &command.occurrence.rule_snapshot.target;
    let wire: Option<String> = sqlx::query_scalar(
        "SELECT multica.read_schedule_catalog_current($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(command.tenant_id)
    .bind(command.occurrence.run_as_actor_id)
    .bind(t.project_id)
    .bind(t.branch_id)
    .bind(t.engineering_run_id)
    .bind(t.repository_id)
    .bind(t.worktree_id)
    .bind(t.work_item_id)
    .bind(&provider_ids)
    .bind(&skill_ids)
    .fetch_one(&mut **tx)
    .await?;
    let wire = wire.ok_or_else(rejected)?;
    if wire.len() > 524_288 {
        return Err(rejected());
    }
    let value: CatalogEnvelope = serde_json::from_str(&wire).map_err(|_| rejected())?;
    if value.providers.len() != provider_ids.len() || value.skills.len() != skill_ids.len() {
        return Err(rejected());
    }
    let providers = value
        .providers
        .into_iter()
        .map(|e| ExecutionProviderCatalogEntry {
            provider: e.provider,
            available: e.available,
        })
        .collect::<Vec<_>>();
    let skills = value
        .skills
        .into_iter()
        .map(|e| ExecutionSkillCatalogEntry {
            skill: e.skill,
            available: e.available,
        })
        .collect::<Vec<_>>();
    let scope = ExecutionProfileScope {
        tenant_id: command.tenant_id,
        project_id: t.project_id,
        worktree_id: Some(t.worktree_id),
    };
    let identity = ExecutionProfileRevisionIdentity {
        profile_id: t.execution_profile_id,
        version: t.execution_profile_version,
        content_digest: t.execution_profile_digest.clone(),
    };
    let revisions = ExecutionCatalogRevisions {
        provider_catalog_revision: value.provider_revision,
        skill_catalog_revision: value.skill_revision,
        grant_set_id: value.grants.grant_set_id,
        grant_set_version: value.grants.version,
    };
    let catalogs = CurrentExecutionCatalogSnapshot::for_profile(
        scope.clone(),
        identity.clone(),
        policies.profile(),
        revisions,
        observed,
        observed
            .checked_add(EXECUTION_CATALOG_FENCE_MAX_TTL_MS)
            .ok_or_else(rejected)?,
        value.grants,
        Arc::from(providers),
        Arc::from(skills),
    )
    .map_err(|_| rejected())?;
    catalogs
        .validate_for(
            &scope,
            &identity,
            revisions,
            epoch_ms()?,
            EXECUTION_CATALOG_FENCE_MIN_REMAINING_MS,
        )
        .map_err(|_| rejected())?;
    let result = ScheduleExecutionSnapshot {
        policies,
        catalogs,
        scope,
        identity,
        revisions,
    };
    result.validate_lifetime()?;
    Ok(result)
}

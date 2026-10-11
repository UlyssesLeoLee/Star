//! Bounded locked policy verification, not complete execution admission.
/*
@cypher schema=1 source_sha256=e0ba80ca5c49417ae4d91da51fcd0089411fa21843f1e2afd0a34113a4acb6e2
MERGE (self:File {path:"crates/run-owner-runtime/src/schedule_policy.rs"})
MERGE (load:Symbol {id:"crates/run-owner-runtime/src/schedule_policy.rs::load_current",kind:"function"})
MERGE (hook:Symbol {id:"crates/run-owner-runtime/src/schedule_policy.rs::verify_hook",kind:"function"})
MERGE (self)-[:DEFINES]->(load)
MERGE (self)-[:DEFINES]->(hook)
MERGE (load)-[:CALLS]->(hook)
MERGE (hex:Symbol {id:"crates/run-owner-runtime/src/schedule_policy.rs::hex_digest",kind:"function"})
MERGE (self)-[:DEFINES]->(hex)
MERGE (load)-[:CALLS]->(hex)
MERGE (hook)-[:CALLS]->(hex)
MERGE (self)-[:IMPORTS]->(profile_module:File {path:"crates/domain-agent/src/execution_profile.rs"})
MERGE (self)-[:IMPORTS]->(hook_module:File {path:"crates/domain-hook/src/lib.rs"})
MERGE (snapshot:Type {id:"run_owner_runtime::SchedulePolicySnapshot",kind:"struct"})
MERGE (self)-[:DEFINES]->(snapshot)
MERGE (load)-[:CALLS]->(read:Symbol {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql::read_schedule_policies_current",kind:"function"})
MERGE (load)-[:CALLS]->(verify:Symbol {id:"crates/domain-agent/src/execution_profile.rs::AgentExecutionProfileDocument.decode_and_verify",kind:"method"})
MERGE (hook)-[:CALLS]->(hook_verify:Symbol {id:"crates/domain-hook/src/lib.rs::HookPolicyDocument.verify",kind:"method"})
@endcypher
*/
use domain_agent::execution_profile::{
    AgentExecutionProfileDocument, ExecutionProfileScope, HookSetSnapshot,
    VerifiedAgentExecutionProfile,
};
use domain_automation::schedule::{
    ScheduleAdmissionRejectionCodeV1, ScheduleRunAdmissionCommandV1,
};
use domain_hook::{HookPolicyDocument, VerifiedHookPolicySnapshot, MAX_POLICY_DOCUMENT_BYTES};
use serde::Deserialize;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::ScheduleAdmissionStoreError;

/// Verified current immutable policies. This is not a permit to create/start a Run.
pub struct SchedulePolicySnapshot {
    profile: VerifiedAgentExecutionProfile,
    hook_policy: VerifiedHookPolicySnapshot,
}
impl SchedulePolicySnapshot {
    /// Borrow the verified Profile for later catalog/quota admission.
    pub fn profile(&self) -> &VerifiedAgentExecutionProfile {
        &self.profile
    }
    /// Borrow the native effective policy for later lifecycle evaluation.
    pub fn hook_policy(&self) -> &VerifiedHookPolicySnapshot {
        &self.hook_policy
    }
}

#[derive(Deserialize)]
struct ProfileRow {
    profile_revision_id: Uuid,
    tenant_id: Uuid,
    project_id: Uuid,
    profile_id: Uuid,
    profile_version: u64,
    schema_version: u32,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    content_digest: String,
    profile_document: serde_json::Value,
}
#[derive(Deserialize)]
struct HookRow {
    policy_set_id: Uuid,
    tenant_id: Uuid,
    project_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    policy_version: u64,
    schema_version: u16,
    evaluator_api_version: u16,
    digest: String,
    policy_document: serde_json::Value,
    inherited_project_policy_set_id: Option<Uuid>,
}
fn rejected() -> ScheduleAdmissionStoreError {
    ScheduleAdmissionStoreError::PrerequisiteRejected(
        ScheduleAdmissionRejectionCodeV1::TargetUnavailable,
    )
}

pub(crate) async fn load_current(
    tx: &mut Transaction<'_, Postgres>,
    command: &ScheduleRunAdmissionCommandV1,
) -> Result<SchedulePolicySnapshot, ScheduleAdmissionStoreError> {
    let target = &command.occurrence.rule_snapshot.target;
    let rows = sqlx::query("SELECT kind,payload FROM multica.read_schedule_policies_current($1,$2,$3,$4,$5,$6,$7,$8,$9)")
        .bind(command.tenant_id).bind(command.occurrence.run_as_actor_id).bind(target.project_id)
        .bind(target.branch_id).bind(target.engineering_run_id).bind(target.repository_id)
        .bind(target.worktree_id).bind(target.work_item_id).bind(target.execution_profile_id)
        .fetch_all(&mut **tx).await?;
    if !(2..=3).contains(&rows.len()) {
        return Err(rejected());
    }
    let mut payloads = Vec::with_capacity(3);
    for (index, row) in rows.into_iter().enumerate() {
        let expected = ["profile", "project_hook", "worktree_hook"][index];
        let payload: String = row.try_get("payload")?;
        if payload.len() > 73_728 || row.try_get::<String, _>("kind")? != expected {
            return Err(rejected());
        }
        payloads.push(payload);
    }
    let row: ProfileRow = serde_json::from_str(&payloads[0]).map_err(|_| rejected())?;
    if row.profile_revision_id.is_nil()
        || row.tenant_id != command.tenant_id
        || row.project_id != target.project_id
        || row.profile_id != target.execution_profile_id
        || row.profile_version != target.execution_profile_version
        || row.content_digest != target.execution_profile_digest
        || !matches!(
            (row.scope_kind.as_str(), row.worktree_id),
            ("project", None) | ("worktree", Some(_))
        )
    {
        return Err(rejected());
    }
    let bytes = serde_json::to_vec(&row.profile_document).map_err(|_| rejected())?;
    let profile =
        AgentExecutionProfileDocument::decode_and_verify(&bytes).map_err(|_| rejected())?;
    let document = profile.document();
    let scope = ExecutionProfileScope {
        tenant_id: command.tenant_id,
        project_id: target.project_id,
        worktree_id: Some(target.worktree_id),
    };
    profile.validate_for_scope(&scope).map_err(|_| rejected())?;
    if document.profile.scope.worktree_id != row.worktree_id
        || document.profile.schema_version != row.schema_version
        || document.content_digest != row.content_digest
    {
        return Err(rejected());
    }
    let baseline: HookRow = serde_json::from_str(&payloads[1]).map_err(|_| rejected())?;
    let project = verify_hook(&baseline, command, None)?;
    let (identity, effective) = if payloads.len() == 3 {
        let overlay: HookRow = serde_json::from_str(&payloads[2]).map_err(|_| rejected())?;
        let overlay_document = verify_hook(&overlay, command, Some(target.worktree_id))?;
        if overlay.inherited_project_policy_set_id != Some(baseline.policy_set_id)
            || overlay_document.project_version != project.project_version
            || overlay_document.project_rules != project.project_rules
        {
            return Err(rejected());
        }
        (overlay.policy_set_id, overlay_document)
    } else {
        (baseline.policy_set_id, project)
    };
    let hook_policy = effective.verify().map_err(|_| rejected())?;
    let hook_set = HookSetSnapshot {
        hook_set_id: identity,
        version: hook_policy.effective_version(),
        effective_digest: hex_digest(&hook_policy.digest()),
    };
    if hook_set != document.profile.hook_set
        || hook_set.version != target.hook_set_version
        || hook_set.effective_digest != target.hook_set_digest
    {
        return Err(rejected());
    }
    let p = &document.profile.resource_budget;
    let b = &command.infrastructure_binding.resource_budget;
    if i64::try_from(p.max_rss_bytes).ok() != Some(b.max_rss_bytes)
        || i64::try_from(p.max_cpu_ms).ok() != Some(b.max_cpu_ms)
        || i64::try_from(p.max_runtime_ms).ok() != Some(b.max_runtime_ms)
        || i64::from(p.max_child_processes) != i64::from(b.max_child_processes)
        || i64::from(p.max_parallel_tools) != i64::from(b.max_parallel_tools)
        || i64::from(p.max_provider_calls) != b.max_provider_calls
        || i64::try_from(p.max_output_bytes).ok() != Some(b.max_output_bytes)
        || i64::from(p.max_event_buffer_bytes) != b.max_event_buffer_bytes
    {
        return Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::BindingUnavailable,
        ));
    }
    Ok(SchedulePolicySnapshot {
        profile,
        hook_policy,
    })
}

fn verify_hook(
    row: &HookRow,
    command: &ScheduleRunAdmissionCommandV1,
    worktree: Option<Uuid>,
) -> Result<HookPolicyDocument, ScheduleAdmissionStoreError> {
    let bytes = serde_json::to_vec(&row.policy_document).map_err(|_| rejected())?;
    if bytes.len() > MAX_POLICY_DOCUMENT_BYTES {
        return Err(rejected());
    }
    let document: HookPolicyDocument = serde_json::from_slice(&bytes).map_err(|_| rejected())?;
    let version = if worktree.is_some() {
        document.worktree_version
    } else {
        Some(document.project_version)
    };
    if row.policy_set_id.is_nil()
        || row.tenant_id != command.tenant_id
        || row.project_id != command.occurrence.rule_snapshot.target.project_id
        || row.scope_kind
            != if worktree.is_some() {
                "worktree"
            } else {
                "project"
            }
        || row.worktree_id != worktree
        || document.tenant_id != command.tenant_id.into_bytes()
        || document.project_id != row.project_id.into_bytes()
        || document.worktree_id != worktree.map(Uuid::into_bytes)
        || version != Some(row.policy_version)
        || row.schema_version != document.schema_version
        || row.evaluator_api_version != document.evaluator_api_version
        || row.digest != format!("\\x{}", hex_digest(&document.digest))
        || (worktree.is_none() && row.inherited_project_policy_set_id.is_some())
    {
        return Err(rejected());
    }
    document.clone().verify().map_err(|_| rejected())?;
    Ok(document)
}
fn hex_digest(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(64);
    for byte in bytes {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 15)]));
    }
    result
}

//! Versioned Schedule rule, occurrence snapshot, and lease-fence contracts.
//!
//! This module defines the durable contract used by Automation persistence and Run admission.
//! It intentionally contains no in-memory scheduler or worker; materialization, claiming, and
//! Run creation must be committed by the application layer against PostgreSQL.
//!
//! @cypher schema=1 source_sha256=e42cd71bfa6e3150c8fe81dce9ef2aeb41be8959c5f3cade5ae5722d7849305c
//! MERGE (self:File {path:"crates/domain-automation/src/schedule.rs"})
//! MERGE (module:Symbol {id:"crates/domain-automation/src/schedule.rs::schedule",kind:"module"})
//! MERGE (target:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleTargetV1"})
//! MERGE (target_branch:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleTargetV1.branch_id",kind:"field"})
//! MERGE (retry:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleRetryPolicyV1"})
//! MERGE (overlap:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleOverlapPolicyV1"})
//! MERGE (misfire:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleMisfirePolicyV1"})
//! MERGE (gap:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleDstGapPolicyV1"})
//! MERGE (fold:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleDstFoldPolicyV1"})
//! MERGE (pause:Type {id:"crates/domain-automation/src/schedule.rs::SchedulePausePolicyV1"})
//! MERGE (rule:Type {id:"crates/domain-automation/src/schedule.rs::AutomationScheduleRuleRevisionV1"})
//! MERGE (run_as_actor:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationScheduleRuleRevisionV1.run_as_actor_id",kind:"field"})
//! MERGE (key:Type {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceKey"})
//! MERGE (occurrence:Type {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceSnapshotV1"})
//! MERGE (occurrence_run_as:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceSnapshotV1.run_as_actor_id",kind:"field"})
//! MERGE (fence:Type {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceLeaseFenceV1"})
//! MERGE (error:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleContractError"})
//! MERGE (tests:Symbol {id:"crates/domain-automation/src/schedule.rs::tests",kind:"module"})
//! MERGE (target_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleTargetV1::validate",kind:"method"})
//! MERGE (retry_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRetryPolicyV1::validate",kind:"method"})
//! MERGE (rule_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationScheduleRuleRevisionV1::validate",kind:"method"})
//! MERGE (key_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceKey::validate",kind:"method"})
//! MERGE (occurrence_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceSnapshotV1::validate",kind:"method"})
//! MERGE (materializer:Symbol {id:"crates/domain-automation/src/schedule.rs::materialize_schedule_window",kind:"function"})
//! MERGE (recurrence_validator:Symbol {id:"crates/domain-automation/src/schedule.rs::validate_schedule_recurrence",kind:"function"})
//! MERGE (candidate_collector:Symbol {id:"crates/domain-automation/src/schedule.rs::collect_schedule_candidates",kind:"function"})
//! MERGE (fence_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceLeaseFenceV1::validate",kind:"method"})
//! MERGE (digest:Symbol {id:"crates/domain-automation/src/schedule.rs::is_lower_hex_sha256",kind:"function"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (self)-[:DEFINES]->(target)
//! MERGE (target)-[:CONTAINS]->(target_branch)
//! MERGE (self)-[:DEFINES]->(retry)
//! MERGE (self)-[:DEFINES]->(overlap)
//! MERGE (self)-[:DEFINES]->(misfire)
//! MERGE (self)-[:DEFINES]->(gap)
//! MERGE (self)-[:DEFINES]->(fold)
//! MERGE (self)-[:DEFINES]->(pause)
//! MERGE (self)-[:DEFINES]->(rule)
//! MERGE (rule)-[:CONTAINS]->(run_as_actor)
//! MERGE (self)-[:DEFINES]->(key)
//! MERGE (self)-[:DEFINES]->(occurrence)
//! MERGE (occurrence)-[:CONTAINS]->(occurrence_run_as)
//! MERGE (self)-[:DEFINES]->(fence)
//! MERGE (self)-[:DEFINES]->(error)
//! MERGE (self)-[:DEFINES]->(tests)
//! MERGE (self)-[:DEFINES]->(target_validate)
//! MERGE (self)-[:DEFINES]->(retry_validate)
//! MERGE (self)-[:DEFINES]->(rule_validate)
//! MERGE (self)-[:DEFINES]->(key_validate)
//! MERGE (self)-[:DEFINES]->(occurrence_validate)
//! MERGE (self)-[:DEFINES]->(materializer)
//! MERGE (self)-[:DEFINES]->(recurrence_validator)
//! MERGE (self)-[:DEFINES]->(candidate_collector)
//! MERGE (self)-[:DEFINES]->(fence_validate)
//! MERGE (self)-[:DEFINES]->(digest)
//! MERGE (target_validate)-[:CALLS]->(digest)
//! MERGE (target_validate)-[:ALLOWS_NULL]->(target_branch)
//! MERGE (rule_validate)-[:CALLS]->(target_validate)
//! MERGE (rule_validate)-[:CALLS]->(retry_validate)
//! MERGE (occurrence_validate)-[:CALLS]->(key_validate)
//! MERGE (occurrence_validate)-[:CALLS]->(rule_validate)
//! MERGE (occurrence_validate)-[:REQUIRES_PRESENT]->(target_branch)
//! MERGE (recurrence_validator)-[:CALLS]->(candidate_collector)
//! MERGE (materializer)-[:CALLS]->(candidate_collector)
//! MERGE (candidate_collector)-[:CALLS]->(rule_validate)
//! MERGE (materializer)-[:REQUIRES_PRESENT]->(target_branch)
//! MERGE (materializer)-[:EMITS]->(occurrence)
//! MERGE (admission_command:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionCommandV1"})
//! MERGE (admission_result:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResultV1"})
//! MERGE (admission_outcome:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionOutcomeV1"})
//! MERGE (admission_rejection:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleAdmissionRejectionCodeV1"})
//! MERGE (admission_resolution:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResolutionV1"})
//! MERGE (resolution_kind:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleAdmissionResolutionKindV1"})
//! MERGE (compensation_reason:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleAdmissionCompensationReasonV1"})
//! MERGE (admission_error:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleAdmissionContractError"})
//! MERGE (binding_snapshot:Type {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2"})
//! MERGE (command_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionCommandV1::validate",kind:"method"})
//! MERGE (command_encode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionCommandV1::encode_json",kind:"method"})
//! MERGE (command_decode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionCommandV1::decode_json",kind:"method"})
//! MERGE (command_binding_digest:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionCommandV1::binding_digest",kind:"method"})
//! MERGE (binding_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2::validate",kind:"method"})
//! MERGE (binding_snapshot_digest:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2::digest",kind:"method"})
//! MERGE (result_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResultV1::validate",kind:"method"})
//! MERGE (result_encode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResultV1::encode_json",kind:"method"})
//! MERGE (result_decode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResultV1::decode_json",kind:"method"})
//! MERGE (result_match:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResultV1::validate_for_command",kind:"method"})
//! MERGE (resolution_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResolutionV1::validate",kind:"method"})
//! MERGE (resolution_match:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResolutionV1::validate_for_result",kind:"method"})
//! MERGE (resolution_encode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResolutionV1::encode_json",kind:"method"})
//! MERGE (resolution_decode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResolutionV1::decode_json",kind:"method"})
//! MERGE (encode_bounded:Symbol {id:"crates/domain-automation/src/schedule.rs::encode_bounded",kind:"function"})
//! MERGE (decode_bounded:Symbol {id:"crates/domain-automation/src/schedule.rs::decode_bounded",kind:"function"})
//! MERGE (admission_command_test:Symbol {id:"crates/domain-automation/src/schedule.rs::tests::admission_command_round_trips_and_pins_canonical_binding_scope",kind:"test"})
//! MERGE (admission_result_test:Symbol {id:"crates/domain-automation/src/schedule.rs::tests::admission_result_and_resolution_must_match_the_same_request_and_binding",kind:"test"})
//! MERGE (admission_wire_test:Symbol {id:"crates/domain-automation/src/schedule.rs::tests::admission_wire_rejects_unknown_fields_and_oversized_payloads",kind:"test"})
//! MERGE (hostless_config_test:Symbol {id:"crates/domain-automation/src/schedule.rs::tests::hostless_rule_can_be_configured_but_cannot_be_materialized",kind:"test"})
//! MERGE (occurrence_snapshot_test:Symbol {id:"crates/domain-automation/src/schedule.rs::tests::occurrence_key_and_snapshot_bind_exact_rule_revision_and_utc_slot",kind:"test"})
//! MERGE (self)-[:DEFINES]->(admission_command)
//! MERGE (self)-[:DEFINES]->(admission_result)
//! MERGE (self)-[:DEFINES]->(admission_outcome)
//! MERGE (self)-[:DEFINES]->(admission_rejection)
//! MERGE (self)-[:DEFINES]->(admission_resolution)
//! MERGE (self)-[:DEFINES]->(resolution_kind)
//! MERGE (self)-[:DEFINES]->(compensation_reason)
//! MERGE (self)-[:DEFINES]->(admission_error)
//! MERGE (self)-[:DEFINES]->(command_validate)
//! MERGE (self)-[:DEFINES]->(command_encode)
//! MERGE (self)-[:DEFINES]->(command_decode)
//! MERGE (self)-[:DEFINES]->(command_binding_digest)
//! MERGE (self)-[:DEFINES]->(result_validate)
//! MERGE (self)-[:DEFINES]->(result_encode)
//! MERGE (self)-[:DEFINES]->(result_decode)
//! MERGE (self)-[:DEFINES]->(result_match)
//! MERGE (self)-[:DEFINES]->(resolution_validate)
//! MERGE (self)-[:DEFINES]->(resolution_match)
//! MERGE (self)-[:DEFINES]->(resolution_encode)
//! MERGE (self)-[:DEFINES]->(resolution_decode)
//! MERGE (self)-[:DEFINES]->(encode_bounded)
//! MERGE (self)-[:DEFINES]->(decode_bounded)
//! MERGE (tests)-[:DEFINES]->(admission_command_test)
//! MERGE (tests)-[:DEFINES]->(admission_result_test)
//! MERGE (tests)-[:DEFINES]->(admission_wire_test)
//! MERGE (tests)-[:DEFINES]->(hostless_config_test)
//! MERGE (tests)-[:DEFINES]->(occurrence_snapshot_test)
//! MERGE (admission_command)-[:CONTAINS]->(occurrence)
//! MERGE (admission_command)-[:USES_TYPE]->(binding_snapshot)
//! MERGE (admission_result)-[:CONTAINS]->(admission_outcome)
//! MERGE (admission_result)-[:CONTAINS]->(admission_rejection)
//! MERGE (admission_resolution)-[:CONTAINS]->(resolution_kind)
//! MERGE (admission_resolution)-[:CONTAINS]->(compensation_reason)
//! MERGE (command_validate)-[:CALLS]->(occurrence_validate)
//! MERGE (command_validate)-[:CALLS]->(binding_validate)
//! MERGE (command_binding_digest)-[:CALLS]->(command_validate)
//! MERGE (command_binding_digest)-[:CALLS]->(binding_snapshot_digest)
//! MERGE (command_encode)-[:CALLS]->(command_validate)
//! MERGE (command_encode)-[:CALLS]->(encode_bounded)
//! MERGE (command_decode)-[:CALLS]->(decode_bounded)
//! MERGE (command_decode)-[:CALLS]->(command_validate)
//! MERGE (result_encode)-[:CALLS]->(result_validate)
//! MERGE (result_encode)-[:CALLS]->(encode_bounded)
//! MERGE (result_decode)-[:CALLS]->(decode_bounded)
//! MERGE (result_decode)-[:CALLS]->(result_validate)
//! MERGE (result_match)-[:CALLS]->(result_validate)
//! MERGE (result_match)-[:CALLS]->(command_binding_digest)
//! MERGE (resolution_match)-[:CALLS]->(resolution_validate)
//! MERGE (resolution_match)-[:CALLS]->(result_validate)
//! MERGE (resolution_encode)-[:CALLS]->(resolution_validate)
//! MERGE (resolution_encode)-[:CALLS]->(encode_bounded)
//! MERGE (resolution_decode)-[:CALLS]->(decode_bounded)
//! MERGE (resolution_decode)-[:CALLS]->(resolution_validate)
//! MERGE (admission_command_test)-[:TESTS]->(command_encode)
//! MERGE (admission_command_test)-[:TESTS]->(command_decode)
//! MERGE (admission_command_test)-[:TESTS]->(command_binding_digest)
//! MERGE (admission_result_test)-[:TESTS]->(result_match)
//! MERGE (admission_result_test)-[:TESTS]->(result_encode)
//! MERGE (admission_result_test)-[:TESTS]->(result_decode)
//! MERGE (admission_result_test)-[:TESTS]->(resolution_match)
//! MERGE (admission_result_test)-[:TESTS]->(resolution_encode)
//! MERGE (admission_result_test)-[:TESTS]->(resolution_decode)
//! MERGE (admission_wire_test)-[:TESTS]->(command_encode)
//! MERGE (admission_wire_test)-[:TESTS]->(command_decode)
//! MERGE (hostless_config_test)-[:TESTS]->(rule_validate)
//! MERGE (hostless_config_test)-[:TESTS]->(recurrence_validator)
//! MERGE (hostless_config_test)-[:TESTS]->(materializer)
//! MERGE (occurrence_snapshot_test)-[:TESTS]->(occurrence_validate)
//! MERGE (self)-[:IMPORTS]->(:File {path:"crates/domain-infrastructure/src/lib.rs"})
//! @endcypher

use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use domain_infrastructure::RunInfrastructureBindingSnapshotV2;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

use crate::RuleId;

/// Exact cron parser build accepted for new Schedule materialization.
pub const SCHEDULE_PARSER_VERSION: &str = "star-cron-compat-1+cron-0.17.0";
/// Exact bundled IANA timezone-data provider build accepted for materialization.
pub const SCHEDULE_TZDB_VERSION: &str = "chrono-tz-0.10.4";
const MAX_SCHEDULE_SCAN_SLOTS: usize = 32_768;
const MAX_TRANSITION_PROBE_SECONDS: i64 = 3_600;
const MAX_TRANSITION_PROBES: usize = 32_768;

/// A configured Worktree-first Schedule target. Remote Branch context may be absent while a
/// hostless Run is being configured; occurrence materialization still requires that context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleTargetV1 {
    /// Owning Project.
    pub project_id: Uuid,
    /// Optional remote Branch identity; not an Engineering Run authorization prerequisite.
    pub branch_id: Option<Uuid>,
    /// Engineering Run workspace identity.
    pub engineering_run_id: Uuid,
    /// Repository containing the local checkout.
    pub repository_id: Uuid,
    /// Canonical local Worktree checkout.
    pub worktree_id: Uuid,
    /// Task Card / Work Item to execute.
    pub work_item_id: Uuid,
    /// Immutable AgentExecutionProfile identity selected by the rule revision.
    pub execution_profile_id: Uuid,
    /// Immutable AgentExecutionProfile revision.
    pub execution_profile_version: u64,
    /// Lowercase SHA-256 digest of the selected execution profile.
    pub execution_profile_digest: String,
    /// HookSet revision evaluated by the scheduled Run.
    pub hook_set_version: u64,
    /// Lowercase SHA-256 digest of the effective HookSet.
    pub hook_set_digest: String,
}

impl ScheduleTargetV1 {
    /// Validate that the target is complete and that versioned policy snapshots are pinned.
    pub fn validate(&self) -> Result<(), ScheduleContractError> {
        if [
            self.project_id,
            self.engineering_run_id,
            self.repository_id,
            self.worktree_id,
            self.work_item_id,
            self.execution_profile_id,
        ]
        .iter()
        .any(Uuid::is_nil)
            || self.execution_profile_version == 0
            || self.hook_set_version == 0
            || self.branch_id.is_some_and(|branch_id| branch_id.is_nil())
        {
            return Err(ScheduleContractError::IncompleteTarget);
        }
        if !is_lower_hex_sha256(&self.execution_profile_digest)
            || !is_lower_hex_sha256(&self.hook_set_digest)
        {
            return Err(ScheduleContractError::InvalidDigest);
        }
        Ok(())
    }
}

/// Behavior when a due occurrence overlaps another active occurrence for the same rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScheduleOverlapPolicyV1 {
    /// Do not dispatch while the prior Run is active; record a skip fact.
    Skip,
    /// Keep at most one queued occurrence behind the active Run.
    QueueOne,
    /// Permit bounded parallel Runs for this rule.
    AllowBounded {
        /// Maximum concurrent Runs for this rule; bounded by validation.
        max_active: u16,
    },
}

/// Behavior for scheduled slots missed while the scheduler was unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScheduleMisfirePolicyV1 {
    /// Record missed slots as skipped.
    Skip,
    /// Materialize only the latest missed slot.
    CoalesceLatest,
    /// Replay no more than the configured number of missed slots.
    CatchUp {
        /// Maximum missed slots materialized in one recovery pass.
        max_occurrences: u16,
    },
}

/// Resolution for a local time that does not exist during a timezone clock jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleDstGapPolicyV1 {
    /// Skip the nonexistent local time.
    Skip,
    /// Move the local time forward to the first valid instant.
    ShiftForward,
}

/// Resolution for a local time that occurs twice when clocks move backward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleDstFoldPolicyV1 {
    /// Materialize the earlier UTC instant once.
    EarlierInstant,
    /// Materialize the later UTC instant once.
    LaterInstant,
}

/// Behavior for slots elapsed while an administrator has paused a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulePausePolicyV1 {
    /// Do not replay slots elapsed during the pause.
    SkipElapsed,
    /// Coalesce elapsed slots to one occurrence when the rule resumes.
    CoalesceLatest,
}

/// Bounded retry policy applied after a scheduled Run fails before its deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRetryPolicyV1 {
    /// Total attempts, including the initial attempt; maximum 25.
    pub max_attempts: u16,
    /// Initial retry delay in seconds.
    pub initial_backoff_seconds: u32,
    /// Exponential backoff ceiling in seconds.
    pub max_backoff_seconds: u32,
}

impl ScheduleRetryPolicyV1 {
    /// Validate bounded attempts and monotonic backoff settings.
    pub fn validate(&self) -> Result<(), ScheduleContractError> {
        if self.max_attempts == 0
            || self.max_attempts > 25
            || self.initial_backoff_seconds == 0
            || self.max_backoff_seconds < self.initial_backoff_seconds
            || self.max_backoff_seconds > 86_400
        {
            return Err(ScheduleContractError::InvalidRetryPolicy);
        }
        Ok(())
    }
}

/// Immutable version of a scheduled Automation rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomationScheduleRuleRevisionV1 {
    /// Owning tenant.
    pub tenant_id: Uuid,
    /// Rule identity shared by all immutable revisions.
    pub rule_id: RuleId,
    /// Positive SCD2 revision number.
    pub rule_version: u64,
    /// Immutable user principal that created the rule and is reauthorized for every scheduled Run.
    pub run_as_actor_id: Uuid,
    /// Whether this revision is eligible for new occurrences.
    pub enabled: bool,
    /// Owning Project.
    pub project_id: Uuid,
    /// Bounded cron expression interpreted by the pinned parser version.
    pub cron_expression: String,
    /// IANA timezone used to interpret local schedule slots.
    pub time_zone: String,
    /// Version of the recurrence parser used for slot materialization.
    pub parser_version: String,
    /// Version of the IANA timezone database used for slot materialization.
    pub tzdb_version: String,
    /// Local-time gap behavior.
    pub dst_gap_policy: ScheduleDstGapPolicyV1,
    /// Local-time fold behavior.
    pub dst_fold_policy: ScheduleDstFoldPolicyV1,
    /// Overlap behavior.
    pub overlap_policy: ScheduleOverlapPolicyV1,
    /// Misfire behavior.
    pub misfire_policy: ScheduleMisfirePolicyV1,
    /// Pause behavior.
    pub pause_policy: SchedulePausePolicyV1,
    /// Retry behavior.
    pub retry_policy: ScheduleRetryPolicyV1,
    /// Maximum wall-clock duration of one occurrence, in seconds.
    pub deadline_seconds: u32,
    /// Fully resolved immutable execution target.
    pub target: ScheduleTargetV1,
}

impl AutomationScheduleRuleRevisionV1 {
    /// Validate identity, bounded recurrence inputs, schedule policies, and target snapshot.
    pub fn validate(&self) -> Result<(), ScheduleContractError> {
        if self.tenant_id.is_nil()
            || self.project_id.is_nil()
            || self.rule_id.as_uuid().is_nil()
            || self.run_as_actor_id.is_nil()
            || self.rule_version == 0
            || self.deadline_seconds == 0
            || self.deadline_seconds > 86_400
            || self.cron_expression.trim().is_empty()
            || self.cron_expression.len() > 120
            || !self.cron_expression.is_ascii()
            || self.cron_expression.chars().any(char::is_control)
            || self.time_zone.trim().is_empty()
            || self.time_zone.len() > 128
            || self.time_zone.chars().any(char::is_control)
            || self.parser_version.trim().is_empty()
            || self.parser_version.len() > 64
            || self.parser_version.chars().any(char::is_control)
            || self.tzdb_version.trim().is_empty()
            || self.tzdb_version.len() > 64
            || self.tzdb_version.chars().any(char::is_control)
        {
            return Err(ScheduleContractError::InvalidRule);
        }
        match self.overlap_policy {
            ScheduleOverlapPolicyV1::AllowBounded { max_active }
                if max_active == 0 || max_active > 64 =>
            {
                return Err(ScheduleContractError::InvalidOverlapPolicy);
            }
            _ => {}
        }
        match self.misfire_policy {
            ScheduleMisfirePolicyV1::CatchUp { max_occurrences }
                if max_occurrences == 0 || max_occurrences > 256 =>
            {
                return Err(ScheduleContractError::InvalidMisfirePolicy);
            }
            _ => {}
        }
        if self.target.project_id != self.project_id {
            return Err(ScheduleContractError::TargetProjectMismatch);
        }
        self.retry_policy.validate()?;
        self.target.validate()
    }
}

/// Stable identity for the UTC slot emitted by one immutable rule revision.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomationOccurrenceKey {
    /// Tenant that owns the rule and its occurrence namespace.
    pub tenant_id: Uuid,
    /// Rule identity.
    pub rule_id: RuleId,
    /// Rule revision that interpreted the slot.
    pub rule_version: u64,
    /// Exact UTC instant; distinguishes both sides of a DST fold.
    pub scheduled_for_utc: DateTime<Utc>,
}

impl AutomationOccurrenceKey {
    /// Validate the unique slot key used for idempotent materialization.
    pub fn validate(&self) -> Result<(), ScheduleContractError> {
        if self.tenant_id.is_nil() || self.rule_id.as_uuid().is_nil() || self.rule_version == 0 {
            return Err(ScheduleContractError::InvalidOccurrenceKey);
        }
        Ok(())
    }
}

/// Immutable explanation of how a schedule slot became an execution opportunity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomationOccurrenceSnapshotV1 {
    /// Occurrence schema version.
    pub schema_version: u16,
    /// Database-generated occurrence identity.
    pub occurrence_id: Uuid,
    /// Stable rule-version/UTC-slot idempotency key.
    pub key: AutomationOccurrenceKey,
    /// Immutable creator/run-as principal copied into this materialized occurrence.
    pub run_as_actor_id: Uuid,
    /// Immutable rule and target snapshot used for materialization.
    pub rule_snapshot: AutomationScheduleRuleRevisionV1,
    /// Local-time label retained for DST auditability.
    pub scheduled_local_label: String,
    /// UTC offset in seconds for the resolved local-time slot.
    pub utc_offset_seconds: i32,
    /// Time when the occurrence was durably materialized.
    pub materialized_at: DateTime<Utc>,
}

impl AutomationOccurrenceSnapshotV1 {
    /// Validate that the occurrence key and immutable rule snapshot agree.
    pub fn validate(&self) -> Result<(), ScheduleContractError> {
        self.key.validate()?;
        self.rule_snapshot.validate()?;
        if self.schema_version != 1
            || self.occurrence_id.is_nil()
            || self.key.tenant_id != self.rule_snapshot.tenant_id
            || self.key.rule_id != self.rule_snapshot.rule_id
            || self.key.rule_version != self.rule_snapshot.rule_version
            || self.run_as_actor_id.is_nil()
            || self.run_as_actor_id != self.rule_snapshot.run_as_actor_id
            || self.rule_snapshot.target.branch_id.is_none()
            || self.scheduled_local_label.trim().is_empty()
            || self.scheduled_local_label.len() > 64
            || self.scheduled_local_label.chars().any(char::is_control)
            || !(-86_400..=86_400).contains(&self.utc_offset_seconds)
        {
            return Err(ScheduleContractError::InvalidOccurrenceSnapshot);
        }
        Ok(())
    }
}

/// Monotonic claim identity. Every reclaim must increment generation in the same DB transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomationOccurrenceLeaseFenceV1 {
    /// Tenant that owns the occurrence.
    pub tenant_id: Uuid,
    /// Durable occurrence identity.
    pub occurrence_id: Uuid,
    /// Monotonically increasing claim generation.
    pub fencing_generation: u64,
    /// Unique worker identity for this lease.
    pub lease_owner_id: Uuid,
    /// Current lease expiry.
    pub lease_expires_at: DateTime<Utc>,
    /// Hard execution deadline captured from the occurrence snapshot.
    pub deadline_at: DateTime<Utc>,
}

impl AutomationOccurrenceLeaseFenceV1 {
    /// Reject stale, empty, expired, or over-deadline leases before a state-changing write.
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), ScheduleContractError> {
        if self.tenant_id.is_nil()
            || self.occurrence_id.is_nil()
            || self.lease_owner_id.is_nil()
            || self.fencing_generation == 0
        {
            return Err(ScheduleContractError::InvalidLeaseFence);
        }
        if self.lease_expires_at <= now || self.deadline_at <= now {
            return Err(ScheduleContractError::ExpiredLeaseFence);
        }
        if self.lease_expires_at > self.deadline_at {
            return Err(ScheduleContractError::LeasePastDeadline);
        }
        Ok(())
    }
}

/// Current protocol version for the owner-local Schedule→Run admission messages.
pub const SCHEDULE_ADMISSION_SCHEMA_VERSION: u16 = 1;

/// Maximum serialized JSON size accepted for one Schedule admission owner message.
pub const MAX_SCHEDULE_ADMISSION_MESSAGE_BYTES: usize = 16 * 1024;

/// Immutable Schedule→Run request for one admission attempt.
///
/// The broker-authenticated Schedule service identity is outside this payload; tenant and
/// run-as fields are scope claims, not proof of authorization. The request_id and resolved
/// Infrastructure binding are persisted before publication and
/// reused byte-for-byte for transport retries. Schedule lease fences stay in Schedule-owner
/// state; they are deliberately excluded from this replay-stable request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRunAdmissionCommandV1 {
    /// Wire schema version.
    pub schema_version: u16,
    /// Tenant owning the Rule and occurrence.
    pub tenant_id: Uuid,
    /// Stable identity of this logical admission attempt and its Run-owner Inbox row.
    pub request_id: Uuid,
    /// Correlation shared by this Saga's owner messages.
    pub correlation_id: Uuid,
    /// Monotonic attempt number within the occurrence; transport redelivery does not increment it.
    pub admission_attempt: u32,
    /// Immutable Rule, creator, target, and UTC-slot snapshot.
    pub occurrence: AutomationOccurrenceSnapshotV1,
    /// Canonical Infrastructure binding resolved for this attempt.
    pub infrastructure_binding: RunInfrastructureBindingSnapshotV2,
}

impl ScheduleRunAdmissionCommandV1 {
    /// Validate message identity and ensure the pinned target and binding share one owner scope.
    pub fn validate(&self) -> Result<(), ScheduleAdmissionContractError> {
        if self.schema_version != SCHEDULE_ADMISSION_SCHEMA_VERSION {
            return Err(ScheduleAdmissionContractError::UnsupportedSchemaVersion);
        }
        if self.tenant_id.is_nil()
            || self.request_id.is_nil()
            || self.correlation_id.is_nil()
            || self.admission_attempt == 0
        {
            return Err(ScheduleAdmissionContractError::InvalidIdentity);
        }
        self.occurrence
            .validate()
            .map_err(|_| ScheduleAdmissionContractError::InvalidOccurrence)?;
        self.infrastructure_binding
            .validate()
            .map_err(|_| ScheduleAdmissionContractError::InvalidInfrastructureBinding)?;

        let target = &self.occurrence.rule_snapshot.target;
        let binding = &self.infrastructure_binding;
        if self.tenant_id != self.occurrence.key.tenant_id
            || self.tenant_id != binding.tenant_id
            || target.project_id != binding.project_id
            || target.branch_id != Some(binding.branch_id)
            || target.engineering_run_id != binding.engineering_run_id
        {
            return Err(ScheduleAdmissionContractError::ScopeMismatch);
        }
        Ok(())
    }

    /// Return the canonical digest that the Run result and Schedule confirmation must echo.
    pub fn binding_digest(&self) -> Result<[u8; 32], ScheduleAdmissionContractError> {
        self.validate()?;
        self.infrastructure_binding
            .digest()
            .map_err(|_| ScheduleAdmissionContractError::InvalidInfrastructureBinding)
    }

    /// Serialize the validated message, rejecting payloads above the protocol bound.
    pub fn encode_json(&self) -> Result<Vec<u8>, ScheduleAdmissionContractError> {
        self.validate()?;
        encode_bounded(self)
    }

    /// Decode a bounded, strict JSON payload and validate its cross-domain scope.
    pub fn decode_json(payload: &[u8]) -> Result<Self, ScheduleAdmissionContractError> {
        let message: Self = decode_bounded(payload)?;
        message.validate()?;
        Ok(message)
    }
}

/// Whether the Run owner accepted a provisional Run or rejected admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleRunAdmissionOutcomeV1 {
    /// A non-executing TaskExecutionRun and Run-owner facts were committed.
    Accepted,
    /// Admission was terminally rejected without creating a TaskExecutionRun.
    Rejected,
}

/// Stable, non-sensitive terminal rejection codes returned by the Run owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleAdmissionRejectionCodeV1 {
    /// The fixed creator no longer has current Project/Run execution authorization.
    RunAsUnauthorized,
    /// The pinned Worktree, Task, Profile, or HookSet is no longer eligible.
    TargetUnavailable,
    /// The Project or Run execution budget does not permit admission.
    QuotaExceeded,
    /// The current policy rejects the requested admission.
    PolicyRejected,
    /// The exact pinned Infrastructure binding is no longer current.
    BindingUnavailable,
    /// The occurrence deadline has expired.
    OccurrenceExpired,
    /// The occurrence was cancelled before admission completed.
    OccurrenceCancelled,
}

/// Run→Schedule result durably published after the Run owner transaction commits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRunAdmissionResultV1 {
    /// Wire schema version.
    pub schema_version: u16,
    /// Tenant owning the Rule and occurrence.
    pub tenant_id: Uuid,
    /// Stable identity of this immutable Run-owner result fact.
    pub result_id: Uuid,
    /// Admission request being answered.
    pub request_id: Uuid,
    /// Attempt ordinal copied from the immutable Schedule request.
    pub admission_attempt: u32,
    /// Saga correlation identity.
    pub correlation_id: Uuid,
    /// Occurrence grouped by this admission attempt.
    pub occurrence_id: Uuid,
    /// Terminal outcome committed by the Run owner.
    pub outcome: ScheduleRunAdmissionOutcomeV1,
    /// Provisional non-executing Run ID when accepted.
    pub task_execution_run_id: Option<Uuid>,
    /// Stable rejection reason when rejected.
    pub rejection_code: Option<ScheduleAdmissionRejectionCodeV1>,
    /// Digest of the exact Infrastructure binding pinned by the request.
    pub binding_digest: [u8; 32],
}

impl ScheduleRunAdmissionResultV1 {
    /// Validate result identity and the mutually exclusive accepted/rejected payload.
    pub fn validate(&self) -> Result<(), ScheduleAdmissionContractError> {
        if self.schema_version != SCHEDULE_ADMISSION_SCHEMA_VERSION {
            return Err(ScheduleAdmissionContractError::UnsupportedSchemaVersion);
        }
        if self.tenant_id.is_nil()
            || self.result_id.is_nil()
            || self.request_id.is_nil()
            || self.admission_attempt == 0
            || self.correlation_id.is_nil()
            || self.occurrence_id.is_nil()
        {
            return Err(ScheduleAdmissionContractError::InvalidIdentity);
        }
        let accepted_payload = self
            .task_execution_run_id
            .is_some_and(|run_id| !run_id.is_nil())
            && self.rejection_code.is_none();
        let rejected_payload =
            self.task_execution_run_id.is_none() && self.rejection_code.is_some();
        match self.outcome {
            ScheduleRunAdmissionOutcomeV1::Accepted if accepted_payload => Ok(()),
            ScheduleRunAdmissionOutcomeV1::Rejected if rejected_payload => Ok(()),
            _ => Err(ScheduleAdmissionContractError::InvalidResult),
        }
    }

    /// Ensure the result answers exactly the persisted request and its binding snapshot.
    pub fn validate_for_command(
        &self,
        command: &ScheduleRunAdmissionCommandV1,
    ) -> Result<(), ScheduleAdmissionContractError> {
        self.validate()?;
        let binding_digest = command.binding_digest()?;
        if self.tenant_id != command.tenant_id
            || self.request_id != command.request_id
            || self.admission_attempt != command.admission_attempt
            || self.correlation_id != command.correlation_id
            || self.occurrence_id != command.occurrence.occurrence_id
            || self.binding_digest != binding_digest
        {
            return Err(ScheduleAdmissionContractError::ScopeMismatch);
        }
        Ok(())
    }

    /// Serialize the validated result, rejecting payloads above the protocol bound.
    pub fn encode_json(&self) -> Result<Vec<u8>, ScheduleAdmissionContractError> {
        self.validate()?;
        encode_bounded(self)
    }

    /// Decode a bounded, strict JSON payload and validate its result shape.
    pub fn decode_json(payload: &[u8]) -> Result<Self, ScheduleAdmissionContractError> {
        let message: Self = decode_bounded(payload)?;
        message.validate()?;
        Ok(message)
    }
}

/// Schedule's terminal decision for a successfully accepted Run-owner result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleAdmissionResolutionKindV1 {
    /// Schedule linked the Run under the current lease fence and confirmed admission.
    Confirm,
    /// Schedule could not safely link the Run and requests owner-local compensation.
    Compensate,
}

/// Stable compensation reasons understood by the Run owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleAdmissionCompensationReasonV1 {
    /// The Schedule lease owner or generation changed before the result was linked.
    ScheduleFenceLost,
    /// The occurrence was cancelled before it could be linked.
    OccurrenceCancelled,
    /// The occurrence deadline expired before it could be linked.
    OccurrenceExpired,
    /// The Schedule owner detected a conflicting link.
    ScheduleLinkConflict,
}

/// Schedule→Run confirmation or compensation command for one accepted result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRunAdmissionResolutionV1 {
    /// Wire schema version.
    pub schema_version: u16,
    /// Tenant owning the Rule and occurrence.
    pub tenant_id: Uuid,
    /// Stable identity of this immutable Schedule resolution fact.
    pub resolution_id: Uuid,
    /// Admission request being resolved.
    pub request_id: Uuid,
    /// Attempt ordinal copied from the immutable Schedule request.
    pub admission_attempt: u32,
    /// Run-owner result being confirmed or compensated.
    pub result_id: Uuid,
    /// Saga correlation identity.
    pub correlation_id: Uuid,
    /// Occurrence grouped by this admission attempt.
    pub occurrence_id: Uuid,
    /// Provisional Run accepted by the Run owner.
    pub task_execution_run_id: Uuid,
    /// Digest of the exact Infrastructure binding pinned by the request.
    pub binding_digest: [u8; 32],
    /// Schedule's owner-local terminal decision.
    pub decision: ScheduleAdmissionResolutionKindV1,
    /// Required only for a compensation command.
    pub compensation_reason: Option<ScheduleAdmissionCompensationReasonV1>,
}

impl ScheduleRunAdmissionResolutionV1 {
    /// Validate message identity and confirm/compensate shape.
    pub fn validate(&self) -> Result<(), ScheduleAdmissionContractError> {
        if self.schema_version != SCHEDULE_ADMISSION_SCHEMA_VERSION {
            return Err(ScheduleAdmissionContractError::UnsupportedSchemaVersion);
        }
        if self.tenant_id.is_nil()
            || self.resolution_id.is_nil()
            || self.request_id.is_nil()
            || self.admission_attempt == 0
            || self.result_id.is_nil()
            || self.correlation_id.is_nil()
            || self.occurrence_id.is_nil()
            || self.task_execution_run_id.is_nil()
        {
            return Err(ScheduleAdmissionContractError::InvalidIdentity);
        }
        let valid_decision = match self.decision {
            ScheduleAdmissionResolutionKindV1::Confirm => self.compensation_reason.is_none(),
            ScheduleAdmissionResolutionKindV1::Compensate => self.compensation_reason.is_some(),
        };
        if !valid_decision {
            return Err(ScheduleAdmissionContractError::InvalidResolution);
        }
        Ok(())
    }

    /// Ensure this resolution matches the exact accepted Run-owner result.
    pub fn validate_for_result(
        &self,
        result: &ScheduleRunAdmissionResultV1,
    ) -> Result<(), ScheduleAdmissionContractError> {
        self.validate()?;
        result.validate()?;
        if result.outcome != ScheduleRunAdmissionOutcomeV1::Accepted
            || self.tenant_id != result.tenant_id
            || self.request_id != result.request_id
            || self.admission_attempt != result.admission_attempt
            || self.result_id != result.result_id
            || self.correlation_id != result.correlation_id
            || self.occurrence_id != result.occurrence_id
            || self.task_execution_run_id != result.task_execution_run_id.unwrap_or(Uuid::nil())
            || self.binding_digest != result.binding_digest
        {
            return Err(ScheduleAdmissionContractError::ScopeMismatch);
        }
        Ok(())
    }

    /// Serialize the validated resolution, rejecting payloads above the protocol bound.
    pub fn encode_json(&self) -> Result<Vec<u8>, ScheduleAdmissionContractError> {
        self.validate()?;
        encode_bounded(self)
    }

    /// Decode a bounded, strict JSON payload and validate its decision shape.
    pub fn decode_json(payload: &[u8]) -> Result<Self, ScheduleAdmissionContractError> {
        let message: Self = decode_bounded(payload)?;
        message.validate()?;
        Ok(message)
    }
}

/// Stable contract validation failures for owner-local Schedule admission messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ScheduleAdmissionContractError {
    /// Message schema version is not supported.
    #[error("unsupported Schedule admission schema version")]
    UnsupportedSchemaVersion,
    /// Required tenant or message identity is absent.
    #[error("Schedule admission message identity is invalid")]
    InvalidIdentity,
    /// The embedded immutable occurrence snapshot is invalid.
    #[error("Schedule admission occurrence snapshot is invalid")]
    InvalidOccurrence,
    /// The embedded canonical Infrastructure binding is invalid.
    #[error("Schedule admission Infrastructure binding is invalid")]
    InvalidInfrastructureBinding,
    /// Related messages or embedded owner scopes do not match.
    #[error("Schedule admission owner-message scope does not match")]
    ScopeMismatch,
    /// Accepted/rejected result fields are inconsistent.
    #[error("Schedule admission result shape is invalid")]
    InvalidResult,
    /// Confirmation/compensation fields are inconsistent.
    #[error("Schedule admission resolution shape is invalid")]
    InvalidResolution,
    /// Serialized message exceeds the protocol byte limit.
    #[error("Schedule admission message exceeds the byte limit")]
    MessageTooLarge,
    /// JSON is malformed or contains fields unknown to this schema version.
    #[error("Schedule admission JSON is malformed or contains unknown fields")]
    InvalidJson,
    /// A validated message could not be serialized.
    #[error("Schedule admission message could not be serialized")]
    SerializationFailed,
}

fn encode_bounded<T: Serialize>(message: &T) -> Result<Vec<u8>, ScheduleAdmissionContractError> {
    let payload = serde_json::to_vec(message)
        .map_err(|_| ScheduleAdmissionContractError::SerializationFailed)?;
    if payload.len() > MAX_SCHEDULE_ADMISSION_MESSAGE_BYTES {
        return Err(ScheduleAdmissionContractError::MessageTooLarge);
    }
    Ok(payload)
}

fn decode_bounded<T: DeserializeOwned>(
    payload: &[u8],
) -> Result<T, ScheduleAdmissionContractError> {
    if payload.len() > MAX_SCHEDULE_ADMISSION_MESSAGE_BYTES {
        return Err(ScheduleAdmissionContractError::MessageTooLarge);
    }
    serde_json::from_slice(payload).map_err(|_| ScheduleAdmissionContractError::InvalidJson)
}

/// Stable validation failures for schedule and occurrence contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ScheduleContractError {
    /// A required canonical target identity or pinned version is absent.
    #[error("schedule target is incomplete")]
    IncompleteTarget,
    /// A content digest is not lowercase SHA-256 hex.
    #[error("schedule snapshot digest is invalid")]
    InvalidDigest,
    /// The retry policy exceeds its explicit resource bounds.
    #[error("schedule retry policy is invalid")]
    InvalidRetryPolicy,
    /// The rule has an invalid identity, time input, or deadline.
    #[error("schedule rule is invalid")]
    InvalidRule,
    /// The overlap policy is outside the bounded concurrency limit.
    #[error("schedule overlap policy is invalid")]
    InvalidOverlapPolicy,
    /// The misfire policy is outside the bounded catch-up limit.
    #[error("schedule misfire policy is invalid")]
    InvalidMisfirePolicy,
    /// Target and rule Project identities do not match.
    #[error("schedule target Project does not match rule Project")]
    TargetProjectMismatch,
    /// The occurrence idempotency key is incomplete.
    #[error("automation occurrence key is invalid")]
    InvalidOccurrenceKey,
    /// The occurrence snapshot disagrees with its immutable rule revision.
    #[error("automation occurrence snapshot is invalid")]
    InvalidOccurrenceSnapshot,
    /// The claim fence is incomplete or has no positive generation.
    #[error("automation occurrence lease fence is invalid")]
    InvalidLeaseFence,
    /// The lease or occurrence deadline has elapsed.
    #[error("automation occurrence lease fence has expired")]
    ExpiredLeaseFence,
    /// The lease extends beyond the occurrence deadline.
    #[error("automation occurrence lease extends beyond deadline")]
    LeasePastDeadline,
}

/// Whether a materialized slot should be dispatched or retained as a skipped fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleOccurrenceDispositionV1 {
    /// The slot is eligible for a worker lease.
    Pending,
    /// The slot is retained for audit but must not start a Run.
    Skipped,
}

/// One bounded, reproducible cron slot and its durable occurrence snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializedScheduleOccurrenceV1 {
    /// Immutable identity, rule snapshot, local label, and offset for this UTC slot.
    pub snapshot: AutomationOccurrenceSnapshotV1,
    /// The initial dispatch state derived from the rule's misfire policy.
    pub disposition: ScheduleOccurrenceDispositionV1,
}

/// A bounded materialization page. Continue after `resume_after_utc` when `has_more` is true.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleMaterializationBatchV1 {
    /// Ordered occurrence snapshots and initial dispatch dispositions.
    pub occurrences: Vec<MaterializedScheduleOccurrenceV1>,
    /// Last emitted UTC slot; use as the exclusive lower bound for the next page.
    pub resume_after_utc: Option<DateTime<Utc>>,
    /// More matching slots exist in the requested interval.
    pub has_more: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotCandidate {
    scheduled_for_utc: DateTime<Utc>,
    scheduled_local: NaiveDateTime,
    utc_offset_seconds: i32,
    shifted_from_gap: bool,
}

/// Stable failures from the pinned recurrence parser and timezone-aware materializer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ScheduleMaterializationError {
    /// The versioned Rule DTO itself is invalid.
    #[error("schedule rule contract is invalid")]
    InvalidRule,
    /// This binary does not contain the parser release pinned by the Rule.
    #[error("schedule cron parser version is not supported by this worker")]
    UnsupportedParserVersion,
    /// This binary does not contain the timezone-data build pinned by the Rule.
    #[error("schedule timezone data version is not supported by this worker")]
    UnsupportedTzdbVersion,
    /// The cron expression is not accepted by the pinned parser.
    #[error("schedule cron expression is invalid")]
    InvalidCronExpression,
    /// The named timezone is not in the pinned IANA database.
    #[error("schedule timezone is not present in the pinned IANA database")]
    UnknownTimeZone,
    /// The materialization interval is empty, reversed, or has an invalid misfire cutoff.
    #[error("schedule materialization interval is invalid")]
    InvalidWindow,
    /// The requested result page exceeds the explicit per-call bound.
    #[error("schedule materialization page bound is invalid")]
    InvalidPageLimit,
    /// Candidate slots or timezone-transition probes exceed the bound; split into smaller windows.
    #[error("schedule materialization window exceeds the bounded scan capacity")]
    WindowTooDense,
    /// A timestamp in the requested interval cannot be represented by Chrono.
    #[error("schedule materialization timestamp is out of range")]
    TimestampOutOfRange,
}

/// Validate a configured recurrence without creating an occurrence snapshot.
///
/// This is suitable for configuration-time API validation, including a hostless Run. A later
/// occurrence materialization still requires an exact remote Branch identity.
pub fn validate_schedule_recurrence(
    rule: &AutomationScheduleRuleRevisionV1,
    after_utc: DateTime<Utc>,
    through_utc: DateTime<Utc>,
    misfire_cutoff_utc: DateTime<Utc>,
    page_limit: usize,
) -> Result<(), ScheduleMaterializationError> {
    collect_schedule_candidates(
        rule,
        after_utc,
        through_utc,
        misfire_cutoff_utc,
        page_limit,
    )
    .map(|_| ())
}

fn collect_schedule_candidates(
    rule: &AutomationScheduleRuleRevisionV1,
    after_utc: DateTime<Utc>,
    through_utc: DateTime<Utc>,
    misfire_cutoff_utc: DateTime<Utc>,
    page_limit: usize,
) -> Result<Vec<SlotCandidate>, ScheduleMaterializationError> {
    if !rule.enabled || rule.validate().is_err() {
        return Err(ScheduleMaterializationError::InvalidRule);
    }
    if rule.parser_version != SCHEDULE_PARSER_VERSION {
        return Err(ScheduleMaterializationError::UnsupportedParserVersion);
    }
    if rule.tzdb_version != SCHEDULE_TZDB_VERSION {
        return Err(ScheduleMaterializationError::UnsupportedTzdbVersion);
    }
    if after_utc >= through_utc
        || misfire_cutoff_utc > through_utc
        || !(1..=256).contains(&page_limit)
    {
        return Err(if (1..=256).contains(&page_limit) {
            ScheduleMaterializationError::InvalidWindow
        } else {
            ScheduleMaterializationError::InvalidPageLimit
        });
    }

    let parser_expression = normalize_cron_expression(&rule.cron_expression);
    let schedule = Schedule::from_str(&parser_expression)
        .map_err(|_| ScheduleMaterializationError::InvalidCronExpression)?;
    let timezone: Tz = rule
        .time_zone
        .parse()
        .map_err(|_| ScheduleMaterializationError::UnknownTimeZone)?;
    let mut candidates = Vec::new();
    let start_local = after_utc.with_timezone(&timezone);

    for scheduled_local in schedule
        .after(&start_local)
        .take(MAX_SCHEDULE_SCAN_SLOTS + 1)
    {
        let scheduled_for_utc = scheduled_local.with_timezone(&Utc);
        if scheduled_for_utc > through_utc {
            break;
        }
        if candidates.len() == MAX_SCHEDULE_SCAN_SLOTS {
            return Err(ScheduleMaterializationError::WindowTooDense);
        }
        if fold_candidate_is_selected(&timezone, &scheduled_local, rule.dst_fold_policy) {
            candidates.push(SlotCandidate {
                scheduled_for_utc,
                scheduled_local: scheduled_local.naive_local(),
                utc_offset_seconds: scheduled_local.offset().fix().local_minus_utc(),
                shifted_from_gap: false,
            });
        }
    }

    if rule.dst_gap_policy == ScheduleDstGapPolicyV1::ShiftForward {
        append_shifted_gap_slots(
            &schedule,
            &timezone,
            after_utc,
            through_utc,
            &mut candidates,
        )?;
    }

    candidates.sort_by_key(|candidate| {
        (
            candidate.scheduled_for_utc,
            candidate.shifted_from_gap,
            candidate.scheduled_local,
        )
    });
    candidates.dedup_by(|right, left| right.scheduled_for_utc == left.scheduled_for_utc);
    Ok(candidates)
}

/// Resolve a versioned Rule into a bounded page of timezone-aware UTC occurrences.
///
/// `after_utc` is exclusive, `through_utc` is inclusive, and `misfire_cutoff_utc` is a
/// caller-owned checkpoint boundary. Slots at or before that boundary follow the frozen
/// Skip/CoalesceLatest/CatchUp policy. The caller must persist the returned page atomically
/// before advancing its checkpoint.
pub fn materialize_schedule_window(
    rule: &AutomationScheduleRuleRevisionV1,
    after_utc: DateTime<Utc>,
    through_utc: DateTime<Utc>,
    misfire_cutoff_utc: DateTime<Utc>,
    materialized_at: DateTime<Utc>,
    page_limit: usize,
) -> Result<ScheduleMaterializationBatchV1, ScheduleMaterializationError> {
    if rule.target.branch_id.is_none() {
        return Err(ScheduleMaterializationError::InvalidRule);
    }
    let candidates = collect_schedule_candidates(
        rule,
        after_utc,
        through_utc,
        misfire_cutoff_utc,
        page_limit,
    )?;

    let newest_misfire = candidates
        .iter()
        .filter(|slot| slot.scheduled_for_utc <= misfire_cutoff_utc)
        .map(|slot| slot.scheduled_for_utc)
        .max();
    let catch_up_slots: Vec<DateTime<Utc>> = match rule.misfire_policy {
        ScheduleMisfirePolicyV1::CatchUp { max_occurrences } => candidates
            .iter()
            .filter(|slot| slot.scheduled_for_utc <= misfire_cutoff_utc)
            .rev()
            .take(usize::from(max_occurrences))
            .map(|slot| slot.scheduled_for_utc)
            .collect(),
        _ => Vec::new(),
    };

    let mut occurrences = Vec::with_capacity(page_limit.min(candidates.len()));
    for candidate in candidates.iter().take(page_limit) {
        let is_misfire = candidate.scheduled_for_utc <= misfire_cutoff_utc;
        let disposition = if !is_misfire {
            ScheduleOccurrenceDispositionV1::Pending
        } else {
            match rule.misfire_policy {
                ScheduleMisfirePolicyV1::Skip => ScheduleOccurrenceDispositionV1::Skipped,
                ScheduleMisfirePolicyV1::CoalesceLatest
                    if Some(candidate.scheduled_for_utc) == newest_misfire =>
                {
                    ScheduleOccurrenceDispositionV1::Pending
                }
                ScheduleMisfirePolicyV1::CoalesceLatest => ScheduleOccurrenceDispositionV1::Skipped,
                ScheduleMisfirePolicyV1::CatchUp { .. }
                    if catch_up_slots.contains(&candidate.scheduled_for_utc) =>
                {
                    ScheduleOccurrenceDispositionV1::Pending
                }
                ScheduleMisfirePolicyV1::CatchUp { .. } => ScheduleOccurrenceDispositionV1::Skipped,
            }
        };
        occurrences.push(MaterializedScheduleOccurrenceV1 {
            snapshot: AutomationOccurrenceSnapshotV1 {
                schema_version: 1,
                occurrence_id: Uuid::new_v4(),
                key: AutomationOccurrenceKey {
                    tenant_id: rule.tenant_id,
                    rule_id: rule.rule_id,
                    rule_version: rule.rule_version,
                    scheduled_for_utc: candidate.scheduled_for_utc,
                },
                run_as_actor_id: rule.run_as_actor_id,
                rule_snapshot: rule.clone(),
                scheduled_local_label: candidate
                    .scheduled_local
                    .format("%Y-%m-%dT%H:%M:%S")
                    .to_string(),
                utc_offset_seconds: candidate.utc_offset_seconds,
                materialized_at,
            },
            disposition,
        });
    }

    Ok(ScheduleMaterializationBatchV1 {
        resume_after_utc: occurrences
            .last()
            .map(|occurrence| occurrence.snapshot.key.scheduled_for_utc),
        has_more: candidates.len() > occurrences.len(),
        occurrences,
    })
}

fn normalize_cron_expression(expression: &str) -> String {
    match expression.split_whitespace().count() {
        5 => format!("0 {expression}"),
        _ => expression.to_owned(),
    }
}

fn fold_candidate_is_selected(
    timezone: &Tz,
    occurrence: &DateTime<Tz>,
    policy: ScheduleDstFoldPolicyV1,
) -> bool {
    match timezone.from_local_datetime(&occurrence.naive_local()) {
        LocalResult::Single(_) => true,
        LocalResult::Ambiguous(first, second) => {
            let (earlier, later) = if first.timestamp() <= second.timestamp() {
                (first, second)
            } else {
                (second, first)
            };
            let selected = match policy {
                ScheduleDstFoldPolicyV1::EarlierInstant => earlier,
                ScheduleDstFoldPolicyV1::LaterInstant => later,
            };
            selected.timestamp() == occurrence.timestamp()
        }
        LocalResult::None => false,
    }
}

fn append_shifted_gap_slots(
    schedule: &Schedule,
    timezone: &Tz,
    after_utc: DateTime<Utc>,
    through_utc: DateTime<Utc>,
    candidates: &mut Vec<SlotCandidate>,
) -> Result<(), ScheduleMaterializationError> {
    let mut probe = after_utc.timestamp();
    let end = through_utc.timestamp();
    let mut probe_count = 0;
    while probe < end {
        probe_count += 1;
        if probe_count > MAX_TRANSITION_PROBES {
            return Err(ScheduleMaterializationError::WindowTooDense);
        }
        let next_probe = probe.saturating_add(MAX_TRANSITION_PROBE_SECONDS).min(end);
        let old_offset = offset_at_utc_second(timezone, probe)?;
        let new_offset = offset_at_utc_second(timezone, next_probe)?;
        if new_offset > old_offset {
            let transition_second =
                locate_offset_transition(timezone, probe, next_probe, old_offset)?;
            let transition = Utc
                .timestamp_opt(transition_second, 0)
                .single()
                .ok_or(ScheduleMaterializationError::TimestampOutOfRange)?;
            if transition > after_utc && transition <= through_utc {
                let missing_local_start = naive_from_wall_second(
                    transition_second.saturating_add(i64::from(old_offset)),
                )?;
                let missing_local_end = naive_from_wall_second(
                    transition_second.saturating_add(i64::from(new_offset)),
                )?;
                let resolved_offset = offset_at_utc_second(timezone, transition_second)?;
                let mut wall_time = missing_local_start;
                while wall_time < missing_local_end {
                    if schedule.includes(DateTime::<Utc>::from_naive_utc_and_offset(wall_time, Utc))
                    {
                        if candidates.len() == MAX_SCHEDULE_SCAN_SLOTS {
                            return Err(ScheduleMaterializationError::WindowTooDense);
                        }
                        candidates.push(SlotCandidate {
                            scheduled_for_utc: transition,
                            scheduled_local: wall_time,
                            utc_offset_seconds: resolved_offset,
                            shifted_from_gap: true,
                        });
                    }
                    wall_time = wall_time
                        .checked_add_signed(Duration::seconds(1))
                        .ok_or(ScheduleMaterializationError::TimestampOutOfRange)?;
                }
            }
            probe = transition_second;
        } else {
            probe = next_probe;
        }
    }
    Ok(())
}

fn offset_at_utc_second(
    timezone: &Tz,
    unix_second: i64,
) -> Result<i32, ScheduleMaterializationError> {
    let instant = Utc
        .timestamp_opt(unix_second, 0)
        .single()
        .ok_or(ScheduleMaterializationError::TimestampOutOfRange)?;
    Ok(timezone
        .offset_from_utc_datetime(&instant.naive_utc())
        .fix()
        .local_minus_utc())
}

fn locate_offset_transition(
    timezone: &Tz,
    mut low: i64,
    mut high: i64,
    old_offset: i32,
) -> Result<i64, ScheduleMaterializationError> {
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if offset_at_utc_second(timezone, middle)? == old_offset {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok(high)
}

fn naive_from_wall_second(unix_second: i64) -> Result<NaiveDateTime, ScheduleMaterializationError> {
    Utc.timestamp_opt(unix_second, 0)
        .single()
        .map(|instant| instant.naive_utc())
        .ok_or(ScheduleMaterializationError::TimestampOutOfRange)
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use uuid::Uuid;

    use super::{
        materialize_schedule_window, AutomationOccurrenceKey, AutomationOccurrenceLeaseFenceV1,
        AutomationOccurrenceSnapshotV1, AutomationScheduleRuleRevisionV1,
        ScheduleAdmissionCompensationReasonV1, ScheduleAdmissionContractError,
        ScheduleAdmissionRejectionCodeV1, ScheduleAdmissionResolutionKindV1, ScheduleContractError,
        ScheduleDstFoldPolicyV1, ScheduleDstGapPolicyV1, ScheduleMaterializationError,
        ScheduleMisfirePolicyV1, ScheduleOccurrenceDispositionV1, ScheduleOverlapPolicyV1,
        SchedulePausePolicyV1, ScheduleRetryPolicyV1, ScheduleRunAdmissionCommandV1,
        ScheduleRunAdmissionOutcomeV1, ScheduleRunAdmissionResolutionV1,
        ScheduleRunAdmissionResultV1, ScheduleTargetV1, MAX_SCHEDULE_ADMISSION_MESSAGE_BYTES,
        SCHEDULE_ADMISSION_SCHEMA_VERSION, SCHEDULE_PARSER_VERSION, SCHEDULE_TZDB_VERSION,
        validate_schedule_recurrence,
    };
    use crate::RuleId;

    fn schedule_target(project_id: Uuid) -> ScheduleTargetV1 {
        ScheduleTargetV1 {
            project_id,
            branch_id: Some(Uuid::new_v4()),
            engineering_run_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            work_item_id: Uuid::new_v4(),
            execution_profile_id: Uuid::new_v4(),
            execution_profile_version: 1,
            execution_profile_digest: "a".repeat(64),
            hook_set_version: 1,
            hook_set_digest: "b".repeat(64),
        }
    }

    fn schedule_rule() -> AutomationScheduleRuleRevisionV1 {
        let project_id = Uuid::new_v4();
        AutomationScheduleRuleRevisionV1 {
            tenant_id: Uuid::new_v4(),
            rule_id: RuleId::new(),
            rule_version: 1,
            run_as_actor_id: Uuid::new_v4(),
            enabled: true,
            project_id,
            cron_expression: "0 9 * * 1-5".to_owned(),
            time_zone: "Asia/Tokyo".to_owned(),
            parser_version: SCHEDULE_PARSER_VERSION.to_owned(),
            tzdb_version: SCHEDULE_TZDB_VERSION.to_owned(),
            dst_gap_policy: ScheduleDstGapPolicyV1::Skip,
            dst_fold_policy: ScheduleDstFoldPolicyV1::EarlierInstant,
            overlap_policy: ScheduleOverlapPolicyV1::QueueOne,
            misfire_policy: ScheduleMisfirePolicyV1::CatchUp { max_occurrences: 3 },
            pause_policy: SchedulePausePolicyV1::CoalesceLatest,
            retry_policy: ScheduleRetryPolicyV1 {
                max_attempts: 3,
                initial_backoff_seconds: 5,
                max_backoff_seconds: 300,
            },
            deadline_seconds: 3600,
            target: schedule_target(project_id),
        }
    }

    #[test]
    fn schedule_revision_requires_pinned_target_and_bounded_policies() {
        assert!(schedule_rule().validate().is_ok());

        let mut invalid = schedule_rule();
        invalid.run_as_actor_id = Uuid::nil();
        assert_eq!(invalid.validate(), Err(ScheduleContractError::InvalidRule));

        let mut invalid = schedule_rule();
        invalid.overlap_policy = ScheduleOverlapPolicyV1::AllowBounded { max_active: 65 };
        assert_eq!(
            invalid.validate(),
            Err(ScheduleContractError::InvalidOverlapPolicy)
        );

        let mut invalid = schedule_rule();
        invalid.target.hook_set_digest = "not-a-digest".to_owned();
        assert_eq!(
            invalid.validate(),
            Err(ScheduleContractError::InvalidDigest)
        );
    }

    #[test]
    fn hostless_rule_can_be_configured_but_cannot_be_materialized() {
        let mut rule = schedule_rule();
        rule.target.branch_id = None;
        assert!(rule.validate().is_ok());
        assert!(validate_schedule_recurrence(
            &rule,
            utc("2026-10-04T23:00:00Z"),
            utc("2026-10-05T01:00:00Z"),
            utc("2026-10-04T23:30:00Z"),
            16,
        )
        .is_ok());

        assert_eq!(
            materialize_schedule_window(
                &rule,
                utc("2026-10-04T23:00:00Z"),
                utc("2026-10-05T01:00:00Z"),
                utc("2026-10-04T23:30:00Z"),
                utc("2026-10-04T23:31:00Z"),
                16,
            ),
            Err(ScheduleMaterializationError::InvalidRule)
        );
    }

    #[test]
    fn occurrence_key_and_snapshot_bind_exact_rule_revision_and_utc_slot() {
        let rule = schedule_rule();
        let scheduled_for_utc = chrono::DateTime::parse_from_rfc3339("2026-10-05T00:00:00Z")
            .expect("fixed timestamp")
            .with_timezone(&chrono::Utc);
        let snapshot = AutomationOccurrenceSnapshotV1 {
            schema_version: 1,
            occurrence_id: Uuid::new_v4(),
            key: AutomationOccurrenceKey {
                tenant_id: rule.tenant_id,
                rule_id: rule.rule_id,
                rule_version: rule.rule_version,
                scheduled_for_utc,
            },
            run_as_actor_id: rule.run_as_actor_id,
            rule_snapshot: rule,
            scheduled_local_label: "2026-10-05T09:00:00".to_owned(),
            utc_offset_seconds: 32_400,
            materialized_at: scheduled_for_utc - Duration::minutes(1),
        };
        assert!(snapshot.validate().is_ok());

        let mut hostless = snapshot.clone();
        hostless.rule_snapshot.target.branch_id = None;
        assert_eq!(
            hostless.validate(),
            Err(ScheduleContractError::InvalidOccurrenceSnapshot)
        );

        let mut wrong_run_as = snapshot.clone();
        wrong_run_as.run_as_actor_id = Uuid::new_v4();
        assert_eq!(
            wrong_run_as.validate(),
            Err(ScheduleContractError::InvalidOccurrenceSnapshot)
        );

        let mut wrong_tenant = snapshot.clone();
        wrong_tenant.key.tenant_id = Uuid::new_v4();
        assert_eq!(
            wrong_tenant.validate(),
            Err(ScheduleContractError::InvalidOccurrenceSnapshot)
        );

        let mut mismatched = snapshot;
        mismatched.key.rule_version += 1;
        assert_eq!(
            mismatched.validate(),
            Err(ScheduleContractError::InvalidOccurrenceSnapshot)
        );
    }

    #[test]
    fn occurrence_fence_rejects_expired_or_over_deadline_lease() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-05T00:00:00Z")
            .expect("fixed timestamp")
            .with_timezone(&chrono::Utc);
        let mut fence = AutomationOccurrenceLeaseFenceV1 {
            tenant_id: Uuid::new_v4(),
            occurrence_id: Uuid::new_v4(),
            fencing_generation: 1,
            lease_owner_id: Uuid::new_v4(),
            lease_expires_at: now + Duration::minutes(1),
            deadline_at: now + Duration::minutes(2),
        };
        assert!(fence.validate(now).is_ok());

        fence.lease_expires_at = now;
        assert_eq!(
            fence.validate(now),
            Err(ScheduleContractError::ExpiredLeaseFence)
        );
        fence.lease_expires_at = now + Duration::minutes(3);
        assert_eq!(
            fence.validate(now),
            Err(ScheduleContractError::LeasePastDeadline)
        );
    }

    fn utc(value: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(value)
            .expect("fixed timestamp")
            .with_timezone(&chrono::Utc)
    }

    #[test]
    fn materializer_pins_parser_and_tzdb_and_emits_bounded_utc_slots() {
        let mut rule = schedule_rule();
        rule.cron_expression = "0 9 * * *".to_owned();
        let batch = materialize_schedule_window(
            &rule,
            utc("2026-10-04T23:00:00Z"),
            utc("2026-10-05T01:00:00Z"),
            utc("2026-10-04T23:30:00Z"),
            utc("2026-10-04T23:31:00Z"),
            16,
        )
        .expect("valid UTC slot");
        assert_eq!(batch.occurrences.len(), 1);
        assert!(!batch.has_more);
        assert_eq!(
            batch.occurrences[0].snapshot.key.scheduled_for_utc,
            utc("2026-10-05T00:00:00Z")
        );
        assert_eq!(
            batch.occurrences[0].snapshot.scheduled_local_label,
            "2026-10-05T09:00:00"
        );
        assert_eq!(batch.occurrences[0].snapshot.utc_offset_seconds, 32_400);
        assert_eq!(
            batch.occurrences[0].disposition,
            ScheduleOccurrenceDispositionV1::Pending
        );

        rule.parser_version = "cron-unsupported".to_owned();
        assert_eq!(
            materialize_schedule_window(
                &rule,
                utc("2026-10-04T23:00:00Z"),
                utc("2026-10-05T01:00:00Z"),
                utc("2026-10-04T23:30:00Z"),
                utc("2026-10-04T23:31:00Z"),
                16,
            ),
            Err(ScheduleMaterializationError::UnsupportedParserVersion)
        );
    }

    #[test]
    fn disabled_rule_cannot_be_materialized() {
        let mut rule = schedule_rule();
        rule.enabled = false;
        assert_eq!(
            materialize_schedule_window(
                &rule,
                utc("2026-10-04T23:00:00Z"),
                utc("2026-10-05T01:00:00Z"),
                utc("2026-10-04T23:30:00Z"),
                utc("2026-10-04T23:31:00Z"),
                16,
            ),
            Err(ScheduleMaterializationError::InvalidRule)
        );
    }

    #[test]
    fn materializer_selects_the_configured_dst_fold_instant() {
        let mut rule = schedule_rule();
        rule.cron_expression = "0 30 1 * * *".to_owned();
        rule.time_zone = "America/New_York".to_owned();
        rule.misfire_policy = ScheduleMisfirePolicyV1::Skip;
        rule.dst_fold_policy = ScheduleDstFoldPolicyV1::EarlierInstant;
        let earlier = materialize_schedule_window(
            &rule,
            utc("2026-11-01T04:00:00Z"),
            utc("2026-11-01T07:00:00Z"),
            utc("2026-11-01T04:00:00Z"),
            utc("2026-11-01T04:00:01Z"),
            16,
        )
        .expect("earlier fold slot");
        assert_eq!(earlier.occurrences.len(), 1);
        assert_eq!(
            earlier.occurrences[0].snapshot.key.scheduled_for_utc,
            utc("2026-11-01T05:30:00Z")
        );

        rule.dst_fold_policy = ScheduleDstFoldPolicyV1::LaterInstant;
        let later = materialize_schedule_window(
            &rule,
            utc("2026-11-01T04:00:00Z"),
            utc("2026-11-01T07:00:00Z"),
            utc("2026-11-01T04:00:00Z"),
            utc("2026-11-01T04:00:01Z"),
            16,
        )
        .expect("later fold slot");
        assert_eq!(later.occurrences.len(), 1);
        assert_eq!(
            later.occurrences[0].snapshot.key.scheduled_for_utc,
            utc("2026-11-01T06:30:00Z")
        );
    }

    #[test]
    fn materializer_skips_or_shifts_dst_gap_slots_and_records_the_local_label() {
        let mut rule = schedule_rule();
        rule.cron_expression = "0 30 2 * * *".to_owned();
        rule.time_zone = "America/New_York".to_owned();
        rule.misfire_policy = ScheduleMisfirePolicyV1::Skip;
        rule.dst_gap_policy = ScheduleDstGapPolicyV1::Skip;
        let skipped = materialize_schedule_window(
            &rule,
            utc("2026-03-08T06:00:00Z"),
            utc("2026-03-08T08:00:00Z"),
            utc("2026-03-08T06:00:00Z"),
            utc("2026-03-08T06:00:01Z"),
            16,
        )
        .expect("gap skip");
        assert!(skipped.occurrences.is_empty());

        rule.dst_gap_policy = ScheduleDstGapPolicyV1::ShiftForward;
        let shifted = materialize_schedule_window(
            &rule,
            utc("2026-03-08T06:00:00Z"),
            utc("2026-03-08T08:00:00Z"),
            utc("2026-03-08T06:00:00Z"),
            utc("2026-03-08T06:00:01Z"),
            16,
        )
        .expect("gap shift");
        assert_eq!(shifted.occurrences.len(), 1);
        assert_eq!(
            shifted.occurrences[0].snapshot.key.scheduled_for_utc,
            utc("2026-03-08T07:00:00Z")
        );
        assert_eq!(
            shifted.occurrences[0].snapshot.scheduled_local_label,
            "2026-03-08T02:30:00"
        );
        assert_eq!(shifted.occurrences[0].snapshot.utc_offset_seconds, -14_400);
    }

    #[test]
    fn materializer_applies_misfire_policy_and_returns_a_resume_cursor() {
        let mut rule = schedule_rule();
        rule.cron_expression = "0 9 * * *".to_owned();
        rule.time_zone = "UTC".to_owned();
        rule.misfire_policy = ScheduleMisfirePolicyV1::CatchUp { max_occurrences: 1 };
        let batch = materialize_schedule_window(
            &rule,
            utc("2026-10-04T08:00:00Z"),
            utc("2026-10-06T10:00:00Z"),
            utc("2026-10-06T10:00:00Z"),
            utc("2026-10-06T10:00:01Z"),
            1,
        )
        .expect("bounded catch-up");
        assert_eq!(batch.occurrences.len(), 1);
        assert!(batch.has_more);
        assert_eq!(batch.resume_after_utc, Some(utc("2026-10-04T09:00:00Z")));
        assert_eq!(
            batch.occurrences[0].disposition,
            ScheduleOccurrenceDispositionV1::Skipped
        );
    }

    #[test]
    fn shift_forward_transition_scan_has_an_independent_window_bound() {
        let mut rule = schedule_rule();
        rule.cron_expression = "0 0 0 1 1 *".to_owned();
        rule.time_zone = "America/New_York".to_owned();
        rule.dst_gap_policy = ScheduleDstGapPolicyV1::ShiftForward;
        assert_eq!(
            materialize_schedule_window(
                &rule,
                utc("2026-01-01T00:00:00Z"),
                utc("2036-01-01T00:00:00Z"),
                utc("2026-01-01T00:00:00Z"),
                utc("2026-01-01T00:00:01Z"),
                16,
            ),
            Err(ScheduleMaterializationError::WindowTooDense)
        );
    }
    fn admission_command() -> ScheduleRunAdmissionCommandV1 {
        use domain_infrastructure::{
            ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, KubernetesDnsLabel, ProviderKind,
            RunInfrastructureBindingSnapshotV2,
        };

        let rule = schedule_rule();
        let scheduled_for_utc = utc("2026-10-05T00:00:00Z");
        let occurrence_id = Uuid::new_v4();
        let occurrence = AutomationOccurrenceSnapshotV1 {
            schema_version: 1,
            occurrence_id,
            key: AutomationOccurrenceKey {
                tenant_id: rule.tenant_id,
                rule_id: rule.rule_id,
                rule_version: rule.rule_version,
                scheduled_for_utc,
            },
            run_as_actor_id: rule.run_as_actor_id,
            rule_snapshot: rule.clone(),
            scheduled_local_label: "2026-10-05T09:00:00".to_owned(),
            utc_offset_seconds: 32_400,
            materialized_at: scheduled_for_utc - Duration::minutes(1),
        };
        let binding = RunInfrastructureBindingSnapshotV2 {
            schema_version: 2,
            tenant_id: rule.tenant_id,
            project_id: rule.project_id,
            branch_id: rule
                .target
                .branch_id
                .expect("admission fixtures require a remote Branch"),
            engineering_run_id: rule.target.engineering_run_id,
            binding_id: Uuid::new_v4(),
            binding_revision: 1,
            host_environment_id: Uuid::new_v4(),
            k3s_cluster_id: Uuid::new_v4(),
            k3s_cluster_generation: 2,
            cluster_placement_policy: ClusterPlacementPolicyV1::SharedPool,
            provider: ProviderKind::MultipassK3sCluster,
            provider_profile_id: Uuid::new_v4(),
            provider_profile_revision: 1,
            provider_profile_digest: [0x11; 32],
            namespace: KubernetesDnsLabel::new("run-04").expect("valid namespace"),
            service_account: KubernetesDnsLabel::new("agent-runtime").expect("valid account"),
            runtime_profile_id: Uuid::new_v4(),
            runtime_profile_revision: 1,
            runtime_profile_digest: [0x22; 32],
            resource_budget: ExecutionResourceBudgetV1 {
                max_rss_bytes: 512 * 1024 * 1024,
                max_cpu_ms: 60_000,
                max_runtime_ms: 600_000,
                max_child_processes: 8,
                max_parallel_tools: 4,
                max_provider_calls: 100,
                max_output_bytes: 1_048_576,
                max_event_buffer_bytes: 262_144,
            },
        };
        ScheduleRunAdmissionCommandV1 {
            schema_version: SCHEDULE_ADMISSION_SCHEMA_VERSION,
            tenant_id: rule.tenant_id,
            request_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            admission_attempt: 1,
            occurrence,
            infrastructure_binding: binding,
        }
    }

    #[test]
    fn admission_command_round_trips_and_pins_canonical_binding_scope() {
        let command = admission_command();
        let payload = command.encode_json().expect("encode valid command");
        let decoded =
            ScheduleRunAdmissionCommandV1::decode_json(&payload).expect("decode valid command");
        assert_eq!(decoded, command);
        assert_eq!(
            command.binding_digest().expect("valid binding digest"),
            command
                .infrastructure_binding
                .digest()
                .expect("canonical binding digest")
        );

        let mut hostless = command.clone();
        hostless.occurrence.rule_snapshot.target.branch_id = None;
        assert_eq!(
            hostless.validate(),
            Err(ScheduleAdmissionContractError::InvalidOccurrence)
        );

        let mut wrong_scope = command;
        wrong_scope.infrastructure_binding.branch_id = Uuid::new_v4();
        assert_eq!(
            wrong_scope.validate(),
            Err(ScheduleAdmissionContractError::ScopeMismatch)
        );
    }

    #[test]
    fn admission_result_and_resolution_must_match_the_same_request_and_binding() {
        let command = admission_command();
        let run_id = Uuid::new_v4();
        let result = ScheduleRunAdmissionResultV1 {
            schema_version: SCHEDULE_ADMISSION_SCHEMA_VERSION,
            tenant_id: command.tenant_id,
            result_id: Uuid::new_v4(),
            request_id: command.request_id,
            admission_attempt: command.admission_attempt,
            correlation_id: command.correlation_id,
            occurrence_id: command.occurrence.occurrence_id,
            outcome: ScheduleRunAdmissionOutcomeV1::Accepted,
            task_execution_run_id: Some(run_id),
            rejection_code: None,
            binding_digest: command.binding_digest().expect("binding digest"),
        };
        assert!(result.validate_for_command(&command).is_ok());
        let result_payload = result.encode_json().expect("encode result");
        assert_eq!(
            ScheduleRunAdmissionResultV1::decode_json(&result_payload).expect("decode result"),
            result
        );

        let resolution = ScheduleRunAdmissionResolutionV1 {
            schema_version: SCHEDULE_ADMISSION_SCHEMA_VERSION,
            tenant_id: result.tenant_id,
            resolution_id: Uuid::new_v4(),
            request_id: result.request_id,
            admission_attempt: result.admission_attempt,
            result_id: result.result_id,
            correlation_id: result.correlation_id,
            occurrence_id: result.occurrence_id,
            task_execution_run_id: run_id,
            binding_digest: result.binding_digest,
            decision: ScheduleAdmissionResolutionKindV1::Confirm,
            compensation_reason: None,
        };
        assert!(resolution.validate_for_result(&result).is_ok());
        let resolution_payload = resolution.encode_json().expect("encode resolution");
        assert_eq!(
            ScheduleRunAdmissionResolutionV1::decode_json(&resolution_payload)
                .expect("decode resolution"),
            resolution
        );

        let compensation = ScheduleRunAdmissionResolutionV1 {
            resolution_id: Uuid::new_v4(),
            decision: ScheduleAdmissionResolutionKindV1::Compensate,
            compensation_reason: Some(ScheduleAdmissionCompensationReasonV1::ScheduleFenceLost),
            ..resolution.clone()
        };
        assert!(compensation.validate_for_result(&result).is_ok());

        let rejected = ScheduleRunAdmissionResultV1 {
            result_id: Uuid::new_v4(),
            outcome: ScheduleRunAdmissionOutcomeV1::Rejected,
            task_execution_run_id: None,
            rejection_code: Some(ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized),
            ..result.clone()
        };
        assert!(rejected.validate_for_command(&command).is_ok());

        let mut stale_resolution = resolution;
        stale_resolution.binding_digest[0] ^= 0xff;
        assert_eq!(
            stale_resolution.validate_for_result(&result),
            Err(ScheduleAdmissionContractError::ScopeMismatch)
        );
    }

    #[test]
    fn admission_wire_rejects_unknown_fields_and_oversized_payloads() {
        let command = admission_command();
        let mut value: serde_json::Value =
            serde_json::from_slice(&command.encode_json().expect("encode command"))
                .expect("parse command value");
        value
            .as_object_mut()
            .expect("command object")
            .insert("unexpected".to_owned(), serde_json::Value::Bool(true));
        let unknown_field = serde_json::to_vec(&value).expect("serialize unknown field");
        assert_eq!(
            ScheduleRunAdmissionCommandV1::decode_json(&unknown_field),
            Err(ScheduleAdmissionContractError::InvalidJson)
        );

        let oversized = vec![b' '; MAX_SCHEDULE_ADMISSION_MESSAGE_BYTES + 1];
        assert_eq!(
            ScheduleRunAdmissionCommandV1::decode_json(&oversized),
            Err(ScheduleAdmissionContractError::MessageTooLarge)
        );
    }
}

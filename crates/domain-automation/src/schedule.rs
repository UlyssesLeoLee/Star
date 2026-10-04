//! Versioned Schedule rule, occurrence snapshot, and lease-fence contracts.
//!
//! This module defines the durable contract used by Automation persistence and Run admission.
//! It intentionally contains no in-memory scheduler or worker; materialization, claiming, and
//! Run creation must be committed by the application layer against PostgreSQL.
//!
//! @cypher schema=1 source_sha256=dd55a3b46c7a35d339a1c6de8395789f9d881fe26dd258e4d78acb9c88f65955
//! MERGE (self:File {path:"crates/domain-automation/src/schedule.rs"})
//! MERGE (module:Symbol {id:"crates/domain-automation/src/schedule.rs::schedule",kind:"module"})
//! MERGE (target:Type {id:"crates/domain-automation/src/schedule.rs::ScheduleTargetV1"})
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
//! MERGE (fence_validate:Symbol {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceLeaseFenceV1::validate",kind:"method"})
//! MERGE (digest:Symbol {id:"crates/domain-automation/src/schedule.rs::is_lower_hex_sha256",kind:"function"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (self)-[:DEFINES]->(target)
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
//! MERGE (self)-[:DEFINES]->(fence_validate)
//! MERGE (self)-[:DEFINES]->(digest)
//! MERGE (target_validate)-[:CALLS]->(digest)
//! MERGE (rule_validate)-[:CALLS]->(target_validate)
//! MERGE (rule_validate)-[:CALLS]->(retry_validate)
//! MERGE (occurrence_validate)-[:CALLS]->(key_validate)
//! MERGE (occurrence_validate)-[:CALLS]->(rule_validate)
//! @endcypher

use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;
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

/// A fully resolved Worktree-first target for one scheduled run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleTargetV1 {
    /// Owning Project.
    pub project_id: Uuid,
    /// Cloud Branch identity.
    pub branch_id: Uuid,
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
            self.branch_id,
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
        AutomationOccurrenceKey, AutomationOccurrenceLeaseFenceV1, AutomationOccurrenceSnapshotV1,
        AutomationScheduleRuleRevisionV1, SCHEDULE_PARSER_VERSION, SCHEDULE_TZDB_VERSION,
        ScheduleContractError, ScheduleDstFoldPolicyV1, ScheduleDstGapPolicyV1,
        ScheduleMaterializationError, ScheduleMisfirePolicyV1, ScheduleOccurrenceDispositionV1,
        ScheduleOverlapPolicyV1, SchedulePausePolicyV1, ScheduleRetryPolicyV1, ScheduleTargetV1,
        materialize_schedule_window,
    };
    use crate::RuleId;

    fn schedule_target(project_id: Uuid) -> ScheduleTargetV1 {
        ScheduleTargetV1 {
            project_id,
            branch_id: Uuid::new_v4(),
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
}

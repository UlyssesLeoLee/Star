//! Versioned Schedule rule, occurrence snapshot, and lease-fence contracts.
//!
//! This module defines the durable contract used by Automation persistence and Run admission.
//! It intentionally contains no in-memory scheduler or worker; materialization, claiming, and
//! Run creation must be committed by the application layer against PostgreSQL.
//!
//! @cypher schema=1 source_sha256=226d2380bc53a0173347f8dc5e5ac0b4b862460468a9bab9677249f24ba5f6c8
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
//! MERGE (key:Type {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceKey"})
//! MERGE (occurrence:Type {id:"crates/domain-automation/src/schedule.rs::AutomationOccurrenceSnapshotV1"})
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
//! MERGE (self)-[:DEFINES]->(key)
//! MERGE (self)-[:DEFINES]->(occurrence)
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::RuleId;

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
        AutomationScheduleRuleRevisionV1, ScheduleContractError, ScheduleDstFoldPolicyV1,
        ScheduleDstGapPolicyV1, ScheduleMisfirePolicyV1, ScheduleOverlapPolicyV1,
        SchedulePausePolicyV1, ScheduleRetryPolicyV1, ScheduleTargetV1,
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
            enabled: true,
            project_id,
            cron_expression: "0 9 * * 1-5".to_owned(),
            time_zone: "Asia/Tokyo".to_owned(),
            parser_version: "star-cron-v1".to_owned(),
            tzdb_version: "2026a".to_owned(),
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
            rule_snapshot: rule,
            scheduled_local_label: "2026-10-05T09:00:00".to_owned(),
            utc_offset_seconds: 32_400,
            materialized_at: scheduled_for_utc - Duration::minutes(1),
        };
        assert!(snapshot.validate().is_ok());

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
}

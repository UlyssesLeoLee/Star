//! ```cypher
//! CREATE
//!   (f:File {name:"domain-hook/src/lib.rs",type:"file",language:"rust"}),
//!   (d:Module {name:"domain_hook",type:"module",language:"rust"}),
//!   (phase:Class {name:"HookPhase",type:"class",language:"rust"}),
//!   (decision:Class {name:"HookDecision",type:"class",language:"rust"}),
//!   (reason:Class {name:"HookReasonCode",type:"class",language:"rust"}),
//!   (lock:Class {name:"RetentionLockState",type:"class",language:"rust"}),
//!   (scope:Class {name:"HookScope",type:"class",language:"rust"}),
//!   (event:Class {name:"HookEventEnvelope",type:"class",language:"rust"}),
//!   (field:Class {name:"HookFactField",type:"class",language:"rust"}),
//!   (operator:Class {name:"HookOperator",type:"class",language:"rust"}),
//!   (value:Class {name:"HookValue",type:"class",language:"rust"}),
//!   (condition:Class {name:"HookCondition",type:"class",language:"rust"}),
//!   (rule:Class {name:"HookRule",type:"class",language:"rust"}),
//!   (policy:Class {name:"HookPolicySnapshot",type:"class",language:"rust"}),
//!   (evaluation:Class {name:"HookEvaluation",type:"class",language:"rust"}),
//!   (api:Variable {name:"EVALUATOR_API_VERSION",type:"variable",language:"rust"}),
//!   (event_schema:Variable {name:"EVENT_SCHEMA_VERSION",type:"variable",language:"rust"}),
//!   (schema:Variable {name:"POLICY_SCHEMA_VERSION",type:"variable",language:"rust"}),
//!   (max_rules:Variable {name:"MAX_RULES_PER_SCOPE",type:"variable",language:"rust"}),
//!   (max_conditions:Variable {name:"MAX_CONDITIONS_PER_RULE",type:"variable",language:"rust"}),
//!   (evaluate:Function {name:"evaluate",type:"function",signature:"fn evaluate(event: &HookEventEnvelope, policy: Option<&HookPolicySnapshot>) -> HookEvaluation",visibility:"pub",language:"rust"}),
//!   (valid_policy:Function {name:"policy_is_valid",type:"function",signature:"fn policy_is_valid(event: &HookEventEnvelope, policy: &HookPolicySnapshot) -> bool",visibility:"private",language:"rust"}),
//!   (valid_condition:Function {name:"condition_is_well_typed",type:"function",signature:"fn condition_is_well_typed(condition: &HookCondition) -> bool",visibility:"private",language:"rust"}),
//!   (matches:Function {name:"condition_matches",type:"function",signature:"fn condition_matches(condition: &HookCondition, event: &HookEventEnvelope) -> bool",visibility:"private",language:"rust"}),
//!   (build:Function {name:"build_evaluation",type:"function",signature:"fn build_evaluation(decision: HookDecision, reason: HookReasonCode, rule_id: Option<[u8; 16]>, policy: Option<&HookPolicySnapshot>, steps: u16) -> HookEvaluation",visibility:"private",language:"rust"}),
//!   (t1:Function {name:"missing_policy_fails_closed",type:"function",language:"rust"}),
//!   (t2:Function {name:"unknown_retention_lock_fails_closed",type:"function",language:"rust"}),
//!   (t3:Function {name:"safe_archive_gate_allows",type:"function",language:"rust"}),
//!   (t4:Function {name:"project_deny_overrides_other_restrictions",type:"function",language:"rust"}),
//!   (t5:Function {name:"worktree_allow_rule_is_rejected",type:"function",language:"rust"}),
//!   (t6:Function {name:"scope_mismatch_fails_closed",type:"function",language:"rust"}),
//!   (t7:Function {name:"wrong_event_schema_fails_closed",type:"function",language:"rust"}),
//!   (t8:Function {name:"tenant_policy_mismatch_fails_closed",type:"function",language:"rust"}),
//!   (event_fixture:Function {name:"safe_event",type:"function",language:"rust"}),
//!   (policy_fixture:Function {name:"safe_policy",type:"function",language:"rust"}),
//!   (slice_len:Function {name:"slice.len",type:"function",language:"rust"}),
//!   (slice_iter:Function {name:"slice.iter",type:"function",language:"rust"}),
//!   (iterator_enumerate:Function {name:"Iterator.enumerate",type:"function",language:"rust"}),
//!   (into_iter:Function {name:"IntoIterator.into_iter",type:"function",language:"rust"}),
//!   (option_is_some_and:Function {name:"Option.is_some_and",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(d),
//!   (d)-[:CONTAINS]->(phase), (d)-[:CONTAINS]->(decision), (d)-[:CONTAINS]->(reason),
//!   (d)-[:CONTAINS]->(lock), (d)-[:CONTAINS]->(scope), (d)-[:CONTAINS]->(event),
//!   (d)-[:CONTAINS]->(field), (d)-[:CONTAINS]->(operator), (d)-[:CONTAINS]->(value),
//!   (d)-[:CONTAINS]->(condition), (d)-[:CONTAINS]->(rule), (d)-[:CONTAINS]->(policy),
//!   (d)-[:CONTAINS]->(evaluation), (d)-[:CONTAINS]->(evaluate),
//!   (d)-[:CONTAINS]->(valid_policy), (d)-[:CONTAINS]->(valid_condition),
//!   (d)-[:CONTAINS]->(matches), (d)-[:CONTAINS]->(build), (d)-[:CONTAINS]->(into_iter),
//!   (d)-[:CONTAINS]->(slice_len), (d)-[:CONTAINS]->(slice_iter),
//!   (d)-[:CONTAINS]->(iterator_enumerate), (d)-[:CONTAINS]->(option_is_some_and),
//!   (d)-[:CONTAINS]->(api), (d)-[:CONTAINS]->(event_schema), (d)-[:CONTAINS]->(schema),
//!   (d)-[:CONTAINS]->(max_rules), (d)-[:CONTAINS]->(max_conditions),
//!   (d)-[:CONTAINS]->(event_fixture), (d)-[:CONTAINS]->(policy_fixture),
//!   (evaluate)-[:CALLS]->(policy_is_valid), (evaluate)-[:CALLS]->(condition_matches),
//!   (evaluate)-[:CALLS]->(build_evaluation), (policy_is_valid)-[:CALLS]->(condition_is_well_typed),
//!   (policy_is_valid)-[:CALLS]->(slice_len), (policy_is_valid)-[:CALLS]->(slice_iter),
//!   (policy_is_valid)-[:CALLS]->(iterator_enumerate), (policy_is_valid)-[:CALLS]->(into_iter),
//!   (evaluate)-[:CALLS]->(into_iter),
//!   (evaluate)-[:CALLS]->(option_is_some_and), (matches)-[:CALLS]->(option_is_some_and),
//!   (policy_is_valid)-[:USES]->(schema), (policy_is_valid)-[:USES]->(api),
//!   (policy_is_valid)-[:USES]->(max_rules), (policy_is_valid)-[:USES]->(max_conditions),
//!   (evaluate)-[:USES]->(event_schema), (build)-[:USES]->(api),
//!   (t1)-[:CALLS]->(event_fixture), (t1)-[:CALLS]->(evaluate),
//!   (t2)-[:CALLS]->(event_fixture), (t2)-[:CALLS]->(evaluate),
//!   (t3)-[:CALLS]->(event_fixture), (t3)-[:CALLS]->(policy_fixture), (t3)-[:CALLS]->(evaluate),
//!   (t4)-[:CALLS]->(event_fixture), (t4)-[:CALLS]->(policy_fixture), (t4)-[:CALLS]->(evaluate),
//!   (t5)-[:CALLS]->(event_fixture), (t5)-[:CALLS]->(policy_fixture), (t5)-[:CALLS]->(evaluate),
//!   (t6)-[:CALLS]->(event_fixture), (t6)-[:CALLS]->(policy_fixture), (t6)-[:CALLS]->(evaluate),
//!   (t7)-[:CALLS]->(event_fixture), (t7)-[:CALLS]->(policy_fixture), (t7)-[:CALLS]->(evaluate),
//!   (t8)-[:CALLS]->(event_fixture), (t8)-[:CALLS]->(policy_fixture), (t8)-[:CALLS]->(evaluate);
//! ```

use serde::{Deserialize, Serialize};

/// Version of the serialized evaluation result contract.
pub const EVALUATOR_API_VERSION: u16 = 1;
/// Version of the typed event envelope accepted by this evaluator.
pub const EVENT_SCHEMA_VERSION: u16 = 1;
/// Version of the typed policy snapshot accepted by this evaluator.
pub const POLICY_SCHEMA_VERSION: u16 = 1;
/// Maximum number of custom rules accepted for one policy scope.
pub const MAX_RULES_PER_SCOPE: usize = 64;
/// Maximum number of flat AND conditions accepted by one rule.
pub const MAX_CONDITIONS_PER_RULE: usize = 16;

/// Synchronous point at which a Rust builtin Hook must finish before cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookPhase {
    /// Before a Worktree archive or physical cleanup lifecycle command.
    BeforeWorktreeArchiveCleanup,
}

/// Authorization outcome returned before the protected Domain Command proceeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookDecision {
    /// No builtin or configured restrictive rule blocked the operation.
    Allow,
    /// The protected operation must not proceed.
    Deny,
    /// A human approval must be recorded before the operation can proceed.
    RequireHuman,
    /// An external condition or drain operation must finish before retry.
    Defer,
}

/// Sanitized, stable classification written to RunEvent/Audit projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookReasonCode {
    /// Every mandatory builtin guard passed and no custom restriction matched.
    AllowedByBuiltinBaseline,
    /// One or more required scope identifiers were absent.
    IncompleteScope,
    /// The producer supplied an unsupported event schema version.
    EventSchemaUnsupported,
    /// The caller did not have current authorization for the operation.
    ActorNotAuthorized,
    /// The expected Worktree lifecycle version was stale.
    LifecycleVersionStale,
    /// Runtime health was false or unavailable.
    RuntimeUnhealthyOrUnknown,
    /// The Git retention-lock observation was not fresh and conflict-free.
    RetentionLockUnusable,
    /// A Run, Agent lease, file claim, or owned process had not drained.
    ExecutionNotDrained,
    /// No authorized immutable policy snapshot was available.
    PolicyUnavailable,
    /// The policy snapshot failed version, scope, bound, or rule validation.
    PolicyInvalid,
    /// A typed custom rule denied the operation.
    RuleDenied,
    /// A typed custom rule requires a human approval.
    HumanApprovalRequired,
    /// A typed custom rule deferred until an external condition is met.
    ExternalConditionPending,
}

/// Freshness/consistency classification for a Git retention-lock observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionLockState {
    /// Observation is current and reports no conflicting lock.
    Fresh,
    /// No observation was obtained.
    Missing,
    /// Observation exceeded its freshness window.
    Stale,
    /// Observation reports an active lock or conflict.
    Conflict,
    /// The observer could not classify lock state.
    Unknown,
}

/// Authenticated identifiers that scope one lifecycle evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookScope {
    /// Tenant UUID in network byte order.
    pub tenant_id: [u8; 16],
    /// Project UUID in network byte order.
    pub project_id: [u8; 16],
    /// Worktree UUID in network byte order.
    pub worktree_id: [u8; 16],
    /// Actor UUID in network byte order.
    pub actor_id: [u8; 16],
    /// Correlation UUID shared with the command and audit event.
    pub correlation_id: [u8; 16],
}

/// Immutable, already-authorized facts supplied by the Worktree Domain Command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEventEnvelope {
    /// Unique producer event UUID in network byte order.
    pub event_id: [u8; 16],
    /// Typed event schema revision.
    pub event_schema_version: u16,
    /// Synchronous operation boundary.
    pub phase: HookPhase,
    /// Tenant, Project, Worktree, Actor, and correlation scope.
    pub scope: HookScope,
    /// Result of the command's current membership/capability preflight.
    pub actor_authorized: bool,
    /// Result of comparing the command's expected lifecycle version.
    pub lifecycle_version_matches: bool,
    /// Current Runtime health; absent is treated as unsafe.
    pub runtime_healthy: Option<bool>,
    /// Freshness result from the Git retention-lock observer.
    pub retention_lock: RetentionLockState,
    /// Number of active Runs associated with the Worktree.
    pub active_run_count: u32,
    /// Number of active Agent leases associated with the Worktree.
    pub active_agent_lease_count: u32,
    /// Number of outstanding file claims associated with the Worktree.
    pub file_claim_count: u32,
    /// Number of PTY/process owners that have not drained.
    pub owned_process_count: u32,
}

/// Finite field vocabulary available to user-authored typed conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookFactField {
    /// Current actor authorization result.
    ActorAuthorized,
    /// Lifecycle version compare result.
    LifecycleVersionMatches,
    /// Runtime health result.
    RuntimeHealthy,
    /// Retention-lock freshness classification.
    RetentionLock,
    /// Active Run count.
    ActiveRunCount,
    /// Active Agent lease count.
    ActiveAgentLeaseCount,
    /// Outstanding file claim count.
    FileClaimCount,
    /// Undrained PTY/process owner count.
    OwnedProcessCount,
}

/// Comparison operators supported for the corresponding typed field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookOperator {
    /// Exact equality.
    Equal,
    /// Exact inequality.
    NotEqual,
    /// Numeric greater-than comparison.
    GreaterThan,
    /// Numeric greater-than-or-equal comparison.
    GreaterThanOrEqual,
    /// Numeric less-than comparison.
    LessThan,
    /// Numeric less-than-or-equal comparison.
    LessThanOrEqual,
}

/// Literal value with a type fixed by the selected condition field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookValue {
    /// Boolean literal for boolean fields.
    Boolean(bool),
    /// Unsigned integer literal for bounded count fields.
    Count(u32),
    /// Lock classification literal for the retention-lock field.
    RetentionLock(RetentionLockState),
}

/// One typed comparison; all conditions in a rule are combined with AND.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookCondition {
    /// Registered event fact to inspect.
    pub field: HookFactField,
    /// Type-compatible comparison operator.
    pub operator: HookOperator,
    /// Literal compared against the event fact.
    pub expected: HookValue,
}

/// One immutable published rule borrowed from the policy snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookRule<'a> {
    /// Stable UUID bytes for audit correlation.
    pub rule_id: [u8; 16],
    /// Higher values win deterministic same-decision tie breaks.
    pub priority: i16,
    /// Disabled rules are retained in policy history but are not evaluated.
    pub enabled: bool,
    /// Restrictive decision; custom rules cannot issue `Allow`.
    pub decision: HookDecision,
    /// Sanitized reason compatible with the selected decision.
    pub reason_code: HookReasonCode,
    /// Bounded typed AND conditions.
    pub conditions: &'a [HookCondition],
}

/// Authorized, versioned policy data pinned for one command evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookPolicySnapshot<'a> {
    /// Typed policy schema version.
    pub schema_version: u16,
    /// Evaluator contract requested by the policy publisher.
    pub evaluator_api_version: u16,
    /// Tenant UUID to which the effective policy belongs.
    pub tenant_id: [u8; 16],
    /// Project UUID to which the baseline belongs.
    pub project_id: [u8; 16],
    /// Worktree UUID for a restrictive overlay, or `None` for Project-only policy.
    pub worktree_id: Option<[u8; 16]>,
    /// Immutable Project policy revision.
    pub project_version: u64,
    /// Immutable Worktree overlay revision, when present.
    pub worktree_version: Option<u64>,
    /// Digest of the fully resolved effective policy.
    pub digest: [u8; 32],
    /// Project-level restrictive rules.
    pub project_rules: &'a [HookRule<'a>],
    /// Worktree-level rules, which may only add restrictions.
    pub worktree_rules: &'a [HookRule<'a>],
}

/// Sanitized evaluation result suitable for append-only RunEvent/Audit storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEvaluation {
    /// Evaluated synchronous boundary.
    pub phase: HookPhase,
    /// Final decision after builtin and restrictive policy evaluation.
    pub decision: HookDecision,
    /// Stable explanation code; contains no raw command/prompt content.
    pub reason_code: HookReasonCode,
    /// Matched custom rule, absent for a builtin guard or baseline allow.
    pub matched_rule_id: Option<[u8; 16]>,
    /// Evaluator contract that produced this result.
    pub evaluator_api_version: u16,
    /// Project policy revision used, if a policy snapshot was available.
    pub project_version: Option<u64>,
    /// Worktree policy revision used, if an overlay was available.
    pub worktree_version: Option<u64>,
    /// Effective policy digest used, if a policy snapshot was available.
    pub policy_digest: Option<[u8; 32]>,
    /// Conditions visited within the fixed per-call work bound.
    pub evaluated_condition_count: u16,
}

/// Evaluates one Worktree cleanup event without allocating or performing I/O.
///
/// Builtin safety checks always run first. Missing or invalid policy data is
/// fail-closed; configured rules can restrict an operation but cannot allow it.
pub fn evaluate(
    event: &HookEventEnvelope,
    policy: Option<&HookPolicySnapshot<'_>>,
) -> HookEvaluation {
    if event.scope.tenant_id == [0; 16]
        || event.scope.project_id == [0; 16]
        || event.scope.worktree_id == [0; 16]
        || event.scope.actor_id == [0; 16]
        || event.scope.correlation_id == [0; 16]
        || event.event_id == [0; 16]
    {
        return build_evaluation(
            HookDecision::Deny,
            HookReasonCode::IncompleteScope,
            None,
            policy,
            0,
        );
    }
    if event.event_schema_version != EVENT_SCHEMA_VERSION {
        return build_evaluation(
            HookDecision::Deny,
            HookReasonCode::EventSchemaUnsupported,
            None,
            policy,
            0,
        );
    }

    // Compiled-in guards run before configurable rules and cannot be disabled or relaxed.
    let builtin_failure = if !event.actor_authorized {
        Some(HookReasonCode::ActorNotAuthorized)
    } else if !event.lifecycle_version_matches {
        Some(HookReasonCode::LifecycleVersionStale)
    } else if event.runtime_healthy != Some(true) {
        Some(HookReasonCode::RuntimeUnhealthyOrUnknown)
    } else if event.retention_lock != RetentionLockState::Fresh {
        Some(HookReasonCode::RetentionLockUnusable)
    } else if event.active_run_count > 0
        || event.active_agent_lease_count > 0
        || event.file_claim_count > 0
        || event.owned_process_count > 0
    {
        Some(HookReasonCode::ExecutionNotDrained)
    } else {
        None
    };
    if let Some(reason) = builtin_failure {
        return build_evaluation(HookDecision::Deny, reason, None, policy, 0);
    }

    let Some(policy) = policy else {
        return build_evaluation(
            HookDecision::Deny,
            HookReasonCode::PolicyUnavailable,
            None,
            None,
            0,
        );
    };
    if !policy_is_valid(event, policy) {
        return build_evaluation(
            HookDecision::Deny,
            HookReasonCode::PolicyInvalid,
            None,
            Some(policy),
            0,
        );
    }

    let mut decision = HookDecision::Allow;
    let mut reason = HookReasonCode::AllowedByBuiltinBaseline;
    let mut matched_rule_id = None;
    let mut matched_priority = i16::MIN;
    let mut evaluated_condition_count = 0_u16;

    for rules in [policy.project_rules, policy.worktree_rules] {
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let mut matched = true;
            for condition in rule.conditions {
                evaluated_condition_count += 1;
                if !condition_matches(condition, event) {
                    matched = false;
                    break;
                }
            }
            if !matched {
                continue;
            }

            let candidate_rank = match rule.decision {
                HookDecision::Allow => 0,
                HookDecision::Defer => 1,
                HookDecision::RequireHuman => 2,
                HookDecision::Deny => 3,
            };
            let current_rank = match decision {
                HookDecision::Allow => 0,
                HookDecision::Defer => 1,
                HookDecision::RequireHuman => 2,
                HookDecision::Deny => 3,
            };
            let stable_tie_break = candidate_rank == current_rank
                && (rule.priority > matched_priority
                    || (rule.priority == matched_priority
                        && matched_rule_id.is_some_and(|current| rule.rule_id < current)));
            if candidate_rank > current_rank || stable_tie_break {
                decision = rule.decision;
                reason = rule.reason_code;
                matched_rule_id = Some(rule.rule_id);
                matched_priority = rule.priority;
            }
        }
    }

    build_evaluation(
        decision,
        reason,
        matched_rule_id,
        Some(policy),
        evaluated_condition_count,
    )
}

fn policy_is_valid(event: &HookEventEnvelope, policy: &HookPolicySnapshot<'_>) -> bool {
    if policy.schema_version != POLICY_SCHEMA_VERSION
        || policy.evaluator_api_version != EVALUATOR_API_VERSION
        || policy.tenant_id != event.scope.tenant_id
        || policy.project_id != event.scope.project_id
        || policy.project_version == 0
        || policy.digest == [0; 32]
        || policy.project_rules.len() > MAX_RULES_PER_SCOPE
        || policy.worktree_rules.len() > MAX_RULES_PER_SCOPE
    {
        return false;
    }
    match (policy.worktree_id, policy.worktree_version) {
        (None, None) if policy.worktree_rules.is_empty() => {}
        (Some(worktree_id), Some(version))
            if worktree_id == event.scope.worktree_id && version > 0 => {}
        _ => return false,
    }

    for rules in [policy.project_rules, policy.worktree_rules] {
        for rule in rules {
            if rule.rule_id == [0; 16]
                || rule.decision == HookDecision::Allow
                || rule.conditions.len() > MAX_CONDITIONS_PER_RULE
                || !matches!(
                    (rule.decision, rule.reason_code),
                    (HookDecision::Deny, HookReasonCode::RuleDenied)
                        | (
                            HookDecision::RequireHuman,
                            HookReasonCode::HumanApprovalRequired
                        )
                        | (
                            HookDecision::Defer,
                            HookReasonCode::ExternalConditionPending
                        )
                )
            {
                return false;
            }
            for condition in rule.conditions {
                if !condition_is_well_typed(condition) {
                    return false;
                }
            }
        }
    }

    for (project_index, project_rule) in policy.project_rules.iter().enumerate() {
        for later_project_rule in &policy.project_rules[project_index + 1..] {
            if project_rule.rule_id == later_project_rule.rule_id {
                return false;
            }
        }
        for worktree_rule in policy.worktree_rules {
            if project_rule.rule_id == worktree_rule.rule_id {
                return false;
            }
        }
    }
    for (worktree_index, worktree_rule) in policy.worktree_rules.iter().enumerate() {
        for later_worktree_rule in &policy.worktree_rules[worktree_index + 1..] {
            if worktree_rule.rule_id == later_worktree_rule.rule_id {
                return false;
            }
        }
    }
    true
}

fn condition_is_well_typed(condition: &HookCondition) -> bool {
    matches!(
        (condition.field, condition.operator, condition.expected),
        (
            HookFactField::ActorAuthorized
                | HookFactField::LifecycleVersionMatches
                | HookFactField::RuntimeHealthy,
            HookOperator::Equal | HookOperator::NotEqual,
            HookValue::Boolean(_),
        ) | (
            HookFactField::RetentionLock,
            HookOperator::Equal | HookOperator::NotEqual,
            HookValue::RetentionLock(_),
        ) | (
            HookFactField::ActiveRunCount
                | HookFactField::ActiveAgentLeaseCount
                | HookFactField::FileClaimCount
                | HookFactField::OwnedProcessCount,
            HookOperator::Equal
                | HookOperator::NotEqual
                | HookOperator::GreaterThan
                | HookOperator::GreaterThanOrEqual
                | HookOperator::LessThan
                | HookOperator::LessThanOrEqual,
            HookValue::Count(_),
        )
    )
}

fn condition_matches(condition: &HookCondition, event: &HookEventEnvelope) -> bool {
    match (condition.field, condition.operator, condition.expected) {
        (HookFactField::ActorAuthorized, HookOperator::Equal, HookValue::Boolean(expected)) => {
            event.actor_authorized == expected
        }
        (HookFactField::ActorAuthorized, HookOperator::NotEqual, HookValue::Boolean(expected)) => {
            event.actor_authorized != expected
        }
        (
            HookFactField::LifecycleVersionMatches,
            HookOperator::Equal,
            HookValue::Boolean(expected),
        ) => event.lifecycle_version_matches == expected,
        (
            HookFactField::LifecycleVersionMatches,
            HookOperator::NotEqual,
            HookValue::Boolean(expected),
        ) => event.lifecycle_version_matches != expected,
        (HookFactField::RuntimeHealthy, HookOperator::Equal, HookValue::Boolean(expected)) => {
            event.runtime_healthy == Some(expected)
        }
        (HookFactField::RuntimeHealthy, HookOperator::NotEqual, HookValue::Boolean(expected)) => {
            event
                .runtime_healthy
                .is_some_and(|actual| actual != expected)
        }
        (HookFactField::RetentionLock, HookOperator::Equal, HookValue::RetentionLock(expected)) => {
            event.retention_lock == expected
        }
        (
            HookFactField::RetentionLock,
            HookOperator::NotEqual,
            HookValue::RetentionLock(expected),
        ) => event.retention_lock != expected,
        (field, operator, HookValue::Count(expected)) => {
            let actual = match field {
                HookFactField::ActiveRunCount => event.active_run_count,
                HookFactField::ActiveAgentLeaseCount => event.active_agent_lease_count,
                HookFactField::FileClaimCount => event.file_claim_count,
                HookFactField::OwnedProcessCount => event.owned_process_count,
                _ => return false,
            };
            match operator {
                HookOperator::Equal => actual == expected,
                HookOperator::NotEqual => actual != expected,
                HookOperator::GreaterThan => actual > expected,
                HookOperator::GreaterThanOrEqual => actual >= expected,
                HookOperator::LessThan => actual < expected,
                HookOperator::LessThanOrEqual => actual <= expected,
            }
        }
        _ => false,
    }
}

fn build_evaluation(
    decision: HookDecision,
    reason_code: HookReasonCode,
    matched_rule_id: Option<[u8; 16]>,
    policy: Option<&HookPolicySnapshot<'_>>,
    evaluated_condition_count: u16,
) -> HookEvaluation {
    let (project_version, worktree_version, policy_digest) = match policy {
        Some(policy) => (
            Some(policy.project_version),
            policy.worktree_version,
            Some(policy.digest),
        ),
        None => (None, None, None),
    };
    HookEvaluation {
        phase: HookPhase::BeforeWorktreeArchiveCleanup,
        decision,
        reason_code,
        matched_rule_id,
        evaluator_api_version: EVALUATOR_API_VERSION,
        project_version,
        worktree_version,
        policy_digest,
        evaluated_condition_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_event() -> HookEventEnvelope {
        HookEventEnvelope {
            event_id: [6; 16],
            event_schema_version: EVENT_SCHEMA_VERSION,
            phase: HookPhase::BeforeWorktreeArchiveCleanup,
            scope: HookScope {
                tenant_id: [1; 16],
                project_id: [2; 16],
                worktree_id: [3; 16],
                actor_id: [4; 16],
                correlation_id: [5; 16],
            },
            actor_authorized: true,
            lifecycle_version_matches: true,
            runtime_healthy: Some(true),
            retention_lock: RetentionLockState::Fresh,
            active_run_count: 0,
            active_agent_lease_count: 0,
            file_claim_count: 0,
            owned_process_count: 0,
        }
    }

    fn safe_policy<'a>(
        project_rules: &'a [HookRule<'a>],
        worktree_rules: &'a [HookRule<'a>],
    ) -> HookPolicySnapshot<'a> {
        HookPolicySnapshot {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [9; 32],
            project_rules,
            worktree_rules,
        }
    }

    #[test]
    fn missing_policy_fails_closed() {
        let result = evaluate(&safe_event(), None);
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::PolicyUnavailable);
    }

    #[test]
    fn unknown_retention_lock_fails_closed() {
        let mut event = safe_event();
        event.retention_lock = RetentionLockState::Unknown;
        let result = evaluate(&event, None);
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::RetentionLockUnusable);
    }

    #[test]
    fn safe_archive_gate_allows() {
        let event = safe_event();
        let policy = safe_policy(&[], &[]);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Allow);
        assert_eq!(result.reason_code, HookReasonCode::AllowedByBuiltinBaseline);
    }

    #[test]
    fn project_deny_overrides_other_restrictions() {
        let event = safe_event();
        let condition = HookCondition {
            field: HookFactField::ActiveRunCount,
            operator: HookOperator::Equal,
            expected: HookValue::Count(0),
        };
        let project_rule = HookRule {
            rule_id: [7; 16],
            priority: 1,
            enabled: true,
            decision: HookDecision::Deny,
            reason_code: HookReasonCode::RuleDenied,
            conditions: &[condition],
        };
        let worktree_rule = HookRule {
            rule_id: [8; 16],
            priority: 10,
            enabled: true,
            decision: HookDecision::Defer,
            reason_code: HookReasonCode::ExternalConditionPending,
            conditions: &[],
        };
        let project_rules = [project_rule];
        let worktree_rules = [worktree_rule];
        let policy = safe_policy(&project_rules, &worktree_rules);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.matched_rule_id, Some([7; 16]));
        assert_eq!(result.evaluated_condition_count, 1);
    }

    #[test]
    fn worktree_allow_rule_is_rejected() {
        let event = safe_event();
        let allow_rule = HookRule {
            rule_id: [6; 16],
            priority: 1,
            enabled: true,
            decision: HookDecision::Allow,
            reason_code: HookReasonCode::AllowedByBuiltinBaseline,
            conditions: &[],
        };
        let worktree_rules = [allow_rule];
        let policy = safe_policy(&[], &worktree_rules);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::PolicyInvalid);
    }

    #[test]
    fn scope_mismatch_fails_closed() {
        let event = safe_event();
        let mut policy = safe_policy(&[], &[]);
        policy.project_id = [99; 16];
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::PolicyInvalid);
    }

    #[test]
    fn wrong_event_schema_fails_closed() {
        let mut event = safe_event();
        event.event_schema_version = EVENT_SCHEMA_VERSION + 1;
        let policy = safe_policy(&[], &[]);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::EventSchemaUnsupported);
    }

    #[test]
    fn tenant_policy_mismatch_fails_closed() {
        let event = safe_event();
        let mut policy = safe_policy(&[], &[]);
        policy.tenant_id = [99; 16];
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::PolicyInvalid);
    }
}

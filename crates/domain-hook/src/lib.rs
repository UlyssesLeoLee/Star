//! ```cypher
//! CREATE
//!   (f:File {name:"domain-hook/src/lib.rs",type:"file",language:"rust"}),
//!   (d:Module {name:"domain_hook",type:"module",language:"rust"}),
//!   (event:Class {name:"HookEventEnvelope",type:"class",language:"rust"}),
//!   (phase:Class {name:"HookPhase",type:"class",language:"rust"}),
//!   (decision:Class {name:"HookDecision",type:"class",language:"rust"}),
//!   (reason:Class {name:"HookReasonCode",type:"class",language:"rust"}),
//!   (lock:Class {name:"RetentionLockState",type:"class",language:"rust"}),
//!   (scope:Class {name:"HookScope",type:"class",language:"rust"}),
//!   (field:Class {name:"HookFactField",type:"class",language:"rust"}),
//!   (operator:Class {name:"HookOperator",type:"class",language:"rust"}),
//!   (value:Class {name:"HookValue",type:"class",language:"rust"}),
//!   (condition:Class {name:"HookCondition",type:"class",language:"rust"}),
//!   (rule:Class {name:"HookRule",type:"class",language:"rust"}),
//!   (document:Class {name:"HookPolicyDocument",type:"class",language:"rust"}),
//!   (verified:Class {name:"VerifiedHookPolicySnapshot",type:"class",language:"rust"}),
//!   (validation_error:Class {name:"HookPolicyValidationError",type:"class",language:"rust"}),
//!   (evaluation:Class {name:"HookEvaluation",type:"class",language:"rust"}),
//!   (api:Variable {name:"EVALUATOR_API_VERSION",type:"variable",language:"rust"}),
//!   (event_schema:Variable {name:"EVENT_SCHEMA_VERSION",type:"variable",language:"rust"}),
//!   (schema:Variable {name:"POLICY_SCHEMA_VERSION",type:"variable",language:"rust"}),
//!   (max_rules:Variable {name:"MAX_RULES_PER_SCOPE",type:"variable",language:"rust"}),
//!   (max_conditions:Variable {name:"MAX_CONDITIONS_PER_RULE",type:"variable",language:"rust"}),
//!   (max_bytes:Variable {name:"MAX_POLICY_DOCUMENT_BYTES",type:"variable",language:"rust"}),
//!   (evaluate:Function {name:"evaluate",type:"function",signature:"fn evaluate(event: &HookEventEnvelope, policy: Option<&VerifiedHookPolicySnapshot>) -> HookEvaluation",visibility:"pub",language:"rust"}),
//!   (decode:Function {name:"HookPolicyDocument::decode_and_verify",type:"function",signature:"fn decode_and_verify(bytes: &[u8]) -> Result<VerifiedHookPolicySnapshot, HookPolicyValidationError>",visibility:"pub",language:"rust"}),
//!   (verify:Function {name:"HookPolicyDocument::verify",type:"function",signature:"fn verify(self) -> Result<VerifiedHookPolicySnapshot, HookPolicyValidationError>",visibility:"pub",language:"rust"}),
//!   (digest:Function {name:"HookPolicyDocument::computed_digest",type:"function",signature:"fn computed_digest(&self) -> Result<[u8; 32], HookPolicyValidationError>",visibility:"pub",language:"rust"}),
//!   (valid_policy:Function {name:"policy_is_valid",type:"function",signature:"fn policy_is_valid(event: &HookEventEnvelope, policy: &VerifiedHookPolicySnapshot) -> bool",visibility:"private",language:"rust"}),
//!   (valid_rules:Function {name:"rules_are_valid",type:"function",signature:"fn rules_are_valid(rules: &[HookRule]) -> bool",visibility:"private",language:"rust"}),
//!   (valid_condition:Function {name:"condition_is_well_typed",type:"function",signature:"fn condition_is_well_typed(condition: &HookCondition) -> bool",visibility:"private",language:"rust"}),
//!   (matches:Function {name:"condition_matches",type:"function",signature:"fn condition_matches(condition: &HookCondition, event: &HookEventEnvelope) -> bool",visibility:"private",language:"rust"}),
//!   (build:Function {name:"build_evaluation",type:"function",signature:"fn build_evaluation(decision: HookDecision, reason: HookReasonCode, rule_id: Option<[u8; 16]>, policy: Option<&VerifiedHookPolicySnapshot>, steps: u16) -> HookEvaluation",visibility:"private",language:"rust"}),
//!   (tests:Module {name:"domain_hook_tests",type:"module",language:"rust"}),
//!   (fixture_event:Function {name:"safe_event",type:"function",language:"rust"}),
//!   (fixture_policy:Function {name:"safe_policy",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(d), (d)-[:CONTAINS]->(event), (d)-[:CONTAINS]->(phase),
//!   (d)-[:CONTAINS]->(decision), (d)-[:CONTAINS]->(reason), (d)-[:CONTAINS]->(lock),
//!   (d)-[:CONTAINS]->(scope), (d)-[:CONTAINS]->(field), (d)-[:CONTAINS]->(operator),
//!   (d)-[:CONTAINS]->(value), (d)-[:CONTAINS]->(condition), (d)-[:CONTAINS]->(rule),
//!   (d)-[:CONTAINS]->(document), (d)-[:CONTAINS]->(verified),
//!   (d)-[:CONTAINS]->(validation_error), (d)-[:CONTAINS]->(evaluation),
//!   (d)-[:CONTAINS]->(evaluate), (d)-[:CONTAINS]->(decode),
//!   (d)-[:CONTAINS]->(verify), (d)-[:CONTAINS]->(digest),
//!   (d)-[:CONTAINS]->(valid_policy), (d)-[:CONTAINS]->(valid_rules),
//!   (d)-[:CONTAINS]->(valid_condition), (d)-[:CONTAINS]->(matches),
//!   (d)-[:CONTAINS]->(build), (d)-[:CONTAINS]->(api), (d)-[:CONTAINS]->(event_schema),
//!   (d)-[:CONTAINS]->(schema), (d)-[:CONTAINS]->(max_rules),
//!   (d)-[:CONTAINS]->(max_conditions), (d)-[:CONTAINS]->(max_bytes),
//!   (d)-[:CONTAINS]->(tests), (tests)-[:CONTAINS]->(fixture_event),
//!   (tests)-[:CONTAINS]->(fixture_policy), (document)-[:CALLS]->(digest),
//!   (decode)-[:CALLS]->(verify), (decode)-[:USES]->(max_bytes),
//!   (verify)-[:CALLS]->(valid_rules), (evaluate)-[:CALLS]->(valid_policy),
//!   (evaluate)-[:CALLS]->(matches), (evaluate)-[:CALLS]->(build),
//!   (valid_rules)-[:CALLS]->(valid_condition), (valid_policy)-[:USES]->(schema),
//!   (valid_policy)-[:USES]->(api), (valid_rules)-[:USES]->(max_rules),
//!   (valid_rules)-[:USES]->(max_conditions),
//!   (evaluate)-[:USES]->(event_schema), (build)-[:USES]->(api);
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
/// Maximum encoded policy document size accepted by store/API adapters before parsing.
pub const MAX_POLICY_DOCUMENT_BYTES: usize = 65_536;

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
#[serde(deny_unknown_fields)]
pub struct HookCondition {
    /// Registered event fact to inspect.
    pub field: HookFactField,
    /// Type-compatible comparison operator.
    pub operator: HookOperator,
    /// Literal compared against the event fact.
    pub expected: HookValue,
}

/// One bounded typed rule stored in a published policy document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookRule {
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
    pub conditions: Vec<HookCondition>,
}

/// Serialized policy payload loaded from the authoritative policy store.
///
/// This is an untrusted transport value until [`HookPolicyDocument::verify`]
/// checks its bounds, restrictive-only actions, scope metadata, and digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookPolicyDocument {
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
    pub project_rules: Vec<HookRule>,
    /// Worktree-level rules, which may only add restrictions.
    pub worktree_rules: Vec<HookRule>,
}

/// Validated immutable policy snapshot. Its private fields prevent callers from
/// changing policy facts after verification; the evaluator borrows it without
/// allocating or performing I/O.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedHookPolicySnapshot {
    schema_version: u16,
    evaluator_api_version: u16,
    tenant_id: [u8; 16],
    project_id: [u8; 16],
    worktree_id: Option<[u8; 16]>,
    project_version: u64,
    worktree_version: Option<u64>,
    digest: [u8; 32],
    project_rules: Vec<HookRule>,
    worktree_rules: Vec<HookRule>,
}

/// Rejection reason for a policy document before it enters the runtime cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPolicyValidationError {
    /// Serialized bytes are not a valid policy document.
    InvalidEncoding,
    /// The document uses an unsupported policy schema revision.
    UnsupportedSchema,
    /// The document targets a different evaluator contract.
    UnsupportedEvaluator,
    /// Required tenant, Project, or Worktree identifiers are missing/invalid.
    InvalidScope,
    /// Scope version metadata is inconsistent or zero.
    InvalidVersion,
    /// The supplied digest is missing or does not match the canonical payload.
    InvalidDigest,
    /// A rule contains an invalid action, reason, condition, or duplicate ID.
    InvalidRule,
    /// The encoded document, rule count, or condition count exceeds its bound.
    PolicyTooLarge,
}

#[derive(Serialize)]
struct HookPolicyDigestInput<'a> {
    schema_version: u16,
    evaluator_api_version: u16,
    tenant_id: [u8; 16],
    project_id: [u8; 16],
    worktree_id: Option<[u8; 16]>,
    project_version: u64,
    worktree_version: Option<u64>,
    project_rules: &'a [HookRule],
    worktree_rules: &'a [HookRule],
}

impl HookPolicyDocument {
    /// Parses a bounded JSON document and verifies it before runtime use.
    ///
    /// Store and API adapters should use this method for untrusted bytes rather
    /// than calling `serde_json::from_slice` directly.
    pub fn decode_and_verify(
        bytes: &[u8],
    ) -> Result<VerifiedHookPolicySnapshot, HookPolicyValidationError> {
        if bytes.len() > MAX_POLICY_DOCUMENT_BYTES {
            return Err(HookPolicyValidationError::PolicyTooLarge);
        }
        let document: HookPolicyDocument = serde_json::from_slice(bytes)
            .map_err(|_| HookPolicyValidationError::InvalidEncoding)?;
        document.verify()
    }

    /// Computes the canonical SHA-256 digest of the policy payload.
    ///
    /// This is called at load/publish time, outside the evaluator hot path.
    pub fn computed_digest(&self) -> Result<[u8; 32], HookPolicyValidationError> {
        if self.project_rules.len() > MAX_RULES_PER_SCOPE
            || self.worktree_rules.len() > MAX_RULES_PER_SCOPE
            || self
                .project_rules
                .iter()
                .chain(self.worktree_rules.iter())
                .any(|rule| rule.conditions.len() > MAX_CONDITIONS_PER_RULE)
        {
            return Err(HookPolicyValidationError::PolicyTooLarge);
        }
        let payload = HookPolicyDigestInput {
            schema_version: self.schema_version,
            evaluator_api_version: self.evaluator_api_version,
            tenant_id: self.tenant_id,
            project_id: self.project_id,
            worktree_id: self.worktree_id,
            project_version: self.project_version,
            worktree_version: self.worktree_version,
            project_rules: &self.project_rules,
            worktree_rules: &self.worktree_rules,
        };
        let bytes =
            serde_json::to_vec(&payload).map_err(|_| HookPolicyValidationError::InvalidDigest)?;
        if bytes.len() > MAX_POLICY_DOCUMENT_BYTES {
            return Err(HookPolicyValidationError::PolicyTooLarge);
        }
        Ok(Sha256::digest(bytes).into())
    }

    /// Validates an untrusted serialized policy once before it is cached.
    pub fn verify(self) -> Result<VerifiedHookPolicySnapshot, HookPolicyValidationError> {
        if self.schema_version != POLICY_SCHEMA_VERSION {
            return Err(HookPolicyValidationError::UnsupportedSchema);
        }
        if self.evaluator_api_version != EVALUATOR_API_VERSION {
            return Err(HookPolicyValidationError::UnsupportedEvaluator);
        }
        if self.tenant_id == [0; 16]
            || self.project_id == [0; 16]
            || self.worktree_id.is_some_and(|id| id == [0; 16])
        {
            return Err(HookPolicyValidationError::InvalidScope);
        }
        if self.project_version == 0
            || self.worktree_id.is_some() != self.worktree_version.is_some()
            || self.worktree_version.is_some_and(|version| version == 0)
            || (self.worktree_id.is_none() && !self.worktree_rules.is_empty())
        {
            return Err(HookPolicyValidationError::InvalidVersion);
        }
        if self.project_rules.len() > MAX_RULES_PER_SCOPE
            || self.worktree_rules.len() > MAX_RULES_PER_SCOPE
        {
            return Err(HookPolicyValidationError::PolicyTooLarge);
        }
        if !rules_are_valid(&self.project_rules)
            || !rules_are_valid(&self.worktree_rules)
            || rule_ids_overlap(&self.project_rules, &self.worktree_rules)
        {
            return Err(HookPolicyValidationError::InvalidRule);
        }
        if self.digest == [0; 32] || self.computed_digest()? != self.digest {
            return Err(HookPolicyValidationError::InvalidDigest);
        }

        Ok(VerifiedHookPolicySnapshot {
            schema_version: self.schema_version,
            evaluator_api_version: self.evaluator_api_version,
            tenant_id: self.tenant_id,
            project_id: self.project_id,
            worktree_id: self.worktree_id,
            project_version: self.project_version,
            worktree_version: self.worktree_version,
            digest: self.digest,
            project_rules: self.project_rules,
            worktree_rules: self.worktree_rules,
        })
    }
}

impl VerifiedHookPolicySnapshot {
    /// Published Project baseline version pinned by this snapshot.
    pub fn project_version(&self) -> u64 {
        self.project_version
    }

    /// Published Worktree restrictive-overlay version, if present.
    pub fn worktree_version(&self) -> Option<u64> {
        self.worktree_version
    }

    /// Canonical digest used to identify the complete immutable policy.
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
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
    policy: Option<&VerifiedHookPolicySnapshot>,
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

    for rule in policy
        .project_rules
        .iter()
        .chain(policy.worktree_rules.iter())
    {
        if !rule.enabled {
            continue;
        }
        let mut matched = true;
        for condition in &rule.conditions {
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

    build_evaluation(
        decision,
        reason,
        matched_rule_id,
        Some(policy),
        evaluated_condition_count,
    )
}

fn policy_is_valid(event: &HookEventEnvelope, policy: &VerifiedHookPolicySnapshot) -> bool {
    if policy.schema_version != POLICY_SCHEMA_VERSION
        || policy.evaluator_api_version != EVALUATOR_API_VERSION
        || policy.tenant_id != event.scope.tenant_id
        || policy.project_id != event.scope.project_id
    {
        return false;
    }
    match (policy.worktree_id, policy.worktree_version) {
        (None, None) if policy.worktree_rules.is_empty() => {}
        (Some(worktree_id), Some(version))
            if worktree_id == event.scope.worktree_id && version > 0 => {}
        _ => return false,
    }

    true
}

fn rules_are_valid(rules: &[HookRule]) -> bool {
    if rules.len() > MAX_RULES_PER_SCOPE {
        return false;
    }
    for (index, rule) in rules.iter().enumerate() {
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
            || rule
                .conditions
                .iter()
                .any(|condition| !condition_is_well_typed(condition))
            || rules[..index]
                .iter()
                .any(|prior| prior.rule_id == rule.rule_id)
        {
            return false;
        }
    }
    true
}

fn rule_ids_overlap(project_rules: &[HookRule], worktree_rules: &[HookRule]) -> bool {
    project_rules.iter().any(|project| {
        worktree_rules
            .iter()
            .any(|worktree| worktree.rule_id == project.rule_id)
    })
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
    policy: Option<&VerifiedHookPolicySnapshot>,
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

    fn safe_policy(
        project_rules: Vec<HookRule>,
        worktree_rules: Vec<HookRule>,
    ) -> VerifiedHookPolicySnapshot {
        safe_policy_for_project([2; 16], project_rules, worktree_rules)
    }

    fn safe_policy_for_project(
        project_id: [u8; 16],
        project_rules: Vec<HookRule>,
        worktree_rules: Vec<HookRule>,
    ) -> VerifiedHookPolicySnapshot {
        let mut document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id,
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [0; 32],
            project_rules,
            worktree_rules,
        };
        document.digest = document.computed_digest().unwrap();
        document.verify().unwrap()
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
        let policy = safe_policy(vec![], vec![]);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Allow);
        assert_eq!(result.reason_code, HookReasonCode::AllowedByBuiltinBaseline);
    }

    #[test]
    fn bounded_policy_loader_verifies_serialized_snapshot() {
        let mut document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [0; 32],
            project_rules: vec![],
            worktree_rules: vec![],
        };
        document.digest = document.computed_digest().unwrap();
        let bytes = serde_json::to_vec(&document).unwrap();
        let verified = HookPolicyDocument::decode_and_verify(&bytes).unwrap();
        assert_eq!(verified.digest(), document.digest);
        assert_eq!(verified.project_version(), 1);
        assert_eq!(verified.worktree_version(), Some(1));
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
            conditions: vec![condition],
        };
        let worktree_rule = HookRule {
            rule_id: [8; 16],
            priority: 10,
            enabled: true,
            decision: HookDecision::Defer,
            reason_code: HookReasonCode::ExternalConditionPending,
            conditions: vec![],
        };
        let policy = safe_policy(vec![project_rule], vec![worktree_rule]);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.matched_rule_id, Some([7; 16]));
        assert_eq!(result.evaluated_condition_count, 1);
    }

    #[test]
    fn worktree_allow_rule_is_rejected_before_evaluation() {
        let allow_rule = HookRule {
            rule_id: [6; 16],
            priority: 1,
            enabled: true,
            decision: HookDecision::Allow,
            reason_code: HookReasonCode::AllowedByBuiltinBaseline,
            conditions: vec![],
        };
        let mut document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [0; 32],
            project_rules: vec![],
            worktree_rules: vec![allow_rule],
        };
        document.digest = document.computed_digest().unwrap();
        assert_eq!(
            document.verify(),
            Err(HookPolicyValidationError::InvalidRule)
        );
    }

    #[test]
    fn scope_mismatch_fails_closed() {
        let event = safe_event();
        let policy = safe_policy_for_project([99; 16], vec![], vec![]);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::PolicyInvalid);
    }

    #[test]
    fn wrong_event_schema_fails_closed() {
        let mut event = safe_event();
        event.event_schema_version = EVENT_SCHEMA_VERSION + 1;
        let policy = safe_policy(vec![], vec![]);
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::EventSchemaUnsupported);
    }

    #[test]
    fn tenant_policy_mismatch_fails_closed() {
        let event = safe_event();
        let mut document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [99; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [0; 32],
            project_rules: vec![],
            worktree_rules: vec![],
        };
        document.digest = document.computed_digest().unwrap();
        let policy = document.verify().unwrap();
        let result = evaluate(&event, Some(&policy));
        assert_eq!(result.decision, HookDecision::Deny);
        assert_eq!(result.reason_code, HookReasonCode::PolicyInvalid);
    }

    #[test]
    fn changed_policy_payload_is_rejected_even_when_digest_is_present() {
        let mut document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [0; 32],
            project_rules: vec![],
            worktree_rules: vec![],
        };
        document.digest = document.computed_digest().unwrap();
        document.project_version += 1;
        assert_eq!(
            document.verify(),
            Err(HookPolicyValidationError::InvalidDigest)
        );
    }

    #[test]
    fn oversized_policy_is_rejected_before_digest_serialization() {
        let too_many_rules = vec![
            HookRule {
                rule_id: [7; 16],
                priority: 0,
                enabled: false,
                decision: HookDecision::Deny,
                reason_code: HookReasonCode::RuleDenied,
                conditions: vec![],
            };
            MAX_RULES_PER_SCOPE + 1
        ];
        let document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [9; 32],
            project_rules: too_many_rules,
            worktree_rules: vec![],
        };
        assert_eq!(
            document.verify(),
            Err(HookPolicyValidationError::PolicyTooLarge)
        );
    }

    #[test]
    fn oversized_encoded_policy_is_rejected_before_json_parse() {
        let bytes = vec![b' '; MAX_POLICY_DOCUMENT_BYTES + 1];
        assert_eq!(
            HookPolicyDocument::decode_and_verify(&bytes),
            Err(HookPolicyValidationError::PolicyTooLarge)
        );
    }

    #[test]
    fn unknown_policy_fields_are_rejected() {
        let mut document = HookPolicyDocument {
            schema_version: POLICY_SCHEMA_VERSION,
            evaluator_api_version: EVALUATOR_API_VERSION,
            tenant_id: [1; 16],
            project_id: [2; 16],
            worktree_id: Some([3; 16]),
            project_version: 1,
            worktree_version: Some(1),
            digest: [0; 32],
            project_rules: vec![],
            worktree_rules: vec![],
        };
        document.digest = document.computed_digest().unwrap();
        let encoded = serde_json::to_string(&document).unwrap();
        let extended = encoded.replacen(
            "{\"schema_version\"",
            "{\"unexpected\":true,\"schema_version\"",
            1,
        );
        assert_eq!(
            HookPolicyDocument::decode_and_verify(extended.as_bytes()),
            Err(HookPolicyValidationError::InvalidEncoding)
        );
    }
}

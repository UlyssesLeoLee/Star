//! Run-local Engineering Loop control with bounded state and immutable execution snapshots.
//!
//! The controller records only digests, tool categories, validation identity, and resource
//! counters. It never stores prompts, model reasoning, or unbounded logs. Durable persistence,
//! Run authorization, and global fair scheduling remain application-layer responsibilities.
//!
//! @cypher schema=1 source_sha256=7328dc108233e24ab36e1ee1bd21bab868bbb8c6810e03e9472d8cf9a3b1666e
//! MERGE (self:File {path:"crates/domain-agent/src/engineering_loop.rs"})
//! MERGE (module:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::engineering_loop",kind:"module"})
//! MERGE (controller:Type {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController"})
//! MERGE (binding:Type {id:"crates/domain-agent/src/engineering_loop.rs::LoopRunBinding"})
//! MERGE (identity:Type {id:"crates/domain-agent/src/engineering_loop.rs::LoopSnapshotIdentity"})
//! MERGE (observation:Type {id:"crates/domain-agent/src/engineering_loop.rs::LoopObservation"})
//! MERGE (receipt:Type {id:"crates/domain-agent/src/engineering_loop.rs::LoopIterationReceipt"})
//! MERGE (pool:Type {id:"crates/domain-agent/src/engineering_loop.rs::ToolPermitPool"})
//! MERGE (permit:Type {id:"crates/domain-agent/src/engineering_loop.rs::ToolPermit"})
//! MERGE (tests:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::tests",kind:"module"})
//! MERGE (new:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::new",kind:"function"})
//! MERGE (begin:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::begin_iteration",kind:"method"})
//! MERGE (finish:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::finish_iteration",kind:"method"})
//! MERGE (stop:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::request_stop",kind:"method"})
//! MERGE (drain:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::observe_drain",kind:"method"})
//! MERGE (tool_admit:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::try_acquire_tool",kind:"method"})
//! MERGE (pool_admit:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::ToolPermitPool::try_acquire",kind:"method"})
//! MERGE (permit_drop:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::ToolPermit::drop",kind:"method"})
//! MERGE (test_case:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::tests::validation_pass_waits_for_review_and_never_completes_the_task",kind:"test"})
//! MERGE (resource_reason:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::resource_stop_reason",kind:"method"})
//! MERGE (stop_counts:Symbol {id:"crates/domain-agent/src/engineering_loop.rs::EngineeringLoopController::stop_with_counts",kind:"method"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (self)-[:DEFINES]->(controller)
//! MERGE (self)-[:DEFINES]->(binding)
//! MERGE (self)-[:DEFINES]->(identity)
//! MERGE (self)-[:DEFINES]->(observation)
//! MERGE (self)-[:DEFINES]->(receipt)
//! MERGE (self)-[:DEFINES]->(pool)
//! MERGE (self)-[:DEFINES]->(permit)
//! MERGE (self)-[:DEFINES]->(tests)
//! MERGE (self)-[:DEFINES]->(new)
//! MERGE (self)-[:DEFINES]->(begin)
//! MERGE (self)-[:DEFINES]->(finish)
//! MERGE (self)-[:DEFINES]->(stop)
//! MERGE (self)-[:DEFINES]->(drain)
//! MERGE (self)-[:DEFINES]->(tool_admit)
//! MERGE (self)-[:DEFINES]->(pool_admit)
//! MERGE (self)-[:DEFINES]->(permit_drop)
//! MERGE (self)-[:IMPORTS]->(profile_file:File {path:"crates/domain-agent/src/execution_profile.rs"})
//! MERGE (new)-[:CALLS]->(scope_check:Symbol {id:"crates/domain-agent/src/execution_profile.rs::VerifiedAgentExecutionProfile::validate_for_scope",kind:"method"})
//! MERGE (new)-[:USES_TYPE]->(verified_profile:Type {id:"crates/domain-agent/src/execution_profile.rs::VerifiedAgentExecutionProfile"})
//! MERGE (new)-[:USES_TYPE]->(profile_scope:Type {id:"crates/domain-agent/src/execution_profile.rs::ExecutionProfileScope"})
//! MERGE (controller)-[:USES_TYPE]->(verified_profile)
//! MERGE (controller)-[:USES_TYPE]->(profile_scope)
//! MERGE (finish)-[:USES_TYPE]->(observation)
//! MERGE (begin)-[:USES_TYPE]->(identity)
//! MERGE (tool_admit)-[:CALLS]->(pool_admit)
//! MERGE (begin)-[:CALLS]->(stop_counts)
//! MERGE (begin)-[:CALLS]->(resource_reason)
//! MERGE (finish)-[:CALLS]->(resource_reason)
//! MERGE (stop)-[:CALLS]->(stop_counts)
//! MERGE (drain)-[:CALLS]->(stop_counts)
//! MERGE (permit_drop)-[:WRITES]->(pool)
//! MERGE (tests)-[:TESTS]->(self)
//! MERGE (test_case)-[:TESTS]->(self)
//! @endcypher

use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::execution_profile::{
    ExecutionProfileScope, LoopBudgetSnapshot, ProviderReference, ResourceBudgetSnapshot,
    VerifiedAgentExecutionProfile,
};

const LOOP_PHASES: [EngineeringLoopPhase; 5] = [
    EngineeringLoopPhase::Plan,
    EngineeringLoopPhase::Act,
    EngineeringLoopPhase::Observe,
    EngineeringLoopPhase::Verify,
    EngineeringLoopPhase::Decide,
];

/// The fixed five-step record shape for one completed iteration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineeringLoopPhase {
    /// Select the next bounded action from the immutable task contract.
    Plan,
    /// Execute admitted provider/tool work.
    Act,
    /// Capture bounded observable results.
    Observe,
    /// Run the profile-pinned independent validator when requested.
    Verify,
    /// Continue, stop, or hand control to review.
    Decide,
}

/// Run and Task identity plus the immutable contract/acceptance snapshot digests.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopRunBinding {
    /// Tenant/Project/Worktree scope checked against the verified Profile.
    pub scope: ExecutionProfileScope,
    /// Durable TaskExecutionRun identity.
    pub run_id: Uuid,
    /// Run-owned Task Card identity.
    pub task_id: Uuid,
    /// Exact local checkout bound to the Run.
    pub worktree_id: Uuid,
    /// Digest of the immutable Task Contract.
    pub contract_digest: String,
    /// Digest of the immutable acceptance snapshot.
    pub acceptance_digest: String,
}

/// Run-time identities that must remain byte-for-byte stable across the loop.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopSnapshotIdentity {
    /// Verified AgentExecutionProfile payload digest.
    pub profile_digest: String,
    /// Task Contract digest.
    pub contract_digest: String,
    /// Acceptance snapshot digest.
    pub acceptance_digest: String,
    /// Effective HookSet identity.
    pub hook_set_id: Uuid,
    /// Effective HookSet revision.
    pub hook_set_version: u64,
    /// Effective inherited HookSet digest.
    pub hook_set_digest: String,
    /// Independent validation provider identity.
    pub validation_provider: ProviderReference,
    /// Validation suite digest.
    pub validation_suite_digest: String,
    /// Validation toolchain/environment digest.
    pub validation_toolchain_digest: String,
}

/// Bounded runtime counters sampled at an iteration boundary.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopResourceSample {
    /// Monotonic elapsed Run time in milliseconds.
    pub elapsed_ms: u64,
    /// Cumulative CPU time in milliseconds.
    pub cpu_ms: u64,
    /// Peak resident memory observed so far in bytes.
    pub peak_rss_bytes: u64,
    /// Currently active child processes.
    pub active_child_processes: u16,
    /// Cumulative provider calls.
    pub provider_calls: u32,
    /// Cumulative captured output bytes.
    pub captured_output_bytes: u64,
    /// Current event-buffer occupancy in bytes.
    pub event_buffer_bytes: u32,
}

/// Coarse tool category; raw commands and arguments are deliberately excluded.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopToolCategory {
    /// Read-only repository or workspace access.
    Read,
    /// Repository or workspace mutation.
    Write,
    /// Validation/build/test command.
    Validation,
    /// Explicit external integration call.
    External,
    /// Other policy-approved tool.
    Other,
}

/// Independent validator result bound to the frozen acceptance and toolchain.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopValidationResult {
    /// Exact provider implementation and configuration identity.
    pub provider: ProviderReference,
    /// Validation suite digest.
    pub suite_digest: String,
    /// Validation toolchain/environment digest.
    pub toolchain_digest: String,
    /// Acceptance snapshot digest actually checked.
    pub acceptance_digest: String,
    /// Validator verdict.
    pub verdict: ValidationVerdict,
    /// Digest of the bounded validation evidence bundle.
    pub evidence_digest: String,
}

/// Validation verdict from the profile-pinned independent validator.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationVerdict {
    /// All pinned acceptance criteria passed.
    Passed,
    /// One or more pinned acceptance criteria failed.
    Failed,
}

/// Bounded iteration boundary supplied after the agent/tool work has finished.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopObservation {
    /// Iteration sequence returned by the begin call.
    pub sequence: u32,
    /// Current immutable Run identity observed by the caller.
    pub snapshot: LoopSnapshotIdentity,
    /// Digest of the bounded, user-visible work summary.
    pub summary_digest: String,
    /// Digest of the produced evidence references; no raw logs are stored here.
    pub evidence_digest: String,
    /// Stable digest of the observable worktree/task state used for progress detection.
    pub progress_fingerprint: [u8; 32],
    /// One coarse tool category for the iteration, when tools were used.
    pub tool_category: Option<LoopToolCategory>,
    /// Independent validation result, if this iteration invoked validation.
    pub validation: Option<LoopValidationResult>,
    /// Cumulative resource counters at this boundary.
    pub resources: LoopResourceSample,
}

/// Structured, bounded receipt returned to an application-layer event/outbox writer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopIterationReceipt {
    /// One-based iteration sequence.
    pub sequence: u32,
    /// Fixed-size phase sequence; callers may attach timing in their durable event store.
    pub phases: [EngineeringLoopPhase; 5],
    /// Bounded summary digest.
    pub summary_digest: String,
    /// Evidence bundle digest.
    pub evidence_digest: String,
    /// Progress fingerprint.
    pub progress_fingerprint: [u8; 32],
    /// Coarse tool category.
    pub tool_category: Option<LoopToolCategory>,
    /// Independent validation verdict.
    pub validation: Option<ValidationVerdict>,
    /// Resource sample.
    pub resources: LoopResourceSample,
}

/// Loop lifecycle state. AwaitingReview never mutates the Work Item itself.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineeringLoopState {
    /// Created and not yet admitted to its first iteration.
    Ready,
    /// An iteration may be admitted or completed.
    Running,
    /// Independent validation passed; Task status remains unchanged pending review.
    AwaitingReview,
    /// Stop requested and child/tool resources are draining.
    Draining,
    /// Stop reason is final and all child/tool resources are drained.
    Stopped,
    /// Drain deadline expired with outstanding child/tool resources.
    Incomplete,
}

/// Explicit stop reason recorded in the loop stop receipt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopStopReason {
    /// The configured iteration limit was reached.
    MaxIterations,
    /// The configured no-progress limit was reached.
    NoProgress,
    /// A two-state or longer oscillation was observed.
    Oscillation,
    /// Wall-clock budget expired.
    WallClockBudget,
    /// CPU budget was exhausted.
    CpuBudget,
    /// Memory budget was exceeded.
    MemoryBudget,
    /// Child process limit was exceeded.
    ChildProcessBudget,
    /// Concurrent tool limit was exceeded.
    ToolConcurrencyBudget,
    /// Provider-call budget was exhausted.
    ProviderCallBudget,
    /// Captured output budget was exceeded.
    OutputBudget,
    /// Event buffer capacity was exceeded.
    EventBufferBudget,
    /// A pinned Run snapshot changed during execution.
    SnapshotChanged,
    /// Current authorization was revoked.
    AuthorizationRevoked,
    /// User or system cancellation was requested.
    Cancelled,
    /// An observation regressed or exceeded its protocol bounds.
    InvalidObservation,
}

/// Child and tool resource drain status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopDrainStatus {
    /// No active child processes or tool permits remain.
    Complete,
    /// At least one child process or tool permit remains active.
    Pending,
    /// The drain deadline expired with resources still active.
    Incomplete,
}

/// Immutable terminal stop/drain receipt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopStopReceipt {
    /// Why the loop stopped.
    pub reason: LoopStopReason,
    /// Child/tool drain state.
    pub drain: LoopDrainStatus,
    /// Outstanding child processes.
    pub outstanding_child_processes: u16,
    /// Outstanding tool permits.
    pub outstanding_tool_permits: u16,
}

/// Decision returned after beginning or completing an iteration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopStepResult {
    /// Bounded iteration receipt; absent when no iteration completed.
    pub receipt: Option<LoopIterationReceipt>,
    /// Current controller decision.
    pub decision: LoopDecision,
}

/// Current scheduling/stop decision. This is a run-local state, not a persisted Run status.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "decision", content = "detail", rename_all = "snake_case")]
pub enum LoopDecision {
    /// The next bounded iteration was admitted.
    IterationStarted(u32),
    /// The loop may continue within its immutable snapshots and budgets.
    Continue,
    /// Validation passed; task completion still requires the Task workflow/review gate.
    AwaitingReview,
    /// The loop has stopped or is draining.
    Stopped(LoopStopReceipt),
}

/// Errors that indicate caller protocol misuse and do not perform a partial iteration.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum EngineeringLoopError {
    /// The Run binding is incomplete or inconsistent.
    #[error("invalid Engineering Loop binding")]
    InvalidBinding,
    /// The verified Profile is outside the Run's tenant/project/worktree scope.
    #[error("AgentExecutionProfile scope does not match the Run")]
    ProfileScopeMismatch,
    /// The configured independent validator is the same implementation as the agent.
    #[error("validation provider must be independent of the agent provider")]
    ValidatorNotIndependent,
    /// A digest is absent or is not 64 hexadecimal characters.
    #[error("invalid Engineering Loop digest")]
    InvalidDigest,
    /// An iteration is already active or no iteration is active.
    #[error("invalid Engineering Loop iteration transition")]
    InvalidIterationTransition,
    /// A sample contains a counter lower than the previous boundary.
    #[error("Engineering Loop counters must be monotonic")]
    CounterRegression,
    /// Validation arrived before all child/tool work drained.
    #[error("validation cannot pass while child or tool work remains active")]
    ValidationBeforeDrain,
    /// Reported tool permit count did not match the atomic Run-local pool.
    #[error("reported tool drain count does not match active permits")]
    DrainObservationMismatch,
}

/// Immediate admission outcome for bounded concurrent tool work.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ToolAdmissionError {
    /// Run-local concurrency budget is currently full; no unbounded wait queue is created.
    #[error("tool concurrency limit reached; apply backpressure")]
    AtCapacity,
    /// The loop is no longer accepting new tools.
    #[error("Engineering Loop is not accepting new tool work")]
    LoopNotRunning,
}

/// Atomic, per-Run semaphore with immediate backpressure and no waiting queue.
#[derive(Debug)]
pub struct ToolPermitPool {
    limit: u16,
    active: AtomicU16,
}

impl ToolPermitPool {
    fn new(limit: u16) -> Self {
        Self {
            limit,
            active: AtomicU16::new(0),
        }
    }

    /// Try to acquire one tool slot without allocating or enqueueing a waiter.
    pub fn try_acquire(self: &Arc<Self>) -> Result<ToolPermit, ToolAdmissionError> {
        let mut current = self.active.load(Ordering::Acquire);
        loop {
            if current >= self.limit {
                return Err(ToolAdmissionError::AtCapacity);
            }
            match self.active.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(ToolPermit {
                        pool: Arc::clone(self),
                    })
                }
                Err(observed) => current = observed,
            }
        }
    }

    /// Current active tool slots.
    #[must_use]
    pub fn active(&self) -> u16 {
        self.active.load(Ordering::Acquire)
    }

    /// Configured per-Run concurrency ceiling.
    #[must_use]
    pub fn limit(&self) -> u16 {
        self.limit
    }
}

/// RAII lease that releases its Run-local concurrency slot on drop.
#[derive(Debug)]
pub struct ToolPermit {
    pool: Arc<ToolPermitPool>,
}

impl Drop for ToolPermit {
    fn drop(&mut self) {
        let previous = self.pool.active.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0, "ToolPermitPool active counter underflow");
    }
}

/// Run-local Engineering Loop controller; it stores only bounded counters and two fingerprints.
#[derive(Debug)]
pub struct EngineeringLoopController {
    snapshot: LoopSnapshotIdentity,
    loop_budget: LoopBudgetSnapshot,
    resource_budget: ResourceBudgetSnapshot,
    tool_permits: Arc<ToolPermitPool>,
    state: EngineeringLoopState,
    completed_iterations: u32,
    pending_iteration: Option<u32>,
    no_progress_iterations: u32,
    previous_fingerprint: Option<[u8; 32]>,
    before_previous_fingerprint: Option<[u8; 32]>,
    last_resources: LoopResourceSample,
    stop_receipt: Option<LoopStopReceipt>,
}

impl EngineeringLoopController {
    /// Create a finite loop bound to one verified Profile and immutable Run/task snapshots.
    pub fn new(
        profile: &VerifiedAgentExecutionProfile,
        binding: LoopRunBinding,
    ) -> Result<Self, EngineeringLoopError> {
        if binding.run_id.is_nil()
            || binding.task_id.is_nil()
            || binding.worktree_id.is_nil()
            || binding.scope.worktree_id != Some(binding.worktree_id)
            || !valid_digest(&binding.contract_digest)
            || !valid_digest(&binding.acceptance_digest)
        {
            return Err(EngineeringLoopError::InvalidBinding);
        }
        profile
            .validate_for_scope(&binding.scope)
            .map_err(|_| EngineeringLoopError::ProfileScopeMismatch)?;

        let document = profile.document();
        let execution = &document.profile;
        if same_provider_implementation(&execution.agent_provider, &execution.validation.provider) {
            return Err(EngineeringLoopError::ValidatorNotIndependent);
        }
        if !valid_digest(&document.content_digest)
            || !valid_digest(&execution.hook_set.effective_digest)
            || !valid_digest(&execution.validation.suite_digest)
            || !valid_digest(&execution.validation.toolchain_digest)
        {
            return Err(EngineeringLoopError::InvalidDigest);
        }

        let snapshot = LoopSnapshotIdentity {
            profile_digest: document.content_digest.clone(),
            contract_digest: binding.contract_digest,
            acceptance_digest: binding.acceptance_digest,
            hook_set_id: execution.hook_set.hook_set_id,
            hook_set_version: execution.hook_set.version,
            hook_set_digest: execution.hook_set.effective_digest.clone(),
            validation_provider: execution.validation.provider.clone(),
            validation_suite_digest: execution.validation.suite_digest.clone(),
            validation_toolchain_digest: execution.validation.toolchain_digest.clone(),
        };
        Ok(Self {
            snapshot,
            loop_budget: execution.loop_budget.clone(),
            resource_budget: execution.resource_budget.clone(),
            tool_permits: Arc::new(ToolPermitPool::new(
                execution.resource_budget.max_parallel_tools,
            )),
            state: EngineeringLoopState::Ready,
            completed_iterations: 0,
            pending_iteration: None,
            no_progress_iterations: 0,
            previous_fingerprint: None,
            before_previous_fingerprint: None,
            last_resources: LoopResourceSample::default(),
            stop_receipt: None,
        })
    }

    /// Expected immutable identities for every iteration boundary.
    #[must_use]
    pub fn expected_snapshot(&self) -> &LoopSnapshotIdentity {
        &self.snapshot
    }

    /// Current Run-local state.
    #[must_use]
    pub fn state(&self) -> EngineeringLoopState {
        self.state
    }

    /// Completed iteration count; bounded by the verified Profile.
    #[must_use]
    pub fn completed_iterations(&self) -> u32 {
        self.completed_iterations
    }

    /// Current immutable stop/drain receipt, if the loop has stopped.
    #[must_use]
    pub fn stop_receipt(&self) -> Option<LoopStopReceipt> {
        self.stop_receipt
    }

    /// Atomically acquire a bounded tool slot. Capacity exhaustion returns immediately.
    pub fn try_acquire_tool(&self) -> Result<ToolPermit, ToolAdmissionError> {
        if !matches!(self.state, EngineeringLoopState::Running) {
            return Err(ToolAdmissionError::LoopNotRunning);
        }
        self.tool_permits.try_acquire()
    }

    /// Admit an iteration only while its snapshots and budgets remain valid.
    pub fn begin_iteration(
        &mut self,
        observed: &LoopSnapshotIdentity,
        elapsed_ms: u64,
    ) -> Result<LoopDecision, EngineeringLoopError> {
        match self.state {
            EngineeringLoopState::Ready | EngineeringLoopState::Running => {}
            EngineeringLoopState::AwaitingReview => return Ok(LoopDecision::AwaitingReview),
            _ => {
                return Ok(LoopDecision::Stopped(
                    self.stop_receipt.expect("terminal state has receipt"),
                ))
            }
        }
        if self.pending_iteration.is_some() {
            return Err(EngineeringLoopError::InvalidIterationTransition);
        }
        if observed != &self.snapshot {
            self.stop_with_current_resources(LoopStopReason::SnapshotChanged);
            return Ok(self.current_stopped_decision());
        }
        if elapsed_ms < self.last_resources.elapsed_ms {
            self.stop_with_current_resources(LoopStopReason::InvalidObservation);
            return Ok(self.current_stopped_decision());
        }
        self.last_resources.elapsed_ms = elapsed_ms;
        let wall_limit = self
            .loop_budget
            .max_wall_clock_ms
            .min(self.resource_budget.max_runtime_ms);
        if elapsed_ms >= wall_limit {
            self.stop_with_current_resources(LoopStopReason::WallClockBudget);
            return Ok(self.current_stopped_decision());
        }
        if self.completed_iterations >= self.loop_budget.max_iterations {
            self.stop_with_current_resources(LoopStopReason::MaxIterations);
            return Ok(self.current_stopped_decision());
        }
        if self.last_resources.cpu_ms >= self.resource_budget.max_cpu_ms {
            self.stop_with_current_resources(LoopStopReason::CpuBudget);
            return Ok(self.current_stopped_decision());
        }
        if self.last_resources.provider_calls
            >= self
                .loop_budget
                .max_provider_calls
                .min(self.resource_budget.max_provider_calls)
        {
            self.stop_with_current_resources(LoopStopReason::ProviderCallBudget);
            return Ok(self.current_stopped_decision());
        }
        if self.last_resources.captured_output_bytes >= self.resource_budget.max_output_bytes {
            self.stop_with_current_resources(LoopStopReason::OutputBudget);
            return Ok(self.current_stopped_decision());
        }
        if self.last_resources.event_buffer_bytes >= self.resource_budget.max_event_buffer_bytes {
            self.stop_with_current_resources(LoopStopReason::EventBufferBudget);
            return Ok(self.current_stopped_decision());
        }
        let sequence = self.completed_iterations + 1;
        self.pending_iteration = Some(sequence);
        self.state = EngineeringLoopState::Running;
        Ok(LoopDecision::IterationStarted(sequence))
    }

    /// Finish the currently admitted iteration and return a bounded event receipt.
    pub fn finish_iteration(
        &mut self,
        observation: LoopObservation,
    ) -> Result<LoopStepResult, EngineeringLoopError> {
        if self.state != EngineeringLoopState::Running
            || self.pending_iteration != Some(observation.sequence)
        {
            return Err(EngineeringLoopError::InvalidIterationTransition);
        }
        self.pending_iteration = None;
        if observation.snapshot != self.snapshot {
            self.stop_with_sample(LoopStopReason::SnapshotChanged, observation.resources);
            return Ok(LoopStepResult {
                receipt: None,
                decision: self.current_stopped_decision(),
            });
        }
        if !valid_digest(&observation.summary_digest) || !valid_digest(&observation.evidence_digest)
        {
            self.stop_with_sample(LoopStopReason::InvalidObservation, observation.resources);
            return Ok(LoopStepResult {
                receipt: None,
                decision: self.current_stopped_decision(),
            });
        }
        if !self.sample_is_monotonic(observation.resources) {
            self.stop_with_sample(LoopStopReason::InvalidObservation, observation.resources);
            return Ok(LoopStepResult {
                receipt: None,
                decision: self.current_stopped_decision(),
            });
        }
        self.last_resources = observation.resources;

        if let Some(reason) = self.resource_stop_reason(observation.resources) {
            self.stop_with_current_resources(reason);
            return Ok(LoopStepResult {
                receipt: None,
                decision: self.current_stopped_decision(),
            });
        }
        if let Some(validation) = &observation.validation {
            if !self.validation_identity_matches(validation)
                || !valid_digest(&validation.evidence_digest)
            {
                self.stop_with_current_resources(LoopStopReason::SnapshotChanged);
                return Ok(LoopStepResult {
                    receipt: None,
                    decision: self.current_stopped_decision(),
                });
            }
            if validation.verdict == ValidationVerdict::Passed
                && (observation.resources.active_child_processes != 0
                    || self.tool_permits.active() != 0)
            {
                self.stop_with_current_resources(LoopStopReason::InvalidObservation);
                return Err(EngineeringLoopError::ValidationBeforeDrain);
            }
        }

        let receipt = LoopIterationReceipt {
            sequence: observation.sequence,
            phases: LOOP_PHASES,
            summary_digest: observation.summary_digest,
            evidence_digest: observation.evidence_digest,
            progress_fingerprint: observation.progress_fingerprint,
            tool_category: observation.tool_category,
            validation: observation.validation.as_ref().map(|result| result.verdict),
            resources: observation.resources,
        };
        self.completed_iterations = observation.sequence;
        let oscillating = self.before_previous_fingerprint
            == Some(observation.progress_fingerprint)
            && self.previous_fingerprint != Some(observation.progress_fingerprint);
        if self.previous_fingerprint == Some(observation.progress_fingerprint) {
            self.no_progress_iterations = self.no_progress_iterations.saturating_add(1);
        } else {
            self.no_progress_iterations = 0;
        }
        self.before_previous_fingerprint = self.previous_fingerprint;
        self.previous_fingerprint = Some(observation.progress_fingerprint);

        if observation
            .validation
            .as_ref()
            .is_some_and(|result| result.verdict == ValidationVerdict::Passed)
        {
            self.state = EngineeringLoopState::AwaitingReview;
            return Ok(LoopStepResult {
                receipt: Some(receipt),
                decision: LoopDecision::AwaitingReview,
            });
        }
        if oscillating {
            self.stop_with_current_resources(LoopStopReason::Oscillation);
        } else if self.no_progress_iterations >= self.loop_budget.max_no_progress_iterations {
            self.stop_with_current_resources(LoopStopReason::NoProgress);
        } else if self.completed_iterations >= self.loop_budget.max_iterations {
            self.stop_with_current_resources(LoopStopReason::MaxIterations);
        } else if let Some(reason) = self.resource_boundary_reason(observation.resources) {
            self.stop_with_current_resources(reason);
        } else {
            self.state = EngineeringLoopState::Running;
            return Ok(LoopStepResult {
                receipt: Some(receipt),
                decision: LoopDecision::Continue,
            });
        }
        Ok(LoopStepResult {
            receipt: Some(receipt),
            decision: self.current_stopped_decision(),
        })
    }

    /// Stop on cancel/revocation/deadline and begin explicit child/tool drain.
    pub fn request_stop(
        &mut self,
        reason: LoopStopReason,
        active_child_processes: u16,
    ) -> Result<LoopDecision, EngineeringLoopError> {
        if !matches!(
            self.state,
            EngineeringLoopState::Ready | EngineeringLoopState::Running
        ) {
            return Ok(match self.state {
                EngineeringLoopState::AwaitingReview => LoopDecision::AwaitingReview,
                _ => LoopDecision::Stopped(self.stop_receipt.expect("terminal state has receipt")),
            });
        }
        self.pending_iteration = None;
        self.stop_with_counts(reason, active_child_processes, self.tool_permits.active());
        Ok(self.current_stopped_decision())
    }

    /// Report child/tool cleanup. A timeout with remaining resources is explicitly incomplete.
    pub fn observe_drain(
        &mut self,
        active_child_processes: u16,
        active_tool_permits: u16,
        deadline_expired: bool,
    ) -> Result<LoopDecision, EngineeringLoopError> {
        if self.state != EngineeringLoopState::Draining {
            return Err(EngineeringLoopError::InvalidIterationTransition);
        }
        let current = self.stop_receipt.expect("draining state has receipt");
        if active_tool_permits != self.tool_permits.active() {
            return Err(EngineeringLoopError::DrainObservationMismatch);
        }
        if active_child_processes == 0 && active_tool_permits == 0 {
            self.stop_receipt = Some(LoopStopReceipt {
                reason: current.reason,
                drain: LoopDrainStatus::Complete,
                outstanding_child_processes: 0,
                outstanding_tool_permits: 0,
            });
            self.state = EngineeringLoopState::Stopped;
        } else if deadline_expired {
            self.stop_receipt = Some(LoopStopReceipt {
                reason: current.reason,
                drain: LoopDrainStatus::Incomplete,
                outstanding_child_processes: active_child_processes,
                outstanding_tool_permits: active_tool_permits,
            });
            self.state = EngineeringLoopState::Incomplete;
        } else {
            self.stop_receipt = Some(LoopStopReceipt {
                reason: current.reason,
                drain: LoopDrainStatus::Pending,
                outstanding_child_processes: active_child_processes,
                outstanding_tool_permits: active_tool_permits,
            });
        }
        Ok(self.current_stopped_decision())
    }

    fn validation_identity_matches(&self, result: &LoopValidationResult) -> bool {
        result.provider == self.snapshot.validation_provider
            && result.suite_digest == self.snapshot.validation_suite_digest
            && result.toolchain_digest == self.snapshot.validation_toolchain_digest
            && result.acceptance_digest == self.snapshot.acceptance_digest
    }

    fn sample_is_monotonic(&self, current: LoopResourceSample) -> bool {
        current.elapsed_ms >= self.last_resources.elapsed_ms
            && current.cpu_ms >= self.last_resources.cpu_ms
            && current.peak_rss_bytes >= self.last_resources.peak_rss_bytes
            && current.provider_calls >= self.last_resources.provider_calls
            && current.captured_output_bytes >= self.last_resources.captured_output_bytes
    }

    fn resource_stop_reason(&self, sample: LoopResourceSample) -> Option<LoopStopReason> {
        if sample.elapsed_ms
            > self
                .loop_budget
                .max_wall_clock_ms
                .min(self.resource_budget.max_runtime_ms)
        {
            Some(LoopStopReason::WallClockBudget)
        } else if sample.cpu_ms > self.resource_budget.max_cpu_ms {
            Some(LoopStopReason::CpuBudget)
        } else if sample.peak_rss_bytes > self.resource_budget.max_rss_bytes {
            Some(LoopStopReason::MemoryBudget)
        } else if sample.active_child_processes > self.resource_budget.max_child_processes {
            Some(LoopStopReason::ChildProcessBudget)
        } else if self.tool_permits.active() > self.resource_budget.max_parallel_tools {
            Some(LoopStopReason::ToolConcurrencyBudget)
        } else if sample.provider_calls
            > self
                .loop_budget
                .max_provider_calls
                .min(self.resource_budget.max_provider_calls)
        {
            Some(LoopStopReason::ProviderCallBudget)
        } else if sample.captured_output_bytes > self.resource_budget.max_output_bytes {
            Some(LoopStopReason::OutputBudget)
        } else if sample.event_buffer_bytes > self.resource_budget.max_event_buffer_bytes {
            Some(LoopStopReason::EventBufferBudget)
        } else {
            None
        }
    }

    fn resource_boundary_reason(&self, sample: LoopResourceSample) -> Option<LoopStopReason> {
        if sample.elapsed_ms
            >= self
                .loop_budget
                .max_wall_clock_ms
                .min(self.resource_budget.max_runtime_ms)
        {
            Some(LoopStopReason::WallClockBudget)
        } else if sample.cpu_ms >= self.resource_budget.max_cpu_ms {
            Some(LoopStopReason::CpuBudget)
        } else if sample.peak_rss_bytes >= self.resource_budget.max_rss_bytes {
            Some(LoopStopReason::MemoryBudget)
        } else if sample.provider_calls
            >= self
                .loop_budget
                .max_provider_calls
                .min(self.resource_budget.max_provider_calls)
        {
            Some(LoopStopReason::ProviderCallBudget)
        } else if sample.captured_output_bytes >= self.resource_budget.max_output_bytes {
            Some(LoopStopReason::OutputBudget)
        } else if sample.event_buffer_bytes >= self.resource_budget.max_event_buffer_bytes {
            Some(LoopStopReason::EventBufferBudget)
        } else {
            None
        }
    }

    fn stop_with_current_resources(&mut self, reason: LoopStopReason) {
        self.stop_with_counts(
            reason,
            self.last_resources.active_child_processes,
            self.tool_permits.active(),
        );
    }

    fn stop_with_sample(&mut self, reason: LoopStopReason, sample: LoopResourceSample) {
        self.last_resources = sample;
        self.stop_with_counts(
            reason,
            sample.active_child_processes,
            self.tool_permits.active(),
        );
    }

    fn stop_with_counts(&mut self, reason: LoopStopReason, children: u16, tools: u16) {
        let drained = children == 0 && tools == 0;
        self.stop_receipt = Some(LoopStopReceipt {
            reason,
            drain: if drained {
                LoopDrainStatus::Complete
            } else {
                LoopDrainStatus::Pending
            },
            outstanding_child_processes: children,
            outstanding_tool_permits: tools,
        });
        self.state = if drained {
            EngineeringLoopState::Stopped
        } else {
            EngineeringLoopState::Draining
        };
    }

    fn current_stopped_decision(&self) -> LoopDecision {
        LoopDecision::Stopped(self.stop_receipt.expect("stopped decision has receipt"))
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn same_provider_implementation(left: &ProviderReference, right: &ProviderReference) -> bool {
    left.provider_id == right.provider_id
        && left.version == right.version
        && left.implementation_digest == right.implementation_digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_profile::{
        AgentExecutionProfileDraft, ContextPolicySnapshot, GrantSnapshot, HookSetSnapshot,
        MemoryPolicySnapshot, SkillBindingSnapshot, ValidationPolicySnapshot,
        EXECUTION_PROFILE_SCHEMA_VERSION,
    };

    fn digest(character: char) -> String {
        std::iter::repeat(character).take(64).collect()
    }

    fn provider(provider_id: &str, character: char) -> ProviderReference {
        ProviderReference {
            provider_id: provider_id.to_owned(),
            version: 1,
            implementation_digest: digest(character),
            configuration_digest: digest(character),
            capabilities: Vec::new(),
        }
    }

    fn profile() -> VerifiedAgentExecutionProfile {
        AgentExecutionProfileDraft {
            schema_version: EXECUTION_PROFILE_SCHEMA_VERSION,
            scope: ExecutionProfileScope {
                tenant_id: Uuid::from_u128(1),
                project_id: Uuid::from_u128(2),
                worktree_id: Some(Uuid::from_u128(3)),
            },
            agent_provider: provider("agent.local", 'a'),
            memory: MemoryPolicySnapshot::Disabled,
            skills: vec![SkillBindingSnapshot {
                skill_id: "review.rust".to_owned(),
                version: 1,
                content_digest: digest('b'),
                capabilities: Vec::new(),
            }],
            context: ContextPolicySnapshot {
                assembler: provider("context.local", 'c'),
                max_input_bytes: 32_768,
                max_input_tokens: 8_192,
                max_sources: 32,
                compaction_policy_digest: digest('d'),
                preserve_task_contract: true,
                preserve_acceptance_criteria: true,
                preserve_authorization_scope: true,
            },
            validation: ValidationPolicySnapshot {
                provider: provider("validator.local", 'e'),
                suite_digest: digest('f'),
                toolchain_digest: digest('1'),
                acceptance_criteria: vec!["ac-build".to_owned()],
            },
            loop_budget: crate::execution_profile::LoopBudgetSnapshot {
                policy: provider("loop.engineering", '2'),
                max_iterations: 4,
                max_no_progress_iterations: 2,
                max_wall_clock_ms: 50_000,
                max_provider_calls: 8,
            },
            resource_budget: ResourceBudgetSnapshot {
                max_rss_bytes: 1_048_576,
                max_cpu_ms: 10_000,
                max_runtime_ms: 60_000,
                max_child_processes: 2,
                max_parallel_tools: 2,
                max_provider_calls: 10,
                max_output_bytes: 1_048_576,
                max_event_buffer_bytes: 4_096,
            },
            hook_set: HookSetSnapshot {
                hook_set_id: Uuid::from_u128(4),
                version: 1,
                effective_digest: digest('3'),
            },
            grants: GrantSnapshot {
                grant_set_id: Uuid::from_u128(5),
                version: 1,
                capabilities: Vec::new(),
                expires_at_epoch_ms: 10_000_000,
            },
            engineering_manifest: None,
        }
        .seal()
        .expect("valid profile")
    }

    fn controller() -> EngineeringLoopController {
        let profile = profile();
        EngineeringLoopController::new(
            &profile,
            LoopRunBinding {
                scope: ExecutionProfileScope {
                    tenant_id: Uuid::from_u128(1),
                    project_id: Uuid::from_u128(2),
                    worktree_id: Some(Uuid::from_u128(3)),
                },
                run_id: Uuid::from_u128(6),
                task_id: Uuid::from_u128(7),
                worktree_id: Uuid::from_u128(3),
                contract_digest: digest('4'),
                acceptance_digest: digest('5'),
            },
        )
        .expect("valid Run binding")
    }

    fn start(controller: &mut EngineeringLoopController, elapsed_ms: u64) -> u32 {
        let snapshot = controller.expected_snapshot().clone();
        match controller
            .begin_iteration(&snapshot, elapsed_ms)
            .expect("valid transition")
        {
            LoopDecision::IterationStarted(sequence) => sequence,
            other => panic!("expected admitted iteration, got {other:?}"),
        }
    }

    fn observation(
        controller: &EngineeringLoopController,
        sequence: u32,
        elapsed_ms: u64,
        fingerprint: u8,
        validation: Option<ValidationVerdict>,
    ) -> LoopObservation {
        let snapshot = controller.expected_snapshot().clone();
        let validation = validation.map(|verdict| LoopValidationResult {
            provider: snapshot.validation_provider.clone(),
            suite_digest: snapshot.validation_suite_digest.clone(),
            toolchain_digest: snapshot.validation_toolchain_digest.clone(),
            acceptance_digest: snapshot.acceptance_digest.clone(),
            verdict,
            evidence_digest: digest('6'),
        });
        LoopObservation {
            sequence,
            snapshot,
            summary_digest: digest('7'),
            evidence_digest: digest('8'),
            progress_fingerprint: [fingerprint; 32],
            tool_category: Some(LoopToolCategory::Validation),
            validation,
            resources: LoopResourceSample {
                elapsed_ms,
                cpu_ms: elapsed_ms / 2,
                peak_rss_bytes: 512 * 1024,
                active_child_processes: 0,
                provider_calls: sequence,
                captured_output_bytes: 512,
                event_buffer_bytes: 128,
            },
        }
    }

    #[test]
    fn validation_pass_waits_for_review_and_never_completes_the_task() {
        let mut loop_run = controller();
        let sequence = start(&mut loop_run, 1);
        let input = observation(&loop_run, sequence, 10, 1, Some(ValidationVerdict::Passed));
        let result = loop_run.finish_iteration(input).unwrap();
        assert_eq!(result.decision, LoopDecision::AwaitingReview);
        assert_eq!(loop_run.state(), EngineeringLoopState::AwaitingReview);
        assert_eq!(result.receipt.unwrap().phases, LOOP_PHASES);
        assert_eq!(
            loop_run.try_acquire_tool().unwrap_err(),
            ToolAdmissionError::LoopNotRunning
        );
    }

    #[test]
    fn failed_validation_continues_without_mutating_the_task() {
        let mut loop_run = controller();
        let sequence = start(&mut loop_run, 1);
        let input = observation(&loop_run, sequence, 10, 1, Some(ValidationVerdict::Failed));
        let result = loop_run.finish_iteration(input).unwrap();
        assert_eq!(result.decision, LoopDecision::Continue);
        assert_eq!(loop_run.state(), EngineeringLoopState::Running);
        assert_eq!(loop_run.completed_iterations(), 1);
    }

    #[test]
    fn no_progress_stops_and_drains_before_reporting_complete() {
        let mut loop_run = controller();
        for elapsed in [10, 20, 30] {
            let sequence = start(&mut loop_run, elapsed);
            let mut sample = observation(&loop_run, sequence, elapsed + 1, 9, None);
            sample.resources.active_child_processes = u16::from(sequence == 3);
            let result = loop_run.finish_iteration(sample).unwrap();
            if sequence < 3 {
                assert_eq!(result.decision, LoopDecision::Continue);
            } else {
                assert_eq!(loop_run.state(), EngineeringLoopState::Draining);
                assert_eq!(
                    result.decision,
                    LoopDecision::Stopped(LoopStopReceipt {
                        reason: LoopStopReason::NoProgress,
                        drain: LoopDrainStatus::Pending,
                        outstanding_child_processes: 1,
                        outstanding_tool_permits: 0,
                    })
                );
            }
        }
        assert_eq!(
            loop_run.observe_drain(0, 0, false).unwrap(),
            LoopDecision::Stopped(LoopStopReceipt {
                reason: LoopStopReason::NoProgress,
                drain: LoopDrainStatus::Complete,
                outstanding_child_processes: 0,
                outstanding_tool_permits: 0,
            })
        );
    }

    #[test]
    fn alternating_progress_fingerprint_stops_as_oscillation() {
        let mut loop_run = controller();
        for (elapsed, fingerprint) in [(10, 1), (20, 2), (30, 1)] {
            let sequence = start(&mut loop_run, elapsed);
            let input = observation(&loop_run, sequence, elapsed + 1, fingerprint, None);
            let result = loop_run.finish_iteration(input).unwrap();
            if sequence == 3 {
                assert!(matches!(
                    result.decision,
                    LoopDecision::Stopped(LoopStopReceipt {
                        reason: LoopStopReason::Oscillation,
                        drain: LoopDrainStatus::Complete,
                        ..
                    })
                ));
            }
        }
    }

    #[test]
    fn immutable_snapshot_drift_fails_closed() {
        let mut loop_run = controller();
        let mut changed = loop_run.expected_snapshot().clone();
        changed.acceptance_digest = digest('9');
        assert!(matches!(
            loop_run.begin_iteration(&changed, 1).unwrap(),
            LoopDecision::Stopped(LoopStopReceipt {
                reason: LoopStopReason::SnapshotChanged,
                ..
            })
        ));
        assert_eq!(loop_run.state(), EngineeringLoopState::Stopped);
    }

    #[test]
    fn validator_identity_drift_stops_loop() {
        let mut loop_run = controller();
        let sequence = start(&mut loop_run, 1);
        let mut sample = observation(&loop_run, sequence, 10, 1, Some(ValidationVerdict::Failed));
        sample.validation.as_mut().unwrap().suite_digest = digest('a');
        let result = loop_run.finish_iteration(sample).unwrap();
        assert!(matches!(
            result.decision,
            LoopDecision::Stopped(LoopStopReceipt {
                reason: LoopStopReason::SnapshotChanged,
                ..
            })
        ));
    }

    #[test]
    fn provider_budget_and_deadline_prevent_more_work() {
        let mut loop_run = controller();
        let sequence = start(&mut loop_run, 1);
        let mut sample = observation(&loop_run, sequence, 10, 1, None);
        sample.resources.provider_calls = 8;
        let result = loop_run.finish_iteration(sample).unwrap();
        assert!(matches!(
            result.decision,
            LoopDecision::Stopped(LoopStopReceipt {
                reason: LoopStopReason::ProviderCallBudget,
                ..
            })
        ));

        let mut timed = controller();
        let snapshot = timed.expected_snapshot().clone();
        assert!(matches!(
            timed.begin_iteration(&snapshot, 50_000).unwrap(),
            LoopDecision::Stopped(LoopStopReceipt {
                reason: LoopStopReason::WallClockBudget,
                ..
            })
        ));
    }

    #[test]
    fn tool_admission_is_bounded_and_has_no_wait_queue() {
        let mut loop_run = controller();
        assert_eq!(
            loop_run.try_acquire_tool().unwrap_err(),
            ToolAdmissionError::LoopNotRunning
        );
        start(&mut loop_run, 0);
        let first = loop_run.try_acquire_tool().unwrap();
        let second = loop_run.try_acquire_tool().unwrap();
        assert_eq!(loop_run.tool_permits.active(), 2);
        assert_eq!(
            loop_run.try_acquire_tool().unwrap_err(),
            ToolAdmissionError::AtCapacity
        );
        drop(first);
        assert_eq!(loop_run.tool_permits.active(), 1);
        drop(second);
        assert_eq!(loop_run.tool_permits.active(), 0);
    }

    #[test]
    fn cancellation_drains_children_and_reports_timeout_as_incomplete() {
        let mut loop_run = controller();
        start(&mut loop_run, 0);
        let tool = loop_run.try_acquire_tool().unwrap();
        assert!(matches!(
            loop_run.request_stop(LoopStopReason::Cancelled, 1).unwrap(),
            LoopDecision::Stopped(LoopStopReceipt {
                drain: LoopDrainStatus::Pending,
                ..
            })
        ));
        assert_eq!(
            loop_run.try_acquire_tool().unwrap_err(),
            ToolAdmissionError::LoopNotRunning
        );
        drop(tool);
        assert_eq!(
            loop_run.observe_drain(1, 1, false).unwrap_err(),
            EngineeringLoopError::DrainObservationMismatch
        );
        assert_eq!(loop_run.state(), EngineeringLoopState::Draining);
        assert!(matches!(
            loop_run.observe_drain(1, 0, true).unwrap(),
            LoopDecision::Stopped(LoopStopReceipt {
                drain: LoopDrainStatus::Incomplete,
                ..
            })
        ));
        assert_eq!(loop_run.state(), EngineeringLoopState::Incomplete);
    }
}

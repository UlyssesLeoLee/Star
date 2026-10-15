//! Run-scoped six-state Task Card command contract.
//!
//! This module is deliberately separate from the historical three-state `WorkItem` API. It
//! models the canonical Multica Task Card lifecycle owned by an Engineering Run.
//!
//! @cypher schema=1 source_sha256=590447a18da1306e187202694ec56e3c8a06bd86cf7d88cc3e0159b8cd6f44f4
//! MERGE (self:File {path:"crates/domain-work-item/src/run_task_card.rs"})
//! MERGE (module:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::module",kind:"module"})
//! MERGE (context:Type {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandContext",kind:"struct"})
//! MERGE (create:Type {id:"crates/domain-work-item/src/run_task_card.rs::CreateRunTaskCardCommand",kind:"struct"})
//! MERGE (transition:Type {id:"crates/domain-work-item/src/run_task_card.rs::TransitionRunTaskCardCommand",kind:"struct"})
//! MERGE (review:Type {id:"crates/domain-work-item/src/run_task_card.rs::ReviewRunTaskCardCommand",kind:"struct"})
//! MERGE (status:Type {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardStatus",kind:"enum"})
//! MERGE (action:Type {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardReviewAction",kind:"enum"})
//! MERGE (snapshot:Type {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardSnapshot",kind:"struct"})
//! MERGE (receipt:Type {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardWriteReceipt",kind:"struct"})
//! MERGE (error:Type {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardError",kind:"enum"})
//! MERGE (port:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort",kind:"trait"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(context)
//! MERGE (module)-[:DEFINES]->(create)
//! MERGE (module)-[:DEFINES]->(transition)
//! MERGE (module)-[:DEFINES]->(review)
//! MERGE (module)-[:DEFINES]->(status)
//! MERGE (module)-[:DEFINES]->(action)
//! MERGE (module)-[:DEFINES]->(snapshot)
//! MERGE (module)-[:DEFINES]->(receipt)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (module)-[:DEFINES]->(port)
//! MERGE (createPortMethod:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort.create",kind:"method"})
//! MERGE (transitionPortMethod:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort.transition",kind:"method"})
//! MERGE (reviewPortMethod:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort.review",kind:"method"})
//! MERGE (port)-[:HAS_METHOD]->(createPortMethod)
//! MERGE (port)-[:HAS_METHOD]->(transitionPortMethod)
//! MERGE (port)-[:HAS_METHOD]->(reviewPortMethod)
//! @endcypher

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{RepositoryId, TenantId, UserId, WorkItemId, WorktreeId};

/// Trusted Project/Run identity supplied by authenticated application composition.
///
/// A Worktree is optional execution context, not the Task Card owner. The adapter revalidates the
/// Project, Run, optional Worktree binding, and grants in its owner transaction. These values must
/// never be copied from an untrusted request body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTaskCardCommandContext {
    /// Authenticated tenant.
    pub tenant_id: TenantId,
    /// Authenticated actor.
    pub actor_id: UserId,
    /// Authorized Project when the caller addresses a Run directly.
    pub project_id: Option<Uuid>,
    /// Canonical owner Run when the caller addresses a Run directly.
    pub engineering_run_id: Option<Uuid>,
    /// Optional checkout focus. Legacy Worktree routes may resolve Project/Run from this binding.
    pub worktree_id: Option<WorktreeId>,
    /// Bounded idempotency key scoped to tenant and actor.
    pub idempotency_key: String,
    /// SHA-256 fingerprint over the canonical operation, route scope, and request body.
    pub request_fingerprint: [u8; 32],
    /// Correlation identity used by audit and the metadata-triggered Run Outbox.
    pub correlation_id: Uuid,
}

impl RunTaskCardCommandContext {
    /// Validate identifiers and the persisted key bounds before opening a write path.
    pub fn validate(&self) -> Result<(), RunTaskCardError> {
        if self.tenant_id.as_uuid().is_nil()
            || self.actor_id.as_uuid().is_nil()
            || self.project_id.is_some_and(|id| id.is_nil())
            || self.engineering_run_id.is_some_and(|id| id.is_nil())
            || self
                .worktree_id
                .is_some_and(|worktree_id| worktree_id.as_uuid().is_nil())
            || (self.engineering_run_id.is_some() != self.project_id.is_some())
            || (self.engineering_run_id.is_none() && self.worktree_id.is_none())
            || self.correlation_id.is_nil()
            || self.idempotency_key.trim().is_empty()
            || self.idempotency_key.len() > 128
        {
            return Err(RunTaskCardError::InvalidCommand("invalid_command_context"));
        }
        Ok(())
    }
}

/// Valid Run Task Card type persisted by the current schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTaskCardItemType {
    /// Epic.
    Epic,
    /// Story.
    Story,
    /// Task.
    Task,
    /// Bug.
    Bug,
    /// Subtask.
    Subtask,
    /// Agent task with a repository-scoped objective.
    AiTask,
}

impl RunTaskCardItemType {
    /// Stable database and API spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Epic => "epic",
            Self::Story => "story",
            Self::Task => "task",
            Self::Bug => "bug",
            Self::Subtask => "subtask",
            Self::AiTask => "ai_task",
        }
    }
}

impl TryFrom<&str> for RunTaskCardItemType {
    type Error = RunTaskCardError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "epic" => Ok(Self::Epic),
            "story" => Ok(Self::Story),
            "task" => Ok(Self::Task),
            "bug" => Ok(Self::Bug),
            "subtask" => Ok(Self::Subtask),
            "ai_task" => Ok(Self::AiTask),
            _ => Err(RunTaskCardError::InvalidCommand("invalid_item_type")),
        }
    }
}

/// Supported priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTaskCardPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    Medium,
    /// High priority.
    High,
    /// Urgent priority.
    Urgent,
}

impl RunTaskCardPriority {
    /// Stable database and API spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

impl TryFrom<&str> for RunTaskCardPriority {
    type Error = RunTaskCardError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "urgent" => Ok(Self::Urgent),
            _ => Err(RunTaskCardError::InvalidCommand("invalid_priority")),
        }
    }
}

/// Six-state lifecycle owned by the Run Task Card application port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTaskCardStatus {
    /// Ready to be claimed.
    Pending,
    /// Claimed by an actor with a lease.
    Claimed,
    /// Being worked in its active Worktree.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Work failed.
    Failed,
    /// Work was cancelled.
    Cancelled,
}

impl RunTaskCardStatus {
    /// Stable database and API spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Claimed => "claimed",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Whether a normal lifecycle transition allows this target from `from`.
    pub const fn allows(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Pending, Self::Claimed | Self::Cancelled)
                | (Self::Claimed, Self::InProgress | Self::Cancelled)
                | (
                    Self::InProgress,
                    Self::Completed | Self::Failed | Self::Cancelled
                )
                | (Self::Failed, Self::Pending)
        )
    }
}

impl TryFrom<&str> for RunTaskCardStatus {
    type Error = RunTaskCardError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(Self::Pending),
            "claimed" => Ok(Self::Claimed),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(RunTaskCardError::InvalidCommand("invalid_lifecycle_status")),
        }
    }
}

/// Review action for a claimed Run Task Card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTaskCardReviewAction {
    /// Submit completed work for review.
    Submit,
    /// Accept the submitted work.
    Accept,
    /// Reject the submitted work.
    Reject,
}

impl RunTaskCardReviewAction {
    /// Stable API spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }
}

impl TryFrom<&str> for RunTaskCardReviewAction {
    type Error = RunTaskCardError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "submit" => Ok(Self::Submit),
            "accept" => Ok(Self::Accept),
            "reject" => Ok(Self::Reject),
            _ => Err(RunTaskCardError::InvalidCommand("invalid_review_action")),
        }
    }
}

/// Review state tracked independently from the six-state work lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTaskCardReviewState {
    /// No review has been requested.
    None,
    /// Work is waiting for a different actor to review it.
    PendingReview,
    /// Review accepted the work.
    Accepted,
    /// Review rejected the work.
    Rejected,
}

impl RunTaskCardReviewState {
    /// Stable database and API spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PendingReview => "pending_review",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

/// Agent-specific task objective scoped to exactly the Run's repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTaskCardAiData {
    /// Agent objective.
    pub objective: String,
    /// Explicit repository allowlist; the adapter verifies it against the current Run binding.
    pub repository_scope: Vec<RepositoryId>,
}

/// Create command for a Task Card already scoped by the trusted Worktree context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRunTaskCardCommand {
    /// Task Card type.
    pub item_type: RunTaskCardItemType,
    /// Display title.
    pub title: String,
    /// Long-form task description.
    pub description: String,
    /// Priority.
    pub priority: RunTaskCardPriority,
    /// Search and grouping labels.
    pub labels: Vec<String>,
    /// Required only for `ai_task` cards.
    pub ai_task_data: Option<RunTaskCardAiData>,
}

impl CreateRunTaskCardCommand {
    /// Validate all bounds and type-specific invariants before any database write.
    pub fn validate(&self) -> Result<(), RunTaskCardError> {
        if self.title.trim().is_empty()
            || self.title.len() > 500
            || self.description.len() > 100_000
            || self.labels.len() > 100
            || self.labels.iter().any(|label| label.len() > 128)
            || (self.item_type == RunTaskCardItemType::AiTask) != self.ai_task_data.is_some()
            || self.ai_task_data.as_ref().is_some_and(|data| {
                data.objective.trim().is_empty()
                    || data.objective.len() > 10_000
                    || data.repository_scope.is_empty()
                    || data.repository_scope.len() > 100
            })
        {
            return Err(RunTaskCardError::InvalidCommand("invalid_work_item"));
        }
        Ok(())
    }
}

/// Six-state lifecycle transition command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRunTaskCardCommand {
    /// Canonical Task Card identity.
    pub work_item_id: WorkItemId,
    /// Target lifecycle status.
    pub target_status: RunTaskCardStatus,
    /// Optimistic concurrency version supplied by the caller.
    pub expected_version: i32,
    /// Optional audit reason.
    pub reason: Option<String>,
}

impl TransitionRunTaskCardCommand {
    /// Validate the expected version and reason bound.
    pub fn validate(&self) -> Result<(), RunTaskCardError> {
        if self.work_item_id.as_uuid().is_nil()
            || self.expected_version < 1
            || self
                .reason
                .as_ref()
                .is_some_and(|reason| reason.len() > 4_000)
        {
            return Err(RunTaskCardError::InvalidCommand(
                "invalid_lifecycle_command",
            ));
        }
        Ok(())
    }
}

/// Review command for a live, Worktree-bound Task Card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRunTaskCardCommand {
    /// Canonical Task Card identity.
    pub work_item_id: WorkItemId,
    /// Submit, accept, or reject.
    pub action: RunTaskCardReviewAction,
    /// Optimistic concurrency version supplied by the caller.
    pub expected_version: i32,
    /// Optional audit reason; required for reject.
    pub reason: Option<String>,
}

impl ReviewRunTaskCardCommand {
    /// Validate the expected version and review reason bounds.
    pub fn validate(&self) -> Result<(), RunTaskCardError> {
        if self.work_item_id.as_uuid().is_nil()
            || self.expected_version < 1
            || self
                .reason
                .as_ref()
                .is_some_and(|reason| reason.len() > 4_000)
            || (self.action == RunTaskCardReviewAction::Reject
                && self
                    .reason
                    .as_ref()
                    .is_none_or(|reason| reason.trim().is_empty()))
        {
            return Err(RunTaskCardError::InvalidCommand("invalid_review_command"));
        }
        Ok(())
    }
}

/// Current API projection stored in an idempotency receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTaskCardSnapshot {
    /// Canonical Work Item identity.
    pub work_item_id: WorkItemId,
    /// Stable alias used by Task Card UI.
    pub task_card_id: WorkItemId,
    /// Task type as exposed by the existing Group API.
    pub item_type: RunTaskCardItemType,
    /// Display title.
    pub title: String,
    /// Long-form task description.
    pub description: String,
    /// Priority.
    pub priority: RunTaskCardPriority,
    /// Labels.
    pub labels: Vec<String>,
    /// Agent-specific objective and repository scope.
    pub ai_task_data: Option<RunTaskCardAiData>,
    /// Authenticated reporter.
    pub reporter_user_id: UserId,
    /// Current six-state lifecycle projection.
    pub lifecycle: RunTaskCardLifecycleSnapshot,
    /// Last metadata change time.
    pub metadata_updated_at: DateTime<Utc>,
}

/// Current lifecycle projection in a command receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTaskCardLifecycleSnapshot {
    /// Current six-state status.
    pub status: RunTaskCardStatus,
    /// Current review state, represented as its stable wire value.
    pub review_state: RunTaskCardReviewState,
    /// Active Worktree while claimed or in progress.
    pub active_worktree_id: Option<WorktreeId>,
    /// Monotone lifecycle version.
    pub version: i32,
    /// Last lifecycle transition time.
    pub updated_at: DateTime<Utc>,
}

/// A committed or idempotently replayed command result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTaskCardWriteReceipt {
    /// Canonical response snapshot.
    pub snapshot: RunTaskCardSnapshot,
    /// True if the request replayed a prior durable receipt.
    pub replayed: bool,
}

/// Stable command and persistence errors safe to map at the HTTP boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RunTaskCardError {
    /// The command failed a bounded input or type invariant.
    #[error("invalid Run Task Card command")]
    InvalidCommand(&'static str),
    /// A current Worktree, Run, or Task Card binding could not be found.
    #[error("Run Task Card scope not found")]
    NotFound,
    /// The active Project/Run role does not allow this command.
    #[error("Run Task Card command is forbidden")]
    Forbidden,
    /// A version, lifecycle, review, claim, or idempotency invariant conflicted.
    #[error("Run Task Card command conflict")]
    Conflict(&'static str),
    /// Persistence failed; storage details stay inside the adapter logs.
    #[error("Run Task Card persistence unavailable")]
    Unavailable,
}

impl RunTaskCardError {
    /// HTTP status class for a stable API error response.
    pub const fn http_status(self) -> u16 {
        match self {
            Self::InvalidCommand(_) => 400,
            Self::NotFound => 404,
            Self::Forbidden => 403,
            Self::Conflict(_) => 409,
            Self::Unavailable => 500,
        }
    }

    /// Stable API error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidCommand(code) | Self::Conflict(code) => code,
            Self::NotFound => "not_found",
            Self::Forbidden => "forbidden",
            Self::Unavailable => "internal_error",
        }
    }
}

/// Command port for the canonical Run-scoped six-state Task Card.
#[async_trait]
pub trait RunTaskCardCommandPort: Send + Sync {
    /// Create and persist a Task Card, lifecycle fact, audit, and replay receipt atomically.
    async fn create(
        &self,
        context: RunTaskCardCommandContext,
        command: CreateRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError>;

    /// Apply a six-state lifecycle transition atomically.
    async fn transition(
        &self,
        context: RunTaskCardCommandContext,
        command: TransitionRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError>;

    /// Submit or decide review atomically.
    async fn review(
        &self,
        context: RunTaskCardCommandContext,
        command: ReviewRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError>;
}

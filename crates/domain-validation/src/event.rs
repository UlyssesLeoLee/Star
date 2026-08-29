//! Validation 域事件(Domain Events,CloudEvents 1.0)
//!
//! 来源:`docs/specs/domain-validation-spec.md` §5

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AcceptanceCoverageId, CoverageStatus, TenantId, ValidationEvidenceId, ValidationId,
    ValidationKind, ValidationOverrideId, ValidationStatus, WorkItemId,
};

/// 事件通用元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// **ValidationResultSubmitted**(spec §5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultSubmitted {
    pub meta: EventMeta,
    pub validation_id: ValidationId,
    pub work_item_id: Option<WorkItemId>,
    pub worktree_id: Option<crate::value_object::WorktreeId>,
    pub kind: ValidationKind,
    pub status: ValidationStatus,
}

/// **ValidationPassed**(Running → Passed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPassed {
    pub meta: EventMeta,
    pub validation_id: ValidationId,
    pub kind: ValidationKind,
    pub evidence_ref: Option<String>,
}

/// **ValidationFailed**(Running → Failed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailed {
    pub meta: EventMeta,
    pub validation_id: ValidationId,
    pub kind: ValidationKind,
    pub failure_summary: Option<String>,
    pub work_item_id: Option<WorkItemId>,
}

/// **ValidationOverridden**(Protected 审批)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOverridden {
    pub meta: EventMeta,
    pub override_id: ValidationOverrideId,
    pub validation_id: ValidationId,
    pub reason: String,
    pub approver_user_id: uuid::Uuid,
}

/// **AcceptanceCoverageAchieved**(100% 覆盖达成)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCoverageAchieved {
    pub meta: EventMeta,
    pub work_item_id: WorkItemId,
    pub total_count: u32,
    pub covered_count: u32,
}

/// **FeedbackRequired**(ValidationFailed 触发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRequired {
    pub meta: EventMeta,
    pub work_item_id: WorkItemId,
    pub validation_id: ValidationId,
    pub intervention_queue_priority: u8,
}

/// **EvidenceLinked**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLinked {
    pub meta: EventMeta,
    pub evidence_id: ValidationEvidenceId,
    pub validation_id: ValidationId,
}

/// **AcceptanceCoverageLinked**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCoverageLinked {
    pub meta: EventMeta,
    pub coverage_id: AcceptanceCoverageId,
    pub acceptance_criterion_id: uuid::Uuid,
    pub validation_id: ValidationId,
    pub new_status: CoverageStatus,
}

/// 全部 Validation 域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationEvent {
    Submitted(ValidationResultSubmitted),
    Passed(ValidationPassed),
    Failed(ValidationFailed),
    Overridden(ValidationOverridden),
    CoverageAchieved(AcceptanceCoverageAchieved),
    FeedbackRequired(FeedbackRequired),
    EvidenceLinked(EvidenceLinked),
    AcceptanceCoverageLinked(AcceptanceCoverageLinked),
}

impl ValidationEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Submitted(_) => "star.events.validation.validation_result.submitted.v1",
            Self::Passed(_) => "star.events.validation.validation_result.passed.v1",
            Self::Failed(_) => "star.events.validation.validation_result.failed.v1",
            Self::Overridden(_) => "star.events.validation.validation_result.overridden.v1",
            Self::CoverageAchieved(_) => "star.events.validation.acceptance_coverage.achieved.v1",
            Self::FeedbackRequired(_) => "star.events.validation.feedback_required.v1",
            Self::EvidenceLinked(_) => "star.events.validation.evidence.linked.v1",
            Self::AcceptanceCoverageLinked(_) => {
                "star.events.validation.acceptance_coverage.linked.v1"
            }
        }
    }
}

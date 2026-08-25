//! Feedback 域事件(Domain Events, CloudEvents 1.0)
//!
//! 来源: `docs/specs/domain-feedback-spec.md` §5 (6 状态迁移事件)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::ResolutionEvidence;
use crate::value_object::{FeedbackId, FeedbackStatus, FeedbackTarget, FeedbackType, Severity, TenantId, UserId};

/// 事件通用元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件 ID
    pub event_id: uuid::Uuid,
    /// 租户 ID(必带)
    pub tenant_id: TenantId,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者 user
    pub actor_user_id: Option<uuid::Uuid>,
    /// 触发者 agent
    pub actor_agent_id: Option<uuid::Uuid>,
}

impl EventMeta {
    /// 构造元数据
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
            actor_agent_id: None,
        }
    }
}

/// **`star.events.feedback.feedback.created.v1`**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackCreated {
    pub meta: EventMeta,
    pub feedback_id: FeedbackId,
    pub target: FeedbackTarget,
    pub r#type: FeedbackType,
    pub severity: Severity,
    pub author_user_id: UserId,
    pub author_agent_id: Option<uuid::Uuid>,
}

/// **`star.events.feedback.feedback.acknowledged.v1`**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAcknowledged {
    pub meta: EventMeta,
    pub feedback_id: FeedbackId,
    pub consumed_by_agent_session_id: uuid::Uuid,
}

/// **`star.events.feedback.feedback.applied.v1`**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackApplied {
    pub meta: EventMeta,
    pub feedback_id: FeedbackId,
    pub change_set_id: uuid::Uuid,
}

/// **`star.events.feedback.feedback.verified.v1`**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackVerified {
    pub meta: EventMeta,
    pub feedback_id: FeedbackId,
    pub validation_result_id: uuid::Uuid,
    pub evidence: Vec<ResolutionEvidence>,
}

/// **`star.events.feedback.feedback.rejected.v1`**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRejected {
    pub meta: EventMeta,
    pub feedback_id: FeedbackId,
    pub reason: String,
}

/// **`star.events.feedback.feedback.superseded.v1`**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSuperseded {
    pub meta: EventMeta,
    pub feedback_id: FeedbackId,
    pub successor_feedback_id: FeedbackId,
    /// 旧状态(任意 → Superseded)
    pub from_status: FeedbackStatus,
}

/// Feedback 域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FeedbackEvent {
    Created(FeedbackCreated),
    Acknowledged(FeedbackAcknowledged),
    Applied(FeedbackApplied),
    Verified(FeedbackVerified),
    Rejected(FeedbackRejected),
    Superseded(FeedbackSuperseded),
}

impl FeedbackEvent {
    /// NATS subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.feedback.feedback.created.v1",
            Self::Acknowledged(_) => "star.events.feedback.feedback.acknowledged.v1",
            Self::Applied(_) => "star.events.feedback.feedback.applied.v1",
            Self::Verified(_) => "star.events.feedback.feedback.verified.v1",
            Self::Rejected(_) => "star.events.feedback.feedback.rejected.v1",
            Self::Superseded(_) => "star.events.feedback.feedback.superseded.v1",
        }
    }
}

//! Comment 域事件(Domain Events,CloudEvents 1.0)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{CommentId, MentionId, ParentType, TenantId, UserId};

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

/// `CommentCreated`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentCreated {
    pub meta: EventMeta,
    pub comment_id: CommentId,
    pub parent_type: ParentType,
    pub parent_id: uuid::Uuid,
    pub author_user_id: UserId,
    pub author_agent_id: Option<uuid::Uuid>,
    pub mentions: Vec<UserId>,
}

/// `CommentUpdated`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentUpdated {
    pub meta: EventMeta,
    pub comment_id: CommentId,
    pub updated_at: DateTime<Utc>,
    pub diff_summary: String,
}

/// `CommentDeleted`(软删除)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentDeleted {
    pub meta: EventMeta,
    pub comment_id: CommentId,
    pub deleted_at: DateTime<Utc>,
}

/// `AttachmentUploaded`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentUploaded {
    pub meta: EventMeta,
    pub attachment_id: crate::value_object::AttachmentId,
    pub filename: String,
    pub size: u64,
    pub object_key: String,
}

/// `MentionNotified`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionNotified {
    pub meta: EventMeta,
    pub mention_id: MentionId,
    pub user_id: UserId,
    pub comment_id: CommentId,
}

/// 全部 Comment 域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommentEvent {
    Created(CommentCreated),
    Updated(CommentUpdated),
    Deleted(CommentDeleted),
    AttachmentUploaded(AttachmentUploaded),
    MentionNotified(MentionNotified),
}

impl CommentEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.comment.comment.created.v1",
            Self::Updated(_) => "star.events.comment.comment.updated.v1",
            Self::Deleted(_) => "star.events.comment.comment.deleted.v1",
            Self::AttachmentUploaded(_) => "star.events.comment.attachment.uploaded.v1",
            Self::MentionNotified(_) => "star.events.comment.mention.notified.v1",
        }
    }
}

//! Comment 域实体
//!
//! 来源:
//! - `docs/data-design.md` §4.9 (`comment` schema)
//! - `docs/specs/domain-comment-spec.md` §2 (实体清单)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AgentId, AttachmentId, CommentId, CommentStatus, DiscussionId, MentionId, ParentType,
    ProjectId, PullRequestId, ReactionId, TenantId, UserId, WorkItemId,
};

// =====================================================================
// Comment 聚合根
// =====================================================================

/// **Comment 聚合根**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    /// 父类型
    pub parent_type: ParentType,
    /// 父 ID(对应 WorkItemId / PullRequestId / DiscussionId)
    pub parent_id: uuid::Uuid,
    /// 纯文本内容(含 @mention 引用)
    pub body: String,
    /// 作者(用户)
    pub author_user_id: UserId,
    /// 作者(AI Agent,可空)
    pub author_agent_id: Option<AgentId>,
    /// 提及的 User IDs(INV-C-06 → 触发 Notification)
    pub mentions: Vec<UserId>,
    /// 附件 IDs
    pub attachment_ids: Vec<AttachmentId>,
    /// 状态(Open/Edited/Deleted,INV-C-04 软删除)
    pub status: CommentStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 软删除时间
    pub deleted_at: Option<DateTime<Utc>>,
    /// 乐观锁版本
    pub lock_version: u32,
}

impl Comment {
    /// 字段数
    pub const FIELD_COUNT: usize = 14;
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some() || self.status == CommentStatus::Deleted
    }
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Mention 实体
// =====================================================================

/// **Mention**(@提及)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub id: MentionId,
    pub tenant_id: TenantId,
    pub comment_id: CommentId,
    pub user_id: UserId,
    pub notified_at: DateTime<Utc>,
}

impl Mention {
    pub const FIELD_COUNT: usize = 5;
}

// =====================================================================
// Attachment 实体
// =====================================================================

/// **Attachment**(附件)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub tenant_id: TenantId,
    pub uploader_user_id: UserId,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    /// Object Storage Key(INV-C-03 必带 tenant_id 前缀)
    pub object_key: String,
    pub uploaded_at: DateTime<Utc>,
}

impl Attachment {
    pub const FIELD_COUNT: usize = 8;
}

// =====================================================================
// Reaction 实体
// =====================================================================

/// **Reaction**(emoji 反应)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: ReactionId,
    pub tenant_id: TenantId,
    pub comment_id: CommentId,
    pub user_id: UserId,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

impl Reaction {
    pub const FIELD_COUNT: usize = 6;
}

// =====================================================================
// AttachmentDownloadURL(查询返回)
// =====================================================================

/// **AttachmentDownloadURL**(短期预签名 URL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentDownloadURL {
    pub attachment_id: AttachmentId,
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

// 静默引用
#[allow(dead_code)]
fn _unused_pull(_: PullRequestId) -> PullRequestId {
    PullRequestId::new()
}
#[allow(dead_code)]
fn _unused_disc(_: DiscussionId) -> DiscussionId {
    DiscussionId::new()
}
#[allow(dead_code)]
fn _unused_wi(_: WorkItemId) -> WorkItemId {
    WorkItemId::new()
}

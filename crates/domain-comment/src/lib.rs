//! domain-comment crate
//!
//! 详细 spec: docs/specs/domain-comment-spec.md
//! 上游基本设计: docs/basic-design.md §2.1(表 13) / §3.2.1
//! 数据设计: docs/data-design.md §4.9 (`comment` schema)
//! API 设计: docs/api-design.md §3.10
//!
//! ## 职责
//!
//! WorkItem / PR / Discussion 上的评论(§10)。
//! **不**替代 Feedback(§25.1,REQ-FBK-001) — Comment 是普通对话,Feedback 是结构化指令。
//!
//! ## 关键不变量
//!
//! - INV-C-01:Comment 必带 tenant_id
//! - INV-C-02:Comment 状态机 Open / Edited / Deleted(软删除)
//! - INV-C-03:Reaction 唯一 (comment_id, user_id, emoji)
//! - INV-C-04:Attachment 必带 tenant_id 前缀(§4.3 Security)
//!
//! Lead 责任: comment Lead

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(CommentId);
define_uuid_id!(MentionId);
define_uuid_id!(AttachmentId);
define_uuid_id!(ReactionId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 实体
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub parent_type: ParentType,
    pub parent_id: Uuid,
    pub body: String,
    pub author_user_id: Option<UserId>,
    pub author_agent_id: Option<AgentId>,
    pub mentions: Vec<UserId>,
    pub attachment_ids: Vec<AttachmentId>,
    pub status: CommentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Comment {
    /// INV-C-01:作者必有 user 或 agent 之一
    pub fn validate_author(&self) -> Result<(), CommentError> {
        if self.author_user_id.is_none() && self.author_agent_id.is_none() {
            return Err(CommentError::InvalidState(
                "comment must have author_user_id or author_agent_id".to_string(),
            ));
        }
        if self.author_user_id.is_some() && self.author_agent_id.is_some() {
            return Err(CommentError::InvalidState(
                "comment cannot have both user and agent author".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentType {
    WorkItem,
    PullRequest,
    Discussion,
}

impl ParentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkItem => "work_item",
            Self::PullRequest => "pull_request",
            Self::Discussion => "discussion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentStatus {
    Open,
    Edited,
    Deleted,
}

impl CommentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Edited => "EDITED",
            Self::Deleted => "DELETED",
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Deleted)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub id: MentionId,
    pub comment_id: CommentId,
    pub user_id: UserId,
    pub notified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub tenant_id: TenantId,
    pub uploader_user_id: UserId,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    /// INV-C-04:tenant_id 前缀
    pub object_key: String,
    pub uploaded_at: DateTime<Utc>,
}

impl Attachment {
    pub fn validate_object_key(&self) -> Result<(), CommentError> {
        let prefix = format!("tenants/{}/", self.tenant_id.as_uuid());
        if !self.object_key.starts_with(&prefix) {
            return Err(CommentError::InvalidObjectKey);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: ReactionId,
    pub comment_id: CommentId,
    pub user_id: UserId,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum CommentError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// INV-C-04
    #[error("object_key must start with tenant_id prefix (INV-C-04)")]
    InvalidObjectKey,
    /// INV-C-03
    #[error("reaction already exists for (comment, user, emoji) (INV-C-03)")]
    ReactionExists,
    #[error("cannot edit deleted comment")]
    EditDeleted,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub parent_type: ParentType,
    pub parent_id: Uuid,
    pub body: String,
    pub author_user_id: Option<UserId>,
    pub author_agent_id: Option<AgentId>,
    pub mentions: Vec<UserId>,
    pub attachment_ids: Vec<AttachmentId>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCommentCommand {
    pub tenant_id: TenantId,
    pub comment_id: CommentId,
    pub new_body: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCommentCommand {
    pub tenant_id: TenantId,
    pub comment_id: CommentId,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReactionCommand {
    pub tenant_id: TenantId,
    pub comment_id: CommentId,
    pub user_id: UserId,
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAttachmentCommand {
    pub tenant_id: TenantId,
    pub uploader_user_id: UserId,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub object_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCommentQuery {
    pub tenant_id: TenantId,
    pub comment_id: CommentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByParentQuery {
    pub tenant_id: TenantId,
    pub parent_type: ParentType,
    pub parent_id: Uuid,
    pub include_deleted: bool,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait CommentCommandPort: Send + Sync {
    async fn create_comment(
        &self,
        cmd: CreateCommentCommand,
        actor: &ActorContext,
    ) -> Result<Comment, CommentError>;

    async fn edit_comment(
        &self,
        cmd: EditCommentCommand,
        actor: &ActorContext,
    ) -> Result<Comment, CommentError>;

    async fn delete_comment(
        &self,
        cmd: DeleteCommentCommand,
        actor: &ActorContext,
    ) -> Result<Comment, CommentError>;

    async fn add_reaction(
        &self,
        cmd: AddReactionCommand,
        actor: &ActorContext,
    ) -> Result<Reaction, CommentError>;

    async fn register_attachment(
        &self,
        cmd: RegisterAttachmentCommand,
        actor: &ActorContext,
    ) -> Result<Attachment, CommentError>;
}

#[async_trait]
pub trait CommentQueryPort: Send + Sync {
    async fn get(&self, q: GetCommentQuery, actor: &ActorContext) -> Result<Comment, CommentError>;

    async fn list_by_parent(
        &self,
        q: ListByParentQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Comment>, CommentError>;
}

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn insert_comment(&self, c: Comment) -> Result<(), CommentError>;
    async fn get_comment(&self, id: CommentId) -> Result<Comment, CommentError>;
    async fn update_comment(&self, c: Comment) -> Result<(), CommentError>;
    async fn list_by_parent(
        &self,
        tid: TenantId,
        pt: ParentType,
        pid: Uuid,
        include_deleted: bool,
    ) -> Result<Vec<Comment>, CommentError>;

    async fn insert_reaction(&self, r: Reaction) -> Result<(), CommentError>;
    async fn reaction_exists(
        &self,
        cid: CommentId,
        uid: UserId,
        emoji: &str,
    ) -> Result<bool, CommentError>;

    async fn insert_attachment(&self, a: Attachment) -> Result<(), CommentError>;
}

// =====================================================================
// InMemoryCommentService
// =====================================================================

pub struct InMemoryCommentService {
    repo: Arc<dyn CommentRepository>,
    comments: Arc<RwLock<HashMap<CommentId, Comment>>>,
    reactions: Arc<RwLock<HashMap<ReactionId, Reaction>>>,
    attachments: Arc<RwLock<HashMap<AttachmentId, Attachment>>>,
}

impl InMemoryCommentService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryCommentRepository::new()),
            comments: Arc::new(RwLock::new(HashMap::new())),
            reactions: Arc::new(RwLock::new(HashMap::new())),
            attachments: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCommentService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommentCommandPort for InMemoryCommentService {
    async fn create_comment(
        &self,
        cmd: CreateCommentCommand,
        actor: &ActorContext,
    ) -> Result<Comment, CommentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if cmd.body.is_empty() {
            return Err(CommentError::InvalidState("body required".to_string()));
        }
        let now = Utc::now();
        let c = Comment {
            id: CommentId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            parent_type: cmd.parent_type,
            parent_id: cmd.parent_id,
            body: cmd.body,
            author_user_id: cmd.author_user_id,
            author_agent_id: cmd.author_agent_id,
            mentions: cmd.mentions,
            attachment_ids: cmd.attachment_ids,
            status: CommentStatus::Open,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        c.validate_author()?;
        self.repo.insert_comment(c.clone()).await?;
        self.comments.write().unwrap().insert(c.id, c.clone());
        Ok(c)
    }

    async fn edit_comment(
        &self,
        cmd: EditCommentCommand,
        actor: &ActorContext,
    ) -> Result<Comment, CommentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut c = self
            .comments
            .write()
            .unwrap()
            .get_mut(&cmd.comment_id)
            .cloned()
            .ok_or(CommentError::NotFound(format!(
                "comment:{}",
                cmd.comment_id.as_uuid()
            )))?;
        if c.tenant_id != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(c.tenant_id, cmd.tenant_id));
        }
        if c.status == CommentStatus::Deleted {
            return Err(CommentError::EditDeleted);
        }
        // 仅作者可编辑
        if c.author_user_id != Some(UserId::from(actor.user_id)) {
            return Err(CommentError::PermissionDenied);
        }
        c.body = cmd.new_body;
        c.status = CommentStatus::Edited;
        c.updated_at = Utc::now();
        self.repo.update_comment(c.clone()).await?;
        self.comments.write().unwrap().insert(c.id, c.clone());
        Ok(c)
    }

    async fn delete_comment(
        &self,
        cmd: DeleteCommentCommand,
        actor: &ActorContext,
    ) -> Result<Comment, CommentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut c = self
            .comments
            .write()
            .unwrap()
            .get_mut(&cmd.comment_id)
            .cloned()
            .ok_or(CommentError::NotFound(format!(
                "comment:{}",
                cmd.comment_id.as_uuid()
            )))?;
        if c.tenant_id != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(c.tenant_id, cmd.tenant_id));
        }
        // 作者或 admin 可删
        if c.author_user_id != Some(UserId::from(actor.user_id)) && !actor.has_role("project_admin")
        {
            return Err(CommentError::PermissionDenied);
        }
        if c.status == CommentStatus::Deleted {
            return Err(CommentError::InvalidState("already deleted".to_string()));
        }
        c.status = CommentStatus::Deleted;
        c.deleted_at = Some(Utc::now());
        c.updated_at = Utc::now();
        self.repo.update_comment(c.clone()).await?;
        self.comments.write().unwrap().insert(c.id, c.clone());
        Ok(c)
    }

    async fn add_reaction(
        &self,
        cmd: AddReactionCommand,
        actor: &ActorContext,
    ) -> Result<Reaction, CommentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-C-03:唯一 (comment, user, emoji)
        if self
            .repo
            .reaction_exists(cmd.comment_id, cmd.user_id, &cmd.emoji)
            .await?
        {
            return Err(CommentError::ReactionExists);
        }
        let r = Reaction {
            id: ReactionId::new(),
            comment_id: cmd.comment_id,
            user_id: UserId::from(cmd.user_id),
            emoji: cmd.emoji,
            created_at: Utc::now(),
        };
        self.repo.insert_reaction(r.clone()).await?;
        self.reactions.write().unwrap().insert(r.id, r.clone());
        Ok(r)
    }

    async fn register_attachment(
        &self,
        cmd: RegisterAttachmentCommand,
        actor: &ActorContext,
    ) -> Result<Attachment, CommentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if UserId::from(actor.user_id) != cmd.uploader_user_id {
            return Err(CommentError::PermissionDenied);
        }
        let a = Attachment {
            id: AttachmentId::new(),
            tenant_id: cmd.tenant_id,
            uploader_user_id: cmd.uploader_user_id,
            filename: cmd.filename,
            content_type: cmd.content_type,
            size_bytes: cmd.size_bytes,
            object_key: cmd.object_key,
            uploaded_at: Utc::now(),
        };
        a.validate_object_key()?;
        self.repo.insert_attachment(a.clone()).await?;
        self.attachments.write().unwrap().insert(a.id, a.clone());
        Ok(a)
    }
}

#[async_trait]
impl CommentQueryPort for InMemoryCommentService {
    async fn get(&self, q: GetCommentQuery, actor: &ActorContext) -> Result<Comment, CommentError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let c = self
            .comments
            .read()
            .unwrap()
            .get(&q.comment_id)
            .cloned()
            .ok_or(CommentError::NotFound(format!(
                "comment:{}",
                q.comment_id.as_uuid()
            )))?;
        if c.tenant_id != q.tenant_id {
            return Err(CommentError::CrossTenantDenied(c.tenant_id, q.tenant_id));
        }
        Ok(c)
    }

    async fn list_by_parent(
        &self,
        q: ListByParentQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Comment>, CommentError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(CommentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let comments = self.comments.read().unwrap();
        Ok(comments
            .values()
            .filter(|c| {
                c.tenant_id == q.tenant_id
                    && c.parent_type == q.parent_type
                    && c.parent_id == q.parent_id
            })
            .filter(|c| q.include_deleted || c.status != CommentStatus::Deleted)
            .cloned()
            .collect())
    }
}

// =====================================================================
// InMemoryCommentRepository
// =====================================================================

pub struct InMemoryCommentRepository {
    comments: RwLock<HashMap<CommentId, Comment>>,
    reactions: RwLock<HashMap<ReactionId, Reaction>>,
    attachments: RwLock<HashMap<AttachmentId, Attachment>>,
}

impl InMemoryCommentRepository {
    pub fn new() -> Self {
        Self {
            comments: RwLock::new(HashMap::new()),
            reactions: RwLock::new(HashMap::new()),
            attachments: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCommentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommentRepository for InMemoryCommentRepository {
    async fn insert_comment(&self, c: Comment) -> Result<(), CommentError> {
        self.comments.write().unwrap().insert(c.id, c);
        Ok(())
    }
    async fn get_comment(&self, id: CommentId) -> Result<Comment, CommentError> {
        self.comments
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(CommentError::NotFound(format!("comment:{}", id.as_uuid())))
    }
    async fn update_comment(&self, c: Comment) -> Result<(), CommentError> {
        self.comments.write().unwrap().insert(c.id, c);
        Ok(())
    }
    async fn list_by_parent(
        &self,
        tid: TenantId,
        pt: ParentType,
        pid: Uuid,
        include_deleted: bool,
    ) -> Result<Vec<Comment>, CommentError> {
        Ok(self
            .comments
            .read()
            .unwrap()
            .values()
            .filter(|c| c.tenant_id == tid && c.parent_type == pt && c.parent_id == pid)
            .filter(|c| include_deleted || c.status != CommentStatus::Deleted)
            .cloned()
            .collect())
    }
    async fn insert_reaction(&self, r: Reaction) -> Result<(), CommentError> {
        self.reactions.write().unwrap().insert(r.id, r);
        Ok(())
    }
    async fn reaction_exists(
        &self,
        cid: CommentId,
        uid: UserId,
        emoji: &str,
    ) -> Result<bool, CommentError> {
        Ok(self
            .reactions
            .read()
            .unwrap()
            .values()
            .any(|r| r.comment_id == cid && r.user_id == uid && r.emoji == emoji))
    }
    async fn insert_attachment(&self, a: Attachment) -> Result<(), CommentError> {
        self.attachments.write().unwrap().insert(a.id, a);
        Ok(())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn dev(tid: uuid::Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid).with_role("developer")
    }

    fn make_cmd(tid: uuid::Uuid) -> CreateCommentCommand {
        let me = uuid::Uuid::new_v4();
        CreateCommentCommand {
            tenant_id: TenantId(tid),
            project_id: ProjectId::new(),
            parent_type: ParentType::WorkItem,
            parent_id: Uuid::new_v4(),
            body: "first comment".to_string(),
            author_user_id: Some(UserId::from(me)),
            author_agent_id: None,
            mentions: vec![],
            attachment_ids: vec![],
            actor_user_id: UserId::from(me),
        }
    }

    #[test]
    fn parent_type_as_str() {
        assert_eq!(ParentType::WorkItem.as_str(), "work_item");
        assert_eq!(ParentType::PullRequest.as_str(), "pull_request");
    }

    #[test]
    fn comment_status_is_terminal() {
        assert!(CommentStatus::Deleted.is_terminal());
        assert!(!CommentStatus::Open.is_terminal());
    }

    #[tokio::test]
    async fn create_comment_basic() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(tid);
        let c = svc.create_comment(make_cmd(tid), &actor).await.unwrap();
        assert_eq!(c.status, CommentStatus::Open);
    }

    #[tokio::test]
    async fn create_comment_requires_body() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(tid);
        let mut cmd = make_cmd(tid);
        cmd.body = "".to_string();
        let res = svc.create_comment(cmd, &actor).await;
        assert!(matches!(res, Err(CommentError::InvalidState(_))));
    }

    #[tokio::test]
    async fn create_comment_requires_exactly_one_author() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(tid);
        // 既没 user 也没 agent
        let mut cmd = make_cmd(tid);
        cmd.author_user_id = None;
        cmd.author_agent_id = None;
        let res = svc.create_comment(cmd, &actor).await;
        assert!(matches!(res, Err(CommentError::InvalidState(_))));
        // 同时有 user 和 agent
        let mut cmd2 = make_cmd(tid);
        cmd2.author_user_id = Some(UserId::from(uuid::Uuid::new_v4()));
        cmd2.author_agent_id = Some(AgentId::new());
        let res2 = svc.create_comment(cmd2, &actor).await;
        assert!(matches!(res2, Err(CommentError::InvalidState(_))));
    }

    #[tokio::test]
    async fn create_comment_by_agent() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        // Agent 用 as_agent 然后覆盖 tenant_id
        let mut agent_actor =
            ActorContext::new(AgentId::new().as_uuid(), tid).with_agent_session(true);
        agent_actor.tenant_id = tid;
        let mut cmd = make_cmd(tid);
        cmd.author_agent_id = Some(AgentId::new());
        cmd.author_user_id = None;
        let c = svc.create_comment(cmd, &agent_actor).await.unwrap();
        assert!(c.author_agent_id.is_some());
    }

    #[tokio::test]
    async fn edit_comment_self_only() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let mut cmd = make_cmd(tid);
        cmd.author_user_id = Some(UserId::from(me));
        let actor = ActorContext::new(me, tid);
        let c = svc.create_comment(cmd, &actor).await.unwrap();
        let c2 = svc
            .edit_comment(
                EditCommentCommand {
                    tenant_id: TenantId(tid),
                    comment_id: c.id,
                    new_body: "edited".to_string(),
                    actor_user_id: UserId::from(me),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(c2.status, CommentStatus::Edited);
    }

    #[tokio::test]
    async fn edit_other_users_comment_denied() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let other = uuid::Uuid::new_v4();
        let mut cmd = make_cmd(tid);
        cmd.author_user_id = Some(UserId::from(me));
        let actor = dev(tid);
        let c = svc.create_comment(cmd, &actor).await.unwrap();
        let other_actor = ActorContext::new(other, tid);
        let res = svc
            .edit_comment(
                EditCommentCommand {
                    tenant_id: TenantId(tid),
                    comment_id: c.id,
                    new_body: "x".to_string(),
                    actor_user_id: UserId::from(other),
                },
                &other_actor,
            )
            .await;
        assert!(matches!(res, Err(CommentError::PermissionDenied)));
    }

    #[tokio::test]
    async fn edit_deleted_comment_rejected() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let mut cmd = make_cmd(tid);
        cmd.author_user_id = Some(UserId::from(me));
        let actor = ActorContext::new(me, tid);
        let c = svc.create_comment(cmd, &actor).await.unwrap();
        svc.delete_comment(
            DeleteCommentCommand {
                tenant_id: TenantId(tid),
                comment_id: c.id,
                actor_user_id: UserId::from(me),
            },
            &actor,
        )
        .await
        .unwrap();
        let res = svc
            .edit_comment(
                EditCommentCommand {
                    tenant_id: TenantId(tid),
                    comment_id: c.id,
                    new_body: "x".to_string(),
                    actor_user_id: UserId::from(me),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(CommentError::EditDeleted)));
    }

    #[tokio::test]
    async fn delete_comment_soft() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let mut cmd = make_cmd(tid);
        cmd.author_user_id = Some(UserId::from(me));
        let actor = ActorContext::new(me, tid);
        let c = svc.create_comment(cmd, &actor).await.unwrap();
        let c2 = svc
            .delete_comment(
                DeleteCommentCommand {
                    tenant_id: TenantId(tid),
                    comment_id: c.id,
                    actor_user_id: UserId::from(me),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(c2.status, CommentStatus::Deleted);
        assert!(c2.deleted_at.is_some());
    }

    #[tokio::test]
    async fn add_reaction_unique_invc03() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let mut cmd = make_cmd(tid);
        cmd.author_user_id = Some(UserId::from(me));
        let actor = ActorContext::new(me, tid);
        let c = svc.create_comment(cmd, &actor).await.unwrap();
        let r = svc
            .add_reaction(
                AddReactionCommand {
                    tenant_id: TenantId(tid),
                    comment_id: c.id,
                    user_id: UserId::from(me),
                    emoji: "👍".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(r.emoji, "👍");
        let res = svc
            .add_reaction(
                AddReactionCommand {
                    tenant_id: TenantId(tid),
                    comment_id: c.id,
                    user_id: UserId::from(me),
                    emoji: "👍".to_string(),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(CommentError::ReactionExists)));
    }

    #[tokio::test]
    async fn attachment_requires_tenant_prefix_invc04() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let actor = ActorContext::new(me, tid);
        let res = svc
            .register_attachment(
                RegisterAttachmentCommand {
                    tenant_id: TenantId(tid),
                    uploader_user_id: UserId::from(me),
                    filename: "design.pdf".to_string(),
                    content_type: "application/pdf".to_string(),
                    size_bytes: 1024,
                    object_key: "wrong-prefix/file.pdf".to_string(), // 错
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(CommentError::InvalidObjectKey)));
    }

    #[tokio::test]
    async fn attachment_with_tenant_prefix_ok() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let actor = ActorContext::new(me, tid);
        let a = svc
            .register_attachment(
                RegisterAttachmentCommand {
                    tenant_id: TenantId(tid),
                    uploader_user_id: UserId::from(me),
                    filename: "design.pdf".to_string(),
                    content_type: "application/pdf".to_string(),
                    size_bytes: 1024,
                    object_key: format!("tenants/{}/design.pdf", tid),
                },
                &actor,
            )
            .await
            .unwrap();
        assert!(a.object_key.starts_with(&format!("tenants/{}/", tid)));
    }

    #[tokio::test]
    async fn list_by_parent_excludes_deleted() {
        let svc = InMemoryCommentService::new();
        let tid = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let parent_id = Uuid::new_v4();
        let actor = ActorContext::new(me, tid);
        // 创建 2 个
        for _ in 0..2 {
            let mut cmd = make_cmd(tid);
            cmd.parent_id = parent_id;
            cmd.author_user_id = Some(UserId::from(me));
            svc.create_comment(cmd, &actor).await.unwrap();
        }
        // 删 1 个
        let list = svc
            .list_by_parent(
                ListByParentQuery {
                    tenant_id: TenantId(tid),
                    parent_type: ParentType::WorkItem,
                    parent_id,
                    include_deleted: false,
                },
                &actor,
            )
            .await
            .unwrap();
        let first_id = list[0].id;
        svc.delete_comment(
            DeleteCommentCommand {
                tenant_id: TenantId(tid),
                comment_id: first_id,
                actor_user_id: UserId::from(me),
            },
            &actor,
        )
        .await
        .unwrap();
        let active = svc
            .list_by_parent(
                ListByParentQuery {
                    tenant_id: TenantId(tid),
                    parent_type: ParentType::WorkItem,
                    parent_id,
                    include_deleted: false,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(active.len(), 1);
        let all = svc
            .list_by_parent(
                ListByParentQuery {
                    tenant_id: TenantId(tid),
                    parent_type: ParentType::WorkItem,
                    parent_id,
                    include_deleted: true,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn cross_tenant_create_denied() {
        let svc = InMemoryCommentService::new();
        let actor_t = uuid::Uuid::new_v4();
        let cmd_t = uuid::Uuid::new_v4();
        let actor = dev(actor_t);
        let res = svc.create_comment(make_cmd(cmd_t), &actor).await;
        assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
    }
}

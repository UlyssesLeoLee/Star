//! Comment 端口(Port Traits)与命令/查询 DTO

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::entity::{Attachment, AttachmentDownloadURL, Comment, Mention, Reaction};
use crate::error::CommentError;
use crate::value_object::{
    CommentId, ParentType, ProjectId, ReactionId, TenantId, UserId, WorkItemId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateCommentCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub parent_type: ParentType,
    pub parent_id: uuid::Uuid,
    pub body: String,
    /// AI Agent 会话触发(可空)
    pub author_agent_id: Option<uuid::Uuid>,
    /// 提及的 User IDs
    pub mentions: Vec<UserId>,
}

/// `UpdateCommentCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommentCommand {
    pub comment_id: CommentId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub new_body: String,
}

/// `AddReactionCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReactionCommand {
    pub comment_id: CommentId,
    pub tenant_id: TenantId,
    pub emoji: String,
}

/// `UploadAttachmentCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadAttachmentCommand {
    pub tenant_id: TenantId,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub object_key: String,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListCommentQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommentQuery {
    pub tenant_id: TenantId,
    pub parent_type: ParentType,
    pub parent_id: uuid::Uuid,
    pub limit: u32,
    pub offset: u32,
    /// 是否包含已软删除
    pub include_deleted: bool,
}

// =====================================================================
// 端口:CommentCommandPort(6 方法)
// =====================================================================

/// **Comment 命令端口**
#[async_trait]
pub trait CommentCommandPort: Send + Sync {
    async fn create_comment(
        &self,
        cmd: CreateCommentCommand,
        actor: ActorContext,
    ) -> Result<Comment, CommentError>;

    async fn update_comment(
        &self,
        cmd: UpdateCommentCommand,
        actor: ActorContext,
    ) -> Result<Comment, CommentError>;

    /// 软删除(INV-C-04)
    async fn delete_comment(
        &self,
        comment_id: CommentId,
        actor: ActorContext,
    ) -> Result<(), CommentError>;

    async fn add_reaction(
        &self,
        cmd: AddReactionCommand,
        actor: ActorContext,
    ) -> Result<Reaction, CommentError>;

    async fn remove_reaction(
        &self,
        reaction_id: ReactionId,
        actor: ActorContext,
    ) -> Result<(), CommentError>;

    async fn upload_attachment(
        &self,
        cmd: UploadAttachmentCommand,
        actor: ActorContext,
    ) -> Result<Attachment, CommentError>;
}

// =====================================================================
// 端口:CommentQueryPort(3 方法)
// =====================================================================

/// **Comment 查询端口**
#[async_trait]
pub trait CommentQueryPort: Send + Sync {
    async fn list_by_parent(
        &self,
        q: ListCommentQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Comment>, CommentError>;
    async fn get_by_id(
        &self,
        id: CommentId,
        viewer: ActorContext,
    ) -> Result<Comment, CommentError>;
    async fn get_attachment_url(
        &self,
        attachment_id: crate::value_object::AttachmentId,
        viewer: ActorContext,
    ) -> Result<AttachmentDownloadURL, CommentError>;
}

// =====================================================================
// 仓库端口
// =====================================================================

/// **Comment 仓库端口**
#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn insert_comment(&self, c: &Comment) -> Result<(), CommentError>;
    async fn find_comment(&self, id: CommentId) -> Result<Option<Comment>, CommentError>;
    async fn save_comment(&self, c: &Comment) -> Result<(), CommentError>;
    async fn list_comments_raw(
        &self,
        parent_type: ParentType,
        parent_id: uuid::Uuid,
    ) -> Result<Vec<Comment>, CommentError>;

    async fn insert_mention(&self, m: &Mention) -> Result<(), CommentError>;
    async fn list_mentions(&self, user_id: UserId) -> Result<Vec<Mention>, CommentError>;

    async fn insert_attachment(&self, a: &Attachment) -> Result<(), CommentError>;
    async fn find_attachment(
        &self,
        id: crate::value_object::AttachmentId,
    ) -> Result<Option<Attachment>, CommentError>;

    async fn insert_reaction(&self, r: &Reaction) -> Result<(), CommentError>;
    async fn find_reaction(&self, id: ReactionId) -> Result<Option<Reaction>, CommentError>;
    async fn list_reactions(
        &self,
        comment_id: CommentId,
    ) -> Result<Vec<Reaction>, CommentError>;
    /// 按 (comment_id, user_id, emoji) 查 Reaction(用于 INV-C-06 重复反应检查)
    async fn find_reaction_by_triple(
        &self,
        comment_id: CommentId,
        user_id: UserId,
        emoji: &str,
    ) -> Result<Option<Reaction>, CommentError>;
}

// 静默引用
#[allow(dead_code)]
fn _unused_wi(_: WorkItemId) -> WorkItemId {
    WorkItemId::new()
}

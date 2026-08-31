//! InMemoryCommentService:Phase 2 内存实现

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::entity::{Attachment, AttachmentDownloadURL, Comment, Mention, Reaction};
use crate::error::CommentError;
use crate::event::{
    AttachmentUploaded, CommentCreated, CommentDeleted, CommentEvent, CommentUpdated, EventMeta,
    MentionNotified,
};
use crate::invariants::{
    check_attachment_size, check_body_length, check_create_invariants,
    check_invariant_03_object_key_tenant_prefix,
};
use crate::port::{
    AddReactionCommand, CommentCommandPort, CommentQueryPort, CommentRepository,
    CreateCommentCommand, ListCommentQuery, UpdateCommentCommand, UploadAttachmentCommand,
};
use crate::value_object::{
    AgentId, AttachmentId, CommentId, CommentStatus, MentionId, ParentType, ProjectId, ReactionId,
    TenantId, UserId,
};

/// 附件最大 50MB
const MAX_ATTACHMENT_BYTES: u64 = 50 * 1024 * 1024;
/// body 最大 10K 字符
const MAX_BODY_CHARS: usize = 10_000;

// =====================================================================
// InMemoryCommentService
// =====================================================================

pub struct InMemoryCommentService {
    comments: Arc<RwLock<HashMap<CommentId, Comment>>>,
    mentions: Arc<RwLock<HashMap<MentionId, Mention>>>,
    attachments: Arc<RwLock<HashMap<AttachmentId, Attachment>>>,
    reactions: Arc<RwLock<HashMap<ReactionId, Reaction>>>,
    /// 重复 reaction 检查用
    reaction_triples: Arc<RwLock<HashSet<(CommentId, UserId, String)>>>,
    event_tx: mpsc::UnboundedSender<CommentEvent>,
}

impl InMemoryCommentService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<CommentEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            comments: Arc::new(RwLock::new(HashMap::new())),
            mentions: Arc::new(RwLock::new(HashMap::new())),
            attachments: Arc::new(RwLock::new(HashMap::new())),
            reactions: Arc::new(RwLock::new(HashMap::new())),
            reaction_triples: Arc::new(RwLock::new(HashSet::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    pub async fn count_comments(&self) -> usize {
        self.comments.read().expect("lock").len()
    }

    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), CommentError> {
        if actor.tenant_id != expected.0 {
            return Err(CommentError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryCommentService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryCommentService {
    fn clone(&self) -> Self {
        Self {
            comments: self.comments.clone(),
            mentions: self.mentions.clone(),
            attachments: self.attachments.clone(),
            reactions: self.reactions.clone(),
            reaction_triples: self.reaction_triples.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// CommentCommandPort 实现
// =====================================================================

#[async_trait]
impl CommentCommandPort for InMemoryCommentService {
    async fn create_comment(
        &self,
        cmd: CreateCommentCommand,
        actor: ActorContext,
    ) -> Result<Comment, CommentError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let is_agent = cmd.author_agent_id.is_some();
        let author_agent = cmd
            .author_agent_id
            .map(crate::value_object::AgentId::from_uuid);
        let c = Comment {
            id: CommentId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            parent_type: cmd.parent_type,
            parent_id: cmd.parent_id,
            body: cmd.body.clone(),
            author_user_id: UserId::from(actor.user_id),
            author_agent_id: author_agent,
            mentions: cmd.mentions.clone(),
            attachment_ids: Vec::new(),
            status: CommentStatus::Open,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            lock_version: 1,
        };
        check_create_invariants(&c, is_agent)?;
        // 持久化
        self.comments
            .write()
            .expect("lock")
            .insert(c.id, c.clone());

        // 处理 mentions
        let mut mentions = self.mentions.write().expect("lock");
        for uid in &cmd.mentions {
            let m = Mention {
                id: MentionId::new(),
                tenant_id: cmd.tenant_id,
                comment_id: c.id,
                user_id: *uid,
                notified_at: now,
            };
            mentions.insert(m.id, m.clone());
            // 发布 MentionNotified 事件
            let evt = CommentEvent::MentionNotified(MentionNotified {
                meta: EventMeta {
                    actor_user_id: Some(actor.user_id),
                    ..EventMeta::new(cmd.tenant_id)
                },
                mention_id: m.id,
                user_id: UserId::from(m.user_id),
                comment_id: c.id,
            });
            let _ = self.event_tx.send(evt);
        }
        drop(mentions);

        // 发布 Created 事件
        let evt = CommentEvent::Created(CommentCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            comment_id: c.id,
            parent_type: c.parent_type,
            parent_id: c.parent_id,
            author_user_id: c.author_user_id,
            author_agent_id: c.author_agent_id.map(|a| a.into_uuid()),
            mentions: c.mentions.clone(),
        });
        let _ = self.event_tx.send(evt);
        Ok(c)
    }

    async fn update_comment(
        &self,
        cmd: UpdateCommentCommand,
        actor: ActorContext,
    ) -> Result<Comment, CommentError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut comments = self.comments.write().expect("lock");
        let c = comments
            .get_mut(&cmd.comment_id)
            .ok_or(CommentError::NotFound(cmd.comment_id))?;
        if c.tenant_id != cmd.tenant_id {
            return Err(CommentError::PermissionDenied);
        }
        // C-002:仅作者 / admin 可更新
        if c.author_user_id != UserId::from(actor.user_id) && !actor.is_tenant_admin() {
            return Err(CommentError::PermissionDenied);
        }
        if c.is_deleted() {
            return Err(CommentError::InvalidState(
                "已删除 Comment 不可更新".to_string(),
            ));
        }
        if c.lock_version != cmd.expected_version {
            return Err(CommentError::Conflict(format!(
                "lock_version mismatch: expected={}, actual={}",
                cmd.expected_version, c.lock_version
            )));
        }
        check_body_length(&cmd.new_body, MAX_BODY_CHARS)?;
        c.body = cmd.new_body;
        c.status = CommentStatus::Edited;
        c.bump_version();

        let evt = CommentEvent::Updated(CommentUpdated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            comment_id: c.id,
            updated_at: c.updated_at,
            diff_summary: "body updated".to_string(),
        });
        let _ = self.event_tx.send(evt);
        Ok(c.clone())
    }

    async fn delete_comment(
        &self,
        comment_id: CommentId,
        actor: ActorContext,
    ) -> Result<(), CommentError> {
        let mut comments = self.comments.write().expect("lock");
        let c = comments
            .get_mut(&comment_id)
            .ok_or(CommentError::NotFound(comment_id))?;
        if c.tenant_id != actor.tenant_id {
            return Err(CommentError::PermissionDenied);
        }
        // C-002:仅作者 / admin 可删除
        if c.author_user_id != UserId::from(actor.user_id) && !actor.is_tenant_admin() {
            return Err(CommentError::PermissionDenied);
        }
        let now = chrono::Utc::now();
        c.deleted_at = Some(now);
        c.status = CommentStatus::Deleted;
        c.bump_version();

        let evt = CommentEvent::Deleted(CommentDeleted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(c.tenant_id)
            },
            comment_id: c.id,
            deleted_at: now,
        });
        let _ = self.event_tx.send(evt);
        Ok(())
    }

    async fn add_reaction(
        &self,
        cmd: AddReactionCommand,
        actor: ActorContext,
    ) -> Result<Reaction, CommentError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let comments = self.comments.read().expect("lock");
        let c = comments
            .get(&cmd.comment_id)
            .ok_or(CommentError::NotFound(cmd.comment_id))?;
        if c.tenant_id != cmd.tenant_id {
            return Err(CommentError::PermissionDenied);
        }
        drop(comments);
        // 重复检查
        {
            let triples = self.reaction_triples.read().expect("lock");
            if triples.contains(&(cmd.comment_id, actor.user_id, cmd.emoji.clone())) {
                return Err(CommentError::Conflict(format!(
                    "C-006: 重复 reaction (comment_id, user_id, emoji)"
                )));
            }
        }
        let now = chrono::Utc::now();
        let r = Reaction {
            id: ReactionId::new(),
            tenant_id: cmd.tenant_id,
            comment_id: cmd.comment_id,
            user_id: UserId::from(actor.user_id),
            emoji: cmd.emoji.clone(),
            created_at: now,
        };
        self.reactions
            .write()
            .expect("lock")
            .insert(r.id, r.clone());
        self.reaction_triples
            .write()
            .expect("lock")
            .insert((cmd.comment_id, actor.user_id, cmd.emoji));
        Ok(r)
    }

    async fn remove_reaction(
        &self,
        reaction_id: ReactionId,
        actor: ActorContext,
    ) -> Result<(), CommentError> {
        let mut reactions = self.reactions.write().expect("lock");
        let r = reactions
            .get(&reaction_id)
            .ok_or(CommentError::NotFound(CommentId::from_uuid(uuid::Uuid::nil())))?
            .clone();
        if r.user_id != UserId::from(actor.user_id) && !actor.is_tenant_admin() {
            return Err(CommentError::PermissionDenied);
        }
        reactions.remove(&reaction_id);
        drop(reactions);
        // 清理 triple
        self.reaction_triples
            .write()
            .expect("lock")
            .remove(&(r.comment_id, r.user_id, r.emoji.clone()));
        Ok(())
    }

    async fn upload_attachment(
        &self,
        cmd: UploadAttachmentCommand,
        actor: ActorContext,
    ) -> Result<Attachment, CommentError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        check_attachment_size(cmd.size_bytes, MAX_ATTACHMENT_BYTES)?;
        let now = chrono::Utc::now();
        let a = Attachment {
            id: AttachmentId::new(),
            tenant_id: cmd.tenant_id,
            uploader_user_id: UserId::from(actor.user_id),
            filename: cmd.filename.clone(),
            content_type: cmd.content_type,
            size_bytes: cmd.size_bytes,
            object_key: cmd.object_key.clone(),
            uploaded_at: now,
        };
        // INV-C-03
        check_invariant_03_object_key_tenant_prefix(&a)?;
        self.attachments
            .write()
            .expect("lock")
            .insert(a.id, a.clone());

        let evt = CommentEvent::AttachmentUploaded(AttachmentUploaded {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            attachment_id: a.id,
            filename: a.filename.clone(),
            size: a.size_bytes,
            object_key: a.object_key.clone(),
        });
        let _ = self.event_tx.send(evt);
        Ok(a)
    }
}

// =====================================================================
// CommentQueryPort 实现
// =====================================================================

#[async_trait]
impl CommentQueryPort for InMemoryCommentService {
    async fn list_by_parent(
        &self,
        q: ListCommentQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Comment>, CommentError> {
        if viewer.tenant_id != q.tenant_id {
            return Err(CommentError::PermissionDenied);
        }
        let comments = self.comments.read().expect("lock");
        let mut out: Vec<Comment> = comments
            .values()
            .filter(|c| c.tenant_id == q.tenant_id)
            .filter(|c| c.parent_type == q.parent_type && c.parent_id == q.parent_id)
            .filter(|c| q.include_deleted || !c.is_deleted())
            .cloned()
            .collect();
        out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        let start = q.offset as usize;
        let end = std::cmp::min(start + q.limit as usize, out.len());
        if start >= out.len() {
            out.clear();
        } else {
            out = out.split_off(start);
            out.truncate(end - start);
        }
        Ok(out)
    }

    async fn get_by_id(
        &self,
        id: CommentId,
        viewer: ActorContext,
    ) -> Result<Comment, CommentError> {
        let comments = self.comments.read().expect("lock");
        let c = comments
            .get(&id)
            .ok_or(CommentError::NotFound(id))?
            .clone();
        if c.tenant_id != viewer.tenant_id {
            return Err(CommentError::PermissionDenied);
        }
        Ok(c)
    }

    async fn get_attachment_url(
        &self,
        attachment_id: AttachmentId,
        viewer: ActorContext,
    ) -> Result<AttachmentDownloadURL, CommentError> {
        let atts = self.attachments.read().expect("lock");
        let a = atts
            .get(&attachment_id)
            .ok_or(CommentError::NotFound(CommentId::from_uuid(uuid::Uuid::nil())))?
            .clone();
        if a.tenant_id != viewer.tenant_id {
            return Err(CommentError::PermissionDenied);
        }
        Ok(AttachmentDownloadURL {
            attachment_id,
            url: format!("https://signed.example.com/{}", a.object_key),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        })
    }
}

// =====================================================================
// CommentRepository 实现
// =====================================================================

#[async_trait]
impl CommentRepository for InMemoryCommentService {
    async fn insert_comment(&self, c: &Comment) -> Result<(), CommentError> {
        self.comments.write().expect("lock").insert(c.id, c.clone());
        Ok(())
    }
    async fn find_comment(&self, id: CommentId) -> Result<Option<Comment>, CommentError> {
        Ok(self.comments.read().expect("lock").get(&id).cloned())
    }
    async fn save_comment(&self, c: &Comment) -> Result<(), CommentError> {
        self.comments.write().expect("lock").insert(c.id, c.clone());
        Ok(())
    }
    async fn list_comments_raw(
        &self,
        parent_type: ParentType,
        parent_id: uuid::Uuid,
    ) -> Result<Vec<Comment>, CommentError> {
        let comments = self.comments.read().expect("lock");
        Ok(comments
            .values()
            .filter(|c| c.parent_type == parent_type && c.parent_id == parent_id)
            .cloned()
            .collect())
    }

    async fn insert_mention(&self, m: &Mention) -> Result<(), CommentError> {
        self.mentions.write().expect("lock").insert(m.id, m.clone());
        Ok(())
    }
    async fn list_mentions(&self, user_id: UserId) -> Result<Vec<Mention>, CommentError> {
        Ok(self
            .mentions
            .read()
            .expect("lock")
            .values()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn insert_attachment(&self, a: &Attachment) -> Result<(), CommentError> {
        self.attachments
            .write()
            .expect("lock")
            .insert(a.id, a.clone());
        Ok(())
    }
    async fn find_attachment(
        &self,
        id: AttachmentId,
    ) -> Result<Option<Attachment>, CommentError> {
        Ok(self.attachments.read().expect("lock").get(&id).cloned())
    }

    async fn insert_reaction(&self, r: &Reaction) -> Result<(), CommentError> {
        self.reactions.write().expect("lock").insert(r.id, r.clone());
        self.reaction_triples
            .write()
            .expect("lock")
            .insert((r.comment_id, r.user_id, r.emoji.clone()));
        Ok(())
    }
    async fn find_reaction(&self, id: ReactionId) -> Result<Option<Reaction>, CommentError> {
        Ok(self.reactions.read().expect("lock").get(&id).cloned())
    }
    async fn list_reactions(
        &self,
        comment_id: CommentId,
    ) -> Result<Vec<Reaction>, CommentError> {
        Ok(self
            .reactions
            .read()
            .expect("lock")
            .values()
            .filter(|r| r.comment_id == comment_id)
            .cloned()
            .collect())
    }
    async fn find_reaction_by_triple(
        &self,
        comment_id: CommentId,
        user_id: UserId,
        emoji: &str,
    ) -> Result<Option<Reaction>, CommentError> {
        Ok(self
            .reactions
            .read()
            .expect("lock")
            .values()
            .find(|r| r.comment_id == comment_id && r.user_id == user_id && r.emoji == emoji)
            .cloned())
    }
}

// 静默引用
#[allow(dead_code)]
fn _unused_ag(_: AgentId) -> AgentId {
    AgentId::new()
}
#[allow(dead_code)]
fn _unused_pj(_: ProjectId) -> ProjectId {
    ProjectId::new()
}

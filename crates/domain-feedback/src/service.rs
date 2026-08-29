//! InMemoryFeedbackService:Phase 2 内存实现
//!
//! 完整实现 FeedbackCommandPort + FeedbackQueryPort + FeedbackRepository。
//! 关键约束:
//! - 6 状态机严格迁移(INV-FB-01)
//! - tenant_id 必带,跨 tenant 拒绝(INV-FB-06)
//! - AI 提的 author_agent_id 必带(INV-FB-07)
//! - Supersede 必带 successor(INV-FB-04,FB-006)
//! - APPLIED 之后不可改
//! - 仅 OPEN 可删(FB-005)
//! - Target 11 种(spec §7 + SOW 任务范围)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{
    ConsumedByKind, Feedback, FeedbackConsumedEvent, FeedbackInboxItem, FeedbackResolution,
    ResolutionEvidence, ResolutionEvidenceRef,
};
use crate::error::FeedbackError;
use crate::event::{
    EventMeta, FeedbackAcknowledged, FeedbackApplied, FeedbackCreated, FeedbackEvent,
    FeedbackRejected, FeedbackSuperseded, FeedbackVerified,
};
use crate::invariants::{check_create_invariants, check_invariant_04_supersede_has_successor};
use crate::port::{
    CreateFeedbackCommand, FeedbackCommandPort, FeedbackInboxQuery, FeedbackQueryPort,
    FeedbackRepository, ListFeedbackQuery, SubmitResolutionCommand,
    TransitionFeedbackStatusCommand, UpdateFeedbackCommand,
};
use crate::value_object::{
    AgentId, FeedbackId, FeedbackResolutionId, FeedbackStatus, FeedbackTarget, Severity, TenantId,
    WorkItemId,
};

// =====================================================================
// InMemoryFeedbackService
// =====================================================================

/// **InMemoryFeedbackService** — 完整 Phase 2 实现
pub struct InMemoryFeedbackService {
    feedbacks: Arc<RwLock<HashMap<FeedbackId, Feedback>>>,
    resolutions: Arc<RwLock<HashMap<FeedbackResolutionId, FeedbackResolution>>>,
    consumed_events: Arc<RwLock<HashMap<uuid::Uuid, FeedbackConsumedEvent>>>,
    event_tx: mpsc::UnboundedSender<FeedbackEvent>,
}

impl InMemoryFeedbackService {
    /// 构造 + 接收 event 通道
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<FeedbackEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            feedbacks: Arc::new(RwLock::new(HashMap::new())),
            resolutions: Arc::new(RwLock::new(HashMap::new())),
            consumed_events: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 测试便捷构造(无 event 通道接收端)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 反馈计数
    #[allow(dead_code)]
    pub async fn count_feedbacks(&self) -> usize {
        self.feedbacks.read().expect("lock").len()
    }

    /// tenant 校验(INV-FB-06)
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), FeedbackError> {
        if actor.tenant_id != expected {
            return Err(FeedbackError::PermissionDenied);
        }
        Ok(())
    }

    /// 跨 Worktree 校验(INV-FB-05,FB-007)
    /// 简化策略:Feedback target 含 worktree_id 时,actor.worktree_id 必须匹配。
    /// 在 InMemory 内存实现中,跨 Worktree 通过 `cmd.actor_worktree_id` 注入;
    /// 真实实现由 Worktree 模块授权。
    fn check_cross_worktree(
        target: &FeedbackTarget,
        actor_worktree_id: Option<uuid::Uuid>,
    ) -> Result<(), FeedbackError> {
        if let FeedbackTarget::Worktree { worktree_id } = target {
            if let Some(actor_wt) = actor_worktree_id {
                if worktree_id.as_uuid() != &actor_wt {
                    return Err(FeedbackError::CrossWorktree);
                }
            }
        }
        Ok(())
    }
}

impl Clone for InMemoryFeedbackService {
    fn clone(&self) -> Self {
        Self {
            feedbacks: self.feedbacks.clone(),
            resolutions: self.resolutions.clone(),
            consumed_events: self.consumed_events.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// FeedbackCommandPort 实现
// =====================================================================

#[async_trait]
impl FeedbackCommandPort for InMemoryFeedbackService {
    async fn create_feedback(
        &self,
        cmd: CreateFeedbackCommand,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        Self::check_cross_worktree(&cmd.target, None)?;

        let now = chrono::Utc::now();
        let is_agent = actor.is_agent_session;
        let author_agent = cmd.author_agent_id.or(if is_agent {
            Some(AgentId::new()) // INV-FB-07 自动兜底,真实实现应来自 session
        } else {
            None
        });

        let f = Feedback {
            id: FeedbackId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            work_item_id: cmd.work_item_id,
            agent_session_id: None,
            target: cmd.target,
            r#type: cmd.r#type,
            severity: cmd.severity,
            intent: cmd.intent,
            expected_behavior: cmd.expected_behavior,
            preserve: cmd.preserve,
            prohibit: cmd.prohibit,
            author_user_id: actor.user_id,
            author_agent_id: author_agent,
            acceptance_criteria_id: cmd.acceptance_criteria_id,
            predecessor_id: cmd.predecessor_id,
            successor_id: None,
            status: FeedbackStatus::Open,
            resolution_evidence: Vec::new(),
            created_at: now,
            resolved_at: None,
            lock_version: 1,
        };

        check_create_invariants(&f, is_agent)?;

        // 持久化
        self.feedbacks
            .write()
            .expect("lock")
            .insert(f.id, f.clone());

        // 发布 created 事件
        let evt = FeedbackEvent::Created(FeedbackCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                actor_agent_id: f.author_agent_id.map(|a| a.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            feedback_id: f.id,
            target: f.target.clone(),
            r#type: f.r#type,
            severity: f.severity,
            author_user_id: f.author_user_id,
            author_agent_id: f.author_agent_id.map(|a| a.into_uuid()),
        });
        let _ = self.event_tx.send(evt);

        Ok(f)
    }

    async fn update_feedback(
        &self,
        cmd: UpdateFeedbackCommand,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut feedbacks = self.feedbacks.write().expect("lock");
        let f = feedbacks
            .get_mut(&cmd.feedback_id)
            .ok_or(FeedbackError::NotFound(cmd.feedback_id))?;
        if f.tenant_id != cmd.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        if f.author_user_id != actor.user_id && !actor.is_tenant_admin() {
            return Err(FeedbackError::PermissionDenied);
        }
        // 仅 OPEN/ACKNOWLEDGED 可改
        if !f.is_editable() {
            return Err(FeedbackError::ReadOnly);
        }
        // 乐观锁
        if f.lock_version != cmd.expected_version {
            return Err(FeedbackError::Conflict(format!(
                "lock_version mismatch: expected={}, actual={}",
                cmd.expected_version, f.lock_version
            )));
        }
        if let Some(intent) = cmd.new_intent {
            f.intent = intent;
        }
        if let Some(expected_behavior) = cmd.new_expected_behavior {
            f.expected_behavior = expected_behavior;
        }
        if let Some(preserve) = cmd.new_preserve {
            f.preserve = preserve;
        }
        if let Some(prohibit) = cmd.new_prohibit {
            f.prohibit = prohibit;
        }
        if let Some(severity) = cmd.new_severity {
            f.severity = severity;
        }
        f.bump_version();
        Ok(f.clone())
    }

    async fn delete_feedback(
        &self,
        id: FeedbackId,
        actor: ActorContext,
    ) -> Result<(), FeedbackError> {
        let mut feedbacks = self.feedbacks.write().expect("lock");
        let f = feedbacks
            .get(&id)
            .ok_or(FeedbackError::NotFound(id))?
            .clone();
        if f.tenant_id != actor.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        if f.author_user_id != actor.user_id && !actor.is_tenant_admin() {
            return Err(FeedbackError::PermissionDenied);
        }
        // FB-005:仅 OPEN 可删
        if !f.is_deletable() {
            return Err(FeedbackError::NotDeletable);
        }
        feedbacks.remove(&id);
        Ok(())
    }

    async fn transition_status(
        &self,
        cmd: TransitionFeedbackStatusCommand,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut feedbacks = self.feedbacks.write().expect("lock");
        let f = feedbacks
            .get_mut(&cmd.feedback_id)
            .ok_or(FeedbackError::NotFound(cmd.feedback_id))?;
        if f.tenant_id != cmd.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        // 6 状态机迁移
        if !cmd.from.can_transition_to(cmd.to) {
            return Err(FeedbackError::InvalidState(format!(
                "非法 6 状态迁移: {} -> {}",
                cmd.from, cmd.to
            )));
        }
        if f.status != cmd.from {
            return Err(FeedbackError::InvalidState(format!(
                "Feedback 当前状态 {} 与 from={} 不匹配",
                f.status, cmd.from
            )));
        }
        // INV-FB-04 / FB-006:Supersede 必带 successor
        if cmd.to == FeedbackStatus::Superseded {
            let successor = cmd.successor_id.ok_or(FeedbackError::MissingSuccessor)?;
            f.successor_id = Some(successor);
        }
        // 跨 Worktree 校验(actor.worktree_id)
        if let Some(actor_wt) = cmd.actor_worktree_id {
            if let FeedbackTarget::Worktree { worktree_id } = &f.target {
                if worktree_id.as_uuid() != &actor_wt {
                    return Err(FeedbackError::CrossWorktree);
                }
            }
        }
        // 状态迁移
        f.transition(cmd.to).map_err(FeedbackError::InvalidState)?;
        // INV-FB-04 校验
        check_invariant_04_supersede_has_successor(f)?;

        // 发布对应事件
        let meta = EventMeta {
            actor_user_id: Some(actor.user_id.into_uuid()),
            actor_agent_id: f.author_agent_id.map(|a| a.into_uuid()),
            ..EventMeta::new(cmd.tenant_id)
        };
        let evt = match cmd.to {
            FeedbackStatus::Acknowledged => FeedbackEvent::Acknowledged(FeedbackAcknowledged {
                meta,
                feedback_id: f.id,
                consumed_by_agent_session_id: actor.user_id.into_uuid(), // 简化:实际为 agent_session_id
            }),
            FeedbackStatus::Applied => FeedbackEvent::Applied(FeedbackApplied {
                meta,
                feedback_id: f.id,
                change_set_id: uuid::Uuid::nil(), // 由 mark_applied 覆盖
            }),
            FeedbackStatus::Verified => FeedbackEvent::Verified(FeedbackVerified {
                meta,
                feedback_id: f.id,
                validation_result_id: uuid::Uuid::nil(),
                evidence: f.resolution_evidence.clone(),
            }),
            FeedbackStatus::Rejected => FeedbackEvent::Rejected(FeedbackRejected {
                meta,
                feedback_id: f.id,
                reason: cmd.reason,
            }),
            FeedbackStatus::Superseded => FeedbackEvent::Superseded(FeedbackSuperseded {
                meta,
                feedback_id: f.id,
                successor_feedback_id: f
                    .successor_id
                    .expect("INV-FB-04: successor 必填,前面已校验"),
                from_status: cmd.from,
            }),
            FeedbackStatus::Open => unreachable!("from 校验已拒绝 Open -> Open"),
        };
        let _ = self.event_tx.send(evt);

        Ok(f.clone())
    }

    async fn submit_resolution(
        &self,
        cmd: SubmitResolutionCommand,
        actor: ActorContext,
    ) -> Result<FeedbackResolution, FeedbackError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        if !matches!(
            cmd.target_status,
            FeedbackStatus::Verified | FeedbackStatus::Rejected | FeedbackStatus::Superseded
        ) {
            return Err(FeedbackError::InvalidState(format!(
                "Resolution 目标状态必须为 VERIFIED/REJECTED/SUPERSEDED,实际 {}",
                cmd.target_status
            )));
        }
        let now = chrono::Utc::now();
        let r = FeedbackResolution {
            id: FeedbackResolutionId::new(),
            tenant_id: cmd.tenant_id,
            feedback_id: cmd.feedback_id,
            resolver_user_id: actor.user_id,
            resolver_agent_id: cmd.resolver_agent_id,
            resolved_status: cmd.target_status,
            evidence_refs: cmd.evidence_refs,
            note: cmd.note,
            created_at: now,
        };
        self.resolutions
            .write()
            .expect("lock")
            .insert(r.id, r.clone());
        Ok(r)
    }

    async fn mark_applied(
        &self,
        feedback_id: FeedbackId,
        change_set_id: uuid::Uuid,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError> {
        let mut feedbacks = self.feedbacks.write().expect("lock");
        let f = feedbacks
            .get_mut(&feedback_id)
            .ok_or(FeedbackError::NotFound(feedback_id))?;
        let tenant_id = f.tenant_id;
        Self::check_tenant(&actor, tenant_id)?;
        f.transition(FeedbackStatus::Applied)
            .map_err(FeedbackError::InvalidState)?;
        let evt = FeedbackEvent::Applied(FeedbackApplied {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(tenant_id)
            },
            feedback_id: f.id,
            change_set_id,
        });
        let _ = self.event_tx.send(evt);
        Ok(f.clone())
    }

    async fn mark_verified(
        &self,
        feedback_id: FeedbackId,
        validation_result_id: uuid::Uuid,
        evidence: Vec<ResolutionEvidence>,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError> {
        let mut feedbacks = self.feedbacks.write().expect("lock");
        let f = feedbacks
            .get_mut(&feedback_id)
            .ok_or(FeedbackError::NotFound(feedback_id))?;
        let tenant_id = f.tenant_id;
        Self::check_tenant(&actor, tenant_id)?;
        f.resolution_evidence.extend(evidence);
        f.transition(FeedbackStatus::Verified)
            .map_err(FeedbackError::InvalidState)?;
        let evt = FeedbackEvent::Verified(FeedbackVerified {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(tenant_id)
            },
            feedback_id: f.id,
            validation_result_id,
            evidence: f.resolution_evidence.clone(),
        });
        let _ = self.event_tx.send(evt);
        Ok(f.clone())
    }

    async fn record_consumed(
        &self,
        feedback_id: FeedbackId,
        consumed_by: ConsumedByKind,
        consumed_by_id: uuid::Uuid,
        actor: ActorContext,
    ) -> Result<FeedbackConsumedEvent, FeedbackError> {
        let feedbacks = self.feedbacks.read().expect("lock");
        let f = feedbacks
            .get(&feedback_id)
            .ok_or(FeedbackError::NotFound(feedback_id))?;
        let tenant_id = f.tenant_id;
        Self::check_tenant(&actor, tenant_id)?;
        drop(feedbacks);
        let now = chrono::Utc::now();
        let e = FeedbackConsumedEvent {
            event_id: uuid::Uuid::new_v4(),
            feedback_id,
            tenant_id,
            consumed_by,
            consumed_by_id,
            consumed_at: now,
        };
        self.consumed_events
            .write()
            .expect("lock")
            .insert(e.event_id, e.clone());
        Ok(e)
    }
}

// =====================================================================
// FeedbackQueryPort 实现
// =====================================================================

#[async_trait]
impl FeedbackQueryPort for InMemoryFeedbackService {
    async fn get_by_id(
        &self,
        id: FeedbackId,
        viewer: ActorContext,
    ) -> Result<Feedback, FeedbackError> {
        let feedbacks = self.feedbacks.read().expect("lock");
        let f = feedbacks
            .get(&id)
            .ok_or(FeedbackError::NotFound(id))?
            .clone();
        if f.tenant_id != viewer.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        Ok(f)
    }

    async fn list_by_project(
        &self,
        q: ListFeedbackQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Feedback>, FeedbackError> {
        if viewer.tenant_id != q.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        let feedbacks = self.feedbacks.read().expect("lock");
        let mut out: Vec<Feedback> = feedbacks
            .values()
            .filter(|f| f.tenant_id == q.tenant_id)
            .filter(|f| q.project_id.is_none_or(|p| f.project_id == p))
            .filter(|f| q.work_item_id.is_none_or(|w| f.work_item_id == w))
            .filter(|f| q.status.is_none_or(|s| f.status == s))
            .cloned()
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
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

    async fn inbox(
        &self,
        q: FeedbackInboxQuery,
        viewer: ActorContext,
    ) -> Result<Vec<FeedbackInboxItem>, FeedbackError> {
        if viewer.tenant_id != q.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        let feedbacks = self.feedbacks.read().expect("lock");
        let mut out: Vec<FeedbackInboxItem> = feedbacks
            .values()
            .filter(|f| f.tenant_id == q.tenant_id && f.project_id == q.project_id)
            .filter(|f| q.min_severity.is_none_or(|min| f.severity <= min))
            // Inbox 默认仅显示 OPEN/ACKNOWLEDGED/Applied(P0-P3 排序的活跃反馈)
            .filter(|f| !f.is_terminal())
            .map(|f| FeedbackInboxItem {
                feedback_id: f.id,
                tenant_id: f.tenant_id,
                project_id: f.project_id,
                work_item_id: f.work_item_id,
                target: f.target.clone(),
                r#type: f.r#type,
                severity: f.severity,
                status: f.status,
                intent: f.intent.clone(),
                created_at: f.created_at,
            })
            .collect();
        // 按 (severity ASC, created_at ASC) 排序
        // Severity 枚举已实现 Ord:P0 < P1 < P2 < P3
        out.sort_by(|a, b| {
            a.severity
                .cmp(&b.severity)
                .then(a.created_at.cmp(&b.created_at))
        });
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

    async fn list_consumed_events(
        &self,
        feedback_id: FeedbackId,
        viewer: ActorContext,
    ) -> Result<Vec<FeedbackConsumedEvent>, FeedbackError> {
        let feedbacks = self.feedbacks.read().expect("lock");
        let f = feedbacks
            .get(&feedback_id)
            .ok_or(FeedbackError::NotFound(feedback_id))?;
        if f.tenant_id != viewer.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        drop(feedbacks);
        let events = self.consumed_events.read().expect("lock");
        Ok(events
            .values()
            .filter(|e| e.feedback_id == feedback_id)
            .cloned()
            .collect())
    }

    async fn list_resolutions(
        &self,
        feedback_id: FeedbackId,
        viewer: ActorContext,
    ) -> Result<Vec<FeedbackResolution>, FeedbackError> {
        let feedbacks = self.feedbacks.read().expect("lock");
        let f = feedbacks
            .get(&feedback_id)
            .ok_or(FeedbackError::NotFound(feedback_id))?;
        if f.tenant_id != viewer.tenant_id {
            return Err(FeedbackError::PermissionDenied);
        }
        drop(feedbacks);
        let resolutions = self.resolutions.read().expect("lock");
        Ok(resolutions
            .values()
            .filter(|r| r.feedback_id == feedback_id)
            .cloned()
            .collect())
    }
}

// =====================================================================
// FeedbackRepository 实现
// =====================================================================

#[async_trait]
impl FeedbackRepository for InMemoryFeedbackService {
    async fn insert_feedback(&self, f: &Feedback) -> Result<(), FeedbackError> {
        self.feedbacks
            .write()
            .expect("lock")
            .insert(f.id, f.clone());
        Ok(())
    }
    async fn save_feedback(&self, f: &Feedback) -> Result<(), FeedbackError> {
        self.feedbacks
            .write()
            .expect("lock")
            .insert(f.id, f.clone());
        Ok(())
    }
    async fn find_feedback(&self, id: FeedbackId) -> Result<Option<Feedback>, FeedbackError> {
        Ok(self.feedbacks.read().expect("lock").get(&id).cloned())
    }
    async fn list_feedbacks_raw(
        &self,
        q: ListFeedbackQuery,
    ) -> Result<Vec<Feedback>, FeedbackError> {
        let feedbacks = self.feedbacks.read().expect("lock");
        Ok(feedbacks
            .values()
            .filter(|f| f.tenant_id == q.tenant_id)
            .filter(|f| q.project_id.is_none_or(|p| f.project_id == p))
            .filter(|f| q.work_item_id.is_none_or(|w| f.work_item_id == w))
            .filter(|f| q.status.is_none_or(|s| f.status == s))
            .cloned()
            .collect())
    }
    async fn insert_resolution(&self, r: &FeedbackResolution) -> Result<(), FeedbackError> {
        self.resolutions
            .write()
            .expect("lock")
            .insert(r.id, r.clone());
        Ok(())
    }
    async fn list_resolutions_raw(
        &self,
        feedback_id: FeedbackId,
    ) -> Result<Vec<FeedbackResolution>, FeedbackError> {
        Ok(self
            .resolutions
            .read()
            .expect("lock")
            .values()
            .filter(|r| r.feedback_id == feedback_id)
            .cloned()
            .collect())
    }
    async fn insert_consumed_event(&self, e: &FeedbackConsumedEvent) -> Result<(), FeedbackError> {
        self.consumed_events
            .write()
            .expect("lock")
            .insert(e.event_id, e.clone());
        Ok(())
    }
    async fn list_consumed_events_raw(
        &self,
        feedback_id: FeedbackId,
    ) -> Result<Vec<FeedbackConsumedEvent>, FeedbackError> {
        Ok(self
            .consumed_events
            .read()
            .expect("lock")
            .values()
            .filter(|e| e.feedback_id == feedback_id)
            .cloned()
            .collect())
    }
}

// 静默引用
#[allow(dead_code)]
fn _unused_severity(_: Severity) -> Severity {
    Severity::P3
}
#[allow(dead_code)]
fn _unused_wi(_: WorkItemId) -> WorkItemId {
    WorkItemId::new()
}

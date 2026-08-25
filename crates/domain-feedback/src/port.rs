//! Feedback 端口(Port Traits)与命令/查询 DTO
//!
//! 来源: `docs/specs/domain-feedback-spec.md` §4

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Feedback, FeedbackConsumedEvent, FeedbackInboxItem, FeedbackResolution, ResolutionEvidence, ResolutionEvidenceRef};
use crate::error::FeedbackError;
use crate::value_object::{
    AgentId, AgentSessionId, FeedbackId, FeedbackResolutionId, FeedbackStatus, FeedbackTarget,
    FeedbackType, ProjectId, Severity, TenantId, UserId, WorkItemId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateFeedbackCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFeedbackCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub work_item_id: WorkItemId,
    /// 目标(11 种,SOW 范围)
    pub target: FeedbackTarget,
    /// 类型(7 类)
    pub r#type: FeedbackType,
    /// 严重程度(P0-P3)
    pub severity: Severity,
    /// 短句意图
    pub intent: String,
    /// 期望行为
    pub expected_behavior: String,
    /// 必须保留的
    pub preserve: Vec<String>,
    /// 禁止修改的
    pub prohibit: Vec<String>,
    /// 作者 agent(可空,INV-FB-07)
    pub author_agent_id: Option<AgentId>,
    /// 关联 AC(可空)
    pub acceptance_criteria_id: Option<uuid::Uuid>,
    /// 前驱 Feedback(Supersede 链,INV-FB-04)
    pub predecessor_id: Option<FeedbackId>,
}

/// `UpdateFeedbackCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFeedbackCommand {
    pub feedback_id: FeedbackId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub new_intent: Option<String>,
    pub new_expected_behavior: Option<String>,
    pub new_preserve: Option<Vec<String>>,
    pub new_prohibit: Option<Vec<String>>,
    pub new_severity: Option<Severity>,
}

/// `TransitionFeedbackStatusCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionFeedbackStatusCommand {
    pub feedback_id: FeedbackId,
    pub tenant_id: TenantId,
    pub from: FeedbackStatus,
    pub to: FeedbackStatus,
    pub reason: String,
    /// Supersede 必带 successor(INV-FB-04,FB-006)
    pub successor_id: Option<FeedbackId>,
    /// 跨 Worktree 校验(actor.worktree_id)
    pub actor_worktree_id: Option<uuid::Uuid>,
}

/// `SubmitResolutionCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResolutionCommand {
    pub feedback_id: FeedbackId,
    pub tenant_id: TenantId,
    pub target_status: FeedbackStatus,
    pub note: String,
    pub evidence_refs: Vec<ResolutionEvidenceRef>,
    /// 解析人 agent(可空)
    pub resolver_agent_id: Option<AgentId>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListFeedbackQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFeedbackQuery {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub work_item_id: Option<WorkItemId>,
    pub status: Option<FeedbackStatus>,
    pub limit: u32,
    pub offset: u32,
}

/// `FeedbackInboxQuery`(P0-P3 优先级排序)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackInboxQuery {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    /// 过滤严重程度(默认全部)
    pub min_severity: Option<Severity>,
    pub limit: u32,
    pub offset: u32,
}

// =====================================================================
// 端口:FeedbackCommandPort
// =====================================================================

/// **Feedback 命令端口**
#[async_trait]
pub trait FeedbackCommandPort: Send + Sync {
    /// 创建 Feedback(INV-FB-02 必带 target,INV-FB-06 必带 tenant_id)
    async fn create_feedback(
        &self,
        cmd: CreateFeedbackCommand,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError>;

    /// 更新 Feedback(仅 OPEN/ACKNOWLEDGED 可改,APPLIED 后只读)
    async fn update_feedback(
        &self,
        cmd: UpdateFeedbackCommand,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError>;

    /// 删除 Feedback(仅 OPEN 状态,FB-005)
    async fn delete_feedback(
        &self,
        id: FeedbackId,
        actor: ActorContext,
    ) -> Result<(), FeedbackError>;

    /// 6 状态迁移(由 service 校验 from→to 合法性)
    async fn transition_status(
        &self,
        cmd: TransitionFeedbackStatusCommand,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError>;

    /// 提交 Resolution(VERIFIED / REJECTED / SUPERSEDED)
    async fn submit_resolution(
        &self,
        cmd: SubmitResolutionCommand,
        actor: ActorContext,
    ) -> Result<FeedbackResolution, FeedbackError>;

    /// 标记为已应用(由 ChangeSet 提交触发)
    async fn mark_applied(
        &self,
        feedback_id: FeedbackId,
        change_set_id: uuid::Uuid,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError>;

    /// 标记为已验证(由 Validation 通过触发)
    async fn mark_verified(
        &self,
        feedback_id: FeedbackId,
        validation_result_id: uuid::Uuid,
        evidence: Vec<ResolutionEvidence>,
        actor: ActorContext,
    ) -> Result<Feedback, FeedbackError>;

    /// 记录被消费(AgentSession / ContextPacket / ChangeSet)
    async fn record_consumed(
        &self,
        feedback_id: FeedbackId,
        consumed_by: crate::entity::ConsumedByKind,
        consumed_by_id: uuid::Uuid,
        actor: ActorContext,
    ) -> Result<FeedbackConsumedEvent, FeedbackError>;
}

// =====================================================================
// 端口:FeedbackQueryPort
// =====================================================================

/// **Feedback 查询端口**
#[async_trait]
pub trait FeedbackQueryPort: Send + Sync {
    /// 按 ID 获取
    async fn get_by_id(
        &self,
        id: FeedbackId,
        viewer: ActorContext,
    ) -> Result<Feedback, FeedbackError>;

    /// 按 project / work_item / status 过滤
    async fn list_by_project(
        &self,
        q: ListFeedbackQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Feedback>, FeedbackError>;

    /// P0-P3 优先级排序的 Inbox(§4.3.6)
    async fn inbox(
        &self,
        q: FeedbackInboxQuery,
        viewer: ActorContext,
    ) -> Result<Vec<FeedbackInboxItem>, FeedbackError>;

    /// 列出消费事件
    async fn list_consumed_events(
        &self,
        feedback_id: FeedbackId,
        viewer: ActorContext,
    ) -> Result<Vec<FeedbackConsumedEvent>, FeedbackError>;

    /// 列出 Resolution
    async fn list_resolutions(
        &self,
        feedback_id: FeedbackId,
        viewer: ActorContext,
    ) -> Result<Vec<FeedbackResolution>, FeedbackError>;
}

// =====================================================================
// 仓库端口
// =====================================================================

/// **Feedback 仓库端口**
#[async_trait]
pub trait FeedbackRepository: Send + Sync {
    async fn insert_feedback(&self, f: &Feedback) -> Result<(), FeedbackError>;
    async fn save_feedback(&self, f: &Feedback) -> Result<(), FeedbackError>;
    async fn find_feedback(
        &self,
        id: FeedbackId,
    ) -> Result<Option<Feedback>, FeedbackError>;
    async fn list_feedbacks_raw(
        &self,
        q: ListFeedbackQuery,
    ) -> Result<Vec<Feedback>, FeedbackError>;
    async fn insert_resolution(
        &self,
        r: &FeedbackResolution,
    ) -> Result<(), FeedbackError>;
    async fn list_resolutions_raw(
        &self,
        feedback_id: FeedbackId,
    ) -> Result<Vec<FeedbackResolution>, FeedbackError>;
    async fn insert_consumed_event(
        &self,
        e: &FeedbackConsumedEvent,
    ) -> Result<(), FeedbackError>;
    async fn list_consumed_events_raw(
        &self,
        feedback_id: FeedbackId,
    ) -> Result<Vec<FeedbackConsumedEvent>, FeedbackError>;
}

// 静默引用
#[allow(dead_code)]
fn _unused_user(u: UserId) -> UserId {
    u
}
#[allow(dead_code)]
fn _unused_resid(r: FeedbackResolutionId) -> FeedbackResolutionId {
    r
}
#[allow(dead_code)]
fn _unused_ag_sid(s: AgentSessionId) -> AgentSessionId {
    s
}

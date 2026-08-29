//! Feedback 域实体
//!
//! 来源:
//! - `docs/data-design.md` §4.22 (`feedback` schema)
//! - `docs/specs/domain-feedback-spec.md` §2 (实体清单)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AgentId, AgentSessionId, DecisionId, FeedbackId, FeedbackResolutionId, FeedbackStatus,
    FeedbackTarget, FeedbackType, ProjectId, Severity, TenantId, UserId, WorkItemId,
};

// =====================================================================
// Feedback 聚合根
// =====================================================================

/// **Feedback 聚合根**(spec §2, §7.3 6 状态机)
///
/// 必带字段(INV-FB-06):
/// - `tenant_id` — 跨 tenant 拒绝
/// - `project_id` — Project 隔离
/// - `target` — 必可解析(INV-FB-02),`Feedback` ≠ `Comment`(INV-FB-08)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// 主键
    pub id: FeedbackId,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: TenantId,
    /// Project 隔离
    pub project_id: ProjectId,
    /// 关联 WorkItem(便于 Inbox 投影)
    pub work_item_id: WorkItemId,
    /// 关联 AgentSession(可空;Agent 拉取后填入,用于投影 OPEN → ACKNOWLEDGED)
    pub agent_session_id: Option<AgentSessionId>,
    /// 目标(11 种,spec §7,SOW 任务范围)
    pub target: FeedbackTarget,
    /// Feedback 类型(7 类)
    pub r#type: FeedbackType,
    /// 严重程度(P0-P3)
    pub severity: Severity,
    /// 短句意图(`"重构为 AuthProvider"`)
    pub intent: String,
    /// 期望行为
    pub expected_behavior: String,
    /// 必须保留的(逗号/列表)
    pub preserve: Vec<String>,
    /// 禁止修改的
    pub prohibit: Vec<String>,
    /// 作者 user
    pub author_user_id: UserId,
    /// 作者 agent(INV-FB-07 AI 自己提的也记录)
    pub author_agent_id: Option<AgentId>,
    /// 关联 AC(可空)
    pub acceptance_criteria_id: Option<uuid::Uuid>,
    /// 前驱 Feedback(Supersede 链,INV-FB-04)
    pub predecessor_id: Option<FeedbackId>,
    /// 后继 Feedback(Supersede 链)
    pub successor_id: Option<FeedbackId>,
    /// 状态(6 状态机)
    pub status: FeedbackStatus,
    /// 解决证据(由 FeedbackResolution 收集)
    pub resolution_evidence: Vec<ResolutionEvidence>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 解决时间(VERIFIED/REJECTED/SUPERSEDED 时填)
    pub resolved_at: Option<DateTime<Utc>>,
    /// 乐观锁版本
    pub lock_version: u32,
}

impl Feedback {
    /// 字段数(审计)
    pub const FIELD_COUNT: usize = 20;

    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// 是否能 update(仅 OPEN/ACKNOWLEDGED 可改,APPLIED 后只读,INV-FB 类似)
    pub fn is_editable(&self) -> bool {
        matches!(
            self.status,
            FeedbackStatus::Open | FeedbackStatus::Acknowledged
        )
    }

    /// 是否能 delete(仅 OPEN,FB-005)
    pub fn is_deletable(&self) -> bool {
        matches!(self.status, FeedbackStatus::Open)
    }

    /// 状态迁移(`from -> to` 校验 + 更新)
    pub fn transition(&mut self, to: FeedbackStatus) -> Result<(), String> {
        if !self.status.can_transition_to(to) {
            return Err(format!("非法 6 状态迁移: {} -> {}", self.status, to));
        }
        self.status = to;
        self.bump_version();
        if matches!(
            to,
            FeedbackStatus::Verified | FeedbackStatus::Rejected | FeedbackStatus::Superseded
        ) {
            self.resolved_at = Some(Utc::now());
        }
        Ok(())
    }

    /// 乐观锁版本递增
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
    }
}

// =====================================================================
// ResolutionEvidence(解决证据)
// =====================================================================

/// **解决证据**(1 条 = 1 个引用:ValidationResult / TestResult / Manual)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionEvidence {
    /// 证据类型
    pub kind: EvidenceKind,
    /// 引用 ID(ValidationResult / Test / ChangeSet)
    pub ref_id: uuid::Uuid,
    /// 备注
    pub note: String,
    /// 时间
    pub at: DateTime<Utc>,
}

/// **证据类型**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceKind {
    /// 验证结果
    ValidationResult,
    /// 测试结果
    TestResult,
    /// 手动评审
    ManualReview,
    /// ChangeSet 提交
    ChangeSetCommit,
}

// =====================================================================
// FeedbackResolution 实体(spec §2,basic-design §4.3.2)
// =====================================================================

/// **FeedbackResolution**(Resolution 实体,evidence_refs 集合)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackResolution {
    /// 主键
    pub id: FeedbackResolutionId,
    /// 租户隔离
    pub tenant_id: TenantId,
    /// 关联 Feedback
    pub feedback_id: FeedbackId,
    /// 解析人
    pub resolver_user_id: UserId,
    /// 解析人 agent(可空)
    pub resolver_agent_id: Option<AgentId>,
    /// 目标状态(VERIFIED / REJECTED / SUPERSEDED)
    pub resolved_status: FeedbackStatus,
    /// 证据引用集合
    pub evidence_refs: Vec<ResolutionEvidenceRef>,
    /// 备注
    pub note: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl FeedbackResolution {
    /// 字段数
    pub const FIELD_COUNT: usize = 9;
}

/// **Resolution 证据引用**(比 `ResolutionEvidence` 轻量,只引用,不存内联内容)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionEvidenceRef {
    /// 证据类型
    pub kind: EvidenceKind,
    /// 引用 ID
    pub ref_id: uuid::Uuid,
    /// 备注
    pub note: String,
}

// =====================================================================
// FeedbackConsumedEvent(Projection,§4.3.2)
// =====================================================================

/// **Feedback 消费事件**(Projection,记录被哪些 ContextPacket / AgentSession / ChangeSet 消费)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConsumedEvent {
    /// 事件 ID
    pub event_id: uuid::Uuid,
    /// 关联 Feedback
    pub feedback_id: FeedbackId,
    /// 租户隔离
    pub tenant_id: TenantId,
    /// 消费方
    pub consumed_by: ConsumedByKind,
    /// 消费方 ID(AgentSession / ContextPacket / ChangeSet)
    pub consumed_by_id: uuid::Uuid,
    /// 消费时间
    pub consumed_at: DateTime<Utc>,
}

/// **消费方类型**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsumedByKind {
    /// Agent Session 拉取
    AgentSession,
    /// Context Packet 编译
    ContextPacket,
    /// ChangeSet 提交
    ChangeSet,
}

// =====================================================================
// FeedbackInboxItem(Inbox 投影,§4.3.6)
// =====================================================================

/// **Feedback Inbox 项**(P0-P3 优先级排序的投影)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackInboxItem {
    pub feedback_id: FeedbackId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub work_item_id: WorkItemId,
    pub target: FeedbackTarget,
    pub r#type: FeedbackType,
    pub severity: Severity,
    pub status: FeedbackStatus,
    pub intent: String,
    pub created_at: DateTime<Utc>,
}

// 静默引用
#[allow(dead_code)]
fn _unused_decision(_: DecisionId) -> DecisionId {
    DecisionId::new()
}

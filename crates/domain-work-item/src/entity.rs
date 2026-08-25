//! WorkItem 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.4.1 (`work_item` DDL,22 字段)
//! - `docs/specs/domain-work-item-spec.md` §2 (实体清单)
//!
//! 包含 4 个核心实体:
//! - `WorkItem` — 主聚合根(22 字段)
//! - `Requirement` — 业务需求(§4.4.2)
//! - `AcceptanceCriterion` — 验收标准(§4.4.3)
//! - `BusinessGoal` — 业务目标(§4.4.4)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AcceptanceCriterionId, AgentId, BusinessGoalId, Priority, ProjectId, RepositoryId, RequirementId,
    Severity, SprintId, TenantId, UserId, WorkItemId, WorkItemStatus, WorkItemType, WorktreeId,
    WorkspaceId,
};

// =====================================================================
// WorkItem 聚合根(§4.4.1,22 字段)
// =====================================================================

/// **WorkItem 聚合根**(继承 `data-design §4.4.1` DDL,共 28 字段)
///
/// 关键约束:
/// - 必带 `tenant_id`(INV-WI-07,REQ-SEC-001)
/// - `type = Subtask` 时 `parent_work_item_id` 必非空(INV-WI-08)
/// - `type = AITask` 时必先有 Repository + Agent + Validation 链接(INV-WI-05)
/// - 删除前需级联检查 Worktree(INV-WI-06)
/// - 默认 3 态 TODO/IN_PROGRESS/DONE(INV-WI-01)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// 主键 UUID(由 infra 在 INSERT 时由 DB `gen_random_uuid()` 或本 crate 颁发)
    pub id: WorkItemId,

    /// 租户 ID(13 类对象必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// Workspace ID
    pub workspace_id: WorkspaceId,

    /// Project ID
    pub project_id: ProjectId,

    /// 类型:6 种合法值(见 [`WorkItemType`])
    pub work_item_type: WorkItemType,

    /// 业务键(`Project Key + 序列号`,如 "STAR-100",UQ `(tenant_id, project_id, key, deleted_at)`)
    pub work_item_key: String,

    /// 标题(VARCHAR(500))
    pub title: String,

    /// 详细描述(TEXT,可空)
    pub description: String,

    /// 状态:默认 3 态 TODO/IN_PROGRESS/DONE(扩展 3 态需 Project Policy 启用)
    pub status: WorkItemStatus,

    /// 优先级 P0~P3(默认 P3)
    pub priority: Priority,

    /// 严重度(Bug 类型专用,见 [`Severity`])
    pub severity: Severity,

    /// Scrum 故事点(可空)
    pub story_points: Option<u32>,

    /// 所属 Sprint ID(可空)
    pub sprint_id: Option<SprintId>,

    /// 父 WorkItem ID(Subtask 必填)
    pub parent_work_item_id: Option<WorkItemId>,

    /// 关联的 Requirement IDs(冗余字段,主源在 `requirement.linked_work_item_ids`)
    pub requirement_ids: Vec<RequirementId>,

    /// 关联的 AcceptanceCriterion IDs(冗余字段,主源在 `acceptance_criterion.work_item_id`)
    pub acceptance_criterion_ids: Vec<AcceptanceCriterionId>,

    /// 关联的 Repository IDs(0..N,INV-WI-03)
    pub repository_ids: Vec<RepositoryId>,

    /// 关联的 Worktree IDs(0..N,INV-WI-04;冗余字段,主源在 `worktree.work_item_id`)
    pub worktree_ids: Vec<WorktreeId>,

    /// 分配的用户 ID(可空)
    pub assignee_user_id: Option<UserId>,

    /// 分配的 AI Agent ID(可空,AITask 必填)
    pub assignee_agent_id: Option<AgentId>,

    /// 报告人(创建者,NOT NULL)
    pub reporter_user_id: UserId,

    /// 标签列表
    pub labels: Vec<String>,

    /// 组件列表
    pub components: Vec<String>,

    /// 截止日期(可空)
    pub due_date: Option<DateTime<Utc>>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 软删除时间(可空,NULL 表示未删除)
    pub deleted_at: Option<DateTime<Utc>>,

    /// 乐观锁版本号(初始 1,每次 UPDATE 自增)
    pub version: u32,
}

impl WorkItem {
    /// 字段数(用于 §4.4.1 DDL 对齐审计)。
    pub const FIELD_COUNT: usize = 28;

    /// 是否为软删除状态。
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// 是否有未完成的工作树(INV-WI-06 删除前检查)。
    pub fn has_active_worktrees(&self) -> bool {
        !self.worktree_ids.is_empty()
    }

    /// 升级乐观锁版本号(每次 UPDATE 自增,供 `update_work_item` impl 使用)。
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Requirement(§4.4.2)
// =====================================================================

/// **Requirement**(业务需求,可关联多个 WorkItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// 主键
    pub id: RequirementId,

    /// 租户 ID(必带)
    pub tenant_id: TenantId,

    /// 所属 BusinessGoal(可空)
    pub business_goal_id: Option<BusinessGoalId>,

    /// 需求声明(TEXT,NOT NULL)
    pub statement: String,

    /// 需求理由 / 背景
    pub rationale: Option<String>,

    /// 关联的 WorkItem IDs
    pub linked_work_item_ids: Vec<WorkItemId>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 软删除时间
    pub deleted_at: Option<DateTime<Utc>>,

    /// 乐观锁版本
    pub version: u32,
}

// =====================================================================
// AcceptanceCriterion(§4.4.3)
// =====================================================================

/// **AcceptanceCriterion 覆盖状态**(由 `domain-validation` 写入)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoverageStatus {
    /// 已覆盖
    COVERED,
    /// 部分覆盖
    PARTIAL,
    /// 未覆盖(默认)
    UNCOVERED,
    /// 有争议
    DISPUTED,
}

impl Default for CoverageStatus {
    fn default() -> Self {
        Self::UNCOVERED
    }
}

/// **AcceptanceCriterion**(验收标准;coverage_status 由 validation 写入)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// 主键
    pub id: AcceptanceCriterionId,

    /// 租户 ID(必带)
    pub tenant_id: TenantId,

    /// 所属 WorkItem(必填,NOT NULL)
    pub work_item_id: WorkItemId,

    /// 所属 Requirement(可空)
    pub requirement_id: Option<RequirementId>,

    /// 验收标准声明
    pub statement: String,

    /// 覆盖状态(由 validation 写入)
    pub coverage_status: CoverageStatus,

    /// 覆盖该 AC 的 Validation IDs
    pub covered_by_validation_ids: Vec<uuid::Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 软删除时间
    pub deleted_at: Option<DateTime<Utc>>,

    /// 乐观锁版本
    pub version: u32,
}

// =====================================================================
// BusinessGoal(§4.4.4)
// =====================================================================

/// **BusinessGoal**(业务目标;Requirement 关联到 BusinessGoal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessGoal {
    /// 主键
    pub id: BusinessGoalId,

    /// 租户 ID(必带)
    pub tenant_id: TenantId,

    /// 目标声明
    pub statement: String,

    /// 详细描述
    pub description: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 软删除时间
    pub deleted_at: Option<DateTime<Utc>>,

    /// 乐观锁版本
    pub version: u32,
}

// =====================================================================
// WorkItemRelation(横向关系,§4.4.7)
// =====================================================================

/// **WorkItem 横向关系**(`work_item_relation` 表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemRelation {
    /// 主键
    pub id: uuid::Uuid,

    /// 租户 ID(必带)
    pub tenant_id: TenantId,

    /// 源 WorkItem
    pub source_work_item_id: WorkItemId,

    /// 目标 WorkItem
    pub target_work_item_id: WorkItemId,

    /// 关系类型
    pub relation_type: crate::value_object::RelationType,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

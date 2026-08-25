//! WorkItem 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.5 (CRUD + 状态机 + AC 端点)
//! - `docs/specs/domain-work-item-spec.md` §4 (接口签名)
//!
//! **端口清单**(保持骨架阶段锁定的 14 个方法签名):
//! - `WorkItemCommandPort`:8 方法(写)
//! - `WorkItemQueryPort`:6 方法(读)
//!
//! 具体 Adapter 由 `crates/infrastructure` 提供(SQLx / NATS / SCM),
//! Phase 2 提供 1-2 个 in-memory 实现于 `service.rs`。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::context::ActorContext;
use crate::entity::{AcceptanceCriterion, BusinessGoal, Requirement, WorkItem};
use crate::error::WorkItemError;
use crate::value_object::{
    Priority, ProjectId, RepositoryId, RequirementId, Severity, TenantId,
    WorkItemId, WorkItemStatus, WorkItemType,
};

// =====================================================================
// 命令 DTO(写操作输入)
// =====================================================================

/// `CreateWorkItemCommand`(创建 WorkItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkItemCommand {
    /// 租户 ID(必带)
    pub tenant_id: TenantId,
    /// Workspace ID
    pub workspace_id: Uuid,
    /// Project ID
    pub project_id: ProjectId,
    /// 类型
    pub work_item_type: WorkItemType,
    /// 业务键(`STAR-100`)
    pub work_item_key: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 优先级(默认 P3)
    pub priority: Priority,
    /// 严重度(Bug 类型专用)
    pub severity: Option<Severity>,
    /// 故事点
    pub story_points: Option<u32>,
    /// 父 WorkItem ID(Subtask 必填)
    pub parent_work_item_id: Option<WorkItemId>,
    /// 报告人(创建者,默认取 actor.user_id)
    pub reporter_user_id: Uuid,
    /// 截止日期
    pub due_date: Option<DateTime<Utc>>,
}

/// `UpdateWorkItemCommand`(更新 WorkItem,乐观锁)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkItemCommand {
    /// 目标 WorkItem ID
    pub work_item_id: WorkItemId,
    /// 租户 ID(校验用)
    pub tenant_id: TenantId,
    /// 期望版本号(乐观锁)
    pub expected_version: u32,
    /// 新标题(`None` 表示不修改)
    pub title: Option<String>,
    /// 新描述
    pub description: Option<String>,
    /// 新优先级
    pub priority: Option<Priority>,
    /// 新严重度
    pub severity: Option<Severity>,
    /// 新故事点
    pub story_points: Option<Option<u32>>,
    /// 新截止日期
    pub due_date: Option<Option<DateTime<Utc>>>,
    /// 新分配人(`Some(None)` 表示清空)
    pub assignee_user_id: Option<Option<Uuid>>,
    /// 新分配 Agent(`Some(None)` 表示清空)
    pub assignee_agent_id: Option<Option<Uuid>>,
}

/// `TransitionStatusCommand`(状态机迁移)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionStatusCommand {
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 目标状态
    pub target_status: WorkItemStatus,
    /// 期望版本号(乐观锁)
    pub expected_version: u32,
    /// 迁移原因(可选,记入审计)
    pub reason: Option<String>,
}

/// `DeleteWorkItemCommand`(软删除,带 Worktree 级联检查)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorkItemCommand {
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 期望版本号
    pub expected_version: u32,
}

/// `WorkItemBulkUpdate`(批量更新,Phase 3 实现)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemBulkUpdate {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 目标 IDs
    pub work_item_ids: Vec<WorkItemId>,
    /// 公共更新字段(优先级 / 分配人 / Sprint)
    pub priority: Option<Priority>,
    /// 新分配人(`Some(None)` 表示清空)
    pub assignee_user_id: Option<Option<Uuid>>,
    /// 新 Sprint(`Some(None)` 表示清空)
    pub sprint_id: Option<Option<Uuid>>,
}

/// `LinkRepositoryCommand`(关联 Repository)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRepositoryCommand {
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Repository ID
    pub repository_id: RepositoryId,
}

/// `CreateRequirementCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequirementCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// BusinessGoal ID(可空)
    pub business_goal_id: Option<Uuid>,
    /// 需求声明
    pub statement: String,
    /// 需求理由
    pub rationale: Option<String>,
    /// 关联 WorkItem IDs
    pub linked_work_item_ids: Vec<WorkItemId>,
}

/// `CreateAcceptanceCriterionCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAcceptanceCriterionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 所属 WorkItem
    pub work_item_id: WorkItemId,
    /// 所属 Requirement(可空)
    pub requirement_id: Option<RequirementId>,
    /// 验收标准声明
    pub statement: String,
}

// =====================================================================
// 批量结果 DTO
// =====================================================================

/// 批量操作结果(成功数 / 失败条目)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResult {
    /// 总数
    pub total: usize,
    /// 成功数
    pub succeeded: usize,
    /// 失败条目(WorkItemId + 错误信息)
    pub failed: Vec<BulkFailure>,
}

impl BulkResult {
    /// 全部成功的便捷构造。
    pub fn all_succeeded(total: usize) -> Self {
        Self {
            total,
            succeeded: total,
            failed: Vec::new(),
        }
    }
}

/// 单个失败条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkFailure {
    /// 失败 WorkItem ID
    pub work_item_id: WorkItemId,
    /// 错误信息
    pub error: String,
}

// =====================================================================
// 查询 DTO(读操作输入)
// =====================================================================

/// `ListWorkItemQuery`(列表查询)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkItemQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 按状态过滤(可空)
    pub status: Option<WorkItemStatus>,
    /// 按类型过滤(可空)
    pub work_item_type: Option<WorkItemType>,
    /// 按分配人过滤(可空)
    pub assignee_user_id: Option<Uuid>,
    /// 按 Sprint 过滤(可空)
    pub sprint_id: Option<Uuid>,
    /// 父 WorkItem 过滤(可空,只查子项)
    pub parent_work_item_id: Option<WorkItemId>,
    /// 分页:limit
    pub limit: u32,
    /// 分页:offset
    pub offset: u32,
}

/// `ListBusinessGoalQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBusinessGoalQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 分页:limit
    pub limit: u32,
    /// 分页:offset
    pub offset: u32,
}

impl ListWorkItemQuery {
    /// 构造默认分页 50/0
    pub fn for_project(tenant_id: TenantId, project_id: ProjectId) -> Self {
        Self {
            tenant_id,
            project_id,
            status: None,
            work_item_type: None,
            assignee_user_id: None,
            sprint_id: None,
            parent_work_item_id: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// `Transition`(状态机迁移记录)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// 迁移 ID
    pub id: Uuid,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// 旧状态
    pub from_status: WorkItemStatus,
    /// 新状态
    pub to_status: WorkItemStatus,
    /// 触发者
    pub actor_user_id: Uuid,
    /// 原因
    pub reason: Option<String>,
    /// 发生时间
    pub occurred_at: DateTime<Utc>,
}

// =====================================================================
// 端口:WorkItemCommandPort(8 方法)
// =====================================================================

/// **WorkItem 命令端口**(写操作 8 方法)
#[async_trait]
pub trait WorkItemCommandPort: Send + Sync {
    /// 创建 WorkItem(INV-WI-07 校验 tenant_id,INV-WI-05/08 类型前置条件)
    async fn create_work_item(
        &self,
        cmd: CreateWorkItemCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 更新 WorkItem(乐观锁 version 校验)
    async fn update_work_item(
        &self,
        cmd: UpdateWorkItemCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 软删除(INV-WI-06:有 Worktree 时拒绝)
    async fn delete_work_item(
        &self,
        cmd: DeleteWorkItemCommand,
        actor: ActorContext,
    ) -> Result<(), WorkItemError>;

    /// 状态机迁移(INV-WI-09:由 WorkflowDefinition 判定合法迁移)
    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 批量更新(部分成功)
    async fn bulk_update(
        &self,
        cmd: WorkItemBulkUpdate,
        actor: ActorContext,
    ) -> Result<BulkResult, WorkItemError>;

    /// 关联 Repository(INV-WI-03:0..N)
    async fn link_repository(
        &self,
        cmd: LinkRepositoryCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 创建 Requirement
    async fn create_requirement(
        &self,
        cmd: CreateRequirementCommand,
        actor: ActorContext,
    ) -> Result<Requirement, WorkItemError>;

    /// 创建 AcceptanceCriterion
    async fn create_acceptance_criterion(
        &self,
        cmd: CreateAcceptanceCriterionCommand,
        actor: ActorContext,
    ) -> Result<AcceptanceCriterion, WorkItemError>;
}

// =====================================================================
// 端口:WorkItemQueryPort(6 方法)
// =====================================================================

/// **WorkItem 查询端口**(读操作 6 方法)
#[async_trait]
pub trait WorkItemQueryPort: Send + Sync {
    /// 按项目列出 WorkItem(支持分页 / 过滤)
    async fn list_by_project(
        &self,
        q: ListWorkItemQuery,
        viewer: ActorContext,
    ) -> Result<Vec<WorkItem>, WorkItemError>;

    /// 按 ID 查询(带租户隔离校验)
    async fn get_by_id(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 查询 WorkItem 状态机迁移历史
    async fn list_transitions(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<Transition>, WorkItemError>;

    /// 列出 WorkItem 关联的 Requirement
    async fn list_requirements(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<Requirement>, WorkItemError>;

    /// 列出 WorkItem 关联的 AcceptanceCriterion
    async fn list_acceptance_criteria(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<AcceptanceCriterion>, WorkItemError>;

    /// 列出 BusinessGoal(按租户)
    async fn list_business_goals(
        &self,
        q: ListBusinessGoalQuery,
        viewer: ActorContext,
    ) -> Result<Vec<BusinessGoal>, WorkItemError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **WorkItem 仓库端口**(供 SQLx / 内存 / 测试 Adapter 实现)
///
/// 与 `WorkItemCommandPort` 的区别:仓库是纯数据访问层(无租户校验 / 事件发布 / 不变量),
/// CommandPort 在仓库之上叠加业务规则。
#[async_trait]
pub trait WorkItemRepository: Send + Sync {
    /// 插入新 WorkItem(返回插入后的实体)
    async fn insert(&self, work_item: &WorkItem) -> Result<(), WorkItemError>;

    /// 按 ID 读取(返回 `None` 表示不存在)
    async fn find_by_id(&self, id: WorkItemId) -> Result<Option<WorkItem>, WorkItemError>;

    /// 更新(乐观锁)
    async fn update(&self, work_item: &WorkItem) -> Result<(), WorkItemError>;

    /// 软删除(置 `deleted_at`)
    async fn soft_delete(&self, id: WorkItemId) -> Result<(), WorkItemError>;

    /// 按 project + tenant 列出
    async fn list_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<WorkItem>, WorkItemError>;
}

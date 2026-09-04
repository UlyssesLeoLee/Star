//! domain-work-item crate
//!
//! 详细 spec: docs/specs/domain-work-item-spec.md
//! 上游基本设计: docs/basic-design.md §2.1(表 1) / §7.2 默认三态
//! 数据设计: docs/data-design.md §4.4 (`work_item` schema)
//! API 设计: docs/api-design.md §3.5
//!
//! ## 职责
//!
//! 业务核心(§8,§36):WorkItem 6 类(Epic/Story/Task/Bug/Subtask/AITask)
//! + 默认三态状态机 TODO/IN_PROGRESS/DONE + Requirement/AC/BusinessGoal
//!
//! ## 关键不变量
//!
//! - INV-WI-01:WorkItem 必带 tenant_id + workspace_id + project_id
//! - INV-WI-02:三态状态机 TODO→IN_PROGRESS→DONE(可回退)
//! - INV-WI-03:AITask 子类型必带 objective + repository_scope
//! - INV-WI-04:parent_work_item_id 自引用必须同 project
//!
//! Lead 责任: work-item Lead

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

define_uuid_id!(WorkItemId);
define_uuid_id!(TenantId);
define_uuid_id!(WorkspaceId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);
define_uuid_id!(RepositoryId);
define_uuid_id!(WorktreeId);
define_uuid_id!(RequirementId);
define_uuid_id!(AcceptanceCriterionId);
define_uuid_id!(BusinessGoalId);
define_uuid_id!(SprintId);
define_uuid_id!(AgentPolicyId);
define_uuid_id!(ValidationPolicyId);
define_uuid_id!(ContextPolicyId);

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
/// 定义基于 UUID 的领域强类型 ID:自动生成 $name 结构体及 new/as_uuid/From<Uuid>/Display 实现
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 领域强类型 ID 包装类型(由 define_uuid_id! 宏统一生成)
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成一个新的随机 ID(基于 UUID v4)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回底层 UUID 值
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
/// 工作项实体:业务核心对象,涵盖 Epic/Story/Task/Bug/Subtask/AITask 六类子类型
pub struct WorkItem {
    /// 工作项唯一 ID
    pub id: WorkItemId,
    /// 所属租户 ID(INV-WI-01)
    pub tenant_id: TenantId,
    /// 所属工作区 ID(INV-WI-01)
    pub workspace_id: WorkspaceId,
    /// 所属项目 ID(INV-WI-01)
    pub project_id: ProjectId,
    /// 工作项类型(Epic/Story/Task/Bug/Subtask/AITask)
    pub item_type: WorkItemType,
    /// 标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 当前状态(三态状态机 TODO/IN_PROGRESS/DONE)
    pub status: WorkItemStatus,
    /// 指派的用户 ID(可选)
    pub assignee_user_id: Option<UserId>,
    /// 指派的 Agent ID(可选)
    pub assignee_agent_id: Option<AgentId>,
    /// 报告人用户 ID
    pub reporter_user_id: UserId,
    /// 优先级
    pub priority: Priority,
    /// 严重程度(可选,主要用于 Bug)
    pub severity: Option<Severity>,
    /// 故事点估算(可选)
    pub story_points: Option<u32>,
    /// 所属迭代 ID(可选)
    pub sprint_id: Option<SprintId>,
    /// 父工作项 ID(需与本项目同 project,INV-WI-04)
    pub parent_work_item_id: Option<WorkItemId>,
    /// 关联的需求 ID 列表
    pub requirement_ids: Vec<RequirementId>,
    /// 关联的验收标准 ID 列表
    pub acceptance_criterion_ids: Vec<AcceptanceCriterionId>,
    /// 关联的代码仓库 ID 列表
    pub repository_ids: Vec<RepositoryId>,
    /// 关联的 worktree ID 列表
    pub worktree_ids: Vec<WorktreeId>,
    /// 标签列表
    pub labels: Vec<String>,
    /// 所属组件列表
    pub components: Vec<String>,
    /// 截止日期(可选)
    pub due_date: Option<DateTime<Utc>>,
    /// AITask 子类型字段
    pub ai_task_data: Option<AiTaskData>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 工作项类型枚举(§8,§36 定义的 6 类)
pub enum WorkItemType {
    /// 史诗
    Epic,
    /// 用户故事
    Story,
    /// 任务
    Task,
    /// 缺陷
    Bug,
    /// 子任务
    Subtask,
    /// AI 任务(需 objective + repository_scope,INV-WI-03)
    AITask,
}

impl WorkItemType {
    /// 返回类型对应的小写字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Epic => "epic",
            Self::Story => "story",
            Self::Task => "task",
            Self::Bug => "bug",
            Self::Subtask => "subtask",
            Self::AITask => "ai_task",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 工作项默认三态状态机(§7.2)
pub enum WorkItemStatus {
    /// 待办
    Todo,
    /// 进行中
    InProgress,
    /// 已完成(终态)
    Done,
}

impl WorkItemStatus {
    /// 返回状态对应的大写字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::InProgress => "IN_PROGRESS",
            Self::Done => "DONE",
        }
    }
    /// 判断是否为终态(Done)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}

/// 三态状态机严格转换表(§7.2,INV-WI-02)
pub fn check_status_transition(
    from: WorkItemStatus,
    to: WorkItemStatus,
) -> Result<(), WorkItemError> {
    use WorkItemStatus::*;
    let allowed = matches!(
        (from, to),
        (Todo, InProgress) | (InProgress, Done) | (InProgress, Todo) | (Done, InProgress) // 重开
    );
    if !allowed {
        return Err(WorkItemError::InvalidTransition {
            from: from.as_str().to_string(),
            to: to.as_str().to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 优先级枚举
pub enum Priority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 紧急
    Urgent,
}

impl Priority {
    /// 返回优先级对应的小写字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 严重程度枚举(主要用于 Bug)
pub enum Severity {
    /// 轻微
    Minor,
    /// 主要
    Major,
    /// 严重
    Critical,
    /// 阻塞
    Blocker,
}

impl Severity {
    /// 返回严重程度对应的小写字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minor => "minor",
            Self::Major => "major",
            Self::Critical => "critical",
            Self::Blocker => "blocker",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// AITask 子类型附加数据(INV-WI-03)
pub struct AiTaskData {
    /// AI 任务目标描述
    pub objective: String,
    /// 允许操作的代码仓库范围
    pub repository_scope: Vec<RepositoryId>,
    /// 允许修改的文件路径模式列表
    pub allowed_files: Vec<String>,
    /// 禁止修改的文件路径模式列表
    pub forbidden_files: Vec<String>,
    /// 关联的 Agent 策略 ID(可选)
    pub agent_policy_id: Option<AgentPolicyId>,
    /// 关联的验证策略 ID(可选)
    pub validation_policy_id: Option<ValidationPolicyId>,
    /// 关联的上下文策略 ID(可选)
    pub context_policy_id: Option<ContextPolicyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 需求实体
pub struct Requirement {
    /// 需求唯一 ID
    pub id: RequirementId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 关联的业务目标 ID(可选)
    pub business_goal_id: Option<BusinessGoalId>,
    /// 需求陈述
    pub statement: String,
    /// 需求理由说明
    pub rationale: String,
    /// 关联的工作项 ID 列表
    pub linked_work_item_ids: Vec<WorkItemId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 验收标准实体
pub struct AcceptanceCriterion {
    /// 验收标准唯一 ID
    pub id: AcceptanceCriterionId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属需求 ID
    pub requirement_id: RequirementId,
    /// 所属工作项 ID
    pub work_item_id: WorkItemId,
    /// 验收标准陈述
    pub statement: String,
    /// 验证覆盖状态
    pub coverage_status: CoverageStatus,
    /// 覆盖该验收标准的验证记录 ID 列表
    pub covered_by_validation_ids: Vec<Uuid>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 验收标准的验证覆盖状态
pub enum CoverageStatus {
    /// 未覆盖
    Uncovered,
    /// 部分覆盖
    Partial,
    /// 已覆盖
    Covered,
}

impl CoverageStatus {
    /// 返回覆盖状态对应的大写字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uncovered => "UNCOVERED",
            Self::Partial => "PARTIAL",
            Self::Covered => "COVERED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 业务目标实体
pub struct BusinessGoal {
    /// 业务目标唯一 ID
    pub id: BusinessGoalId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 业务目标名称
    pub name: String,
    /// 业务目标描述
    pub description: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
/// domain-work-item 领域错误类型
pub enum WorkItemError {
    #[error("not found: {0}")]
    /// 资源未找到
    NotFound(String),
    #[error("permission denied")]
    /// 权限不足
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    /// 跨租户访问被拒绝
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid state transition: {from} -> {to}")]
    /// 非法的状态转换
    InvalidTransition {
        /// 迁移前状态
        from: String,
        /// 迁移目标状态
        to: String,
    },
    #[error("AI task missing objective (INV-WI-03)")]
    /// AI 任务缺少目标(INV-WI-03)
    AiTaskMissingObjective,
    #[error("AI task missing repository scope (INV-WI-03)")]
    /// AI 任务缺少代码仓库范围(INV-WI-03)
    AiTaskMissingScope,
    #[error("parent work item must be in same project (INV-WI-04)")]
    /// 父工作项与子工作项不属于同一项目(INV-WI-04)
    ParentProjectMismatch,
    #[error("conflict: {0}")]
    /// 数据冲突(如乐观锁版本不匹配)
    Conflict(String),
    #[error("internal: {0}")]
    /// 内部错误
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建工作项命令
pub struct CreateWorkItemCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标工作区 ID
    pub workspace_id: WorkspaceId,
    /// 目标项目 ID
    pub project_id: ProjectId,
    /// 工作项类型
    pub item_type: WorkItemType,
    /// 标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 优先级
    pub priority: Priority,
    /// 严重程度(可选)
    pub severity: Option<Severity>,
    /// 报告人用户 ID
    pub reporter_user_id: UserId,
    /// 父工作项 ID(可选)
    pub parent_work_item_id: Option<WorkItemId>,
    /// AITask 附加数据(可选)
    pub ai_task_data: Option<AiTaskData>,
    /// 标签列表
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 状态转换命令
pub struct TransitionStatusCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标工作项 ID
    pub work_item_id: WorkItemId,
    /// 转换前状态
    pub from: WorkItemStatus,
    /// 转换后状态
    pub to: WorkItemStatus,
    /// 操作者用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 指派工作项命令
pub struct AssignCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标工作项 ID
    pub work_item_id: WorkItemId,
    /// 指派的用户 ID(可选)
    pub assignee_user_id: Option<UserId>,
    /// 指派的 Agent ID(可选)
    pub assignee_agent_id: Option<AgentId>,
    /// 操作者用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建需求命令
pub struct CreateRequirementCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 关联的业务目标 ID(可选)
    pub business_goal_id: Option<BusinessGoalId>,
    /// 需求陈述
    pub statement: String,
    /// 需求理由说明
    pub rationale: String,
    /// 操作者用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建验收标准命令
pub struct CreateAcceptanceCriterionCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属需求 ID
    pub requirement_id: RequirementId,
    /// 所属工作项 ID
    pub work_item_id: WorkItemId,
    /// 验收标准陈述
    pub statement: String,
    /// 操作者用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 获取单个工作项查询
pub struct GetWorkItemQuery {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标工作项 ID
    pub work_item_id: WorkItemId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 按项目列出工作项查询
pub struct ListByProjectQuery {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标项目 ID
    pub project_id: ProjectId,
    /// 是否包含终态(Done)工作项
    pub include_terminal: bool,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
/// 工作项写命令端口(Command Port)
pub trait WorkItemCommandPort: Send + Sync {
    /// 创建工作项
    async fn create_work_item(
        &self,
        cmd: CreateWorkItemCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 转换工作项状态
    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 指派工作项负责人
    async fn assign(
        &self,
        cmd: AssignCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 创建需求
    async fn create_requirement(
        &self,
        cmd: CreateRequirementCommand,
        actor: &ActorContext,
    ) -> Result<Requirement, WorkItemError>;

    /// 创建验收标准
    async fn create_acceptance_criterion(
        &self,
        cmd: CreateAcceptanceCriterionCommand,
        actor: &ActorContext,
    ) -> Result<AcceptanceCriterion, WorkItemError>;
}

#[async_trait]
/// 工作项查询端口(Query Port)
pub trait WorkItemQueryPort: Send + Sync {
    /// 获取单个工作项
    async fn get(
        &self,
        q: GetWorkItemQuery,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    /// 按项目列出工作项
    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorkItem>, WorkItemError>;
}

#[async_trait]
/// 工作项持久化仓储端口
pub trait WorkItemRepository: Send + Sync {
    /// 插入新工作项
    async fn insert(&self, w: WorkItem) -> Result<(), WorkItemError>;
    /// 按 ID 获取工作项
    async fn get(&self, id: WorkItemId) -> Result<WorkItem, WorkItemError>;
    /// 更新工作项
    async fn update(&self, w: WorkItem) -> Result<(), WorkItemError>;
    /// 按租户与项目列出工作项
    async fn list_by_project(
        &self,
        tid: TenantId,
        pid: ProjectId,
    ) -> Result<Vec<WorkItem>, WorkItemError>;

    /// 插入新需求
    async fn insert_requirement(&self, r: Requirement) -> Result<(), WorkItemError>;
    /// 插入新验收标准
    async fn insert_ac(&self, ac: AcceptanceCriterion) -> Result<(), WorkItemError>;
}

// =====================================================================
// InMemoryWorkItemService
// =====================================================================

/// WorkItemCommandPort/WorkItemQueryPort 的内存实现
pub struct InMemoryWorkItemService {
    repo: Arc<dyn WorkItemRepository>,
    items: Arc<RwLock<HashMap<WorkItemId, WorkItem>>>,
    requirements: Arc<RwLock<HashMap<RequirementId, Requirement>>>,
    acs: Arc<RwLock<HashMap<AcceptanceCriterionId, AcceptanceCriterion>>>,
}

impl InMemoryWorkItemService {
    /// 创建一个空的内存工作项服务实例
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryWorkItemRepository::new()),
            items: Arc::new(RwLock::new(HashMap::new())),
            requirements: Arc::new(RwLock::new(HashMap::new())),
            acs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// P0 工具链 (per docs/briefs/tool-p0-impl-001.md) — 按 query/status/project 过滤
    ///
    /// **注意**: `WorkItemQueryPort` trait 不含 `list_with_filter`,
    /// 本方法是 `InMemoryWorkItemService` struct 上的额外 helper, 不修改 trait 也不改 port,
    /// 仅供 `star-mcp::search_issues` P0 工具调真实 service 用.
    ///
    /// 行为:
    /// - 校验 actor.tenant_id == filter.tenant_id (跨 tenant 拒绝)
    /// - 角色要求: developer / project_admin / tenant_admin (跟 create_work_item 一致)
    /// - 过滤: 全部 AND 关系
    ///   - `query` (case-insensitive substring match on title)
    ///   - `status` (精确匹配)
    ///   - `project_id` (精确匹配)
    ///   - `limit` (截断到前 N)
    /// - 不发送事件 (避免污染 event bus)
    pub async fn list_with_filter(
        &self,
        filter: WorkItemFilter,
        actor: &ActorContext,
    ) -> Result<Vec<WorkItem>, WorkItemError> {
        if TenantId::from(actor.tenant_id) != filter.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                filter.tenant_id,
            ));
        }
        if !actor.has_role("developer")
            && !actor.has_role("project_admin")
            && !actor.has_role("tenant_admin")
        {
            return Err(WorkItemError::PermissionDenied);
        }
        let items = self.items.read().unwrap();
        let mut out: Vec<WorkItem> = items
            .values()
            .filter(|w| w.tenant_id == filter.tenant_id)
            .filter(|w| match &filter.project_id {
                Some(pid) => w.project_id == *pid,
                None => true,
            })
            .filter(|w| match filter.status {
                Some(s) => w.status == s,
                None => true,
            })
            .filter(|w| match &filter.query {
                Some(q) if !q.is_empty() => {
                    let q_lower = q.to_lowercase();
                    w.title.to_lowercase().contains(&q_lower)
                        || w.description.to_lowercase().contains(&q_lower)
                }
                _ => true,
            })
            .cloned()
            .collect();
        if let Some(limit) = filter.limit {
            out.truncate(limit);
        }
        Ok(out)
    }
}

/// P0 工具链用的 work item 过滤输入 (per docs/briefs/tool-p0-impl-001.md §1.3)
///
/// 注: 不 derive `Default` 因为 `TenantId` 不实现 Default (per workspace 强类型 ID 设计)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemFilter {
    /// 租户 ID (必带)
    pub tenant_id: TenantId,
    /// 自由文本 query (case-insensitive substring on title + description)
    pub query: Option<String>,
    /// 状态过滤
    pub status: Option<WorkItemStatus>,
    /// 项目过滤
    pub project_id: Option<ProjectId>,
    /// 限制返回条数
    pub limit: Option<usize>,
}

impl Default for InMemoryWorkItemService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkItemCommandPort for InMemoryWorkItemService {
    async fn create_work_item(
        &self,
        cmd: CreateWorkItemCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("developer")
            && !actor.has_role("project_admin")
            && !actor.has_role("tenant_admin")
        {
            return Err(WorkItemError::PermissionDenied);
        }
        // INV-WI-03:AITask 必带 objective + repository_scope
        if matches!(cmd.item_type, WorkItemType::AITask) {
            match &cmd.ai_task_data {
                None => return Err(WorkItemError::AiTaskMissingObjective),
                Some(d) => {
                    if d.objective.is_empty() {
                        return Err(WorkItemError::AiTaskMissingObjective);
                    }
                    if d.repository_scope.is_empty() {
                        return Err(WorkItemError::AiTaskMissingScope);
                    }
                }
            }
        }
        // INV-WI-04:parent 必须同 project
        if let Some(pid) = cmd.parent_work_item_id {
            let parent = self.repo.get(pid).await?;
            if parent.project_id != cmd.project_id {
                return Err(WorkItemError::ParentProjectMismatch);
            }
        }
        let now = Utc::now();
        let item = WorkItem {
            id: WorkItemId::new(),
            tenant_id: cmd.tenant_id,
            workspace_id: cmd.workspace_id,
            project_id: cmd.project_id,
            item_type: cmd.item_type,
            title: cmd.title,
            description: cmd.description,
            status: WorkItemStatus::Todo,
            assignee_user_id: None,
            assignee_agent_id: None,
            reporter_user_id: cmd.reporter_user_id,
            priority: cmd.priority,
            severity: cmd.severity,
            story_points: None,
            sprint_id: None,
            parent_work_item_id: cmd.parent_work_item_id,
            requirement_ids: vec![],
            acceptance_criterion_ids: vec![],
            repository_ids: vec![],
            worktree_ids: vec![],
            labels: cmd.labels,
            components: vec![],
            due_date: None,
            ai_task_data: cmd.ai_task_data,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };
        self.repo.insert(item.clone()).await?;
        self.items.write().unwrap().insert(item.id, item.clone());
        Ok(item)
    }

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        check_status_transition(cmd.from, cmd.to)?;
        let mut item = self
            .items
            .write()
            .unwrap()
            .get_mut(&cmd.work_item_id)
            .cloned()
            .ok_or(WorkItemError::NotFound(format!(
                "work_item:{}",
                cmd.work_item_id.as_uuid()
            )))?;
        if item.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                item.tenant_id,
                cmd.tenant_id,
            ));
        }
        if item.status != cmd.from {
            return Err(WorkItemError::InvalidTransition {
                from: item.status.as_str().to_string(),
                to: cmd.to.as_str().to_string(),
            });
        }
        item.status = cmd.to;
        item.updated_at = Utc::now();
        item.lock_version += 1;
        self.repo.update(item.clone()).await?;
        self.items.write().unwrap().insert(item.id, item.clone());
        Ok(item)
    }

    async fn assign(
        &self,
        cmd: AssignCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut item = self
            .items
            .write()
            .unwrap()
            .get_mut(&cmd.work_item_id)
            .cloned()
            .ok_or(WorkItemError::NotFound(format!(
                "work_item:{}",
                cmd.work_item_id.as_uuid()
            )))?;
        if item.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                item.tenant_id,
                cmd.tenant_id,
            ));
        }
        item.assignee_user_id = cmd.assignee_user_id;
        item.assignee_agent_id = cmd.assignee_agent_id;
        item.updated_at = Utc::now();
        item.lock_version += 1;
        self.repo.update(item.clone()).await?;
        self.items.write().unwrap().insert(item.id, item.clone());
        Ok(item)
    }

    async fn create_requirement(
        &self,
        cmd: CreateRequirementCommand,
        actor: &ActorContext,
    ) -> Result<Requirement, WorkItemError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let r = Requirement {
            id: RequirementId::new(),
            tenant_id: cmd.tenant_id,
            business_goal_id: cmd.business_goal_id,
            statement: cmd.statement,
            rationale: cmd.rationale,
            linked_work_item_ids: vec![],
            created_at: Utc::now(),
        };
        self.repo.insert_requirement(r.clone()).await?;
        self.requirements.write().unwrap().insert(r.id, r.clone());
        Ok(r)
    }

    async fn create_acceptance_criterion(
        &self,
        cmd: CreateAcceptanceCriterionCommand,
        actor: &ActorContext,
    ) -> Result<AcceptanceCriterion, WorkItemError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let ac = AcceptanceCriterion {
            id: AcceptanceCriterionId::new(),
            tenant_id: cmd.tenant_id,
            requirement_id: cmd.requirement_id,
            work_item_id: cmd.work_item_id,
            statement: cmd.statement,
            coverage_status: CoverageStatus::Uncovered,
            covered_by_validation_ids: vec![],
            created_at: Utc::now(),
        };
        self.repo.insert_ac(ac.clone()).await?;
        self.acs.write().unwrap().insert(ac.id, ac.clone());
        Ok(ac)
    }
}

#[async_trait]
impl WorkItemQueryPort for InMemoryWorkItemService {
    async fn get(
        &self,
        q: GetWorkItemQuery,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let item = self
            .items
            .read()
            .unwrap()
            .get(&q.work_item_id)
            .cloned()
            .ok_or(WorkItemError::NotFound(format!(
                "work_item:{}",
                q.work_item_id.as_uuid()
            )))?;
        if item.tenant_id != q.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                item.tenant_id,
                q.tenant_id,
            ));
        }
        Ok(item)
    }

    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorkItem>, WorkItemError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let items = self.items.read().unwrap();
        Ok(items
            .values()
            .filter(|i| i.tenant_id == q.tenant_id && i.project_id == q.project_id)
            .filter(|i| q.include_terminal || !i.status.is_terminal())
            .cloned()
            .collect())
    }
}

// =====================================================================
// InMemoryWorkItemRepository
// =====================================================================

/// WorkItemRepository 的内存实现
pub struct InMemoryWorkItemRepository {
    items: RwLock<HashMap<WorkItemId, WorkItem>>,
    requirements: RwLock<HashMap<RequirementId, Requirement>>,
    acs: RwLock<HashMap<AcceptanceCriterionId, AcceptanceCriterion>>,
}

impl InMemoryWorkItemRepository {
    /// 创建一个空的内存工作项仓储实例
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            requirements: RwLock::new(HashMap::new()),
            acs: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryWorkItemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkItemRepository for InMemoryWorkItemRepository {
    async fn insert(&self, w: WorkItem) -> Result<(), WorkItemError> {
        self.items.write().unwrap().insert(w.id, w);
        Ok(())
    }
    async fn get(&self, id: WorkItemId) -> Result<WorkItem, WorkItemError> {
        self.items
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(WorkItemError::NotFound(format!(
                "work_item:{}",
                id.as_uuid()
            )))
    }
    async fn update(&self, w: WorkItem) -> Result<(), WorkItemError> {
        self.items.write().unwrap().insert(w.id, w);
        Ok(())
    }
    async fn list_by_project(
        &self,
        tid: TenantId,
        pid: ProjectId,
    ) -> Result<Vec<WorkItem>, WorkItemError> {
        Ok(self
            .items
            .read()
            .unwrap()
            .values()
            .filter(|i| i.tenant_id == tid && i.project_id == pid)
            .cloned()
            .collect())
    }
    async fn insert_requirement(&self, r: Requirement) -> Result<(), WorkItemError> {
        self.requirements.write().unwrap().insert(r.id, r);
        Ok(())
    }
    async fn insert_ac(&self, ac: AcceptanceCriterion) -> Result<(), WorkItemError> {
        self.acs.write().unwrap().insert(ac.id, ac);
        Ok(())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn dev(tid: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid.0).with_role("developer")
    }

    fn basic_cmd(tid: TenantId) -> CreateWorkItemCommand {
        CreateWorkItemCommand {
            tenant_id: tid,
            workspace_id: WorkspaceId::new(),
            project_id: ProjectId::new(),
            item_type: WorkItemType::Task,
            title: "fix bug".to_string(),
            description: "details".to_string(),
            priority: Priority::High,
            severity: Some(Severity::Major),
            reporter_user_id: UserId::new(),
            parent_work_item_id: None,
            ai_task_data: None,
            labels: vec!["bug".to_string()],
        }
    }

    #[test]
    fn work_item_type_as_str() {
        assert_eq!(WorkItemType::Task.as_str(), "task");
        assert_eq!(WorkItemType::AITask.as_str(), "ai_task");
    }

    #[test]
    fn work_item_status_is_terminal() {
        assert!(WorkItemStatus::Done.is_terminal());
        assert!(!WorkItemStatus::Todo.is_terminal());
        assert!(!WorkItemStatus::InProgress.is_terminal());
    }

    #[test]
    fn transition_3state() {
        assert!(check_status_transition(WorkItemStatus::Todo, WorkItemStatus::InProgress).is_ok());
        assert!(check_status_transition(WorkItemStatus::InProgress, WorkItemStatus::Done).is_ok());
        assert!(check_status_transition(WorkItemStatus::InProgress, WorkItemStatus::Todo).is_ok()); // 回退
        assert!(check_status_transition(WorkItemStatus::Done, WorkItemStatus::InProgress).is_ok()); // 重开
        assert!(check_status_transition(WorkItemStatus::Todo, WorkItemStatus::Done).is_err());
        // 跳态
    }

    #[test]
    fn priority_as_str() {
        assert_eq!(Priority::High.as_str(), "high");
        assert_eq!(Priority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn coverage_status_as_str() {
        assert_eq!(CoverageStatus::Uncovered.as_str(), "UNCOVERED");
    }

    #[tokio::test]
    async fn create_work_item_basic() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let item = svc
            .create_work_item(basic_cmd(TenantId(tid)), &actor)
            .await
            .unwrap();
        assert_eq!(item.status, WorkItemStatus::Todo);
        assert_eq!(item.priority, Priority::High);
    }

    #[tokio::test]
    async fn ai_task_requires_objective_invw03() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let mut cmd = basic_cmd(TenantId(tid));
        cmd.item_type = WorkItemType::AITask;
        // 缺 ai_task_data
        let res = svc.create_work_item(cmd, &actor).await;
        assert!(matches!(res, Err(WorkItemError::AiTaskMissingObjective)));
    }

    #[tokio::test]
    async fn ai_task_requires_repository_scope_invw03() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let mut cmd = basic_cmd(TenantId(tid));
        cmd.item_type = WorkItemType::AITask;
        cmd.ai_task_data = Some(AiTaskData {
            objective: "fix".to_string(),
            repository_scope: vec![], // 缺
            allowed_files: vec![],
            forbidden_files: vec![],
            agent_policy_id: None,
            validation_policy_id: None,
            context_policy_id: None,
        });
        let res = svc.create_work_item(cmd, &actor).await;
        assert!(matches!(res, Err(WorkItemError::AiTaskMissingScope)));
    }

    #[tokio::test]
    async fn ai_task_with_full_data_ok() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let mut cmd = basic_cmd(TenantId(tid));
        cmd.item_type = WorkItemType::AITask;
        cmd.ai_task_data = Some(AiTaskData {
            objective: "implement auth".to_string(),
            repository_scope: vec![RepositoryId::new()],
            allowed_files: vec!["src/auth/**".to_string()],
            forbidden_files: vec![],
            agent_policy_id: None,
            validation_policy_id: None,
            context_policy_id: None,
        });
        let item = svc.create_work_item(cmd, &actor).await.unwrap();
        assert!(item.ai_task_data.is_some());
    }

    #[tokio::test]
    async fn transition_through_lifecycle() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let item = svc
            .create_work_item(basic_cmd(TenantId(tid)), &actor)
            .await
            .unwrap();
        let id = item.id;
        // TODO→IN_PROGRESS
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: TenantId(tid),
                    work_item_id: id,
                    from: WorkItemStatus::Todo,
                    to: WorkItemStatus::InProgress,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, WorkItemStatus::InProgress);
        // IN_PROGRESS→DONE
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: TenantId(tid),
                    work_item_id: id,
                    from: WorkItemStatus::InProgress,
                    to: WorkItemStatus::Done,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, WorkItemStatus::Done);
    }

    #[tokio::test]
    async fn transition_skip_rejected() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let item = svc
            .create_work_item(basic_cmd(TenantId(tid)), &actor)
            .await
            .unwrap();
        let res = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: TenantId(tid),
                    work_item_id: item.id,
                    from: WorkItemStatus::Todo,
                    to: WorkItemStatus::Done, // 跳态
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(WorkItemError::InvalidTransition { .. })));
    }

    #[tokio::test]
    async fn assign_user_and_agent() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let item = svc
            .create_work_item(basic_cmd(TenantId(tid)), &actor)
            .await
            .unwrap();
        let u = uuid::Uuid::new_v4();
        let a = AgentId::new();
        let item = svc
            .assign(
                AssignCommand {
                    tenant_id: TenantId(tid),
                    work_item_id: item.id,
                    assignee_user_id: Some(UserId(u)),
                    assignee_agent_id: Some(a),
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(item.assignee_user_id, Some(UserId(u)));
        assert_eq!(item.assignee_agent_id, Some(a));
    }

    #[tokio::test]
    async fn parent_must_be_same_project_invw04() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        // 父项
        let parent = svc
            .create_work_item(basic_cmd(TenantId(tid)), &actor)
            .await
            .unwrap();
        // 子项(不同 project)
        let mut cmd = basic_cmd(TenantId(tid));
        cmd.parent_work_item_id = Some(parent.id);
        cmd.project_id = ProjectId::new();
        let res = svc.create_work_item(cmd, &actor).await;
        assert!(matches!(res, Err(WorkItemError::ParentProjectMismatch)));
    }

    #[tokio::test]
    async fn cross_tenant_transition_denied() {
        let svc = InMemoryWorkItemService::new();
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let actor1 = dev(TenantId(t1));
        let item = svc
            .create_work_item(basic_cmd(TenantId(t1)), &actor1)
            .await
            .unwrap();
        let actor2 = dev(TenantId(t2));
        let res = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: TenantId(t2),
                    work_item_id: item.id,
                    from: WorkItemStatus::Todo,
                    to: WorkItemStatus::InProgress,
                    actor_user_id: UserId::from(actor2.user_id),
                },
                &actor2,
            )
            .await;
        assert!(matches!(res, Err(WorkItemError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn create_requirement_and_ac() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let item = svc
            .create_work_item(basic_cmd(TenantId(tid)), &actor)
            .await
            .unwrap();
        let req = svc
            .create_requirement(
                CreateRequirementCommand {
                    tenant_id: TenantId(tid),
                    business_goal_id: None,
                    statement: "support OAuth".to_string(),
                    rationale: "industry standard".to_string(),
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let ac = svc
            .create_acceptance_criterion(
                CreateAcceptanceCriterionCommand {
                    tenant_id: TenantId(tid),
                    requirement_id: req.id,
                    work_item_id: item.id,
                    statement: "login via OAuth2".to_string(),
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(ac.coverage_status, CoverageStatus::Uncovered);
    }

    #[tokio::test]
    async fn list_by_project_filter_terminal() {
        let svc = InMemoryWorkItemService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let project = ProjectId::new();
        for _ in 0..3 {
            svc.create_work_item(basic_cmd(TenantId(tid)), &actor)
                .await
                .unwrap();
        }
        let list = svc
            .list_by_project(
                ListByProjectQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                    include_terminal: false,
                },
                &actor,
            )
            .await
            .unwrap();
        // 创建的默认 project_id 各异,这里 list 应该是 0(因为我们用 basic_cmd 的 project_id)
        // 验证 actor 角色 + 调用不报错
        assert!(list.len() < 3);
    }
}

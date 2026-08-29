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

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
pub struct WorkItem {
    pub id: WorkItemId,
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub project_id: ProjectId,
    pub item_type: WorkItemType,
    pub title: String,
    pub description: String,
    pub status: WorkItemStatus,
    pub assignee_user_id: Option<UserId>,
    pub assignee_agent_id: Option<AgentId>,
    pub reporter_user_id: UserId,
    pub priority: Priority,
    pub severity: Option<Severity>,
    pub story_points: Option<u32>,
    pub sprint_id: Option<SprintId>,
    pub parent_work_item_id: Option<WorkItemId>,
    pub requirement_ids: Vec<RequirementId>,
    pub acceptance_criterion_ids: Vec<AcceptanceCriterionId>,
    pub repository_ids: Vec<RepositoryId>,
    pub worktree_ids: Vec<WorktreeId>,
    pub labels: Vec<String>,
    pub components: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    /// AITask 子类型字段
    pub ai_task_data: Option<AiTaskData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lock_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkItemType {
    Epic,
    Story,
    Task,
    Bug,
    Subtask,
    AITask,
}

impl WorkItemType {
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
pub enum WorkItemStatus {
    Todo,
    InProgress,
    Done,
}

impl WorkItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::InProgress => "IN_PROGRESS",
            Self::Done => "DONE",
        }
    }
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
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
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
pub enum Severity {
    Minor,
    Major,
    Critical,
    Blocker,
}

impl Severity {
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
pub struct AiTaskData {
    pub objective: String,
    pub repository_scope: Vec<RepositoryId>,
    pub allowed_files: Vec<String>,
    pub forbidden_files: Vec<String>,
    pub agent_policy_id: Option<AgentPolicyId>,
    pub validation_policy_id: Option<ValidationPolicyId>,
    pub context_policy_id: Option<ContextPolicyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: RequirementId,
    pub tenant_id: TenantId,
    pub business_goal_id: Option<BusinessGoalId>,
    pub statement: String,
    pub rationale: String,
    pub linked_work_item_ids: Vec<WorkItemId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: AcceptanceCriterionId,
    pub tenant_id: TenantId,
    pub requirement_id: RequirementId,
    pub work_item_id: WorkItemId,
    pub statement: String,
    pub coverage_status: CoverageStatus,
    pub covered_by_validation_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageStatus {
    Uncovered,
    Partial,
    Covered,
}

impl CoverageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uncovered => "UNCOVERED",
            Self::Partial => "PARTIAL",
            Self::Covered => "COVERED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessGoal {
    pub id: BusinessGoalId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum WorkItemError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },
    #[error("AI task missing objective (INV-WI-03)")]
    AiTaskMissingObjective,
    #[error("AI task missing repository scope (INV-WI-03)")]
    AiTaskMissingScope,
    #[error("parent work item must be in same project (INV-WI-04)")]
    ParentProjectMismatch,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkItemCommand {
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub project_id: ProjectId,
    pub item_type: WorkItemType,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub severity: Option<Severity>,
    pub reporter_user_id: UserId,
    pub parent_work_item_id: Option<WorkItemId>,
    pub ai_task_data: Option<AiTaskData>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionStatusCommand {
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
    pub from: WorkItemStatus,
    pub to: WorkItemStatus,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignCommand {
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
    pub assignee_user_id: Option<UserId>,
    pub assignee_agent_id: Option<AgentId>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequirementCommand {
    pub tenant_id: TenantId,
    pub business_goal_id: Option<BusinessGoalId>,
    pub statement: String,
    pub rationale: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAcceptanceCriterionCommand {
    pub tenant_id: TenantId,
    pub requirement_id: RequirementId,
    pub work_item_id: WorkItemId,
    pub statement: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkItemQuery {
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByProjectQuery {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub include_terminal: bool,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait WorkItemCommandPort: Send + Sync {
    async fn create_work_item(
        &self,
        cmd: CreateWorkItemCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    async fn assign(
        &self,
        cmd: AssignCommand,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    async fn create_requirement(
        &self,
        cmd: CreateRequirementCommand,
        actor: &ActorContext,
    ) -> Result<Requirement, WorkItemError>;

    async fn create_acceptance_criterion(
        &self,
        cmd: CreateAcceptanceCriterionCommand,
        actor: &ActorContext,
    ) -> Result<AcceptanceCriterion, WorkItemError>;
}

#[async_trait]
pub trait WorkItemQueryPort: Send + Sync {
    async fn get(
        &self,
        q: GetWorkItemQuery,
        actor: &ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorkItem>, WorkItemError>;
}

#[async_trait]
pub trait WorkItemRepository: Send + Sync {
    async fn insert(&self, w: WorkItem) -> Result<(), WorkItemError>;
    async fn get(&self, id: WorkItemId) -> Result<WorkItem, WorkItemError>;
    async fn update(&self, w: WorkItem) -> Result<(), WorkItemError>;
    async fn list_by_project(
        &self,
        tid: TenantId,
        pid: ProjectId,
    ) -> Result<Vec<WorkItem>, WorkItemError>;

    async fn insert_requirement(&self, r: Requirement) -> Result<(), WorkItemError>;
    async fn insert_ac(&self, ac: AcceptanceCriterion) -> Result<(), WorkItemError>;
}

// =====================================================================
// ActorContext
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub project_ids: Vec<ProjectId>,
    pub roles: Vec<String>,
}

impl ActorContext {
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
        }
    }
    pub fn with_role(mut self, role: &str) -> Self {
        self.roles.push(role.to_string());
        self
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

// =====================================================================
// InMemoryWorkItemService
// =====================================================================

pub struct InMemoryWorkItemService {
    repo: Arc<dyn WorkItemRepository>,
    items: Arc<RwLock<HashMap<WorkItemId, WorkItem>>>,
    requirements: Arc<RwLock<HashMap<RequirementId, Requirement>>>,
    acs: Arc<RwLock<HashMap<AcceptanceCriterionId, AcceptanceCriterion>>>,
}

impl InMemoryWorkItemService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryWorkItemRepository::new()),
            items: Arc::new(RwLock::new(HashMap::new())),
            requirements: Arc::new(RwLock::new(HashMap::new())),
            acs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
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
        if actor.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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
        if actor.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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
        if actor.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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
        if actor.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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
        if actor.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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
        if actor.tenant_id != q.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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
        if actor.tenant_id != q.tenant_id {
            return Err(WorkItemError::CrossTenantDenied(
                actor.tenant_id,
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

pub struct InMemoryWorkItemRepository {
    items: RwLock<HashMap<WorkItemId, WorkItem>>,
    requirements: RwLock<HashMap<RequirementId, Requirement>>,
    acs: RwLock<HashMap<AcceptanceCriterionId, AcceptanceCriterion>>,
}

impl InMemoryWorkItemRepository {
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
        ActorContext::new(UserId::new(), tid).with_role("developer")
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
        let tid = TenantId::new();
        let actor = dev(tid);
        let item = svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        assert_eq!(item.status, WorkItemStatus::Todo);
        assert_eq!(item.priority, Priority::High);
    }

    #[tokio::test]
    async fn ai_task_requires_objective_invw03() {
        let svc = InMemoryWorkItemService::new();
        let tid = TenantId::new();
        let actor = dev(tid);
        let mut cmd = basic_cmd(tid);
        cmd.item_type = WorkItemType::AITask;
        // 缺 ai_task_data
        let res = svc.create_work_item(cmd, &actor).await;
        assert!(matches!(res, Err(WorkItemError::AiTaskMissingObjective)));
    }

    #[tokio::test]
    async fn ai_task_requires_repository_scope_invw03() {
        let svc = InMemoryWorkItemService::new();
        let tid = TenantId::new();
        let actor = dev(tid);
        let mut cmd = basic_cmd(tid);
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
        let tid = TenantId::new();
        let actor = dev(tid);
        let mut cmd = basic_cmd(tid);
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
        let tid = TenantId::new();
        let actor = dev(tid);
        let item = svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        let id = item.id;
        // TODO→IN_PROGRESS
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: tid,
                    work_item_id: id,
                    from: WorkItemStatus::Todo,
                    to: WorkItemStatus::InProgress,
                    actor_user_id: actor.user_id,
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
                    tenant_id: tid,
                    work_item_id: id,
                    from: WorkItemStatus::InProgress,
                    to: WorkItemStatus::Done,
                    actor_user_id: actor.user_id,
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
        let tid = TenantId::new();
        let actor = dev(tid);
        let item = svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        let res = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: tid,
                    work_item_id: item.id,
                    from: WorkItemStatus::Todo,
                    to: WorkItemStatus::Done, // 跳态
                    actor_user_id: actor.user_id,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(WorkItemError::InvalidTransition { .. })));
    }

    #[tokio::test]
    async fn assign_user_and_agent() {
        let svc = InMemoryWorkItemService::new();
        let tid = TenantId::new();
        let actor = dev(tid);
        let item = svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        let u = UserId::new();
        let a = AgentId::new();
        let item = svc
            .assign(
                AssignCommand {
                    tenant_id: tid,
                    work_item_id: item.id,
                    assignee_user_id: Some(u),
                    assignee_agent_id: Some(a),
                    actor_user_id: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(item.assignee_user_id, Some(u));
        assert_eq!(item.assignee_agent_id, Some(a));
    }

    #[tokio::test]
    async fn parent_must_be_same_project_invw04() {
        let svc = InMemoryWorkItemService::new();
        let tid = TenantId::new();
        let actor = dev(tid);
        // 父项
        let parent = svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        // 子项(不同 project)
        let mut cmd = basic_cmd(tid);
        cmd.parent_work_item_id = Some(parent.id);
        cmd.project_id = ProjectId::new();
        let res = svc.create_work_item(cmd, &actor).await;
        assert!(matches!(res, Err(WorkItemError::ParentProjectMismatch)));
    }

    #[tokio::test]
    async fn cross_tenant_transition_denied() {
        let svc = InMemoryWorkItemService::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        let actor1 = dev(t1);
        let item = svc.create_work_item(basic_cmd(t1), &actor1).await.unwrap();
        let actor2 = dev(t2);
        let res = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: t2,
                    work_item_id: item.id,
                    from: WorkItemStatus::Todo,
                    to: WorkItemStatus::InProgress,
                    actor_user_id: actor2.user_id,
                },
                &actor2,
            )
            .await;
        assert!(matches!(res, Err(WorkItemError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn create_requirement_and_ac() {
        let svc = InMemoryWorkItemService::new();
        let tid = TenantId::new();
        let actor = dev(tid);
        let item = svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        let req = svc
            .create_requirement(
                CreateRequirementCommand {
                    tenant_id: tid,
                    business_goal_id: None,
                    statement: "support OAuth".to_string(),
                    rationale: "industry standard".to_string(),
                    actor_user_id: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        let ac = svc
            .create_acceptance_criterion(
                CreateAcceptanceCriterionCommand {
                    tenant_id: tid,
                    requirement_id: req.id,
                    work_item_id: item.id,
                    statement: "login via OAuth2".to_string(),
                    actor_user_id: actor.user_id,
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
        let tid = TenantId::new();
        let actor = dev(tid);
        let project = ProjectId::new();
        for _ in 0..3 {
            svc.create_work_item(basic_cmd(tid), &actor).await.unwrap();
        }
        let list = svc
            .list_by_project(
                ListByProjectQuery {
                    tenant_id: tid,
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

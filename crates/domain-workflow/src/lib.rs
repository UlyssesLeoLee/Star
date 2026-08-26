//! domain-workflow crate
//!
//! 详细 spec: docs/specs/domain-workflow-spec.md
//! 上游基本设计: docs/basic-design.md §2.1(表 18) / §4.9.3 / §7.6
//! 数据设计: docs/data-design.md §4.5 (`workflow` schema)
//! API 设计: docs/api-design.md §3.6 (Workflow Definition + Transition)
//!
//! ## 职责
//!
//! Workflow 聚合根 + WorkflowState + Transition + WorkflowInstance:
//! - 强类型 ID(`WorkflowId` / `WorkflowStateId` / `TransitionId` / `WorkflowInstanceId` /
//!   `TenantId` / `UserId` / `WorkItemId`)
//! - 4 个核心实体(`Workflow` / `WorkflowState` / `Transition` / `WorkflowInstance`)
//! - 2 个端口(`WorkflowCommandPort` × 3 / `WorkflowQueryPort` × 2) + 1 个仓库端口
//! - 5 条不变量检查(INV-WF-01~05)
//! - 1 个 `InMemoryWorkflowService` 真实实现
//!
//! ## 关键不变量
//!
//! - INV-WF-01:Workflow 必带 tenant_id
//! - INV-WF-02:Workflow 必有 default_initial_state
//! - INV-WF-03:严格转换表(不在表 → 拒绝)
//! - INV-WF-04:Terminal 状态不可再迁出
//! - INV-WF-05:WorkflowInstance history 必带 actor + at(审计)

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
// ID 类型 + define_uuid_id 宏
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

define_uuid_id!(WorkflowId);
define_uuid_id!(WorkflowStateId);
define_uuid_id!(TransitionId);
define_uuid_id!(WorkflowInstanceId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);
define_uuid_id!(ProjectId);

// =====================================================================
// 值对象
// =====================================================================

/// 状态类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateCategory {
    /// 初始态(Workflow 起始)
    Initial,
    /// 中间态
    Intermediate,
    /// 终态(不可迁出,INV-WF-04)
    Terminal,
}

impl StateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "INITIAL",
            Self::Intermediate => "INTERMEDIATE",
            Self::Terminal => "TERMINAL",
        }
    }
}

/// 触发器来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionTrigger {
    /// 用户主动操作
    UserAction,
    /// Agent 自动操作
    AgentAction,
    /// 系统事件
    SystemEvent,
    /// 时间触发
    TimeElapsed,
}

impl TransitionTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserAction => "USER_ACTION",
            Self::AgentAction => "AGENT_ACTION",
            Self::SystemEvent => "SYSTEM_EVENT",
            Self::TimeElapsed => "TIME_ELAPSED",
        }
    }
}

/// 迁移守卫(权限 / 验证 / 审批)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Guard {
    /// 需要指定角色
    RequireRole(String),
    /// 需要执行验证(由调用方在 transition 时附带 result)
    RequireValidation(String),
    /// 需要人工审批
    RequireApproval,
}

impl Guard {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RequireRole(_) => "REQUIRE_ROLE",
            Self::RequireValidation(_) => "REQUIRE_VALIDATION",
            Self::RequireApproval => "REQUIRE_APPROVAL",
        }
    }
}

// =====================================================================
// 实体
// =====================================================================

/// **Workflow 聚合根**(data-design §4.5,简化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// 主键
    pub id: WorkflowId,
    /// 租户 ID(INV-WF-01:必带)
    pub tenant_id: TenantId,
    /// 名称
    pub name: String,
    /// 状态集合
    pub states: Vec<WorkflowState>,
    /// 转换表
    pub transitions: Vec<Transition>,
    /// 默认初始状态 ID(INV-WF-02:必带)
    pub default_initial_state: WorkflowStateId,
    /// 版本号(整体替换时 +1)
    pub version: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// **WorkflowState**(Workflow 的状态定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// 主键
    pub id: WorkflowStateId,
    /// 状态名(TODO / IN_PROGRESS / REVIEW / DONE / BLOCKED)
    pub name: String,
    /// 类别
    pub category: StateCategory,
    /// 是否初始
    pub is_initial: bool,
    /// 是否终态(INV-WF-04)
    pub is_terminal: bool,
}

/// **Transition**(状态转换定义,严格表查找)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// 主键
    pub id: TransitionId,
    /// 源状态
    pub from: WorkflowStateId,
    /// 目标状态
    pub to: WorkflowStateId,
    /// 触发器
    pub trigger: TransitionTrigger,
    /// 可选守卫
    pub guard: Option<Guard>,
}

/// **WorkflowInstance**(运行实例,绑定 work_item 跟踪状态机执行)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// 主键
    pub id: WorkflowInstanceId,
    /// 关联 Workflow
    pub workflow_id: WorkflowId,
    /// 关联 WorkItem
    pub work_item_id: WorkItemId,
    /// 当前状态
    pub current_state: WorkflowStateId,
    /// 状态变更历史(INV-WF-05:必带 actor + at)
    pub history: Vec<StateChange>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// **StateChange**(单次状态变更记录,审计必带)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// 源状态
    pub from: WorkflowStateId,
    /// 目标状态
    pub to: WorkflowStateId,
    /// 变更时间
    pub at: DateTime<Utc>,
    /// 触发者
    pub actor: UserId,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },
    #[error("missing initial state")]
    MissingInitial,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("permission denied: requires {0}")]
    PermissionDenied(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub states: Vec<WorkflowState>,
    pub transitions: Vec<Transition>,
    pub default_initial_state: WorkflowStateId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartInstanceCommand {
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub work_item_id: WorkItemId,
    pub actor: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionCommand {
    pub tenant_id: TenantId,
    pub instance_id: WorkflowInstanceId,
    pub to: WorkflowStateId,
    pub actor: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByTenantQuery {
    pub tenant_id: TenantId,
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
    pub is_local_runtime: bool,
}

impl ActorContext {
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
            is_local_runtime: false,
        }
    }
    pub fn with_role(mut self, role: &str) -> Self {
        self.roles.push(role.to_string());
        self
    }
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_ids.push(project_id);
        self
    }
    pub fn as_local_runtime(mut self) -> Self {
        self.is_local_runtime = true;
        self
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait WorkflowCommandPort: Send + Sync {
    async fn create_workflow(
        &self,
        cmd: CreateWorkflowCommand,
        actor: &ActorContext,
    ) -> Result<Workflow, WorkflowError>;

    async fn start_instance(
        &self,
        cmd: StartInstanceCommand,
        actor: &ActorContext,
    ) -> Result<WorkflowInstance, WorkflowError>;

    async fn transition(
        &self,
        cmd: TransitionCommand,
        actor: &ActorContext,
    ) -> Result<WorkflowInstance, WorkflowError>;
}

#[async_trait]
pub trait WorkflowQueryPort: Send + Sync {
    async fn get(
        &self,
        id: WorkflowId,
        actor: &ActorContext,
    ) -> Result<Workflow, WorkflowError>;

    async fn list_by_tenant(
        &self,
        q: ListByTenantQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Workflow>, WorkflowError>;
}

#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn insert(&self, wf: Workflow) -> Result<(), WorkflowError>;
    async fn get(&self, id: WorkflowId) -> Result<Workflow, WorkflowError>;
    async fn update(&self, wf: Workflow) -> Result<(), WorkflowError>;
    async fn list_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<Workflow>, WorkflowError>;
    async fn insert_instance(
        &self,
        inst: WorkflowInstance,
    ) -> Result<(), WorkflowError>;
    async fn get_instance(
        &self,
        id: WorkflowInstanceId,
    ) -> Result<WorkflowInstance, WorkflowError>;
    async fn update_instance(
        &self,
        inst: WorkflowInstance,
    ) -> Result<(), WorkflowError>;
}

// =====================================================================
// 核心函数:转换表查找(INV-WF-03, INV-WF-04)
// =====================================================================

/// 严格表查找 — 不在表 → InvalidTransition
/// 终态不可再迁出(INV-WF-04)
pub fn check_transition(
    workflow: &Workflow,
    from: WorkflowStateId,
    to: WorkflowStateId,
) -> Result<(), WorkflowError> {
    // 终态不可再迁(INV-WF-04)
    if let Some(state) = workflow.states.iter().find(|s| s.id == from) {
        if state.is_terminal {
            return Err(WorkflowError::InvalidTransition {
                from: state.name.clone(),
                to: to.to_string(),
            });
        }
    } else {
        return Err(WorkflowError::InvalidTransition {
            from: from.to_string(),
            to: to.to_string(),
        });
    }
    // 严格表查找
    let allowed = workflow.transitions.iter().any(|t| t.from == from && t.to == to);
    if !allowed {
        let from_name = workflow
            .states
            .iter()
            .find(|s| s.id == from)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| from.to_string());
        let to_name = workflow
            .states
            .iter()
            .find(|s| s.id == to)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| to.to_string());
        return Err(WorkflowError::InvalidTransition {
            from: from_name,
            to: to_name,
        });
    }
    Ok(())
}

// =====================================================================
// 不变量检查(INV-WF-01~05)
// =====================================================================

/// INV-WF-01:Workflow 必带 tenant_id
pub fn check_invariant_01_tenant(wf: &Workflow) -> Result<(), WorkflowError> {
    if wf.tenant_id.as_uuid().is_nil() {
        return Err(WorkflowError::Internal(
            "INV-WF-01: tenant_id 必带".to_string(),
        ));
    }
    Ok(())
}

/// INV-WF-02:Workflow 必有 default_initial_state
pub fn check_invariant_02_initial_state(wf: &Workflow) -> Result<(), WorkflowError> {
    if wf.states.is_empty() {
        return Err(WorkflowError::MissingInitial);
    }
    if !wf.states.iter().any(|s| s.id == wf.default_initial_state) {
        return Err(WorkflowError::MissingInitial);
    }
    Ok(())
}

/// INV-WF-05:WorkflowInstance history 必带 actor + at
pub fn check_invariant_05_history_audit(
    inst: &WorkflowInstance,
) -> Result<(), WorkflowError> {
    for change in &inst.history {
        if change.actor.as_uuid().is_nil() {
            return Err(WorkflowError::Internal(
                "INV-WF-05: history actor 必带".to_string(),
            ));
        }
        if change.at.timestamp() == 0 {
            return Err(WorkflowError::Internal(
                "INV-WF-05: history at 必带".to_string(),
            ));
        }
    }
    Ok(())
}

/// 跑全部不变量
pub fn run_invariants(wf: &Workflow, inst: Option<&WorkflowInstance>) -> Result<(), WorkflowError> {
    check_invariant_01_tenant(wf)?;
    check_invariant_02_initial_state(wf)?;
    if let Some(i) = inst {
        check_invariant_05_history_audit(i)?;
    }
    Ok(())
}

// =====================================================================
// InMemoryWorkflowService
// =====================================================================

pub struct InMemoryWorkflowService {
    repo: Arc<dyn WorkflowRepository>,
    workflows: Arc<RwLock<HashMap<WorkflowId, Workflow>>>,
    instances: Arc<RwLock<HashMap<WorkflowInstanceId, WorkflowInstance>>>,
}

impl InMemoryWorkflowService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryWorkflowRepository::new()),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_repo(repo: Arc<dyn WorkflowRepository>) -> Self {
        Self {
            repo,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn ensure_cross_tenant(
        &self,
        actor: &ActorContext,
        required: TenantId,
    ) -> Result<(), WorkflowError> {
        if actor.tenant_id != required {
            return Err(WorkflowError::CrossTenantDenied(actor.tenant_id, required));
        }
        Ok(())
    }
}

impl Default for InMemoryWorkflowService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowCommandPort for InMemoryWorkflowService {
    async fn create_workflow(
        &self,
        cmd: CreateWorkflowCommand,
        actor: &ActorContext,
    ) -> Result<Workflow, WorkflowError> {
        self.ensure_cross_tenant(actor, cmd.tenant_id)?;
        let now = Utc::now();
        let wf = Workflow {
            id: WorkflowId::new(),
            tenant_id: cmd.tenant_id,
            name: cmd.name,
            states: cmd.states,
            transitions: cmd.transitions,
            default_initial_state: cmd.default_initial_state,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        run_invariants(&wf, None)?;
        self.repo.insert(wf.clone()).await?;
        self.workflows.write().unwrap().insert(wf.id, wf.clone());
        Ok(wf)
    }

    async fn start_instance(
        &self,
        cmd: StartInstanceCommand,
        actor: &ActorContext,
    ) -> Result<WorkflowInstance, WorkflowError> {
        self.ensure_cross_tenant(actor, cmd.tenant_id)?;
        let wf = self.repo.get(cmd.workflow_id).await?;
        if wf.tenant_id != cmd.tenant_id {
            return Err(WorkflowError::CrossTenantDenied(actor.tenant_id, wf.tenant_id));
        }
        // 校验 default_initial_state 存在
        if !wf.states.iter().any(|s| s.id == wf.default_initial_state) {
            return Err(WorkflowError::MissingInitial);
        }
        let inst = WorkflowInstance {
            id: WorkflowInstanceId::new(),
            workflow_id: wf.id,
            work_item_id: cmd.work_item_id,
            current_state: wf.default_initial_state,
            history: vec![],
            created_at: Utc::now(),
        };
        run_invariants(&wf, Some(&inst))?;
        self.repo.insert_instance(inst.clone()).await?;
        self.instances.write().unwrap().insert(inst.id, inst.clone());
        Ok(inst)
    }

    async fn transition(
        &self,
        cmd: TransitionCommand,
        actor: &ActorContext,
    ) -> Result<WorkflowInstance, WorkflowError> {
        self.ensure_cross_tenant(actor, cmd.tenant_id)?;
        let mut inst = self.repo.get_instance(cmd.instance_id).await?;
        let wf = self.repo.get(inst.workflow_id).await?;
        if wf.tenant_id != cmd.tenant_id {
            return Err(WorkflowError::CrossTenantDenied(actor.tenant_id, wf.tenant_id));
        }
        // 校验转换表 + 终态保护
        check_transition(&wf, inst.current_state, cmd.to)?;
        // 校验守卫
        if let Some(t) = wf
            .transitions
            .iter()
            .find(|t| t.from == inst.current_state && t.to == cmd.to)
        {
            if let Some(guard) = &t.guard {
                match guard {
                    Guard::RequireRole(role) => {
                        if !actor.has_role(role) {
                            return Err(WorkflowError::PermissionDenied(format!(
                                "需要角色: {}",
                                role
                            )));
                        }
                    }
                    Guard::RequireApproval => {
                        // 简化:仅 project_admin 可 Approve
                        if !actor.has_role("project_admin") {
                            return Err(WorkflowError::PermissionDenied(
                                "需要 project_admin 审批".to_string(),
                            ));
                        }
                    }
                    Guard::RequireValidation(_) => {
                        // 简化:不强制验证
                    }
                }
            }
        }
        // 写 history(INV-WF-05)
        inst.history.push(StateChange {
            from: inst.current_state,
            to: cmd.to,
            at: Utc::now(),
            actor: actor.user_id,
        });
        inst.current_state = cmd.to;
        run_invariants(&wf, Some(&inst))?;
        self.repo.update_instance(inst.clone()).await?;
        self.instances.write().unwrap().insert(inst.id, inst.clone());
        Ok(inst)
    }
}

#[async_trait]
impl WorkflowQueryPort for InMemoryWorkflowService {
    async fn get(
        &self,
        id: WorkflowId,
        actor: &ActorContext,
    ) -> Result<Workflow, WorkflowError> {
        let wf = self.repo.get(id).await?;
        self.ensure_cross_tenant(actor, wf.tenant_id)?;
        Ok(wf)
    }

    async fn list_by_tenant(
        &self,
        q: ListByTenantQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Workflow>, WorkflowError> {
        self.ensure_cross_tenant(actor, q.tenant_id)?;
        self.repo.list_by_tenant(q.tenant_id).await
    }
}

// =====================================================================
// InMemoryWorkflowRepository
// =====================================================================

pub struct InMemoryWorkflowRepository {
    workflows: RwLock<HashMap<WorkflowId, Workflow>>,
    instances: RwLock<HashMap<WorkflowInstanceId, WorkflowInstance>>,
}

impl InMemoryWorkflowRepository {
    pub fn new() -> Self {
        Self {
            workflows: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryWorkflowRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowRepository for InMemoryWorkflowRepository {
    async fn insert(&self, wf: Workflow) -> Result<(), WorkflowError> {
        let mut s = self.workflows.write().unwrap();
        if s.contains_key(&wf.id) {
            return Err(WorkflowError::Conflict(format!("Workflow {} 已存在", wf.id)));
        }
        s.insert(wf.id, wf);
        Ok(())
    }
    async fn get(&self, id: WorkflowId) -> Result<Workflow, WorkflowError> {
        self.workflows
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(WorkflowError::NotFound(format!("workflow:{}", id.as_uuid())))
    }
    async fn update(&self, wf: Workflow) -> Result<(), WorkflowError> {
        self.workflows.write().unwrap().insert(wf.id, wf);
        Ok(())
    }
    async fn list_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<Workflow>, WorkflowError> {
        Ok(self
            .workflows
            .read()
            .unwrap()
            .values()
            .filter(|w| w.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
    async fn insert_instance(
        &self,
        inst: WorkflowInstance,
    ) -> Result<(), WorkflowError> {
        let mut s = self.instances.write().unwrap();
        if s.contains_key(&inst.id) {
            return Err(WorkflowError::Conflict(format!(
                "WorkflowInstance {} 已存在",
                inst.id
            )));
        }
        s.insert(inst.id, inst);
        Ok(())
    }
    async fn get_instance(
        &self,
        id: WorkflowInstanceId,
    ) -> Result<WorkflowInstance, WorkflowError> {
        self.instances
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(WorkflowError::NotFound(format!("instance:{}", id.as_uuid())))
    }
    async fn update_instance(
        &self,
        inst: WorkflowInstance,
    ) -> Result<(), WorkflowError> {
        self.instances.write().unwrap().insert(inst.id, inst);
        Ok(())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------- 工具夹具 --------

    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
    }

    /// 构造一个简单三态 TODO → IN_PROGRESS → DONE
    fn make_three_state_workflow(tenant_id: TenantId) -> CreateWorkflowCommand {
        let s_todo = WorkflowStateId::new();
        let s_in_progress = WorkflowStateId::new();
        let s_done = WorkflowStateId::new();
        let t1 = TransitionId::new();
        let t2 = TransitionId::new();
        CreateWorkflowCommand {
            tenant_id,
            name: "Three State".to_string(),
            states: vec![
                WorkflowState {
                    id: s_todo,
                    name: "TODO".to_string(),
                    category: StateCategory::Initial,
                    is_initial: true,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_in_progress,
                    name: "IN_PROGRESS".to_string(),
                    category: StateCategory::Intermediate,
                    is_initial: false,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_done,
                    name: "DONE".to_string(),
                    category: StateCategory::Terminal,
                    is_initial: false,
                    is_terminal: true,
                },
            ],
            transitions: vec![
                Transition {
                    id: t1,
                    from: s_todo,
                    to: s_in_progress,
                    trigger: TransitionTrigger::UserAction,
                    guard: None,
                },
                Transition {
                    id: t2,
                    from: s_in_progress,
                    to: s_done,
                    trigger: TransitionTrigger::UserAction,
                    guard: None,
                },
            ],
            default_initial_state: s_todo,
        }
    }

    // -------- 1. 简单三态工作流 TODO→IN_PROGRESS→DONE --------

    #[tokio::test]
    async fn three_state_workflow_full_transition() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.expect("创建成功");
        assert_eq!(wf.version, 1);

        // start_instance
        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .expect("start_instance");
        assert_eq!(inst.current_state, wf.default_initial_state);
        assert!(inst.history.is_empty());

        // TODO → IN_PROGRESS
        let todo_id = wf
            .states
            .iter()
            .find(|s| s.name == "TODO")
            .unwrap()
            .id;
        let in_progress = wf
            .states
            .iter()
            .find(|s| s.name == "IN_PROGRESS")
            .unwrap()
            .id;
        let inst2 = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: in_progress,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .expect("transition");
        assert_eq!(inst2.current_state, in_progress);
        assert_eq!(inst2.history.len(), 1);

        // IN_PROGRESS → DONE
        let done = wf.states.iter().find(|s| s.name == "DONE").unwrap().id;
        let inst3 = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst2.id,
                    to: done,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .expect("transition");
        assert_eq!(inst3.current_state, done);
        assert_eq!(inst3.history.len(), 2);
    }

    // -------- 2. 跳态拒绝 --------

    #[tokio::test]
    async fn skip_state_rejected() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();

        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        // TODO → DONE 没有 transition
        let todo_id = wf.states.iter().find(|s| s.name == "TODO").unwrap().id;
        let done = wf.states.iter().find(|s| s.name == "DONE").unwrap().id;
        let res = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: done,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(WorkflowError::InvalidTransition { .. })));
        // 错误信息要包含 TODO 和 DONE
        let err = res.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("TODO"));
        assert!(msg.contains("DONE"));
    }

    // -------- 3. 终态不可再迁 --------

    #[tokio::test]
    async fn terminal_state_cannot_transition() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();

        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        let in_progress = wf
            .states
            .iter()
            .find(|s| s.name == "IN_PROGRESS")
            .unwrap()
            .id;
        let done = wf.states.iter().find(|s| s.name == "DONE").unwrap().id;
        let todo_id = wf.states.iter().find(|s| s.name == "TODO").unwrap().id;

        // 先到 DONE
        svc.transition(
            TransitionCommand {
                tenant_id,
                instance_id: inst.id,
                to: in_progress,
                actor: actor.user_id,
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition(
            TransitionCommand {
                tenant_id,
                instance_id: inst.id,
                to: done,
                actor: actor.user_id,
            },
            &actor,
        )
        .await
        .unwrap();

        // DONE → TODO 必被拒(终态保护 INV-WF-04)
        let res = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: todo_id,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(WorkflowError::InvalidTransition { .. })));
    }

    // -------- 4. 无效转换拒绝(同状态 from==to) --------

    #[tokio::test]
    async fn invalid_transition_rejected() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();

        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        // TODO → TODO,不在表
        let todo_id = wf.states.iter().find(|s| s.name == "TODO").unwrap().id;
        let res = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: todo_id,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(WorkflowError::InvalidTransition { .. })));
    }

    // -------- 5. 多状态工作流 --------

    #[tokio::test]
    async fn multi_state_workflow_with_review_and_blocked() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let s_todo = WorkflowStateId::new();
        let s_wip = WorkflowStateId::new();
        let s_review = WorkflowStateId::new();
        let s_done = WorkflowStateId::new();
        let s_blocked = WorkflowStateId::new();
        let cmd = CreateWorkflowCommand {
            tenant_id,
            name: "Multi State".to_string(),
            states: vec![
                WorkflowState {
                    id: s_todo,
                    name: "TODO".to_string(),
                    category: StateCategory::Initial,
                    is_initial: true,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_wip,
                    name: "WIP".to_string(),
                    category: StateCategory::Intermediate,
                    is_initial: false,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_review,
                    name: "REVIEW".to_string(),
                    category: StateCategory::Intermediate,
                    is_initial: false,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_blocked,
                    name: "BLOCKED".to_string(),
                    category: StateCategory::Intermediate,
                    is_initial: false,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_done,
                    name: "DONE".to_string(),
                    category: StateCategory::Terminal,
                    is_initial: false,
                    is_terminal: true,
                },
            ],
            transitions: vec![
                Transition {
                    id: TransitionId::new(),
                    from: s_todo,
                    to: s_wip,
                    trigger: TransitionTrigger::UserAction,
                    guard: None,
                },
                Transition {
                    id: TransitionId::new(),
                    from: s_wip,
                    to: s_review,
                    trigger: TransitionTrigger::UserAction,
                    guard: None,
                },
                Transition {
                    id: TransitionId::new(),
                    from: s_wip,
                    to: s_blocked,
                    trigger: TransitionTrigger::SystemEvent,
                    guard: None,
                },
                Transition {
                    id: TransitionId::new(),
                    from: s_blocked,
                    to: s_wip,
                    trigger: TransitionTrigger::SystemEvent,
                    guard: None,
                },
                Transition {
                    id: TransitionId::new(),
                    from: s_review,
                    to: s_done,
                    trigger: TransitionTrigger::UserAction,
                    guard: None,
                },
            ],
            default_initial_state: s_todo,
        };
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();
        assert_eq!(wf.states.len(), 5);
        assert_eq!(wf.transitions.len(), 5);

        // 走完: TODO → WIP → BLOCKED → WIP → REVIEW → DONE
        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        svc.transition(
            TransitionCommand {
                tenant_id,
                instance_id: inst.id,
                to: s_wip,
                actor: actor.user_id,
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition(
            TransitionCommand {
                tenant_id,
                instance_id: inst.id,
                to: s_blocked,
                actor: actor.user_id,
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition(
            TransitionCommand {
                tenant_id,
                instance_id: inst.id,
                to: s_wip,
                actor: actor.user_id,
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition(
            TransitionCommand {
                tenant_id,
                instance_id: inst.id,
                to: s_review,
                actor: actor.user_id,
            },
            &actor,
        )
        .await
        .unwrap();
        let inst_done = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: s_done,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(inst_done.current_state, s_done);
        assert_eq!(inst_done.history.len(), 5);
    }

    // -------- 6. start_instance 默认状态 --------

    #[tokio::test]
    async fn start_instance_uses_default_initial_state() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();
        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(inst.current_state, wf.default_initial_state);
        let todo_id = wf.states.iter().find(|s| s.name == "TODO").unwrap().id;
        assert_eq!(inst.current_state, todo_id);
    }

    // -------- 7. cross_tenant denied --------

    #[tokio::test]
    async fn cross_tenant_denied() {
        let svc = InMemoryWorkflowService::new();
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let actor_a = make_actor(tenant_a);
        let cmd = make_three_state_workflow(tenant_a);
        let wf = svc.create_workflow(cmd, &actor_a).await.unwrap();

        // 跨 tenant 查询
        let actor_b = make_actor(tenant_b);
        let res = svc.get(wf.id, &actor_b).await;
        assert!(matches!(res, Err(WorkflowError::CrossTenantDenied(_, _))));

        // 跨 tenant 启动实例
        let res2 = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id: tenant_b,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor_b.user_id,
                },
                &actor_b,
            )
            .await;
        assert!(matches!(res2, Err(WorkflowError::CrossTenantDenied(_, _))));
    }

    // -------- 8. workflow version increment --------

    #[tokio::test]
    async fn workflow_version_increment() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf1 = svc.create_workflow(cmd, &actor).await.unwrap();
        assert_eq!(wf1.version, 1);
        // 整体替换 → version +1
        let mut wf2 = wf1.clone();
        wf2.version += 1;
        wf2.updated_at = Utc::now();
        svc.repo.update(wf2.clone()).await.unwrap();
        let fetched = svc.get(wf2.id, &actor).await.unwrap();
        assert_eq!(fetched.version, 2);
    }

    // -------- 9. history 记录 --------

    #[tokio::test]
    async fn history_records_actor_and_at() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();
        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        let in_progress = wf
            .states
            .iter()
            .find(|s| s.name == "IN_PROGRESS")
            .unwrap()
            .id;
        let inst2 = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: in_progress,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        // INV-WF-05: history 必带 actor + at
        assert_eq!(inst2.history.len(), 1);
        let change = &inst2.history[0];
        assert_eq!(change.actor, actor.user_id);
        assert_ne!(change.at.timestamp(), 0);
        assert!(check_invariant_05_history_audit(&inst2).is_ok());
    }

    // -------- 10. 角色 guard(developer 不能 Approve) --------

    #[tokio::test]
    async fn role_guard_blocks_non_approver() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor_developer = make_actor(tenant_id);
        // 不带 project_admin 角色
        let s_todo = WorkflowStateId::new();
        let s_done = WorkflowStateId::new();
        let cmd = CreateWorkflowCommand {
            tenant_id,
            name: "Approval".to_string(),
            states: vec![
                WorkflowState {
                    id: s_todo,
                    name: "TODO".to_string(),
                    category: StateCategory::Initial,
                    is_initial: true,
                    is_terminal: false,
                },
                WorkflowState {
                    id: s_done,
                    name: "DONE".to_string(),
                    category: StateCategory::Terminal,
                    is_initial: false,
                    is_terminal: true,
                },
            ],
            transitions: vec![Transition {
                id: TransitionId::new(),
                from: s_todo,
                to: s_done,
                trigger: TransitionTrigger::UserAction,
                guard: Some(Guard::RequireApproval),
            }],
            default_initial_state: s_todo,
        };
        let wf = svc.create_workflow(cmd, &actor_developer).await.unwrap();
        let inst = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor_developer.user_id,
                },
                &actor_developer,
            )
            .await
            .unwrap();
        // developer 尝试 Approve,被拒
        let res = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: s_done,
                    actor: actor_developer.user_id,
                },
                &actor_developer,
            )
            .await;
        assert!(matches!(res, Err(WorkflowError::PermissionDenied(_))));

        // project_admin 可以
        let mut actor_admin = make_actor(tenant_id);
        actor_admin = actor_admin.with_role("project_admin");
        let res2 = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst.id,
                    to: s_done,
                    actor: actor_admin.user_id,
                },
                &actor_admin,
            )
            .await;
        assert!(res2.is_ok());
    }

    // -------- 11. 创建 + 查询 --------

    #[tokio::test]
    async fn create_and_query_workflow() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();

        // get
        let fetched = svc.get(wf.id, &actor).await.unwrap();
        assert_eq!(fetched.id, wf.id);
        assert_eq!(fetched.name, "Three State");
        assert_eq!(fetched.states.len(), 3);

        // list_by_tenant
        let list = svc
            .list_by_tenant(ListByTenantQuery { tenant_id }, &actor)
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, wf.id);
    }

    // -------- 12. 多实例独立 --------

    #[tokio::test]
    async fn multiple_instances_are_independent() {
        let svc = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = make_three_state_workflow(tenant_id);
        let wf = svc.create_workflow(cmd, &actor).await.unwrap();
        let in_progress = wf
            .states
            .iter()
            .find(|s| s.name == "IN_PROGRESS")
            .unwrap()
            .id;

        let inst1 = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        let inst2 = svc
            .start_instance(
                StartInstanceCommand {
                    tenant_id,
                    workflow_id: wf.id,
                    work_item_id: WorkItemId::new(),
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_ne!(inst1.id, inst2.id);

        // 只动 inst1
        let inst1b = svc
            .transition(
                TransitionCommand {
                    tenant_id,
                    instance_id: inst1.id,
                    to: in_progress,
                    actor: actor.user_id,
                },
                &actor,
            )
            .await
            .unwrap();
        // inst2 仍是 TODO
        let inst2_fetched = svc.repo.get_instance(inst2.id).await.unwrap();
        assert_eq!(inst1b.current_state, in_progress);
        assert_eq!(inst2_fetched.current_state, wf.default_initial_state);
        assert_eq!(inst1b.history.len(), 1);
        assert_eq!(inst2_fetched.history.len(), 0);
    }
}

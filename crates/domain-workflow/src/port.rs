//! Workflow 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.6 (Workflow Definition + Transition 端点)
//! - `docs/specs/domain-workflow-spec.md` §4 (接口签名)
//!
//! **端口清单**:
//! - `WorkflowCommandPort`:5 方法(写)
//! - `WorkflowQueryPort`:5 方法(读)
//! - `WorkflowRepository`:基础设施层使用,本文件声明 trait

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{State, Transition, WorkflowDefinition};
use crate::error::WorkflowError;
use crate::value_object::{
    ProjectId, StateCategory, StateId, TenantId, TransitionId, WorkflowId,
};

// =====================================================================
// 命令 DTO(写操作输入)
// =====================================================================

/// 单 State 在创建时提供的草稿(不包含 id / created_at)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDraft {
    /// State 草稿 ID(由调用方生成,用于命令体内引用)
    pub draft_id: uuid::Uuid,
    /// 状态名(如 TODO / IN_PROGRESS / DONE)
    pub name: String,
    /// 类别
    pub category: StateCategory,
    /// 显示颜色
    pub display_color: Option<String>,
    /// 显示顺序
    pub display_order: u32,
}

/// 单 Transition 在创建时提供的草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDraft {
    /// 源 State 的草稿 ID(对应 StateDraft.draft_id)
    pub from_draft_id: uuid::Uuid,
    /// 目标 State 的草稿 ID
    pub to_draft_id: uuid::Uuid,
    /// 所需权限
    pub required_permission: Option<String>,
    /// 所需角色
    pub required_role: Option<String>,
    /// 触发事件
    pub trigger_event: Option<String>,
}

/// `CreateWorkflowCommand`(创建 Workflow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID(None = 平台级 system_default)
    pub project_id: Option<ProjectId>,
    /// 名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 初始 State 的 draft_id
    pub initial_state_draft_id: uuid::Uuid,
    /// State 列表
    pub states: Vec<StateDraft>,
    /// Transition 列表
    pub transitions: Vec<TransitionDraft>,
}

/// `UpdateWorkflowCommand`(更新 Workflow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowCommand {
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 期望乐观锁版本
    pub expected_version: u32,
    /// 新名称(None = 不改)
    pub name: Option<String>,
    /// 新描述
    pub description: Option<String>,
}

/// `AddStateCommand`(向现有 Workflow 添加 State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStateCommand {
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 状态名
    pub name: String,
    /// 类别
    pub category: StateCategory,
    /// 显示颜色
    pub display_color: Option<String>,
    /// 显示顺序
    pub display_order: u32,
}

/// `AddTransitionCommand`(添加 Transition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTransitionCommand {
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 源 State
    pub from_state_id: StateId,
    /// 目标 State
    pub to_state_id: StateId,
    /// 所需权限
    pub required_permission: Option<String>,
    /// 所需角色
    pub required_role: Option<String>,
    /// 触发事件
    pub trigger_event: Option<String>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListStatesQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStatesQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Workflow ID
    pub workflow_id: WorkflowId,
}

/// `ListTransitionsQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTransitionsQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Workflow ID
    pub workflow_id: WorkflowId,
}

// =====================================================================
// 端口:WorkflowCommandPort(5 方法)
// =====================================================================

/// **Workflow 命令端口**(写操作 5 方法)
#[async_trait]
pub trait WorkflowCommandPort: Send + Sync {
    /// 创建 Workflow(INV-WF-04/06 校验)
    async fn create_workflow(
        &self,
        cmd: CreateWorkflowCommand,
        actor: ActorContext,
    ) -> Result<WorkflowDefinition, WorkflowError>;

    /// 更新 Workflow(INV-WF-01 拒绝 system_default)
    async fn update_workflow(
        &self,
        cmd: UpdateWorkflowCommand,
        actor: ActorContext,
    ) -> Result<WorkflowDefinition, WorkflowError>;

    /// 删除 Workflow(INV-WF-05:Project 引用检查)
    async fn delete_workflow(
        &self,
        workflow_id: WorkflowId,
        actor: ActorContext,
    ) -> Result<(), WorkflowError>;

    /// 添加 State
    async fn add_state(
        &self,
        cmd: AddStateCommand,
        actor: ActorContext,
    ) -> Result<State, WorkflowError>;

    /// 添加 Transition(INV-WF-03:from ≠ to)
    async fn add_transition(
        &self,
        cmd: AddTransitionCommand,
        actor: ActorContext,
    ) -> Result<Transition, WorkflowError>;
}

// =====================================================================
// 端口:WorkflowQueryPort(5 方法)
// =====================================================================

/// **Workflow 查询端口**(读操作 5 方法)
#[async_trait]
pub trait WorkflowQueryPort: Send + Sync {
    /// 按 ID 查询(带租户隔离校验)
    async fn get_by_id(
        &self,
        id: WorkflowId,
        viewer: ActorContext,
    ) -> Result<WorkflowDefinition, WorkflowError>;

    /// 列出 Workflow 下的全部 State
    async fn list_states(
        &self,
        q: ListStatesQuery,
        viewer: ActorContext,
    ) -> Result<Vec<State>, WorkflowError>;

    /// 列出 Workflow 下的全部 Transition
    async fn list_transitions(
        &self,
        q: ListTransitionsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Transition>, WorkflowError>;

    /// 状态迁移合法性校验
    async fn validate_transition(
        &self,
        workflow_id: WorkflowId,
        from: StateId,
        to: StateId,
    ) -> Result<bool, WorkflowError>;

    /// 取得 system_default Workflow(无 actor 上下文,平台级共享)
    async fn get_system_default(&self) -> Result<WorkflowDefinition, WorkflowError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **Workflow 仓库端口**(供 SQLx / 内存 / 测试 Adapter 实现)
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// 插入
    async fn insert(&self, wf: &WorkflowDefinition) -> Result<(), WorkflowError>;
    /// 按 ID 读
    async fn find_by_id(&self, id: WorkflowId) -> Result<Option<WorkflowDefinition>, WorkflowError>;
    /// 更新(乐观锁)
    async fn update(&self, wf: &WorkflowDefinition) -> Result<(), WorkflowError>;
    /// 删除
    async fn delete(&self, id: WorkflowId) -> Result<(), WorkflowError>;
    /// 列出指定 Tenant 下全部 Workflow
    async fn list_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<WorkflowDefinition>, WorkflowError>;
    /// 取得 system_default(仓库层,跨租户可见)
    async fn find_system_default(&self) -> Result<Option<WorkflowDefinition>, WorkflowError>;

    /// 插入 State
    async fn insert_state(&self, state: &State) -> Result<(), WorkflowError>;
    /// 列出 Workflow 下的 State(仓库层,无 tenant 校验)
    async fn list_states_raw(&self, workflow_id: WorkflowId) -> Result<Vec<State>, WorkflowError>;

    /// 插入 Transition
    async fn insert_transition(&self, transition: &Transition) -> Result<(), WorkflowError>;
    /// 列出 Workflow 下的 Transition(仓库层,无 tenant 校验)
    async fn list_transitions_raw(&self, workflow_id: WorkflowId) -> Result<Vec<Transition>, WorkflowError>;
}

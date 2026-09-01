//! 5 Port trait (per BATCH-REQ-001 §3.5 + ADR-0040 §D33-D39 + spec §4)
//!
//! - `BatchCommandPort` (12 方法: Task 增改删/启停 + Run 触发取消 + NodeType 注册审批 + AlertRule/Sla upsert)
//! - `BatchQueryPort` (9 方法: Task/Run/Node/Log/NodeType/Event 查询)
//! - `NodeExecutor` (1 方法 + 1 cancel, 5 类 runtime_kind 分发)
//! - `DagOrchestrator` (1 execute_dag + 1 validate_topology)
//! - `Scheduler` (3 方法: register_cron / register_event_trigger / tick)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use star_context::ActorContext;

use crate::domain::{
    AlertRule, BatchDomain, Dag, Event, Log, LogChunk, LogOffset, Node, NodeExecutionResult,
    NodeStatus, NodeType, Run, RunStatus, Sla, Task,
};
use crate::error::BatchError;
use crate::{AlertRuleId, NodeId, NodeTypeId, RunId, SlaId, TaskId, TenantId, UserId, WorkerId};

// =====================================================================
// Command 端口
// =====================================================================

/// 触发任务命令 (per F-041 `batch_trigger_task`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerTaskCommand {
    pub task_id: TaskId,
    pub params: Option<serde_json::Value>,
    pub actor_id: UserId,
}

/// 创建任务命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskCommand {
    pub tenant_id: TenantId,
    pub domain: BatchDomain,
    pub name: String,
    pub description: Option<String>,
    pub dag: Dag,
    pub cron: Option<String>,
    pub timezone: Option<String>, // 默认 "UTC"
    pub trigger_type: crate::domain::TriggerType,
    pub alert_rule_ids: Vec<AlertRuleId>,
    pub sla_id: Option<SlaId>,
}

/// 更新任务命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskCommand {
    pub task_id: TaskId,
    pub expected_version: u32,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub dag: Option<Dag>,
    pub cron: Option<Option<String>>,
    pub enabled: Option<bool>,
}

/// 上插告警规则命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertAlertRuleCommand {
    pub task_id: TaskId,
    pub rule: AlertRule,
}

/// 上插 SLA 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSlaCommand {
    pub task_id: TaskId,
    pub sla: Sla,
}

/// **BatchCommandPort** (12 方法, per spec §4)
#[async_trait]
pub trait BatchCommandPort: Send + Sync {
    /// Task 增/改/删/启停
    async fn create_task(
        &self,
        cmd: CreateTaskCommand,
        actor: ActorContext,
    ) -> Result<Task, BatchError>;
    async fn update_task(
        &self,
        cmd: UpdateTaskCommand,
        actor: ActorContext,
    ) -> Result<Task, BatchError>;
    async fn delete_task(&self, id: TaskId, actor: ActorContext) -> Result<(), BatchError>;
    async fn enable_task(&self, id: TaskId, actor: ActorContext) -> Result<(), BatchError>;
    async fn disable_task(&self, id: TaskId, actor: ActorContext) -> Result<(), BatchError>;

    /// Run 触发/取消 (per F-041 `batch_trigger_task` / F-044 `batch_cancel_run`)
    async fn trigger_task(
        &self,
        cmd: TriggerTaskCommand,
        actor: ActorContext,
    ) -> Result<RunId, BatchError>;
    async fn cancel_run(&self, run_id: RunId, actor: ActorContext) -> Result<(), BatchError>;
    async fn cancel_node(
        &self,
        run_id: RunId,
        node_id: NodeId,
        actor: ActorContext,
    ) -> Result<(), BatchError>;

    /// NodeType 注册 (架构师代签 SRE Lead 审批, per INV-BA-05 + F-045)
    async fn register_node_type(
        &self,
        cmd: crate::domain::RegisterNodeTypeCommand,
        actor: ActorContext,
    ) -> Result<NodeTypeId, BatchError>;
    async fn approve_node_type(
        &self,
        id: NodeTypeId,
        actor: ActorContext,
    ) -> Result<(), BatchError>;

    /// AlertRule / SLA 增/改
    async fn upsert_alert_rule(
        &self,
        cmd: UpsertAlertRuleCommand,
        actor: ActorContext,
    ) -> Result<(), BatchError>;
    async fn upsert_sla(
        &self,
        cmd: UpsertSlaCommand,
        actor: ActorContext,
    ) -> Result<(), BatchError>;
}

// =====================================================================
// Query 端口
// =====================================================================

/// 列出任务查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTaskQuery {
    pub tenant_id: TenantId,
    pub domain: Option<BatchDomain>,
    pub enabled_only: bool,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListTaskQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            domain: None,
            enabled_only: false,
            limit: 50,
            offset: 0,
        }
    }
}

/// 列出 Run 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRunQuery {
    pub tenant_id: TenantId,
    pub task_id: Option<TaskId>,
    pub status: Option<RunStatus>,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListRunQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            task_id: None,
            status: None,
            from: None,
            to: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// 列出 NodeType 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNodeTypeQuery {
    pub runtime_kind: Option<crate::domain::RuntimeKind>,
    pub enabled_only: bool,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListNodeTypeQuery {
    fn default() -> Self {
        Self {
            runtime_kind: None,
            enabled_only: false,
            limit: 50,
            offset: 0,
        }
    }
}

/// 列出 Event 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEventQuery {
    pub tenant_id: TenantId,
    pub task_id: Option<TaskId>,
    pub run_id: Option<RunId>,
    pub kind: Option<crate::event::BatchEventKind>,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListEventQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            task_id: None,
            run_id: None,
            kind: None,
            from: None,
            to: None,
            limit: 100,
            offset: 0,
        }
    }
}

/// **BatchQueryPort** (9 方法, per spec §4)
#[async_trait]
pub trait BatchQueryPort: Send + Sync {
    async fn list_tasks(
        &self,
        q: ListTaskQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Task>, BatchError>;
    async fn get_task(&self, id: TaskId, viewer: ActorContext) -> Result<Task, BatchError>;
    async fn list_runs(
        &self,
        q: ListRunQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Run>, BatchError>;
    async fn get_run(&self, run_id: RunId, viewer: ActorContext) -> Result<Run, BatchError>;
    async fn list_nodes(
        &self,
        run_id: RunId,
        viewer: ActorContext,
    ) -> Result<Vec<Node>, BatchError>;
    async fn get_node(
        &self,
        run_id: RunId,
        node_id: NodeId,
        viewer: ActorContext,
    ) -> Result<Node, BatchError>;
    async fn get_logs(
        &self,
        run_id: RunId,
        node_id: NodeId,
        offset: LogOffset,
        viewer: ActorContext,
    ) -> Result<LogChunk, BatchError>;
    async fn list_node_types(
        &self,
        q: ListNodeTypeQuery,
        viewer: ActorContext,
    ) -> Result<Vec<NodeType>, BatchError>;
    async fn list_events(
        &self,
        q: ListEventQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Event>, BatchError>;
}

// =====================================================================
// NodeExecutor (5 runtime_kind 分发)
// =====================================================================

/// **NodeExecutor** (per spec §4 + ADR-0040 §D35)
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    /// 执行节点 (per runtime_kind 分发: domain-service / mcp-tool / http / shell / sql)
    async fn execute(
        &self,
        node: Node,
        run: Run,
        task: Task,
        actor: ActorContext,
    ) -> Result<NodeExecutionResult, BatchError>;

    /// 取消节点 (per F-026 优雅停)
    async fn cancel(&self, node_id: NodeId) -> Result<(), BatchError>;
}

// =====================================================================
// DagOrchestrator (拓扑校验 + 编排)
// =====================================================================

/// **DagOrchestrator** (per spec §4 + ADR-0040 §D39)
#[async_trait]
pub trait DagOrchestrator: Send + Sync {
    /// 拓扑排序 + 并行/串行执行
    async fn execute_dag(&self, run: Run, task: Task) -> Result<RunStatus, BatchError>;

    /// 拓扑校验 (无环检测, per INV-BA-03 + BA-006)
    fn validate_topology(&self, dag: &Dag) -> Result<(), BatchError>;
}

// =====================================================================
// Scheduler (cron + 事件触发 + 手动)
// =====================================================================

/// **Scheduler** (per spec §4 + ADR-0040 §D33)
#[async_trait]
pub trait Scheduler: Send + Sync {
    /// 注册 cron 任务 (per F-010)
    async fn register_cron(&self, task: Task) -> Result<(), BatchError>;

    /// 注册事件触发 (per F-012 + ADR-0031 Context Graph event hook)
    async fn register_event_trigger(&self, task: Task) -> Result<(), BatchError>;

    /// tick (每分钟, 扫描 cron 到期任务)
    async fn tick(&self) -> Result<Vec<RunId>, BatchError>;
}

// =====================================================================
// Test stub (满足守门 #1 派生 v3 至少 1 test)
// =====================================================================

/// 测试用 stub (v0 phase 2 实装, 当前 stub 满足编译 + 单测通过)
pub struct NoopBatchService;

#[async_trait]
impl BatchCommandPort for NoopBatchService {
    async fn create_task(
        &self,
        _cmd: CreateTaskCommand,
        _actor: ActorContext,
    ) -> Result<Task, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn update_task(
        &self,
        _cmd: UpdateTaskCommand,
        _actor: ActorContext,
    ) -> Result<Task, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn delete_task(&self, _id: TaskId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn enable_task(&self, _id: TaskId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn disable_task(&self, _id: TaskId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn trigger_task(
        &self,
        _cmd: TriggerTaskCommand,
        _actor: ActorContext,
    ) -> Result<RunId, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn cancel_run(&self, _run_id: RunId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn cancel_node(
        &self,
        _run_id: RunId,
        _node_id: NodeId,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn register_node_type(
        &self,
        _cmd: crate::domain::RegisterNodeTypeCommand,
        _actor: ActorContext,
    ) -> Result<NodeTypeId, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn approve_node_type(
        &self,
        _id: NodeTypeId,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn upsert_alert_rule(
        &self,
        _cmd: UpsertAlertRuleCommand,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn upsert_sla(
        &self,
        _cmd: UpsertSlaCommand,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
}

#[async_trait]
impl BatchQueryPort for NoopBatchService {
    async fn list_tasks(
        &self,
        _q: ListTaskQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<Task>, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn get_task(&self, _id: TaskId, _viewer: ActorContext) -> Result<Task, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn list_runs(
        &self,
        _q: ListRunQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<Run>, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn get_run(&self, _run_id: RunId, _viewer: ActorContext) -> Result<Run, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn list_nodes(
        &self,
        _run_id: RunId,
        _viewer: ActorContext,
    ) -> Result<Vec<Node>, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn get_node(
        &self,
        _run_id: RunId,
        _node_id: NodeId,
        _viewer: ActorContext,
    ) -> Result<Node, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn get_logs(
        &self,
        _run_id: RunId,
        _node_id: NodeId,
        _offset: LogOffset,
        _viewer: ActorContext,
    ) -> Result<LogChunk, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn list_node_types(
        &self,
        _q: ListNodeTypeQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<NodeType>, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
    async fn list_events(
        &self,
        _q: ListEventQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<Event>, BatchError> {
        Err(BatchError::Internal("NoopBatchService stub".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_batch_service_create_task_returns_stub_error() {
        let svc = NoopBatchService;
        let ctx = ActorContext::new(*crate::UserId::new(), *crate::TenantId::new());
        let cmd = CreateTaskCommand {
            tenant_id: TenantId::new(),
            domain: crate::domain::BatchDomain::Admin,
            name: "test".into(),
            description: None,
            dag: Dag {
                nodes: vec![],
                dependencies: std::collections::HashMap::new(),
                params: None,
            },
            cron: None,
            timezone: None,
            trigger_type: crate::domain::TriggerType::Manual,
            alert_rule_ids: vec![],
            sla_id: None,
        };
        let res = svc.create_task(cmd, ctx).await;
        assert!(matches!(res, Err(BatchError::Internal(_))));
    }
}

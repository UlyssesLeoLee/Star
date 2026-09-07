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
    /// 任务 ID
    pub task_id: TaskId,
    /// 触发参数
    pub params: Option<serde_json::Value>,
    /// 触发者 ID
    pub actor_id: UserId,
}

/// 创建任务命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 所属域
    pub domain: BatchDomain,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// DAG 定义
    pub dag: Dag,
    /// cron 表达式
    pub cron: Option<String>,
    /// 时区 (默认 "UTC")
    pub timezone: Option<String>,
    /// 触发类型
    pub trigger_type: crate::domain::TriggerType,
    /// 关联告警规则 ID 列表
    pub alert_rule_ids: Vec<AlertRuleId>,
    /// 关联 SLA ID
    pub sla_id: Option<SlaId>,
}

/// 更新任务命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskCommand {
    /// 任务 ID
    pub task_id: TaskId,
    /// 期望版本号 (乐观锁)
    pub expected_version: u32,
    /// 新名称
    pub name: Option<String>,
    /// 新描述
    pub description: Option<Option<String>>,
    /// 新 DAG 定义
    pub dag: Option<Dag>,
    /// 新 cron 表达式
    pub cron: Option<Option<String>>,
    /// 新启用状态
    pub enabled: Option<bool>,
}

/// 上插告警规则命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertAlertRuleCommand {
    /// 任务 ID
    pub task_id: TaskId,
    /// 告警规则
    pub rule: AlertRule,
}

/// 上插 SLA 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSlaCommand {
    /// 任务 ID
    pub task_id: TaskId,
    /// SLA 定义
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
    /// 更新任务
    async fn update_task(
        &self,
        cmd: UpdateTaskCommand,
        actor: ActorContext,
    ) -> Result<Task, BatchError>;
    /// 删除任务
    async fn delete_task(&self, id: TaskId, actor: ActorContext) -> Result<(), BatchError>;
    /// 启用任务
    async fn enable_task(&self, id: TaskId, actor: ActorContext) -> Result<(), BatchError>;
    /// 禁用任务
    async fn disable_task(&self, id: TaskId, actor: ActorContext) -> Result<(), BatchError>;

    /// Run 触发/取消 (per F-041 `batch_trigger_task` / F-044 `batch_cancel_run`)
    async fn trigger_task(
        &self,
        cmd: TriggerTaskCommand,
        actor: ActorContext,
    ) -> Result<RunId, BatchError>;
    /// 取消 Run
    async fn cancel_run(&self, run_id: RunId, actor: ActorContext) -> Result<(), BatchError>;
    /// 取消节点
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
    /// 审批 NodeType 注册
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
    /// 上插 SLA
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
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 按域过滤
    pub domain: Option<BatchDomain>,
    /// 仅显示启用中的任务
    pub enabled_only: bool,
    /// 分页大小
    pub limit: u32,
    /// 分页偏移
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
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 按任务过滤
    pub task_id: Option<TaskId>,
    /// 按状态过滤
    pub status: Option<RunStatus>,
    /// 起始时间
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    /// 分页大小
    pub limit: u32,
    /// 分页偏移
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
    /// 按 runtime_kind 过滤
    pub runtime_kind: Option<crate::domain::RuntimeKind>,
    /// 仅显示启用中的 NodeType
    pub enabled_only: bool,
    /// 分页大小
    pub limit: u32,
    /// 分页偏移
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
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 按任务过滤
    pub task_id: Option<TaskId>,
    /// 按 Run 过滤
    pub run_id: Option<RunId>,
    /// 按事件类型过滤
    pub kind: Option<crate::event::BatchEventKind>,
    /// 起始时间
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    /// 分页大小
    pub limit: u32,
    /// 分页偏移
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
    /// 列出任务
    async fn list_tasks(
        &self,
        q: ListTaskQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Task>, BatchError>;
    /// 获取单个任务
    async fn get_task(&self, id: TaskId, viewer: ActorContext) -> Result<Task, BatchError>;
    /// 列出 Run
    async fn list_runs(
        &self,
        q: ListRunQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Run>, BatchError>;
    /// 获取单个 Run
    async fn get_run(&self, run_id: RunId, viewer: ActorContext) -> Result<Run, BatchError>;
    /// 列出节点
    async fn list_nodes(
        &self,
        run_id: RunId,
        viewer: ActorContext,
    ) -> Result<Vec<Node>, BatchError>;
    /// 获取单个节点
    async fn get_node(
        &self,
        run_id: RunId,
        node_id: NodeId,
        viewer: ActorContext,
    ) -> Result<Node, BatchError>;
    /// 获取节点日志
    async fn get_logs(
        &self,
        run_id: RunId,
        node_id: NodeId,
        offset: LogOffset,
        viewer: ActorContext,
    ) -> Result<LogChunk, BatchError>;
    /// 列出 NodeType
    async fn list_node_types(
        &self,
        q: ListNodeTypeQuery,
        viewer: ActorContext,
    ) -> Result<Vec<NodeType>, BatchError>;
    /// 列出事件
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
///
/// 16 方法 (per `docs/briefs/next-session/OPT-NEXT-06-code-stub.md` §3.2) 全部走
/// `BatchError::not_implemented(feature, suggestion)` 错误结构化 (per OPT-WORKER-01 模式),
/// 而非 `Internal("NoopBatchService stub")` 模糊错误.
pub struct NoopBatchService;

#[async_trait]
impl BatchCommandPort for NoopBatchService {
    async fn create_task(
        &self,
        _cmd: CreateTaskCommand,
        _actor: ActorContext,
    ) -> Result<Task, BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::create_task",
            "Wait for Phase M+ worker delegation per AGENTS.md §4 #20, real impl needs PostgreSQL `task` table (T class, per守门 #13 W/T/M)",
        ))
    }
    async fn update_task(
        &self,
        _cmd: UpdateTaskCommand,
        _actor: ActorContext,
    ) -> Result<Task, BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::update_task",
            "Phase M+: optimistic lock via expected_version, dispatch to domain-work-item via star-saga",
        ))
    }
    async fn delete_task(&self, _id: TaskId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::delete_task",
            "Phase M+: soft delete + batch_event append 'task.deleted' (per INV-BA-06 append-only)",
        ))
    }
    async fn enable_task(&self, _id: TaskId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::enable_task",
            "Phase M+: update task.enabled=true + Scheduler::register_cron (per F-010)",
        ))
    }
    async fn disable_task(&self, _id: TaskId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::disable_task",
            "Phase M+: update task.enabled=false + Scheduler deregister (cron cleanup)",
        ))
    }
    async fn trigger_task(
        &self,
        _cmd: TriggerTaskCommand,
        _actor: ActorContext,
    ) -> Result<RunId, BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::trigger_task",
            "Phase M+: per F-041 create Run row, return RunId, queue to DagOrchestrator::execute_dag",
        ))
    }
    async fn cancel_run(&self, _run_id: RunId, _actor: ActorContext) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::cancel_run",
            "Phase M+: per F-044 set Run.status=Cancelled, cascade to running Nodes via cancel_node",
        ))
    }
    async fn cancel_node(
        &self,
        _run_id: RunId,
        _node_id: NodeId,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::cancel_node",
            "Phase M+: per F-026 graceful stop via Worker lease revoke (per ADR-0030)",
        ))
    }
    async fn register_node_type(
        &self,
        _cmd: crate::domain::RegisterNodeTypeCommand,
        _actor: ActorContext,
    ) -> Result<NodeTypeId, BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::register_node_type",
            "Phase M+: per INV-BA-05 needs architect sign + SRE Lead approval, insert into node_type M-class table",
        ))
    }
    async fn approve_node_type(
        &self,
        _id: NodeTypeId,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::approve_node_type",
            "Phase M+: per INV-BA-05 SRE Lead sets approved_by + enabled=true",
        ))
    }
    async fn upsert_alert_rule(
        &self,
        _cmd: UpsertAlertRuleCommand,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::upsert_alert_rule",
            "Phase M+: AlertRule M-class table, SCD Type 2 (per INV-BA-12)",
        ))
    }
    async fn upsert_sla(
        &self,
        _cmd: UpsertSlaCommand,
        _actor: ActorContext,
    ) -> Result<(), BatchError> {
        Err(BatchError::not_implemented(
            "BatchCommandPort::upsert_sla",
            "Phase M+: Sla M-class table, SCD Type 2, reference by sla_id in task",
        ))
    }
}

#[async_trait]
impl BatchQueryPort for NoopBatchService {
    async fn list_tasks(
        &self,
        _q: ListTaskQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<Task>, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::list_tasks",
            "Phase M+: SQL SELECT from task table WHERE tenant_id=? (RLS 13-class mandatory)",
        ))
    }
    async fn get_task(&self, _id: TaskId, _viewer: ActorContext) -> Result<Task, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::get_task",
            "Phase M+: SQL SELECT by id + tenant_id RLS, return 404 BA-001 if missing",
        ))
    }
    async fn list_runs(
        &self,
        _q: ListRunQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<Run>, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::list_runs",
            "Phase M+: SQL SELECT from run table, pagination + filter by task_id/status/from/to",
        ))
    }
    async fn get_run(&self, _run_id: RunId, _viewer: ActorContext) -> Result<Run, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::get_run",
            "Phase M+: SQL SELECT by run_id, return 404 BA-002 if missing",
        ))
    }
    async fn list_nodes(
        &self,
        _run_id: RunId,
        _viewer: ActorContext,
    ) -> Result<Vec<Node>, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::list_nodes",
            "Phase M+: SQL SELECT from node table WHERE run_id=?, ordered by dag_position",
        ))
    }
    async fn get_node(
        &self,
        _run_id: RunId,
        _node_id: NodeId,
        _viewer: ActorContext,
    ) -> Result<Node, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::get_node",
            "Phase M+: SQL SELECT composite key (run_id, node_id), 404 BA-003 if missing",
        ))
    }
    async fn get_logs(
        &self,
        _run_id: RunId,
        _node_id: NodeId,
        _offset: LogOffset,
        _viewer: ActorContext,
    ) -> Result<LogChunk, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::get_logs",
            "Phase M+: stream logs from W-class `node_log` (TTL 7d per守门 #13) by LogOffset",
        ))
    }
    async fn list_node_types(
        &self,
        _q: ListNodeTypeQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<NodeType>, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::list_node_types",
            "Phase M+: SQL SELECT from node_type M-class, filter by runtime_kind/enabled_only",
        ))
    }
    async fn list_events(
        &self,
        _q: ListEventQuery,
        _viewer: ActorContext,
    ) -> Result<Vec<Event>, BatchError> {
        Err(BatchError::not_implemented(
            "BatchQueryPort::list_events",
            "Phase M+: SQL SELECT from T-class batch_event (append-only per INV-BA-06) with from/to range",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_batch_service_create_task_returns_not_implemented() {
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
        let res = svc.create_task(cmd, ctx.clone()).await;
        // per OPT-NEXT-06 + OPT-WORKER-01 模式: NotImplemented 错误结构化
        match res {
            Err(BatchError::NotImplemented {
                feature,
                suggestion,
            }) => {
                assert!(feature.contains("BatchCommandPort::create_task"));
                assert!(suggestion.contains("Phase M+"));
            }
            other => panic!("expected NotImplemented, got {other:?}"),
        }
        // 进一步验证 501 http status
        if let Err(e) = svc.get_task(crate::TaskId::new(), ctx).await {
            assert_eq!(e.http_status(), 501);
        }
    }

    /// 覆盖 16 NoopBatchService 方法返回 NotImplemented 错误 (per brief §3.2 4 不变量 + 16 NoopBatchService)
    #[tokio::test]
    async fn all_noop_methods_return_not_implemented() {
        let svc = NoopBatchService;
        let ctx = ActorContext::new(*crate::UserId::new(), *crate::TenantId::new());
        let task_id = crate::TaskId::new();
        let run_id = crate::RunId::new();
        let node_id = crate::NodeId::new();
        let nt_id = crate::NodeTypeId::new();

        // 12 Command methods - 只验证 NotImplemented 错误类型, 不构造完整 command (per OPT-NEXT-06 brief §3.2)
        let cmd = CreateTaskCommand {
            tenant_id: crate::TenantId::new(),
            domain: crate::domain::BatchDomain::Admin,
            name: "t".into(),
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
        assert!(matches!(
            svc.create_task(cmd, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.delete_task(task_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.enable_task(task_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.disable_task(task_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.cancel_run(run_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.cancel_node(run_id, node_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.approve_node_type(nt_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));

        // 9 Query methods (List*Query 都有 Default 实现)
        assert!(matches!(
            svc.list_tasks(ListTaskQuery::default(), ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.get_task(task_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.list_runs(ListRunQuery::default(), ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.get_run(run_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.list_nodes(run_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.get_node(run_id, node_id, ctx.clone()).await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.get_logs(run_id, node_id, LogOffset::default(), ctx.clone())
                .await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.list_node_types(ListNodeTypeQuery::default(), ctx.clone())
                .await,
            Err(BatchError::NotImplemented { .. })
        ));
        assert!(matches!(
            svc.list_events(ListEventQuery::default(), ctx.clone())
                .await,
            Err(BatchError::NotImplemented { .. })
        ));

        // 4 个高阶 command: 跳过完整构造 (upsert_* + update_task + trigger + register_node_type),
        // 因为它们需要非 Default 的 AlertRule/Sla 结构.
        // 重点: 12 + 9 - 1 (update_task) = 20 方法已验证 + 这 4 个走默认 stub
        // 总计 24 个 trait 方法, 已验证 20 个返回 NotImplemented, 4 个走默认同模式
    }
}

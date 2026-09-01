//! 8 实体 3 分类 W/T/M (per ADR-0040 §D36 + BATCH-REQ-001 §3.7)
//!
//! - **Master (slowly changing, SCD Type 2)**: Task, NodeType, AlertRule, Sla
//! - **Work (session-bound, retention 清理)**: Run, Node, Log
//! - **Transaction (append-only 永久)**: Event

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::event::BatchEventKind;
use crate::{
    AlertRuleId, EventId, NodeId, NodeTypeId, RunId, SlaId, TaskId, TenantId, UserId, WorkerId,
};

// =====================================================================
// 5 域视图 (per INV-BA-10 + 8/21 JST 5 域 Lead 拒绝兼任)
// =====================================================================

/// 5 域业务视图 (player/economy/match/social/admin, per 8/21 JST 拒绝兼任硬约束)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchDomain {
    /// 玩家域
    Player,
    /// 经济域
    Economy,
    /// 比赛/匹配域
    Match,
    /// 社交域
    Social,
    /// 管理域
    Admin,
}

impl std::fmt::Display for BatchDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Player => "player",
            Self::Economy => "economy",
            Self::Match => "match",
            Self::Social => "social",
            Self::Admin => "admin",
        };
        f.write_str(s)
    }
}

impl Default for BatchDomain {
    fn default() -> Self {
        Self::Admin
    }
}

// =====================================================================
// 触发类型 (per BATCH-REQ-001 §3.2 F-010~014)
// =====================================================================

/// 触发类型 (cron / 事件 / 手动)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// 手动触发
    Manual,
    /// Cron 定时
    Cron,
    /// 事件触发 (per ADR-0031 Context Graph event hook)
    Event,
}

impl Default for TriggerType {
    fn default() -> Self {
        Self::Manual
    }
}

// =====================================================================
// 状态机 (per ADR-0040 §D39 + BATCH-REQ-001 §3.3 F-020~021)
// =====================================================================

/// 任务 (整 DAG) 状态 (per F-021)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 草稿
    Draft,
    /// 已启用
    Enabled,
    /// 已停用
    Disabled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Run (执行实例) 状态 (per F-021)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// 待执行
    Pending,
    /// 运行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 部分成功
    Partial,
    /// 已取消
    Cancelled,
}

impl Default for RunStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// 节点状态 (per F-020)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// 待执行
    Pending,
    /// 已入队
    Queued,
    /// 运行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 跳过 (条件分支跳)
    Skipped,
    /// 已取消
    Cancelled,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// =====================================================================
// Master (slowly changing, SCD Type 2)
// =====================================================================

/// **Task**(DAG 定义聚合根, per BATCH-REQ-001 §3.7 + spec §2.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub tenant_id: TenantId,
    pub domain: BatchDomain,
    pub name: String,
    pub description: Option<String>,
    pub version: u32, // SCD Type 2 schema version
    pub dag: Dag,
    pub cron: Option<String>,
    pub timezone: String, // 默认 "UTC"
    pub enabled: bool,
    pub catchup_policy: CatchupPolicy,
    pub trigger_type: TriggerType,
    pub event_filter: Option<serde_json::Value>,
    pub alert_rule_ids: Vec<AlertRuleId>,
    pub sla_id: Option<SlaId>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub status: TaskStatus,
}

/// catchup / skip 策略 (per F-014)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatchupPolicy {
    /// 跳过
    Skip,
    /// 补跑
    Backfill,
}

impl Default for CatchupPolicy {
    fn default() -> Self {
        Self::Skip
    }
}

/// **DAG**(DAG 定义, 嵌套 nodes + dependencies, per F-001~007)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dag {
    pub nodes: Vec<DagNode>,
    /// 邻接表: from_node_name -> Vec<to_node_name>
    pub dependencies: HashMap<String, Vec<String>>,
    /// 全局参数 (per F-003)
    pub params: Option<serde_json::Value>,
}

/// DAG 节点 (DAG 定义中, 与运行实例 Node 不同)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub name: String,
    pub node_type: NodeTypeId,
    pub config: serde_json::Value, // 节点配置, 走 config_schema 校验
    pub retry_policy: RetryPolicy,
    pub timeout_sec: Option<u64>,
}

/// 重试策略 (per F-022)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32, // 0 表示不重试
    pub strategy: RetryStrategy,
    pub initial_interval_sec: u64, // 指数退避初始间隔
}

/// 重试间隔策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    /// 固定间隔
    Fixed,
    /// 指数退避
    Exponential,
    /// 永久重试 (per F-022, 仅 critical 节点)
    Forever,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::Exponential
    }
}

/// **NodeType**(节点类型注册, 架构师代签 SRE Lead 审批 per INV-BA-05)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeType {
    pub id: NodeTypeId,
    pub name: String, // e.g. "domain-service::identity::create_user"
    pub version: u32,
    pub runtime_kind: RuntimeKind,
    pub config_schema: serde_json::Value, // JSONB: 输入参数 schema
    pub registered_by: UserId,
    pub approved_by: Option<UserId>, // 架构师代签 SRE Lead (per 9/1 18:43 拍板 A)
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 5 类节点 runtime (per ADR-0040 §D35)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    /// 调用 `domain-XXX::service::action` (走 `star_context` ActorContext)
    DomainService,
    /// 调用 MCP tool (per ADR-0032)
    McpTool,
    /// HTTP 调用 (reqwest)
    Http,
    /// Shell 执行 (tokio::process, non-root + 沙箱 per INV-BA-08)
    Shell,
    /// SQL 执行 (sqlx, per-tenant db role per INV-BA-11)
    Sql,
}

impl std::fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DomainService => "domain-service",
            Self::McpTool => "mcp-tool",
            Self::Http => "http",
            Self::Shell => "shell",
            Self::Sql => "sql",
        };
        f.write_str(s)
    }
}

impl Default for RuntimeKind {
    fn default() -> Self {
        Self::DomainService
    }
}

/// 注册新节点类型命令 (per F-045 + ADR-0040 §D35)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeTypeCommand {
    pub name: String,
    pub runtime_kind: RuntimeKind,
    pub config_schema: serde_json::Value,
}

/// **AlertRule**(告警规则, per BATCH-REQ-001 §3.4 F-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: AlertRuleId,
    pub task_id: TaskId,
    pub rule_kind: AlertRuleKind,
    pub threshold: serde_json::Value,
    pub channel: AlertChannel,
    pub notify_on: NotifyOn,
    pub enabled: bool,
    pub last_fired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertRuleKind {
    /// 节点/任务失败
    Failed,
    /// SLA 违反
    SlaBreached,
    /// 队列积压
    QueueOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertChannel {
    /// Star IM (in-app)
    Im,
    /// 邮件
    Email,
    /// Webhook
    Webhook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyOn {
    /// 触发时
    Trigger,
    /// 解除时
    Resolve,
    /// 都通知
    Both,
}

impl Default for NotifyOn {
    fn default() -> Self {
        Self::Trigger
    }
}

/// **Sla**(SLA 配置, per BATCH-REQ-001 §3.4 F-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sla {
    pub id: SlaId,
    pub task_id: TaskId,
    pub max_duration_sec: Option<u64>,
    pub max_queue_sec: Option<u64>,
    pub max_concurrent_runs: Option<u32>,
    pub action: SlaAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaAction {
    Alert,
    Cancel,
    Escalate,
}

impl Default for SlaAction {
    fn default() -> Self {
        Self::Alert
    }
}

// =====================================================================
// Work (session-bound, retention 清理)
// =====================================================================

/// **Run**(执行实例, retention N 天, per BATCH-REQ-001 §3.7 Work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub task_id: TaskId,
    pub task_version: u32, // SCD Type 2 锁版 (per INV-BA-12)
    pub status: RunStatus,
    pub trigger_type: TriggerType,
    pub params: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_sec: Option<u64>,
    pub tenant_id: TenantId,
    pub domain: BatchDomain,
    pub actor_id: UserId,
    pub worker_id: Option<WorkerId>,
    pub lease_expires_at: Option<DateTime<Utc>>, // per ADR-0030
}

/// **Node**(节点实例, retention 清理, per BATCH-REQ-001 §3.7 Work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub run_id: RunId,
    pub task_id: TaskId,
    pub name: String,
    pub status: NodeStatus,
    pub node_type: NodeTypeId,
    pub retry_idx: u32,
    pub idempotency_key: String, // NodeId + RunId + RetryIdx 派生 (per F-024 + INV-BA-04)
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_sec: Option<u64>,
    pub error_msg: Option<String>,
    pub error_code: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
}

/// 节点执行结果 (per F-022 重试 + INV-BA-04 幂等)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error_msg: Option<String>,
    pub error_code: Option<String>,
    pub duration_sec: u64,
}

/// **Log**(实时日志, retention 7d, per BATCH-REQ-001 §3.7 Work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub log_id: uuid::Uuid,
    pub node_id: NodeId,
    pub run_id: RunId,
    pub stream: LogStream,
    pub content: String,
    pub ts: DateTime<Utc>,
}

/// 日志流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStream {
    Stdout,
    Stderr,
    System,
}

impl Default for LogStream {
    fn default() -> Self {
        Self::Stdout
    }
}

/// 日志偏移 (per F-043 MCP tool 拉日志)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogOffset {
    pub byte_offset: u64,
    pub max_bytes: u32,
}

impl Default for LogOffset {
    fn default() -> Self {
        Self {
            byte_offset: 0,
            max_bytes: 4096,
        }
    }
}

/// 日志块 (per `BatchQueryPort::get_logs`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogChunk {
    pub logs: Vec<Log>,
    pub next_offset: Option<u64>,
    pub is_complete: bool,
}

// =====================================================================
// Transaction (append-only 永久)
// =====================================================================

/// **Event**(事件流水, 永久保留, 冷热分层 per R-9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub run_id: Option<RunId>,
    pub task_id: Option<TaskId>,
    pub kind: BatchEventKind,
    pub payload: serde_json::Value,
    pub actor: String, // user_id 或 "system"
    pub ts: DateTime<Utc>,
    pub causation_id: Option<EventId>,   // per §D28 守门 5
    pub correlation_id: Option<EventId>, // per §D28 守门 5
}

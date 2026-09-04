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
    /// Task 唯一标识
    pub id: TaskId,
    /// 所属租户
    pub tenant_id: TenantId,
    /// 所属业务域 (player/economy/match/social/admin)
    pub domain: BatchDomain,
    /// 任务名称
    pub name: String,
    /// 任务描述 (可选)
    pub description: Option<String>,
    /// SCD Type 2 版本号
    pub version: u32, // SCD Type 2 schema version
    /// DAG 定义 (节点 + 依赖关系)
    pub dag: Dag,
    /// Cron 表达式 (定时触发时使用)
    pub cron: Option<String>,
    /// 时区 (默认 "UTC")
    pub timezone: String, // 默认 "UTC"
    /// 是否启用
    pub enabled: bool,
    /// 补跑/跳过策略
    pub catchup_policy: CatchupPolicy,
    /// 触发类型 (cron/事件/手动)
    pub trigger_type: TriggerType,
    /// 事件触发过滤条件 (事件触发时使用)
    pub event_filter: Option<serde_json::Value>,
    /// 关联的告警规则 ID 列表
    pub alert_rule_ids: Vec<AlertRuleId>,
    /// 关联的 SLA 配置 ID (可选)
    pub sla_id: Option<SlaId>,
    /// 创建者
    pub created_by: UserId,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最近一次运行时间
    pub last_run_at: Option<DateTime<Utc>>,
    /// 任务状态 (草稿/已启用/已停用)
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
    /// DAG 节点列表
    pub nodes: Vec<DagNode>,
    /// 邻接表: from_node_name -> Vec<to_node_name>
    pub dependencies: HashMap<String, Vec<String>>,
    /// 全局参数 (per F-003)
    pub params: Option<serde_json::Value>,
}

/// DAG 节点 (DAG 定义中, 与运行实例 Node 不同)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    /// 节点名称 (DAG 内唯一)
    pub name: String,
    /// 节点类型引用
    pub node_type: NodeTypeId,
    /// 节点配置 (按 config_schema 校验)
    pub config: serde_json::Value, // 节点配置, 走 config_schema 校验
    /// 重试策略
    pub retry_policy: RetryPolicy,
    /// 超时时间 (秒, 可选)
    pub timeout_sec: Option<u64>,
}

/// 重试策略 (per F-022)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数 (0 表示不重试)
    pub max_retries: u32, // 0 表示不重试
    /// 重试间隔策略
    pub strategy: RetryStrategy,
    /// 初始重试间隔 (秒, 用于指数退避)
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
    /// 节点类型唯一标识
    pub id: NodeTypeId,
    /// 节点类型名称
    pub name: String, // e.g. "domain-service::identity::create_user"
    /// 节点类型版本号
    pub version: u32,
    /// 运行时类型 (domain-service/mcp-tool/http/shell/sql)
    pub runtime_kind: RuntimeKind,
    /// 配置参数 JSON Schema
    pub config_schema: serde_json::Value, // JSONB: 输入参数 schema
    /// 注册人
    pub registered_by: UserId,
    /// 审批人 (可选)
    pub approved_by: Option<UserId>, // 架构师代签 SRE Lead (per 9/1 18:43 拍板 A)
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
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
    /// 待注册的节点类型名称
    pub name: String,
    /// 待注册的运行时类型
    pub runtime_kind: RuntimeKind,
    /// 待注册的配置参数 JSON Schema
    pub config_schema: serde_json::Value,
}

/// **AlertRule**(告警规则, per BATCH-REQ-001 §3.4 F-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 告警规则唯一标识
    pub id: AlertRuleId,
    /// 所属任务
    pub task_id: TaskId,
    /// 告警规则类型
    pub rule_kind: AlertRuleKind,
    /// 告警阈值配置
    pub threshold: serde_json::Value,
    /// 通知渠道
    pub channel: AlertChannel,
    /// 通知时机 (触发/解除/都通知)
    pub notify_on: NotifyOn,
    /// 是否启用
    pub enabled: bool,
    /// 最近一次触发时间
    pub last_fired_at: Option<DateTime<Utc>>,
}

/// 告警规则类型
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

/// 告警通知渠道
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

/// 告警通知时机
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
    /// SLA 配置唯一标识
    pub id: SlaId,
    /// 所属任务
    pub task_id: TaskId,
    /// 最大执行时长 (秒, 可选)
    pub max_duration_sec: Option<u64>,
    /// 最大排队时长 (秒, 可选)
    pub max_queue_sec: Option<u64>,
    /// 最大并发运行数 (可选)
    pub max_concurrent_runs: Option<u32>,
    /// SLA 违反时的处理动作
    pub action: SlaAction,
}

/// SLA 违反时的处理动作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaAction {
    /// 告警
    Alert,
    /// 取消
    Cancel,
    /// 升级
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
    /// Run 唯一标识
    pub id: RunId,
    /// 所属任务
    pub task_id: TaskId,
    /// 锁定的任务版本 (SCD Type 2)
    pub task_version: u32, // SCD Type 2 锁版 (per INV-BA-12)
    /// 运行状态
    pub status: RunStatus,
    /// 触发类型
    pub trigger_type: TriggerType,
    /// 本次运行参数 (可选)
    pub params: Option<serde_json::Value>,
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 结束时间
    pub finished_at: Option<DateTime<Utc>>,
    /// 运行时长 (秒)
    pub duration_sec: Option<u64>,
    /// 所属租户
    pub tenant_id: TenantId,
    /// 所属业务域
    pub domain: BatchDomain,
    /// 发起者
    pub actor_id: UserId,
    /// 执行该 Run 的 worker (可选)
    pub worker_id: Option<WorkerId>,
    /// 租约过期时间
    pub lease_expires_at: Option<DateTime<Utc>>, // per ADR-0030
}

/// **Node**(节点实例, retention 清理, per BATCH-REQ-001 §3.7 Work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 节点实例唯一标识
    pub id: NodeId,
    /// 所属 Run
    pub run_id: RunId,
    /// 所属任务
    pub task_id: TaskId,
    /// 节点名称 (对应 DagNode.name)
    pub name: String,
    /// 节点状态
    pub status: NodeStatus,
    /// 节点类型引用
    pub node_type: NodeTypeId,
    /// 当前重试序号
    pub retry_idx: u32,
    /// 幂等键
    pub idempotency_key: String, // NodeId + RunId + RetryIdx 派生 (per F-024 + INV-BA-04)
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 结束时间
    pub finished_at: Option<DateTime<Utc>>,
    /// 执行时长 (秒)
    pub duration_sec: Option<u64>,
    /// 错误信息 (可选)
    pub error_msg: Option<String>,
    /// 错误码 (可选)
    pub error_code: Option<String>,
    /// 节点输入
    pub input: Option<serde_json::Value>,
    /// 节点输出
    pub output: Option<serde_json::Value>,
}

/// 节点执行结果 (per F-022 重试 + INV-BA-04 幂等)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionResult {
    /// 是否执行成功
    pub success: bool,
    /// 执行输出 (可选)
    pub output: Option<serde_json::Value>,
    /// 错误信息 (可选)
    pub error_msg: Option<String>,
    /// 错误码 (可选)
    pub error_code: Option<String>,
    /// 执行时长 (秒)
    pub duration_sec: u64,
}

/// **Log**(实时日志, retention 7d, per BATCH-REQ-001 §3.7 Work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    /// 日志唯一标识
    pub log_id: uuid::Uuid,
    /// 所属节点
    pub node_id: NodeId,
    /// 所属 Run
    pub run_id: RunId,
    /// 日志流类型
    pub stream: LogStream,
    /// 日志内容
    pub content: String,
    /// 日志时间戳
    pub ts: DateTime<Utc>,
}

/// 日志流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStream {
    /// 标准输出
    Stdout,
    /// 标准错误
    Stderr,
    /// 系统日志
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
    /// 起始字节偏移
    pub byte_offset: u64,
    /// 单次拉取最大字节数
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
    /// 本次返回的日志条目
    pub logs: Vec<Log>,
    /// 下一次拉取的偏移量 (可选)
    pub next_offset: Option<u64>,
    /// 是否已拉取完整
    pub is_complete: bool,
}

// =====================================================================
// Transaction (append-only 永久)
// =====================================================================

/// **Event**(事件流水, 永久保留, 冷热分层 per R-9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 事件唯一标识
    pub id: EventId,
    /// 关联的 Run (可选)
    pub run_id: Option<RunId>,
    /// 关联的任务 (可选)
    pub task_id: Option<TaskId>,
    /// 事件类型
    pub kind: BatchEventKind,
    /// 事件负载
    pub payload: serde_json::Value,
    /// 触发者 (user_id 或 "system")
    pub actor: String, // user_id 或 "system"
    /// 事件时间戳
    pub ts: DateTime<Utc>,
    /// 因果链上游事件 ID
    pub causation_id: Option<EventId>, // per §D28 守门 5
    /// 关联链事件 ID
    pub correlation_id: Option<EventId>, // per §D28 守门 5
}

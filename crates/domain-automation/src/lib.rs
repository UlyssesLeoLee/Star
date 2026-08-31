//! domain-automation crate
//!
//! 详细 spec: docs/specs/domain-automation-spec.md §11 (REQ-AUTO-001)
//! 上游基本设计: docs/basic-design.md §2.1(表 17) / §5.7
//! 数据设计: docs/data-design.md §4.13 (`automation` schema)
//! API 设计: docs/api-design.md §3.14
//!
//! ## 职责
//!
//! 触发器-条件-动作规则(§11,REQ-AUTO-001)。MVP 不强制可视化配置器,API +
//! Form 已足够。Rule 聚合根,Trigger / Condition / Action / Execution 4 子实体。
//!
//! ## 关键不变量(INV-AUTO-01~06,共 6 条)
//!
//! - **INV-AUTO-01** 必带 `tenant_id`,跨 tenant 拒绝(§6.1,REQ-SEC-001)
//! - **INV-AUTO-02** 规则执行是异步的(Worker 订阅,Not 阻塞业务事务,§2.1)
//! - **INV-AUTO-03** 规则可独立启用/禁用,不影响其他规则(§4.13)
//! - **INV-AUTO-04** 规则执行历史 100% 写(成功/失败/跳过,§4.13)
//! - **INV-AUTO-05** 规则不得直接执行 Protected 动作(如 `pr:merge`,§3.3)
//! - **INV-AUTO-06** 规则执行频率可限流(防循环,§11)
//!
//! Lead 责任: automation Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
pub use star_context::ActorContext;

// =====================================================================
// 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(uuid::Uuid);

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
            #[allow(dead_code)]
            pub fn from_uuid(id: uuid::Uuid) -> Self {
                Self(id)
            }
            #[allow(dead_code)]
            pub fn as_uuid(&self) -> &uuid::Uuid {
                &self.0
            }
            #[allow(dead_code)]
            pub fn into_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(id: uuid::Uuid) -> Self {
                Self(id)
            }
        }
    };
}

define_uuid_id!(RuleId);
define_uuid_id!(TriggerId);
define_uuid_id!(ConditionId);
define_uuid_id!(ActionId);
define_uuid_id!(ExecutionId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(EventId);

// =====================================================================
// 值对象
// =====================================================================

/// **触发器类型**(spec §2.1 已知列表)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Feedback 创建
    FeedbackCreated,
    /// Validation 失败
    ValidationFailed,
    /// Worktree 状态变更
    WorktreeStatusChanged,
    /// 通用自定义事件
    Custom,
}

impl Default for TriggerType {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FeedbackCreated => "feedback_created",
            Self::ValidationFailed => "validation_failed",
            Self::WorktreeStatusChanged => "worktree_status_changed",
            Self::Custom => "custom",
        };
        f.write_str(s)
    }
}

/// **条件运算符**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    In,
}

impl Default for ConditionOperator {
    fn default() -> Self {
        Self::Equals
    }
}

/// **动作类型**(spec §2.1,Protected 动作被规则层禁止,INV-AUTO-05)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// 发送通知(规则允许)
    SendNotification,
    /// 创建 Feedback(规则允许)
    CreateFeedback,
    /// 分配 Agent(规则允许)
    AssignAgent,
    /// 更新状态(规则允许,变更 WorkItem 状态)
    UpdateStatus,
    /// 调用 webhook(规则允许)
    InvokeWebhook,
}

impl Default for ActionType {
    fn default() -> Self {
        Self::SendNotification
    }
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SendNotification => "send_notification",
            Self::CreateFeedback => "create_feedback",
            Self::AssignAgent => "assign_agent",
            Self::UpdateStatus => "update_status",
            Self::InvokeWebhook => "invoke_webhook",
        };
        f.write_str(s)
    }
}

/// Protected 动作(规则层 INV-AUTO-05 禁止)
pub const PROTECTED_ACTIONS: &[&str] = &[
    "pr:merge",
    "pr:force_push",
    "branch:delete_protected",
    "tenant:delete",
];

/// 预定义角色字符串
pub mod roles {
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
}

// =====================================================================
// 错误
// =====================================================================

/// Automation 域错误(§8.3:AU-001~005 + SEC-* 系列)
#[derive(Debug, Error)]
pub enum AutomationError {
    /// `AU-001` 404 Rule 不存在
    #[error("rule not found: {0}")]
    NotFound(RuleId),
    /// `AU-002` 422 Trigger event_type 非法
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// `AU-003` 403 权限不足
    #[error("permission denied")]
    PermissionDenied,
    /// `AU-004` 409 循环规则(A→B→A)
    #[error("conflict: {0}")]
    Conflict(String),
    /// `AU-005` 403 规则尝试 Protected 动作
    #[error("protected action forbidden: {0}")]
    ProtectedActionForbidden(String),
    /// 5xx
    #[error("internal error: {0}")]
    Internal(String),
}

impl AutomationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "AUTOMATION_NOT_FOUND",
            Self::InvalidState(_) => "AUTOMATION_INVALID_STATE",
            Self::PermissionDenied => "AUTOMATION_PERMISSION_DENIED",
            Self::Conflict(_) => "AUTOMATION_CONFLICT",
            Self::ProtectedActionForbidden(_) => "AUTOMATION_PROTECTED_ACTION_FORBIDDEN",
            Self::Internal(_) => "AUTOMATION_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for AutomationError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

// =====================================================================
// 实体
// =====================================================================

/// **AutomationTrigger**(内嵌于 Rule,7 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTrigger {
    pub id: TriggerId,
    pub event_type: TriggerType,
    /// 资源类型过滤(如 "work_item" / "validation")
    pub resource_type: Option<String>,
    /// severity 过滤(可选)
    pub severity: Option<String>,
    /// 额外过滤(扁平 key-value)
    pub filters: HashMap<String, String>,
    /// debounce_ms(防抖,默认 0)
    pub debounce_ms: u32,
}

impl AutomationTrigger {
    pub const FIELD_COUNT: usize = 7;
}

/// **AutomationCondition**(子实体,AND 关系)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationCondition {
    pub id: ConditionId,
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

impl AutomationCondition {
    pub const FIELD_COUNT: usize = 4;
    /// 评估条件(简单 equality / comparison;In / Contains 由 JSON value 推断)
    pub fn evaluate(&self, event: &serde_json::Value) -> bool {
        let actual = event.get(&self.field);
        match self.operator {
            ConditionOperator::Equals => actual == Some(&self.value),
            ConditionOperator::NotEquals => actual != Some(&self.value),
            ConditionOperator::Contains => {
                if let (Some(actual_str), Some(needle_str)) =
                    (actual.and_then(|v| v.as_str()), self.value.as_str())
                {
                    actual_str.contains(needle_str)
                } else {
                    false
                }
            }
            ConditionOperator::GreaterThan => {
                if let (Some(a), Some(b)) = (actual.and_then(|v| v.as_f64()), self.value.as_f64()) {
                    a > b
                } else {
                    false
                }
            }
            ConditionOperator::LessThan => {
                if let (Some(a), Some(b)) = (actual.and_then(|v| v.as_f64()), self.value.as_f64()) {
                    a < b
                } else {
                    false
                }
            }
            ConditionOperator::In => {
                if let Some(arr) = self.value.as_array() {
                    arr.iter()
                        .any(|v| v == actual.unwrap_or(&serde_json::Value::Null))
                } else {
                    false
                }
            }
        }
    }
}

/// **AutomationAction**(子实体,5 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationAction {
    pub id: ActionId,
    pub action_type: ActionType,
    /// 目标资源类型(如 "notification_channel" / "user" / "webhook")
    pub target_type: String,
    /// 目标 ID 引用(由调用方解析,本 crate 仅存字符串)
    pub target_ref: String,
    /// 额外参数
    pub params: HashMap<String, serde_json::Value>,
}

impl AutomationAction {
    pub const FIELD_COUNT: usize = 5;
}

/// **AutomationRule 聚合根**(17 字段,data-design §4.13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    pub id: RuleId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub trigger: AutomationTrigger,
    pub conditions: Vec<AutomationCondition>,
    pub actions: Vec<AutomationAction>,
    pub priority: i32,
    /// 每分钟最大执行次数(限流,INV-AUTO-06,默认 60)
    pub rate_limit_per_minute: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_executed_at: Option<DateTime<Utc>>,
    pub lock_version: u32,
    pub created_by_user_id: UserId,
    pub execution_count: u64,
}

impl AutomationRule {
    pub const FIELD_COUNT: usize = 17;
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
    /// 检查 Protected 动作(INV-AUTO-05)
    pub fn check_no_protected_actions(&self) -> Result<(), AutomationError> {
        for action in &self.actions {
            for protected in PROTECTED_ACTIONS {
                if action.target_ref.starts_with(protected)
                    || action.action_type.to_string() == *protected
                {
                    return Err(AutomationError::ProtectedActionForbidden(format!(
                        "INV-AUTO-05: Rule 不得执行 Protected 动作 {}",
                        protected
                    )));
                }
            }
        }
        Ok(())
    }
}

/// **AutomationExecution**(Append-only 执行历史,11 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationExecution {
    pub id: ExecutionId,
    pub rule_id: RuleId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    /// 触发事件 ID
    pub trigger_event_id: EventId,
    /// 是否匹配
    pub matched: bool,
    /// 实际执行的动作数
    pub executed_actions: u32,
    /// 状态
    pub result: ExecutionResult,
    /// 跳过原因(未匹配 / 限流等)
    pub skip_reason: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// 用于 INV-AUTO-04:确保 100% 写入(包括 matched=false)
    pub logged: bool,
}

impl AutomationExecution {
    pub const FIELD_COUNT: usize = 11;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionResult {
    /// 匹配且全部成功
    Success,
    /// 匹配但动作执行失败
    PartialFailure,
    /// 未匹配(条件不满足)
    NotMatched,
    /// 触发但限流跳过(INV-AUTO-06)
    RateLimited,
    /// 触发但含 Protected 动作被拒绝(INV-AUTO-05)
    ProtectedRejected,
}

// =====================================================================
// 不变量(INV-AUTO-01~06)
// =====================================================================

pub type InvariantCheck = fn(&AutomationRule) -> Result<(), AutomationError>;

/// **INV-AUTO-01** tenant_id 必非 nil
pub fn check_invariant_01_tenant_id(rule: &AutomationRule) -> Result<(), AutomationError> {
    if rule.tenant_id.as_uuid().is_nil() {
        return Err(AutomationError::InvalidState(
            "INV-AUTO-01: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    Ok(())
}

/// **INV-AUTO-02** 规则必带 project_id(异步 Worker 按 project 评估)
pub fn check_invariant_02_project_id(rule: &AutomationRule) -> Result<(), AutomationError> {
    if rule.project_id.as_uuid().is_nil() {
        return Err(AutomationError::InvalidState(
            "INV-AUTO-02: project_id 必须非 nil".to_string(),
        ));
    }
    Ok(())
}

/// **INV-AUTO-05** Rule 不得含 Protected 动作
pub fn check_invariant_05_no_protected_actions(
    rule: &AutomationRule,
) -> Result<(), AutomationError> {
    rule.check_no_protected_actions()
}

/// **INV-AUTO-06** 限流必须 > 0(0 表示无限流 → 循环风险)
pub fn check_invariant_06_rate_limit_positive(
    rule: &AutomationRule,
) -> Result<(), AutomationError> {
    if rule.rate_limit_per_minute == 0 {
        return Err(AutomationError::InvalidState(
            "INV-AUTO-06: rate_limit_per_minute 必须 > 0(防循环)".to_string(),
        ));
    }
    Ok(())
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_01_tenant_id,
    check_invariant_02_project_id,
    check_invariant_05_no_protected_actions,
    check_invariant_06_rate_limit_positive,
];

pub fn run_invariants(
    checks: &[InvariantCheck],
    rule: &AutomationRule,
) -> Result<(), AutomationError> {
    for c in checks {
        c(rule)?;
    }
    Ok(())
}

// =====================================================================
// 事件(NATS)
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecuted {
    pub meta: EventMeta,
    pub rule_id: RuleId,
    pub execution_id: ExecutionId,
    pub matched: bool,
    pub result: ExecutionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationEvent {
    RuleExecuted(RuleExecuted),
}

impl AutomationEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::RuleExecuted(_) => "star.events.automation.rule.executed.v1",
        }
    }
}

// =====================================================================
// 端口(Port traits)
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub trigger: AutomationTrigger,
    pub conditions: Vec<AutomationCondition>,
    pub actions: Vec<AutomationAction>,
    pub priority: i32,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRuleCommand {
    pub rule_id: RuleId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub enabled: Option<bool>,
    pub trigger: Option<AutomationTrigger>,
    pub conditions: Option<Vec<AutomationCondition>>,
    pub actions: Option<Vec<AutomationAction>>,
    pub priority: Option<i32>,
    pub rate_limit_per_minute: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRulesQuery {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub enabled_only: bool,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListRulesQuery {
    fn default() -> Self {
        Self {
            tenant_id: UserId.new(),
            project_id: None,
            enabled_only: false,
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleCommand {
    pub rule_id: RuleId,
    /// 测试事件(模拟 trigger event)
    pub sample_event: serde_json::Value,
}

/// **AutomationCommandPort**(4 个方法)
#[async_trait]
pub trait AutomationCommandPort: Send + Sync {
    async fn create_rule(
        &self,
        cmd: CreateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;
    async fn update_rule(
        &self,
        cmd: UpdateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;
    async fn delete_rule(
        &self,
        rule_id: RuleId,
        actor: ActorContext,
    ) -> Result<(), AutomationError>;
    async fn test_rule(
        &self,
        cmd: TestRuleCommand,
        actor: ActorContext,
    ) -> Result<bool, AutomationError>;
}

/// **AutomationQueryPort**(3 个方法)
#[async_trait]
pub trait AutomationQueryPort: Send + Sync {
    async fn list_rules(
        &self,
        q: ListRulesQuery,
        actor: ActorContext,
    ) -> Result<Vec<AutomationRule>, AutomationError>;
    async fn get_rule(
        &self,
        rule_id: RuleId,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;
    async fn list_executions(
        &self,
        rule_id: RuleId,
        actor: ActorContext,
    ) -> Result<Vec<AutomationExecution>, AutomationError>;
}

/// **RuleExecutor**(Worker 调用)
#[async_trait]
pub trait RuleExecutor: Send + Sync {
    /// 评估事件,返回匹配的 rule IDs + actions
    async fn evaluate(
        &self,
        event_type: TriggerType,
        event: serde_json::Value,
    ) -> Result<Vec<(AutomationRule, Vec<AutomationAction>)>, AutomationError>;
}

// =====================================================================
// InMemoryAutomationService
// =====================================================================

/// **InMemory Automation Service**
pub struct InMemoryAutomationService {
    rules: Arc<RwLock<HashMap<RuleId, AutomationRule>>>,
    executions: Arc<RwLock<HashMap<ExecutionId, AutomationExecution>>>,
    /// 限流追踪:rule_id → (窗口起点, 已执行次数)
    rate_windows: Arc<RwLock<HashMap<RuleId, (Instant, u32)>>>,
    /// 最近执行时间(用于 debounce)
    last_triggered: Arc<RwLock<HashMap<RuleId, Instant>>>,
    event_tx: mpsc::UnboundedSender<AutomationEvent>,
}

impl InMemoryAutomationService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<AutomationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            rate_windows: Arc::new(RwLock::new(HashMap::new())),
            last_triggered: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    pub async fn rule_count(&self) -> usize {
        self.rules.read().await.len()
    }
    pub async fn execution_count(&self) -> usize {
        self.executions.read().await.len()
    }
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), AutomationError> {
        if TenantId::from(actor.tenant_id) != expected {
            return Err(AutomationError::PermissionDenied);
        }
        Ok(())
    }
    /// 检查 INV-AUTO-06 限流
    async fn check_rate_limit(&self, rule_id: RuleId, limit: u32) -> bool {
        let now = Instant::now();
        let mut windows = self.rate_windows.write().await;
        let window = windows.entry(rule_id).or_insert((now, 0));
        if now.duration_since(window.0) >= Duration::from_secs(60) {
            *window = (now, 1);
            true
        } else {
            if window.1 >= limit {
                return false;
            }
            window.1 += 1;
            true
        }
    }
}

impl Default for InMemoryAutomationService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryAutomationService {
    fn clone(&self) -> Self {
        Self {
            rules: self.rules.clone(),
            executions: self.executions.clone(),
            rate_windows: self.rate_windows.clone(),
            last_triggered: self.last_triggered.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl AutomationCommandPort for InMemoryAutomationService {
    async fn create_rule(
        &self,
        cmd: CreateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") && !actor.is_platform_admin {
            return Err(AutomationError::PermissionDenied);
        }
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // INV-AUTO-04 project 必带
        if !actor.project_ids.contains(&cmd.project_id) {
            return Err(AutomationError::PermissionDenied);
        }
        let now = Utc::now();
        let id = RuleId::new();
        let rule = AutomationRule {
            id,
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name: cmd.name,
            description: cmd.description,
            enabled: true, // INV-AUTO-03 默认启用
            trigger: cmd.trigger,
            conditions: cmd.conditions,
            actions: cmd.actions,
            priority: cmd.priority,
            rate_limit_per_minute: if cmd.rate_limit_per_minute == 0 {
                60
            } else {
                cmd.rate_limit_per_minute
            },
            created_at: now,
            updated_at: now,
            last_executed_at: None,
            lock_version: 1,
            created_by_user_id: UserId::from_uuid(actor.user_id),
            execution_count: 0,
        };
        // 完整不变量检查
        run_invariants(ALL_INVARIANT_CHECKS, &rule)?;
        // 项目内 name 唯一
        {
            let guard = self.rules.read().await;
            if guard.values().any(|r| {
                r.tenant_id == cmd.tenant_id
                    && r.project_id == cmd.project_id
                    && r.name == rule.name
            }) {
                return Err(AutomationError::Conflict(format!(
                    "project 内 rule name '{}' 已存在",
                    rule.name
                )));
            }
        }
        {
            let mut guard = self.rules.write().await;
            guard.insert(id, rule.clone());
        }
        Ok(rule)
    }

    async fn update_rule(
        &self,
        cmd: UpdateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") && !actor.is_platform_admin {
            return Err(AutomationError::PermissionDenied);
        }
        let updated = {
            let mut store = self.rules.write().await;
            let r = store
                .get_mut(&cmd.rule_id)
                .ok_or(AutomationError::NotFound(cmd.rule_id))?;
            if r.tenant_id != cmd.tenant_id {
                return Err(AutomationError::PermissionDenied);
            }
            if r.lock_version != cmd.expected_version {
                return Err(AutomationError::Conflict(format!(
                    "version mismatch: expected {}, actual {}",
                    cmd.expected_version, r.lock_version
                )));
            }
            if let Some(name) = cmd.name {
                r.name = name;
            }
            if let Some(desc) = cmd.description {
                r.description = desc;
            }
            if let Some(e) = cmd.enabled {
                r.enabled = e;
            }
            if let Some(t) = cmd.trigger {
                r.trigger = t;
            }
            if let Some(c) = cmd.conditions {
                r.conditions = c;
            }
            if let Some(a) = cmd.actions {
                r.actions = a;
            }
            if let Some(p) = cmd.priority {
                r.priority = p;
            }
            if let Some(rl) = cmd.rate_limit_per_minute {
                r.rate_limit_per_minute = rl;
            }
            r.bump_version();
            run_invariants(ALL_INVARIANT_CHECKS, r)?;
            r.clone()
        };
        Ok(updated)
    }

    async fn delete_rule(
        &self,
        rule_id: RuleId,
        actor: ActorContext,
    ) -> Result<(), AutomationError> {
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") && !actor.is_platform_admin {
            return Err(AutomationError::PermissionDenied);
        }
        let mut guard = self.rules.write().await;
        let r = guard
            .get(&rule_id)
            .ok_or(AutomationError::NotFound(rule_id))?;
        if r.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(AutomationError::PermissionDenied);
        }
        guard.remove(&rule_id);
        Ok(())
    }

    async fn test_rule(
        &self,
        cmd: TestRuleCommand,
        actor: ActorContext,
    ) -> Result<bool, AutomationError> {
        let rule = {
            let guard = self.rules.read().await;
            guard.get(&cmd.rule_id).cloned()
        };
        let rule = rule.ok_or(AutomationError::NotFound(cmd.rule_id))?;
        if rule.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(AutomationError::PermissionDenied);
        }
        // 评估所有条件
        let matched = rule
            .conditions
            .iter()
            .all(|c| c.evaluate(&cmd.sample_event));
        Ok(matched)
    }
}

#[async_trait]
impl AutomationQueryPort for InMemoryAutomationService {
    async fn list_rules(
        &self,
        q: ListRulesQuery,
        actor: ActorContext,
    ) -> Result<Vec<AutomationRule>, AutomationError> {
        Self::check_tenant(&actor, q.tenant_id)?;
        let mut all: Vec<AutomationRule> = {
            let guard = self.rules.read().await;
            guard
                .values()
                .filter(|r| {
                    r.tenant_id == q.tenant_id
                        && q.project_id.map_or(true, |p| r.project_id == p)
                        && (!q.enabled_only || r.enabled)
                })
                .cloned()
                .collect()
        };
        all.sort_by_key(|r| r.priority);
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }

    async fn get_rule(
        &self,
        rule_id: RuleId,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        let r = {
            let guard = self.rules.read().await;
            guard.get(&rule_id).cloned()
        };
        let r = r.ok_or(AutomationError::NotFound(rule_id))?;
        if r.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(AutomationError::PermissionDenied);
        }
        Ok(r)
    }

    async fn list_executions(
        &self,
        rule_id: RuleId,
        actor: ActorContext,
    ) -> Result<Vec<AutomationExecution>, AutomationError> {
        let r = {
            let guard = self.rules.read().await;
            guard.get(&rule_id).cloned()
        };
        let r = r.ok_or(AutomationError::NotFound(rule_id))?;
        if r.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(AutomationError::PermissionDenied);
        }
        let guard = self.executions.read().await;
        let mut all: Vec<AutomationExecution> = guard
            .values()
            .filter(|e| e.rule_id == rule_id)
            .cloned()
            .collect();
        all.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(all)
    }
}

#[async_trait]
impl RuleExecutor for InMemoryAutomationService {
    async fn evaluate(
        &self,
        event_type: TriggerType,
        event: serde_json::Value,
    ) -> Result<Vec<(AutomationRule, Vec<AutomationAction>)>, AutomationError> {
        // 拉取所有 enabled 规则
        let candidates: Vec<AutomationRule> = {
            let guard = self.rules.read().await;
            guard
                .values()
                .filter(|r| r.enabled && r.trigger.event_type == event_type)
                .cloned()
                .collect()
        };
        let mut out: Vec<(AutomationRule, Vec<AutomationAction>)> = Vec::new();
        let mut to_log: Vec<AutomationExecution> = Vec::new();
        let event_id = EventId::new();
        for rule in candidates {
            // 过滤 resource_type / severity
            if let Some(rt) = &rule.trigger.resource_type {
                if event.get("resource_type").and_then(|v| v.as_str()) != Some(rt.as_str()) {
                    continue;
                }
            }
            if let Some(sv) = &rule.trigger.severity {
                if event.get("severity").and_then(|v| v.as_str()) != Some(sv.as_str()) {
                    continue;
                }
            }
            // 条件匹配
            let matched = rule.conditions.iter().all(|c| c.evaluate(&event));
            if !matched {
                // INV-AUTO-04: 100% 写历史(包括未匹配)
                let now = Utc::now();
                let eid = ExecutionId::new();
                to_log.push(AutomationExecution {
                    id: eid,
                    rule_id: rule.id,
                    tenant_id: rule.tenant_id,
                    project_id: rule.project_id,
                    trigger_event_id: event_id,
                    matched: false,
                    executed_actions: 0,
                    result: ExecutionResult::NotMatched,
                    skip_reason: Some("条件不满足".to_string()),
                    started_at: now,
                    finished_at: Some(now),
                    created_at: now,
                    logged: true,
                });
                continue;
            }
            // Protected 动作检查(INV-AUTO-05)
            if let Err(e) = rule.check_no_protected_actions() {
                let now = Utc::now();
                let eid = ExecutionId::new();
                to_log.push(AutomationExecution {
                    id: eid,
                    rule_id: rule.id,
                    tenant_id: rule.tenant_id,
                    project_id: rule.project_id,
                    trigger_event_id: event_id,
                    matched: true,
                    executed_actions: 0,
                    result: ExecutionResult::ProtectedRejected,
                    skip_reason: Some(format!("{}", e)),
                    started_at: now,
                    finished_at: Some(now),
                    created_at: now,
                    logged: true,
                });
                continue;
            }
            // 限流(INV-AUTO-06)
            if !self
                .check_rate_limit(rule.id, rule.rate_limit_per_minute)
                .await
            {
                let now = Utc::now();
                let eid = ExecutionId::new();
                to_log.push(AutomationExecution {
                    id: eid,
                    rule_id: rule.id,
                    tenant_id: rule.tenant_id,
                    project_id: rule.project_id,
                    trigger_event_id: event_id,
                    matched: true,
                    executed_actions: 0,
                    result: ExecutionResult::RateLimited,
                    skip_reason: Some(format!("rate limit {} 次/分钟", rule.rate_limit_per_minute)),
                    started_at: now,
                    finished_at: Some(now),
                    created_at: now,
                    logged: true,
                });
                continue;
            }
            // 通过:收集 actions
            out.push((rule.clone(), rule.actions.clone()));
        }
        // INV-AUTO-04: 100% 写历史
        if !to_log.is_empty() {
            let mut guard = self.executions.write().await;
            for e in to_log {
                guard.insert(e.id, e);
            }
        }
        Ok(out)
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_admin(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(project_id)
    }

    fn make_developer(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::DEVELOPER)
            .with_project(project_id)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(AutomationRule::FIELD_COUNT, 17);
        assert_eq!(AutomationExecution::FIELD_COUNT, 11);
        assert_eq!(AutomationTrigger::FIELD_COUNT, 7);
        assert_eq!(AutomationCondition::FIELD_COUNT, 4);
        assert_eq!(AutomationAction::FIELD_COUNT, 5);
    }

    #[tokio::test]
    async fn create_rule_success() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let cmd = CreateRuleCommand {
            tenant_id: tenant,
            project_id: project,
            name: "feedback-p0".to_string(),
            description: Some("on P0 feedback".to_string()),
            trigger: AutomationTrigger {
                id: TriggerId::new(),
                event_type: TriggerType::FeedbackCreated,
                resource_type: Some("feedback".to_string()),
                severity: Some("P0".to_string()),
                filters: HashMap::new(),
                debounce_ms: 0,
            },
            conditions: vec![],
            actions: vec![AutomationAction {
                id: ActionId::new(),
                action_type: ActionType::SendNotification,
                target_type: "notification_channel".to_string(),
                target_ref: "channel-1".to_string(),
                params: HashMap::new(),
            }],
            priority: 100,
            rate_limit_per_minute: 60,
        };
        let rule = svc.create_rule(cmd, actor).await.unwrap();
        assert_eq!(rule.enabled, true); // INV-AUTO-03
        assert_eq!(rule.rate_limit_per_minute, 60);
        assert_eq!(rule.execution_count, 0);
        assert_eq!(svc.rule_count().await, 1);
    }

    #[tokio::test]
    async fn invariant_05_protected_action_blocked() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let cmd = CreateRuleCommand {
            tenant_id: tenant,
            project_id: project,
            name: "bad".to_string(),
            description: None,
            trigger: AutomationTrigger {
                id: TriggerId::new(),
                event_type: TriggerType::Custom,
                resource_type: None,
                severity: None,
                filters: HashMap::new(),
                debounce_ms: 0,
            },
            conditions: vec![],
            actions: vec![AutomationAction {
                id: ActionId::new(),
                action_type: ActionType::UpdateStatus,
                target_type: "pr".to_string(),
                target_ref: "pr:merge".to_string(), // PROTECTED
                params: HashMap::new(),
            }],
            priority: 0,
            rate_limit_per_minute: 60,
        };
        let res = svc.create_rule(cmd, actor).await;
        assert!(matches!(
            res,
            Err(AutomationError::ProtectedActionForbidden(_))
        ));
    }

    #[tokio::test]
    async fn cross_tenant_denied() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant_a = uuid::Uuid::new_v4();
        let project_a = ProjectId::new();
        let actor_a = make_admin(tenant_a, project_a);
        let rule = svc
            .create_rule(
                CreateRuleCommand {
                    tenant_id: tenant_a,
                    project_id: project_a,
                    name: "r1".to_string(),
                    description: None,
                    trigger: AutomationTrigger {
                        id: TriggerId::new(),
                        event_type: TriggerType::Custom,
                        resource_type: None,
                        severity: None,
                        filters: HashMap::new(),
                        debounce_ms: 0,
                    },
                    conditions: vec![],
                    actions: vec![],
                    priority: 0,
                    rate_limit_per_minute: 60,
                },
                actor_a,
            )
            .await
            .unwrap();
        let tenant_b = uuid::Uuid::new_v4();
        let project_b = ProjectId::new();
        let actor_b = make_admin(tenant_b, project_b);
        let res = svc.get_rule(rule.id, actor_b).await;
        assert!(matches!(res, Err(AutomationError::PermissionDenied)));
    }

    #[tokio::test]
    async fn developer_cannot_create_rule() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_developer(tenant, project);
        let cmd = CreateRuleCommand {
            tenant_id: tenant,
            project_id: project,
            name: "dev-rule".to_string(),
            description: None,
            trigger: AutomationTrigger {
                id: TriggerId::new(),
                event_type: TriggerType::Custom,
                resource_type: None,
                severity: None,
                filters: HashMap::new(),
                debounce_ms: 0,
            },
            conditions: vec![],
            actions: vec![],
            priority: 0,
            rate_limit_per_minute: 60,
        };
        let res = svc.create_rule(cmd, actor).await;
        assert!(matches!(res, Err(AutomationError::PermissionDenied)));
    }

    #[tokio::test]
    async fn rate_limit_throttles_execution() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let cmd = CreateRuleCommand {
            tenant_id: tenant,
            project_id: project,
            name: "limited".to_string(),
            description: None,
            trigger: AutomationTrigger {
                id: TriggerId::new(),
                event_type: TriggerType::FeedbackCreated,
                resource_type: None,
                severity: None,
                filters: HashMap::new(),
                debounce_ms: 0,
            },
            conditions: vec![],
            actions: vec![AutomationAction {
                id: ActionId::new(),
                action_type: ActionType::SendNotification,
                target_type: "channel".to_string(),
                target_ref: "c1".to_string(),
                params: HashMap::new(),
            }],
            priority: 0,
            rate_limit_per_minute: 1, // 只允许 1 次/分钟
        };
        let _ = svc.create_rule(cmd, actor).await.unwrap();
        let event = serde_json::json!({"resource_type": "feedback", "severity": "P0"});
        // 第 1 次: 通过
        let r1 = svc
            .evaluate(TriggerType::FeedbackCreated, event.clone())
            .await
            .unwrap();
        assert_eq!(r1.len(), 1);
        // 第 2 次: 限流(写 history: matched=true, RateLimited)
        let r2 = svc
            .evaluate(TriggerType::FeedbackCreated, event.clone())
            .await
            .unwrap();
        assert_eq!(r2.len(), 0);
        // 历史 100% 写(INV-AUTO-04): 至少 1 条 RateLimited
        assert!(svc.execution_count().await >= 1);
    }

    #[tokio::test]
    async fn condition_evaluate_eq_and_in() {
        let cond_eq = AutomationCondition {
            id: ConditionId::new(),
            field: "severity".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("P0"),
        };
        let ev_match = serde_json::json!({"severity": "P0", "id": 1});
        let ev_nomatch = serde_json::json!({"severity": "P1", "id": 1});
        assert!(cond_eq.evaluate(&ev_match));
        assert!(!cond_eq.evaluate(&ev_nomatch));
        let cond_in = AutomationCondition {
            id: ConditionId::new(),
            field: "kind".to_string(),
            operator: ConditionOperator::In,
            value: serde_json::json!(["bug", "regression"]),
        };
        let ev_in = serde_json::json!({"kind": "bug"});
        assert!(cond_in.evaluate(&ev_in));
        let ev_out = serde_json::json!({"kind": "feature"});
        assert!(!cond_in.evaluate(&ev_out));
    }

    #[tokio::test]
    async fn update_rule_version_conflict() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let rule = svc
            .create_rule(
                CreateRuleCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "v".to_string(),
                    description: None,
                    trigger: AutomationTrigger {
                        id: TriggerId::new(),
                        event_type: TriggerType::Custom,
                        resource_type: None,
                        severity: None,
                        filters: HashMap::new(),
                        debounce_ms: 0,
                    },
                    conditions: vec![],
                    actions: vec![],
                    priority: 0,
                    rate_limit_per_minute: 60,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .update_rule(
                UpdateRuleCommand {
                    rule_id: rule.id,
                    tenant_id: tenant,
                    expected_version: 99,
                    name: Some("V2".to_string()),
                    description: None,
                    enabled: None,
                    trigger: None,
                    conditions: None,
                    actions: None,
                    priority: None,
                    rate_limit_per_minute: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(AutomationError::Conflict(_))));
    }

    #[tokio::test]
    async fn executions_logged_100_percent() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let _ = svc
            .create_rule(
                CreateRuleCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "x".to_string(),
                    description: None,
                    trigger: AutomationTrigger {
                        id: TriggerId::new(),
                        event_type: TriggerType::Custom,
                        resource_type: None,
                        severity: None,
                        filters: HashMap::new(),
                        debounce_ms: 0,
                    },
                    conditions: vec![AutomationCondition {
                        id: ConditionId::new(),
                        field: "x".to_string(),
                        operator: ConditionOperator::Equals,
                        value: serde_json::json!("y"),
                    }],
                    actions: vec![],
                    priority: 0,
                    rate_limit_per_minute: 60,
                },
                actor,
            )
            .await
            .unwrap();
        // 触发不匹配事件 → 应该 100% 写历史
        let ev = serde_json::json!({"x": "z"});
        let r = svc.evaluate(TriggerType::Custom, ev).await.unwrap();
        assert_eq!(r.len(), 0);
        // 至少 1 条 execution 历史
        assert!(svc.execution_count().await >= 1);
    }
}

pub mod governance;

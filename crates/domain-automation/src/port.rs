//! Automation 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.14 (Automation endpoints)
//! - `docs/specs/domain-automation-spec.md` §4 (接口签名)
//!
//! **端口清单**:
//! - `AutomationCommandPort`: 7 方法(写)
//! - `AutomationQueryPort`: 4 方法(读)
//! - `AutomationRepository`: 基础设施层使用,本文件声明 trait
//! - `RuleExecutor`: Worker 调用,执行规则

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{AutomationAction, AutomationCondition, AutomationRule, AutomationExecution, AutomationTrigger};
use crate::error::AutomationError;
use crate::value_object::{
    ActionType, EventId, ExecutionId, ProjectId, RuleId, TenantId, TriggerType,
};

// =====================================================================
// 命令 DTO(写操作输入)
// =====================================================================

/// 单 Condition 在创建时提供的草稿(不包含 id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionDraft {
    /// 字段路径(如 `payload.severity`)
    pub field: String,
    /// 操作符
    pub operator: crate::value_object::ConditionOperator,
    /// 比较值
    pub value: serde_json::Value,
}

impl ConditionDraft {
    /// 落地为完整 `AutomationCondition`
    pub fn into_entity(self) -> AutomationCondition {
        AutomationCondition {
            id: crate::value_object::ConditionId::new(),
            field: self.field,
            operator: self.operator,
            value: self.value,
        }
    }
}

/// 单 Action 在创建时提供的草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDraft {
    /// 动作类型
    pub action_type: ActionType,
    /// 动作参数
    pub params: HashMap<String, serde_json::Value>,
    /// 顺序
    pub order: u32,
}

impl ActionDraft {
    /// 落地为完整 `AutomationAction`
    pub fn into_entity(self) -> AutomationAction {
        AutomationAction {
            id: crate::value_object::ActionId::new(),
            action_type: self.action_type,
            params: self.params,
            order: self.order,
            enabled: true,
        }
    }
}

/// 单 Trigger 草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDraft {
    /// 事件类型字符串
    pub event_type: String,
    /// 触发器类型
    pub trigger_type: TriggerType,
    /// 过滤条件
    pub filter: HashMap<String, serde_json::Value>,
    /// 是否启用
    pub enabled: bool,
}

impl TriggerDraft {
    /// 落地为完整 `AutomationTrigger`
    pub fn into_entity(self) -> AutomationTrigger {
        AutomationTrigger {
            id: crate::value_object::TriggerId::new(),
            event_type: self.event_type,
            trigger_type: self.trigger_type,
            filter: self.filter,
            enabled: self.enabled,
        }
    }
}

/// `CreateRuleCommand`(创建 Rule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// Trigger 草稿
    pub trigger: TriggerDraft,
    /// Condition 草稿列表
    pub conditions: Vec<ConditionDraft>,
    /// Action 草稿列表
    pub actions: Vec<ActionDraft>,
    /// 优先级
    pub priority: i32,
    /// 限流(每分钟最大执行次数;0 = 不限)
    pub rate_limit_per_minute: u32,
    /// 是否启用
    pub enabled: bool,
}

/// `UpdateRuleCommand`(更新 Rule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRuleCommand {
    /// Rule ID
    pub rule_id: RuleId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 期望乐观锁版本
    pub expected_version: u32,
    /// 新名称(None = 不改)
    pub name: Option<String>,
    /// 新描述
    pub description: Option<String>,
    /// 新优先级
    pub priority: Option<i32>,
    /// 新限流
    pub rate_limit_per_minute: Option<u32>,
    /// 新 conditions(替换)
    pub conditions: Option<Vec<ConditionDraft>>,
    /// 新 actions(替换)
    pub actions: Option<Vec<ActionDraft>>,
}

/// `TestRuleRequest`(模拟执行)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleRequest {
    /// Rule ID
    pub rule_id: RuleId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 模拟 event_type
    pub sample_event_type: String,
    /// 模拟事件 payload
    pub sample_event_payload: HashMap<String, serde_json::Value>,
}

/// `TestRuleResult`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleResult {
    /// Rule ID
    pub rule_id: RuleId,
    /// 是否匹配
    pub matched: bool,
    /// 匹配原因(命中 trigger? 命中 conditions?)
    pub reason: String,
    /// 若匹配,会执行哪些 action
    pub would_execute_actions: Vec<ActionType>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListRuleQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRuleQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 仅列出 enabled 的
    pub enabled_only: bool,
}

/// `ListExecutionsQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExecutionsQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Rule ID
    pub rule_id: RuleId,
    /// 上限(默认 100)
    pub limit: Option<usize>,
}

/// `FindMatchingRulesQuery`(Worker 调用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindMatchingRulesQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 事件类型
    pub event_type: String,
    /// 事件 payload(用于 trigger.filter 匹配)
    pub event_payload: HashMap<String, serde_json::Value>,
}

// =====================================================================
// 端口:AutomationCommandPort(7 方法)
// =====================================================================

/// **Automation 命令端口**(写操作 7 方法)
#[async_trait]
pub trait AutomationCommandPort: Send + Sync {
    /// 创建 Rule(INV-AUTO-01,02,05,06,07,09,10)
    async fn create_rule(
        &self,
        cmd: CreateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;

    /// 更新 Rule(INV-AUTO-01)
    async fn update_rule(
        &self,
        cmd: UpdateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;

    /// 删除 Rule
    async fn delete_rule(
        &self,
        id: RuleId,
        actor: ActorContext,
    ) -> Result<(), AutomationError>;

    /// 启用 Rule(INV-AUTO-03)
    async fn enable_rule(
        &self,
        id: RuleId,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;

    /// 禁用 Rule(INV-AUTO-03)
    async fn disable_rule(
        &self,
        id: RuleId,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;

    /// 模拟执行 Rule(spec §4 test_rule,无副作用)
    async fn test_rule(
        &self,
        cmd: TestRuleRequest,
        actor: ActorContext,
    ) -> Result<TestRuleResult, AutomationError>;

    /// 记录一次执行(INV-AUTO-04 100% 写历史)
    async fn record_execution(
        &self,
        execution: AutomationExecution,
        actor: ActorContext,
    ) -> Result<ExecutionId, AutomationError>;
}

// =====================================================================
// 端口:AutomationQueryPort(4 方法)
// =====================================================================

/// **Automation 查询端口**(读操作 4 方法)
#[async_trait]
pub trait AutomationQueryPort: Send + Sync {
    /// 按 ID 查询
    async fn get_rule(
        &self,
        id: RuleId,
        viewer: ActorContext,
    ) -> Result<AutomationRule, AutomationError>;

    /// 按 Project 列出 Rule
    async fn list_rules(
        &self,
        q: ListRuleQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AutomationRule>, AutomationError>;

    /// 列出 Rule 的执行历史
    async fn list_executions(
        &self,
        q: ListExecutionsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AutomationExecution>, AutomationError>;

    /// 查找与给定 event_type 匹配的所有 enabled Rule(Worker 调用)
    async fn find_rules_matching_trigger(
        &self,
        q: FindMatchingRulesQuery,
    ) -> Result<Vec<AutomationRule>, AutomationError>;
}

// =====================================================================
// 端口:RuleExecutor(Worker 异步执行规则,spec §4 + §10 AC)
// =====================================================================

/// **Rule Executor**(Worker 调用入口,spec §4 + §10 AC "规则匹配" 场景)
#[async_trait]
pub trait RuleExecutor: Send + Sync {
    /// 评估一个 event,返回匹配且可执行的 Rule + Execution 草稿
    ///
    /// **不直接执行 Action**(本 crate 仅触发,不直接改业务聚合,见 spec §1)。
    /// 实际执行由 application / infrastructure 层根据 Execution 草稿触发。
    async fn evaluate(
        &self,
        event_id: EventId,
        event_type: String,
        event_payload: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<AutomationExecution>, AutomationError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **Automation 仓库端口**
#[async_trait]
pub trait AutomationRepository: Send + Sync {
    /// 插入 Rule
    async fn insert(&self, rule: &AutomationRule) -> Result<(), AutomationError>;
    /// 按 ID 读
    async fn find_by_id(&self, id: RuleId) -> Result<Option<AutomationRule>, AutomationError>;
    /// 更新
    async fn update(&self, rule: &AutomationRule) -> Result<(), AutomationError>;
    /// 删除
    async fn delete(&self, id: RuleId) -> Result<(), AutomationError>;
    /// 列出 Tenant 下全部 Rule
    async fn list_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<AutomationRule>, AutomationError>;
    /// 列出 Project 下全部 Rule
    async fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<AutomationRule>, AutomationError>;

    /// 插入 Execution(append-only)
    async fn insert_execution(&self, e: &AutomationExecution) -> Result<(), AutomationError>;
    /// 列出 Rule 的 Execution(降序)
    async fn list_executions_raw(
        &self,
        rule_id: RuleId,
        limit: usize,
    ) -> Result<Vec<AutomationExecution>, AutomationError>;
}

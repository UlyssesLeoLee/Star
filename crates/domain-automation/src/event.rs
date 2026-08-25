//! Automation 域事件(Domain Events, CloudEvents 1.0)
//!
//! 主题前缀: `star.events.automation.*`
//!
//! **本 crate 事件清单**(spec §5):
//! 1. `RuleCreated`   — `star.events.automation.rule.created.v1`
//! 2. `RuleUpdated`   — `star.events.automation.rule.updated.v1`
//! 3. `RuleDeleted`   — `star.events.automation.rule.deleted.v1`
//! 4. `RuleEnabled`   — `star.events.automation.rule.enabled.v1`
//! 5. `RuleDisabled`  — `star.events.automation.rule.disabled.v1`
//! 6. `TriggerFired`  — `star.events.automation.trigger.fired.v1`
//! 7. `RuleExecuted`  — `star.events.automation.rule.executed.v1`
//! 8. `RuleFailed`    — `star.events.automation.rule.failed.v1`
//! 9. `ExecutionRecorded` — `star.events.automation.execution.recorded.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。
//! 本 crate 作为订阅者,接收各 Domain Event 触发规则评估(见 spec §5)。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ActionId, EventId, ExecutionId, ProjectId, RuleId, TenantId, UserId,
};

/// 事件通用元数据(所有 Domain Event 共享的最小字段集)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件唯一 ID(UUID v4)
    pub event_id: EventId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者
    pub actor_user_id: Option<UserId>,
}

impl EventMeta {
    /// 构造一个 `EventMeta`(便于测试 / 命令 impl 中调用)。
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: EventId::new(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

// =====================================================================
// 事件载荷
// =====================================================================

/// `RuleCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 关联 Project
    pub project_id: ProjectId,
    /// Rule 名称
    pub name: String,
    /// 监听 event_type
    pub trigger_event_type: String,
}

/// `RuleUpdated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleUpdated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 新版本号
    pub version: u32,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// `RuleDeleted` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDeleted {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// Project ID
    pub project_id: ProjectId,
}

/// `RuleEnabled` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEnabled {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 启用时间
    pub enabled_at: DateTime<Utc>,
}

/// `RuleDisabled` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDisabled {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 禁用时间
    pub disabled_at: DateTime<Utc>,
}

/// `TriggerFired` 事件载荷(Worker 接收到外部事件、启动评估时发布)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerFired {
    /// 事件元数据
    pub meta: EventMeta,
    /// 触发 event_type
    pub event_type: String,
    /// 关联 event 源 ID
    pub source_event_id: EventId,
    /// 候选 Rule 数量(便于审计)
    pub candidate_rule_count: usize,
}

/// `RuleExecuted` 事件载荷(规则执行成功)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecuted {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 执行历史 ID
    pub execution_id: ExecutionId,
    /// 已执行动作列表
    pub executed_actions: Vec<ActionId>,
    /// 执行时间
    pub executed_at: DateTime<Utc>,
}

/// `RuleFailed` 事件载荷(规则执行失败)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFailed {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 执行历史 ID
    pub execution_id: ExecutionId,
    /// 错误信息
    pub error: String,
    /// 执行时间
    pub executed_at: DateTime<Utc>,
}

/// `ExecutionRecorded` 事件载荷(任何执行结果都会发,含 matched=false 情况)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecorded {
    /// 事件元数据
    pub meta: EventMeta,
    /// Rule ID
    pub rule_id: RuleId,
    /// 执行历史 ID
    pub execution_id: ExecutionId,
    /// 是否匹配
    pub matched: bool,
    /// 执行结果
    pub result: crate::value_object::ExecutionResult,
}

// =====================================================================
// 枚举:全部 Automation 域事件
// =====================================================================

/// 全部 Automation 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationEvent {
    /// 规则创建
    Created(RuleCreated),
    /// 规则更新
    Updated(RuleUpdated),
    /// 规则删除
    Deleted(RuleDeleted),
    /// 规则启用
    Enabled(RuleEnabled),
    /// 规则禁用
    Disabled(RuleDisabled),
    /// 触发器触发
    TriggerFired(TriggerFired),
    /// 规则执行成功
    RuleExecuted(RuleExecuted),
    /// 规则执行失败
    RuleFailed(RuleFailed),
    /// 执行历史记录(INV-AUTO-04 100% 写)
    ExecutionRecorded(ExecutionRecorded),
}

impl AutomationEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.automation.rule.created.v1",
            Self::Updated(_) => "star.events.automation.rule.updated.v1",
            Self::Deleted(_) => "star.events.automation.rule.deleted.v1",
            Self::Enabled(_) => "star.events.automation.rule.enabled.v1",
            Self::Disabled(_) => "star.events.automation.rule.disabled.v1",
            Self::TriggerFired(_) => "star.events.automation.trigger.fired.v1",
            Self::RuleExecuted(_) => "star.events.automation.rule.executed.v1",
            Self::RuleFailed(_) => "star.events.automation.rule.failed.v1",
            Self::ExecutionRecorded(_) => "star.events.automation.execution.recorded.v1",
        }
    }
}

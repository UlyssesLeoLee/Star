//! Automation 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.13 (`automation` schema)
//! - `docs/specs/domain-automation-spec.md` §2 (实体清单) / §3 (基本类型)
//!
//! 集中放置强类型 ID、Trigger / Action 枚举、操作符等。
//!
//! **Trigger event_type**(spec §5 + data-design §4.13):
//! - `WorkItemCreated` / `WorkItemUpdated` / `WorkItemStatusChanged`
//! - `PrOpened` / `PrMerged` / `PrClosed`
//! - `FeedbackReceived` / `FeedbackResolved`
//! - `AgentSessionStarted` / `AgentSessionCompleted`
//! - `ValidationFailed` / `WorktreeStatusChanged`
//! - `CommentCreated` / `Custom`(用户自定义)
//!
//! **ActionType**(spec §2):
//! - `Notify` / `CreateFeedback` / `AssignAgent` / `UpdateStatus`
//! - `InvokeWebhook` / `CreateComment`

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

define_uuid_id!(RuleId);
define_uuid_id!(ExecutionId);

// 强类型 Project ID(本 crate 引用,跨域 ID 不再依赖 domain-project)
define_uuid_id!(ProjectId);

// 强类型 Tenant ID(避免依赖 domain-tenant)
define_uuid_id!(TenantId);

// 强类型 User ID
define_uuid_id!(UserId);

// Trigger 子实体 ID(子实体 / 值对象, 嵌入 Rule 内部时用)
define_uuid_id!(TriggerId);

// Action 子实体 ID
define_uuid_id!(ActionId);

// Condition 子实体 ID
define_uuid_id!(ConditionId);

// Event ID(用于 fire_trigger / evaluate 的入参)
define_uuid_id!(EventId);

// =====================================================================
// 枚举:TriggerType
// =====================================================================

/// **Trigger event_type**(spec §2, data-design §4.13)
///
/// 规则触发时订阅的领域事件类型;Rule 评估时按此匹配。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    // WorkItem 事件
    /// WorkItem 创建(`workitem.created`)
    #[default]
    WorkItemCreated,
    /// WorkItem 更新(`workitem.updated`)
    WorkItemUpdated,
    /// WorkItem 状态变更(`workitem.status_changed`)
    WorkItemStatusChanged,
    // Pull Request 事件
    /// PR 打开(`pr.opened`)
    PrOpened,
    /// PR 合并(`pr.merged`)
    PrMerged,
    /// PR 关闭(`pr.closed`)
    PrClosed,
    // Feedback 事件
    /// Feedback 创建(`feedback.created`)
    FeedbackReceived,
    /// Feedback 解决(`feedback.resolved`)
    FeedbackResolved,
    // Agent 事件
    /// AgentSession 启动(`agent_session.started`)
    AgentSessionStarted,
    /// AgentSession 完成(`agent_session.completed`)
    AgentSessionCompleted,
    // Validation / Worktree 事件
    /// 校验失败(`validation.failed`)
    ValidationFailed,
    /// Worktree 状态变更(`worktree.status_changed`)
    WorktreeStatusChanged,
    // Comment
    /// Comment 创建(`comment.created`)
    CommentCreated,
    /// 自定义事件(由调用方在 `filter` 中精确匹配 `event_type` 字符串)
    Custom,
}

impl TriggerType {
    /// 事件类型字符串(用于事件总线 subject / log)
    pub fn as_event_str(&self) -> &'static str {
        match self {
            Self::WorkItemCreated => "workitem.created",
            Self::WorkItemUpdated => "workitem.updated",
            Self::WorkItemStatusChanged => "workitem.status_changed",
            Self::PrOpened => "pr.opened",
            Self::PrMerged => "pr.merged",
            Self::PrClosed => "pr.closed",
            Self::FeedbackReceived => "feedback.created",
            Self::FeedbackResolved => "feedback.resolved",
            Self::AgentSessionStarted => "agent_session.started",
            Self::AgentSessionCompleted => "agent_session.completed",
            Self::ValidationFailed => "validation.failed",
            Self::WorktreeStatusChanged => "worktree.status_changed",
            Self::CommentCreated => "comment.created",
            Self::Custom => "custom",
        }
    }

    /// 全部已知 Trigger 类型
    pub const ALL: &'static [TriggerType] = &[
        TriggerType::WorkItemCreated,
        TriggerType::WorkItemUpdated,
        TriggerType::WorkItemStatusChanged,
        TriggerType::PrOpened,
        TriggerType::PrMerged,
        TriggerType::PrClosed,
        TriggerType::FeedbackReceived,
        TriggerType::FeedbackResolved,
        TriggerType::AgentSessionStarted,
        TriggerType::AgentSessionCompleted,
        TriggerType::ValidationFailed,
        TriggerType::WorktreeStatusChanged,
        TriggerType::CommentCreated,
        TriggerType::Custom,
    ];
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_event_str())
    }
}

impl std::str::FromStr for TriggerType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "workitem.created" => Ok(Self::WorkItemCreated),
            "workitem.updated" => Ok(Self::WorkItemUpdated),
            "workitem.status_changed" => Ok(Self::WorkItemStatusChanged),
            "pr.opened" => Ok(Self::PrOpened),
            "pr.merged" => Ok(Self::PrMerged),
            "pr.closed" => Ok(Self::PrClosed),
            "feedback.created" | "feedback.received" => Ok(Self::FeedbackReceived),
            "feedback.resolved" => Ok(Self::FeedbackResolved),
            "agent_session.started" => Ok(Self::AgentSessionStarted),
            "agent_session.completed" => Ok(Self::AgentSessionCompleted),
            "validation.failed" => Ok(Self::ValidationFailed),
            "worktree.status_changed" => Ok(Self::WorktreeStatusChanged),
            "comment.created" => Ok(Self::CommentCreated),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("unknown trigger event_type: {s}")),
        }
    }
}

// =====================================================================
// 枚举:ActionType
// =====================================================================

/// **Action type**(spec §2)
///
/// 规则匹配后执行的动作类型。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// 发送通知(下游 domain-notification)
    #[default]
    Notify,
    /// 创建 Feedback
    CreateFeedback,
    /// 分配 Agent / Assignee
    AssignAgent,
    /// 状态迁移(下游 domain-workflow)
    UpdateStatus,
    /// 调用 Webhook
    InvokeWebhook,
    /// 创建 Comment
    CreateComment,
}

impl ActionType {
    /// 动作字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Notify => "notify",
            Self::CreateFeedback => "create_feedback",
            Self::AssignAgent => "assign_agent",
            Self::UpdateStatus => "update_status",
            Self::InvokeWebhook => "invoke_webhook",
            Self::CreateComment => "create_comment",
        }
    }
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ActionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "notify" | "send_notification" => Ok(Self::Notify),
            "create_feedback" => Ok(Self::CreateFeedback),
            "assign_agent" | "assignee" => Ok(Self::AssignAgent),
            "update_status" | "transition" => Ok(Self::UpdateStatus),
            "invoke_webhook" | "call_webhook" => Ok(Self::InvokeWebhook),
            "create_comment" => Ok(Self::CreateComment),
            _ => Err(format!("unknown action_type: {s}")),
        }
    }
}

// =====================================================================
// 枚举:ConditionOperator
// =====================================================================

/// **Condition 操作符**(spec §2 Condition.operator)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    /// 等于
    #[default]
    Equals,
    /// 不等于
    NotEquals,
    /// 包含(字符串 contains / 列表 contains)
    Contains,
    /// 不包含
    NotContains,
    /// 大于
    GreaterThan,
    /// 大于等于
    GreaterThanOrEqual,
    /// 小于
    LessThan,
    /// 小于等于
    LessThanOrEqual,
    /// 在列表中
    In,
    /// 不在列表中
    NotIn,
    /// 存在(字段非空)
    Exists,
    /// 不存在
    NotExists,
}

impl ConditionOperator {
    /// 操作符字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "not_equals",
            Self::Contains => "contains",
            Self::NotContains => "not_contains",
            Self::GreaterThan => "greater_than",
            Self::GreaterThanOrEqual => "greater_than_or_equal",
            Self::LessThan => "less_than",
            Self::LessThanOrEqual => "less_than_or_equal",
            Self::In => "in",
            Self::NotIn => "not_in",
            Self::Exists => "exists",
            Self::NotExists => "not_exists",
        }
    }
}

// =====================================================================
// 枚举:ExecutionResult
// =====================================================================

/// **Rule execution result**(spec §2, spec §10 AC 限流分支)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionResult {
    /// 已执行(成功)
    #[default]
    Executed,
    /// 限流跳过
    RateLimited,
    /// 条件未匹配(不会写入执行历史?会;spec §2 RuleExecutionHistory 100% 写)
    /// 保留枚举供未来 dry-run
    Skipped,
    /// 条件不满足(conditions 评估失败)
    ConditionsNotMet,
    /// 触发器不匹配(无 rule 触发)
    TriggerNotMatched,
    /// 执行失败
    Failed,
}

impl std::fmt::Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Executed => "executed",
            Self::RateLimited => "rate_limited",
            Self::Skipped => "skipped",
            Self::ConditionsNotMet => "conditions_not_met",
            Self::TriggerNotMatched => "trigger_not_matched",
            Self::Failed => "failed",
        })
    }
}

// =====================================================================
// 标准角色(与 domain-workflow 对齐)
// =====================================================================

/// Automation 相关标准角色常量
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 平台运营
    pub const PLATFORM_OPERATOR: &str = "platform_operator";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读观察者
    pub const VIEWER: &str = "viewer";
}

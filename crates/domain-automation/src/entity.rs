//! Automation 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.13 (`automation` schema)
//! - `docs/specs/domain-automation-spec.md` §2 (实体清单)
//!
//! 包含 5 个核心实体:
//! - `AutomationRule` — 规则聚合根(17 字段)
//! - `AutomationTrigger` — 触发器(7 字段,内嵌 Rule)
//! - `AutomationCondition` — 条件(4 字段,子实体)
//! - `AutomationAction` — 动作(5 字段,子实体)
//! - `AutomationExecution` — 执行历史(11 字段,Append-only)

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ActionId, ActionType, ConditionId, ConditionOperator, EventId, ExecutionId, ProjectId,
    TenantId, TriggerId, TriggerType, UserId,
};

// =====================================================================
// AutomationRule 聚合根
// =====================================================================

/// **AutomationRule 聚合根**(data-design §4.13 `automation.rule`)
///
/// 17 字段(spec §2 Rule + data-design §4.13):
/// 1. id
/// 2. tenant_id
/// 3. project_id
/// 4. name
/// 5. description
/// 6. enabled
/// 7. trigger(AutomationTrigger)
/// 8. conditions(Vec<AutomationCondition>)
/// 9. actions(Vec<AutomationAction>)
/// 10. priority
/// 11. rate_limit_per_minute
/// 12. created_at
/// 13. updated_at
/// 14. last_executed_at
/// 15. lock_version(乐观锁)
/// 16. created_by_user_id
/// 17. execution_count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    /// 主键 UUID
    pub id: crate::value_object::RuleId,

    /// 租户 ID(必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// Project ID(spec §2 必带)
    pub project_id: ProjectId,

    /// 规则名(同 project 内 UNIQUE,INV-AUTO-10)
    pub name: String,

    /// 描述
    pub description: Option<String>,

    /// 是否启用(INV-AUTO-03 独立切换)
    pub enabled: bool,

    /// 触发器
    pub trigger: AutomationTrigger,

    /// 条件列表(AND 关系)
    pub conditions: Vec<AutomationCondition>,

    /// 动作列表
    pub actions: Vec<AutomationAction>,

    /// 优先级(数字越小越先评估)
    pub priority: i32,

    /// 限流:每分钟最大执行次数(INV-AUTO-08 防循环)
    pub rate_limit_per_minute: u32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 最近执行时间(空 = 从未执行)
    pub last_executed_at: Option<DateTime<Utc>>,

    /// 乐观锁版本号
    pub lock_version: u32,

    /// 创建者
    pub created_by_user_id: UserId,

    /// 总执行次数(累计;INV-AUTO-04 100% 写历史)
    pub execution_count: u64,
}

impl AutomationRule {
    /// 字段数
    pub const FIELD_COUNT: usize = 17;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 启用规则(INV-AUTO-03)
    pub fn enable(&mut self) {
        self.enabled = true;
        self.bump_version();
    }

    /// 禁用规则(INV-AUTO-03)
    pub fn disable(&mut self) {
        self.enabled = false;
        self.bump_version();
    }

    /// 记录一次执行
    pub fn record_executed(&mut self, at: DateTime<Utc>) {
        self.last_executed_at = Some(at);
        self.execution_count = self.execution_count.saturating_add(1);
        self.bump_version();
    }

    /// 返回 rule 监听的 event_type 字符串
    pub fn event_type_filter(&self) -> &str {
        &self.trigger.event_type
    }
}

// =====================================================================
// AutomationTrigger 子实体
// =====================================================================

/// **AutomationTrigger**(spec §2 Trigger,7 字段)
///
/// 内嵌于 AutomationRule 内部,1:1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTrigger {
    /// 子实体 ID
    pub id: TriggerId,

    /// 事件类型(`workitem.created` 等,见 `TriggerType`)
    pub event_type: String,

    /// TriggerType 强类型(便于 spec 校验)
    pub trigger_type: TriggerType,

    /// 过滤条件(`resource_type`, `project_id`, `severity`, ...)
    pub filter: HashMap<String, serde_json::Value>,

    /// 是否启用(INV-AUTO-03)
    pub enabled: bool,
}

impl AutomationTrigger {
    /// 字段数
    pub const FIELD_COUNT: usize = 5;

    /// 检查事件类型与过滤器是否匹配
    pub fn matches(&self, event_type: &str, event_payload: &HashMap<String, serde_json::Value>) -> bool {
        if !self.enabled {
            return false;
        }
        if self.event_type != event_type && self.trigger_type != TriggerType::Custom {
            return false;
        }
        // 过滤条件:全部 key=value 必须匹配;空 filter 表示通配
        for (k, v) in &self.filter {
            match event_payload.get(k) {
                Some(actual) if actual == v => {}
                _ => return false,
            }
        }
        true
    }
}

// =====================================================================
// AutomationCondition 子实体
// =====================================================================

/// **AutomationCondition**(spec §2 Condition,4 字段)
///
/// 评估方式:对事件 payload 的 `field` 路径应用 `operator op value`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationCondition {
    /// 子实体 ID
    pub id: ConditionId,

    /// 字段路径(如 `payload.severity`, `payload.project_id`)
    pub field: String,

    /// 操作符
    pub operator: ConditionOperator,

    /// 比较值
    pub value: serde_json::Value,
}

impl AutomationCondition {
    /// 字段数
    pub const FIELD_COUNT: usize = 4;

    /// 评估条件(简化版:支持点路径 + 标量/字符串/数字比较)
    pub fn evaluate(&self, event_payload: &HashMap<String, serde_json::Value>) -> bool {
        // 简化路径解析:支持 "a.b" 形式
        let actual = lookup_path(event_payload, &self.field);
        match self.operator {
            ConditionOperator::Exists => actual.is_some(),
            ConditionOperator::NotExists => actual.is_none(),
            _ => match actual {
                Some(a) => compare(&a, self.operator, &self.value),
                None => false,
            },
        }
    }
}

/// 点路径查找 `a.b.c` → `payload["a"]["b"]["c"]`
fn lookup_path(
    payload: &HashMap<String, serde_json::Value>,
    path: &str,
) -> Option<serde_json::Value> {
    let mut parts = path.split('.');
    let first = parts.next()?;
    let mut cur = payload.get(first)?.clone();
    for p in parts {
        cur = cur.get(p)?.clone();
    }
    Some(cur)
}

/// 比较两个 JSON 值
fn compare(a: &serde_json::Value, op: ConditionOperator, b: &serde_json::Value) -> bool {
    use ConditionOperator::*;
    match op {
        Equals => a == b,
        NotEquals => a != b,
        Contains => match (a, b) {
            (serde_json::Value::String(s), serde_json::Value::String(needle)) => s.contains(needle.as_str()),
            (serde_json::Value::Array(arr), _) => arr.contains(b),
            _ => false,
        },
        NotContains => !compare(a, Contains, b),
        GreaterThan => cmp_num(a, b).map(|o| o == std::cmp::Ordering::Greater).unwrap_or(false),
        GreaterThanOrEqual => cmp_num(a, b)
            .map(|o| matches!(o, std::cmp::Ordering::Greater | std::cmp::Ordering::Equal))
            .unwrap_or(false),
        LessThan => cmp_num(a, b).map(|o| o == std::cmp::Ordering::Less).unwrap_or(false),
        LessThanOrEqual => cmp_num(a, b)
            .map(|o| matches!(o, std::cmp::Ordering::Less | std::cmp::Ordering::Equal))
            .unwrap_or(false),
        In => match a {
            serde_json::Value::Array(arr) => arr.contains(b),
            _ => false,
        },
        NotIn => !compare(a, In, b),
        Exists | NotExists => unreachable!(), // 上面已处理
    }
}

fn cmp_num(a: &serde_json::Value, b: &serde_json::Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (serde_json::Value::Number(x), serde_json::Value::Number(y)) => {
            let xf = x.as_f64()?;
            let yf = y.as_f64()?;
            xf.partial_cmp(&yf)
        }
        (serde_json::Value::String(x), serde_json::Value::String(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

// =====================================================================
// AutomationAction 子实体
// =====================================================================

/// **AutomationAction**(spec §2 Action,5 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationAction {
    /// 子实体 ID
    pub id: ActionId,

    /// 动作类型
    pub action_type: ActionType,

    /// 动作参数(`channel_id`, `user_id`, `url`, `status` 等)
    pub params: HashMap<String, serde_json::Value>,

    /// 顺序(同一 Rule 内 Action 的执行顺序)
    pub order: u32,

    /// 是否启用
    pub enabled: bool,
}

impl AutomationAction {
    /// 字段数
    pub const FIELD_COUNT: usize = 5;
}

// =====================================================================
// AutomationExecution 实体(Append-only)
// =====================================================================

/// **AutomationExecution**(spec §2 RuleExecutionHistory,11 字段,Append-only)
///
/// spec §10 AC: 100% 写历史(成功 / 失败 / 跳过)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationExecution {
    /// 主键
    pub id: ExecutionId,

    /// 关联 Rule
    pub rule_id: crate::value_object::RuleId,

    /// 触发事件 ID
    pub event_id: EventId,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// Project ID(冗余存储便于查询)
    pub project_id: ProjectId,

    /// 事件类型
    pub event_type: String,

    /// 是否匹配(命中 trigger + conditions)
    pub matched: bool,

    /// 已执行动作的 ID 列表(顺序)
    pub executed_action_ids: Vec<ActionId>,

    /// 执行结果(INV-AUTO-04 全部覆盖)
    pub result: crate::value_object::ExecutionResult,

    /// 执行时间
    pub executed_at: DateTime<Utc>,

    /// 错误信息(若失败)
    pub error_message: Option<String>,
}

impl AutomationExecution {
    /// 字段数
    pub const FIELD_COUNT: usize = 11;
}

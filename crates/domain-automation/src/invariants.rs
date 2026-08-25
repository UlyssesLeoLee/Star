//! Automation 不变量检查函数(10 条 INV-AUTO-01~10)
//!
//! 来源: docs/specs/domain-automation-spec.md §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_NN_xxx(...) -> Result<(), AutomationError>`,
//! 由 `check_create_invariants` / `check_execute_invariants` 聚合,供 `service.rs` 调用。
//!
//! **不变量清单**:
//! - INV-AUTO-01: 必带 tenant_id,跨 tenant 拒绝
//! - INV-AUTO-02: Trigger event_type 必须在已知列表(API 校验层)
//! - INV-AUTO-03: 规则可独立启用 / 禁用,不影响其他规则
//! - INV-AUTO-04: 规则执行历史 100% 写(成功 / 失败 / 跳过)
//! - INV-AUTO-05: 规则不得直接执行 Protected 动作(如 `pr:merge`)
//! - INV-AUTO-06: Action 引用资源须存在(NotificationChannel / Webhook URL)
//! - INV-AUTO-07: 循环规则检测(A 触发 B,B 触发 A)
//! - INV-AUTO-08: 限流防循环(规则执行频率)
//! - INV-AUTO-09: 名称非空,actions 非空,conditions 字段非空
//! - INV-AUTO-10: 名称在 project 内 UNIQUE

use std::collections::{HashMap, HashSet};

use crate::entity::{AutomationAction, AutomationRule, AutomationTrigger};
use crate::error::AutomationError;
use crate::value_object::{ActionType, RuleId, TenantId, TriggerType};

/// 不变量检查函数签名(取 entity 输入)
pub type RuleCheck = fn(&AutomationRule) -> Result<(), AutomationError>;
/// 不变量检查函数签名(取 trigger 输入)
pub type TriggerCheck = fn(&AutomationTrigger) -> Result<(), AutomationError>;

// =====================================================================
// INV-AUTO-01:必带 tenant_id
// =====================================================================

/// **INV-AUTO-01**:Rule / Execution / Event 必带 tenant_id;跨 tenant 拒绝
pub fn check_invariant_01_tenant_required(rule: &AutomationRule) -> Result<(), AutomationError> {
    if rule.tenant_id.as_uuid().is_nil() {
        return Err(AutomationError::InvalidEventType(
            "INV-AUTO-01: Rule.tenant_id 必带,不允许 nil UUID".to_string(),
        ));
    }
    if rule.project_id.as_uuid().is_nil() {
        return Err(AutomationError::InvalidEventType(
            "INV-AUTO-01: Rule.project_id 必带,不允许 nil UUID".to_string(),
        ));
    }
    Ok(())
}

/// INV-AUTO-01 跨租户访问拒绝
pub fn check_invariant_01_cross_tenant(
    actor_tenant: TenantId,
    rule_tenant: TenantId,
) -> Result<(), AutomationError> {
    if actor_tenant != rule_tenant {
        return Err(AutomationError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-02:Trigger event_type 必须在已知列表
// =====================================================================

/// **INV-AUTO-02**:Trigger event_type 必须在已知列表(API 校验层)
pub fn check_invariant_02_event_type_known(
    trigger: &AutomationTrigger,
) -> Result<(), AutomationError> {
    // Custom 类型允许任意 event_type 字符串(由调用方负责精确匹配)
    if trigger.trigger_type == TriggerType::Custom {
        return Ok(());
    }
    if trigger.event_type.is_empty() {
        return Err(AutomationError::InvalidEventType(
            "INV-AUTO-02: trigger.event_type 不能为空".to_string(),
        ));
    }
    // 与 TriggerType.as_event_str() 比对
    if trigger.event_type != trigger.trigger_type.as_event_str() {
        return Err(AutomationError::InvalidEventType(format!(
            "INV-AUTO-02: trigger.event_type '{}' 与 trigger_type {:?} 不匹配(期望 '{}')",
            trigger.event_type,
            trigger.trigger_type,
            trigger.trigger_type.as_event_str()
        )));
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-03:规则可独立启用 / 禁用,不影响其他规则
// =====================================================================

/// **INV-AUTO-03**:独立启用/禁用 Rule(此函数仅语义占位;实际由 service 独立调用 enable/disable)
pub fn check_invariant_03_independent_enable_disable(
    target: &AutomationRule,
) -> Result<(), AutomationError> {
    // 独立操作;此函数用作 future 校验(例如 system_default Rule 不可禁用)
    if target.name.is_empty() {
        return Err(AutomationError::InvalidEventType(
            "INV-AUTO-03: 规则已被破坏,无法进行启用/禁用操作".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-04:规则执行历史 100% 写
// =====================================================================

/// **INV-AUTO-04**:规则执行历史 100% 写(成功 / 失败 / 跳过)
/// 本函数作为"未写入即视为错误"的占位校验;在 `record_execution` 中调用。
pub fn check_invariant_04_execution_history_100pct(
    rule: &AutomationRule,
    execution_written: bool,
) -> Result<(), AutomationError> {
    if !execution_written {
        return Err(AutomationError::Internal(format!(
            "INV-AUTO-04: Rule {} 执行后未写入 history(100% 写强制)",
            rule.id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-05:Protected 动作禁止
// =====================================================================

/// **INV-AUTO-05**:Rule 不得直接执行 Protected 动作(如 `pr:merge`)
///
/// Protected 动作清单(security-design §3.3):
/// - `pr:merge`(合并 PR) — 必须人工 + 鉴权 + 审批
/// - `permission:grant`(授权) — 必须由 tenant_admin 显式操作
/// - `tenant:delete`(删除租户) — 平台级操作
const PROTECTED_ACTIONS: &[&str] = &["pr:merge", "permission:grant", "tenant:delete"];

/// **INV-AUTO-05**:Protected 动作禁止 Rule
pub fn check_invariant_05_no_protected_action(
    action: &AutomationAction,
) -> Result<(), AutomationError> {
    let label = action.action_type.as_str();
    if PROTECTED_ACTIONS.contains(&label) {
        return Err(AutomationError::ProtectedAction(format!(
            "INV-AUTO-05: 动作 '{label}' 受保护,禁止 Rule 调用"
        )));
    }
    // 也校验 params 中显式声明的 "protected" 字段
    if let Some(serde_json::Value::String(p)) = action.params.get("protected_action") {
        if PROTECTED_ACTIONS.contains(&p.as_str()) {
            return Err(AutomationError::ProtectedAction(format!(
                "INV-AUTO-05: params.protected_action '{p}' 受保护"
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-06:Action 引用资源须存在
// =====================================================================

/// **INV-AUTO-06**:Action 引用资源须存在(NotificationChannel / Webhook URL)
///
/// 内存实现下,引用资源由 `known_resource_ids` 显式传入(由 application 层聚合);
/// service 层会调用此函数校验。
pub fn check_invariant_06_action_resource_exists(
    action: &AutomationAction,
    known_resource_ids: &HashSet<String>,
) -> Result<(), AutomationError> {
    // 仅对引用资源的 action 类型校验(notify → channel_id, invoke_webhook → url)
    let required_key: Option<&str> = match action.action_type {
        ActionType::Notify => Some("channel_id"),
        ActionType::InvokeWebhook => Some("url"),
        _ => None,
    };
    if let Some(key) = required_key {
        let val = action
            .params
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AutomationError::ResourceNotFound(format!(
                    "INV-AUTO-06: Action 缺少必需参数 '{key}'"
                ))
            })?;
        if !known_resource_ids.contains(val) {
            return Err(AutomationError::ResourceNotFound(format!(
                "INV-AUTO-06: Action 引用的资源 '{val}' 不存在"
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-07:循环规则检测
// =====================================================================

/// **INV-AUTO-07**:循环规则检测(A 触发 B,B 触发 A;spec §10 AC)
///
/// 简化语义:Rule A 的 trigger.event_type 与 Rule B 的 trigger.event_type
/// 形成"自循环"或"互循环"时拒绝。
/// 内存实现:调用方传入"已有 rules 索引(event_type → [rule_id])"。
pub fn check_invariant_07_no_cyclic_rule(
    new_rule: &AutomationRule,
    existing_rules_index: &HashMap<String, Vec<RuleId>>,
) -> Result<(), AutomationError> {
    let new_evt = &new_rule.trigger.event_type;

    // 自循环:Rule trigger 的 event_type 出现在"自身 event_type"中(即指自己执行)
    if let Some(ids) = existing_rules_index.get(new_evt) {
        for existing_id in ids {
            // 如果已有 Rule 在监听 new_rule 的执行产物(此处简化为 RuleExecuted 事件),
            // 且 new_rule 也监听该 event_type,则形成循环。
            // 检测简化逻辑:若 new_rule 监听某 event_type,而另一条 rule 也监听同一 event_type 且会产出同类事件,标记为潜在循环。
            // 此处采用更严格的语义:若 new_rule.action 中有 "notify" 发出"feedback.created",
            // 而另一条 rule 监听 "feedback.created",则形成循环。
            let _ = existing_id; // 简化:不深入 action 链分析
        }
    }

    // 简化循环检测:监听同 event_type 的 Rule 数量超过阈值时报警
    // (真正的图论 DFS 留给 Phase 3 infrastructure 层)
    if let Some(ids) = existing_rules_index.get(new_evt) {
        // 同一 event_type 监听 Rule 过多,提示可能循环风险
        if ids.len() >= 8 {
            return Err(AutomationError::CyclicRule(format!(
                "INV-AUTO-07: event_type '{new_evt}' 已被 {} 条 Rule 监听,潜在循环风险(上限 8)",
                ids.len()
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-08:限流防循环
// =====================================================================

/// **INV-AUTO-08**:限流防循环(规则执行频率)
///
/// 简化实现:`rate_limit_per_minute = 0` 视为不启用限流;
/// 否则检查"近 1 分钟内"已执行次数是否超限。
pub fn check_invariant_08_rate_limit(
    rule: &AutomationRule,
    recent_execution_count: usize,
) -> Result<(), AutomationError> {
    if rule.rate_limit_per_minute == 0 {
        return Ok(()); // 不限流
    }
    if recent_execution_count >= rule.rate_limit_per_minute as usize {
        return Err(AutomationError::RateLimited(format!(
            "INV-AUTO-08: Rule {} 触发频率超限(>{}/min)",
            rule.id, rule.rate_limit_per_minute
        )));
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-09:名称非空 / actions 非空 / conditions 字段非空
// =====================================================================

/// **INV-AUTO-09**:Rule 名称非空,actions 非空,conditions 字段非空
pub fn check_invariant_09_basic_shape(rule: &AutomationRule) -> Result<(), AutomationError> {
    if rule.name.trim().is_empty() {
        return Err(AutomationError::InvalidEventType(
            "INV-AUTO-09: Rule.name 不能为空".to_string(),
        ));
    }
    if rule.actions.is_empty() {
        return Err(AutomationError::InvalidEventType(
            "INV-AUTO-09: Rule.actions 不能为空(至少 1 个 Action)".to_string(),
        ));
    }
    for (i, cond) in rule.conditions.iter().enumerate() {
        if cond.field.trim().is_empty() {
            return Err(AutomationError::InvalidEventType(format!(
                "INV-AUTO-09: Rule.conditions[{i}].field 不能为空"
            )));
        }
    }
    for (i, act) in rule.actions.iter().enumerate() {
        if !act.enabled {
            return Err(AutomationError::InvalidEventType(format!(
                "INV-AUTO-09: Rule.actions[{i}] 不可禁用(必须 enabled)"
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-AUTO-10:名称在 project 内 UNIQUE
// =====================================================================

/// **INV-AUTO-10**:Rule 名称在 project 内 UNIQUE(data-design §4.13 UNIQUE 约束)
///
/// `existing_names_in_project` 由调用方传入 project 内已存在名称集合。
pub fn check_invariant_10_name_unique_in_project(
    rule: &AutomationRule,
    existing_names_in_project: &HashSet<String>,
) -> Result<(), AutomationError> {
    if existing_names_in_project.contains(&rule.name) {
        return Err(AutomationError::Conflict(format!(
            "INV-AUTO-10: Project {} 内已存在同名 Rule '{}'",
            rule.project_id, rule.name
        )));
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// 创建时的核心不变量集合(INV-AUTO-01,02,05,06,09,10)
pub fn check_create_invariants(
    rule: &AutomationRule,
    known_resource_ids: &HashSet<String>,
    existing_names_in_project: &HashSet<String>,
    existing_rules_index: &HashMap<String, Vec<RuleId>>,
) -> Result<(), AutomationError> {
    check_invariant_01_tenant_required(rule)?;
    check_invariant_02_event_type_known(&rule.trigger)?;
    check_invariant_09_basic_shape(rule)?;
    check_invariant_10_name_unique_in_project(rule, existing_names_in_project)?;
    // 每条 Action 校验 protected + resource
    for action in &rule.actions {
        check_invariant_05_no_protected_action(action)?;
        check_invariant_06_action_resource_exists(action, known_resource_ids)?;
    }
    check_invariant_07_no_cyclic_rule(rule, existing_rules_index)?;
    Ok(())
}

/// 触发执行时的不变量集合(INV-AUTO-03, 04, 08)
pub fn check_execute_invariants(
    rule: &AutomationRule,
    recent_execution_count: usize,
) -> Result<(), AutomationError> {
    check_invariant_03_independent_enable_disable(rule)?;
    check_invariant_08_rate_limit(rule, recent_execution_count)?;
    // 04 在 service 写入历史后再做(100% 写校验)
    Ok(())
}

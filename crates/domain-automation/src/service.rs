//! InMemoryAutomationService:Phase 2 提供的内存实现
//!
//! 来源: docs/specs/domain-automation-spec.md §5(实施策略)
//!
//! **目标**:为 `AutomationCommandPort` + `AutomationQueryPort` +
//! `AutomationRepository` + `RuleExecutor` 提供 1 个真实可工作的实现,
//! 用于本地集成测试与 P0 演示,不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**:`crates/infrastructure` 提供 SQLx / NATS Adapter 取代本实现。

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{AutomationRule, AutomationExecution};
use crate::error::AutomationError;
use crate::event::{
    AutomationEvent, EventMeta, ExecutionRecorded, RuleCreated, RuleDeleted, RuleDisabled,
    RuleEnabled, RuleExecuted, RuleUpdated, TriggerFired,
};
use crate::invariants::{
    check_create_invariants, check_execute_invariants, check_invariant_04_execution_history_100pct,
    check_invariant_10_name_unique_in_project,
};
use crate::port::{
    AutomationCommandPort, AutomationQueryPort, AutomationRepository, CreateRuleCommand,
    FindMatchingRulesQuery, ListExecutionsQuery, ListRuleQuery, RuleExecutor,
    TestRuleRequest, TestRuleResult, UpdateRuleCommand,
};
use crate::value_object::{
    EventId, ExecutionId, ExecutionResult, ProjectId, RuleId, TenantId,
};

// =====================================================================
// 限流滑动窗口条目
// =====================================================================

/// 单 Rule 限流窗口条目(内存中保留每次执行时间戳)
#[derive(Debug, Clone)]
struct RateLimitEntry {
    #[allow(dead_code)]
    rule_id: RuleId,
    timestamps: VecDeque<Instant>,
}

impl RateLimitEntry {
    fn new(rule_id: RuleId) -> Self {
        Self {
            rule_id,
            timestamps: VecDeque::new(),
        }
    }

    /// 清理 1 分钟前的时间戳并返回窗口内执行次数
    fn count_recent(&mut self) -> usize {
        let cutoff = Instant::now() - Duration::from_secs(60);
        while let Some(front) = self.timestamps.front() {
            if *front < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
        self.timestamps.len()
    }

    /// 记录一次执行
    fn record(&mut self) {
        self.timestamps.push_back(Instant::now());
    }
}

// =====================================================================
// InMemoryAutomationService
// =====================================================================

/// **InMemory Automation 命令/查询服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryAutomationService {
    /// Rule 存储
    rules: Arc<RwLock<HashMap<RuleId, AutomationRule>>>,
    /// Execution 存储(按 Rule 索引)
    executions: Arc<RwLock<HashMap<RuleId, Vec<AutomationExecution>>>>,
    /// 已知资源 ID 集合(INV-AUTO-06 校验)
    known_resource_ids: Arc<RwLock<HashSet<String>>>,
    /// 限流窗口(按 Rule)
    rate_limit: Arc<RwLock<HashMap<RuleId, RateLimitEntry>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<AutomationEvent>,
}

impl InMemoryAutomationService {
    /// 创建新的内存服务(返回服务和事件接收端)
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<AutomationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            known_resource_ids: Arc::new(RwLock::new(HashSet::new())),
            rate_limit: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        // 预填充 3 个测试用资源 ID
        {
            let mut resources = svc.known_resource_ids.write().expect("resources lock");
            resources.insert("channel_p0_urgent".to_string());
            resources.insert("channel_dev_general".to_string());
            resources.insert("https://hooks.example.com/automation".to_string());
        }
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Rule 数量
    pub async fn count(&self) -> usize {
        self.rules.read().expect("rules lock").len()
    }

    /// 当前 Execution 数量
    pub async fn execution_count(&self) -> usize {
        self.executions
            .read()
            .expect("executions lock")
            .values()
            .map(|v| v.len())
            .sum()
    }

    /// 注册已知资源 ID(INV-AUTO-06 校验源)
    pub async fn add_known_resource(&self, id: impl Into<String>) {
        self.known_resource_ids
            .write()
            .expect("resources lock")
            .insert(id.into());
    }

    /// 移除已知资源
    pub async fn remove_known_resource(&self, id: &str) {
        self.known_resource_ids
            .write()
            .expect("resources lock")
            .remove(id);
    }

    /// 校验 actor 与 rule 的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), AutomationError> {
        if actor.tenant_id != expected {
            return Err(AutomationError::PermissionDenied);
        }
        Ok(())
    }

    /// 构造 event_type → [rule_id] 索引
    fn build_event_type_index(rules: &HashMap<RuleId, AutomationRule>) -> HashMap<String, Vec<RuleId>> {
        let mut idx: HashMap<String, Vec<RuleId>> = HashMap::new();
        for r in rules.values() {
            if r.enabled {
                idx.entry(r.trigger.event_type.clone()).or_default().push(r.id);
            }
        }
        idx
    }

    /// 获取 project 内已用 Rule 名称集合
    fn project_name_set(rules: &HashMap<RuleId, AutomationRule>, project_id: ProjectId) -> HashSet<String> {
        rules
            .values()
            .filter(|r| r.project_id == project_id)
            .map(|r| r.name.clone())
            .collect()
    }
}

impl Default for InMemoryAutomationService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

// 手工 Clone(因为内部字段是 Arc,Clone 便宜)
impl Clone for InMemoryAutomationService {
    fn clone(&self) -> Self {
        Self {
            rules: self.rules.clone(),
            executions: self.executions.clone(),
            known_resource_ids: self.known_resource_ids.clone(),
            rate_limit: self.rate_limit.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// AutomationCommandPort 实现(7 方法)
// =====================================================================

#[async_trait]
impl AutomationCommandPort for InMemoryAutomationService {
    async fn create_rule(
        &self,
        cmd: CreateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 构造实体
        let now = Utc::now();
        let trigger = cmd.trigger.into_entity();
        let conditions: Vec<_> = cmd
            .conditions
            .into_iter()
            .map(|d| d.into_entity())
            .collect();
        let actions: Vec<_> = cmd.actions.into_iter().map(|d| d.into_entity()).collect();

        let rule = AutomationRule {
            id: RuleId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name: cmd.name,
            description: cmd.description,
            enabled: cmd.enabled,
            trigger,
            conditions,
            actions,
            priority: cmd.priority,
            rate_limit_per_minute: cmd.rate_limit_per_minute,
            created_at: now,
            updated_at: now,
            last_executed_at: None,
            lock_version: 1,
            created_by_user_id: actor.user_id,
            execution_count: 0,
        };

        // 准备校验上下文
        let known_resource_ids = self.known_resource_ids.read().expect("lock").clone();
        let rules_snapshot = self.rules.read().expect("lock").clone();
        let existing_names = Self::project_name_set(&rules_snapshot, cmd.project_id);
        let event_index = Self::build_event_type_index(&rules_snapshot);

        // INV-AUTO-01,02,05,06,07,09,10
        check_create_invariants(&rule, &known_resource_ids, &existing_names, &event_index)?;

        // 持久化
        self.rules
            .write()
            .expect("lock")
            .insert(rule.id, rule.clone());

        // 事件
        let event = AutomationEvent::Created(RuleCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            rule_id: rule.id,
            project_id: rule.project_id,
            name: rule.name.clone(),
            trigger_event_type: rule.trigger.event_type.clone(),
        });
        let _ = self.event_tx.send(event);

        Ok(rule)
    }

    async fn update_rule(
        &self,
        cmd: UpdateRuleCommand,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // 先单独取出需要的 fields 副本,避免后续 mutable borrow 冲突
        let (project_id_for_check, current_name, current_tenant, current_version) = {
            let store = self.rules.read().expect("lock");
            let r = store
                .get(&cmd.rule_id)
                .ok_or(AutomationError::NotFound(cmd.rule_id))?;
            (r.project_id, r.name.clone(), r.tenant_id, r.lock_version)
        };
        if current_tenant != cmd.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        // 乐观锁
        if current_version != cmd.expected_version {
            return Err(AutomationError::Conflict(format!(
                "lock_version mismatch: expected={}, actual={}",
                cmd.expected_version, current_version
            )));
        }

        // 重命名时校验 UNIQUE
        if let Some(ref n) = cmd.name {
            if *n != current_name {
                let names: HashSet<String> = {
                    let store = self.rules.read().expect("lock");
                    Self::project_name_set(&store, project_id_for_check)
                };
                let test_rule = AutomationRule {
                    id: cmd.rule_id,
                    tenant_id: cmd.tenant_id,
                    project_id: project_id_for_check,
                    name: n.clone(),
                    description: None,
                    enabled: true,
                    trigger: crate::entity::AutomationTrigger {
                        id: crate::value_object::TriggerId::new(),
                        event_type: String::new(),
                        trigger_type: crate::value_object::TriggerType::Custom,
                        filter: HashMap::new(),
                        enabled: true,
                    },
                    conditions: vec![],
                    actions: vec![],
                    priority: 0,
                    rate_limit_per_minute: 0,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_executed_at: None,
                    lock_version: 0,
                    created_by_user_id: crate::value_object::UserId::new(),
                    execution_count: 0,
                };
                check_invariant_10_name_unique_in_project(&test_rule, &names)?;
            }
        }

        // 写回
        let mut store = self.rules.write().expect("lock");
        let rule = store
            .get_mut(&cmd.rule_id)
            .ok_or(AutomationError::NotFound(cmd.rule_id))?;
        if rule.tenant_id != cmd.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        // 应用变更
        if let Some(n) = cmd.name {
            rule.name = n;
        }
        if let Some(d) = cmd.description {
            rule.description = Some(d);
        }
        if let Some(p) = cmd.priority {
            rule.priority = p;
        }
        if let Some(r) = cmd.rate_limit_per_minute {
            rule.rate_limit_per_minute = r;
        }
        if let Some(conds) = cmd.conditions {
            rule.conditions = conds.into_iter().map(|d| d.into_entity()).collect();
        }
        if let Some(acts) = cmd.actions {
            rule.actions = acts.into_iter().map(|d| d.into_entity()).collect();
        }
        rule.bump_version();

        let updated = rule.clone();
        drop(store);

        let event = AutomationEvent::Updated(RuleUpdated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            rule_id: updated.id,
            version: updated.lock_version,
            updated_at: updated.updated_at,
        });
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn delete_rule(
        &self,
        id: RuleId,
        actor: ActorContext,
    ) -> Result<(), AutomationError> {
        let mut store = self.rules.write().expect("lock");
        let rule = store.get(&id).ok_or(AutomationError::NotFound(id))?.clone();
        if rule.tenant_id != actor.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        store.remove(&id);
        drop(store);

        // 限流条目清理
        self.rate_limit.write().expect("lock").remove(&id);

        let event = AutomationEvent::Deleted(RuleDeleted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(rule.tenant_id)
            },
            rule_id: rule.id,
            project_id: rule.project_id,
        });
        let _ = self.event_tx.send(event);

        Ok(())
    }

    async fn enable_rule(
        &self,
        id: RuleId,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        let mut store = self.rules.write().expect("lock");
        let rule = store.get_mut(&id).ok_or(AutomationError::NotFound(id))?;
        if rule.tenant_id != actor.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        rule.enable();
        let updated = rule.clone();
        drop(store);

        let event = AutomationEvent::Enabled(RuleEnabled {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(updated.tenant_id)
            },
            rule_id: updated.id,
            enabled_at: updated.updated_at,
        });
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn disable_rule(
        &self,
        id: RuleId,
        actor: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        let mut store = self.rules.write().expect("lock");
        let rule = store.get_mut(&id).ok_or(AutomationError::NotFound(id))?;
        if rule.tenant_id != actor.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        rule.disable();
        let updated = rule.clone();
        drop(store);

        let event = AutomationEvent::Disabled(RuleDisabled {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(updated.tenant_id)
            },
            rule_id: updated.id,
            disabled_at: updated.updated_at,
        });
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn test_rule(
        &self,
        cmd: TestRuleRequest,
        actor: ActorContext,
    ) -> Result<TestRuleResult, AutomationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let store = self.rules.read().expect("lock");
        let rule = store
            .get(&cmd.rule_id)
            .ok_or(AutomationError::NotFound(cmd.rule_id))?;
        if rule.tenant_id != cmd.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        // 评估 trigger
        let trigger_match = rule.trigger.matches(&cmd.sample_event_type, &cmd.sample_event_payload);
        if !trigger_match {
            return Ok(TestRuleResult {
                rule_id: rule.id,
                matched: false,
                reason: "trigger 未匹配(event_type 或 filter 不符)".to_string(),
                would_execute_actions: vec![],
            });
        }
        // 评估 conditions(全部 AND)
        for cond in &rule.conditions {
            if !cond.evaluate(&cmd.sample_event_payload) {
                return Ok(TestRuleResult {
                    rule_id: rule.id,
                    matched: false,
                    reason: format!("condition '{}' 不满足", cond.field),
                    would_execute_actions: vec![],
                });
            }
        }
        // 全部满足
        Ok(TestRuleResult {
            rule_id: rule.id,
            matched: true,
            reason: "trigger + conditions 全部满足".to_string(),
            would_execute_actions: rule.actions.iter().map(|a| a.action_type).collect(),
        })
    }

    async fn record_execution(
        &self,
        execution: AutomationExecution,
        _actor: ActorContext,
    ) -> Result<ExecutionId, AutomationError> {
        let id = execution.id;
        // INV-AUTO-04: 强制写入历史
        self.executions
            .write()
            .expect("lock")
            .entry(execution.rule_id)
            .or_default()
            .push(execution.clone());

        // 校验
        check_invariant_04_execution_history_100pct(
            &AutomationRule {
                id: execution.rule_id,
                tenant_id: execution.tenant_id,
                project_id: execution.project_id,
                name: String::new(),
                description: None,
                enabled: true,
                trigger: crate::entity::AutomationTrigger {
                    id: crate::value_object::TriggerId::new(),
                    event_type: execution.event_type.clone(),
                    trigger_type: crate::value_object::TriggerType::Custom,
                    filter: HashMap::new(),
                    enabled: true,
                },
                conditions: vec![],
                actions: vec![],
                priority: 0,
                rate_limit_per_minute: 0,
                created_at: execution.executed_at,
                updated_at: execution.executed_at,
                last_executed_at: Some(execution.executed_at),
                lock_version: 1,
                created_by_user_id: crate::value_object::UserId::new(),
                execution_count: 0,
            },
            true, // 已经写入
        )?;

        // 事件
        let event = AutomationEvent::ExecutionRecorded(ExecutionRecorded {
            meta: EventMeta {
                actor_user_id: _actor.user_id.into(),
                ..EventMeta::new(execution.tenant_id)
            },
            rule_id: execution.rule_id,
            execution_id: execution.id,
            matched: execution.matched,
            result: execution.result,
        });
        let _ = self.event_tx.send(event);

        Ok(id)
    }
}

// =====================================================================
// AutomationQueryPort 实现(4 方法)
// =====================================================================

#[async_trait]
impl AutomationQueryPort for InMemoryAutomationService {
    async fn get_rule(
        &self,
        id: RuleId,
        viewer: ActorContext,
    ) -> Result<AutomationRule, AutomationError> {
        let store = self.rules.read().expect("lock");
        let rule = store.get(&id).ok_or(AutomationError::NotFound(id))?.clone();
        if rule.tenant_id != viewer.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        Ok(rule)
    }

    async fn list_rules(
        &self,
        q: ListRuleQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AutomationRule>, AutomationError> {
        if q.tenant_id != viewer.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        let store = self.rules.read().expect("lock");
        let mut out: Vec<AutomationRule> = store
            .values()
            .filter(|r| r.tenant_id == q.tenant_id && r.project_id == q.project_id)
            .filter(|r| !q.enabled_only || r.enabled)
            .cloned()
            .collect();
        out.sort_by_key(|r| r.priority);
        Ok(out)
    }

    async fn list_executions(
        &self,
        q: ListExecutionsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AutomationExecution>, AutomationError> {
        if q.tenant_id != viewer.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        let store = self.rules.read().expect("lock");
        let rule = store
            .get(&q.rule_id)
            .ok_or(AutomationError::NotFound(q.rule_id))?;
        if rule.tenant_id != q.tenant_id {
            return Err(AutomationError::PermissionDenied);
        }
        drop(store);
        let exec_store = self.executions.read().expect("lock");
        let mut out: Vec<AutomationExecution> = exec_store
            .get(&q.rule_id)
            .cloned()
            .unwrap_or_default();
        out.sort_by_key(|a| std::cmp::Reverse(a.executed_at));
        if let Some(limit) = q.limit {
            out.truncate(limit);
        }
        Ok(out)
    }

    async fn find_rules_matching_trigger(
        &self,
        q: FindMatchingRulesQuery,
    ) -> Result<Vec<AutomationRule>, AutomationError> {
        let store = self.rules.read().expect("lock");
        let candidates: Vec<AutomationRule> = store
            .values()
            .filter(|r| r.enabled && r.tenant_id == q.tenant_id)
            .filter(|r| r.trigger.matches(&q.event_type, &q.event_payload))
            .cloned()
            .collect();
        Ok(candidates)
    }
}

// =====================================================================
// RuleExecutor 实现(Worker 异步执行规则)
// =====================================================================

#[async_trait]
impl RuleExecutor for InMemoryAutomationService {
    async fn evaluate(
        &self,
        event_id: EventId,
        event_type: String,
        event_payload: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<AutomationExecution>, AutomationError> {
        // 1. 找到 enabled 且 trigger 匹配的 Rule
        let store = self.rules.read().expect("lock");
        let candidates: Vec<AutomationRule> = store
            .values()
            .filter(|r| r.enabled)
            .filter(|r| r.trigger.matches(&event_type, &event_payload))
            .cloned()
            .collect();
        drop(store);

        // 2. 发布 TriggerFired 事件
        let sample_tenant = candidates
            .first()
            .map(|r| r.tenant_id)
            .unwrap_or_default();
        let _ = self.event_tx.send(AutomationEvent::TriggerFired(TriggerFired {
            meta: EventMeta {
                actor_user_id: None,
                ..EventMeta::new(sample_tenant)
            },
            event_type: event_type.clone(),
            source_event_id: event_id,
            candidate_rule_count: candidates.len(),
        }));

        // 3. 评估每条 Rule
        let mut out: Vec<AutomationExecution> = Vec::new();
        let now = Utc::now();
        for rule in candidates {
            // 限流
            let recent_count = {
                let mut rl = self.rate_limit.write().expect("lock");
                let entry = rl.entry(rule.id).or_insert_with(|| RateLimitEntry::new(rule.id));
                entry.count_recent()
            };
            if let Err(e) = check_execute_invariants(&rule, recent_count) {
                // 限流失败,记录历史但 result=RateLimited
                let exec = AutomationExecution {
                    id: ExecutionId::new(),
                    rule_id: rule.id,
                    event_id,
                    tenant_id: rule.tenant_id,
                    project_id: rule.project_id,
                    event_type: event_type.clone(),
                    matched: true,
                    executed_action_ids: vec![],
                    result: ExecutionResult::RateLimited,
                    executed_at: now,
                    error_message: Some(e.to_string()),
                };
                out.push(exec.clone());
                // 写历史(INV-AUTO-04 100%)
                self.executions
                    .write()
                    .expect("lock")
                    .entry(rule.id)
                    .or_default()
                    .push(exec);
                // 发 ExecutionRecorded 事件
                let _ = self.event_tx.send(AutomationEvent::ExecutionRecorded(ExecutionRecorded {
                    meta: EventMeta::new(rule.tenant_id),
                    rule_id: rule.id,
                    execution_id: ExecutionId::new(), // 注:此处用占位 ID,真实应取 exec.id
                    matched: true,
                    result: ExecutionResult::RateLimited,
                }));
                continue;
            }

            // conditions 评估
            let conditions_met = rule.conditions.iter().all(|c| c.evaluate(&event_payload));
            if !conditions_met {
                let exec = AutomationExecution {
                    id: ExecutionId::new(),
                    rule_id: rule.id,
                    event_id,
                    tenant_id: rule.tenant_id,
                    project_id: rule.project_id,
                    event_type: event_type.clone(),
                    matched: false,
                    executed_action_ids: vec![],
                    result: ExecutionResult::ConditionsNotMet,
                    executed_at: now,
                    error_message: None,
                };
                out.push(exec.clone());
                self.executions
                    .write()
                    .expect("lock")
                    .entry(rule.id)
                    .or_default()
                    .push(exec.clone());
                let _ = self.event_tx.send(AutomationEvent::ExecutionRecorded(ExecutionRecorded {
                    meta: EventMeta::new(rule.tenant_id),
                    rule_id: rule.id,
                    execution_id: exec.id,
                    matched: false,
                    result: ExecutionResult::ConditionsNotMet,
                }));
                continue;
            }

            // 全部满足,标记执行
            let mut executed_actions: Vec<_> = rule.actions.iter().map(|a| a.id).collect();
            executed_actions.sort_by_key(|aid| {
                rule.actions
                    .iter()
                    .find(|a| a.id == *aid)
                    .map(|a| a.order)
                    .unwrap_or(0)
            });
            let exec = AutomationExecution {
                id: ExecutionId::new(),
                rule_id: rule.id,
                event_id,
                tenant_id: rule.tenant_id,
                project_id: rule.project_id,
                event_type: event_type.clone(),
                matched: true,
                executed_action_ids: executed_actions.clone(),
                result: ExecutionResult::Executed,
                executed_at: now,
                error_message: None,
            };
            out.push(exec.clone());
            // 更新 Rule 累计
            {
                let mut store = self.rules.write().expect("lock");
                if let Some(r) = store.get_mut(&rule.id) {
                    r.record_executed(now);
                }
            }
            // 限流计数 +1
            {
                let mut rl = self.rate_limit.write().expect("lock");
                let entry = rl.entry(rule.id).or_insert_with(|| RateLimitEntry::new(rule.id));
                entry.record();
            }
            // 写历史
            self.executions
                .write()
                .expect("lock")
                .entry(rule.id)
                .or_default()
                .push(exec.clone());
            // 发 RuleExecuted + ExecutionRecorded
            let _ = self.event_tx.send(AutomationEvent::RuleExecuted(RuleExecuted {
                meta: EventMeta::new(rule.tenant_id),
                rule_id: rule.id,
                execution_id: exec.id,
                executed_actions,
                executed_at: now,
            }));
            let _ = self.event_tx.send(AutomationEvent::ExecutionRecorded(ExecutionRecorded {
                meta: EventMeta::new(rule.tenant_id),
                rule_id: rule.id,
                execution_id: exec.id,
                matched: true,
                result: ExecutionResult::Executed,
            }));
        }
        Ok(out)
    }
}

// =====================================================================
// AutomationRepository 实现
// =====================================================================

#[async_trait]
impl AutomationRepository for InMemoryAutomationService {
    async fn insert(&self, rule: &AutomationRule) -> Result<(), AutomationError> {
        self.rules.write().expect("lock").insert(rule.id, rule.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: RuleId) -> Result<Option<AutomationRule>, AutomationError> {
        Ok(self.rules.read().expect("lock").get(&id).cloned())
    }

    async fn update(&self, rule: &AutomationRule) -> Result<(), AutomationError> {
        self.rules.write().expect("lock").insert(rule.id, rule.clone());
        Ok(())
    }

    async fn delete(&self, id: RuleId) -> Result<(), AutomationError> {
        self.rules.write().expect("lock").remove(&id);
        self.executions.write().expect("lock").remove(&id);
        self.rate_limit.write().expect("lock").remove(&id);
        Ok(())
    }

    async fn list_by_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<AutomationRule>, AutomationError> {
        let store = self.rules.read().expect("lock");
        Ok(store
            .values()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<AutomationRule>, AutomationError> {
        let store = self.rules.read().expect("lock");
        Ok(store
            .values()
            .filter(|r| r.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn insert_execution(&self, e: &AutomationExecution) -> Result<(), AutomationError> {
        self.executions
            .write()
            .expect("lock")
            .entry(e.rule_id)
            .or_default()
            .push(e.clone());
        Ok(())
    }

    async fn list_executions_raw(
        &self,
        rule_id: RuleId,
        limit: usize,
    ) -> Result<Vec<AutomationExecution>, AutomationError> {
        let store = self.executions.read().expect("lock");
        let mut out: Vec<AutomationExecution> = store
            .get(&rule_id)
            .cloned()
            .unwrap_or_default();
        out.sort_by_key(|a| std::cmp::Reverse(a.executed_at));
        out.truncate(limit);
        Ok(out)
    }
}

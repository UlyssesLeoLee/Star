//! Automation 规则领域
//!
//! **crate**: `domain-automation`
//! **上游 spec**: docs/specs/domain-automation-spec.md §1-§12
//! **基本设计**: docs/basic-design.md §2.1 (表 17 REQ-AUTO-001) / §5.7 / §11
//! **数据设计**: docs/data-design.md §4.13 (`automation` schema)
//! **API 设计**: docs/api-design.md §3.14 (Automation endpoints)
//!
//! ## 职责
//!
//! 触发器-条件-动作规则(spec §1, REQ-AUTO-001):
//! - 强类型 ID / 值对象(`RuleId` / `ExecutionId` / `TriggerType` / `ActionType` / `ConditionOperator` / `ExecutionResult`)
//! - 5 个核心实体(`AutomationRule` / `AutomationTrigger` / `AutomationCondition` / `AutomationAction` / `AutomationExecution`)
//! - 9 个核心 Domain Event(CloudEvents 1.0,订阅者模式)
//! - 4 个端口(`AutomationCommandPort` × 7 / `AutomationQueryPort` × 4 / `RuleExecutor` / `AutomationRepository`)
//! - 10 条不变量检查(INV-AUTO-01~10)
//! - 1 个 `InMemoryAutomationService` 真实实现
//!
//! ## 关键不变量
//!
//! - 必带 tenant_id,跨 tenant 拒绝(INV-AUTO-01,§6.1)
//! - Trigger event_type 必须在已知列表(INV-AUTO-02,§4.13)
//! - 规则可独立启用/禁用,不影响其他规则(INV-AUTO-03,§4.13)
//! - 规则执行历史 100% 写(成功/失败/跳过)(INV-AUTO-04,§10 AC)
//! - Rule 不得直接执行 Protected 动作(如 pr:merge)(INV-AUTO-05,security-design §3.3)
//! - Action 引用资源须存在(INV-AUTO-06,AU-003)
//! - 循环规则检测(A→B→A)(INV-AUTO-07,AU-004)
//! - 限流防循环(规则执行频率)(INV-AUTO-08,§11)
//! - 名称非空 / actions 非空 / conditions 字段非空(INV-AUTO-09)
//! - 名称在 project 内 UNIQUE(INV-AUTO-10,§4.13 UNIQUE 约束)
//!
//! ## 上游依赖
//!
//! 本 crate 仅依赖自身外部依赖,无跨 domain-* crate 依赖。
//! 逻辑上游: domain-work-item / domain-feedback(提供 trigger 事件),
//! domain-notification(被 Action 触发)。本阶段不强引用,符合骨架 §2.3 禁线。
//!
//! ## 关键引用
//!
//! Rule 由 Worker role=automation 异步执行(§13.4)

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

// =====================================================================
// 子模块装载
// =====================================================================

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

// =====================================================================
// 便捷 re-export
// =====================================================================

pub use context::ActorContext;
pub use entity::{
    AutomationAction, AutomationCondition, AutomationExecution, AutomationRule, AutomationTrigger,
};
pub use error::AutomationError;
pub use event::{
    AutomationEvent, EventMeta, ExecutionRecorded, RuleCreated, RuleDeleted, RuleDisabled,
    RuleEnabled, RuleExecuted, RuleFailed, RuleUpdated, TriggerFired,
};
pub use invariants::{
    check_create_invariants, check_execute_invariants, check_invariant_01_tenant_required,
    check_invariant_02_event_type_known, check_invariant_03_independent_enable_disable,
    check_invariant_04_execution_history_100pct, check_invariant_05_no_protected_action,
    check_invariant_06_action_resource_exists, check_invariant_07_no_cyclic_rule,
    check_invariant_08_rate_limit, check_invariant_09_basic_shape,
    check_invariant_10_name_unique_in_project,
};
pub use port::{
    ActionDraft, AutomationCommandPort, AutomationQueryPort, AutomationRepository, ConditionDraft,
    CreateRuleCommand, FindMatchingRulesQuery, ListExecutionsQuery, ListRuleQuery, RuleExecutor,
    TestRuleRequest, TestRuleResult, TriggerDraft, UpdateRuleCommand,
};
pub use service::InMemoryAutomationService;
pub use value_object::{
    roles, ActionId, ActionType, ConditionId, ConditionOperator, EventId, ExecutionId,
    ExecutionResult, ProjectId, RuleId, TenantId, TriggerId, TriggerType, UserId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::AutomationExecution;
    use std::collections::HashMap;

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new())
    }

    fn make_minimal_create_cmd(tenant_id: TenantId, project_id: ProjectId) -> CreateRuleCommand {
        CreateRuleCommand {
            tenant_id,
            project_id,
            name: "P0 Feedback Notifier".to_string(),
            description: Some("Notify P0 feedback on creation".to_string()),
            trigger: TriggerDraft {
                event_type: "feedback.created".to_string(),
                trigger_type: TriggerType::FeedbackReceived,
                filter: HashMap::new(),
                enabled: true,
            },
            conditions: vec![ConditionDraft {
                field: "severity".to_string(),
                operator: ConditionOperator::Equals,
                value: serde_json::json!("P0"),
            }],
            actions: vec![ActionDraft {
                action_type: ActionType::Notify,
                params: {
                    let mut m = HashMap::new();
                    m.insert("channel_id".to_string(), serde_json::json!("channel_p0_urgent"));
                    m
                },
                order: 0,
            }],
            priority: 100,
            rate_limit_per_minute: 10,
            enabled: true,
        }
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.has_role(roles::PROJECT_ADMIN));
        assert!(actor.is_project_admin());
    }

    // -------- 2. AutomationRule 字段数审计 --------

    #[test]
    fn field_count_audit() {
        assert_eq!(AutomationRule::FIELD_COUNT, 17);
        assert_eq!(AutomationTrigger::FIELD_COUNT, 5);
        assert_eq!(AutomationCondition::FIELD_COUNT, 4);
        assert_eq!(AutomationAction::FIELD_COUNT, 5);
        assert_eq!(AutomationExecution::FIELD_COUNT, 11);
    }

    // -------- 3. bump_version 单调递增 --------

    #[tokio::test]
    async fn bump_version_increments() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        let mut rule = svc
            .create_rule(cmd, actor.clone())
            .await
            .expect("create ok");
        let v1 = rule.lock_version;
        rule.bump_version();
        assert_eq!(rule.lock_version, v1 + 1);
    }

    // -------- 4. create_rule 成功路径 --------

    #[tokio::test]
    async fn create_rule_success() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        let rule = svc.create_rule(cmd, actor).await.expect("create ok");
        assert_eq!(rule.name, "P0 Feedback Notifier");
        assert!(rule.enabled);
        assert_eq!(rule.lock_version, 1);
        assert_eq!(rule.execution_count, 0);
        assert_eq!(rule.actions.len(), 1);
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(svc.count().await, 1);
    }

    // -------- 5. INV-AUTO-01:跨租户访问被拒 --------

    #[tokio::test]
    async fn invariant_01_cross_tenant_denied() {
        let svc = InMemoryAutomationService::new_for_test();
        let tenant_a = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let cmd = make_minimal_create_cmd(tenant_a, ProjectId::new());
        let rule = svc.create_rule(cmd, actor_a).await.unwrap();

        // tenant_b actor 尝试 get
        let tenant_b = TenantId::new();
        let actor_b = make_test_actor(tenant_b);
        let res = svc.get_rule(rule.id, actor_b).await;
        assert!(matches!(res, Err(AutomationError::PermissionDenied)));
    }

    // -------- 6. INV-AUTO-05:Protected 动作拒绝(pr:merge) --------

    #[tokio::test]
    async fn invariant_05_protected_action_rejected() {
        // 通过手工 ActionDraft(故意绕过 ActionType 枚举限制,直接构造
        // 触发 INV-AUTO-05 的 params.protected_action 字段)
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        // 替换为 invoke_webhook + protected_action param
        cmd.actions = vec![ActionDraft {
            action_type: ActionType::InvokeWebhook,
            params: {
                let mut m = HashMap::new();
                m.insert("url".to_string(), serde_json::json!("https://hooks.example.com/automation"));
                m.insert("protected_action".to_string(), serde_json::json!("pr:merge"));
                m
            },
            order: 0,
        }];
        let res = svc.create_rule(cmd, actor).await;
        match res {
            Err(AutomationError::ProtectedAction(_)) => {}
            _ => panic!("expected ProtectedAction error, got: {:?}", res),
        }
    }

    // -------- 7. INV-AUTO-06:Action 引用不存在的资源 --------

    #[tokio::test]
    async fn invariant_06_resource_not_found() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        // channel_id 改为不存在的
        cmd.actions = vec![ActionDraft {
            action_type: ActionType::Notify,
            params: {
                let mut m = HashMap::new();
                m.insert("channel_id".to_string(), serde_json::json!("nonexistent_channel"));
                m
            },
            order: 0,
        }];
        let res = svc.create_rule(cmd, actor).await;
        match res {
            Err(AutomationError::ResourceNotFound(_)) => {}
            _ => panic!("expected ResourceNotFound, got: {:?}", res),
        }
    }

    // -------- 8. INV-AUTO-10:名称在 project 内 UNIQUE --------

    #[tokio::test]
    async fn invariant_10_name_unique_in_project() {
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd1 = make_minimal_create_cmd(tenant_id, project_id);
        svc.create_rule(cmd1, actor.clone()).await.unwrap();

        // 再次创建同名 Rule
        let cmd2 = make_minimal_create_cmd(tenant_id, project_id);
        let res = svc.create_rule(cmd2, actor).await;
        match res {
            Err(AutomationError::Conflict(_)) => {}
            _ => panic!("expected Conflict, got: {:?}", res),
        }
    }

    // -------- 9. INV-AUTO-09:actions 为空被拒 --------

    #[tokio::test]
    async fn invariant_09_empty_actions_rejected() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        cmd.actions = vec![];
        let res = svc.create_rule(cmd, actor).await;
        match res {
            Err(AutomationError::InvalidEventType(_)) => {}
            _ => panic!("expected InvalidEventType, got: {:?}", res),
        }
    }

    // -------- 10. enable / disable 独立切换 --------

    #[tokio::test]
    async fn enable_disable_independent() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        let rule = svc.create_rule(cmd, actor.clone()).await.unwrap();

        // 禁用
        let disabled = svc.disable_rule(rule.id, actor.clone()).await.unwrap();
        assert!(!disabled.enabled);
        assert!(disabled.lock_version > rule.lock_version);

        // 启用
        let enabled = svc.enable_rule(rule.id, actor.clone()).await.unwrap();
        assert!(enabled.enabled);
        assert!(enabled.lock_version > disabled.lock_version);
    }

    // -------- 11. RuleExecutor::evaluate — 触发匹配 + conditions 满足 → Executed --------

    #[tokio::test]
    async fn executor_evaluate_match_and_execute() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        let rule = svc.create_rule(cmd, actor).await.unwrap();

        // 构造触发事件
        let mut payload = HashMap::new();
        payload.insert("severity".to_string(), serde_json::json!("P0"));
        let event_id = EventId::new();
        let results = svc
            .evaluate(event_id, "feedback.created".to_string(), payload)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        let exec = &results[0];
        assert!(exec.matched);
        assert_eq!(exec.result, ExecutionResult::Executed);
        assert_eq!(exec.rule_id, rule.id);
        assert_eq!(exec.executed_action_ids.len(), 1);

        // 历史已写
        assert_eq!(svc.execution_count().await, 1);
    }

    // -------- 12. RuleExecutor::evaluate — condition 不满足 → ConditionsNotMet + 100% 写历史 --------

    #[tokio::test]
    async fn executor_evaluate_condition_not_met_still_records() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        svc.create_rule(cmd, actor).await.unwrap();

        // severity=P1 (不满足 == P0)
        let mut payload = HashMap::new();
        payload.insert("severity".to_string(), serde_json::json!("P1"));
        let results = svc
            .evaluate(EventId::new(), "feedback.created".to_string(), payload)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].matched);
        assert_eq!(results[0].result, ExecutionResult::ConditionsNotMet);
        // INV-AUTO-04: 仍写历史
        assert_eq!(svc.execution_count().await, 1);
    }

    // -------- 13. RuleExecutor::evaluate — trigger 不匹配 → 不评估 --------

    #[tokio::test]
    async fn executor_evaluate_trigger_not_matched() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        svc.create_rule(cmd, actor).await.unwrap();

        // 事件类型 pr.opened(不是 feedback.created)
        let results = svc
            .evaluate(EventId::new(), "pr.opened".to_string(), HashMap::new())
            .await
            .unwrap();
        // 不匹配的 trigger 不产生 execution
        assert_eq!(results.len(), 0);
        assert_eq!(svc.execution_count().await, 0);
    }

    // -------- 14. find_rules_matching_trigger 跨租户隔离 --------

    #[tokio::test]
    async fn find_matching_trigger_tenant_isolated() {
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor_a = make_test_actor(tenant_a);
        let cmd = make_minimal_create_cmd(tenant_a, ProjectId::new());
        svc.create_rule(cmd, actor_a).await.unwrap();

        // tenant_b 查询应返回 0
        let q = FindMatchingRulesQuery {
            tenant_id: tenant_b,
            event_type: "feedback.created".to_string(),
            event_payload: HashMap::new(),
        };
        let results = svc.find_rules_matching_trigger(q).await.unwrap();
        assert_eq!(results.len(), 0);

        // tenant_a 查询应返回 1
        let q = FindMatchingRulesQuery {
            tenant_id: tenant_a,
            event_type: "feedback.created".to_string(),
            event_payload: HashMap::new(),
        };
        let results = svc.find_rules_matching_trigger(q).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    // -------- 15. 事件总线收到 Created 事件 --------

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryAutomationService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        svc.create_rule(cmd, actor).await.unwrap();

        let mut found_created = false;
        for _ in 0..10 {
            if let Ok(evt) = rx.try_recv() {
                if matches!(evt, AutomationEvent::Created(_)) {
                    found_created = true;
                    break;
                }
            }
        }
        assert!(found_created, "应收到 Created 事件");
    }

    // -------- 16. test_rule 模拟执行 --------

    #[tokio::test]
    async fn test_rule_match_and_mismatch() {
        let tenant_id = TenantId::new();
        let svc = InMemoryAutomationService::new_for_test();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id, ProjectId::new());
        let rule = svc.create_rule(cmd, actor.clone()).await.unwrap();

        // 匹配场景
        let mut payload = HashMap::new();
        payload.insert("severity".to_string(), serde_json::json!("P0"));
        let res = svc
            .test_rule(
                TestRuleRequest {
                    rule_id: rule.id,
                    tenant_id,
                    sample_event_type: "feedback.created".to_string(),
                    sample_event_payload: payload,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert!(res.matched);
        assert_eq!(res.would_execute_actions.len(), 1);

        // 不匹配场景(severity=P1)
        let mut payload = HashMap::new();
        payload.insert("severity".to_string(), serde_json::json!("P1"));
        let res = svc
            .test_rule(
                TestRuleRequest {
                    rule_id: rule.id,
                    tenant_id,
                    sample_event_type: "feedback.created".to_string(),
                    sample_event_payload: payload,
                },
                actor,
            )
            .await
            .unwrap();
        assert!(!res.matched);
    }
}

//! Workflow 领域
//!
//! **crate**: `domain-workflow`
//! **上游 spec**: docs/specs/domain-workflow-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.9.3 / §7.6
//! **数据设计**: docs/data-design.md §4.5 (`workflow` schema)
//! **API 设计**: docs/api-design.md §3.6 (Workflow Definition + Transition)
//!
//! ## 职责
//!
//! WorkflowDefinition 聚合根 + WorkItem 状态机定义(§4.9.3, INV-WI-09):
//! - 强类型 ID / 值对象(`WorkflowId` / `StateId` / `TransitionId` / `StateCategory`)
//! - 3 个核心实体(`WorkflowDefinition` / `State` / `Transition`)
//! - 5 个核心 Domain Event(CloudEvents 1.0)
//! - 2 个端口(`WorkflowCommandPort` × 5 方法 / `WorkflowQueryPort` × 5 方法) + 1 个仓库端口
//! - 6 条不变量检查(INV-WF-01~06)
//! - 1 个 `InMemoryWorkflowService` 真实实现(已 seed system_default 三态)
//!
//! ## 关键不变量
//!
//! - system_default Workflow 平台级只读(INV-WF-01,§7.2)
//! - 每个 Workflow 必含一个且唯一 Initial State(INV-WF-02,§4.5)
//! - Transition 必须 from ≠ to(INV-WF-03,§4.5)
//! - State 名同 Workflow 内 UNIQUE(INV-WF-04,§4.5)
//! - 删除 Workflow 需级联检查 Project Policy 引用(INV-WF-05,§5.7)
//! - 自定义 Workflow 必须含 system default 三态 TODO/IN_PROGRESS/DONE(INV-WF-06,REQ-WF-001)
//!
//! ## 上游依赖
//!
//! 本 crate 仅依赖自身外部依赖,无跨 domain-* crate 依赖。
//!
//! ## 关键引用
//!
//! WorkItem 默认三态由本 crate system_default WorkflowDefinition 判定(§4.9.3, INV-WI-09)

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
pub use entity::{State, SystemDefault, Transition, WorkflowDefinition};
pub use error::WorkflowError;
pub use event::{
    EventMeta, StateAdded, TransitionAdded, WorkflowCreated, WorkflowDeleted, WorkflowEvent,
    WorkflowUpdated,
};
pub use invariants::{
    check_create_invariants, check_invariant_01_system_default_readonly,
    check_invariant_02_initial_state_unique, check_invariant_03_transition_distinct,
    check_invariant_04_state_name_unique, check_invariant_05_no_project_reference,
    check_invariant_06_inherit_default_states, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    AddStateCommand, AddTransitionCommand, CreateWorkflowCommand, ListStatesQuery,
    ListTransitionsQuery, StateDraft, TransitionDraft, UpdateWorkflowCommand,
    WorkflowCommandPort, WorkflowQueryPort, WorkflowRepository,
};
pub use service::InMemoryWorkflowService;
pub use value_object::{
    roles, ProjectId, StateCategory, StateId, TenantId, TransitionId, UserId, WorkflowId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{ProjectId, StateCategory, StateId, TenantId, UserId, WorkflowId};

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new())
    }

    fn make_minimal_create_cmd(tenant_id: TenantId) -> CreateWorkflowCommand {
        let sd_todo = uuid::Uuid::new_v4();
        let sd_in_progress = uuid::Uuid::new_v4();
        let sd_done = uuid::Uuid::new_v4();
        CreateWorkflowCommand {
            tenant_id,
            project_id: Some(ProjectId::new()),
            name: "Test Workflow".to_string(),
            description: Some("Test description".to_string()),
            initial_state_draft_id: sd_todo,
            states: vec![
                StateDraft {
                    draft_id: sd_todo,
                    name: "TODO".to_string(),
                    category: StateCategory::Initial,
                    display_color: Some("#999".to_string()),
                    display_order: 0,
                },
                StateDraft {
                    draft_id: sd_in_progress,
                    name: "IN_PROGRESS".to_string(),
                    category: StateCategory::Intermediate,
                    display_color: Some("#06c".to_string()),
                    display_order: 1,
                },
                StateDraft {
                    draft_id: sd_done,
                    name: "DONE".to_string(),
                    category: StateCategory::Terminal,
                    display_color: Some("#0a6".to_string()),
                    display_order: 2,
                },
            ],
            transitions: vec![TransitionDraft {
                from_draft_id: sd_todo,
                to_draft_id: sd_in_progress,
                required_permission: None,
                required_role: None,
                trigger_event: None,
            }],
        }
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.has_role(roles::PROJECT_ADMIN));
    }

    // -------- 2. WorkflowDefinition 字段数审计 --------

    #[test]
    fn workflow_field_count_audit() {
        assert_eq!(WorkflowDefinition::FIELD_COUNT, 13);
        assert_eq!(State::FIELD_COUNT, 9);
        assert_eq!(Transition::FIELD_COUNT, 8);
    }

    // -------- 3. system_default 已 seed --------

    #[tokio::test]
    async fn system_default_seeded() {
        let svc = InMemoryWorkflowService::new_for_test();
        let sd = svc.get_system_default().await.expect("system_default 已 seed");
        assert!(sd.is_system_default);
        assert_eq!(sd.name, "system_default");
        assert!(sd.lock_version >= 1);
        // 验证 system_default 3 态
        let viewer = ActorContext::new(UserId::new(), TenantId::new()).with_role(roles::PLATFORM_OPERATOR);
        let q = ListStatesQuery {
            tenant_id: TenantId::new(),
            workflow_id: sd.id,
        };
        let states = svc.list_states(q, viewer).await.expect("list states");
        let names: Vec<&str> = states.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"TODO"));
        assert!(names.contains(&"IN_PROGRESS"));
        assert!(names.contains(&"DONE"));
    }

    // -------- 4. create_workflow 成功路径 --------

    #[tokio::test]
    async fn create_workflow_success() {
        let svc = InMemoryWorkflowService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id);
        let wf = svc
            .create_workflow(cmd, actor)
            .await
            .expect("创建成功");
        assert!(!wf.is_system_default);
        assert_eq!(wf.name, "Test Workflow");
        assert_eq!(wf.lock_version, 1);
        assert_eq!(svc.count().await, 2); // system_default + 1
    }

    // -------- 5. INV-WF-06:缺少 system default 三态被拒 --------

    #[tokio::test]
    async fn invariant_06_missing_default_states() {
        let tenant_id = TenantId::new();
        // 只提供 1 个 State(TODO)
        let sd_todo = uuid::Uuid::new_v4();
        let cmd = CreateWorkflowCommand {
            tenant_id,
            project_id: Some(ProjectId::new()),
            name: "Incomplete".to_string(),
            description: None,
            initial_state_draft_id: sd_todo,
            states: vec![StateDraft {
                draft_id: sd_todo,
                name: "TODO".to_string(),
                category: StateCategory::Initial,
                display_color: None,
                display_order: 0,
            }],
            transitions: vec![],
        };
        let actor = make_test_actor(tenant_id);
        let svc = InMemoryWorkflowService::new_for_test();
        let res = svc.create_workflow(cmd, actor).await;
        assert!(matches!(res, Err(WorkflowError::InvalidState(_))));
    }

    // -------- 6. INV-WF-04:State 名重复被拒 --------

    #[tokio::test]
    async fn invariant_04_duplicate_state_name() {
        let tenant_id = TenantId::new();
        let s1 = uuid::Uuid::new_v4();
        let s2 = uuid::Uuid::new_v4();
        let s3 = uuid::Uuid::new_v4();
        // s1 和 s2 名为 "TODO",重复
        let cmd = CreateWorkflowCommand {
            tenant_id,
            project_id: Some(ProjectId::new()),
            name: "Dup".to_string(),
            description: None,
            initial_state_draft_id: s1,
            states: vec![
                StateDraft {
                    draft_id: s1,
                    name: "TODO".to_string(),
                    category: StateCategory::Initial,
                    display_color: None,
                    display_order: 0,
                },
                StateDraft {
                    draft_id: s2,
                    name: "TODO".to_string(), // 重复
                    category: StateCategory::Intermediate,
                    display_color: None,
                    display_order: 1,
                },
                StateDraft {
                    draft_id: s3,
                    name: "DONE".to_string(),
                    category: StateCategory::Terminal,
                    display_color: None,
                    display_order: 2,
                },
            ],
            transitions: vec![],
        };
        let actor = make_test_actor(tenant_id);
        let svc = InMemoryWorkflowService::new_for_test();
        let res = svc.create_workflow(cmd, actor).await;
        assert!(matches!(res, Err(WorkflowError::Conflict(_))));
    }

    // -------- 7. INV-WF-01:更新 system_default 被拒 --------

    #[tokio::test]
    async fn invariant_01_update_system_default_rejected() {
        let svc = InMemoryWorkflowService::new_for_test();
        let sd = svc.get_system_default().await.unwrap();
        let viewer = ActorContext::new(UserId::new(), sd.tenant_id).with_role(roles::PLATFORM_OPERATOR);
        let res = svc
            .update_workflow(
                UpdateWorkflowCommand {
                    workflow_id: sd.id,
                    tenant_id: sd.tenant_id,
                    expected_version: sd.lock_version,
                    name: Some("Hacked".to_string()),
                    description: None,
                },
                viewer,
            )
            .await;
        assert!(matches!(res, Err(WorkflowError::InvalidState(_))));
    }

    // -------- 8. validate_transition 状态机合法/非法判定 --------

    #[tokio::test]
    async fn validate_transition_legality() {
        let svc = InMemoryWorkflowService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id);
        let wf = svc.create_workflow(cmd, actor.clone()).await.unwrap();

        let viewer = actor;
        let states_q = ListStatesQuery {
            tenant_id,
            workflow_id: wf.id,
        };
        let states = svc.list_states(states_q, viewer.clone()).await.unwrap();
        let todo = states.iter().find(|s| s.name == "TODO").unwrap();
        let in_progress = states.iter().find(|s| s.name == "IN_PROGRESS").unwrap();
        let done = states.iter().find(|s| s.name == "DONE").unwrap();

        // 合法的 TODO → IN_PROGRESS
        assert!(svc
            .validate_transition(wf.id, todo.id, in_progress.id)
            .await
            .unwrap());
        // 非法的 TODO → DONE(没有此 transition)
        assert!(!svc
            .validate_transition(wf.id, todo.id, done.id)
            .await
            .unwrap());
    }

    // -------- 9. INV-WF-05:被 Project 引用时删除被拒 --------

    #[tokio::test]
    async fn invariant_05_project_reference_blocks_delete() {
        let svc = InMemoryWorkflowService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id);
        let wf = svc.create_workflow(cmd, actor.clone()).await.unwrap();

        // 注册 Project 引用
        let project_uuid = uuid::Uuid::new_v4();
        svc.add_project_reference(project_uuid, wf.id).await;

        // 尝试删除,应该被拒
        let res = svc.delete_workflow(wf.id, actor).await;
        assert!(matches!(res, Err(WorkflowError::Conflict(_))));

        // 移除引用后删除成功
        svc.remove_project_reference(project_uuid, wf.id).await;
        let actor2 = make_test_actor(tenant_id);
        let res2 = svc.delete_workflow(wf.id, actor2).await;
        assert!(res2.is_ok());
    }

    // -------- 10. add_state + add_transition 成功路径 --------

    #[tokio::test]
    async fn add_state_and_transition() {
        let svc = InMemoryWorkflowService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id);
        let wf = svc.create_workflow(cmd, actor.clone()).await.unwrap();

        // 添加 IN_REVIEW state
        let new_state = svc
            .add_state(
                AddStateCommand {
                    workflow_id: wf.id,
                    tenant_id,
                    name: "IN_REVIEW".to_string(),
                    category: StateCategory::Intermediate,
                    display_color: Some("#ff0".to_string()),
                    display_order: 3,
                },
                actor.clone(),
            )
            .await
            .expect("add_state 成功");

        // 添加 IN_PROGRESS → IN_REVIEW transition
        let states_q = ListStatesQuery {
            tenant_id,
            workflow_id: wf.id,
        };
        let viewer = actor.clone();
        let states = svc.list_states(states_q, viewer).await.unwrap();
        let in_progress = states.iter().find(|s| s.name == "IN_PROGRESS").unwrap();
        let t = svc
            .add_transition(
                AddTransitionCommand {
                    workflow_id: wf.id,
                    tenant_id,
                    from_state_id: in_progress.id,
                    to_state_id: new_state.id,
                    required_permission: None,
                    required_role: None,
                    trigger_event: None,
                },
                actor,
            )
            .await
            .expect("add_transition 成功");
        assert_eq!(t.from_state_id, in_progress.id);
        assert_eq!(t.to_state_id, new_state.id);
    }

    // -------- 11. 跨租户访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryWorkflowService::new_for_test();
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let cmd = make_minimal_create_cmd(tenant_a);
        let wf = svc.create_workflow(cmd, actor_a).await.unwrap();

        let actor_b = ActorContext::new(UserId::new(), tenant_b).with_role(roles::PROJECT_ADMIN);
        let res = svc.get_by_id(wf.id, actor_b).await;
        assert!(matches!(res, Err(WorkflowError::PermissionDenied)));
    }

    // -------- 12. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryWorkflowService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_minimal_create_cmd(tenant_id);
        svc.create_workflow(cmd, actor).await.unwrap();
        // 找到第一个非 system_default 的事件
        let mut found_created = false;
        for _ in 0..10 {
            if let Ok(evt) = rx.try_recv() {
                if matches!(evt, WorkflowEvent::Created(_)) {
                    found_created = true;
                    break;
                }
            }
        }
        assert!(found_created, "应收到 Created 事件");
    }
}

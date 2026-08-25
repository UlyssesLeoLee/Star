//! Sprint / Backlog / Roadmap 规划领域
//!
//! **crate**: `domain-planning`
//! **上游 spec**: docs/specs/domain-planning-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.9.2 / §4.9.4
//! **数据设计**: docs/data-design.md §4.7 (`planning` schema)
//! **API 设计**: docs/api-design.md §3.8 (Sprint / Backlog / Roadmap)
//!
//! ## 职责
//!
//! 敏捷规划核心数据(§9, REQ-PLAN-001~005):
//! - 5 个核心实体(`Sprint` / `Backlog` / `Roadmap` / `Milestone` / `BurndownSnapshot`)
//! - 5 个核心 Domain Event(CloudEvents 1.0)
//! - 2 个端口(`PlanningCommandPort` × 7 / `PlanningQueryPort` × 5) + 1 个仓库端口
//! - 6 条不变量(INV-PL-01~06)
//! - 1 个 `InMemoryPlanningService` 真实实现
//!
//! ## 关键不变量
//!
//! - Sprint 状态机 Planning → Active → Closed 不可逆(INV-PL-01,§4.9.2)
//! - Sprint 时长 1-4 周(INV-PL-02,REQ-PLAN-001)
//! - 同 Project 同时刻最多 1 个 Active Sprint(INV-PL-03,§4.9.4)
//! - Burndown 是 Projection(INV-PL-05,§5.7)

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
pub use entity::{Backlog, BurndownReport, BurndownSnapshot, Milestone, Roadmap, Sprint};
pub use error::PlanningError;
pub use event::{
    BacklogReordered, EventMeta, PlanningEvent, SprintClosed, SprintCreated, SprintStarted,
    WorkItemAddedToSprint,
};
pub use invariants::{
    check_create_invariants, check_invariant_01_sprint_state_legal,
    check_invariant_02_sprint_duration, check_invariant_03_single_active_sprint,
    check_invariant_04_no_duplicate_work_item, check_invariant_05_burndown_projection_placeholder,
    check_invariant_06_backlog_no_duplicates, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    AddWorkItemToSprintCommand, BacklogReorderCommand, CloseSprintCommand, CreateSprintCommand,
    ListSprintQuery, PlanningCommandPort, PlanningQueryPort, PlanningRepository,
    RemoveWorkItemFromSprintCommand, UpdateSprintCommand,
};
pub use service::InMemoryPlanningService;
pub use value_object::{
    roles, BacklogId, BurndownSnapshotId, CloseMoveTarget, MilestoneId, ProjectId, RoadmapId,
    SprintId, SprintState, TenantId, UserId, WorkItemId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{CloseMoveTarget, ProjectId, SprintState, TenantId, UserId, WorkItemId};
    use chrono::{Duration, Utc};

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new())
    }

    fn make_create_cmd(tenant_id: TenantId, project_id: ProjectId) -> CreateSprintCommand {
        let now = Utc::now();
        CreateSprintCommand {
            tenant_id,
            project_id,
            name: "Sprint 1".to_string(),
            goal: Some("Ship MVP".to_string()),
            start_at: now,
            end_at: now + Duration::days(14),
            capacity_story_points: Some(50),
        }
    }

    // -------- 1. ActorContext smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.has_role(roles::PROJECT_ADMIN));
    }

    // -------- 2. 字段数审计 --------

    #[test]
    fn field_count_audit() {
        assert_eq!(Sprint::FIELD_COUNT, 15);
        assert_eq!(Backlog::FIELD_COUNT, 8);
        assert_eq!(Roadmap::FIELD_COUNT, 9);
        assert_eq!(Milestone::FIELD_COUNT, 8);
    }

    // -------- 3. create_sprint 成功路径 --------

    #[tokio::test]
    async fn create_sprint_success() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, project_id);
        let sprint = svc
            .create_sprint(cmd, actor)
            .await
            .expect("创建成功");
        assert_eq!(sprint.state, SprintState::Planning);
        assert_eq!(sprint.lock_version, 1);
        assert_eq!(svc.count_sprints().await, 1);
    }

    // -------- 4. INV-PL-02:时长 1-4 周 --------

    #[tokio::test]
    async fn invariant_02_sprint_duration() {
        // 5 天 < 1 周 → InvalidState
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let now = Utc::now();
        let cmd = CreateSprintCommand {
            tenant_id,
            project_id,
            name: "Short".to_string(),
            goal: None,
            start_at: now,
            end_at: now + Duration::days(5), // < 1 周
            capacity_story_points: None,
        };
        let svc = InMemoryPlanningService::new_for_test();
        let res = svc.create_sprint(cmd, actor).await;
        assert!(matches!(res, Err(PlanningError::InvalidState(_))));

        // 6 周 > 4 周 → InvalidState
        let actor2 = make_test_actor(tenant_id);
        let now2 = Utc::now();
        let cmd2 = CreateSprintCommand {
            tenant_id,
            project_id,
            name: "Long".to_string(),
            goal: None,
            start_at: now2,
            end_at: now2 + Duration::days(42),
            capacity_story_points: None,
        };
        let res2 = svc.create_sprint(cmd2, actor2).await;
        assert!(matches!(res2, Err(PlanningError::InvalidState(_))));
    }

    // -------- 5. INV-PL-01 + start_sprint 状态机迁移 --------

    #[tokio::test]
    async fn start_sprint_success() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, project_id);
        let sprint = svc.create_sprint(cmd, actor.clone()).await.unwrap();
        let started = svc
            .start_sprint(sprint.id, actor)
            .await
            .expect("启动成功");
        assert_eq!(started.state, SprintState::Active);
        assert!(started.started_at.is_some());
    }

    // -------- 6. INV-PL-01:已 Closed 不可再启动 --------

    #[tokio::test]
    async fn invariant_01_closed_sprint_cannot_restart() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, project_id);
        let sprint = svc.create_sprint(cmd, actor.clone()).await.unwrap();
        // 启动
        let s = svc
            .start_sprint(sprint.id, actor.clone())
            .await
            .unwrap();
        // 关闭
        let s = svc
            .close_sprint(
                s.id,
                CloseSprintCommand {
                    move_incomplete_to: CloseMoveTarget::Backlog,
                    next_sprint_id: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert_eq!(s.state, SprintState::Closed);
        // 尝试再次启动 → InvalidState
        let res = svc.start_sprint(s.id, actor).await;
        assert!(matches!(res, Err(PlanningError::InvalidState(_))));
    }

    // -------- 7. INV-PL-03:同 Project 同时刻只能 1 Active --------

    #[tokio::test]
    async fn invariant_03_single_active_sprint() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);

        // 创建并启动第一个 sprint
        let s1 = svc
            .create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();
        svc.start_sprint(s1.id, actor.clone()).await.unwrap();

        // 创建第二个 sprint
        let s2 = svc
            .create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();
        // 尝试启动第二个 → 冲突
        let res = svc.start_sprint(s2.id, actor).await;
        assert!(matches!(res, Err(PlanningError::Conflict(_))));
    }

    // -------- 8. add/remove work_item to sprint --------

    #[tokio::test]
    async fn add_remove_work_item_to_sprint() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, project_id);
        let sprint = svc.create_sprint(cmd, actor.clone()).await.unwrap();

        let wi1 = WorkItemId::new();
        let s = svc
            .add_work_item_to_sprint(
                AddWorkItemToSprintCommand {
                    sprint_id: sprint.id,
                    tenant_id,
                    work_item_id: wi1,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert_eq!(s.work_item_ids.len(), 1);
        assert_eq!(s.work_item_ids[0], wi1);

        // 重复添加 → Conflict
        let res = svc
            .add_work_item_to_sprint(
                AddWorkItemToSprintCommand {
                    sprint_id: sprint.id,
                    tenant_id,
                    work_item_id: wi1,
                },
                actor.clone(),
            )
            .await;
        assert!(matches!(res, Err(PlanningError::Conflict(_))));

        // 移除
        let s = svc
            .remove_work_item_from_sprint(
                RemoveWorkItemFromSprintCommand {
                    sprint_id: sprint.id,
                    tenant_id,
                    work_item_id: wi1,
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(s.work_item_ids.len(), 0);
    }

    // -------- 9. list_sprints 过滤 --------

    #[tokio::test]
    async fn list_sprints_filter_by_state() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let s1 = svc
            .create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();
        svc.start_sprint(s1.id, actor.clone()).await.unwrap();
        // 第二个 planning
        svc.create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();

        let q = ListSprintQuery {
            tenant_id,
            project_id: Some(project_id),
            state: Some(SprintState::Active),
            limit: 10,
            offset: 0,
        };
        let viewer = make_test_actor(tenant_id);
        let active = svc.list_sprints(q, viewer).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, s1.id);
    }

    // -------- 10. update_sprint 乐观锁 --------

    #[tokio::test]
    async fn update_sprint_version_conflict() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let sprint = svc
            .create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();
        let res = svc
            .update_sprint(
                UpdateSprintCommand {
                    sprint_id: sprint.id,
                    tenant_id,
                    expected_version: 99,
                    name: Some("X".to_string()),
                    goal: None,
                    start_at: None,
                    end_at: None,
                    capacity_story_points: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(PlanningError::Conflict(_))));
    }

    // -------- 11. close_sprint 带 move target --------

    #[tokio::test]
    async fn close_sprint_with_move_target() {
        let svc = InMemoryPlanningService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let sprint = svc
            .create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();
        svc.start_sprint(sprint.id, actor.clone()).await.unwrap();
        let closed = svc
            .close_sprint(
                sprint.id,
                CloseSprintCommand {
                    move_incomplete_to: CloseMoveTarget::NextSprint,
                    next_sprint_id: Some(SprintId::new()),
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(closed.state, SprintState::Closed);
        assert!(closed.closed_at.is_some());
    }

    // -------- 12. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_sprint_lifecycle() {
        let (svc, mut rx) = InMemoryPlanningService::new();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id);
        let sprint = svc
            .create_sprint(make_create_cmd(tenant_id, project_id), actor.clone())
            .await
            .unwrap();
        svc.start_sprint(sprint.id, actor.clone()).await.unwrap();
        svc.close_sprint(
            sprint.id,
            CloseSprintCommand {
                move_incomplete_to: CloseMoveTarget::Backlog,
                next_sprint_id: None,
            },
            actor,
        )
        .await
        .unwrap();

        let mut events = Vec::new();
        for _ in 0..10 {
            if let Ok(e) = rx.try_recv() {
                events.push(e);
            }
        }
        // 至少 3 个事件:Created, Started, Closed
        let kinds: Vec<&str> = events.iter().map(|e| e.subject()).collect();
        assert!(kinds.iter().any(|s| s.contains("created")));
        assert!(kinds.iter().any(|s| s.contains("started")));
        assert!(kinds.iter().any(|s| s.contains("closed")));
    }
}

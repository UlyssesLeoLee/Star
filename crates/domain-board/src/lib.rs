//! Board 视图领域
//!
//! **crate**: `domain-board`
//! **上游 spec**: docs/specs/domain-board-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.9.2 / §4.9.4
//! **数据设计**: docs/data-design.md §4.6 (`board` schema)
//! **API 设计**: docs/api-design.md §3.7 (Board / Column / Swimlane)
//!
//! ## 职责
//!
//! Kanban / Scrum 板视图配置(§9,REQ-PLAN-003),Board 与 Sprint / Gantt 共享数据模型:
//! - 3 个核心实体(`Board` / `Column` / `Swimlane`)
//! - 3 个核心 Domain Event(CloudEvents 1.0)
//! - 2 个端口(`BoardCommandPort` × 3 方法 / `BoardQueryPort` × 3 方法) + 1 个仓库端口
//! - 5 条不变量检查(INV-B-01~05)
//! - 1 个 `InMemoryBoardService` 真实实现
//!
//! ## 关键不变量
//!
//! - Board 必须属一个 Project(INV-B-01,§6.1)
//! - Column.state_id 必引用存在的 Workflow State(INV-B-02,§4.5)
//! - Column display_order 同 Board 内 UNIQUE(INV-B-03,§4.6)
//! - Board 视图不存业务事实(INV-B-04,§5.7)
//! - WIP 限制是软告警(INV-B-05,§4.9.4)
//!
//! ## 上游依赖
//!
//! 本 crate 仅依赖自身外部依赖,无跨 domain-* crate 依赖。
//!
//! ## 关键引用
//!
//! Column 引用 WorkflowStateID(本 crate 用强类型 StateId,不直接依赖 domain-workflow)

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
pub use entity::{Board, Column, Swimlane};
pub use error::BoardError;
pub use event::{
    BoardEvent, BoardPatched, BoardReplaced, ColumnReordered, EventMeta,
};
pub use invariants::{
    check_create_invariants, check_invariant_01_board_has_project,
    check_invariant_02_column_state_exists, check_invariant_03_display_order_unique,
    check_invariant_04_swimlane_group_by_valid, check_invariant_05_wip_limit_positive,
    run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    BoardCommandPort, BoardQueryPort, BoardRepository, ColumnDraft, ColumnOrderUpdate,
    ListColumnsQuery, ListSwimlanesQuery, PatchBoardCommand, ReplaceBoardCommand, SwimlaneDraft,
};
pub use service::InMemoryBoardService;
pub use value_object::{
    roles, BoardId, BoardType, ColumnId, GroupByField, ProjectId, StateId, SwimlaneId, TenantId,
    UserId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{GroupByField, StateId};

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new())
    }

    fn make_replace_cmd(tenant_id: TenantId, project_id: ProjectId, states: &[StateId]) -> ReplaceBoardCommand {
        let c1 = uuid::Uuid::new_v4();
        let c2 = uuid::Uuid::new_v4();
        let c3 = uuid::Uuid::new_v4();
        let s1 = uuid::Uuid::new_v4();
        ReplaceBoardCommand {
            tenant_id,
            project_id,
            board_type: BoardType::Kanban,
            name: "Sprint Board".to_string(),
            description: Some("Default sprint board".to_string()),
            filter_assignee: None,
            filter_label: None,
            columns: vec![
                ColumnDraft {
                    draft_id: c1,
                    name: "TODO".to_string(),
                    state_id: states[0],
                    display_order: 0,
                    wip_limit: Some(5),
                    display_color: Some("#999".to_string()),
                },
                ColumnDraft {
                    draft_id: c2,
                    name: "IN_PROGRESS".to_string(),
                    state_id: states[1],
                    display_order: 1,
                    wip_limit: Some(3),
                    display_color: Some("#06c".to_string()),
                },
                ColumnDraft {
                    draft_id: c3,
                    name: "DONE".to_string(),
                    state_id: states[2],
                    display_order: 2,
                    wip_limit: None,
                    display_color: Some("#0a6".to_string()),
                },
            ],
            swimlanes: vec![SwimlaneDraft {
                draft_id: s1,
                name: "By Assignee".to_string(),
                group_by_field: GroupByField::Assignee,
                display_order: 0,
            }],
            expected_version: 0,
        }
    }

    fn make_test_states() -> Vec<StateId> {
        vec![StateId::new(), StateId::new(), StateId::new()]
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.has_role(roles::PROJECT_ADMIN));
    }

    // -------- 2. 字段数审计 --------

    #[test]
    fn board_field_count_audit() {
        assert_eq!(Board::FIELD_COUNT, 12);
        assert_eq!(Column::FIELD_COUNT, 10);
        assert_eq!(Swimlane::FIELD_COUNT, 8);
    }

    // -------- 3. replace_board 成功路径(创建) --------

    #[tokio::test]
    async fn replace_board_create_success() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;

        let actor = make_test_actor(tenant_id);
        let cmd = make_replace_cmd(tenant_id, project_id, &states);
        let board = svc
            .replace_board(cmd, actor.clone())
            .await
            .expect("创建成功");
        assert_eq!(board.project_id, project_id);
        assert_eq!(board.lock_version, 1);
        assert_eq!(svc.count().await, 1);

        // Column / Swimlane 可查
        let q = ListColumnsQuery {
            tenant_id,
            board_id: board.id,
        };
        let cols = svc.list_columns(q, actor).await.unwrap();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name, "TODO");
    }

    // -------- 4. INV-B-01:同 Project 重复 Board 被拒 --------

    #[tokio::test]
    async fn invariant_01_duplicate_project_board() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;
        let actor = make_test_actor(tenant_id);

        let cmd1 = make_replace_cmd(tenant_id, project_id, &states);
        svc.replace_board(cmd1, actor.clone()).await.unwrap();
        let cmd2 = make_replace_cmd(tenant_id, project_id, &states);
        let res = svc.replace_board(cmd2, actor).await;
        assert!(matches!(res, Err(BoardError::Conflict(_))));
    }

    // -------- 5. INV-B-02:Column.state_id 引用不存在被拒 --------

    #[tokio::test]
    async fn invariant_02_column_state_not_registered() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        // 不注册任何 state_id
        let states = make_test_states();
        let actor = make_test_actor(tenant_id);
        let cmd = make_replace_cmd(tenant_id, project_id, &states);
        let res = svc.replace_board(cmd, actor).await;
        assert!(matches!(res, Err(BoardError::InvalidState(_))));
    }

    // -------- 6. INV-B-03:Column display_order 重复被拒 --------

    #[tokio::test]
    async fn invariant_03_duplicate_display_order() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;
        let actor = make_test_actor(tenant_id);

        // 构造 2 个 Column 同 display_order
        let c1 = uuid::Uuid::new_v4();
        let c2 = uuid::Uuid::new_v4();
        let cmd = ReplaceBoardCommand {
            tenant_id,
            project_id,
            board_type: BoardType::Kanban,
            name: "Dup".to_string(),
            description: None,
            filter_assignee: None,
            filter_label: None,
            columns: vec![
                ColumnDraft {
                    draft_id: c1,
                    name: "Col1".to_string(),
                    state_id: states[0],
                    display_order: 0, // dup
                    wip_limit: None,
                    display_color: None,
                },
                ColumnDraft {
                    draft_id: c2,
                    name: "Col2".to_string(),
                    state_id: states[1],
                    display_order: 0, // dup
                    wip_limit: None,
                    display_color: None,
                },
            ],
            swimlanes: vec![],
            expected_version: 0,
        };
        let res = svc.replace_board(cmd, actor).await;
        assert!(matches!(res, Err(BoardError::Conflict(_))));
    }

    // -------- 7. patch_board 部分更新 --------

    #[tokio::test]
    async fn patch_board_success() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;
        let actor = make_test_actor(tenant_id);
        let cmd = make_replace_cmd(tenant_id, project_id, &states);
        let board = svc.replace_board(cmd, actor.clone()).await.unwrap();

        let patched = svc
            .patch_board(
                PatchBoardCommand {
                    board_id: board.id,
                    tenant_id,
                    expected_version: 1,
                    name: Some("Renamed".to_string()),
                    description: None,
                    board_type: None,
                    filter_assignee: None,
                    filter_label: None,
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(patched.name, "Renamed");
        assert_eq!(patched.lock_version, 2);
    }

    // -------- 8. patch_board 乐观锁冲突 --------

    #[tokio::test]
    async fn patch_board_version_conflict() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;
        let actor = make_test_actor(tenant_id);
        let cmd = make_replace_cmd(tenant_id, project_id, &states);
        let board = svc.replace_board(cmd, actor.clone()).await.unwrap();
        let res = svc
            .patch_board(
                PatchBoardCommand {
                    board_id: board.id,
                    tenant_id,
                    expected_version: 99,
                    name: Some("X".to_string()),
                    description: None,
                    board_type: None,
                    filter_assignee: None,
                    filter_label: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(BoardError::Conflict(_))));
    }

    // -------- 9. reorder_columns 成功路径 --------

    #[tokio::test]
    async fn reorder_columns_success() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;
        let actor = make_test_actor(tenant_id);
        let cmd = make_replace_cmd(tenant_id, project_id, &states);
        let board = svc.replace_board(cmd, actor.clone()).await.unwrap();

        let q = ListColumnsQuery {
            tenant_id,
            board_id: board.id,
        };
        let cols_before = svc.list_columns(q, viewer_for(tenant_id)).await.unwrap();
        assert_eq!(cols_before.len(), 3);

        // 交换第一列与第二列的 order
        let new_orders = vec![
            (cols_before[0].id, 1u32),
            (cols_before[1].id, 0u32),
        ];
        let actor2 = make_test_actor(tenant_id);
        let cols_after = svc
            .reorder_columns(
                ColumnOrderUpdate {
                    board_id: board.id,
                    tenant_id,
                    new_orders,
                },
                actor2,
            )
            .await
            .unwrap();
        assert_eq!(cols_after[0].id, cols_before[1].id);
        assert_eq!(cols_after[1].id, cols_before[0].id);
    }

    // -------- 10. 跨租户访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryBoardService::new_for_test();
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;

        let actor_a = make_test_actor(tenant_a);
        let cmd = make_replace_cmd(tenant_a, project_id, &states);
        svc.replace_board(cmd, actor_a).await.unwrap();

        let actor_b = ActorContext::new(UserId::new(), tenant_b).with_role(roles::PROJECT_ADMIN);
        let res = svc.get_by_project(project_id, actor_b).await;
        assert!(matches!(res, Err(BoardError::PermissionDenied)));
    }

    // -------- 11. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_replaced() {
        let (svc, mut rx) = InMemoryBoardService::new();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let states = make_test_states();
        svc.register_valid_states(states.clone()).await;
        let actor = make_test_actor(tenant_id);
        let cmd = make_replace_cmd(tenant_id, project_id, &states);
        svc.replace_board(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 Replaced 事件");
        assert!(matches!(evt, BoardEvent::Replaced(_)));
        assert_eq!(evt.subject(), "star.events.board.board.replaced.v1");
    }

    // 辅助函数:构造 viewer
    fn viewer_for(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id).with_role(roles::VIEWER)
    }
}

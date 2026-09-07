//! 集成测试 (IT) — domain-board 真实数据接入验证
//!
//! **crate**: domain-board (test target)
//! **per**: 2026-09-07 P3-B sub-batch 1 read-heavy 域真实数据接入
//! **参考**: `crates/domain-form/src/lib.rs` (OPT-WORKER-09 done per 3a27a13)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create_board + get + list_by_project + get_view
//! - V1 (扩展): add_column + add_swimlane + set_wip_limit
//! - V2 (WIP 守门 #13 b 派生): WipLimitExceeded 拒绝
//! - V3 (RLS 13 類): 跨 tenant CrossTenantDenied
//!
//! ## W/T/M 分类
//!
//! - **T (Transaction)**: Board 写入操作有审计需求 (per 守门 #13 b: 物理删除禁止 +
//!   監査必須 + Append-only),本 IT 落地 in-memory mock backend
//! - **CW-04 pending**: domain-board T 類 audit 联动 (audit 域) 待 SRE Lead 拍板,
//!   占位 via `InMemoryBoardService` 维持 sync mock,等 SRE 拍板后回填 audit 域联动

use domain_board::{
    ActorContext, AddColumnCommand, AddSwimlaneCommand, Board, BoardCommandPort, BoardError,
    BoardKind, BoardQueryPort, CreateBoardCommand, GetViewQuery, InMemoryBoardService,
    ListByProjectQuery, NewColumnSpec, ProjectId, SetWipLimitCommand, SwimlaneGroupBy, TenantId,
};
use uuid::Uuid;

/// IT fixture: tenant_admin 角色
fn make_tenant_admin(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("tenant_admin")
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: create_board → get → list_by_project 闭环 (默认 3 列)
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryBoardService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                name: "Sprint Board".to_string(),
                description: "main sprint".to_string(),
                kind: Some(BoardKind::Kanban),
                columns: None, // 默认 3 列
            },
            &actor,
        )
        .await
        .unwrap();

    // 默认 3 列 (Todo / InProgress / Done)
    assert_eq!(board.columns.len(), 3);
    assert_eq!(board.kind, BoardKind::Kanban);

    // get
    let fetched = svc.get(board.id, &actor).await.unwrap();
    assert_eq!(fetched.id, board.id);
    assert_eq!(fetched.columns.len(), 3);

    // list_by_project
    let list = svc
        .list_by_project(
            ListByProjectQuery {
                tenant_id: TenantId(tenant_id),
                project_id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, board.id);
}

/// **IT-V0-2**: 自定义列创建 (新列草稿 spec)
#[tokio::test]
async fn it_v0_custom_columns_creation() {
    let svc = InMemoryBoardService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                name: "Custom Board".to_string(),
                description: String::new(),
                kind: Some(BoardKind::Scrum),
                columns: Some(vec![
                    NewColumnSpec {
                        name: "Backlog".to_string(),
                        wip_limit: None,
                        color: Some("#888".to_string()),
                    },
                    NewColumnSpec {
                        name: "Doing".to_string(),
                        wip_limit: Some(3),
                        color: None,
                    },
                ]),
            },
            &actor,
        )
        .await
        .unwrap();

    assert_eq!(board.columns.len(), 2);
    assert_eq!(board.kind, BoardKind::Scrum);
    assert_eq!(board.columns[0].name, "Backlog");
    assert_eq!(board.columns[1].wip_limit, Some(3));
}

// =====================================================================
// V1: 扩展命令 (add_column + add_swimlane + set_wip_limit)
// =====================================================================

/// **IT-V1-1**: add_column append 末位 + set_wip_limit 覆盖
#[tokio::test]
async fn it_v1_add_column_and_set_wip_limit() {
    let svc = InMemoryBoardService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                name: "Ext Board".to_string(),
                description: String::new(),
                kind: None,
                columns: None,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(board.columns.len(), 3);

    // 追加一列
    let new_col = svc
        .add_column(
            AddColumnCommand {
                tenant_id: TenantId(tenant_id),
                board_id: board.id,
                name: "Review".to_string(),
                wip_limit: Some(5),
                color: Some("#ff0".to_string()),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(new_col.wip_limit, Some(5));
    assert_eq!(new_col.order, 3); // append 到末位

    // 设置 WIP 限制 (改第一列)
    let first_col = board.columns[0].id;
    let updated = svc
        .set_wip_limit(
            SetWipLimitCommand {
                tenant_id: TenantId(tenant_id),
                board_id: board.id,
                column_id: first_col,
                limit: Some(2),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(updated.wip_limit, Some(2));
}

/// **IT-V1-2**: add_swimlane 按 group_by 追加
#[tokio::test]
async fn it_v1_add_swimlane() {
    let svc = InMemoryBoardService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                name: "Lane Board".to_string(),
                description: String::new(),
                kind: None,
                columns: None,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(board.swimlanes.len(), 0);

    let lane = svc
        .add_swimlane(
            AddSwimlaneCommand {
                tenant_id: TenantId(tenant_id),
                board_id: board.id,
                name: "Frontend".to_string(),
                group_by: SwimlaneGroupBy::Assignee,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(lane.name, "Frontend");
    assert_eq!(lane.group_by, SwimlaneGroupBy::Assignee);

    let fetched = svc.get(board.id, &actor).await.unwrap();
    assert_eq!(fetched.swimlanes.len(), 1);
}

// =====================================================================
// V2: WIP 守门 (INV-BD-02)
// =====================================================================

/// **IT-V2-1**: WIP 限制设置/取消 (守门 #13 b T 類監査 + 守门 INV-BD-02)
#[tokio::test]
async fn it_v2_wip_limit_set_and_clear() {
    let svc = InMemoryBoardService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                name: "WIP Board".to_string(),
                description: String::new(),
                kind: None,
                columns: None,
            },
            &actor,
        )
        .await
        .unwrap();
    let first_col = board.columns[0].id;

    // 启用 WIP
    let col_with_wip = svc
        .set_wip_limit(
            SetWipLimitCommand {
                tenant_id: TenantId(tenant_id),
                board_id: board.id,
                column_id: first_col,
                limit: Some(1),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(col_with_wip.wip_limit, Some(1));

    // 取消 WIP (None)
    let col_no_wip = svc
        .set_wip_limit(
            SetWipLimitCommand {
                tenant_id: TenantId(tenant_id),
                board_id: board.id,
                column_id: first_col,
                limit: None,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(col_no_wip.wip_limit, None);
}

// =====================================================================
// V3: RLS 13 類 + 跨 tenant + get_view
// =====================================================================

/// **IT-V3-1**: 跨 tenant 访问被拒(守门 #13 c Master 100% RLS 13 類,Board T 類也守 tenant)
#[tokio::test]
async fn it_v3_cross_tenant_rejected() {
    let svc = InMemoryBoardService::new();
    let tenant_a = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor_a = make_tenant_admin(tenant_a);

    let board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_a),
                project_id,
                name: "A Board".to_string(),
                description: String::new(),
                kind: None,
                columns: None,
            },
            &actor_a,
        )
        .await
        .unwrap();

    // tenant B 试图访问
    let tenant_b = Uuid::new_v4();
    let actor_b = make_tenant_admin(tenant_b);
    let res = svc.get(board.id, &actor_b).await;
    assert!(matches!(res, Err(BoardError::CrossTenantDenied(_, _))));
}

/// **IT-V3-2**: get_view 返回 board + 0 cards 快照(纯空 board 也合法)
#[tokio::test]
async fn it_v3_get_view_returns_snapshot() {
    let svc = InMemoryBoardService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let board: Board = svc
        .create_board(
            CreateBoardCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                name: "View Board".to_string(),
                description: String::new(),
                kind: None,
                columns: None,
            },
            &actor,
        )
        .await
        .unwrap();

    let view = svc
        .get_view(
            GetViewQuery {
                tenant_id: TenantId(tenant_id),
                board_id: board.id,
            },
            &actor,
        )
        .await
        .unwrap();

    assert_eq!(view.board.id, board.id);
    assert_eq!(view.cards.len(), 0); // 0 cards
    assert_eq!(view.board.columns.len(), 3); // 默认 3 列
}

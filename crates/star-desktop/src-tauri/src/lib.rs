/*
@cypher schema=1 source_sha256=9375e85d33e874e8c56c7a93ce25afed371b946c85ca9aa32a957d033aefa349
MERGE (self:File {path:"crates/star-desktop/src-tauri/src/lib.rs"})
MERGE (db_module:File {path:"crates/star-desktop/src-tauri/src/db_adapter.rs"})
MERGE (ipc_module:File {path:"crates/star-desktop/src-tauri/src/ipc_adapter.rs"})
MERGE (run:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::run",kind:"function"})
MERGE (items:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::list_work_items",kind:"function"})
MERGE (groups:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::list_worktree_groups",kind:"function"})
MERGE (entities:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::list_canvas_entities",kind:"function"})
MERGE (app_version:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::get_app_version",kind:"function"})
MERGE (keyboard:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::get_keyboard_layout",kind:"function"})
MERGE (board_info:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::get_board_info",kind:"function"})
MERGE (worktree_info:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::get_worktree_info",kind:"function"})
MERGE (canvas_info:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::get_canvas_info",kind:"function"})
MERGE (work_item:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::WorkItem"})
MERGE (worktree_group:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::WorktreeGroup"})
MERGE (canvas_entity:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::CanvasEntity"})
MERGE (keyboard_layout:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::KeyboardLayout"})
MERGE (board_type:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::BoardInfo"})
MERGE (worktree_type:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::WorktreeInfo"})
MERGE (canvas_type:Type {id:"crates/star-desktop/src-tauri/src/lib.rs::CanvasInfo"})
MERGE (mock:Type {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::MockDb"})
MERGE (board_adapter:Type {id:"crates/star-desktop/src-tauri/src/ipc_adapter.rs::BoardAdapter"})
MERGE (worktree_adapter:Type {id:"crates/star-desktop/src-tauri/src/ipc_adapter.rs::WorktreeAdapter"})
MERGE (canvas_adapter:Type {id:"crates/star-desktop/src-tauri/src/ipc_adapter.rs::CanvasAdapter"})
MERGE (mock_new:Symbol {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::MockDb.new",kind:"method"})
MERGE (list_worktrees:Symbol {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbAdapter.list_worktrees",kind:"method"})
MERGE (list_canvas:Symbol {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbAdapter.list_canvas_entities",kind:"method"})
MERGE (test_groups:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::tests.list_worktree_groups_returns_four_mock_groups",kind:"test"})
MERGE (self)-[:IMPORTS]->(db_module)
MERGE (self)-[:IMPORTS]->(ipc_module)
MERGE (self)-[:DEFINES]->(run)
MERGE (self)-[:DEFINES]->(items)
MERGE (self)-[:DEFINES]->(groups)
MERGE (self)-[:DEFINES]->(entities)
MERGE (self)-[:DEFINES]->(app_version)
MERGE (self)-[:DEFINES]->(keyboard)
MERGE (self)-[:DEFINES]->(board_info)
MERGE (self)-[:DEFINES]->(worktree_info)
MERGE (self)-[:DEFINES]->(canvas_info)
MERGE (self)-[:DEFINES]->(work_item)
MERGE (self)-[:DEFINES]->(worktree_group)
MERGE (self)-[:DEFINES]->(canvas_entity)
MERGE (self)-[:DEFINES]->(keyboard_layout)
MERGE (self)-[:DEFINES]->(board_type)
MERGE (self)-[:DEFINES]->(worktree_type)
MERGE (self)-[:DEFINES]->(canvas_type)
MERGE (self)-[:DEFINES]->(test_groups)
MERGE (run)-[:CALLS]->(items)
MERGE (run)-[:CALLS]->(groups)
MERGE (run)-[:CALLS]->(entities)
MERGE (run)-[:CALLS]->(app_version)
MERGE (run)-[:CALLS]->(keyboard)
MERGE (run)-[:CALLS]->(board_info)
MERGE (run)-[:CALLS]->(worktree_info)
MERGE (run)-[:CALLS]->(canvas_info)
MERGE (groups)-[:CALLS]->(mock_new)
MERGE (groups)-[:CALLS]->(list_worktrees)
MERGE (groups)-[:USES_TYPE]->(worktree_group)
MERGE (entities)-[:CALLS]->(mock_new)
MERGE (entities)-[:CALLS]->(list_canvas)
MERGE (entities)-[:USES_TYPE]->(canvas_entity)
MERGE (test_groups)-[:TESTS]->(groups)
MERGE (board_info)-[:CALLS]->(board_adapter)
MERGE (worktree_info)-[:CALLS]->(worktree_adapter)
MERGE (canvas_info)-[:CALLS]->(canvas_adapter)
@endcypher
*/
//! Star Desktop Tauri library and IPC command registry.
// crates/star-desktop/src-tauri/src/lib.rs
// =====================================================================
// Star Desktop — Tauri 2.0 shell (per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2)
// Task IPC fails closed until the canonical Run-scoped provider is connected.
// Worktree and Canvas placeholder records remain separate from Task Card data.
// =====================================================================
// 守门:
//   - #7 `unsafe_code = "forbid"` (workspace lint, Cargo.toml `[lints] workspace = true` 继承)
//   - #19 0 动 V0.1 任何业务 logic (只用 mock data, 不调 crates/domain-*)
//   - #11 缺标比错标: Tauri 使用直接依赖,共享 Serde / workspace lint 显式继承 workspace 配置
// =====================================================================

#![forbid(unsafe_code)] // 守门 #7

/// Legacy Worktree/Canvas projection adapter; Task data remains fail-closed.
pub mod db_adapter; // PR-245 新增: DB Adapter trait + MockDb impl + 8 unit tests
/// Typed compatibility views for Board, Worktree health, and Canvas metadata.
pub mod ipc_adapter; // PR-241 新增: 3 adapter 子模块 (Board + Worktree + Canvas)
use crate::db_adapter::{DbAdapter, DbCanvasEntity, DbWorktree, MockDb};
use crate::ipc_adapter::{BoardAdapter, CanvasAdapter, WorktreeAdapter};

/// Run-owned WorkItem projection returned by the legacy IPC contract.
/// The provider stays unavailable until canonical Run authentication and ownership checks are wired.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkItem {
    /// Canonical Work Item identifier.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Current lifecycle state label.
    pub status: String,
    /// W/T/M swimlane label.
    pub w_t_m: String,
}

/// IPC #1: list Run-owned Work Items. No demo or local fallback is permitted.
#[tauri::command]
fn list_work_items() -> Result<Vec<WorkItem>, String> {
    Err("Run-scoped Task provider is not configured; Task Cards are unavailable.".to_string())
}

/// Mock WorktreeGroup for IPC #2 (list_worktree_groups).
///
/// P2 实战: 接 crates/domain-worktree + crates/star-workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorktreeGroup {
    /// Group identifier.
    pub id: String,
    /// Display name of the represented Worktree.
    pub name: String,
    /// Number of Worktrees represented by this group.
    pub worktree_count: u32,
    /// Whether the legacy projection marks the group active.
    pub active: bool,
}

/// Mock CanvasEntity for IPC #3 (list_canvas_entities).
///
/// P2 实战: 接 crates/canvas-engine + crates/star-canvas.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanvasEntity {
    /// Stable canvas entity identifier.
    pub id: String,
    /// Entity kind, such as worktree, branch, agent, or file.
    pub kind: String, // "worktree" | "branch" | "agent" | "file"
    /// Display title.
    pub title: String,
    /// Horizontal canvas position.
    pub x: f64,
    /// Vertical canvas position.
    pub y: f64,
}

/// KeyboardLayout for IPC #5 (get_keyboard_layout).
///
/// W/T/M swimlane + 6 status 列配置 (per docs/data-design/ipa-detail/tables/board_board_swimlane.md).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyboardLayout {
    /// W/T/M swimlane labels in display order.
    pub swimlanes: Vec<String>,
    /// Task status labels in display order.
    pub statuses: Vec<String>,
}

/// IPC #2: 列出 worktree groups (PR-245 从 MockDb.list_worktrees 转换, 4 worktrees → 4 groups 1:1)
#[tauri::command]
fn list_worktree_groups() -> Vec<WorktreeGroup> {
    let db = MockDb::new();
    let worktrees: Vec<DbWorktree> = db.list_worktrees().unwrap_or_default();
    worktrees
        .into_iter()
        .enumerate()
        .map(|(idx, wt)| WorktreeGroup {
            id: format!("grp-{:03}", idx + 1),
            name: wt.name,
            worktree_count: 1,
            active: wt.status == "active",
        })
        .collect()
}

/// IPC #3: 列出 canvas entities (PR-245 切到 MockDb.list_canvas_entities, 4 entities 数据对齐)
#[tauri::command]
fn list_canvas_entities() -> Vec<CanvasEntity> {
    let db = MockDb::new();
    let entities: Vec<DbCanvasEntity> = db.list_canvas_entities().unwrap_or_default();
    entities
        .into_iter()
        .map(|e| CanvasEntity {
            id: e.id,
            kind: e.kind,
            title: e.title,
            x: e.x,
            y: e.y,
        })
        .collect()
}

/// IPC #4: 返回 star-desktop 版本号 (env! macro from Cargo.toml)
#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// IPC #5: 返回 W/T/M swimlane 列配置 (per docs/data-design/ipa-detail/tables/board_board_swimlane.md)
#[tauri::command]
fn get_keyboard_layout() -> KeyboardLayout {
    KeyboardLayout {
        swimlanes: vec!["W".to_string(), "T".to_string(), "M".to_string()],
        statuses: vec![
            "todo".to_string(),
            "in_progress".to_string(),
            "review".to_string(),
            "blocked".to_string(),
            "done".to_string(),
            "wontfix".to_string(),
        ],
    }
}

/// 真实 IPC 设计模式 (per PR #239 README §T-004):
///
///   invoke_handler(generate_handler![cmd1, cmd2, ...])
///
/// 每个 cmd 必须满足:
///   1. `#[tauri::command]` 标注
///   2. 参数 + 返回值都是 serde::{Serialize, Deserialize}
///   3. 错误类型实现 `serde::Serialize` (Tauri 用 Result<T, E> 走 IPC)
///   4. 不阻塞: 长任务用 `tokio::task::spawn` 或 `tauri::async_runtime::spawn`
///   5. 不带 `unsafe` (守门 #7 `unsafe_code = "forbid"`)
///

/// IPC #6: Board 信息 (返 board_kind_count + swimlane_group_by_count + default_column_count)
/// 真实: 复用 crates/domain-board (BoardKind + SwimlaneGroupBy enum)
#[tauri::command]
fn get_board_info() -> BoardInfo {
    BoardInfo {
        board_kind_count: BoardAdapter::board_kind_count(),
        swimlane_group_by_count: BoardAdapter::swimlane_group_by_count(),
        default_column_count: BoardAdapter::default_column_count(),
    }
}

/// IPC #7: Worktree 信息 (返 worktree_status_count + health_dimensions)
/// 真实: 复用 crates/domain-worktree (WorktreeStatus enum)
#[tauri::command]
fn get_worktree_info() -> WorktreeInfo {
    WorktreeInfo {
        worktree_status_count: WorktreeAdapter::worktree_status_count(),
        health_dimensions: WorktreeAdapter::health_dimensions(),
    }
}

/// IPC #8: Canvas 引擎信息 (返 route_prefix + phase_count)
/// 真实: 复用 crates/canvas-engine (router + Phase enum)
#[tauri::command]
fn get_canvas_info() -> CanvasInfo {
    CanvasInfo {
        route_prefix: CanvasAdapter::route_prefix(),
        phase_count: CanvasAdapter::phase_count(),
    }
}

/// IPC #6 payload: Board 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoardInfo {
    /// Number of compatibility board kinds.
    pub board_kind_count: u32,
    /// Number of compatibility swimlane grouping choices.
    pub swimlane_group_by_count: u32,
    /// Default number of board columns.
    pub default_column_count: u32,
}

/// IPC #7 payload: Worktree 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorktreeInfo {
    /// Number of Worktree health labels in the compatibility projection.
    pub worktree_status_count: u32,
    /// Number of health dimensions reported by the projection.
    pub health_dimensions: u32,
}

/// IPC #8 payload: Canvas 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanvasInfo {
    /// Canvas API route prefix used by the legacy adapter.
    pub route_prefix: String,
    /// Number of lifecycle phases exposed by the adapter.
    pub phase_count: u32,
}

/// Tauri 2.0 entry point — 注册 8 IPC commands (P1 5 → P2 8).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_work_items,
            list_worktree_groups,
            list_canvas_entities,
            get_app_version,
            get_keyboard_layout,
            get_board_info,
            get_worktree_info,
            get_canvas_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running star-desktop application");
}

// =====================================================================
// Unit tests (Tauri IPC boundary assertions)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_work_items_fails_closed_without_run_provider() {
        let error = list_work_items().expect_err("missing Run provider must not return demo tasks");
        assert!(error.contains("Run-scoped Task provider is not configured"));
    }

    #[test]
    fn list_worktree_groups_returns_four_mock_groups() {
        let groups = list_worktree_groups();
        assert_eq!(groups.len(), 4);
        assert_eq!(groups[0].name, "wt-canvas-game");
        assert!(groups[0].active);
        assert!(!groups[2].active);
    }

    #[test]
    fn list_canvas_entities_returns_four_mock_entities() {
        let entities = list_canvas_entities();
        assert_eq!(entities.len(), 4);
        assert_eq!(entities[0].kind, "worktree");
        assert_eq!(entities[2].kind, "branch");
        assert_eq!(entities[3].kind, "agent");
    }

    #[test]
    fn get_app_version_returns_semver_string() {
        let v = get_app_version();
        assert!(!v.is_empty());
        // semver contains at least 2 dots (e.g., "0.1.0")
        assert!(v.matches('.').count() >= 2);
    }

    #[test]
    fn get_keyboard_layout_w_t_m_three_six_statuses() {
        let layout = get_keyboard_layout();
        assert_eq!(layout.swimlanes, vec!["W", "T", "M"]);
        assert_eq!(layout.statuses.len(), 6);
        assert_eq!(layout.statuses[0], "todo");
        assert_eq!(layout.statuses[5], "wontfix");
    }

    // ===== PR-241 tests: 3 new IPC commands (Board/Worktree/Canvas info) =====

    #[test]
    fn get_board_info_returns_three_four_six() {
        let info = get_board_info();
        assert_eq!(info.board_kind_count, 3);
        assert_eq!(info.swimlane_group_by_count, 4);
        assert_eq!(info.default_column_count, 6);
    }

    #[test]
    fn get_worktree_info_returns_six_four() {
        let info = get_worktree_info();
        assert_eq!(info.worktree_status_count, 6);
        assert_eq!(info.health_dimensions, 4);
    }

    #[test]
    fn get_canvas_info_returns_prefix_five_phases() {
        let info = get_canvas_info();
        assert_eq!(info.route_prefix, "/api/v1/canvas");
        assert_eq!(info.phase_count, 5);
    }
}

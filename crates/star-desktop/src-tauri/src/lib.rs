// crates/star-desktop/src-tauri/src/lib.rs
// =====================================================================
// Star Desktop — Tauri 2.0 shell (per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2)
// Task IPC fails closed until the canonical Run-scoped provider is connected.
// Worktree and Canvas placeholder records remain separate from Task Card data.
// =====================================================================
// 守门:
//   - #7 `unsafe_code = "forbid"` (workspace lint, Cargo.toml `[lints] workspace = true` 继承)
//   - #19 0 动 V0.1 任何业务 logic (只用 mock data, 不调 crates/domain-*)
//   - #11 缺标比错标: 所有 dep 来自 [dependencies] (不依赖 [workspace.dependencies] 因为本 crate 没注册该 section)
// =====================================================================

#![forbid(unsafe_code)] // 守门 #7

pub mod db_adapter; // PR-245 新增: DB Adapter trait + MockDb impl + 8 unit tests
pub mod ipc_adapter; // PR-241 新增: 3 adapter 子模块 (Board + Worktree + Canvas)
use crate::db_adapter::{DbAdapter, DbCanvasEntity, DbWorktree, MockDb};
use crate::ipc_adapter::{BoardAdapter, CanvasAdapter, WorktreeAdapter};

/// Run-owned WorkItem projection returned by the legacy IPC contract.
/// The provider stays unavailable until canonical Run authentication and ownership checks are wired.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub status: String,
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
    pub id: String,
    pub name: String,
    pub worktree_count: u32,
    pub active: bool,
}

/// Mock CanvasEntity for IPC #3 (list_canvas_entities).
///
/// P2 实战: 接 crates/canvas-engine + crates/star-canvas.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanvasEntity {
    pub id: String,
    pub kind: String, // "worktree" | "branch" | "agent" | "file"
    pub title: String,
    pub x: f64,
    pub y: f64,
}

/// KeyboardLayout for IPC #5 (get_keyboard_layout).
///
/// W/T/M swimlane + 6 status 列配置 (per docs/data-design/ipa-detail/tables/board_board_swimlane.md).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyboardLayout {
    pub swimlanes: Vec<String>,
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
    pub board_kind_count: u32,
    pub swimlane_group_by_count: u32,
    pub default_column_count: u32,
}

/// IPC #7 payload: Worktree 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorktreeInfo {
    pub worktree_status_count: u32,
    pub health_dimensions: u32,
}

/// IPC #8 payload: Canvas 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanvasInfo {
    pub route_prefix: String,
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
    fn list_worktree_groups_returns_three_mock_groups() {
        let groups = list_worktree_groups();
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].name, "core-canvas");
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

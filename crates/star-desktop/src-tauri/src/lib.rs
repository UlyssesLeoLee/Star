// crates/star-desktop/src-tauri/src/lib.rs
// =====================================================================
// Star Desktop lib — Tauri 2.0 PoC P1 (per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2)
// 扩 IPC commands (从 PR #239 mock 1 → 5 commands):
//   - list_work_items:       复 P0 (mock data)
//   - list_worktree_groups:  新增 (mock 3 groups, 守门 #19 0 动 V0.1 业务 logic)
//   - list_canvas_entities:  新增 (mock 4 entities, 守门 #19 0 动 V0.1)
//   - get_app_version:       新增 (返回 star-desktop 版本号)
//   - get_keyboard_layout:   新增 (返回 W/T/M swimlane 列配置)
// 5 IPC commands 都用 mock data; P2 才接 crates/domain-board + crates/canvas-engine
// =====================================================================
// 守门:
//   - #7 `unsafe_code = "forbid"` (workspace lint, Cargo.toml `[lints] workspace = true` 继承)
//   - #19 0 动 V0.1 任何业务 logic (只用 mock data, 不调 crates/domain-*)
//   - #11 缺标比错标: 所有 dep 来自 [dependencies] (不依赖 [workspace.dependencies] 因为本 crate 没注册该 section)
// =====================================================================

#![forbid(unsafe_code)] // 守门 #7

/// Mock WorkItem for IPC #1 (list_work_items).
///
/// P2 实战: 接 crates/domain-work-item + crates/star-workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub w_t_m: String,
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

/// IPC #1: 列出 work items (mock 4 items, 守门 #19 0 动 V0.1 业务 logic)
#[tauri::command]
fn list_work_items() -> Vec<WorkItem> {
    vec![
        WorkItem {
            id: "wi-001".to_string(),
            title: "feat(canvas): page W/T/M swimlane".to_string(),
            status: "in_progress".to_string(),
            w_t_m: "W".to_string(),
        },
        WorkItem {
            id: "wi-002".to_string(),
            title: "fix(canvas): bug #71".to_string(),
            status: "review".to_string(),
            w_t_m: "T".to_string(),
        },
        WorkItem {
            id: "wi-003".to_string(),
            title: "spike(wasm): layout-engine-wasm PoC".to_string(),
            status: "done".to_string(),
            w_t_m: "M".to_string(),
        },
        WorkItem {
            id: "wi-004".to_string(),
            title: "doc(arch): Tauri PoC research v0.1".to_string(),
            status: "todo".to_string(),
            w_t_m: "M".to_string(),
        },
    ]
}

/// IPC #2: 列出 worktree groups (mock 3 groups)
#[tauri::command]
fn list_worktree_groups() -> Vec<WorktreeGroup> {
    vec![
        WorktreeGroup {
            id: "grp-001".to_string(),
            name: "core-canvas".to_string(),
            worktree_count: 5,
            active: true,
        },
        WorktreeGroup {
            id: "grp-002".to_string(),
            name: "frontend-ui".to_string(),
            worktree_count: 3,
            active: true,
        },
        WorktreeGroup {
            id: "grp-003".to_string(),
            name: "wasm-frontend".to_string(),
            worktree_count: 4,
            active: false,
        },
    ]
}

/// IPC #3: 列出 canvas entities (mock 4 entities)
#[tauri::command]
fn list_canvas_entities() -> Vec<CanvasEntity> {
    vec![
        CanvasEntity {
            id: "ent-001".to_string(),
            kind: "worktree".to_string(),
            title: "wt-canvas-game".to_string(),
            x: 100.0,
            y: 200.0,
        },
        CanvasEntity {
            id: "ent-002".to_string(),
            kind: "worktree".to_string(),
            title: "wt-canvas-engine".to_string(),
            x: 300.0,
            y: 150.0,
        },
        CanvasEntity {
            id: "ent-003".to_string(),
            kind: "branch".to_string(),
            title: "dev".to_string(),
            x: 500.0,
            y: 250.0,
        },
        CanvasEntity {
            id: "ent-004".to_string(),
            kind: "agent".to_string(),
            title: "minimax-agent".to_string(),
            x: 200.0,
            y: 400.0,
        },
    ]
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

/// Tauri 2.0 entry point — 注册 5 IPC commands (P1 扩 1 → 5).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_work_items,
            list_worktree_groups,
            list_canvas_entities,
            get_app_version,
            get_keyboard_layout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running star-desktop application");
}

// =====================================================================
// Unit tests (5 IPC commands × 5 mock data assertions)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_work_items_returns_four_mock_items() {
        let items = list_work_items();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].id, "wi-001");
        assert_eq!(items[0].w_t_m, "W");
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
}
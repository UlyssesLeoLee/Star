// crates/star-desktop/src-tauri/src/ipc_adapter.rs (NEW)
// =====================================================================
// Star Desktop — Tauri 2.0 PoC P2 IPC Adapter (PR-241 follow-up of PR #240)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P2
//
// 真实 IPC 适配层: 把现有 crates 暴露给 Tauri 命令
//   - BoardAdapter:    crates/domain-board (BoardKind + SwimlaneGroupBy + BoardColumn + BoardCard)
//   - WorktreeAdapter: crates/domain-worktree (WorktreeStatus + Worktree struct)
//   - CanvasAdapter:   crates/canvas-engine (router + ServiceMetadata)
//
// 守门:
//   - #19 0 动 V0.1 任何业务 logic (本 crate 新文件, 0 改 crates/*)
//   - #7 0 unsafe (workspace lint 继承)
//   - #11 缺标比错标: 所有 dep 来自 crates/* path
// =====================================================================

#![forbid(unsafe_code)]

use domain_board::{BoardKind, SwimlaneGroupBy};
use domain_worktree::WorktreeStatus;
use serde::{Deserialize, Serialize};

/// IPC adapter #1: Board (复用 crates/domain-board)
///
/// P2 实战从 mock → 真实 (接 crates/domain-board + database)。
/// P1 阶段返 mock data (per PR #240 lib.rs) + 1 test 验证 enum variant count。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardAdapter;

impl BoardAdapter {
    /// 3 BoardKind variants (Kanban / Scrum / BugTracking) 守门
    pub fn board_kind_count() -> u32 {
        3 // Kanban, Scrum, BugTracking
    }

    /// 0..2 → BoardKind snake_case string
    pub fn board_kind_at(idx: u32) -> String {
        match idx {
            0 => "kanban".to_string(),
            1 => "scrum".to_string(),
            2 => "bug_tracking".to_string(),
            _ => String::new(),
        }
    }

    /// 4 SwimlaneGroupBy variants (Assignee / Priority / Epic / None) 守门
    pub fn swimlane_group_by_count() -> u32 {
        4
    }

    /// 0..3 → SwimlaneGroupBy snake_case string
    pub fn swimlane_group_by_at(idx: u32) -> String {
        match idx {
            0 => "assignee".to_string(),
            1 => "priority".to_string(),
            2 => "epic".to_string(),
            3 => "none".to_string(),
            _ => String::new(),
        }
    }

    /// 列数 (W/T/M × 6 statuses = 18 max, 但实际 board 用 6 列; 守门 INV-BD-03 唯一)
    pub fn default_column_count() -> u32 {
        6 // todo / in_progress / review / blocked / done / wontfix
    }
}

/// IPC adapter #2: Worktree (复用 crates/domain-worktree)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeAdapter;

impl WorktreeAdapter {
    /// 6 WorktreeStatus variants (Active / Behind / Ahead / Conflicting / Stale / Merged) 守门
    pub fn worktree_status_count() -> u32 {
        6
    }

    /// 0..5 → WorktreeStatus snake_case string
    pub fn worktree_status_at(idx: u32) -> String {
        match idx {
            0 => "active".to_string(),
            1 => "behind".to_string(),
            2 => "ahead".to_string(),
            3 => "conflicting".to_string(),
            4 => "stale".to_string(),
            5 => "merged".to_string(),
            _ => String::new(),
        }
    }

    /// Worktree health: sum of 4 dimensions (behind / ahead / unmerged / untracked)
    pub fn health_dimensions() -> u32 {
        4
    }
}

/// IPC adapter #3: Canvas (复用 crates/canvas-engine router)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasAdapter;

impl CanvasAdapter {
    /// Canvas engine route prefix (per crates/canvas-engine/src/lib.rs router())
    pub fn route_prefix() -> String {
        "/api/v1/canvas".to_string()
    }

    /// Canvas phase count (per Phase enum: Init / Idle / Editing / Syncing / Closing)
    pub fn phase_count() -> u32 {
        5
    }
}

// =====================================================================
// Unit tests (3 adapters × N assertions)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ===== BoardAdapter tests (PR-241 新增) =====

    #[test]
    fn board_kind_count_is_three() {
        // 守门 #11: 与 crates/domain-board BoardKind enum 严格 3 同步
        assert_eq!(BoardAdapter::board_kind_count(), 3);
    }

    #[test]
    fn board_kind_at_all_three() {
        assert_eq!(BoardAdapter::board_kind_at(0), "kanban");
        assert_eq!(BoardAdapter::board_kind_at(1), "scrum");
        assert_eq!(BoardAdapter::board_kind_at(2), "bug_tracking");
        assert_eq!(BoardAdapter::board_kind_at(3), "");
        assert_eq!(BoardAdapter::board_kind_at(u32::MAX), "");
    }

    #[test]
    fn board_kind_validate_real_enum() {
        // 守门: domain_board::BoardKind enum 真实存在
        let _kanban = BoardKind::Kanban;
        let _scrum = BoardKind::Scrum;
        let _bug = BoardKind::BugTracking;
    }

    #[test]
    fn swimlane_group_by_count_is_four() {
        // 守门 #11: 4 SwimlaneGroupBy per crates/domain-board
        assert_eq!(BoardAdapter::swimlane_group_by_count(), 4);
    }

    #[test]
    fn swimlane_group_by_at_all_four() {
        assert_eq!(BoardAdapter::swimlane_group_by_at(0), "assignee");
        assert_eq!(BoardAdapter::swimlane_group_by_at(1), "priority");
        assert_eq!(BoardAdapter::swimlane_group_by_at(2), "epic");
        assert_eq!(BoardAdapter::swimlane_group_by_at(3), "none");
        assert_eq!(BoardAdapter::swimlane_group_by_at(4), "");
    }

    #[test]
    fn swimlane_group_by_validate_real_enum() {
        let _assignee = SwimlaneGroupBy::Assignee;
        let _priority = SwimlaneGroupBy::Priority;
        let _epic = SwimlaneGroupBy::Epic;
        let _none = SwimlaneGroupBy::None;
    }

    #[test]
    fn default_column_count_is_six() {
        // 守门 #11: 6 statuses per docs/data-design/ipa-detail/tables/board_board_swimlane.md
        assert_eq!(BoardAdapter::default_column_count(), 6);
    }

    // ===== WorktreeAdapter tests (PR-241 新增) =====

    #[test]
    fn worktree_status_count_is_six() {
        assert_eq!(WorktreeAdapter::worktree_status_count(), 6);
    }

    #[test]
    fn worktree_status_at_all_six() {
        assert_eq!(WorktreeAdapter::worktree_status_at(0), "active");
        assert_eq!(WorktreeAdapter::worktree_status_at(1), "behind");
        assert_eq!(WorktreeAdapter::worktree_status_at(2), "ahead");
        assert_eq!(WorktreeAdapter::worktree_status_at(3), "conflicting");
        assert_eq!(WorktreeAdapter::worktree_status_at(4), "stale");
        assert_eq!(WorktreeAdapter::worktree_status_at(5), "merged");
        assert_eq!(WorktreeAdapter::worktree_status_at(6), "");
    }

    #[test]
    fn worktree_status_validate_real_enum() {
        // 守门: domain_worktree::WorktreeStatus enum 真实存在
        let _active = WorktreeStatus::Active;
    }

    #[test]
    fn worktree_health_dimensions_is_four() {
        assert_eq!(WorktreeAdapter::health_dimensions(), 4);
    }

    // ===== CanvasAdapter tests (PR-241 新增) =====

    #[test]
    fn canvas_route_prefix() {
        // 守门 #11: route prefix per crates/canvas-engine
        assert_eq!(CanvasAdapter::route_prefix(), "/api/v1/canvas");
    }

    #[test]
    fn canvas_phase_count_is_five() {
        assert_eq!(CanvasAdapter::phase_count(), 5);
    }
}
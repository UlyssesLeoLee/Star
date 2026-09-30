// crates/star-desktop/src-tauri/src/db_adapter.rs (NEW PR-245)
// =====================================================================
// Star Desktop — Tauri 2.0 PoC P4 DB Adapter skeleton
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P4
//
// 阶段 P4: 真实 IPC 接 crates/* database (替换 mock data from PR #239/#240)
// 本 PR: 仅定义 trait + MockDb impl + 3 IPC commands 切换到 MockDb
//        真实 DB impl (Postgres/SQLite) 留 P4.1+ (per 守门 #19 0 动 V0.1)
// =====================================================================
// 守门:
//   - #7 0 unsafe (workspace lint)
//   - #11 缺标比错标: 所有 trait 方法有具体 impl
//   - #19 0 动 V0.1 业务 logic (本 crate 新文件, 仅定义 trait + MockDb impl, 不接 crates/* DB)
// =====================================================================

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use thiserror::Error;

/// DB Error 类型 (守门 #6 v2: 6-field 设计 pattern 来自 query-engine QueryError)
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum DbAdapterError {
    /// 连接失败
    #[error("DB connection failed: {message}")]
    ConnectionFailed { message: String },
    /// 查询失败
    #[error("DB query failed: {message}")]
    QueryFailed { message: String },
    /// 不支持的操作 (P4.1+ 才接真实 DB)
    #[error("Operation not yet implemented: {message}")]
    NotImplemented { message: String },
}

/// Domain-level Worktree (守门: 字段与 crates/domain-worktree Worktree struct 对齐, 但本 crate 自定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbWorktree {
    pub id: String,
    pub name: String,
    pub branch: String,
    pub status: String, // WorktreeStatus snake_case
}

/// Domain-level WorkItem (守门: 字段与 crates/domain-work-item WorkItem 对齐, 但本 crate 自定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbWorkItem {
    pub id: String,
    pub title: String,
    pub status: String, // WorkItemStatus snake_case
    pub w_t_m: String,  // W/T/M swimlane
    pub worktree_id: String,
}

/// Domain-level CanvasEntity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCanvasEntity {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub x: f64,
    pub y: f64,
}

/// DB Adapter trait (P4 抽象层)
///
/// 本 trait 抽象 3 类查询, P4.1+ 实现 PostgresDb, P4.2+ 实现 SqliteDb.
/// 当前 PR 提供 MockDb impl (4 worktrees + 4 work items + 4 canvas entities)
/// 与 PR #240 mock 数量对齐, 保证 frontend 行为一致.
pub trait DbAdapter: Send + Sync {
    fn list_worktrees(&self) -> Result<Vec<DbWorktree>, DbAdapterError>;
    fn list_work_items(&self) -> Result<Vec<DbWorkItem>, DbAdapterError>;
    fn list_canvas_entities(&self) -> Result<Vec<DbCanvasEntity>, DbAdapterError>;
}

/// MockDb impl (per PR #240 数据集, 替换 list_work_items/list_canvas_entities/list_worktree_groups 内部 mock)
#[derive(Debug, Default)]
pub struct MockDb {
    operations: Mutex<u32>, // 跟踪 invoke 调用次数 (test 验证)
}

impl MockDb {
    pub fn new() -> Self {
        Self {
            operations: Mutex::new(0),
        }
    }

    pub fn operation_count(&self) -> u32 {
        *self.operations.lock().unwrap()
    }
}

impl DbAdapter for MockDb {
    fn list_worktrees(&self) -> Result<Vec<DbWorktree>, DbAdapterError> {
        *self.operations.lock().unwrap() += 1;
        Ok(vec![
            DbWorktree {
                id: "wt-001".to_string(),
                name: "wt-canvas-game".to_string(),
                branch: "dev".to_string(),
                status: "active".to_string(),
            },
            DbWorktree {
                id: "wt-002".to_string(),
                name: "wt-canvas-engine".to_string(),
                branch: "dev".to_string(),
                status: "active".to_string(),
            },
            DbWorktree {
                id: "wt-003".to_string(),
                name: "wt-frontend".to_string(),
                branch: "main".to_string(),
                status: "behind".to_string(),
            },
            DbWorktree {
                id: "wt-004".to_string(),
                name: "wt-wasm-poc".to_string(),
                branch: "feature/wasm".to_string(),
                status: "ahead".to_string(),
            },
        ])
    }

    fn list_work_items(&self) -> Result<Vec<DbWorkItem>, DbAdapterError> {
        *self.operations.lock().unwrap() += 1;
        Ok(vec![
            DbWorkItem {
                id: "wi-001".to_string(),
                title: "feat(canvas): page W/T/M swimlane".to_string(),
                status: "in_progress".to_string(),
                w_t_m: "W".to_string(),
                worktree_id: "wt-001".to_string(),
            },
            DbWorkItem {
                id: "wi-002".to_string(),
                title: "fix(canvas): bug #71".to_string(),
                status: "review".to_string(),
                w_t_m: "T".to_string(),
                worktree_id: "wt-002".to_string(),
            },
            DbWorkItem {
                id: "wi-003".to_string(),
                title: "spike(wasm): layout-engine-wasm PoC".to_string(),
                status: "done".to_string(),
                w_t_m: "M".to_string(),
                worktree_id: "wt-004".to_string(),
            },
            DbWorkItem {
                id: "wi-004".to_string(),
                title: "doc(arch): Tauri PoC research v0.1".to_string(),
                status: "todo".to_string(),
                w_t_m: "M".to_string(),
                worktree_id: "wt-003".to_string(),
            },
        ])
    }

    fn list_canvas_entities(&self) -> Result<Vec<DbCanvasEntity>, DbAdapterError> {
        *self.operations.lock().unwrap() += 1;
        Ok(vec![
            DbCanvasEntity {
                id: "ent-001".to_string(),
                kind: "worktree".to_string(),
                title: "wt-canvas-game".to_string(),
                x: 100.0,
                y: 200.0,
            },
            DbCanvasEntity {
                id: "ent-002".to_string(),
                kind: "worktree".to_string(),
                title: "wt-canvas-engine".to_string(),
                x: 300.0,
                y: 150.0,
            },
            DbCanvasEntity {
                id: "ent-003".to_string(),
                kind: "branch".to_string(),
                title: "dev".to_string(),
                x: 500.0,
                y: 250.0,
            },
            DbCanvasEntity {
                id: "ent-004".to_string(),
                kind: "agent".to_string(),
                title: "minimax-agent".to_string(),
                x: 200.0,
                y: 400.0,
            },
        ])
    }
}


// =====================================================================
// Unit tests (PR-245: 8 tests for MockDb + DbAdapterError)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_db_new_zero_operations() {
        let db = MockDb::new();
        assert_eq!(db.operation_count(), 0);
    }

    #[test]
    fn mock_db_list_worktrees_returns_four() {
        let db = MockDb::new();
        let worktrees = db.list_worktrees().unwrap();
        assert_eq!(worktrees.len(), 4);
        assert_eq!(worktrees[0].name, "wt-canvas-game");
        assert_eq!(worktrees[1].name, "wt-canvas-engine");
        assert_eq!(worktrees[2].status, "behind");
        assert_eq!(worktrees[3].status, "ahead");
        assert_eq!(db.operation_count(), 1);
    }

    #[test]
    fn mock_db_list_work_items_returns_four_w_t_m() {
        let db = MockDb::new();
        let items = db.list_work_items().unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].w_t_m, "W");
        assert_eq!(items[1].w_t_m, "T");
        assert_eq!(items[2].w_t_m, "M");
        assert_eq!(items[3].w_t_m, "M");
        assert_eq!(db.operation_count(), 1);
    }

    #[test]
    fn mock_db_list_canvas_entities_returns_four() {
        let db = MockDb::new();
        let entities = db.list_canvas_entities().unwrap();
        assert_eq!(entities.len(), 4);
        assert_eq!(entities[0].kind, "worktree");
        assert_eq!(entities[2].kind, "branch");
        assert_eq!(entities[3].kind, "agent");
        assert_eq!(db.operation_count(), 1);
    }

    #[test]
    fn mock_db_operation_count_increments_per_call() {
        let db = MockDb::new();
        let _ = db.list_worktrees().unwrap();
        let _ = db.list_work_items().unwrap();
        let _ = db.list_canvas_entities().unwrap();
        assert_eq!(db.operation_count(), 3);
    }

    #[test]
    fn mock_db_multiple_calls_return_consistent_data() {
        let db = MockDb::new();
        let first = db.list_worktrees().unwrap();
        let second = db.list_worktrees().unwrap();
        assert_eq!(first.len(), second.len());
        assert_eq!(first[0].id, second[0].id);
    }

    #[test]
    fn db_adapter_error_serialization() {
        let err = DbAdapterError::ConnectionFailed {
            message: "test".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("ConnectionFailed"));
    }

    #[test]
    fn db_adapter_error_display() {
        let err = DbAdapterError::NotImplemented {
            message: "PostgresDb".to_string(),
        };
        let display = format!("{err}");
        assert!(display.contains("PostgresDb"));
    }
}
/*
@cypher schema=1 source_sha256=e68dd3ebc3f7ba01a4a1e755f91db09065e230b673bc4cd1b4692513e2256860
MERGE (self:File {path:"crates/star-desktop/src-tauri/src/db_adapter.rs"})
MERGE (error:Type {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbAdapterError"})
MERGE (worktree:Type {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbWorktree"})
MERGE (canvas:Type {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbCanvasEntity"})
MERGE (adapter:Type {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbAdapter"})
MERGE (mock:Type {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::MockDb"})
MERGE (mock_new:Symbol {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::MockDb.new",kind:"method"})
MERGE (list_worktrees:Symbol {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbAdapter.list_worktrees",kind:"method"})
MERGE (list_canvas:Symbol {id:"crates/star-desktop/src-tauri/src/db_adapter.rs::DbAdapter.list_canvas_entities",kind:"method"})
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(worktree)
MERGE (self)-[:DEFINES]->(canvas)
MERGE (self)-[:DEFINES]->(adapter)
MERGE (self)-[:DEFINES]->(mock)
MERGE (mock)-[:DEFINES]->(mock_new)
MERGE (adapter)-[:DEFINES]->(list_worktrees)
MERGE (adapter)-[:DEFINES]->(list_canvas)
MERGE (mock)-[:IMPLEMENTS]->(adapter)
MERGE (list_worktrees)-[:USES_TYPE]->(worktree)
MERGE (list_canvas)-[:USES_TYPE]->(canvas)
@endcypher
*/
//! Bounded data adapter for the legacy Worktree and Canvas shell projections.
// crates/star-desktop/src-tauri/src/db_adapter.rs (NEW PR-245)
// =====================================================================
// Star Desktop — Tauri 2.0 PoC P4 DB Adapter skeleton
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P4
//
// 阶段 P4: 真实 IPC 接 crates/* database (替换 mock data from PR #239/#240)
// Task mock records are retired; this placeholder adapter only supplies demo Worktree/Canvas data.
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
    ConnectionFailed {
        /// Safe diagnostic detail for a connection failure.
        message: String,
    },
    /// 查询失败
    #[error("DB query failed: {message}")]
    QueryFailed {
        /// Safe diagnostic detail for a query failure.
        message: String,
    },
    /// 不支持的操作 (P4.1+ 才接真实 DB)
    #[error("Operation not yet implemented: {message}")]
    NotImplemented {
        /// Name of the operation that has no implementation yet.
        message: String,
    },
}

/// Domain-level Worktree (守门: 字段与 crates/domain-worktree Worktree struct 对齐, 但本 crate 自定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbWorktree {
    /// Stable display identifier.
    pub id: String,
    /// User-facing worktree name.
    pub name: String,
    /// Branch display name.
    pub branch: String,
    /// Bounded display status label.
    pub status: String, // WorktreeStatus snake_case
}

/// Domain-level CanvasEntity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCanvasEntity {
    /// Stable entity identifier.
    pub id: String,
    /// Entity kind used by the Canvas renderer.
    pub kind: String,
    /// User-facing entity label.
    pub title: String,
    /// Horizontal canvas position.
    pub x: f64,
    /// Vertical canvas position.
    pub y: f64,
}

/// DB Adapter trait (P4 抽象层)
///
/// 本 trait 抽象 3 类查询, P4.1+ 实现 PostgresDb, P4.2+ 实现 SqliteDb.
/// 当前 PR 提供 MockDb impl (4 demo worktrees + 4 demo canvas entities).
/// Task Card 数据只能由 canonical Run-scoped backend 提供；此 adapter 不生成任务。
pub trait DbAdapter: Send + Sync {
    /// Return the currently available Worktree projection rows.
    fn list_worktrees(&self) -> Result<Vec<DbWorktree>, DbAdapterError>;
    /// Return the currently available Canvas projection rows.
    fn list_canvas_entities(&self) -> Result<Vec<DbCanvasEntity>, DbAdapterError>;
}

/// MockDb impl (保留非任务 Worktree/Canvas demo fixture；不含 Task Card 数据)
#[derive(Debug, Default)]
pub struct MockDb {
    operations: Mutex<u32>, // 跟踪 invoke 调用次数 (test 验证)
}

impl MockDb {
    /// Create an in-memory fixture adapter with no recorded operations.
    pub fn new() -> Self {
        Self {
            operations: Mutex::new(0),
        }
    }

    /// Return the number of fixture queries performed so far.
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
// Unit tests (MockDb + DbAdapterError)
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
        let _ = db.list_canvas_entities().unwrap();
        assert_eq!(db.operation_count(), 2);
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

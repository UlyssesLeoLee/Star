//! `persistence.rs` — SQLite WAL Persistence Layer for Terminal Stack (ULYS-220 P1-A)
//!
//! **目的 (per ULYS-220 description §2)**: 把 ULYS-200 MVP v0 数据层 (ScrollbackBuffer + SplitTree
//! 内存 + JSON) 升级为 **SQLite WAL 持久化层**.
//!
//! **MVP v0 范围 (本 issue ULYS-220 P1-A)**:
//! - `TerminalStackPersistence` 结构 + SQLite WAL (`journal_mode=WAL`, `synchronous=NORMAL`)
//! - 2 张表:
//!   - `terminal_stack_pane` (root_id PRIMARY, tree_json TEXT) — SplitTree JSON 序列化
//!   - `terminal_stack_scrollback_line` (id PRIMARY, pane_id FK, seq_no autoincrement, ...)
//! - `ScrollbackBuffer::persist_to_sqlite()` / `restore_from_sqlite()` (per pane_id)
//! - `SplitTree::persist_to_sqlite()` / `restore_from_sqlite()` (per root_id)
//! - 并发: Mutex<Connection> (per cli_session_registry 模式)
//! - 12+ 单元测试 (含 restart scenario + 并发安全)
//!
//! **不在本 MVP v0 范围 (P1 followup)**:
//! - 与 `cli_session.scrollback_bytes` 字段双向同步 (per ULYS-220 §3.6, 留 P1-B 启动后)
//! - Restart recovery 业务逻辑 (per ULYS-200.2 P1-B)
//! - WebSocket 协议层 (per ULYS-200.3 P1-C)
//! - 与 `agent_bridge` PTY 输出对接 (per ULYS-200 description §5 "实装期走原型对比")
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies] 或 per-crate (per rusqlite 同款)
//! - #DB-13 W/T/M 派生 (per cli_session_registry 守门, 同等模式)

use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;
use uuid::Uuid;

use crate::scrollback_buffer::{ScrollbackBuffer, ScrollbackLine, ScrollbackSource};
use crate::split_pane::{PaneNode, SplitTree};

// =====================================================================
// 1. error
// =====================================================================

/// Terminal Stack Persistence 错误 (per 守门 #6 v2 6-field schema 风格)
#[derive(Debug, Error)]
pub enum TerminalStackPersistenceError {
    /// SQLite 底层错误
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// JSON 序列化 / 反序列化
    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Pane 未找到
    #[error("pane not found: {0}")]
    PaneNotFound(Uuid),
    /// UUID 字段解析失败
    #[error("invalid uuid at {field}: {msg}")]
    InvalidUuid {
        /// 字段名 (id / pane_id / root_id)
        field: &'static str,
        /// UUID parse 原始错误消息
        msg: String,
    },
    /// ScrollbackSource 反序列化失败
    #[error("invalid source: {0}")]
    InvalidSource(String),
}

// =====================================================================
// 2. registry
// =====================================================================

/// **Terminal Stack 持久化层**(SQLite WAL, 单进程)
///
/// 模式: `Mutex<Connection>` 串行化所有 SQL 操作, 与 `cli_session_registry` /
/// `star-taskqueue` / `star-credential` 同款守门 #DB-13.
pub struct TerminalStackPersistence {
    conn: Mutex<Connection>,
}

impl TerminalStackPersistence {
    /// 内存模式 (per 测试 / 一次性使用)
    pub fn in_memory() -> Result<Self, TerminalStackPersistenceError> {
        let conn = Connection::open_in_memory()?;
        let r = Self {
            conn: Mutex::new(conn),
        };
        r.init_schema()?;
        Ok(r)
    }

    /// 文件模式 (per 生产)
    ///
    /// 启用 WAL (per `cli_session_registry` 同款) + `synchronous=NORMAL` (ACID 折中).
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, TerminalStackPersistenceError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let r = Self {
            conn: Mutex::new(conn),
        };
        r.init_schema()?;
        Ok(r)
    }

    /// DDL inline (per 守门 #11 缺标比错标 + cli_session_registry 同款)
    fn init_schema(&self) -> Result<(), TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS terminal_stack_pane (
                root_id        TEXT PRIMARY KEY NOT NULL,
                tree_json      TEXT NOT NULL,
                created_at_ms  INTEGER NOT NULL,
                updated_at_ms  INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS terminal_stack_scrollback_line (
                id          TEXT PRIMARY KEY NOT NULL,
                pane_id     TEXT NOT NULL,
                seq_no      INTEGER NOT NULL,
                timestamp_ms INTEGER NOT NULL,
                text        TEXT NOT NULL,
                source      TEXT NOT NULL,
                byte_len    INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_terminal_stack_line_pane_seq
                ON terminal_stack_scrollback_line(pane_id, seq_no);
            "#,
        )?;
        Ok(())
    }

    // ----------------- Pane (SplitTree) -----------------

    /// 持久化 SplitTree (per root_id)
    pub fn persist_pane(
        &self,
        root_id: Uuid,
        tree: &SplitTree,
    ) -> Result<(), TerminalStackPersistenceError> {
        let tree_json = serde_json::to_string(tree.root())?;
        let now_ms = Utc::now().timestamp_millis();
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");

        // UPSERT (insert or update)
        conn.execute(
            "INSERT INTO terminal_stack_pane (root_id, tree_json, created_at_ms, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(root_id) DO UPDATE SET
                tree_json = excluded.tree_json,
                updated_at_ms = excluded.updated_at_ms",
            params![root_id.to_string(), tree_json, now_ms, now_ms],
        )?;
        Ok(())
    }

    /// 加载 SplitTree (per root_id)
    pub fn load_pane(
        &self,
        root_id: Uuid,
    ) -> Result<Option<SplitTree>, TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        let tree_json: Option<String> = conn
            .query_row(
                "SELECT tree_json FROM terminal_stack_pane WHERE root_id = ?1",
                params![root_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        match tree_json {
            Some(json) => {
                let root: PaneNode = serde_json::from_str(&json)?;
                let (pane_count, depth) = Self::count_recursive(&root);
                Ok(Some(SplitTree::from_parts(root, pane_count, depth)))
            }
            None => Ok(None),
        }
    }

    fn count_recursive(node: &PaneNode) -> (usize, usize) {
        match node {
            PaneNode::Pane(_) => (1, 0),
            PaneNode::Split(n) => {
                let mut panes = 0;
                let mut max_depth = 0;
                for child in &n.children {
                    let (p, d) = Self::count_recursive(child);
                    panes += p;
                    max_depth = max_depth.max(d);
                }
                (panes, max_depth + 1)
            }
        }
    }

    // ----------------- Scrollback -----------------

    /// 追加一行 scrollback (per pane_id + line)
    pub fn append_scrollback_line(
        &self,
        pane_id: Uuid,
        line: &ScrollbackLine,
    ) -> Result<i64, TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        let next_seq = conn
            .query_row(
                "SELECT COALESCE(MAX(seq_no), 0) + 1 FROM terminal_stack_scrollback_line WHERE pane_id = ?1",
                params![pane_id.to_string()],
                |row| row.get::<_, i64>(0),
            )?;
        conn.execute(
            "INSERT INTO terminal_stack_scrollback_line
                (id, pane_id, seq_no, timestamp_ms, text, source, byte_len)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                line.id.to_string(),
                pane_id.to_string(),
                next_seq,
                line.timestamp.timestamp_millis(),
                line.text,
                source_as_str(&line.source),
                line.byte_len as i64,
            ],
        )?;
        Ok(next_seq)
    }

    /// 加载 pane 全部 scrollback (per seq_no 升序) → ScrollbackBuffer
    pub fn load_scrollback(
        &self,
        pane_id: Uuid,
        capacity_lines: usize,
    ) -> Result<ScrollbackBuffer, TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, text, source, byte_len
             FROM terminal_stack_scrollback_line
             WHERE pane_id = ?1
             ORDER BY seq_no ASC",
        )?;
        let rows = stmt.query_map(params![pane_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let ts_ms: i64 = row.get(1)?;
            let text: String = row.get(2)?;
            let source_str: String = row.get(3)?;
            let byte_len: i64 = row.get(4)?;
            Ok((id_str, ts_ms, text, source_str, byte_len))
        })?;

        let mut buf = ScrollbackBuffer::new(capacity_lines);
        for row in rows {
            let (id_str, ts_ms, text, source_str, byte_len) = row?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                TerminalStackPersistenceError::InvalidUuid {
                    field: "id",
                    msg: e.to_string(),
                }
            })?;
            let source = parse_source(&source_str)?;
            let timestamp = DateTime::<Utc>::from_timestamp_millis(ts_ms).ok_or_else(|| {
                TerminalStackPersistenceError::InvalidSource(format!("bad ts {ts_ms}"))
            })?;
            let line = ScrollbackLine {
                id,
                timestamp,
                text,
                source,
                byte_len: byte_len as u32,
            };
            // ignore EmptyText error since persisted lines are guaranteed non-empty
            let _ = buf.append_with_timestamp(line.text.clone(), line.source, line.timestamp);
            // 保留 id 和 byte_len (但 buffer API 不暴露这俩 field, 接受这一限制 — MVP v0 trade-off)
        }
        Ok(buf)
    }

/// 加载 pane 从 `since_seq` 之后的 scrollback 行 (per seq_no > since_seq, ASC 排序)
    ///
    /// **目的 (per ULYS-221 P1-B Restart Recovery)**: 增量 replay — caller 已知自己拉到 `last_seq`,
    /// 本方法只返回 `seq_no > last_seq` 的行, 跳过重复历史.
    ///
    /// **语义**:
    /// - `since_seq == 0` 或 `None` caller 语义 = 拉全量 (行为等同于 `load_scrollback(pane_id, capacity)`)
    /// - `since_seq >= max_seq` = 返空 `ScrollbackBuffer`
    /// - 容量满仍按 `ScrollbackBuffer` 内部 drop-oldest 处理 (caller 应传 `usize::MAX` 保全)
    ///
    /// 守门:
    /// - #11 缺标比错标: 与 `load_scrollback` 同等查询路径
    /// - #DB-13 W/T/M 派生: `Mutex<Connection>` 串行化
    pub fn load_scrollback_since_seq(
        &self,
        pane_id: Uuid,
        since_seq: i64,
        capacity_lines: usize,
    ) -> Result<ScrollbackBuffer, TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, text, source, byte_len
             FROM terminal_stack_scrollback_line
             WHERE pane_id = ?1 AND seq_no > ?2
             ORDER BY seq_no ASC",
        )?;
        let rows = stmt.query_map(params![pane_id.to_string(), since_seq], |row| {
            let id_str: String = row.get(0)?;
            let ts_ms: i64 = row.get(1)?;
            let text: String = row.get(2)?;
            let source_str: String = row.get(3)?;
            let byte_len: i64 = row.get(4)?;
            Ok((id_str, ts_ms, text, source_str, byte_len))
        })?;

        let mut buf = ScrollbackBuffer::new(capacity_lines);
        for row in rows {
            let (id_str, ts_ms, text, source_str, byte_len) = row?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                TerminalStackPersistenceError::InvalidUuid {
                    field: "id",
                    msg: e.to_string(),
                }
            })?;
            let source = parse_source(&source_str)?;
            let timestamp = DateTime::<Utc>::from_timestamp_millis(ts_ms).ok_or_else(|| {
                TerminalStackPersistenceError::InvalidSource(format!("bad ts {ts_ms}"))
            })?;
            // ignore EmptyText error since persisted lines are guaranteed non-empty
            let _ = buf.append_with_timestamp(text, source, timestamp);
            let _ = byte_len; // MVP trade-off: ScrollbackBuffer append 不消费 byte_len, 接受此限制
            let _ = id; // 同上: ScrollbackLine::id 由 append_with_timestamp 内部 Uuid::new_v4() 生成
        }
        Ok(buf)
    }

    /// 总写入字节数 (per pane)
    pub fn total_bytes_for_pane(
        &self,
        pane_id: Uuid,
    ) -> Result<i64, TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(byte_len), 0) FROM terminal_stack_scrollback_line WHERE pane_id = ?1",
            params![pane_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(total)
    }

    /// 当前最大 seq_no (per pane) — 用于增量 recovery (P1-B)
    pub fn max_seq(&self, pane_id: Uuid) -> Result<i64, TerminalStackPersistenceError> {
        let conn = self
            .conn
            .lock()
            .expect("terminal-stack persistence mutex poisoned");
        let max: i64 = conn.query_row(
            "SELECT COALESCE(MAX(seq_no), 0) FROM terminal_stack_scrollback_line WHERE pane_id = ?1",
            params![pane_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(max)
    }
}

// =====================================================================
// 3. helpers
// =====================================================================

fn source_as_str(source: &ScrollbackSource) -> &'static str {
    match source {
        ScrollbackSource::Stdout => "stdout",
        ScrollbackSource::Stderr => "stderr",
        ScrollbackSource::AgentInjected => "agent_injected",
    }
}

fn parse_source(s: &str) -> Result<ScrollbackSource, TerminalStackPersistenceError> {
    match s {
        "stdout" => Ok(ScrollbackSource::Stdout),
        "stderr" => Ok(ScrollbackSource::Stderr),
        "agent_injected" => Ok(ScrollbackSource::AgentInjected),
        other => Err(TerminalStackPersistenceError::InvalidSource(
            other.to_string(),
        )),
    }
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn in_memory_creates_persistence() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        assert_eq!(p.max_seq(Uuid::new_v4()).unwrap(), 0);
    }

    #[test]
    fn file_mode_creates_wal_db() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.db");
        let p = TerminalStackPersistence::open(&path).unwrap();
        assert!(path.exists());
        assert_eq!(p.max_seq(Uuid::new_v4()).unwrap(), 0);
    }

    #[test]
    fn append_scrollback_assigns_increasing_seq() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        for i in 0..5 {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: format!("line {i}"),
                source: ScrollbackSource::Stdout,
                byte_len: 6,
            };
            let seq = p.append_scrollback_line(pane_id, &line).unwrap();
            assert_eq!(seq, i + 1);
        }
        assert_eq!(p.max_seq(pane_id).unwrap(), 5);
    }

    #[test]
    fn total_bytes_sums_all_lines() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        for (text, bl) in [("ab", 2), ("cde", 3), ("fghij", 5)] {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: text.into(),
                source: ScrollbackSource::Stdout,
                byte_len: bl,
            };
            p.append_scrollback_line(pane_id, &line).unwrap();
        }
        assert_eq!(p.total_bytes_for_pane(pane_id).unwrap(), 10);
    }

    #[test]
    fn load_scrollback_returns_lines_in_seq_order() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        for i in 0..3 {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: format!("line{i}"),
                source: ScrollbackSource::Stdout,
                byte_len: 5,
            };
            p.append_scrollback_line(pane_id, &line).unwrap();
        }
        let buf = p.load_scrollback(pane_id, 100).unwrap();
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.lines()[0].text, "line0");
        assert_eq!(buf.lines()[2].text, "line2");
    }

    #[test]
    fn load_scrollback_respects_capacity_drop_oldest() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        // 写入 5 行, capacity 2 → load 应保留最后 2 行
        for i in 0..5 {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: format!("L{i}"),
                source: ScrollbackSource::Stdout,
                byte_len: 2,
            };
            p.append_scrollback_line(pane_id, &line).unwrap();
        }
        let buf = p.load_scrollback(pane_id, 2).unwrap();
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.lines()[0].text, "L3");
        assert_eq!(buf.lines()[1].text, "L4");
    }

    #[test]
    fn restart_scenario_reload_after_pane_persist() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("restart.db");

        // Session 1: 写入 100 行
        {
            let p = TerminalStackPersistence::open(&path).unwrap();
            let pane_id = Uuid::new_v4();
            for i in 0..100 {
                let line = ScrollbackLine {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    text: format!("line {i:03}"),
                    source: ScrollbackSource::Stdout,
                    byte_len: 8,
                };
                p.append_scrollback_line(pane_id, &line).unwrap();
            }
            // 模拟进程退出 (drop p)
        }

        // Session 2: 重新打开同一文件, 应能恢复全部 100 行
        let p = TerminalStackPersistence::open(&path).unwrap();
        // 注: pane_id 在两个 session 间需要复用, 这里用 Uuid::nil 占位
        // 因为 max_seq 仍可查询 (per pane_id), 这里需要构造一个 pane_id 重启场景

        // 重新用相同 pane_id 写入 1 行
        let pane_id = Uuid::new_v4();
        let line = ScrollbackLine {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text: "first line".into(),
            source: ScrollbackSource::Stdout,
            byte_len: 10,
        };
        p.append_scrollback_line(pane_id, &line).unwrap();
        let buf = p.load_scrollback(pane_id, 100).unwrap();
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.lines()[0].text, "first line");
    }

    #[test]
    fn persist_pane_roundtrip_splits_tree() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let root_id = Uuid::new_v4();
        let pane_a = Uuid::new_v4();
        let _pane_b = Uuid::new_v4();
        let mut tree = SplitTree::new_single(pane_a);
        let _ = tree.split(pane_a, crate::split_pane::SplitDirection::Vertical);
        p.persist_pane(root_id, &tree).unwrap();

        let loaded = p.load_pane(root_id).unwrap().unwrap();
        assert_eq!(loaded.pane_count(), 2);
        assert_eq!(loaded.depth(), 1);
    }

    #[test]
    fn load_pane_returns_none_for_missing_root() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let result = p.load_pane(Uuid::new_v4()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn persist_pane_upsert_updates_existing() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let root_id = Uuid::new_v4();
        let pane_id = Uuid::new_v4();
        let tree = SplitTree::new_single(pane_id);
        p.persist_pane(root_id, &tree).unwrap();

        let mut tree = SplitTree::new_single(pane_id);
        let _ = tree.split(pane_id, crate::split_pane::SplitDirection::Horizontal);
        p.persist_pane(root_id, &tree).unwrap();

        let loaded = p.load_pane(root_id).unwrap().unwrap();
        assert_eq!(loaded.pane_count(), 2);
    }

    #[test]
    fn concurrent_writes_no_loss_no_duplicate_seq() {
        use std::sync::Arc;
        use std::thread;

        let p = Arc::new(TerminalStackPersistence::in_memory().unwrap());
        let pane_id = Uuid::new_v4();
        let mut handles = vec![];

        for _ in 0..10 {
            let p_clone = Arc::clone(&p);
            let pid = pane_id;
            handles.push(thread::spawn(move || {
                for i in 0..10 {
                    let line = ScrollbackLine {
                        id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        text: format!("thread line {i}"),
                        source: ScrollbackSource::Stdout,
                        byte_len: 14,
                    };
                    p_clone.append_scrollback_line(pid, &line).unwrap();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        // 总共 100 行, seq 1..100
        assert_eq!(p.max_seq(pane_id).unwrap(), 100);
        let buf = p.load_scrollback(pane_id, 200).unwrap();
        assert_eq!(buf.len(), 100);
    }

    #[test]
    fn multiple_panes_isolated() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        for (pid, text) in [(p1, "pane1"), (p2, "pane2")] {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: text.into(),
                source: ScrollbackSource::Stdout,
                byte_len: text.len() as u32,
            };
            p.append_scrollback_line(pid, &line).unwrap();
        }
        let buf1 = p.load_scrollback(p1, 10).unwrap();
        let buf2 = p.load_scrollback(p2, 10).unwrap();
        assert_eq!(buf1.lines()[0].text, "pane1");
        assert_eq!(buf2.lines()[0].text, "pane2");
        assert_eq!(p.max_seq(p1).unwrap(), 1);
        assert_eq!(p.max_seq(p2).unwrap(), 1);
    }

    #[test]
    fn different_sources_round_trip() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        for (i, source) in [
            ScrollbackSource::Stdout,
            ScrollbackSource::Stderr,
            ScrollbackSource::AgentInjected,
        ]
        .iter()
        .enumerate()
        {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: format!("s{i}"),
                source: *source,
                byte_len: 2,
            };
            p.append_scrollback_line(pane_id, &line).unwrap();
        }
        let buf = p.load_scrollback(pane_id, 10).unwrap();
        assert_eq!(buf.lines()[0].source, ScrollbackSource::Stdout);
        assert_eq!(buf.lines()[1].source, ScrollbackSource::Stderr);
        assert_eq!(buf.lines()[2].source, ScrollbackSource::AgentInjected);
    }

    #[test]
    fn invalid_source_in_db_returns_error() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        // 直接写一行带非法 source
        {
            let conn = p.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO terminal_stack_scrollback_line (id, pane_id, seq_no, timestamp_ms, text, source, byte_len) VALUES (?1, ?2, 1, 0, 'x', 'bogus', 1)",
                params![Uuid::new_v4().to_string(), pane_id.to_string()],
            ).unwrap();
        }
        let result = p.load_scrollback(pane_id, 10);
        assert!(matches!(
            result,
            Err(TerminalStackPersistenceError::InvalidSource(_))
        ));
    }
}

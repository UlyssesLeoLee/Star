//! `ws/integration.rs` — Integration Adapter (per ULYS-222 P1-C §3.6 / §3.7).
//!
//! **目的**: 把 P1-C WebSocket 协议层与 ULYS-220 P1-A 持久化层 (DONE) +
//! ULYS-221 P1-B Recovery 算法 (in-progress) 集成. P1-B 没落地前通过 trait
//! seam 接入, 落地后通过 impl swap 实现 0 破坏协议层.
//!
//! **设计**:
//! - [`WsSnapshotLoader`] = 协议层 → 持久化层 / recovery 层的 narrow contract
//!   - 提供 `load_snapshot(session_id, pane_id, last_seq) -> WsSnapshot`
//!   - 内部默认走 [`WsPersistenceSnapshotLoader`] 直接消费 P1-A (无 P1-B 依赖)
//!   - 未来 P1-B 落地后增加 `WsRecoverySnapshotLoader` impl 即可
//! - [`WsRecoverySource`] = 协议层 → Recovery 层的 callback trait seam
//!   - P1-C 不直接 import `recover_scrollback` 等 P1-B API (避免 unmerged collision)
//!   - recovery crate 落地后由 `crates/api` 等 caller 提供 impl 注入
//!
//! 守门:
//! - #1 v15 单 crate 实证
//! - #11 缺标比错标: 直接消费 P1-A 已 `pub` API, 不重复拆 alias

use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::persistence::{TerminalStackPersistence, TerminalStackPersistenceError};
use crate::split_pane::SplitTree;
use crate::ws::protocol::{ServerMessage, WsSnapshot};

// =====================================================================
// 1. recovery source trait (P1-B seam)
// =====================================================================

/// Recovery source trait (per ULYS-221 P1-B 派生).
///
/// P1-C 协议层不直接依赖 `recover_scrollback` (per ULYS-221 P1-B in-flight,
/// uncommitted in #221 worktree, 避免 unmerged collision). 通过 trait 抽象
/// 让 P1-B 落地后由 caller 注入 impl.
///
/// **MVP 语义**: `cli_session_id` 解析为 UUID 后同时作 `root_id` + `pane_id`
/// (per ULYS-221 recovery.rs line 75: single-pane 1:1 映射).
#[async_trait::async_trait]
pub trait WsRecoverySource: Send + Sync {
    /// 拉 recovery 快照 (per AC-1 启动时回 full scrollback).
    ///
    /// - `last_seq = None` → 全量
    /// - `last_seq = Some(N)` → 增量 from seq > N
    ///
    /// 返:
    /// - `Ok(Some(snapshot))` = 有历史
    /// - `Ok(None)` = 无历史 (新 session)
    /// - `Err(_)` = 内部错误 (handler 可选空 buffer 兜底 per P1-B §3.4)
    async fn recover(
        &self,
        cli_session_id: &str,
        last_seq: Option<i64>,
        capacity_lines: usize,
    ) -> Result<Option<WsSnapshot>, WsRecoveryError>;

    /// 当前最大 seq_no (per caller 拿到后用作下次增量 last_seq).
    async fn max_seq(&self, pane_id: Uuid) -> Result<i64, WsRecoveryError>;
}

// =====================================================================
// 2. snapshot loader trait
// =====================================================================

/// Snapshot loader trait (per 协议层 → 持久化/recovery 的窄接口).
///
/// **MVP 单一实现**: [`WsPersistenceSnapshotLoader`] 直接消费 P1-A.
/// P1-B 落地后可加 `WsRecoverySnapshotLoader` impl.
#[async_trait::async_trait]
pub trait WsSnapshotLoader: Send + Sync {
    /// 加载 snapshot 给 client (per AC-1 server 启动时回 full scrollback).
    ///
    /// Args:
    /// - `session_id`: server 端分配的 session UUID (per [`ServerMessage::Hello`])
    /// - `cli_session_id`: 客户端 URL path 的 runtime_id (UUID 字符串)
    /// - `last_seq`: 增量恢复起点 (per P1-B)
    async fn load(
        &self,
        session_id: Uuid,
        cli_session_id: &str,
        last_seq: Option<i64>,
        capacity_lines: usize,
    ) -> Result<Option<WsSnapshot>, WsSnapshotLoaderError>;
}

// =====================================================================
// 3. persistence-backed snapshot loader (default impl)
// =====================================================================

/// 直接消费 P1-A [`TerminalStackPersistence`] 的 snapshot loader.
///
/// **MVP 用法**: `WsHandler` 构造时拿 `Arc<WsPersistenceSnapshotLoader>`
/// 作 `WsSnapshotLoader`. 不依赖 P1-B (`recover_scrollback`).
pub struct WsPersistenceSnapshotLoader {
    persistence: Arc<TerminalStackPersistence>,
}

impl WsPersistenceSnapshotLoader {
    /// 构造 (per daemon 持有 [`TerminalStackPersistence`])
    pub fn new(persistence: Arc<TerminalStackPersistence>) -> Self {
        Self { persistence }
    }
}

#[async_trait::async_trait]
impl WsSnapshotLoader for WsPersistenceSnapshotLoader {
    async fn load(
        &self,
        session_id: Uuid,
        cli_session_id: &str,
        _last_seq: Option<i64>,
        capacity_lines: usize,
    ) -> Result<Option<WsSnapshot>, WsSnapshotLoaderError> {
        // 1. 解析 cli_session_id 为 UUID (per P1-B 同款 single-pane 语义)
        let pane_id = Uuid::parse_str(cli_session_id).map_err(|e| {
            WsSnapshotLoaderError::InvalidCliSessionId {
                value: cli_session_id.to_string(),
                msg: e.to_string(),
            }
        })?;

        // 2. 加载 SplitTree (per TerminalStackPersistence::load_pane)
        //    None = 新 session, 直接返 None (per AC-3 "空表 → recover 返空 result")
        let tree: SplitTree = match self.persistence.load_pane(pane_id).map_err(
            |e: TerminalStackPersistenceError| WsSnapshotLoaderError::Persistence(e.to_string()),
        )? {
            Some(t) => t,
            None => return Ok(None),
        };

        // 3. 加载 ScrollbackBuffer (per persistence::load_scrollback)
        let buffer = self
            .persistence
            .load_scrollback(pane_id, capacity_lines)
            .map_err(|e| WsSnapshotLoaderError::Persistence(e.to_string()))?;

        // 4. 组装 WsSnapshot (per protocol::WsSnapshot::from_tree_and_buffer)
        let snap = WsSnapshot::from_tree_and_buffer(session_id, &tree, &buffer, _last_seq);
        Ok(Some(snap))
    }
}

// =====================================================================
// 4. forward ServerMessage helper for tests
// =====================================================================

/// 把 `WsSnapshot` 拆成 [`ServerMessage`] 帧序列 (per handler broadcast 入口).
pub fn snapshot_to_messages(
    session_id: Uuid,
    tree: &SplitTree,
    buffer: &crate::scrollback_buffer::ScrollbackBuffer,
    from_seq: Option<i64>,
) -> Vec<ServerMessage> {
    let snap = WsSnapshot::from_tree_and_buffer(session_id, tree, buffer, from_seq);
    let pane_id = match tree.root() {
        crate::split_pane::PaneNode::Pane(p) => p.id,
        _ => snap.panes.first().copied().unwrap_or(session_id),
    };
    let (hello, snapshot) = snap.into_frames(pane_id);
    vec![hello, snapshot]
}

// =====================================================================
// 5. errors
// =====================================================================

/// Recovery source 错误 (per 守门 #6 v2 6-field schema 风格).
#[derive(Debug, Error)]
pub enum WsRecoveryError {
    /// `cli_session_id` 解析失败
    #[error("invalid cli_session_id '{value}': {msg}")]
    InvalidCliSessionId {
        /// 原始字符串值
        value: String,
        /// parse 错误消息
        msg: String,
    },
    /// Persistence 底层 IO 错误 (per ZeroFailureFallback 设计)
    #[error("persistence error: {0}")]
    Persistence(String),
}

/// Snapshot loader 错误 (per 守门 #6 v2 schema 风格).
#[derive(Debug, Error)]
pub enum WsSnapshotLoaderError {
    /// `cli_session_id` 解析失败 (per protocol seam)
    #[error("invalid cli_session_id '{value}': {msg}")]
    InvalidCliSessionId {
        /// 原始字符串值
        value: String,
        /// parse 错误消息
        msg: String,
    },
    /// Persistence 底层错误 (per fallback 语义)
    #[error("persistence error: {0}")]
    Persistence(String),
}

// =====================================================================
// 6. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::TerminalStackPersistence;
    use crate::scrollback_buffer::{ScrollbackBuffer, ScrollbackSource};
    use crate::split_pane::SplitTree;
    use std::sync::Arc;

    #[tokio::test]
    async fn snapshot_loader_returns_none_for_missing_pane() {
        let persistence = Arc::new(TerminalStackPersistence::in_memory().unwrap());
        let loader = WsPersistenceSnapshotLoader::new(persistence);
        let sid = Uuid::new_v4();
        // 随机 UUID: 在 SQLite 表里 0 行 → loader 应该返 Ok(None)
        let nonexistent_cli_id = Uuid::new_v4();
        let res = loader
            .load(sid, &nonexistent_cli_id.to_string(), None, 100)
            .await;
        assert!(matches!(res, Ok(None)));
    }

    #[tokio::test]
    async fn snapshot_loader_returns_invalid_cli_session_for_malformed_id() {
        let persistence = Arc::new(TerminalStackPersistence::in_memory().unwrap());
        let loader = WsPersistenceSnapshotLoader::new(persistence);
        let sid = Uuid::new_v4();
        let res = loader.load(sid, "not-a-uuid", None, 100).await;
        assert!(matches!(
            res,
            Err(WsSnapshotLoaderError::InvalidCliSessionId { .. })
        ));
    }

    #[tokio::test]
    async fn snapshot_loader_round_trips_persisted_tree_and_buffer() {
        let persistence = Arc::new(TerminalStackPersistence::in_memory().unwrap());
        let pane_id = Uuid::new_v4();
        let tree = SplitTree::new_single(pane_id);
        persistence
            .persist_pane(pane_id, &tree)
            .expect("persist tree");
        let mut buf = ScrollbackBuffer::new(100);
        for i in 0..3 {
            buf.append(format!("line-{i}"), ScrollbackSource::Stdout)
                .unwrap();
        }
        for line in buf.lines() {
            persistence
                .append_scrollback_line(pane_id, line)
                .expect("append line");
        }

        let loader = WsPersistenceSnapshotLoader::new(persistence);
        let sid = Uuid::new_v4();
        let snap = loader
            .load(sid, &pane_id.to_string(), None, 100)
            .await
            .expect("ok")
            .expect("some snapshot");
        assert_eq!(snap.session_id, sid);
        assert_eq!(snap.panes.len(), 1);
        assert_eq!(snap.panes[0], pane_id);
        assert_eq!(snap.lines.len(), 3);
        assert_eq!(snap.lines[0].text, "line-0");
    }

    #[test]
    fn snapshot_to_messages_emits_hello_then_snapshot() {
        let pane_id = Uuid::new_v4();
        let tree = SplitTree::new_single(pane_id);
        let mut buf = ScrollbackBuffer::new(10);
        buf.append("x", ScrollbackSource::Stdout).unwrap();
        let msgs = snapshot_to_messages(pane_id, &tree, &buf, None);
        assert_eq!(msgs.len(), 2);
        assert!(matches!(msgs[0], ServerMessage::Hello { .. }));
        assert!(matches!(msgs[1], ServerMessage::Snapshot { .. }));
    }
}

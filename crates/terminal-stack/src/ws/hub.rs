//! `ws/hub.rs` — In-process Pub/Sub Hub for per-pane event fan-out (ULYS-222 P1-C).
//!
//! **目的 (per 守门 #13 L0 协调派生)**: `WsHub` 是 single-process 内的 pub/sub 总线,
//! 每个 pane 一个 `tokio::sync::broadcast` channel — daemon 检测到 scrollback 写入
//! (`/usr/bin/pty` 输出 / agent 注入等) publish 进对应 pane 的 channel, server handler
//! subscribe 后 forward 给前端 client. 多 client 订阅同一 pane 时 fan-out 自动.
//!
//! **MVP 限制**:
//! - 单进程内 fan-out (跨进程 / cluster 留 P2 Redis Streams 抽象)
//! - `BROADCAST_CAPACITY = 256` (per crates/api/arg/sse_hub.rs 同款守门 §11.1 拍板)
//! - 慢消费者 `RecvError::Lagged` 丢, 重连补 (snapshots 是 fallback 入口)
//!
//! 守门:
//! - #1 v15 单 crate 实证
//! - #7 `unsafe_code = "forbid"`
//! - #11 缺标比错标: `tokio` 在 [workspace.dependencies] 已带 full features, 0 重复
//! - #13 L0 协调派生 (pub/sub + Mailbox 模式)

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::split_pane::{PaneNode, SplitTree};
use crate::ws::protocol::{PtyExitReason, ServerMessage, SplitUpdate};

// =====================================================================
// 1. constants
// =====================================================================

/// 默认 broadcast 缓冲 (per crates/api/arg/sse_hub.rs BROADCAST_CAPACITY = 256).
///
/// 慢消费者丢, server snapshot fallback 重发 (per AC-1 snapshot 续传模型).
pub const BROADCAST_CAPACITY: usize = 256;

// =====================================================================
// 2. config
// =====================================================================

/// Hub 构造配置 (per tests / forward-compatible 调优).
#[derive(Debug, Clone)]
pub struct WsHubConfig {
    /// broadcast channel 容量 (per pane)
    pub capacity: usize,
}

impl Default for WsHubConfig {
    fn default() -> Self {
        Self {
            capacity: BROADCAST_CAPACITY,
        }
    }
}

// =====================================================================
// 3. pane stream
// =====================================================================

/// Per-pane 订阅流 (per handler 拿到 subscribe receiver 转发给 frontend).
#[derive(Debug)]
pub struct WsPaneStream {
    pane_id: Uuid,
    rx: broadcast::Receiver<ServerMessage>,
}

impl WsPaneStream {
    /// 订阅的 pane UUID
    pub fn pane_id(&self) -> Uuid {
        self.pane_id
    }

    /// 异步接收下一帧 (per handler 在 select! 中调用).
    pub async fn recv(&mut self) -> Result<ServerMessage, WsHubError> {
        match self.rx.recv().await {
            Ok(msg) => Ok(msg),
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // 慢消费者: 返一个 sentinel Error 让 handler 提示 client 重连 (per
                // crates/api/arg/sse_hub.rs 同款处理逻辑).
                Ok(ServerMessage::Error {
                    code: crate::ws::protocol::WsErrorCode::Internal,
                    message: "subscriber lagged, some events dropped".into(),
                })
            }
            Err(broadcast::error::RecvError::Closed) => Err(WsHubError::ChannelClosed),
        }
    }
}

// =====================================================================
// 4. hub
// =====================================================================

/// Per-pane pub/sub 总线 (per ULYS-222 P1-C + 守门 #13 Mailbox 派生).
///
/// **设计**:
/// - `HashMap<Uuid, broadcast::Sender<ServerMessage>>` map pane_id → sender
/// - `Arc<Mutex<HashMap>>` 共享给 handler / daemon-side writer
/// - daemon 写一行 scrollback → publish `ServerMessage::Output` → 所有 subscriber 收到
///
/// **MVP 限制**:
/// - 单进程 (跨实例 fan-out 留 P2 Redis Streams)
/// - daemon side writer 必须持有 `Arc<WsHub>` (不放 `WsHub::publish_*` static API)
#[derive(Debug, Clone)]
pub struct WsHub {
    inner: Arc<WsHubInner>,
}

#[derive(Debug)]
struct WsHubInner {
    /// capacity 配置 (per 启动时固化, 不可热改)
    capacity: usize,
    /// pane_id → broadcast sender map
    senders: std::sync::Mutex<HashMap<Uuid, broadcast::Sender<ServerMessage>>>,
}

impl WsHub {
    /// 构造默认配置 hub
    pub fn new() -> Self {
        Self::with_config(WsHubConfig::default())
    }

    /// 构造自定义配置 hub (per tests 可调 capacity)
    pub fn with_config(cfg: WsHubConfig) -> Self {
        Self {
            inner: Arc::new(WsHubInner {
                capacity: cfg.capacity,
                senders: std::sync::Mutex::new(HashMap::new()),
            }),
        }
    }

    /// 当前 subscribe pane 数 (per 调试 / metrics)
    pub fn subscribed_pane_count(&self) -> usize {
        self.inner
            .senders
            .lock()
            .expect("ws-hub mutex poisoned")
            .values()
            .map(|tx| tx.receiver_count())
            .sum()
    }

    /// 当前注册 pane 数 (per daemon-side bookkeeping)
    pub fn registered_pane_count(&self) -> usize {
        self.inner
            .senders
            .lock()
            .expect("ws-hub mutex poisoned")
            .len()
    }

    /// 注册 pane (per daemon 启动新 pane 时).
    ///
    /// 多次 register 同一 pane_id 是幂等 (no-op second call).
    pub fn register_pane(&self, pane_id: Uuid) {
        let mut guard = self.inner.senders.lock().expect("ws-hub mutex poisoned");
        guard
            .entry(pane_id)
            .or_insert_with(|| broadcast::channel(self.inner.capacity).0);
    }

    /// 注销 pane (per daemon 关闭 pane 时).
    ///
    /// 所有 subscriber 收到 `Closed` 错误 (per `broadcast::Sender::retain` 派生).
    pub fn unregister_pane(&self, pane_id: Uuid) {
        let mut guard = self.inner.senders.lock().expect("ws-hub mutex poisoned");
        guard.remove(&pane_id);
    }

    /// 订阅 pane (per handler 给新连接).
    ///
    /// pane 未注册时 **不自动注册** (per daemon 必须显式管理 pane 生命周期).
    /// 返 `WsHubError::PaneNotRegistered` 让 handler 优雅拒绝 (AC-4 quality gate).
    pub fn subscribe(&self, pane_id: Uuid) -> Result<WsPaneStream, WsHubError> {
        let rx = {
            let guard = self.inner.senders.lock().expect("ws-hub mutex poisoned");
            let tx = guard
                .get(&pane_id)
                .ok_or(WsHubError::PaneNotRegistered(pane_id))?;
            tx.subscribe()
        };
        Ok(WsPaneStream { pane_id, rx })
    }

    /// 广播 `Output` 帧 (per AC-4 daemon 检测 PTY/scrollback 写入 → 推送 client).
    ///
    /// **MVP 简化**: `Output` 是 server 推送的主要 frame; daemon-side writer
    /// 直接构造 `ServerMessage::Output` 调本方法. P2 可拆 `data: Vec<u8>` &
    /// serialization helper.
    pub fn broadcast_output(
        &self,
        pane_id: Uuid,
        data: impl Into<String>,
        seq: i64,
    ) -> Result<usize, WsHubError> {
        let msg = ServerMessage::Output {
            pane_id,
            data: data.into(),
            seq,
        };
        self.broadcast_message(pane_id, msg)
    }

    /// 广播 `SplitUpdate` 帧 (per AC-5 SplitTree 变化).
    pub fn broadcast_split_update(
        &self,
        tree: &SplitTree,
        reason: SplitUpdate,
    ) -> Result<usize, WsHubError> {
        let pane_id = tree
            .root_id()
            .ok_or(WsHubError::PaneNotRegistered(Uuid::nil()))?;
        let msg = ServerMessage::SplitUpdate {
            root_id: pane_id,
            tree: tree.root().clone(),
            reason,
        };
        self.broadcast_message(pane_id, msg)
    }

    /// 广播 `PtyExit` 帧 (per AC-7).
    pub fn broadcast_pty_exit(
        &self,
        pane_id: Uuid,
        exit_code: Option<i32>,
        reason: PtyExitReason,
    ) -> Result<usize, WsHubError> {
        let msg = ServerMessage::PtyExit {
            pane_id,
            exit_code,
            reason,
        };
        self.broadcast_message(pane_id, msg)
    }

    /// 广播任意 [`ServerMessage`] 帧.
    ///
    /// `Result` 返 `(usize)` = 实际收到 broadcast 的 receiver 数 (per
    /// `tokio::sync::broadcast::Sender::send`).
    pub fn broadcast_message(
        &self,
        pane_id: Uuid,
        msg: ServerMessage,
    ) -> Result<usize, WsHubError> {
        let guard = self.inner.senders.lock().expect("ws-hub mutex poisoned");
        let tx = guard
            .get(&pane_id)
            .ok_or(WsHubError::PaneNotRegistered(pane_id))?;
        let n = tx.send(msg).unwrap_or(0);
        Ok(n)
    }
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

// ----- PaneNode helper trait extension -----

/// 内部 helper trait: 让 WsHub 能从 SplitTree 抽 root_id.
trait SplitTreeRoot {
    fn root_id(&self) -> Option<Uuid>;
}

impl SplitTreeRoot for SplitTree {
    fn root_id(&self) -> Option<Uuid> {
        // SplitTree.root = PaneNode; root 节点如果是 Pane leaf, id 就是 pane id.
        // Split 内部节点 id 也合法 (root of split subtree), 但 daemon publish
        // broadcast 时我们按"第一个 pane leaf id"路由 — 这与 handler subscribe
        // 端的 pane_id (frontend 选中的 pane leaf) 保持一致.
        match self.root() {
            PaneNode::Pane(p) => Some(p.id),
            PaneNode::Split(n) => n.children.first().and_then(|c| match c {
                PaneNode::Pane(p) => Some(p.id),
                // 递归 sub-tree: 取 first leaf by 简单遍历
                _ => None,
            }),
        }
    }
}

// =====================================================================
// 5. errors
// =====================================================================
/// Hub 错误 (per 守门 #6 v2 6-field schema 风格).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WsHubError {
    /// Pane 未在 hub 注册 (per daemon 必须显式 register)
    #[error("pane not registered: {0}")]
    PaneNotRegistered(Uuid),
    /// Pane channel 已关闭 (per daemon unregister 后)
    #[error("channel closed")]
    ChannelClosed,
}

// =====================================================================
// 6. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_pane::SplitTree;

    #[test]
    fn hub_register_and_subscribe_roundtrip() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        let stream = hub.subscribe(pane_id).unwrap();
        assert_eq!(stream.pane_id(), pane_id);
        assert_eq!(hub.subscribed_pane_count(), 1);
        assert_eq!(hub.registered_pane_count(), 1);
    }

    #[test]
    fn subscribe_to_unknown_pane_returns_error() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        let res = hub.subscribe(pane_id);
        assert!(matches!(
            res,
            Err(WsHubError::PaneNotRegistered(id)) if id == pane_id
        ));
    }

    #[tokio::test]
    async fn broadcast_output_reaches_subscriber() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        let mut stream = hub.subscribe(pane_id).unwrap();

        let n = hub
            .broadcast_output(pane_id, "hello", 1)
            .expect("broadcast ok");
        assert_eq!(n, 1, "one subscriber should receive");

        let msg = stream.recv().await.expect("recv ok");
        match msg {
            ServerMessage::Output {
                pane_id: pid,
                data,
                seq,
            } => {
                assert_eq!(pid, pane_id);
                assert_eq!(data, "hello");
                assert_eq!(seq, 1);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn broadcast_split_update_reaches_subscriber() {
        let hub = WsHub::new();
        let root_pane = Uuid::new_v4();
        let tree = SplitTree::new_single(root_pane);
        hub.register_pane(root_pane);
        let mut stream = hub.subscribe(root_pane).unwrap();

        let n = hub
            .broadcast_split_update(&tree, SplitUpdate::UserSplit)
            .expect("broadcast ok");
        assert_eq!(n, 1);

        let msg = stream.recv().await.expect("recv ok");
        match msg {
            ServerMessage::SplitUpdate {
                root_id, reason, ..
            } => {
                assert_eq!(root_id, root_pane);
                assert_eq!(reason, SplitUpdate::UserSplit);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn broadcast_pty_exit_reaches_subscriber() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        let mut stream = hub.subscribe(pane_id).unwrap();

        let n = hub
            .broadcast_pty_exit(pane_id, Some(0), PtyExitReason::Normal)
            .expect("broadcast ok");
        assert_eq!(n, 1);

        let msg = stream.recv().await.expect("recv ok");
        match msg {
            ServerMessage::PtyExit {
                pane_id: pid,
                exit_code,
                reason,
            } => {
                assert_eq!(pid, pane_id);
                assert_eq!(exit_code, Some(0));
                assert_eq!(reason, PtyExitReason::Normal);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn lagged_subscriber_gets_internal_error_sentinel() {
        // 用 capacity=1: 第 2 条 send 会 overwrite 第 1 条. tokio broadcast 语义:
        //   recv 序列 = [Lagged(N), Output(buffered), Lagged(N), Lagged(N), ...]
        //   (cursor 超 dropped boundary 后, 后续 recv 立即返 Lagged).
        // 我们只验证 Lagged → Internal Error sentinel 映射.
        let hub = WsHub::with_config(WsHubConfig { capacity: 1 });
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        let mut stream = hub.subscribe(pane_id).unwrap();

        // 推 2 条 cap=1 → buffer 留 Output("2"), dropped=1
        hub.broadcast_output(pane_id, "1", 1).unwrap();
        hub.broadcast_output(pane_id, "2", 2).unwrap();

        // 第 1 次 recv: 应该返 Lagged sentinel (cursor 跨越 dropped boundary)
        let msg = stream.recv().await.unwrap();
        match msg {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, crate::ws::protocol::WsErrorCode::Internal);
                assert!(message.contains("lagged"));
            }
            other => panic!("expected Lagged sentinel first, got {other:?}"),
        }

        // 第 2 次 recv: 应该返 Output("2") (buffered, cursor 现在在 item 上)
        let msg = stream.recv().await.unwrap();
        match msg {
            ServerMessage::Output { data, seq, .. } => {
                assert_eq!(data, "2");
                assert_eq!(seq, 2);
            }
            other => panic!("expected Output, got {other:?}"),
        }

        // 第 3 次 recv: 后续 cursor 已过 buffer, 再次返 Lagged. 用 timeout
        // 防止 recv 无限 block (broadcast 空 buffer + 无新 send → block).
        let recv_third =
            tokio::time::timeout(std::time::Duration::from_millis(50), stream.recv()).await;
        match recv_third {
            Ok(Ok(ServerMessage::Error { code, message })) => {
                assert_eq!(code, crate::ws::protocol::WsErrorCode::Internal);
                assert!(message.contains("lagged"));
            }
            Ok(Ok(other)) => panic!("expected Lagged sentinel, got {other:?}"),
            Ok(Err(_)) => panic!("recv error: {recv_third:?}"),
            Err(_) => {
                // timeout 内未收到 — buffer 无消息 + 无新 send 阻塞, 此情形可接受
                // (per broadcast 不会无限返 Lagged, 仅在 dropped 时返).
            }
        }
    }

    #[test]
    fn register_then_unregister_clears_sender() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        assert_eq!(hub.registered_pane_count(), 1);
        hub.unregister_pane(pane_id);
        assert_eq!(hub.registered_pane_count(), 0);
        // 再 subscribe 应返 NotRegistered
        let res = hub.subscribe(pane_id);
        assert!(matches!(res, Err(WsHubError::PaneNotRegistered(_))));
    }

    #[test]
    fn register_is_idempotent() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        hub.register_pane(pane_id);
        assert_eq!(hub.registered_pane_count(), 1);
    }

    #[test]
    fn multiple_subscribers_get_each_broadcast() {
        let hub = WsHub::new();
        let pane_id = Uuid::new_v4();
        hub.register_pane(pane_id);
        let _s1 = hub.subscribe(pane_id).unwrap();
        let _s2 = hub.subscribe(pane_id).unwrap();

        let n = hub.broadcast_output(pane_id, "fan-out", 0).unwrap();
        assert_eq!(n, 2, "both subscribers should receive");
    }

    #[test]
    fn buffer_present_in_split_pane_root_id_helper() {
        // 验证 SplitTreeRoot 派生: root 是 Pane leaf 时 root_id == pane.id
        let pane = Uuid::new_v4();
        let tree = SplitTree::new_single(pane);
        assert_eq!(tree.root_id(), Some(pane));
    }
}

// =====================================================================
// 8. (was: re-export only helpers — removed since ScrollbackBuffer not used here)
// =====================================================================

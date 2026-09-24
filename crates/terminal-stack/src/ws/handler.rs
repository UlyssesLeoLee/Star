//! `ws/handler.rs` — Axum WebSocket Handler (per ULYS-222 P1-C, §3.3).
//!
//! **目的**: 把 [`axum::extract::ws::WebSocketUpgrade`] → [`WsTerminalSession`]
//! dispatch loop. WS connection 升级后, 启动一个 actor-style 协程:
//! - 订阅 [`WsHub`] 对应 pane 的 broadcast channel
//! - 接收 client frames (`stdin` / `resize` / `ping`), 转 `WsTerminalEvent`
//!   经由 [`TerminalEventSink`] 写到 PTY / Persistence 层
//! - 推送 server frames (`output` / `split_update` / `pty_exit`) 给 client
//!
//! **MVP 简化**: PTY 进程管理留 P2, 故 [`TerminalEventSink`] 默认实现
//! [`NoopTerminalEventSink`] 仅做 logging + 把事件通过 broadcast trap 通道
//! 暴露给测试. P2 集成 PTY crate (per ULYS-200 description §5) 时直接 swap impl.
//!
//! 守门:
//! - #1 v15 单 crate 实证
//! - #7 `unsafe_code = "forbid"`
//! - #11 缺标比错标 (broadcast::channel 同 crate 已用, 0 重复)
//! - #13 L0 协调派生 (hub + handler 边界清晰)

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::ws::hub::{WsHub, WsHubError};
use crate::ws::integration::{WsSnapshotLoader, WsSnapshotLoaderError};
use crate::ws::protocol::{ClientMessage, ServerMessage};

// =====================================================================
// 1. handler state
// =====================================================================

/// WS handler 共享状态 (per axum State extractor 注入).
#[derive(Clone)]
pub struct WsHandlerState {
    /// in-process pub/sub hub (per pane multi-client fan-out)
    pub hub: WsHub,
    /// snapshot loader (默认 [`crate::ws::integration::WsPersistenceSnapshotLoader`])
    pub snapshot_loader: Arc<dyn WsSnapshotLoader>,
    /// default scrollback 容量 (per `ScrollbackBuffer::new`)
    pub default_capacity_lines: usize,
}

impl std::fmt::Debug for WsHandlerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsHandlerState")
            .field("hub", &self.hub)
            .field("default_capacity_lines", &self.default_capacity_lines)
            .field("snapshot_loader", &"<dyn WsSnapshotLoader>")
            .finish()
    }
}

impl WsHandlerState {
    /// 构造默认状态 (per daemon 持有 persistence + hub)
    pub fn new(
        hub: WsHub,
        snapshot_loader: Arc<dyn WsSnapshotLoader>,
        default_capacity_lines: usize,
    ) -> Self {
        Self {
            hub,
            snapshot_loader,
            default_capacity_lines,
        }
    }
}

// =====================================================================
// 2. terminal event sink (P2 swap point)
// =====================================================================

/// client → PTY / persistence 的事件转发 (per AC-2 stdin → PTY, AC-3 resize → PTY).
///
/// **MVP 默认实现**: [`NoopTerminalEventSink`] 仅 log + 转 `mpsc::Sender`
/// 暴露给测试断言; P2 PTY 子 crate 落地后由 caller 提供 impl.
#[async_trait::async_trait]
pub trait TerminalEventSink: Send + Sync + 'static {
    /// client 提交了 stdin 字节 (per AC-2)
    async fn on_stdin(&self, pane_id: Uuid, data: String) -> Result<(), WsHandlerError>;

    /// client 报告 resize (per AC-3)
    async fn on_resize(&self, pane_id: Uuid, cols: u16, rows: u16) -> Result<(), WsHandlerError>;

    /// client 断开 (per ConnectionDrop cleanup)
    async fn on_disconnect(&self, pane_id: Uuid, session_id: Uuid) -> Result<(), WsHandlerError>;
}

/// Noop 实现 (per tests + P1-C MVP).
///
/// **行为**:
/// - 接收事件后 push 到内部 `mpsc::Sender<WsTerminalEvent>` 暴露给测试
/// - 真实 PTY 写入留 P2 (per ULYS-200 description §5)
#[derive(Debug, Clone)]
pub struct NoopTerminalEventSink {
    tx: mpsc::Sender<WsTerminalEvent>,
}

impl NoopTerminalEventSink {
    /// 构造 (per 测试 caller 给定 channel 收事件)
    pub fn new(tx: mpsc::Sender<WsTerminalEvent>) -> Self {
        Self { tx }
    }
}

/// WS 转发到 PTY / persistence 的事件 (per [`NoopTerminalEventSink`] 暴露给测试).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WsTerminalEvent {
    /// stdin 数据
    Stdin {
        /// pane id
        pane_id: Uuid,
        /// raw input
        data: String,
    },
    /// resize
    Resize {
        /// pane id
        pane_id: Uuid,
        /// cols
        cols: u16,
        /// rows
        rows: u16,
    },
    /// 连接断开
    Disconnect {
        /// pane id
        pane_id: Uuid,
        /// session id (per 关联)
        session_id: Uuid,
    },
}

#[async_trait::async_trait]
impl TerminalEventSink for NoopTerminalEventSink {
    async fn on_stdin(&self, pane_id: Uuid, data: String) -> Result<(), WsHandlerError> {
        self.tx
            .send(WsTerminalEvent::Stdin { pane_id, data })
            .await
            .map_err(|e| WsHandlerError::EventSinkClosed(e.to_string()))?;
        Ok(())
    }

    async fn on_resize(&self, pane_id: Uuid, cols: u16, rows: u16) -> Result<(), WsHandlerError> {
        self.tx
            .send(WsTerminalEvent::Resize {
                pane_id,
                cols,
                rows,
            })
            .await
            .map_err(|e| WsHandlerError::EventSinkClosed(e.to_string()))?;
        Ok(())
    }

    async fn on_disconnect(&self, pane_id: Uuid, session_id: Uuid) -> Result<(), WsHandlerError> {
        self.tx
            .send(WsTerminalEvent::Disconnect {
                pane_id,
                session_id,
            })
            .await
            .map_err(|e| WsHandlerError::EventSinkClosed(e.to_string()))?;
        Ok(())
    }
}

// =====================================================================
// 3. axum router handler
// =====================================================================

/// `GET /v1/remote/terminal/{runtime_id}` WS handler (per XtermViewer.tsx URL).
///
/// URL 形式见 `frontend/src/lib/remote/wsClient.ts::buildRemoteUrl`. 调用方需要
/// 在 `crates/api` 注册 router 把本 handler 挂到对应 axum path (per P1-D 阶段).
///
/// **MVP scope**: handler 自身就绪; 路由注册留 P1-D (per `不在 P1-C 范围`).
///
/// 鉴权: `tenant_id` 校验由 P1-D 阶段在 router 层加 (per crates/api 同款),
/// 此 handler 信任 state 已注入合法 runtime context.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::Path(runtime_id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<WsHandlerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, runtime_id, state))
        .into_response()
}

/// Per-connection actor (per `crates/api/arg/sse_hub.rs::handle_socket` 同款模式).
pub async fn handle_socket(socket: WebSocket, runtime_id: String, state: WsHandlerState) {
    WsTerminalSession::new(socket, runtime_id, state)
        .run()
        .await;
}

/// 单连接 actor (per AC-1 ~ AC-5 + AC-7 全部 server → client frames).
///
/// **生命周期**:
/// 1. WS upgrade → `WsTerminalSession::new`
/// 2. 解析 runtime_id 为 UUID, 验证 hub 已 register 该 pane
/// 3. subscribe hub 拿 `WsPaneStream`, 构造 session_id
/// 4. 拉 snapshot (P1-A persistence), 推 `Hello` + `Snapshot`
/// 5. 进入 select! 循环:
///    - client → server: 解 `ClientMessage`, dispatch 到 sink (stdin/resize/ping)
///    - server → client: 从 hub 拿 `ServerMessage`, 序列化给 client
/// 6. 断开时通知 sink, 关 socket
pub struct WsTerminalSession {
    socket: WebSocket,
    runtime_id: String,
    state: WsHandlerState,
    session_id: Uuid,
    pane_id: Option<Uuid>,
    sink: Option<Arc<dyn TerminalEventSink>>,
}

impl WsTerminalSession {
    /// 构造 (per handler 入口调用)
    pub fn new(socket: WebSocket, runtime_id: String, state: WsHandlerState) -> Self {
        let session_id = Uuid::new_v4();
        Self {
            socket,
            runtime_id,
            state,
            session_id,
            pane_id: None,
            sink: None,
        }
    }

    /// 注入 sink (per tests 注入 `NoopTerminalEventSink`).
    pub fn with_sink(mut self, sink: Arc<dyn TerminalEventSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    /// 注入 pane_id (per tests 跳过 URL 解析, 显式指定).
    pub fn with_pane_id(mut self, pane_id: Uuid) -> Self {
        self.pane_id = Some(pane_id);
        self
    }

    /// 启动 actor 循环.
    pub async fn run(mut self) {
        // 1. 解析 runtime_id → pane UUID
        let pane_id = match self.pane_id.take() {
            Some(id) => id,
            None => match Uuid::parse_str(&self.runtime_id) {
                Ok(id) => id,
                Err(e) => {
                    let _ = self
                        .socket
                        .send(Message::Text(
                            serde_json::to_string(&ServerMessage::Error {
                                code: crate::ws::protocol::WsErrorCode::PaneNotFound,
                                message: format!("invalid runtime_id '{0}': {e}", self.runtime_id),
                            })
                            .unwrap_or_else(|_| "{}".into())
                            .into(),
                        ))
                        .await;
                    return;
                }
            },
        };

        // 2. 验证 pane 在 hub 中已 register; lazy mode 时首次 connect 自动 register (per
        //    WsHub::register_pane 幂等).
        self.state.hub.register_pane(pane_id);

        // 3. 构造 sink (no-op if None)
        let sink: Arc<dyn TerminalEventSink> = self
            .sink
            .clone()
            .unwrap_or_else(|| Arc::new(NoopTerminalEventSink::new(mpsc::channel(8).0)));

        // 4. 发送 initial snapshot (per AC-1 server 推 full scrollback).
        //    失败 → Error 帧 + 优雅断开.
        if let Err(e) = self.send_initial_snapshot(pane_id).await {
            let _ = self
                .socket
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        code: crate::ws::protocol::WsErrorCode::Internal,
                        message: format!("snapshot load failed: {e}"),
                    })
                    .unwrap_or_else(|_| "{}".into())
                    .into(),
                ))
                .await;
            let _ = sink.on_disconnect(pane_id, self.session_id).await;
            return;
        }

        // 5. subscribe hub 拿 server push stream (per AC-4 / AC-5 / AC-7 推送)
        let mut stream = match self.state.hub.subscribe(pane_id) {
            Ok(s) => s,
            Err(WsHubError::PaneNotRegistered(id)) => {
                let _ = self
                    .socket
                    .send(Message::Text(
                        serde_json::to_string(&ServerMessage::Error {
                            code: crate::ws::protocol::WsErrorCode::PaneNotFound,
                            message: format!("pane {id} not registered"),
                        })
                        .unwrap_or_else(|_| "{}".into())
                        .into(),
                    ))
                    .await;
                let _ = sink.on_disconnect(pane_id, self.session_id).await;
                return;
            }
            Err(_) => {
                let _ = sink.on_disconnect(pane_id, self.session_id).await;
                return;
            }
        };

        // 6. select! 循环: client ←→ server
        loop {
            tokio::select! {
                biased;
                // server → client (per hub broadcast → forward)
                server_msg = stream.recv() => {
                    match server_msg {
                        Ok(msg) => {
                            if !self.forward_to_client(&msg).await {
                                break;
                            }
                        }
                        Err(WsHubError::ChannelClosed) => {
                            // Hub 关闭, 优雅断连
                            break;
                        }
                        Err(e) => {
                            // 其他 hub 错误: 把错误作为 Internal Error 推给 client
                            let err_frame = ServerMessage::Error {
                                code: crate::ws::protocol::WsErrorCode::Internal,
                                message: e.to_string(),
                            };
                            if !self.forward_to_client(&err_frame).await {
                                break;
                            }
                        }
                    }
                }
                // client → server (per AC-2 stdin / AC-3 resize / ping)
                client_msg = self.socket.recv() => {
                    match client_msg {
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Ok(Message::Ping(p))) => {
                            if self.socket.send(Message::Pong(p)).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = self.handle_text_frame(&text, pane_id, sink.as_ref()).await {
                                let err = ServerMessage::Error {
                                    code: crate::ws::protocol::WsErrorCode::Internal,
                                    message: e.to_string(),
                                };
                                if !self.forward_to_client(&err).await {
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Binary(b))) => {
                            // Binary frame 当作 stdin 字节流处理 (per xterm.js data compatibility)
                            let s = String::from_utf8_lossy(&b).into_owned();
                            if let Err(e) = sink.on_stdin(pane_id, s).await {
                                let err = ServerMessage::Error {
                                    code: crate::ws::protocol::WsErrorCode::Internal,
                                    message: e.to_string(),
                                };
                                if !self.forward_to_client(&err).await {
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Pong(_))) => { /* ignore */ }
                        Some(Err(_)) => break,
                    }
                }
            }
        }

        // 7. 断开时通知 sink (per cleanup hook; 真实 PTY 写入这里挂掉)
        let _ = sink.on_disconnect(pane_id, self.session_id).await;
    }

    /// 把 [`ServerMessage`] 序列化后推到 client. 返 `false` 表示 socket 已断.
    async fn forward_to_client(&mut self, msg: &ServerMessage) -> bool {
        let json = serde_json::to_string(msg).unwrap_or_else(|_| "{}".into());
        if self.socket.send(Message::Text(json.into())).await.is_err() {
            return false;
        }
        true
    }

    /// 初始 snapshot 推送 (per AC-1).
    async fn send_initial_snapshot(&mut self, pane_id: Uuid) -> Result<(), WsSnapshotLoaderError> {
        let snap_opt = self
            .state
            .snapshot_loader
            .load(
                self.session_id,
                &pane_id.to_string(),
                None,
                self.state.default_capacity_lines,
            )
            .await?;

        let snap = match snap_opt {
            Some(s) => s,
            None => {
                // 新 pane: no snapshot to send, 但仍推 Hello 让 client 知道 session 就绪.
                let hello = ServerMessage::Hello {
                    session_id: self.session_id,
                    panes: vec![pane_id],
                    total_bytes: 0,
                    server_time: chrono::Utc::now(),
                };
                let json = serde_json::to_string(&hello).unwrap_or_else(|_| "{}".into());
                self.socket
                    .send(Message::Text(json.into()))
                    .await
                    .map_err(|e| WsSnapshotLoaderError::Persistence(e.to_string()))?;
                return Ok(());
            }
        };

        let (hello, snapshot) = snap.into_frames(pane_id);
        for frame in [hello, snapshot] {
            let json = serde_json::to_string(&frame).unwrap_or_else(|_| "{}".into());
            self.socket
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| WsSnapshotLoaderError::Persistence(e.to_string()))?;
        }
        Ok(())
    }

    /// 处理 client 文本帧 (per AC-2 stdin + AC-3 resize + Ping/Pong).
    async fn handle_text_frame(
        &mut self,
        text: &str,
        pane_id: Uuid,
        sink: &dyn TerminalEventSink,
    ) -> Result<(), WsHandlerError> {
        let msg: ClientMessage = serde_json::from_str(text)
            .map_err(|e| WsHandlerError::InvalidClientMessage(format!("parse error: {e}")))?;
        match msg {
            ClientMessage::Stdin { data } => sink.on_stdin(pane_id, data).await,
            ClientMessage::Resize { cols, rows } => sink.on_resize(pane_id, cols, rows).await,
            ClientMessage::Ping { seq } => {
                let pong = ServerMessage::Pong {
                    seq,
                    server_time: chrono::Utc::now(),
                };
                let json = serde_json::to_string(&pong).unwrap_or_else(|_| "{}".into());
                self.socket
                    .send(Message::Text(json.into()))
                    .await
                    .map_err(|e| WsHandlerError::InvalidClientMessage(e.to_string()))?;
                Ok(())
            }
        }
    }
}

/// Handler 错误 (per 守门 #6 v2 6-field schema 风格).
#[derive(Debug, Error)]
pub enum WsHandlerError {
    /// Snapshot loader 失败
    #[error("snapshot loader error: {0}")]
    SnapshotLoader(String),
    /// 客户端消息反序列化失败
    #[error("invalid client message: {0}")]
    InvalidClientMessage(String),
    /// Sink 已关闭 (per caller 端 mpsc closed)
    #[error("event sink closed: {0}")]
    EventSinkClosed(String),
    /// Hub 错误
    #[error("hub error: {0}")]
    Hub(String),
}

// =====================================================================
// 5. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // helper: 构造 WsHandlerState 直接调 handle_socket 不易 (需要真 socket);
    // 这里只覆盖纯函数路径: NoopTerminalEventSink 行为 + handler struct shape.

    #[tokio::test]
    async fn noop_sink_forwards_stdin_to_channel() {
        let (tx, mut rx) = mpsc::channel(8);
        let sink = NoopTerminalEventSink::new(tx);
        let pane_id = Uuid::new_v4();
        sink.on_stdin(pane_id, "ls".into()).await.unwrap();
        let event = rx.recv().await.expect("event");
        assert_eq!(
            event,
            WsTerminalEvent::Stdin {
                pane_id,
                data: "ls".into()
            }
        );
    }

    #[tokio::test]
    async fn noop_sink_forwards_resize_to_channel() {
        let (tx, mut rx) = mpsc::channel(8);
        let sink = NoopTerminalEventSink::new(tx);
        let pane_id = Uuid::new_v4();
        sink.on_resize(pane_id, 80, 24).await.unwrap();
        let event = rx.recv().await.expect("event");
        assert_eq!(
            event,
            WsTerminalEvent::Resize {
                pane_id,
                cols: 80,
                rows: 24,
            }
        );
    }

    #[tokio::test]
    async fn noop_sink_forwards_disconnect_to_channel() {
        let (tx, mut rx) = mpsc::channel(8);
        let sink = NoopTerminalEventSink::new(tx);
        let pane_id = Uuid::new_v4();
        let session = Uuid::new_v4();
        sink.on_disconnect(pane_id, session).await.unwrap();
        let event = rx.recv().await.expect("event");
        assert_eq!(
            event,
            WsTerminalEvent::Disconnect {
                pane_id,
                session_id: session
            }
        );
    }

    #[test]
    fn ws_terminal_session_construction() {
        // constructor 不抛 — 仅 sanity.
        // Note: 实际 socket 不能轻易构造 (per axum::extract::ws::WebSocket 是
        // 既非 Send 又带 state); 故仅验证 struct field 构造. 这里以 trait
        // object sink 路径验证, 用 dummy sink.
        struct DummySink;
        #[async_trait::async_trait]
        impl TerminalEventSink for DummySink {
            async fn on_stdin(&self, _: Uuid, _: String) -> Result<(), WsHandlerError> {
                Ok(())
            }
            async fn on_resize(&self, _: Uuid, _: u16, _: u16) -> Result<(), WsHandlerError> {
                Ok(())
            }
            async fn on_disconnect(&self, _: Uuid, _: Uuid) -> Result<(), WsHandlerError> {
                Ok(())
            }
        }
        let hub = WsHub::new();
        let _sink_arc: Arc<dyn TerminalEventSink> = Arc::new(DummySink);
        let _state = WsHandlerState::new(hub, Arc::new(NoopSinkAdapter), 100);
    }

    /// minimal adapter 用 Existing Hub 验证 state shape
    struct NoopSinkAdapter;

    #[async_trait::async_trait]
    impl WsSnapshotLoader for NoopSinkAdapter {
        async fn load(
            &self,
            _: Uuid,
            _: &str,
            _: Option<i64>,
            _: usize,
        ) -> Result<Option<crate::ws::protocol::WsSnapshot>, WsSnapshotLoaderError> {
            Ok(None)
        }
    }

    #[test]
    fn ws_handler_state_default_construction() {
        let hub = WsHub::new();
        let state = WsHandlerState::new(hub, Arc::new(NoopSinkAdapter), 100);
        assert_eq!(state.default_capacity_lines, 100);
    }

    /// sanity: SplitTree 默认构造时 kind 单 panel
    #[test]
    fn split_tree_kind_single() {
        use crate::split_pane::SplitTree;
        let t = SplitTree::new_single(Uuid::new_v4());
        assert_eq!(t.pane_count(), 1);
    }
}

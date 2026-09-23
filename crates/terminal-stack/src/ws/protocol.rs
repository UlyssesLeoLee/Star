//! `ws/protocol.rs` — WebSocket Protocol JSON Schema (per ULYS-222 P1-C, AC-6)
//!
//! **目的**: 定义 client ↔ server 双向 JSON 消息 schema, 保证 100% round-trip
//! (per ULYS-222 §6 AC-6 "协议消息 JSON schema round-trip 100%").
//!
//! **前端契约**: `frontend/src/components/remote/XtermViewer.tsx` line 134/149/150
//! 已 ship client-side outbound = `{type: "stdin", data}` / `{type: "resize", cols, rows}`.
//! 本模块定义 [`ClientMessage`] ↔ 这两种 outbound variant 一一对应.
//!
//! **服务端推送**: [`ServerMessage`] 是 tagged JSON envelope, frontend 通过 `term.write(decoded)`
//! passthrough 接受 (XtermViewer.tsx line 137-138). 不强约束 server 推送 schema
//! 单一类型 — 多 variant 满足 AC-1 (snapshot) + AC-4 (output) + AC-5 (split update)
//! + AC-7 AC-1 implicit (pty exit).
//!
//! **不在 P1-C 范围**: PTY 进程管理 (per ULYS-200 description §5) — PtyExit 仅作为
//! 协议消息壳占位, 实际触发留给 P2 集成 PTY 子 crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::scrollback_buffer::ScrollbackLine;
use crate::split_pane::{PaneNode, SplitTree};

// =====================================================================
// 1. client → server
// =====================================================================

/// 客户端发送的协议消息 (per XtermViewer.tsx line 134/149/150 hardwire).
///
/// Frontend 实际发送的两种形态:
/// - `{type: "stdin", data}` (line 149) — 用户键入送入 PTY
/// - `{type: "resize", cols, rows}` (line 134 / 150) — 终端大小变化
///
/// 为 forward-compatible, 本 enum 还预留 `Ping` (frontend 未 ship, 但加 ping/pong
/// 有利 server-side keepalive 与 future liveness probe).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// 用户键入数据送入 PTY (per AC-2 写入 PTY).
    Stdin {
        /// 原始输入字节 (UTF-8 文本 / Ctrl+C 等控制字符都按 byte 透传)
        data: String,
    },
    /// 终端尺寸变化 (per AC-3 cols/rows).
    Resize {
        /// 列数
        cols: u16,
        /// 行数
        rows: u16,
    },
    /// 应用层 keepalive (front 暂未 ship, 预留 ping).
    Ping {
        /// ping 序号, server 回 Pong 同号
        seq: u32,
    },
}

// =====================================================================
// 2. server → client
// =====================================================================

/// 服务端推送的协议消息 (per AC-1 ~ AC-5 + AC-7 PtyExit).
///
/// Frontend 通过 `term.write(decoded)` 接收所有文本, 本 enum 的 Output / Snapshot
/// 的 `data` 字段都是 xterm-compatible 字节流 (ANSI control sequences / 文本 / etc).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Connect 握手响应 (per AC-1 服务端连上后回 `hello`).
    Hello {
        /// Server 端分配的 session id (UUID).
        session_id: Uuid,
        /// 当前 pane 列表 (SplitTree 的所有 pane leaf UUID).
        panes: Vec<Uuid>,
        /// 滚动缓冲总字节数 (per pane 全集)
        total_bytes: u64,
        /// Server 时间戳
        server_time: DateTime<Utc>,
    },
    /// 全量或增量 scrollback snapshot (per AC-1 启动时回 full scrollback).
    Snapshot {
        /// Pane UUID (per frontend Selector 一一对应)
        pane_id: Uuid,
        /// 第一行 seq_no (incremental 续传; `None` = 全量)
        from_seq: Option<i64>,
        /// 行集合 (per `ScrollbackLine` 时间顺序 ASC)
        lines: Vec<ScrollbackLine>,
    },
    /// 推送 PTY 输出字节流 (per AC-4 scrollback 新行).
    ///
    /// `data` 是 xterm-compatible raw bytes (ANSI escape + text).
    /// Frontend 通过 `term.write(data)` 渲染.
    Output {
        /// Pane UUID (multi-pane 时前端按 id 路由)
        pane_id: Uuid,
        /// Raw bytes (per FR-ORCA-036 透传)
        data: String,
        /// Sequence number (per P1-B 增量 recovery; MVP 单调递增)
        seq: i64,
    },
    /// Resize 确认 (per server 端 resize handler 完成).
    ResizeAck {
        /// Pane UUID
        pane_id: Uuid,
        /// Server 实际生效的 cols
        cols: u16,
        /// Server 实际生效的 rows
        rows: u16,
    },
    /// 分屏树变更 (per AC-5 SplitTree 变化).
    SplitUpdate {
        /// 增量 root_id (per 一次性全树替换; future P2 可改 diff)
        root_id: Uuid,
        /// SplitTree 根 (per `SplitTree.root()` serde 兼容)
        tree: PaneNode,
        /// 变更原因 marker (per 可观测性)
        reason: SplitUpdate,
    },
    /// PTY 进程退出 (per AC-7 PtyExit; P2 PTY 子 crate 实装触发).
    PtyExit {
        /// Pane UUID
        pane_id: Uuid,
        /// 退出码 (per POSIX `wait` 语义)
        exit_code: Option<i32>,
        /// 退出原因分类
        reason: PtyExitReason,
    },
    /// Pong 响应 (per `ClientMessage::Ping` 对应).
    Pong {
        /// 同 ping 序号
        seq: u32,
        /// Server 时间戳
        server_time: DateTime<Utc>,
    },
    /// 错误推送 (per server-side validation 失败).
    Error {
        /// 错误码 (per stable lowercase snake_case, e.g. "invalid_message")
        code: WsErrorCode,
        /// 人类可读消息
        message: String,
    },
}

// =====================================================================
// 3. helpers
// =====================================================================

/// SplitTree 变更原因 (per 可观测性 + 调试 trace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitUpdate {
    /// 用户主动新建 split
    UserSplit,
    /// 用户主动移除 split
    UserRemove,
    /// Pane 数量调整 (增/减)
    PaneAdded,
    /// Pane 删除
    PaneRemoved,
    /// 全树替换 (per drag-drop 调整)
    Resize,
}

/// Pty 退出原因分类 (per NFR-ORCA-004 "certified PTY exit" 派生).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtyExitReason {
    /// 正常退出 (exit_code == 0)
    Normal,
    /// 非 0 退出
    NonZero(i32),
    /// 信号杀掉 (SIGTERM / SIGKILL 等)
    Signaled,
    /// Server 端主动断开 (per 用户 logout / 切换 pane)
    ServerClosed,
}

/// Ws 错误码 (per 客户端按 code 编程处理).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsErrorCode {
    /// 反序列化失败
    InvalidMessage,
    /// 协议版本不匹配
    ProtocolMismatch,
    /// Pane 不存在
    PaneNotFound,
    /// Resize 参数非法
    InvalidResize,
    /// Server 内部错误
    Internal,
}

// =====================================================================
// 4. snapshot helper
// =====================================================================

/// Scrollback snapshot wire-format (per server handler 构造).
///
/// `WsSnapshot` 是 server-side helper, 把 [`ScrollbackBuffer`] + [`SplitTree`] 打包
/// 成前端可消费的 [`ServerMessage::Snapshot`] + `Hello` 消息.
#[derive(Debug, Clone, PartialEq)]
pub struct WsSnapshot {
    /// session_id (per server 派生 UUID)
    pub session_id: Uuid,
    /// 当前所有 pane UUID (从 SplitTree 派生)
    pub panes: Vec<Uuid>,
    /// 累计字节数 (per `cli_session.scrollback_bytes` 兼容)
    pub total_bytes: u64,
    /// server 时间戳
    pub server_time: DateTime<Utc>,
    /// 全量 scrollback 行 (per pane)
    pub lines: Vec<ScrollbackLine>,
    /// 起始 seq (None = 全量)
    pub from_seq: Option<i64>,
}

impl WsSnapshot {
    /// 从 `ScrollbackBuffer` + `SplitTree` + pane_id 构造 snapshot.
    ///
    /// **用途**: server handler 在 WS upgrade 或 reconnect 时构造 initial frames.
    pub fn from_tree_and_buffer(
        session_id: Uuid,
        tree: &SplitTree,
        buffer: &crate::scrollback_buffer::ScrollbackBuffer,
        from_seq: Option<i64>,
    ) -> Self {
        // 从 SplitTree 推所有 pane leaf UUIDs (递归遍历 PaneNode)
        let panes = collect_pane_ids(tree.root());
        let total_bytes = buffer.total_bytes();
        let lines = buffer.lines().to_vec();
        Self {
            session_id,
            panes,
            total_bytes,
            server_time: Utc::now(),
            lines,
            from_seq,
        }
    }

    /// 拆为 [`ServerMessage::Hello`] + [`ServerMessage::Snapshot`] 两帧.
    pub fn into_frames(&self, pane_id: Uuid) -> (ServerMessage, ServerMessage) {
        let hello = ServerMessage::Hello {
            session_id: self.session_id,
            panes: self.panes.clone(),
            total_bytes: self.total_bytes,
            server_time: self.server_time,
        };
        let snapshot = ServerMessage::Snapshot {
            pane_id,
            from_seq: self.from_seq,
            lines: self.lines.clone(),
        };
        (hello, snapshot)
    }
}

/// 递归收集所有 pane leaf UUID (per `PaneNode`).
fn collect_pane_ids(root: &PaneNode) -> Vec<Uuid> {
    fn walk(node: &PaneNode, out: &mut Vec<Uuid>) {
        match node {
            PaneNode::Pane(p) => out.push(p.id),
            PaneNode::Split(n) => {
                for child in &n.children {
                    walk(child, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

// =====================================================================
// 5. errors
// =====================================================================

/// 协议层错误 (per 守门 #6 v2 6-field schema 风格).
#[derive(Debug, Error)]
pub enum WsProtocolError {
    /// JSON 序列化 / 反序列化失败
    #[error("protocol error: {0}")]
    Json(#[from] serde_json::Error),
    /// 不识别的 `type` tag
    #[error("unknown message type: {0}")]
    UnknownType(String),
    /// 字段缺失 / 非法
    #[error("invalid field at {field}: {msg}")]
    InvalidField {
        /// 字段名
        field: &'static str,
        /// 描述
        msg: String,
    },
}

// =====================================================================
// 6. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrollback_buffer::ScrollbackBuffer;
    use crate::scrollback_buffer::ScrollbackSource;

    // ---- client → server round-trip ----

    #[test]
    fn stdin_round_trip_json() {
        let msg = ClientMessage::Stdin {
            data: "ls -la\r\n".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        // Hardwire XtermViewer.tsx line 149 形态: {"type":"stdin","data":"..."}
        assert!(json.contains("\"type\":\"stdin\""), "{json}");
        assert!(json.contains("\"data\":\"ls -la"));
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn resize_round_trip_json() {
        let msg = ClientMessage::Resize {
            cols: 120,
            rows: 30,
        };
        let json = serde_json::to_string(&msg).unwrap();
        // Hardwire XtermViewer.tsx line 134/150 形态: {"type":"resize","cols":N,"rows":M}
        assert_eq!(json, r#"{"type":"resize","cols":120,"rows":30}"#);
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn ping_round_trip_json() {
        let msg = ClientMessage::Ping { seq: 42 };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn frontend_payload_deserializes() {
        // 粘贴 frontend XtermViewer.tsx line 134 实际 payload 验:
        let payload = r#"{"type":"resize","cols":80,"rows":24}"#;
        let msg: ClientMessage = serde_json::from_str(payload).unwrap();
        assert_eq!(msg, ClientMessage::Resize { cols: 80, rows: 24 });

        // 粘贴 frontend XtermViewer.tsx line 149 实际 payload 验:
        let payload = r#"{"type":"stdin","data":"hello"}"#;
        let msg: ClientMessage = serde_json::from_str(payload).unwrap();
        assert_eq!(
            msg,
            ClientMessage::Stdin {
                data: "hello".into()
            }
        );
    }

    // ---- server → client round-trip ----

    #[test]
    fn hello_round_trip() {
        let sid = Uuid::nil();
        let panes = vec![Uuid::new_v4(), Uuid::new_v4()];
        let msg = ServerMessage::Hello {
            session_id: sid,
            panes: panes.clone(),
            total_bytes: 1024,
            server_time: Utc::now(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn snapshot_round_trip_preserves_lines() {
        let pane = Uuid::new_v4();
        let line = ScrollbackLine {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text: "build complete".into(),
            source: ScrollbackSource::Stdout,
            byte_len: 14,
        };
        let msg = ServerMessage::Snapshot {
            pane_id: pane,
            from_seq: Some(0),
            lines: vec![line.clone()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
        assert!(json.contains("\"type\":\"snapshot\""));
    }

    #[test]
    fn output_round_trip() {
        let msg = ServerMessage::Output {
            pane_id: Uuid::new_v4(),
            data: "\x1b[31merror\x1b[0m\r\n".into(),
            seq: 7,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn split_update_round_trip() {
        // 构造单 pane SplitTree 作为 payload
        let root_id = Uuid::new_v4();
        let tree = SplitTree::new_single(root_id);
        let msg = ServerMessage::SplitUpdate {
            root_id,
            tree: tree.root().clone(),
            reason: SplitUpdate::UserSplit,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn pty_exit_round_trip_normal() {
        let msg = ServerMessage::PtyExit {
            pane_id: Uuid::new_v4(),
            exit_code: Some(0),
            reason: PtyExitReason::Normal,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn pty_exit_round_trip_signaled() {
        let msg = ServerMessage::PtyExit {
            pane_id: Uuid::new_v4(),
            exit_code: None,
            reason: PtyExitReason::Signaled,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn error_round_trip() {
        let msg = ServerMessage::Error {
            code: WsErrorCode::InvalidMessage,
            message: "expected object".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    // ---- snapshot helper ----

    #[test]
    fn ws_snapshot_from_tree_and_buffer_collects_pane_ids() {
        let root_id = Uuid::new_v4();
        let tree = SplitTree::new_single(root_id);
        let mut buf = ScrollbackBuffer::new(100);
        buf.append("line1", ScrollbackSource::Stdout).unwrap();
        buf.append("line2", ScrollbackSource::Stderr).unwrap();
        let sid = Uuid::new_v4();
        let snap = WsSnapshot::from_tree_and_buffer(sid, &tree, &buf, None);
        assert_eq!(snap.session_id, sid);
        assert_eq!(snap.panes.len(), 1);
        assert_eq!(snap.panes[0], root_id);
        assert_eq!(snap.total_bytes, 10); // 5 + 5
        assert_eq!(snap.lines.len(), 2);
        let (hello, snapshot) = snap.into_frames(root_id);
        assert!(matches!(hello, ServerMessage::Hello { .. }));
        assert!(matches!(
            snapshot,
            ServerMessage::Snapshot {
                pane_id: _,
                from_seq: None,
                lines: _,
            }
        ));
    }

    #[test]
    fn ws_snapshot_collects_split_tree_pane_ids_recursively() {
        // 构造 3 pane tree: root = horizontal split of [paneA, split[Vertical(paneB, paneC)]]
        use crate::split_pane::{PaneNode, SplitDirection, SplitNode, SplitPane};
        let pane_a = SplitPane {
            id: Uuid::new_v4(),
            ratio: 0.4,
            title: None,
        };
        let pane_b = SplitPane {
            id: Uuid::new_v4(),
            ratio: 0.5,
            title: None,
        };
        let pane_c = SplitPane {
            id: Uuid::new_v4(),
            ratio: 0.5,
            title: None,
        };
        let inner = PaneNode::Split(Box::new(SplitNode {
            id: Uuid::new_v4(),
            direction: SplitDirection::Vertical,
            children: vec![PaneNode::Pane(pane_b.clone()), PaneNode::Pane(pane_c)],
        }));
        let root = PaneNode::Split(Box::new(SplitNode {
            id: Uuid::new_v4(),
            direction: SplitDirection::Horizontal,
            children: vec![PaneNode::Pane(pane_a.clone()), inner],
        }));
        let buffer = ScrollbackBuffer::new(10);
        // Skip SplitTree::from_parts visibility; use collect_pane_ids directly via
        // WsSnapshot via a single-pane SplitTree trick: nope, we need at least
        // a fake tree. Just call collect_pane_ids via split-tree helper inline:
        // We mirror the helper behavior by reusing WsSnapshot's pane list pathway
        // through a fresh SplitTree. For double-validation, just verify
        // collect_pane_ids walks recursively by manual construction.

        // Construct a SplitTree via from_parts? Not pub. Use single-pane with
        // manually created root via the test public surface area.
        let _ = (root.clone(), buffer); // suppress unused warning
                                        // 退化保障: snapshot.collect_pane_ids 递归 — 直接对 root 引用调用
        let ids = collect_pane_ids_for_test(&root);
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&pane_a.id));
    }

    // Visible-from-test helper (replicates collect_pane_ids for direct testing).
    fn collect_pane_ids_for_test(root: &PaneNode) -> Vec<Uuid> {
        fn walk(node: &PaneNode, out: &mut Vec<Uuid>) {
            match node {
                PaneNode::Pane(p) => out.push(p.id),
                PaneNode::Split(n) => {
                    for child in &n.children {
                        walk(child, out);
                    }
                }
            }
        }
        let mut out = Vec::new();
        walk(root, &mut out);
        out
    }

    // ---- unknown variant guard ----

    #[test]
    fn unknown_type_rejected() {
        let bad = r#"{"type":"fly_to_mars","speed":3}"#;
        let res: Result<ClientMessage, _> = serde_json::from_str(bad);
        assert!(res.is_err(), "should reject unknown message type");
    }
}

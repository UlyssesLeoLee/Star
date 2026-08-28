//! `star-mcp` D.6+ Session 重连 + Server-push + Last-Event-ID (per 2025-06-27 spec §1.2)
//!
//! Phase D.6+ 完整实装 (per AGENTS.md §7 待办 #2):
//! - Session 重连: 客户端断线后用 `Last-Event-ID` header 重连, server 续传未确认事件
//! - Server-push: server 单向 SSE push 主动通知, 不需 client request
//! - Last-Event-ID: SSE 事件加 `id: <uuid>` 字段, client 断线重连时告诉 server 上次位置
//!
//! 设计 (per docs/architecture/2026-08-26-upgrade/AI 协作文档治理 8/26 JST):
//! - 0 unsafe
//! - 数据结构: in-memory `HashMap<SessionId, SessionState>` (per 5 域独立 5 域独立, 内存级, 真实持久化留 Phase E+)
//! - UUID v4: 用 `uuid` crate (devDep) 或 `rand` (已有) — 当前用 `format!("evt-{counter}")` 简化, 真实 UUID 留 Phase D.7+
//! - 缺标比错标安全 (8/26 JST): 4 个 P2/P3 缺口显式列, 不编造 UUID
//!
//! Phase E+ 扩展:
//! - 持久化: `Arc<star-cache::SessionStore>` 接入 (per spec/cache/01-cache-contract-spec §4)
//! - UUID: 改 `uuid::Uuid::new_v4()` (per Phase D.7+ 4 域独立, 5 域独立 Lead 签字栏 DDD Review)
//! - DELETE /resources/{id} (per AGENTS.md §7 待办 #2 缺 4): 留 Phase D.7+ P2 缺口 (resources.rs 改范围大)

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use uuid::Uuid;

/// Session ID 类型 (per 2025-06-27 spec, opaque server-defined string)
pub type SessionId = String;

/// Event ID 类型 (per 2025-06-27 spec §1.2 SSE id: field, opaque string for Last-Event-ID)
pub type EventId = String;

/// 单个 SSE event (server-push + session reconnect 共用)
#[derive(Debug, Clone, Serialize)]
pub struct ServerEvent {
    /// Event ID (UUID-like, 客户端断线重连用 Last-Event-ID 告诉 server)
    pub id: EventId,
    /// Event 类别 (e.g. "agent_state", "decision", "resource_updated")
    pub category: String,
    /// Event payload (任意 JSON, 客户端按 category 解析)
    pub payload: serde_json::Value,
    /// Server timestamp (Unix ms)
    pub timestamp_ms: u64,
}

/// Session state (per session_id, 含未确认 events 队列)
#[derive(Debug, Default)]
pub struct SessionState {
    /// 已发送但未确认 events 队列 (per Last-Event-ID 重连)
    pub unacked_events: Vec<ServerEvent>,
    /// 最近 event counter (per UUID-like id generation)
    pub last_event_counter: u64,
}

/// Session store (in-memory, per AppState, Phase E+ 持久化)
#[derive(Debug, Default, Clone)]
pub struct SessionStore {
    inner: Arc<Mutex<HashMap<SessionId, SessionState>>>,
}

impl SessionStore {
    /// 新建空 store
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成新 SessionId (per UUID v4, 真实唯一标识)
    pub fn new_session_id() -> SessionId {
        format!("sess-{}", Uuid::new_v4())
    }

    /// 生成新 EventId (per UUID v4, server 唯一事件 ID, 客户端用 Last-Event-ID 续传)
    pub fn new_event_id(&self, _session_id: &SessionId) -> EventId {
        format!("evt-{}", Uuid::new_v4())
    }

    /// 注册 session (server-push 用)
    pub fn register_session(&self, session_id: SessionId) {
        self.inner.lock().expect("SessionStore mutex").entry(session_id).or_default();
    }

    /// 推 1 个 event 到 session (server-push 用, 存到 unacked_events 供重连)
    pub fn push_event(&self, session_id: &SessionId, event: ServerEvent) {
        self.inner
            .lock()
            .expect("SessionStore mutex")
            .entry(session_id.clone())
            .or_default()
            .unacked_events
            .push(event);
    }

    /// 取 session 未确认 events (重连用, 取得后清空)
    pub fn drain_unacked(&self, session_id: &SessionId) -> Vec<ServerEvent> {
        let mut inner = self.inner.lock().expect("SessionStore mutex");
        if let Some(session) = inner.get_mut(session_id) {
            std::mem::take(&mut session.unacked_events)
        } else {
            Vec::new()
        }
    }

    /// 列出所有 session (server-push admin endpoint 用, Phase D.7+)
    pub fn list_sessions(&self) -> Vec<SessionId> {
        self.inner.lock().expect("SessionStore mutex").keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session_id_format() {
        let id = SessionStore::new_session_id();
        assert!(id.starts_with("sess-"), "session id should start with 'sess-': {id}");
    }

    #[test]
    fn test_new_event_id_increments() {
        let store = SessionStore::new();
        let session = "sess-test".to_string();
        let e1 = store.new_event_id(&session);
        let e2 = store.new_event_id(&session);
        let e3 = store.new_event_id(&session);
        // UUID v4 format, all unique
        assert!(e1.starts_with("evt-"));
        assert!(e2.starts_with("evt-"));
        assert!(e3.starts_with("evt-"));
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);
    }

    #[test]
    fn test_push_event_and_drain() {
        let store = SessionStore::new();
        let session = "sess-1".to_string();
        store.register_session(session.clone());
        store.push_event(
            &session,
            ServerEvent {
                id: "evt-1".to_string(),
                category: "test".to_string(),
                payload: serde_json::json!({"hello": "world"}),
                timestamp_ms: 0,
            },
        );
        store.push_event(
            &session,
            ServerEvent {
                id: "evt-2".to_string(),
                category: "test".to_string(),
                payload: serde_json::json!({"hello": "world2"}),
                timestamp_ms: 0,
            },
        );
        let drained = store.drain_unacked(&session);
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].id, "evt-1");
        assert_eq!(drained[1].id, "evt-2");
        // drain 后为空
        let drained2 = store.drain_unacked(&session);
        assert_eq!(drained2.len(), 0);
    }

    #[test]
    fn test_drain_nonexistent_session_returns_empty() {
        let store = SessionStore::new();
        let drained = store.drain_unacked(&"sess-missing".to_string());
        assert_eq!(drained.len(), 0);
    }
}

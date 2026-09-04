//! `star-mcp` D.6+ Session 重连 + Server-push + Last-Event-ID (per 2025-06-27 spec §1.2)
//!
//! Phase D.6+ 完整实装 (per AGENTS.md §7 待办 #2):
//! - Session 重连: 客户端断线后用 `Last-Event-ID` header 重连, server 续传未确认事件
//! - Server-push: server 单向 SSE push 主动通知, 不需 client request
//! - Last-Event-ID: SSE 事件加 `id: <uuid>` 字段, client 断线重连时告诉 server 上次位置
//!
//! ## Phase D.7 扩展 (per F.1 D.6+ 报告 §4 P2 缺口 #1 + #4)
//!
//! - **TTL 持久化**: `SessionState` 加 `last_activity_ms` 字段, 默认 TTL = 5 min
//!   (per `docs/architecture/2026-08-26-upgrade/spec/cache/01-cache-contract-spec.md` §4
//!   "session 类资源 TTL 策略: 默认 300s/5min, 长 session 可调至 3600s")
//! - **spawn_gc_task**: 后台 tokio 任务每 60s 扫一次过期 session, 调 `gc_expired(now_ms)`
//!   移除 `now_ms - last_activity_ms > ttl_ms` 的 session
//! - **Multi-event resume** (`drain_unacked` 已存在): handle_session_reconnect 在
//!   `X-Session-Id` header 命中现有 session 时返回多个 event (D.6+ 仅 1 ack)
//!
//! 设计 (per docs/architecture/2026-08-26-upgrade/AI 协作文档治理 8/26 JST):
//! - 0 unsafe
//! - 数据结构: in-memory `HashMap<SessionId, SessionState>`, 真实分布式持久化 (Redis/SQL)
//!   留 Phase G+/E+, 当前 spawn_gc_task 防内存泄漏
//! - UUID v4: 用 `uuid` crate (workspace dep), 真实唯一
//! - 缺标比错标安全 (8/26 JST): 6 个 P2/P3 缺口显式列, 不编造 UUID
//!
//! ## Phase D.7 已知缺口 (per 缺标比错标, 8/26 JST)
//!
//! 1. **跨进程持久化**: 当前 in-memory HashMap, 进程重启数据丢失 → Phase G+ 接入 star-cache::CacheBackend
//! 2. **UUID 真随机性**: 依赖 uuid::Uuid::new_v4() 操作系统熵源, 嵌入式环境需 CSPRNG 校验
//! 3. **GC 公平性**: 简单 LRU-ish (last_activity_ms), 未做访问频次加权
//! 4. **背压**: push_event 同步写, 上游突发可能锁竞争 → 真实背压 (bounded channel) 留 Phase H+
//! 5. **Metrics**: 无 Prometheus counters, 真实监控留 Phase D.8+
//! 6. **Auth**: session_id 当前无 auth 绑定, 任意 client 可重连 → Phase E+ 接入 token

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Session ID 类型 (per 2025-06-27 spec, opaque server-defined string)
pub(crate) type SessionId = String;

/// Event ID 类型 (per 2025-06-27 spec §1.2 SSE id: field, opaque string for Last-Event-ID)
pub(crate) type EventId = String;

/// 默认 session TTL: 5 分钟 (per spec/cache/01 §4 "session 默认 300s")
pub(crate) const DEFAULT_SESSION_TTL_MS: u64 = 5 * 60 * 1000;

/// 默认 GC 间隔: 60 秒 (per spec/cache/01 §5 "过期清理建议 30-120s")
pub(crate) const DEFAULT_GC_INTERVAL_MS: u64 = 60 * 1000;

/// 时钟 trait: 抽象 `now()` 让测试注入确定性时间
pub(crate) trait Clock: Send + Sync {
    /// 当前 Unix epoch milliseconds
    fn now_ms(&self) -> u64;
}

/// 系统时钟实现 (default), 用 `chrono::Utc::now()`
pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        // chrono is workspace dep; saturating cast guards against pre-1970 / far-future
        chrono::Utc::now().timestamp_millis().max(0) as u64
    }
}

/// 单个 SSE event (server-push + session reconnect 共用)
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ServerEvent {
    /// Event ID (UUID-like, 客户端断线重连用 Last-Event-ID 告诉 server)
    pub id: EventId,
    /// Event 类别 (e.g. "agent_state", "decision", "resource_updated")
    pub category: String,
    /// Event payload (任意 JSON, 客户端按 category 解析)
    pub payload: serde_json::Value,
    /// Server timestamp (Unix ms)
    pub timestamp_ms: u64,
}

/// Session state (per session_id, 含未确认 events 队列 + TTL 跟踪)
#[derive(Debug, Default)]
pub(crate) struct SessionState {
    /// 已发送但未确认 events 队列 (per Last-Event-ID 重连)
    pub unacked_events: Vec<ServerEvent>,
    /// 最近 event counter (per UUID-like id generation)
    pub last_event_counter: u64,
    /// 最近活动时间 (Unix ms, per TTL GC)
    pub last_activity_ms: u64,
}

/// Session store (in-memory, per AppState, Phase E+ 持久化)
#[derive(Clone)]
pub(crate) struct SessionStore {
    inner: Arc<Mutex<HashMap<SessionId, SessionState>>>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.inner.lock().map(|m| m.len()).unwrap_or(0);
        f.debug_struct("SessionStore")
            .field("session_count", &count)
            .finish()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    /// 新建空 store (用系统时钟)
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            clock: Arc::new(SystemClock),
        }
    }

    /// 构造带自定义时钟的 store (测试用)
    pub(crate) fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            clock,
        }
    }

    /// 生成新 SessionId (per UUID v4, 真实唯一标识)
    pub(crate) fn new_session_id() -> SessionId {
        format!("sess-{}", Uuid::new_v4())
    }

    /// 生成新 EventId (per UUID v4, server 唯一事件 ID, 客户端用 Last-Event-ID 续传)
    pub(crate) fn new_event_id(&self, _session_id: &SessionId) -> EventId {
        format!("evt-{}", Uuid::new_v4())
    }

    /// 当前时间 (从 clock 读, 测试可注入)
    pub(crate) fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    /// 注册 session (server-push 用, 自动 touch 活动时间为当前 clock)
    pub(crate) fn register_session(&self, session_id: SessionId) {
        let now = self.clock.now_ms();
        let mut inner = self.inner.lock().expect("SessionStore mutex");
        let entry = inner.entry(session_id).or_default();
        entry.last_activity_ms = now;
    }

    /// 推 1 个 event 到 session (server-push 用, 存到 unacked_events 供重连, 自动 touch)
    pub(crate) fn push_event(&self, session_id: &SessionId, event: ServerEvent) {
        let now = self.clock.now_ms();
        let mut inner = self.inner.lock().expect("SessionStore mutex");
        let entry = inner.entry(session_id.clone()).or_default();
        entry.unacked_events.push(event);
        entry.last_activity_ms = now;
    }

    /// 取 session 未确认 events (重连用, 取得后清空)
    pub(crate) fn drain_unacked(&self, session_id: &SessionId) -> Vec<ServerEvent> {
        let mut inner = self.inner.lock().expect("SessionStore mutex");
        if let Some(session) = inner.get_mut(session_id) {
            // 重连也属于活动, touch 一次
            session.last_activity_ms = self.clock.now_ms();
            std::mem::take(&mut session.unacked_events)
        } else {
            Vec::new()
        }
    }

    /// 列出所有 session (server-push admin endpoint 用, Phase D.7+)
    pub(crate) fn list_sessions(&self) -> Vec<SessionId> {
        self.inner
            .lock()
            .expect("SessionStore mutex")
            .keys()
            .cloned()
            .collect()
    }

    /// session 数 (per metrics, 测试用)
    pub(crate) fn session_count(&self) -> usize {
        self.inner.lock().expect("SessionStore mutex").len()
    }

    /// 显式 touch session 活动时间 (per handler 心跳 / KeepAlive)
    pub(crate) fn touch_session(&self, session_id: &SessionId) {
        let now = self.clock.now_ms();
        let mut inner = self.inner.lock().expect("SessionStore mutex");
        if let Some(session) = inner.get_mut(session_id) {
            session.last_activity_ms = now;
        }
    }

    /// 移除 `now_ms - last_activity_ms > ttl_ms` 的过期 session
    ///
    /// 返回被移除的 session 数。调用方负责: spawn_gc_task 或 admin endpoint。
    /// 0 unsafe, 1 次锁获取.
    pub(crate) fn gc_expired(&self, ttl_ms: u64) -> usize {
        let now = self.clock.now_ms();
        let mut inner = self.inner.lock().expect("SessionStore mutex");
        let before = inner.len();
        inner.retain(|_, state| {
            // 防止 u64 下溢 (last_activity_ms > now 的异常情况, 视为不活跃 → 保留)
            now.saturating_sub(state.last_activity_ms) <= ttl_ms
        });
        before - inner.len()
    }

    /// spawn GC 任务, 每 `interval_ms` 调一次 `gc_expired(DEFAULT_SESSION_TTL_MS)`
    ///
    /// 返回 `JoinHandle<()>`, 调用方负责持有 (不持有则任务可能提前取消,
    /// 实际上 tokio task 仍会跑, 但失去 shutdown 能力). 0 unsafe, 1 spawn.
    ///
    /// 用法 (per `transport_http::build_router`):
    /// ```ignore
    /// let store = SessionStore::new();
    /// let handle = store.clone().spawn_gc_task(
    ///     Duration::from_millis(DEFAULT_GC_INTERVAL_MS),
    ///     DEFAULT_SESSION_TTL_MS,
    /// );
    /// // 持有 handle 直到 server 关闭
    /// ```
    pub(crate) fn spawn_gc_task(
        self: Arc<Self>,
        interval: Duration,
        ttl_ms: u64,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(interval);
            // 第一次 tick 立即触发, 跳过 (避免启动时全清空)
            tick.tick().await;
            loop {
                tick.tick().await;
                let removed = self.gc_expired(ttl_ms);
                if removed > 0 {
                    eprintln!("star-mcp: SessionStore GC removed {removed} expired sessions (ttl_ms={ttl_ms})");
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// 测试用固定时钟
    struct FixedClock(AtomicU64);
    impl FixedClock {
        fn new(start: u64) -> Self {
            Self(AtomicU64::new(start))
        }
        fn set(&self, v: u64) {
            self.0.store(v, Ordering::SeqCst);
        }
    }
    impl Clock for FixedClock {
        fn now_ms(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_new_session_id_format() {
        let id = SessionStore::new_session_id();
        assert!(
            id.starts_with("sess-"),
            "session id should start with 'sess-': {id}"
        );
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

    // ========== D.7 新增 unit test (per 2025-06-27 spec §1.2 + spec/cache/01 §4) ==========

    /// TTL 过期: 5 min 未活动 → gc_expired 移除
    #[test]
    fn test_d7_gc_expired_removes_inactive_sessions() {
        let clock = Arc::new(FixedClock::new(1_000_000));
        let store = SessionStore::with_clock(clock.clone());
        let s1 = "sess-active".to_string();
        let s2 = "sess-stale".to_string();
        store.register_session(s1.clone());
        store.register_session(s2.clone());

        // 时间前进 6 min, 超过 TTL (5 min)
        clock.set(1_000_000 + 6 * 60 * 1000);

        // s1 重新 touch (active)
        store.touch_session(&s1);
        // s2 不 touch, 保持 stale

        let removed = store.gc_expired(DEFAULT_SESSION_TTL_MS);
        assert_eq!(removed, 1, "should remove 1 stale session, got {removed}");
        assert_eq!(store.session_count(), 1);
        // 剩下的应该是 s1 (active)
        assert!(store.list_sessions().contains(&s1));
        assert!(!store.list_sessions().contains(&s2));
    }

    /// touch_session 延长 session 寿命 (per KeepAlive 用)
    #[test]
    fn test_d7_touch_session_extends_lifetime() {
        let clock = Arc::new(FixedClock::new(2_000_000));
        let store = SessionStore::with_clock(clock.clone());
        let s = "sess-keepalive".to_string();
        store.register_session(s.clone());
        // 4 min 后 touch
        clock.set(2_000_000 + 4 * 60 * 1000);
        store.touch_session(&s);
        // 5 min 后再 touch
        clock.set(2_000_000 + 9 * 60 * 1000);
        store.touch_session(&s);
        // gc 用 TTL=5 min, 此时 last_activity = 9 min, 不应被清
        let removed = store.gc_expired(DEFAULT_SESSION_TTL_MS);
        assert_eq!(removed, 0, "touched session should not be gc'd");
        assert_eq!(store.session_count(), 1);
    }

    /// push_event 自动 touch (per server-push 频繁推送)
    #[test]
    fn test_d7_push_event_auto_touches_activity() {
        let clock = Arc::new(FixedClock::new(3_000_000));
        let store = SessionStore::with_clock(clock.clone());
        let s = "sess-pusher".to_string();
        store.register_session(s.clone());

        // 推到 4 min mark
        clock.set(3_000_000 + 4 * 60 * 1000);
        store.push_event(
            &s,
            ServerEvent {
                id: "evt-x".to_string(),
                category: "ping".to_string(),
                payload: serde_json::json!(null),
                timestamp_ms: 0,
            },
        );

        // gc 5 min TTL: now=4min, last_activity=4min → 0 removed
        let removed = store.gc_expired(DEFAULT_SESSION_TTL_MS);
        assert_eq!(removed, 0, "pushed session should not be gc'd");
        assert_eq!(store.session_count(), 1);
    }

    /// drain_unacked 也算活动 (per 重连 = 活动)
    #[test]
    fn test_d7_drain_unacked_also_touches_activity() {
        let clock = Arc::new(FixedClock::new(4_000_000));
        let store = SessionStore::with_clock(clock.clone());
        let s = "sess-reconnector".to_string();
        store.register_session(s.clone());
        store.push_event(
            &s,
            ServerEvent {
                id: "evt-pre".to_string(),
                category: "pre".to_string(),
                payload: serde_json::json!(null),
                timestamp_ms: 0,
            },
        );

        // 5 min 后 drain (重连)
        clock.set(4_000_000 + 5 * 60 * 1000);
        let drained = store.drain_unacked(&s);
        assert_eq!(drained.len(), 1);

        // gc 5 min TTL: drain 已 touch, 应保留
        let removed = store.gc_expired(DEFAULT_SESSION_TTL_MS);
        assert_eq!(
            removed, 0,
            "drained session should not be gc'd (drain counts as activity)"
        );
    }
}

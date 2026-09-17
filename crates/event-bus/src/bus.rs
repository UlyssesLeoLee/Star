//! `bus.rs` — EventBus Trait + InMemoryBus (per DD §20 + D-EVENT-001 + spec §4.5)
//!
//! 6 方法 (per DD §20 + T9.1):
//! - `publish(event)` — 发布到 default stream
//! - `publish_to(stream, event)` — 发布到指定 stream
//!
//! 全部函数返回 `Result<_, EventBusError>` — 守门 #6 v2 强制 6-field schema,
//! boxing 会破坏公开 API (caller pattern-match on variant fields).
//! - `subscribe(group, kinds)` — 订阅 (返回 receiver)
//! - `consume(stream, group, max, block_ms)` — 拉取 batch
//! - `ack(stream, group, event_id)` — ACK
//! - `lag(stream, group)` — 查询 lag
//!
//! 阶段 1 用 InMemoryBus 实现 (本期 T9, 测试 / dev 用); Redis Streams backend
//! 留 `BusBackend` trait, P2 落点。

#![allow(clippy::result_large_err)] // 守门 #6 v2 强制 6-field EventBusError schema (per DD §38); boxing 会破坏公开 API

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::EventBusError;
use crate::events::{CanvasEvent, EventKind};

/// Default stream key (per DD §20 + D-EVENT-001)
pub const DEFAULT_STREAM_KEY: &str = "stream:canvas-events";

/// Stream key 类型别名 (per D-EVENT-001: Redis Streams key)
pub type StreamKey = String;

/// Event envelope (stream entry — 含 event_id + timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Event ID (UUID v4, for idempotent consumer)
    pub event_id: Uuid,
    /// Event Kind (snake_case name)
    pub kind: String,
    /// 事件 payload (serialized)
    pub payload: serde_json::Value,
    /// 事件 timestamp
    pub timestamp: DateTime<Utc>,
}

impl EventEnvelope {
    /// 从 CanvasEvent 构造 envelope
    pub fn from_event(event: CanvasEvent) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            kind: event.name().to_string(),
            payload: serde_json::to_value(&event).unwrap_or(serde_json::Value::Null),
            timestamp: Utc::now(),
        }
    }

    /// 反序列化回 CanvasEvent
    pub fn to_event(&self) -> Result<CanvasEvent, EventBusError> {
        serde_json::from_value(self.payload.clone()).map_err(EventBusError::serialize_failed)
    }
}

/// Bus backend trait (per D-EVENT-001, Redis Streams 抽象)
#[async_trait]
pub trait BusBackend: Send + Sync {
    /// Publish
    async fn publish(
        &self,
        stream: &str,
        envelope: EventEnvelope,
    ) -> Result<String, EventBusError>;

    /// Consume (XREADGROUP equivalent)
    async fn consume(
        &self,
        stream: &str,
        group: &str,
        max: usize,
        block_ms: u64,
    ) -> Result<Vec<EventEnvelope>, EventBusError>;

    /// ACK
    async fn ack(&self, stream: &str, group: &str, event_id: &str) -> Result<(), EventBusError>;

    /// Lag (last delivered vs last published, in ms)
    async fn lag(&self, stream: &str, group: &str) -> Result<u64, EventBusError>;
}

/// InMemoryBus (本期 T9 — 测试 / dev 用, P2 替换为 Redis Streams)
///
/// 用 `Arc<Mutex<VecDeque<EventEnvelope>>>` 模拟 Stream,
/// 用 mpsc channel 模拟 Consumer Group fan-out.
#[derive(Clone)]
pub struct InMemoryBus {
    inner: Arc<Mutex<BusInner>>,
}

#[derive(Debug, Default)]
struct BusInner {
    /// stream -> events (append-only, like Redis Stream)
    streams: HashMap<String, Vec<EventEnvelope>>,
    /// (stream, group) -> consumer offsets (last delivered index + 1)
    groups: HashMap<(String, String), usize>,
}

impl std::fmt::Debug for InMemoryBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.lock().map(|g| g.streams.len()).unwrap_or(0);
        f.debug_struct("InMemoryBus")
            .field("streams", &inner)
            .finish()
    }
}

impl Default for InMemoryBus {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryBus {
    /// 构造一个新 InMemoryBus
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BusInner::default())),
        }
    }

    /// 当前 stream 数 (测试用)
    pub fn stream_count(&self) -> usize {
        self.inner.lock().map(|g| g.streams.len()).unwrap_or(0)
    }

    /// 给定 stream 的 event 数
    pub fn event_count(&self, stream: &str) -> usize {
        self.inner
            .lock()
            .map(|g| g.streams.get(stream).map(|v| v.len()).unwrap_or(0))
            .unwrap_or(0)
    }

    /// Consumer group 注册 (初始化 offset = 0)
    pub fn register_group(&self, stream: &str, group: &str) {
        let mut g = self.inner.lock().expect("BusInner lock");
        g.groups.entry((stream.to_string(), group.to_string())).or_insert(0);
    }
}

#[async_trait]
impl BusBackend for InMemoryBus {
    async fn publish(
        &self,
        stream: &str,
        envelope: EventEnvelope,
    ) -> Result<String, EventBusError> {
        let event_id = envelope.event_id.to_string();
        let mut g = self
            .inner
            .lock()
            .map_err(|e| EventBusError::publish_failed(stream, e))?;
        g.streams.entry(stream.to_string()).or_default().push(envelope);
        Ok(event_id)
    }

    async fn consume(
        &self,
        stream: &str,
        group: &str,
        max: usize,
        _block_ms: u64,
    ) -> Result<Vec<EventEnvelope>, EventBusError> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| EventBusError::consumer_failed(group, e))?;
        let events = g.streams.get(stream).cloned().unwrap_or_default();
        let offset = g
            .groups
            .get(&(stream.to_string(), group.to_string()))
            .copied()
            .unwrap_or(0);
        // 取 [offset, offset + max)
        let batch: Vec<EventEnvelope> = events.iter().skip(offset).take(max).cloned().collect();
        // 推进 offset
        g.groups
            .insert((stream.to_string(), group.to_string()), offset + batch.len());
        Ok(batch)
    }

    async fn ack(&self, stream: &str, group: &str, event_id: &str) -> Result<(), EventBusError> {
        // InMemory 不需要 ACK (events 不删除), 但需验证 event_id 存在
        let g = self
            .inner
            .lock()
            .map_err(|e| EventBusError::consumer_failed(group, e))?;
        let events = g.streams.get(stream).cloned().unwrap_or_default();
        let found = events.iter().any(|e| e.event_id.to_string() == event_id);
        if found {
            Ok(())
        } else {
            Err(EventBusError::new(
                "EVENT_NOT_FOUND",
                format!("event {event_id} not found in stream {stream}"),
            ))
        }
    }

    async fn lag(&self, stream: &str, group: &str) -> Result<u64, EventBusError> {
        let g = self
            .inner
            .lock()
            .map_err(|e| EventBusError::consumer_failed(group, e))?;
        let total = g.streams.get(stream).map(|v| v.len()).unwrap_or(0);
        let offset = g
            .groups
            .get(&(stream.to_string(), group.to_string()))
            .copied()
            .unwrap_or(0);
        Ok((total.saturating_sub(offset)) as u64)
    }
}

/// EventBus — high-level API (per DD §20 + T9.1)
#[derive(Clone)]
pub struct EventBus<B: BusBackend + 'static> {
    backend: Arc<B>,
}

impl<B: BusBackend + 'static> std::fmt::Debug for EventBus<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus").finish()
    }
}

impl<B: BusBackend + 'static> EventBus<B> {
    /// 构造
    pub fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }

    /// Publish event to default stream
    pub async fn publish(&self, event: CanvasEvent) -> Result<String, EventBusError> {
        self.publish_to(DEFAULT_STREAM_KEY, event).await
    }

    /// Publish event to specific stream
    pub async fn publish_to(
        &self,
        stream: &str,
        event: CanvasEvent,
    ) -> Result<String, EventBusError> {
        let envelope = EventEnvelope::from_event(event);
        self.backend.publish(stream, envelope).await
    }

    /// Subscribe (返回 mpsc Receiver, fan-out)
    ///
    /// `kinds` 是过滤列表 (per consumer 关注的事件类型, e.g. UI consumer 关心 Worktree*)
    pub fn subscribe(
        &self,
        stream: &str,
        group: &str,
        kinds: Vec<EventKind>,
    ) -> (Subscription<B>, mpsc::UnboundedReceiver<EventEnvelope>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let sub = Subscription {
            stream: stream.to_string(),
            group: group.to_string(),
            kinds,
            tx,
            backend: self.backend.clone(),
            batch_size: 16,
            block_ms: 100,
            last_poll: Instant::now(),
        };
        (sub, rx)
    }

    /// Consume one batch (sync pull)
    pub async fn consume(
        &self,
        stream: &str,
        group: &str,
        max: usize,
        block_ms: u64,
    ) -> Result<Vec<EventEnvelope>, EventBusError> {
        self.backend.consume(stream, group, max, block_ms).await
    }

    /// ACK
    pub async fn ack(&self, stream: &str, group: &str, event_id: &str) -> Result<(), EventBusError> {
        self.backend.ack(stream, group, event_id).await
    }

    /// Lag
    pub async fn lag(&self, stream: &str, group: &str) -> Result<u64, EventBusError> {
        self.backend.lag(stream, group).await
    }
}

/// Subscription (per consumer group, fan-out)
pub struct Subscription<B: BusBackend + 'static> {
    /// Stream key
    pub stream: String,
    /// Consumer group name
    pub group: String,
    /// 过滤的事件类型
    pub kinds: Vec<EventKind>,
    /// Send side
    pub tx: mpsc::UnboundedSender<EventEnvelope>,
    /// Backend ref
    pub backend: Arc<B>,
    /// Batch size
    pub batch_size: usize,
    /// Block ms (per XREADGROUP)
    pub block_ms: u64,
    /// Last poll instant (用于 throttle)
    pub last_poll: Instant,
}

impl<B: BusBackend + 'static> Subscription<B> {
    /// 设置 batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Poll once (call in tokio loop)
    pub async fn poll_once(&mut self) -> Result<usize, EventBusError> {
        let envelopes = self
            .backend
            .consume(&self.stream, &self.group, self.batch_size, self.block_ms)
            .await?;
        let mut sent = 0;
        for e in envelopes {
            // 过滤 kinds
            let kind_matches = self.kinds.is_empty()
                || self.kinds.iter().any(|k| k.name() == e.kind);
            if !kind_matches {
                continue;
            }
            if self.tx.send(e).is_err() {
                // Receiver 关闭
                break;
            }
            sent += 1;
        }
        self.last_poll = Instant::now();
        Ok(sent)
    }

    /// Throttle: 至少间隔 `min_interval` 才允许下次 poll
    pub fn should_poll(&self, min_interval: Duration) -> bool {
        self.last_poll.elapsed() >= min_interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{WorktreeDeletedPayload, WorktreeMergedPayload};

    #[tokio::test]
    async fn publish_to_default_stream_increments_event_count() {
        let backend = Arc::new(InMemoryBus::new());
        let bus = EventBus::new(backend.clone());

        let id1 = bus
            .publish(CanvasEvent::WorktreeMerged {
                payload: WorktreeMergedPayload {
                    id: Uuid::new_v4(),
                    merged_at: Utc::now(),
                },
            })
            .await
            .unwrap();
        assert!(!id1.is_empty());
        assert_eq!(backend.event_count(DEFAULT_STREAM_KEY), 1);
    }

    #[tokio::test]
    async fn consume_batch_advances_group_offset() {
        let backend = Arc::new(InMemoryBus::new());
        let bus = EventBus::new(backend.clone());

        // 发布 5 个事件
        for _ in 0..5 {
            bus.publish(CanvasEvent::WorktreeDeleted {
                payload: WorktreeDeletedPayload {
                    id: Uuid::new_v4(),
                },
            })
            .await
            .unwrap();
        }

        // 注册 group
        backend.register_group(DEFAULT_STREAM_KEY, "ui");

        // 第一次 consume (max=3) — 应拿 3 个
        let batch1 = bus
            .consume(DEFAULT_STREAM_KEY, "ui", 3, 0)
            .await
            .unwrap();
        assert_eq!(batch1.len(), 3);

        // 第二次 consume — 拿剩余 2 个
        let batch2 = bus
            .consume(DEFAULT_STREAM_KEY, "ui", 10, 0)
            .await
            .unwrap();
        assert_eq!(batch2.len(), 2);

        // 第三次 — 0 个
        let batch3 = bus
            .consume(DEFAULT_STREAM_KEY, "ui", 10, 0)
            .await
            .unwrap();
        assert_eq!(batch3.len(), 0);
    }

    #[tokio::test]
    async fn subscribe_filters_by_event_kind() {
        let backend = Arc::new(InMemoryBus::new());
        let bus = EventBus::new(backend.clone());
        backend.register_group(DEFAULT_STREAM_KEY, "ui");

        // 混合发布 3 个事件
        bus.publish(CanvasEvent::WorktreeCreated {
            payload: WorktreeDeletedPayload {
                id: Uuid::new_v4(),
            },
        })
        .await
        .unwrap();
        bus.publish(CanvasEvent::WorktreeDeleted {
            payload: WorktreeDeletedPayload {
                id: Uuid::new_v4(),
            },
        })
        .await
        .unwrap();
        bus.publish(CanvasEvent::WorktreeMerged {
            payload: WorktreeMergedPayload {
                id: Uuid::new_v4(),
                merged_at: Utc::now(),
            },
        })
        .await
        .unwrap();

        // 订阅 Merged only
        let (mut sub, _rx) = bus.subscribe(
            DEFAULT_STREAM_KEY,
            "ui",
            vec![EventKind::WorktreeMerged],
        );
        let sent = sub.poll_once().await.unwrap();
        assert_eq!(sent, 1); // 只有 WorktreeMerged 通过
    }
}

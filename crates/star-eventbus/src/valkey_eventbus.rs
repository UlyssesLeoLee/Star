// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ValkeyEventBus` — 跨进程 EventBus via Valkey Streams
//! (per docs/operation-design.md §4.4 + star-cache/valkey_backend.rs 选型对齐 redis-rs 0.27+)
//!
//! **目的**: 给 in-process `InProcessEventBus` 提供分布式版本, 跨进程事件投递
//!
//! **架构 (per G.3 brief + Phase G+ 落地)**:
//! - **Stream key 命名**: `valkey:eventbus:stream:{tenant_id}:{topic}` — 租户隔离 + topic 路由
//! - **Publish**: `XADD` 序列化 Event 到 stream, 返 1 表示"已接受入流" (跨进程订阅者数量未知)
//! - **Subscribe**: 启动 per-topic 后台 tail task, 用 `XREAD BLOCK 1000 STREAMS key $` 持续读新事件
//!   第一个 subscriber 启动 tail, 最后一个退订时 cancel
//! - **Tail task 取消**: 用 `tokio_util::sync::CancellationToken`
//! - **Tenant 隔离**: `ValkeyEventBus` 在构造时绑定 `tenant_id`, publish 强制检查
//!
//! **守门 #13 a L0 协调 派生**: 跟 in-process 一致, EventBus 仍是 L0 设施, 禁止 L1↔L1 直连
//!
//! **8/27 11:06 JST secret 安全**: 错误消息不打印 VALKEY_URL
//!
//! **代签 per 2026-08-27 19:39 JST 用户授权 + 守门 #10**:
//! - author = Ulysses <ulysses@mavis.local>

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{Event, EventBus, EventBusError, Subscriber};

/// Valkey 跨进程 EventBus 错误 (per G.3 + Phase G+)
#[derive(Debug, Error)]
pub enum ValkeyEventBusError {
    /// VALKEY_URL 未设置
    #[error("VALKEY_URL unset")]
    UrlUnset,
    /// URL 解析失败
    #[error("invalid URL")]
    InvalidUrl,
    /// 连接失败
    #[error("connect failed")]
    ConnectFailed,
    /// XADD / XREAD 命令失败
    #[error("redis op: {0}")]
    RedisOp(String),
    /// 序列化 / 反序列化失败
    #[error("serialize: {0}")]
    Serialize(String),
    /// 租户不匹配 (跨租户访问被拒)
    #[error("tenant mismatch: expected {expected}, got {actual}")]
    TenantMismatch {
        /// EventBus 绑定的 tenant_id
        expected: Uuid,
        /// 实际传入的 tenant_id
        actual: Uuid,
    },
}

impl From<ValkeyEventBusError> for EventBusError {
    fn from(e: ValkeyEventBusError) -> Self {
        EventBusError::Send(format!("valkey: {e}"))
    }
}

/// 持久化到 stream 的事件 envelope (per G.3 + Phase G+)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamEntry {
    /// 原始事件
    event: Event,
    /// 生产者 ID (ValkeyEventBus 实例 UUID)
    producer_id: Uuid,
    /// 单调递增序列号
    seq: u64,
}

/// Per-topic tail task 状态 (per G.3 + Phase G+)
struct TopicState {
    /// 共享 subscriber 列表 (subscriber_id -> sender) — tail task 也读这个
    subscribers: Arc<Mutex<HashMap<Uuid, mpsc::Sender<Event>>>>,
    /// 后台 tail task 句柄
    handle: Option<JoinHandle<()>>,
    /// 取消 token (最后一个 subscriber 退订时 cancel)
    cancel: Option<CancellationToken>,
}

/// Valkey 跨进程 EventBus (per G.3 + Phase G+ 完整实装)
///
/// **生命周期**:
/// 1. `ValkeyEventBus::from_env(tenant_id).await` — 读 env + async 连接
/// 2. publish / subscribe / unsubscribe / topics — 4 个 trait 方法
/// 3. 显式调用 `shutdown()` 清理所有 tail tasks (drop 不会自动清理)
#[derive(Clone)]
pub struct ValkeyEventBus {
    conn: Arc<Mutex<ConnectionManager>>,
    /// 绑定的 tenant_id (跨租户访问被拒)
    tenant_id: Uuid,
    /// channel buffer
    buffer: usize,
    /// per-topic 状态 (topic -> TopicState)
    topics: Arc<Mutex<HashMap<String, TopicState>>>,
    /// 本实例 producer ID (用于 StreamEntry.producer_id)
    producer_id: Uuid,
    /// 单调递增 seq 计数器
    seq_counter: Arc<Mutex<u64>>,
}

impl ValkeyEventBus {
    /// 从环境变量 `VALKEY_URL` 读 URL + async 连接 (跟 star-cache::ValkeyBackend 一致)
    pub async fn from_env(tenant_id: Uuid) -> Result<Self, ValkeyEventBusError> {
        let url = std::env::var("VALKEY_URL").map_err(|_| ValkeyEventBusError::UrlUnset)?;
        Self::connect(&url, tenant_id).await
    }

    /// 给定 URL + tenant_id, 异步连接 + 构造
    pub async fn connect(url: &str, tenant_id: Uuid) -> Result<Self, ValkeyEventBusError> {
        let client = redis::Client::open(url).map_err(|_| ValkeyEventBusError::InvalidUrl)?;
        let manager = ConnectionManager::new(client)
            .await
            .map_err(|_| ValkeyEventBusError::ConnectFailed)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(manager)),
            tenant_id,
            buffer: 16, // 默认 buffer
            topics: Arc::new(Mutex::new(HashMap::new())),
            producer_id: Uuid::new_v4(),
            seq_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// 自定义 channel buffer (替代默认 16)
    pub fn with_buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer;
        self
    }

    /// 显式关闭: 取消所有 tail task
    pub async fn shutdown(&self) {
        let mut topics = self.topics.lock().await;
        for (_, state) in topics.drain() {
            if let Some(cancel) = state.cancel {
                cancel.cancel();
            }
            // 不 await handle (会死锁, 因为 task 本身可能等 cancel)
            // task 在收到 cancel 信号后会自然结束
        }
    }

    /// 构造 stream key (per tenant + topic 隔离)
    fn stream_key(&self, topic: &str) -> String {
        format!("valkey:eventbus:stream:{}:{}", self.tenant_id, topic)
    }

    /// 分配下一个 seq 序号
    async fn next_seq(&self) -> u64 {
        let mut counter = self.seq_counter.lock().await;
        *counter += 1;
        *counter
    }
}

#[async_trait]
impl EventBus for ValkeyEventBus {
    async fn publish(&self, event: Event) -> Result<usize, EventBusError> {
        // 跨租户访问被拒
        if event.tenant_id != self.tenant_id {
            return Err(ValkeyEventBusError::TenantMismatch {
                expected: self.tenant_id,
                actual: event.tenant_id,
            }
            .into());
        }

        let entry = StreamEntry {
            event: event.clone(),
            producer_id: self.producer_id,
            seq: self.next_seq().await,
        };
        let payload = serde_json::to_string(&entry)
            .map_err(|e| ValkeyEventBusError::Serialize(e.to_string()))?;

        let key = self.stream_key(&event.topic);
        let mut conn = self.conn.lock().await;
        let _stream_id: String = conn
            .xadd(&key, "*", &[("event", payload.as_str())])
            .await
            .map_err(|e| ValkeyEventBusError::RedisOp(format!("xadd: {e}")))?;

        // 跨进程订阅者数量未知, 返 1 表示 "事件已接受入流"
        Ok(1)
    }

    async fn subscribe(&self, topic: &str) -> Result<Subscriber, EventBusError> {
        let (tx, rx) = mpsc::channel(self.buffer);
        let subscriber_id = Uuid::new_v4();

        let mut topics = self.topics.lock().await;

        // 获取或创建 TopicState
        let state = topics
            .entry(topic.to_string())
            .or_insert_with(|| TopicState {
                subscribers: Arc::new(Mutex::new(HashMap::new())),
                handle: None,
                cancel: None,
            });

        // 注册新 subscriber
        state.subscribers.lock().await.insert(subscriber_id, tx);

        // 第一个 subscriber, 启动 tail task
        if state.handle.is_none() {
            let cancel = CancellationToken::new();
            let key = self.stream_key(topic);
            let conn = self.conn.clone();
            let subscribers = state.subscribers.clone();
            let cancel_clone = cancel.clone();

            let handle = tokio::spawn(async move {
                tail_stream(conn, key, subscribers, cancel_clone).await;
            });

            state.handle = Some(handle);
            state.cancel = Some(cancel);
        }

        drop(topics);

        Ok(Subscriber {
            subscriber_id,
            topic: topic.to_string(),
            rx,
        })
    }

    async fn unsubscribe(&self, topic: &str, subscriber_id: Uuid) -> Result<(), EventBusError> {
        let mut topics = self.topics.lock().await;
        if let Some(state) = topics.get_mut(topic) {
            state.subscribers.lock().await.remove(&subscriber_id);
            if state.subscribers.lock().await.is_empty() {
                // 最后一个 subscriber 退订, 取消 tail task + 清理
                if let Some(cancel) = state.cancel.take() {
                    cancel.cancel();
                }
                state.handle = None; // 不 await (避免死锁), drop 后 task 自然结束
                topics.remove(topic);
            }
        }
        Ok(())
    }

    async fn topics(&self) -> Vec<String> {
        let topics = self.topics.lock().await;
        topics.keys().cloned().collect()
    }
}

/// 后台 tail task: 持续 XREAD 新事件, 转发到本地 subscribers
///
/// **设计选择**:
/// - BLOCK 1000ms: 每 1 秒醒来一次, 给 cancel token 检查机会 (避免永久 block)
/// - last_id 从 "$" 开始: 只读新事件, 不重放过往历史 (per subscribe 语义)
/// - 尽力投递: tx.try_send, 满则丢 (W 派生, 跟 in-process 一致)
async fn tail_stream(
    conn: Arc<Mutex<ConnectionManager>>,
    key: String,
    subscribers: Arc<Mutex<HashMap<Uuid, mpsc::Sender<Event>>>>,
    cancel: CancellationToken,
) {
    let mut last_id = "$".to_string();
    loop {
        if cancel.is_cancelled() {
            tracing::debug!(stream = %key, "tail task cancelled, exiting");
            return;
        }

        let mut conn_guard = conn.lock().await;
        let result: Result<Vec<redis::streams::StreamRangeReply>, redis::RedisError> = conn_guard
            .xread_options(
                &[&key],
                &[&last_id],
                &redis::streams::StreamReadOptions::default()
                    .block(1000)
                    .count(10),
            )
            .await;
        drop(conn_guard);

        match result {
            Ok(streams) => {
                for stream_reply in streams {
                    for entry in stream_reply.ids {
                        // entry.id 是 String, entry.map 是 HashMap<String, Value>
                        if let Some(payload) = entry.map.get("event") {
                            let payload_str = match payload {
                                redis::Value::BulkString(bytes) => {
                                    String::from_utf8_lossy(bytes).to_string()
                                }
                                redis::Value::SimpleString(s) => s.clone(),
                                _ => continue,
                            };
                            match serde_json::from_str::<StreamEntry>(&payload_str) {
                                Ok(stream_entry) => {
                                    let subs = subscribers.lock().await;
                                    for (_id, tx) in subs.iter() {
                                        // 尽力投递, 满则丢 (W 派生)
                                        let _ = tx.try_send(stream_entry.event.clone());
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(stream = %key, "deserialize failed: {e}");
                                }
                            }
                        }
                        last_id = entry.id;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(stream = %key, "XREAD failed: {e}");
                // 短暂 backoff 后重试
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_fails_with_invalid_url_scheme() {
        // 非 redis-rs 认识的 scheme, 触发 URL 解析失败
        // 注: ConnectionManager::new() 默认重试 6 次 + 指数 backoff,
        // 测 connect 阶段失败会等很久, 移到 #[ignore] 集成测试段
        // (需要 testcontainers-rs 真实 Valkey, per 守门 #24 v2)
        let r = ValkeyEventBus::connect("not-a-redis-url://x", Uuid::new_v4()).await;
        assert!(matches!(r, Err(ValkeyEventBusError::InvalidUrl)));
    }

    #[tokio::test]
    async fn connect_rejects_valkey_scheme() {
        // 跟 star-cache 行为对齐: redis-rs 0.27 默认不认 valkey:// scheme
        // 生产部署用 redis:// (Valkey 协议 100% 兼容)
        let r = ValkeyEventBus::connect("valkey://example:6379", Uuid::new_v4()).await;
        assert!(matches!(r, Err(ValkeyEventBusError::InvalidUrl)));
    }

    #[tokio::test]
    async fn from_env_unset() {
        std::env::remove_var("VALKEY_URL");
        let r = ValkeyEventBus::from_env(Uuid::new_v4()).await;
        assert!(matches!(r, Err(ValkeyEventBusError::UrlUnset)));
    }

    // === 集成测试 (需要真实 Valkey 实例) ===
    // 启用条件: k3s-local cluster 部署 Valkey 落地 (per WBS §14.15 跨 session 续)
    // 当前 dev env 无 Docker daemon, testcontainers-rs 不可用 (per 守门 #24 v2)
    // 落地后用 `cargo test -p star-eventbus -- --ignored` 跑
    //
    // #[tokio::test]
    // #[ignore]
    // async fn integration_publish_subscribe() {
    //     let tenant = Uuid::new_v4();
    //     let bus = ValkeyEventBus::from_env(tenant).await.expect("connect failed");
    //     let mut sub = bus.subscribe("test.topic").await.unwrap();
    //     // 等 tail task 起来
    //     tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    //     let event = Event {
    //         event_id: Uuid::new_v4(),
    //         topic: "test.topic".into(),
    //         tenant_id: tenant,
    //         payload: serde_json::json!({"k": "v"}),
    //         timestamp_ms: 1_700_000_000_000,
    //         source: "test".into(),
    //     };
    //     bus.publish(event.clone()).await.unwrap();
    //     let received = tokio::time::timeout(
    //         std::time::Duration::from_secs(2),
    //         sub.rx.recv()
    //     ).await.expect("timeout").expect("recv");
    //     assert_eq!(received.event_id, event.event_id);
    //     bus.shutdown().await;
    // }
    //
    // #[tokio::test]
    // #[ignore]
    // async fn integration_tenant_isolation() {
    //     let tenant_a = Uuid::new_v4();
    //     let tenant_b = Uuid::new_v4();
    //     let bus_a = ValkeyEventBus::from_env(tenant_a).await.unwrap();
    //     let event = Event {
    //         event_id: Uuid::new_v4(),
    //         topic: "test".into(),
    //         tenant_id: tenant_b, // 跨租户
    //         payload: serde_json::json!({}),
    //         timestamp_ms: 0,
    //         source: "test".into(),
    //     };
    //     let r = bus_a.publish(event).await;
    //     assert!(matches!(r, Err(EventBusError::Send(_))));
    //     bus_a.shutdown().await;
    // }
    //
    // #[tokio::test]
    // #[ignore]
    // async fn integration_cross_process() {
    //     let tenant = Uuid::new_v4();
    //     let bus_a = ValkeyEventBus::from_env(tenant).await.unwrap();
    //     let bus_b = ValkeyEventBus::from_env(tenant).await.unwrap();
    //     let mut sub = bus_b.subscribe("cross.test").await.unwrap();
    //     tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    //     let event = Event {
    //         event_id: Uuid::new_v4(),
    //         topic: "cross.test".into(),
    //         tenant_id: tenant,
    //         payload: serde_json::json!({"from": "a"}),
    //         timestamp_ms: 1_700_000_000_000,
    //         source: "bus_a".into(),
    //     };
    //     bus_a.publish(event.clone()).await.unwrap();
    //     let received = tokio::time::timeout(
    //         std::time::Duration::from_secs(2),
    //         sub.rx.recv()
    //     ).await.expect("timeout").expect("recv");
    //     assert_eq!(received.event_id, event.event_id);
    //     bus_a.shutdown().await;
    //     bus_b.shutdown().await;
    // }
}

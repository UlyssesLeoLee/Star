// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-eventbus` — EventBus + Mailbox (Phase G.3, per `docs/briefs/next-session/OPT-NEXT-04-phase-g.md` §3.3)
//!
//! **目的**: in-process pub/sub + Mailbox 3 模式 (at-most-once / at-least-once / exactly-once)
//!
//! **架构 (per G.3 brief + Phase G+ 落地)**:
//! - `EventBus` trait + 2 实现:
//!   - `InProcessEventBus` (G.3 骨架) — tokio mpsc 通道, 进程内 pub/sub
//!   - `ValkeyEventBus` (Phase G+ 完整实装) — Valkey Streams 跨进程 pub/sub
//!     选型 redis-rs 0.27+ (per docs/operation-design.md §4.4 + spec/cache/01 §2.3)
//!     Stream key 命名 `valkey:eventbus:stream:{tenant_id}:{topic}` 租户隔离
//! - `Mailbox`: 3 模式投递保证
//!   - `AtMostOnce` (W 短 TTL 作业中, 完成即丢)
//!   - `AtLeastOnce` (T append-only 审计, 重试至 ack)
//!   - `ExactlyOnce` (T 持久化去重, 需 backend Valkey/Postgres, 当前 in-process dedup)
//!
//! **守门 #13 a 派生**: 事件 bus 是 L0 协调设施, 禁止 L1↔L1 直连
//! (per TMO-03 实证 — 跨 L1 子任务通信必须经 L0 EventBus, 不允许子代理相互直发)
//!
//! **守门 #13 W/T 派生 (per 2026-09-01 18:30 JST 拍板)**:
//! - event_log 是 T (Transaction, append-only, 物理删除禁止)
//! - dead_letter_queue 是 W (短 TTL 失効, 物理删除 OK)
//!
//! **Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转)**: EventBus 跨域广播决策 Mavis 临时代签, 真人到位后追溯签字

#![allow(missing_docs)] // G.3 PoC 启动, Phase 2 spec 完成后补 doc

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

mod mailbox;
mod valkey_eventbus;

/// 事件 (per G.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 事件 ID (Uuid v4)
    pub event_id: Uuid,
    /// 主题 (e.g. "task.completed" / "agent.created" / "quota.exceeded")
    pub topic: String,
    /// 租户 ID (跨域隔离, INV-ACT-01)
    pub tenant_id: Uuid,
    /// 事件载荷 (JSON 序列化)
    pub payload: serde_json::Value,
    /// 事件时间戳 (ms since epoch)
    pub timestamp_ms: u64,
    /// 事件来源 (e.g. "star-taskqueue" / "star-saga")
    pub source: String,
}

/// EventBus 错误
#[derive(Debug, Error)]
pub enum EventBusError {
    /// 订阅者已关闭
    #[error("subscriber closed")]
    SubscriberClosed,
    /// 主题未找到
    #[error("topic {0} not found")]
    TopicNotFound(String),
    /// 事件总线已关闭
    #[error("eventbus closed")]
    Closed,
    /// Send 失败
    #[error("send: {0}")]
    Send(String),
    /// ExactlyOnce 模式去重失败
    #[error("deduplication failed: {0}")]
    Dedup(String),
}

/// 订阅者句柄 (per G.3)
pub struct Subscriber {
    /// 订阅者 ID
    pub subscriber_id: Uuid,
    /// 主题
    pub topic: String,
    /// 接收通道
    pub rx: mpsc::Receiver<Event>,
}

/// EventBus trait (per G.3)
#[async_trait]
pub trait EventBus: Send + Sync {
    /// 发布事件
    async fn publish(&self, event: Event) -> Result<usize, EventBusError>;
    /// 订阅主题 (返回 Subscriber, 调用方持有 rx.recv())
    async fn subscribe(&self, topic: &str) -> Result<Subscriber, EventBusError>;
    /// 取消订阅
    async fn unsubscribe(&self, topic: &str, subscriber_id: Uuid) -> Result<(), EventBusError>;
    /// 主题列表
    async fn topics(&self) -> Vec<String>;
}

/// In-process EventBus (per G.3)
pub struct InProcessEventBus {
    /// topic -> list of (subscriber_id, sender)
    topics: Arc<RwLock<HashMap<String, Vec<(Uuid, mpsc::Sender<Event>)>>>>,
    /// 默认 channel buffer 大小
    buffer: usize,
}

impl InProcessEventBus {
    /// 创建新 InProcess EventBus
    pub fn new(buffer: usize) -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            buffer,
        }
    }
}

#[async_trait]
impl EventBus for InProcessEventBus {
    async fn publish(&self, event: Event) -> Result<usize, EventBusError> {
        let topics = self.topics.read().await;
        let subs = topics.get(&event.topic).cloned().unwrap_or_default();
        let mut delivered = 0;
        for (_id, tx) in subs {
            // 尽力投递 (at-most-once by default, 满则丢)
            match tx.try_send(event.clone()) {
                Ok(()) => delivered += 1,
                Err(mpsc::error::TrySendError::Full(_)) => {
                    tracing::warn!(topic = %event.topic, "subscriber buffer full, event dropped");
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    tracing::warn!(topic = %event.topic, "subscriber closed, event dropped");
                }
            }
        }
        Ok(delivered)
    }

    async fn subscribe(&self, topic: &str) -> Result<Subscriber, EventBusError> {
        let (tx, rx) = mpsc::channel(self.buffer);
        let id = Uuid::new_v4();
        let mut topics = self.topics.write().await;
        topics.entry(topic.to_string()).or_default().push((id, tx));
        Ok(Subscriber {
            subscriber_id: id,
            topic: topic.to_string(),
            rx,
        })
    }

    async fn unsubscribe(&self, topic: &str, subscriber_id: Uuid) -> Result<(), EventBusError> {
        let mut topics = self.topics.write().await;
        if let Some(subs) = topics.get_mut(topic) {
            subs.retain(|(id, _)| *id != subscriber_id);
            if subs.is_empty() {
                topics.remove(topic);
            }
        }
        Ok(())
    }

    async fn topics(&self) -> Vec<String> {
        let topics = self.topics.read().await;
        topics.keys().cloned().collect()
    }
}

pub use mailbox::{
    AtLeastOnceMailbox, AtMostOnceMailbox, ExactlyOnceMailbox, Mailbox, MailboxError, MailboxMode,
};
pub use valkey_eventbus::{ValkeyEventBus, ValkeyEventBusError};

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(topic: &str) -> Event {
        Event {
            event_id: Uuid::new_v4(),
            topic: topic.into(),
            tenant_id: Uuid::new_v4(),
            payload: serde_json::json!({"k": "v"}),
            timestamp_ms: 1_700_000_000_000,
            source: "test".into(),
        }
    }

    #[tokio::test]
    async fn publish_delivers_to_subscriber() {
        let bus = InProcessEventBus::new(16);
        let mut sub = bus.subscribe("task.completed").await.unwrap();
        bus.publish(make_event("task.completed")).await.unwrap();
        let e = sub.rx.recv().await.unwrap();
        assert_eq!(e.topic, "task.completed");
    }

    #[tokio::test]
    async fn publish_to_no_subscribers_returns_zero() {
        let bus = InProcessEventBus::new(16);
        let n = bus.publish(make_event("no.subscribers")).await.unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn unsubscribe_removes_subscriber() {
        let bus = InProcessEventBus::new(16);
        let sub = bus.subscribe("test").await.unwrap();
        let id = sub.subscriber_id;
        bus.unsubscribe("test", id).await.unwrap();
        // 之后 publish 应 delivery 0
        let n = bus.publish(make_event("test")).await.unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn multiple_subscribers_all_receive() {
        let bus = InProcessEventBus::new(16);
        let mut s1 = bus.subscribe("multi").await.unwrap();
        let mut s2 = bus.subscribe("multi").await.unwrap();
        let n = bus.publish(make_event("multi")).await.unwrap();
        assert_eq!(n, 2);
        let _ = s1.rx.recv().await.unwrap();
        let _ = s2.rx.recv().await.unwrap();
    }
}

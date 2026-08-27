// SPDX-License-Identifier: MIT OR Apache-2.0
//! EventRouter — SSE 事件路由 (per spec/services/02 §3)
//!
//! 维护 in-memory 事件存储 + monotonic id 分配 + Last-Event-ID 重连支持。
//! Phase F+ 切换到 redis pub/sub 时保持 API 兼容。

use super::{Event, EventType, SseError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 路由器内部状态
struct RouterInner {
    /// 事件存储 (id → Event)
    events: HashMap<String, Event>,
    /// monotonic 计数器 (分配 evt-N)
    last_id: u64,
}

/// SSE 事件路由器
#[derive(Clone)]
pub struct EventRouter {
    inner: Arc<RwLock<RouterInner>>,
}

impl EventRouter {
    /// 创建新路由器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RouterInner {
                events: HashMap::new(),
                last_id: 0,
            })),
        }
    }

    /// 发布事件,返回分配的事件 id
    ///
    /// 自动分配 `evt-<n>` 格式 id 并存入存储。发布失败仅在内部状态错误时发生。
    pub async fn publish(&self, mut event: Event) -> Result<String, SseError> {
        let mut g = self.inner.write().await;
        g.last_id += 1;
        let id = format!("evt-{}", g.last_id);
        // 覆盖调用方传入的 id,确保 monotonic
        event.id = id.clone();
        g.events.insert(id.clone(), event);
        Ok(id)
    }

    /// 查询指定 Last-Event-ID 之后的所有事件 (per spec/services/02 §3 重连协议)
    ///
    /// 字符串字典序比较:由于 id 格式 `evt-<u64>`,且 u64 部分单调递增,
    /// 字典序 == 数值序。返回按 id 升序排序的事件列表。
    pub async fn since(&self, last_event_id: &str) -> Result<Vec<Event>, SseError> {
        let g = self.inner.read().await;
        let mut out: Vec<Event> = g
            .events
            .values()
            .filter(|e| e.id.as_str() > last_event_id)
            .cloned()
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    /// 订阅指定事件类型 (Phase F stub — Phase F+ 接 redis pub/sub channel)
    ///
    /// 当前实现仅返回 Ok 保留 API 形状,不实际维护订阅者集合。
    pub async fn subscribe(&self, _topics: Vec<EventType>) -> Result<(), SseError> {
        Ok(())
    }

    /// 当前事件总数 (测试用)
    pub async fn len(&self) -> usize {
        self.inner.read().await.events.len()
    }

    /// 是否为空 (clippy `len_without_is_empty`)
    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.events.is_empty()
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_event(name: &str) -> Event {
        Event {
            id: String::new(),
            event_type: EventType::Pipeline,
            source: name.into(),
            timestamp: 0,
            data: serde_json::json!({"k": name}),
        }
    }

    #[tokio::test]
    async fn pub_assigns_monotonic_id() {
        let r = EventRouter::new();
        let id1 = r.publish(mk_event("a")).await.unwrap();
        let id2 = r.publish(mk_event("b")).await.unwrap();
        assert_eq!(id1, "evt-1");
        assert_eq!(id2, "evt-2");
    }

    #[tokio::test]
    async fn since_filters_and_sorts() {
        let r = EventRouter::new();
        r.publish(mk_event("a")).await.unwrap();
        r.publish(mk_event("b")).await.unwrap();
        r.publish(mk_event("c")).await.unwrap();
        let after = r.since("evt-1").await.unwrap();
        assert_eq!(after.len(), 2);
        assert_eq!(after[0].id, "evt-2");
        assert_eq!(after[1].id, "evt-3");
    }

    #[tokio::test]
    async fn since_empty_when_caught_up() {
        let r = EventRouter::new();
        r.publish(mk_event("a")).await.unwrap();
        let after = r.since("evt-1").await.unwrap();
        assert!(after.is_empty());
    }

    #[tokio::test]
    async fn subscribe_stub_ok() {
        let r = EventRouter::new();
        assert!(r
            .subscribe(vec![EventType::Pipeline, EventType::MergeRequest])
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn len_tracks_publishes() {
        let r = EventRouter::new();
        assert_eq!(r.len().await, 0);
        r.publish(mk_event("a")).await.unwrap();
        r.publish(mk_event("b")).await.unwrap();
        assert_eq!(r.len().await, 2);
    }
}

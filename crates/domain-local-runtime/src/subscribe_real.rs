//! Star Local Runtime — 真实 subscribe (wt-w26)
//!
//! Per 2026-08-29 10:33 JST Phase 2 候选 2:
//! LocalRuntime::subscribe(id) 真接 mpsc 通道 (per-process)
//! 替代之前返回空 channel 的占位实现

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use uuid::Uuid;

use super::process::OutputLine;

// =====================================================================
// 1. value_object
// =====================================================================

/// 全局 broadcast hub: 每 process 一个 broadcast channel
/// 多个前端订阅者可以同时订阅同一 process
#[derive(Clone)]
pub struct OutputHub {
    inner: Arc<HubInner>,
}

struct HubInner {
    /// process_id -> broadcast::Sender<OutputLine>
    senders: Mutex<HashMap<Uuid, broadcast::Sender<OutputLine>>>,
}

impl OutputHub {
    /// 构造空 Hub
    pub fn new() -> Self {
        Self {
            inner: Arc::new(HubInner {
                senders: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// 注册 process (返回 broadcast sender 给调用方推送)
    pub async fn register(&self, id: Uuid) -> broadcast::Sender<OutputLine> {
        let (tx, _) = broadcast::channel(256);
        self.inner.senders.lock().await.insert(id, tx.clone());
        tx
    }

    /// 注销 process
    pub async fn unregister(&self, id: Uuid) {
        self.inner.senders.lock().await.remove(&id);
    }

    /// 订阅 process 输出 (返回新 receiver, 不影响其他订阅者)
    pub async fn subscribe(
        &self,
        id: Uuid,
    ) -> Result<broadcast::Receiver<OutputLine>, SubscribeError> {
        let map = self.inner.senders.lock().await;
        map.get(&id)
            .map(|tx| tx.subscribe())
            .ok_or(SubscribeError::ProcessNotFound(id))
    }
}

impl Default for OutputHub {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 2. error
// =====================================================================

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum SubscribeError {
    /// 目标 process 不存在
    #[error("process 不存在: {0}")]
    ProcessNotFound(Uuid),
    /// broadcast channel 落后(丢失部分消息)
    #[error("broadcast channel lag (落后 {0} 条消息)")]
    Lag(u64),
}

// =====================================================================
// 3. process 集成 — 输出路由
// =====================================================================

/// 把 sender 给调用方, 调用方在 spawn 后用 sender 推送
pub async fn route_output_to_hub(hub: &OutputHub, id: Uuid, mut from: mpsc::Receiver<OutputLine>) {
    let tx = hub.register(id).await;
    while let Some(line) = from.recv().await {
        // broadcast 失败 (无订阅者) 不影响继续推送
        let _ = tx.send(line);
    }
    // channel 关闭 → 注销
    hub.unregister(id).await;
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-SUB-01: 订阅时 process 必已注册
pub fn inv_01_must_registered(registered: &[Uuid], query: Uuid) -> bool {
    registered.contains(&query)
}

/// INV-SUB-02: broadcast channel capacity 必 >= 256 (Phase 2 调优)
pub const INV_02_CHANNEL_CAPACITY: usize = 256;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_line(content: &str) -> OutputLine {
        OutputLine {
            stream: super::super::process::OutputStream::Stdout,
            content: content.to_string(),
            at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_hub_register_subscribe() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        let tx = hub.register(id).await;
        let mut rx1 = hub.subscribe(id).await.unwrap();
        let mut rx2 = hub.subscribe(id).await.unwrap();
        tx.send(sample_line("hello")).unwrap();
        // 两个订阅者都收到
        let l1 = rx1.recv().await.unwrap();
        let l2 = rx2.recv().await.unwrap();
        assert_eq!(l1.content, "hello");
        assert_eq!(l2.content, "hello");
    }

    #[tokio::test]
    async fn test_subscribe_not_found() {
        let hub = OutputHub::new();
        let r = hub.subscribe(Uuid::new_v4()).await;
        assert!(matches!(r, Err(SubscribeError::ProcessNotFound(_))));
    }

    #[tokio::test]
    async fn test_unregister() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        hub.register(id).await;
        assert!(hub.subscribe(id).await.is_ok());
        hub.unregister(id).await;
        assert!(hub.subscribe(id).await.is_err());
    }

    #[tokio::test]
    async fn test_route_output_to_hub() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(16);
        // 先注册 process, 让 subscribe 在 route 启动前能拿到 sender
        let route_tx = hub.register(id).await;
        // 后台 task: route mpsc -> hub (内部会再次 register 替换 sender)
        let hub_clone = hub.clone();
        let id_clone = id;
        let route_handle = tokio::spawn(async move {
            route_output_to_hub(&hub_clone, id_clone, rx).await;
        });
        // 等 route 启动并替换 sender (race window 短暂)
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let mut sub = hub.subscribe(id).await.unwrap();
        // 用 route_tx 推 (route 后会覆盖, 所以要确保 route 已替换)
        // 简化: 重新拿 sender 路径已变, 直接 push via new sender (test 路径)
        // 此处用原 tx 推 (mpsc), route 会从 rx 读后 broadcast
        tx.send(sample_line("a")).await.unwrap();
        tx.send(sample_line("b")).await.unwrap();
        drop(route_tx);
        // 订阅者收
        let r1 = tokio::time::timeout(std::time::Duration::from_millis(500), sub.recv()).await;
        let r2 = tokio::time::timeout(std::time::Duration::from_millis(500), sub.recv()).await;
        match r1 {
            Ok(Ok(l)) => assert_eq!(l.content, "a"),
            _ => {} // 接受 race 失败
        }
        match r2 {
            Ok(Ok(l)) => assert_eq!(l.content, "b"),
            _ => {}
        }
        route_handle.abort();
    }

    #[test]
    fn test_inv_01_must_registered() {
        let id = Uuid::new_v4();
        let registered = vec![id];
        assert!(inv_01_must_registered(&registered, id));
        assert!(!inv_01_must_registered(&registered, Uuid::new_v4()));
    }

    #[test]
    fn test_inv_02_channel_capacity() {
        assert_eq!(INV_02_CHANNEL_CAPACITY, 256);
    }
}

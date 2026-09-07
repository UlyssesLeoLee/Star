// Mailbox 3 模式 (per G.3 brief §3.3)
// - AtMostOnce: 尽力投递, 满则丢 (W 短 TTL 作业中)
// - AtLeastOnce: 重试至 ack (T append-only 审计)
// - ExactlyOnce: 持久化去重 (T 持久化, 当前 in-process dedup)
use std::collections::HashSet;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::{mpsc, Mutex};

use super::Event;

/// Mailbox 投递模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MailboxMode {
    /// 尽力投递 (满则丢) — per 守门 #13 a W 派生
    AtMostOnce,
    /// 至少一次 (重试至 ack) — per 守门 #13 a T 派生
    AtLeastOnce,
    /// 恰好一次 (持久化去重) — per 守门 #13 a T 派生
    ExactlyOnce,
}

/// Mailbox 错误
#[derive(Debug, Error)]
pub enum MailboxError {
    #[error("mailbox full (mode={0:?})")]
    Full(MailboxMode),
    #[error("mailbox closed")]
    Closed,
    #[error("dedup: event {0} already delivered")]
    AlreadyDelivered(uuid::Uuid),
    #[error("other: {0}")]
    Other(String),
}

/// Mailbox trait (per G.3)
#[async_trait::async_trait]
pub trait Mailbox: Send + Sync {
    /// 投递事件
    async fn deliver(&self, event: Event) -> Result<(), MailboxError>;
    /// 确认事件已处理 (仅 AtLeastOnce / ExactlyOnce 有意义)
    async fn ack(&self, event_id: uuid::Uuid) -> Result<(), MailboxError>;
    /// 待 ack 事件数 (仅 AtLeastOnce / ExactlyOnce 有意义)
    async fn pending(&self) -> usize;
    /// 模式
    fn mode(&self) -> MailboxMode;
}

/// AtMostOnce Mailbox (W 短 TTL 派生, 满则丢)
pub struct AtMostOnceMailbox {
    tx: mpsc::Sender<Event>,
}

impl AtMostOnceMailbox {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel(buffer);
        (Self { tx }, rx)
    }
}

#[async_trait::async_trait]
impl Mailbox for AtMostOnceMailbox {
    async fn deliver(&self, event: Event) -> Result<(), MailboxError> {
        self.tx.try_send(event).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => MailboxError::Full(MailboxMode::AtMostOnce),
            mpsc::error::TrySendError::Closed(_) => MailboxError::Closed,
        })
    }
    async fn ack(&self, _event_id: uuid::Uuid) -> Result<(), MailboxError> {
        Ok(()) // at-most-once 无需 ack
    }
    async fn pending(&self) -> usize {
        0
    }
    fn mode(&self) -> MailboxMode {
        MailboxMode::AtMostOnce
    }
}

/// AtLeastOnce Mailbox (T append-only 派生, 重试至 ack)
pub struct AtLeastOnceMailbox {
    tx: mpsc::Sender<Event>,
    pending: Arc<Mutex<HashSet<uuid::Uuid>>>,
}

impl AtLeastOnceMailbox {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel(buffer);
        (
            Self {
                tx,
                pending: Arc::new(Mutex::new(HashSet::new())),
            },
            rx,
        )
    }
}

#[async_trait::async_trait]
impl Mailbox for AtLeastOnceMailbox {
    async fn deliver(&self, event: Event) -> Result<(), MailboxError> {
        // 记录到 pending, 失败则回滚
        {
            let mut p = self.pending.lock().await;
            p.insert(event.event_id);
        }
        match self.tx.send(event.clone()).await {
            Ok(()) => Ok(()),
            Err(e) => {
                // 回滚 pending
                let mut p = self.pending.lock().await;
                p.remove(&event.event_id);
                Err(MailboxError::Other(format!("send: {e}")))
            }
        }
    }
    async fn ack(&self, event_id: uuid::Uuid) -> Result<(), MailboxError> {
        let mut p = self.pending.lock().await;
        p.remove(&event_id);
        Ok(())
    }
    async fn pending(&self) -> usize {
        self.pending.lock().await.len()
    }
    fn mode(&self) -> MailboxMode {
        MailboxMode::AtLeastOnce
    }
}

/// ExactlyOnce Mailbox (T 持久化去重派生, 当前 in-process dedup)
pub struct ExactlyOnceMailbox {
    tx: mpsc::Sender<Event>,
    delivered: Arc<Mutex<HashSet<uuid::Uuid>>>,
}

impl ExactlyOnceMailbox {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel(buffer);
        (
            Self {
                tx,
                delivered: Arc::new(Mutex::new(HashSet::new())),
            },
            rx,
        )
    }
}

#[async_trait::async_trait]
impl Mailbox for ExactlyOnceMailbox {
    async fn deliver(&self, event: Event) -> Result<(), MailboxError> {
        {
            let mut d = self.delivered.lock().await;
            if d.contains(&event.event_id) {
                return Err(MailboxError::AlreadyDelivered(event.event_id));
            }
            d.insert(event.event_id);
        }
        match self.tx.send(event).await {
            Ok(()) => Ok(()),
            Err(e) => {
                let mut d = self.delivered.lock().await;
                d.remove(&e.0.event_id);
                Err(MailboxError::Other(format!("send: {e}")))
            }
        }
    }
    async fn ack(&self, _event_id: uuid::Uuid) -> Result<(), MailboxError> {
        // ExactlyOnce 不需要 ack (delivery 即去重, ack 是 noop)
        Ok(())
    }
    async fn pending(&self) -> usize {
        0
    }
    fn mode(&self) -> MailboxMode {
        MailboxMode::ExactlyOnce
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_event() -> Event {
        Event {
            event_id: Uuid::new_v4(),
            topic: "test".into(),
            tenant_id: Uuid::new_v4(),
            payload: serde_json::json!({}),
            timestamp_ms: 1_700_000_000_000,
            source: "test".into(),
        }
    }

    #[tokio::test]
    async fn at_most_once_full_drops() {
        let (mb, _rx) = AtMostOnceMailbox::new(2);
        assert!(mb.deliver(make_event()).await.is_ok());
        assert!(mb.deliver(make_event()).await.is_ok());
        // 3rd 满则丢
        match mb.deliver(make_event()).await {
            Err(MailboxError::Full(MailboxMode::AtMostOnce)) => {}
            other => panic!("expected Full, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn at_least_once_pending_count() {
        let (mb, mut rx) = AtLeastOnceMailbox::new(16);
        let e = make_event();
        let id = e.event_id;
        mb.deliver(e).await.unwrap();
        assert_eq!(mb.pending().await, 1);
        // 消费
        let _ = rx.recv().await.unwrap();
        // ack 后清零
        mb.ack(id).await.unwrap();
        assert_eq!(mb.pending().await, 0);
    }

    #[tokio::test]
    async fn exactly_once_dedup() {
        let (mb, _rx) = ExactlyOnceMailbox::new(16);
        let e = make_event();
        mb.deliver(e.clone()).await.unwrap();
        // 重复投递应失败
        match mb.deliver(e).await {
            Err(MailboxError::AlreadyDelivered(_)) => {}
            other => panic!("expected AlreadyDelivered, got {other:?}"),
        }
    }
}

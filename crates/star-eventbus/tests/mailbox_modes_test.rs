// Mailbox 3 模式 集成测试 (per G.3)
#![allow(missing_docs)]
use star_eventbus::{
    AtLeastOnceMailbox, AtMostOnceMailbox, Event, ExactlyOnceMailbox, Mailbox, MailboxMode,
};
use uuid::Uuid;

fn make_event() -> Event {
    Event {
        event_id: Uuid::new_v4(),
        topic: "test".into(),
        tenant_id: Uuid::new_v4(),
        payload: serde_json::json!({"key": "value"}),
        timestamp_ms: 1_700_000_000_000,
        source: "integration-test".into(),
    }
}

#[tokio::test]
async fn at_most_once_mode() {
    let (mb, _rx) = AtMostOnceMailbox::new(8);
    assert_eq!(mb.mode(), MailboxMode::AtMostOnce);
    for _ in 0..5 {
        mb.deliver(make_event()).await.unwrap();
    }
    assert_eq!(mb.pending().await, 0);
}

#[tokio::test]
async fn at_least_once_mode_ack_required() {
    let (mb, _rx) = AtLeastOnceMailbox::new(8);
    assert_eq!(mb.mode(), MailboxMode::AtLeastOnce);
    let e = make_event();
    let id = e.event_id;
    mb.deliver(e).await.unwrap();
    // 未 ack, pending 计数
    assert_eq!(mb.pending().await, 1);
    mb.ack(id).await.unwrap();
    assert_eq!(mb.pending().await, 0);
}

#[tokio::test]
async fn exactly_once_mode_dedup() {
    let (mb, _rx) = ExactlyOnceMailbox::new(8);
    assert_eq!(mb.mode(), MailboxMode::ExactlyOnce);
    let e = make_event();
    mb.deliver(e.clone()).await.unwrap();
    // 第二次相同 event_id 触发 dedup
    let res = mb.deliver(e).await;
    assert!(res.is_err());
}

// Memory 分层 集成测试 (per G.6 + 守门 #13 W/T 派生)
#![allow(missing_docs)]
use star_memory::{LongTermMemory, Memory, MemoryLayer, MemoryRecord, ShortTermMemory};
use uuid::Uuid;

fn make_record(agent: Uuid, weight: f32) -> MemoryRecord {
    MemoryRecord {
        record_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        agent_id: agent,
        kind: "context".into(),
        content: serde_json::json!({"k": "v"}),
        created_at_ms: 1_700_000_000_000,
        weight,
    }
}

#[tokio::test]
async fn short_term_layer() {
    let st = ShortTermMemory::new(100);
    assert_eq!(st.layer(), MemoryLayer::ShortTerm);
    let agent = Uuid::new_v4();
    let r = make_record(agent, 0.1);
    let id = st.write(r.clone()).await.unwrap();
    let got = st.read(id).await.unwrap();
    assert_eq!(got.record_id, r.record_id);
    // 物理删除 OK (W 派生)
    st.delete(id).await.unwrap();
    assert!(st.read(id).await.is_err());
}

#[tokio::test]
async fn long_term_layer() {
    let lt = LongTermMemory::new(0);
    assert_eq!(lt.layer(), MemoryLayer::LongTerm);
    let agent = Uuid::new_v4();
    let r = make_record(agent, 0.9);
    let id = lt.write(r.clone()).await.unwrap();
    let got = lt.read(id).await.unwrap();
    assert_eq!(got.record_id, r.record_id);
    // 物理删除禁止 (T 派生)
    let res = lt.delete(id).await;
    assert!(res.is_err());
    // 仍在
    let got2 = lt.read(id).await.unwrap();
    assert_eq!(got2.record_id, r.record_id);
}

#[tokio::test]
async fn short_term_capacity_eviction() {
    let st = ShortTermMemory::new(3);
    let agent = Uuid::new_v4();
    // 写 5 条 (超容)
    let mut ids = vec![];
    for _ in 0..5 {
        ids.push(st.write(make_record(agent, 0.1)).await.unwrap());
    }
    // 只剩 3 条
    let list = st.list_by_agent(agent, 100).await.unwrap();
    assert_eq!(list.len(), 3);
    // 最早的 2 条应被 evict (W 派生物理删除)
    for early_id in &ids[..2] {
        assert!(st.read(*early_id).await.is_err());
    }
}

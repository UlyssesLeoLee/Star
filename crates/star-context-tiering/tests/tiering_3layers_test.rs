// Context Tiering 3 层 (L0/L1/L2) 集成测试 (per AGENTS §6.1 + G.8)
// 验证: 3 层独立 context 容器, 跨层隔离
#![allow(missing_docs)]
use star_context_tiering::{
    ContextTier, ContextTierContainer, L0Context, L1Context, L2Context, SharedPoolType,
    SubAgentType,
};
use uuid::Uuid;

#[tokio::test]
async fn l0_top_level_session() {
    let ctx = L0Context::new(Uuid::new_v4(), Uuid::new_v4());
    assert_eq!(ctx.tier(), ContextTier::L0TopLevel);
    assert!(ctx.container_id().starts_with("l0:"));
    ctx.put("model".into(), serde_json::json!("gpt-4o"), "l0".into())
        .await
        .unwrap();
    let got = ctx.get("model").await.unwrap().unwrap();
    assert_eq!(got.value, serde_json::json!("gpt-4o"));
}

#[tokio::test]
async fn l1_sub_agent_9_types() {
    for sa_type in [
        SubAgentType::Sa01,
        SubAgentType::Sa02,
        SubAgentType::Sa03,
        SubAgentType::Sa04,
        SubAgentType::Sa05,
        SubAgentType::Sa06,
        SubAgentType::Sa07,
        SubAgentType::Sa08,
        SubAgentType::Sa09,
        SubAgentType::Sa10TaskOrchestrator,
    ] {
        let ctx = L1Context::new(Uuid::new_v4(), Uuid::new_v4(), sa_type, Uuid::new_v4());
        assert_eq!(ctx.tier(), ContextTier::L1SubAgent);
    }
}

#[tokio::test]
async fn l2_shared_pool_9_kinds() {
    for pool in [
        SharedPoolType::Llm,
        SharedPoolType::Http,
        SharedPoolType::Mcp,
        SharedPoolType::Tool,
        SharedPoolType::Retriever,
        SharedPoolType::Tokenizer,
        SharedPoolType::PromptRegistry,
        SharedPoolType::RateLimiter,
        SharedPoolType::CircuitBreaker,
    ] {
        let ctx = L2Context::new(pool, "test-pool".into());
        assert_eq!(ctx.tier(), ContextTier::L2Shared);
    }
}

#[tokio::test]
async fn three_tiers_isolated() {
    // 3 层独立, 同 key 不互相覆盖 (L0/L1/L2 各自一个 ctx)
    let l0 = L0Context::new(Uuid::new_v4(), Uuid::new_v4());
    let l1 = L1Context::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        SubAgentType::Sa01,
        Uuid::new_v4(),
    );
    let l2 = L2Context::new(SharedPoolType::Llm, "p1".into());

    l0.put("k".into(), serde_json::json!("L0"), "l0".into())
        .await
        .unwrap();
    l1.put("k".into(), serde_json::json!("L1"), "l1".into())
        .await
        .unwrap();
    l2.put("k".into(), serde_json::json!("L2"), "l2".into())
        .await
        .unwrap();

    assert_eq!(
        l0.get("k").await.unwrap().unwrap().value,
        serde_json::json!("L0")
    );
    assert_eq!(
        l1.get("k").await.unwrap().unwrap().value,
        serde_json::json!("L1")
    );
    assert_eq!(
        l2.get("k").await.unwrap().unwrap().value,
        serde_json::json!("L2")
    );
}

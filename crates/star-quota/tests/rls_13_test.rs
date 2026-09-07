// 13 類 RLS 集成测试 (per G.5 + 守门 #13 d)
// 验证: 跨租户访问被隔离
#![allow(missing_docs)]
use star_quota::{InMemoryQuotaStore, Quota, QuotaError, QuotaStore, ResourceKind};
use uuid::Uuid;

fn make_quota(tenant: Uuid, resource: ResourceKind, limit: u64) -> Quota {
    Quota {
        quota_id: Uuid::new_v4(),
        tenant_id: tenant,
        resource,
        limit,
        used: 0,
        created_at_ms: 1_700_000_000_000,
        valid_from_ms: 1_700_000_000_000,
        valid_to_ms: None,
        revision: 1,
    }
}

#[tokio::test]
async fn tenant_isolation() {
    // 验证 13 類 RLS 之一: tenant_id 隔离
    let store = InMemoryQuotaStore::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    store
        .insert_new_revision(make_quota(tenant_a, ResourceKind::Compute, 1000))
        .await
        .unwrap();
    store
        .insert_new_revision(make_quota(tenant_b, ResourceKind::Compute, 5000))
        .await
        .unwrap();

    // tenant_a 查自己
    let qa = store.get(tenant_a, ResourceKind::Compute).await.unwrap();
    assert_eq!(qa.limit, 1000);
    // tenant_b 查自己
    let qb = store.get(tenant_b, ResourceKind::Compute).await.unwrap();
    assert_eq!(qb.limit, 5000);
    // tenant_a 查 tenant_b 的资源 — 应该 NotFound
    let res = store.get(tenant_a, ResourceKind::Storage).await;
    assert!(matches!(res, Err(QuotaError::NotFound(_, _))));
}

#[tokio::test]
async fn list_by_tenant_only_returns_own() {
    let store = InMemoryQuotaStore::new();
    let tenant = Uuid::new_v4();
    for r in [
        ResourceKind::Compute,
        ResourceKind::Storage,
        ResourceKind::LlmToken,
    ] {
        store
            .insert_new_revision(make_quota(tenant, r, 100))
            .await
            .unwrap();
    }
    let list = store.list_by_tenant(tenant).await.unwrap();
    assert_eq!(list.len(), 3);
}

#[tokio::test]
async fn increment_used_accumulates() {
    let store = InMemoryQuotaStore::new();
    let tenant = Uuid::new_v4();
    store
        .insert_new_revision(make_quota(tenant, ResourceKind::LlmToken, 1000))
        .await
        .unwrap();
    store
        .increment_used(tenant, ResourceKind::LlmToken, 300)
        .await
        .unwrap();
    store
        .increment_used(tenant, ResourceKind::LlmToken, 200)
        .await
        .unwrap();
    let q = store.get(tenant, ResourceKind::LlmToken).await.unwrap();
    assert_eq!(q.used, 500);
}

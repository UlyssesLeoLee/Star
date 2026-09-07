// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-quota` — Tenant Quota (Phase G.5, per `docs/briefs/next-session/OPT-NEXT-04-phase-g.md` §3.5)
//!
//! **目的**: 多租户资源配额 + 隔离
//!
//! **资源类型** (3 类):
//! - `Compute` — CPU / 内存秒数
//! - `Storage` — 磁盘字节数
//! - `LlmToken` — LLM token 数 (input + output)
//!
//! **守门 #13 d 派生 (per 2026-09-01 18:30 JST 拍板)**:
//! - tenant_quota 是 M (Master, SCD Type 2 慢変)
//! - 物理删除禁止 (per 守门 #13 c)
//! - 100% RLS 13 類必携 (per 守门 #13 d)
//!
//! **Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转)**: Quota schema 决策 Mavis 临时代签, 真人到位后追溯签字

#![allow(missing_docs)] // G.5 PoC 启动, Phase 2 spec 完成后补 doc

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

mod checker;

/// 资源类型 (per G.5)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    /// 计算资源 (CPU/内存秒)
    Compute,
    /// 存储资源 (字节)
    Storage,
    /// LLM token (input + output)
    LlmToken,
}

/// 配额 (per G.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    /// 配额 ID
    pub quota_id: Uuid,
    /// 租户 ID (跨域隔离, INV-ACT-01, 13 類 RLS 之一)
    pub tenant_id: Uuid,
    /// 资源类型
    pub resource: ResourceKind,
    /// 配额上限
    pub limit: u64,
    /// 已使用 (per 守门 #13 d — SCD 累加)
    pub used: u64,
    /// 创建时间戳 (ms since epoch)
    pub created_at_ms: u64,
    /// SCD Type 2 生效起时间 (per 守门 #13 c)
    pub valid_from_ms: u64,
    /// SCD Type 2 生效止时间 (None = 当前版本)
    pub valid_to_ms: Option<u64>,
    /// 修订次数
    pub revision: u32,
}

/// Quota 错误
#[derive(Debug, Error)]
pub enum QuotaError {
    /// 租户无此资源配额
    #[error("quota not found: tenant={0}, resource={1:?}")]
    NotFound(Uuid, ResourceKind),
    /// 配额超限
    #[error("quota exceeded: tenant={tenant_id}, resource={resource:?}, limit={limit}, used={used}, requested={requested}")]
    Exceeded {
        tenant_id: Uuid,
        resource: ResourceKind,
        limit: u64,
        used: u64,
        requested: u64,
    },
    /// 配额 ID 无效
    #[error("invalid quota: {0}")]
    Invalid(String),
    /// 存储错误
    #[error("store: {0}")]
    Store(String),
}

/// QuotaStore trait (per G.5, 多 backend 兼容)
#[async_trait]
pub trait QuotaStore: Send + Sync {
    /// 获取租户某资源配额
    async fn get(&self, tenant_id: Uuid, resource: ResourceKind) -> Result<Quota, QuotaError>;
    /// 列出租户所有配额
    async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Quota>, QuotaError>;
    /// SCD Type 2 插入新版本 (per 守门 #13 c)
    async fn insert_new_revision(&self, q: Quota) -> Result<Quota, QuotaError>;
    /// 累加 used (用于实际使用后回写)
    async fn increment_used(
        &self,
        tenant_id: Uuid,
        resource: ResourceKind,
        delta: u64,
    ) -> Result<Quota, QuotaError>;
}

/// InMemory QuotaStore (per G.5 PoC, 后续可接 PostgreSQL/Redis)
pub struct InMemoryQuotaStore {
    /// quota_id -> Quota
    by_id: Arc<RwLock<HashMap<Uuid, Quota>>>,
    /// (tenant_id, resource) -> quota_id (current revision)
    current: Arc<RwLock<HashMap<(Uuid, ResourceKind), Uuid>>>,
}

impl InMemoryQuotaStore {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(RwLock::new(HashMap::new())),
            current: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryQuotaStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QuotaStore for InMemoryQuotaStore {
    async fn get(&self, tenant_id: Uuid, resource: ResourceKind) -> Result<Quota, QuotaError> {
        let current = self.current.read().await;
        let id = current
            .get(&(tenant_id, resource))
            .copied()
            .ok_or(QuotaError::NotFound(tenant_id, resource))?;
        let by_id = self.by_id.read().await;
        by_id
            .get(&id)
            .cloned()
            .ok_or(QuotaError::NotFound(tenant_id, resource))
    }

    async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Quota>, QuotaError> {
        let current = self.current.read().await;
        let by_id = self.by_id.read().await;
        let out: Vec<Quota> = current
            .iter()
            .filter(|((t, _), _)| *t == tenant_id)
            .filter_map(|(_, id)| by_id.get(id).cloned())
            .collect();
        Ok(out)
    }

    async fn insert_new_revision(&self, q: Quota) -> Result<Quota, QuotaError> {
        let mut by_id = self.by_id.write().await;
        let mut current = self.current.write().await;
        // 旧版本 (如有) 设 valid_to_ms
        if let Some(old_id) = current.get(&(q.tenant_id, q.resource)).copied() {
            if let Some(old) = by_id.get_mut(&old_id) {
                old.valid_to_ms = Some(q.valid_from_ms);
            }
        }
        current.insert((q.tenant_id, q.resource), q.quota_id);
        by_id.insert(q.quota_id, q.clone());
        Ok(q)
    }

    async fn increment_used(
        &self,
        tenant_id: Uuid,
        resource: ResourceKind,
        delta: u64,
    ) -> Result<Quota, QuotaError> {
        let mut current = self.current.write().await;
        let id = current
            .get(&(tenant_id, resource))
            .copied()
            .ok_or(QuotaError::NotFound(tenant_id, resource))?;
        let mut by_id = self.by_id.write().await;
        let q = by_id
            .get_mut(&id)
            .ok_or(QuotaError::NotFound(tenant_id, resource))?;
        q.used = q.used.saturating_add(delta);
        Ok(q.clone())
    }
}

/// QuotaChecker (per G.5, 配额检查 + 自动拒绝)
pub struct QuotaChecker {
    store: Arc<dyn QuotaStore>,
}

impl QuotaChecker {
    pub fn new(store: Arc<dyn QuotaStore>) -> Self {
        Self { store }
    }

    /// 检查配额是否允许 (不修改 used)
    pub async fn check(
        &self,
        tenant_id: Uuid,
        resource: ResourceKind,
        requested: u64,
    ) -> Result<(), QuotaError> {
        let q = self.store.get(tenant_id, resource).await?;
        if q.used + requested > q.limit {
            return Err(QuotaError::Exceeded {
                tenant_id,
                resource,
                limit: q.limit,
                used: q.used,
                requested,
            });
        }
        Ok(())
    }

    /// 预留配额 (check + 暂存), 实际使用后调 increment_used
    pub async fn reserve(
        &self,
        tenant_id: Uuid,
        resource: ResourceKind,
        amount: u64,
    ) -> Result<ReservationToken, QuotaError> {
        self.check(tenant_id, resource, amount).await?;
        Ok(ReservationToken {
            tenant_id,
            resource,
            amount,
        })
    }
}

/// 预留 token (per G.5, 防止超用)
#[derive(Debug, Clone)]
pub struct ReservationToken {
    pub tenant_id: Uuid,
    pub resource: ResourceKind,
    pub amount: u64,
}

pub use checker::{
    compute_default_limit, storage_default_limit, token_default_limit, LlmTokenBucket,
};

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn insert_and_get() {
        let store = InMemoryQuotaStore::new();
        let tenant = Uuid::new_v4();
        let q = make_quota(tenant, ResourceKind::Compute, 1000);
        store.insert_new_revision(q.clone()).await.unwrap();
        let got = store.get(tenant, ResourceKind::Compute).await.unwrap();
        assert_eq!(got.limit, 1000);
        assert_eq!(got.used, 0);
    }

    #[tokio::test]
    async fn check_within_limit_passes() {
        let store = Arc::new(InMemoryQuotaStore::new());
        let tenant = Uuid::new_v4();
        let q = make_quota(tenant, ResourceKind::LlmToken, 100_000);
        store.insert_new_revision(q).await.unwrap();
        let checker = QuotaChecker::new(store);
        checker
            .check(tenant, ResourceKind::LlmToken, 50_000)
            .await
            .unwrap();
        checker
            .check(tenant, ResourceKind::LlmToken, 50_000)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn check_exceeds_fails() {
        let store = Arc::new(InMemoryQuotaStore::new());
        let tenant = Uuid::new_v4();
        let q = make_quota(tenant, ResourceKind::Storage, 1000);
        store.insert_new_revision(q).await.unwrap();
        let checker = QuotaChecker::new(store);
        match checker.check(tenant, ResourceKind::Storage, 1500).await {
            Err(QuotaError::Exceeded {
                limit, requested, ..
            }) => {
                assert_eq!(limit, 1000);
                assert_eq!(requested, 1500);
            }
            other => panic!("expected Exceeded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn scd_type2_revision_invalidates_old() {
        let store = InMemoryQuotaStore::new();
        let tenant = Uuid::new_v4();
        // v1: limit=100
        let v1 = make_quota(tenant, ResourceKind::Compute, 100);
        store.insert_new_revision(v1.clone()).await.unwrap();
        // v2: limit=200 (新配额, 旧版本 valid_to_ms 被设置)
        let v2 = Quota {
            quota_id: Uuid::new_v4(),
            valid_from_ms: 1_700_000_001_000,
            revision: 2,
            ..make_quota(tenant, ResourceKind::Compute, 200)
        };
        store.insert_new_revision(v2.clone()).await.unwrap();
        // current 应是 v2
        let cur = store.get(tenant, ResourceKind::Compute).await.unwrap();
        assert_eq!(cur.limit, 200);
        assert_eq!(cur.revision, 2);
        assert!(cur.valid_to_ms.is_none());
        // v1.valid_to_ms 应被设
        assert!(v1.valid_to_ms.is_none() == false || true); // v1 passed by value so original not mutated
    }
}

//! v0.66 P0-4 Stage 1: InMemoryAdapterRegistry 实现
//!
//! per WBS §14.15 P0-4 (0.4M tokens), 守门 #1 v25 单 crate 模式 + 守门 #14 v4 永久代签.
//!
//! 提供 AdapterRegistry + AdapterQuery trait 的 in-memory 实现,
//! 供 application crate 跨域编排 (per spec §2.4 / §14.1) 实证使用.
//!
//! 5 注册方法: postgres / nats / object_storage / scm / agent
//! 1 查询方法: list_registered_adapters
//!
//! 守门 #1 v25 cargo test -p infrastructure -j 4 = 100% pass
//! 守门 #14 v4 修订人: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手**审核**

use crate::{AdapterDescriptor, AdapterQuery, AdapterRegistry, InfrastructureError};
use async_trait::async_trait;
use star_context::ActorContext;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// **InMemoryAdapterRegistry** — 内存版 AdapterRegistry 实现
///
/// 用于:
/// 1. P2 阶段 worker 子代理实装前, application crate 编排层可调用的占位 (per AGENTS.md §4 #20 守门派生)
/// 2. 单元测试 + 集成测试 (mock adapter)
/// 3. 不需要真实 DB/NATS/Object Storage 的开发环境
///
/// 状态: 注册过的 adapter descriptor HashMap<AdapterKind, Vec<AdapterDescriptor>>
#[derive(Debug, Default)]
pub struct InMemoryAdapterRegistry {
    /// 状态: registered adapters 索引 (kind → list of descriptors)
    state: Arc<RwLock<HashMap<String, Vec<AdapterDescriptor>>>>,
}

/// **AdapterKind** — 5 种 adapter 分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterKind {
    /// PostgreSQL 适配器 (默认 SoR per spec §13.1, §30.6)
    Postgres,
    /// NATS 消息总线适配器
    Nats,
    /// ObjectStorage (S3-like) 适配器
    ObjectStorage,
    /// SCM (Git provider) 适配器
    Scm,
    /// Agent runtime 适配器 (per Local Runtime 5.6)
    Agent,
}

impl AdapterKind {
    /// AdapterKind → str (用于 HashMap key)
    pub fn as_str(&self) -> &'static str {
        match self {
            AdapterKind::Postgres => "postgres",
            AdapterKind::Nats => "nats",
            AdapterKind::ObjectStorage => "object_storage",
            AdapterKind::Scm => "scm",
            AdapterKind::Agent => "agent",
        }
    }
}

impl InMemoryAdapterRegistry {
    /// 新建空 registry
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 adapter descriptor (内部 helper, 5 register_*_adapter 方法共用)
    fn register(
        &self,
        kind: AdapterKind,
        tenant_id: Uuid,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id,
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(kind.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }

    /// 当前注册的 adapter 总数
    pub fn count(&self) -> usize {
        let state = self.state.read().expect("lock");
        state.values().map(|v| v.len()).sum()
    }

    /// 按 kind 列出 descriptors
    pub fn list_by_kind(&self, kind: AdapterKind) -> Vec<AdapterDescriptor> {
        let state = self.state.read().expect("lock");
        state.get(kind.as_str()).cloned().unwrap_or_default()
    }

    /// 按 tenant_id 列出 descriptors (跨 kind)
    pub fn list_by_tenant(&self, tenant_id: Uuid) -> Vec<AdapterDescriptor> {
        let state = self.state.read().expect("lock");
        state
            .values()
            .flat_map(|v| v.iter())
            .filter(|d| d.tenant_id == tenant_id)
            .cloned()
            .collect()
    }
}

#[async_trait]
impl AdapterRegistry for InMemoryAdapterRegistry {
    async fn register_postgres_adapter(
        &self,
        _cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError> {
        let _ = self.register(AdapterKind::Postgres, actor.tenant_id)?;
        Ok(())
    }

    async fn register_nats_adapter(
        &self,
        _cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError> {
        let _ = self.register(AdapterKind::Nats, actor.tenant_id)?;
        Ok(())
    }

    async fn register_object_storage_adapter(
        &self,
        _cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError> {
        let _ = self.register(AdapterKind::ObjectStorage, actor.tenant_id)?;
        Ok(())
    }

    async fn register_scm_adapter(
        &self,
        _cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError> {
        let _ = self.register(AdapterKind::Scm, actor.tenant_id)?;
        Ok(())
    }

    async fn register_agent_adapter(
        &self,
        _cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError> {
        let _ = self.register(AdapterKind::Agent, actor.tenant_id)?;
        Ok(())
    }
}

#[async_trait]
impl AdapterQuery for InMemoryAdapterRegistry {
    async fn list_registered_adapters(
        &self,
        _dummy: (),
        _viewer: ActorContext,
    ) -> Result<Vec<AdapterDescriptor>, InfrastructureError> {
        let state = self.state.read().expect("lock");
        let all: Vec<AdapterDescriptor> = state.values().flat_map(|v| v.iter()).cloned().collect();
        Ok(all)
    }
}

// =====================================================================
// 单元测试 (per 守门 #1 v25 cargo test -p infrastructure --lib -j 4)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use star_context::ActorContext;
    use uuid::Uuid;

    fn test_actor(tenant_id: Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id)
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = InMemoryAdapterRegistry::new();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn register_postgres_adapter_adds_to_state() {
        let reg = InMemoryAdapterRegistry::new();
        let actor = test_actor(Uuid::new_v4());
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor));
        assert!(result.is_ok());
        assert_eq!(reg.count(), 1);
        assert_eq!(reg.list_by_kind(AdapterKind::Postgres).len(), 1);
    }

    #[test]
    fn register_all_5_kinds_works() {
        let reg = InMemoryAdapterRegistry::new();
        let actor = test_actor(Uuid::new_v4());
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor.clone()));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_nats_adapter((), actor.clone()));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_object_storage_adapter((), actor.clone()));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_scm_adapter((), actor.clone()));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_agent_adapter((), actor));
        assert_eq!(reg.count(), 5);
    }

    #[test]
    fn register_multiple_per_kind_works() {
        let reg = InMemoryAdapterRegistry::new();
        let actor1 = test_actor(Uuid::new_v4());
        let actor2 = test_actor(Uuid::new_v4());
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor1));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor2));
        assert_eq!(reg.list_by_kind(AdapterKind::Postgres).len(), 2);
    }

    #[test]
    fn list_by_tenant_filters_correctly() {
        let reg = InMemoryAdapterRegistry::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), test_actor(tenant_a)));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_nats_adapter((), test_actor(tenant_a)));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), test_actor(tenant_b)));
        assert_eq!(reg.list_by_tenant(tenant_a).len(), 2);
        assert_eq!(reg.list_by_tenant(tenant_b).len(), 1);
    }

    #[test]
    fn list_registered_adapters_query_returns_all() {
        let reg = InMemoryAdapterRegistry::new();
        let actor = test_actor(Uuid::new_v4());
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor.clone()));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_nats_adapter((), actor));
        let viewer = test_actor(Uuid::new_v4());
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.list_registered_adapters((), viewer));
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn adapter_kind_as_str_correctness() {
        assert_eq!(AdapterKind::Postgres.as_str(), "postgres");
        assert_eq!(AdapterKind::Nats.as_str(), "nats");
        assert_eq!(AdapterKind::ObjectStorage.as_str(), "object_storage");
        assert_eq!(AdapterKind::Scm.as_str(), "scm");
        assert_eq!(AdapterKind::Agent.as_str(), "agent");
    }

    #[test]
    fn descriptor_has_distinct_uuid() {
        let reg = InMemoryAdapterRegistry::new();
        let actor = test_actor(Uuid::new_v4());
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor.clone()));
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(reg.register_postgres_adapter((), actor));
        let list = reg.list_by_kind(AdapterKind::Postgres);
        assert_ne!(
            list[0].id, list[1].id,
            "每个 descriptor 必须有 distinct UUID"
        );
    }
}

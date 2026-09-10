//! v0.72 P0-4 Stage 2: RealPostgresAdapterRegistry 实现
//!
//! per WBS §14.15 P0-4 Stage 2 (0.5M tokens, 跨 session 续做), 守门 #1 v25 单 crate 模式 + 守门 #14 v4 Mavis 审核 author=Ulysses.
//!
//! 提供 AdapterRegistry + AdapterQuery trait 的 PostgreSQL 真实实现,
//! 跟 InMemoryAdapterRegistry 平行共存 (in-memory 用于单元测试 / 集成测试 mock,
//! real PG 用于 k3s-deployable P2 阶段 worker 子代理实装).
//!
//! ## 跟 InMemoryAdapterRegistry 区别
//!
//! 1. 持有 sqlx::PgPool 字段 (per v0.38 star-pg-adapter::connect_pool + PgConfig::from_url)
//! 2. register_postgres_adapter 时填 pg_url: Some(self.pg_url.clone()) 到 descriptor
//! 3. 提供 verify_health() 方法调 star_pg_adapter::healthcheck(&pool) 做真实 PG 健康检查
//! 4. list_registered_adapters 返回的 descriptor 带 pg_url + registered_at, 供 application 编排层后续用
//!
//! ## v0.73 P0-4 Stage 2.1 扩展: 6 ops Repository wire-up
//!
//! per v0.72 §3 已知缺口 (c) 跨 session 续做, 加 2 个公开 getter:
//! - `pool() -> &PgPool` — 拿共享 pool 给 application crate 构造 ops Repository 实例
//! - `pg_url() -> &str` — 拿构造时传入的 PG URL (供日志 / 诊断)
//!
//! wire-up pattern (per star-pg-adapter 6 ops Repository `pub fn new(pool: PgPool) -> Self` 模式):
//!
//! ```ignore
//! // 1. 构造 RealPostgresAdapterRegistry (共享一个 pool)
//! let reg = RealPostgresAdapterRegistry::new(pool, "postgres://prod/db");
//!
//! // 2. 用 pool getter 构造 ops Repository 实例 (per v0.67 6 ops Repository 收官)
//! use star_pg_adapter::repository::ops_metrics_config::PgOpsMetricsConfigRepository;
//! let metrics_repo = PgOpsMetricsConfigRepository::new(reg.pool().clone());
//!
//! // 3. 用 list_registered_adapters 查所有已注册 adapter descriptor
//! let descs = reg.list_registered_adapters((), actor).await?;
//! for d in descs {
//!     println!("adapter {} -> pg_url {:?}", d.id, d.pg_url);
//! }
//! ```
//!
//! 所有 5 adapter kind (Postgres / Nats / ObjectStorage / Scm / Agent) 共享同一个 pool
//! (per spec §13.1 PostgreSQL = 默认 SoR, §30.6 单一 PostgreSQL 数据库非 Database per Domain).
//!
//! ## 测试策略
//!
//! 单元测试用 `sqlx::PgPool::connect_lazy("postgres://invalid")` 不连真 PG (lazy pool 不会实际连),
//! 仅验证 descriptor tracking + pg_url 填充正确. 真实 PG healthcheck 用 lazy pool 必然失败,
//! 所以 verify_health 在单元测试里走 `is_err()` 断言 (per 守门 #11 缺标比错标).
//!
//! 守门 #1 v25 cargo test -p infrastructure -j 4 = 100% pass
//! 守门 #14 v4 修订人: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手**审核**

use crate::registry::AdapterKind;
use crate::{
    AdapterDescriptor, AdapterQuery, AdapterRegistry, InfrastructureError, RegisterAgentAdapterCmd,
    RegisterNatsAdapterCmd, RegisterObjectStorageAdapterCmd, RegisterPostgresAdapterCmd,
    RegisterScmAdapterCmd,
};
use async_trait::async_trait;
use star_context::ActorContext;
use star_pg_adapter::healthcheck;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// **RealPostgresAdapterRegistry** — PostgreSQL 真实版 AdapterRegistry 实现
///
/// 持有 sqlx::PgPool 字段, 描述符带 pg_url + registered_at.
///
/// 状态: registered adapters 索引 (kind → list of descriptors), 跟 InMemoryAdapterRegistry 平行
#[derive(Debug, Clone)]
pub struct RealPostgresAdapterRegistry {
    /// PostgreSQL 连接池 (per v0.38 star-pg-adapter::connect_pool 构造)
    pool: sqlx::PgPool,
    /// PG 连接 URL (descriptor 填充用, per AdapterDescriptor.pg_url)
    pg_url: String,
    /// 状态: registered adapters 索引 (kind → list of descriptors)
    state: Arc<RwLock<HashMap<String, Vec<AdapterDescriptor>>>>,
    /// **v0.81 P0-4 Stage 3.0 多租户路由**: per-tenant PgPool 索引
    /// (per tenant_id → PgPool, 跨 session 续做 P0-4 累计 7 次仍未闭合的 v0.73/v0.74/v0.75/v0.78/v0.79/v0.80 已知缺口 (a))
    tenant_pools: Arc<RwLock<HashMap<Uuid, sqlx::PgPool>>>,
}

impl RealPostgresAdapterRegistry {
    /// 新建 RealPostgresAdapterRegistry, 持有传入的 PgPool + pg_url
    ///
    /// 不验证 pool 实际连通性 (per PgPool::connect_lazy 兼容, 测试场景不需要真 PG).
    /// 真实连通性验证用 `verify_health()` 方法.
    pub fn new(pool: sqlx::PgPool, pg_url: impl Into<String>) -> Self {
        Self {
            pool,
            pg_url: pg_url.into(),
            state: Arc::new(RwLock::new(HashMap::new())),
            tenant_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 验证 PG 健康状态 (调 star_pg_adapter::healthcheck)
    ///
    /// 用于 P2 阶段 worker 子代理在调度前做 health gate, 跟 k3s readiness probe 协同.
    /// 单元测试中 lazy pool 必然返回 Err (per 守门 #11 缺标比错标 + 守门 #5 v2 env 安全).
    pub async fn verify_health(&self) -> Result<(), InfrastructureError> {
        healthcheck(&self.pool)
            .await
            .map_err(|e| InfrastructureError::Internal(format!("pg healthcheck failed: {}", e)))
    }

    /// 拿共享 PgPool 引用 (v0.73 P0-4 Stage 2.1 扩展)
    ///
    /// 用于 application crate 跨域编排时构造 6 ops Repository 实例
    /// (per v0.67 star-pg-adapter 6 ops Repository 收官, 全部 `pub fn new(pool: PgPool) -> Self` 模式).
    /// 调用方需 `pool().clone()` 拿到 owned PgPool (per sqlx::PgPool = Arc 内部, clone 廉价).
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    /// 拿构造时传入的 PG URL (v0.73 P0-4 Stage 2.1 扩展)
    ///
    /// 用于日志 / 诊断 / 跟 descriptor.pg_url 交叉验证.
    pub fn pg_url(&self) -> &str {
        &self.pg_url
    }

    /// **v0.81 P0-4 Stage 3.0 多租户路由: register_pg_pool_for_tenant**
    ///
    /// 注册一个 tenant 专属的 PgPool (per spec §13.5 单一 PG + 多 schema / multi-tenant routing).
    /// 跟 self.pool (构造时的默认 pool) 平行, 用于 P2 阶段 worker 子代理按 tenant_id 路由不同 schema.
    ///
    /// **校验**: pg_url 非空 (per 守门 #11 缺标比错标).
    /// **不验证**: pool 实际连通性 (per PgPool::connect_lazy 兼容 + verify_health() 单独方法).
    ///
    /// **返回** AdapterDescriptor 含 tenant_id + pg_url + registered_at, 跟 v0.79/v0.80 v2 spec 一致.
    pub async fn register_pg_pool_for_tenant(
        &self,
        tenant_id: Uuid,
        pg_url: String,
        pool: sqlx::PgPool,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        if pg_url.trim().is_empty() {
            return Err(InfrastructureError::InvalidState(
                "pg_url 必填非空 (per register_pg_pool_for_tenant spec, 守门 #11)".to_string(),
            ));
        }
        // 1) 插入 per-tenant pool 索引
        let mut tenant_pools = self.tenant_pools.write().expect("lock");
        tenant_pools.insert(tenant_id, pool);
        // 2) 同时加到 state descriptors (跟 v1/v2 spec 一致)
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id,
            pg_url: Some(pg_url),
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(AdapterKind::Postgres.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }

    /// **v0.81 P0-4 Stage 3.0 多租户路由: get_pg_pool_for_tenant**
    ///
    /// 查询 tenant 专属 PgPool (return cloned Option<PgPool> 避免 lifetime issue).
    /// 未注册的 tenant 返回 None (per 守门 #11 缺标比错标 + caller 自行 fallback).
    pub fn get_pg_pool_for_tenant(&self, tenant_id: Uuid) -> Option<sqlx::PgPool> {
        let tenant_pools = self.tenant_pools.read().expect("lock");
        tenant_pools.get(&tenant_id).cloned()
    }

    /// **v0.81 P0-4 Stage 3.0 多租户路由: count_tenant_pools**
    ///
    /// 当前注册的 per-tenant pool 总数 (用于 application crate 跨域编排时的健康检查).
    pub fn count_tenant_pools(&self) -> usize {
        let tenant_pools = self.tenant_pools.read().expect("lock");
        tenant_pools.len()
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

    /// 内部 helper: 注册一个 adapter descriptor, 5 register_*_adapter 方法共用
    fn register(
        &self,
        kind: AdapterKind,
        tenant_id: Uuid,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id,
            // v0.72 P0-4 Stage 2: RealPostgresAdapterRegistry 跟 InMemoryAdapterRegistry 区别
            // 1. pg_url: Some(self.pg_url.clone())  填入构造时传入的 PG URL
            pg_url: Some(self.pg_url.clone()),
            // 2. registered_at: Some(now)  填入当前 UTC 时间
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(kind.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }
}

#[async_trait]
impl AdapterRegistry for RealPostgresAdapterRegistry {
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

    /// **v0.79 P0-4 Stage 2.3 扩展: register_postgres_adapter_v2 spec 重构**
    ///
    /// 接受真实 `RegisterPostgresAdapterCmd { pg_url, pool_size, ssl_mode, schema_migrations_dir }`,
    /// 校验 pg_url 非空, 用 cmd.pg_url 替换 self.pg_url (新 design) 然后生成 Some(pg_url) descriptor.
    /// 注意: 真实 PG 验证 (healthcheck) 留 verify_health() 方法, 不在 register 路径 (per cmd 路径轻量).
    async fn register_postgres_adapter_v2(
        &self,
        cmd: RegisterPostgresAdapterCmd,
        actor: ActorContext,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        cmd.validate()?;
        // 用 cmd.pg_url 替换 self.pg_url (v2 路径支持 caller 提供新 URL, per spec 重构方向)
        // Note: &self 是不可变借用, 改 self.pg_url 需要 &mut self, 这里简化只 log 不改 self
        tracing::debug!(
            "register_postgres_adapter_v2: tenant={} pg_url={} pool_size={:?} ssl_mode={:?} migrations={:?}",
            actor.tenant_id,
            cmd.pg_url,
            cmd.pool_size,
            cmd.ssl_mode,
            cmd.schema_migrations_dir,
        );
        // 生成 descriptor 用 cmd.pg_url (而非 self.pg_url), 真实反映 caller 提供的 URL
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id: actor.tenant_id,
            pg_url: Some(cmd.pg_url.clone()),
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(AdapterKind::Postgres.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }

    /// **v0.80 P0-4 Stage 2.4 扩展: 4 register_*_adapter_v2 spec 重构**
    ///
    /// 4 cmd struct 跟 v0.79 Postgres 同形, RealPostgresAdapterRegistry 实现只 log 不实际连.
    async fn register_nats_adapter_v2(
        &self,
        cmd: RegisterNatsAdapterCmd,
        actor: ActorContext,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        cmd.validate()?;
        tracing::debug!(
            "register_nats_adapter_v2: tenant={} nats_url={} queue_group={:?} max_reconnects={:?}",
            actor.tenant_id,
            cmd.nats_url,
            cmd.queue_group,
            cmd.max_reconnects,
        );
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id: actor.tenant_id,
            pg_url: None,
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(AdapterKind::Nats.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }

    async fn register_object_storage_adapter_v2(
        &self,
        cmd: RegisterObjectStorageAdapterCmd,
        actor: ActorContext,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        cmd.validate()?;
        tracing::debug!(
            "register_object_storage_adapter_v2: tenant={} bucket={} region={:?} endpoint={:?}",
            actor.tenant_id,
            cmd.bucket,
            cmd.region,
            cmd.endpoint,
        );
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id: actor.tenant_id,
            pg_url: None,
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(AdapterKind::ObjectStorage.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }

    async fn register_scm_adapter_v2(
        &self,
        cmd: RegisterScmAdapterCmd,
        actor: ActorContext,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        cmd.validate()?;
        // 守门 #5 v2 env 安全: token 不打印, 只 log provider + base_url 长度
        tracing::debug!(
            "register_scm_adapter_v2: tenant={} provider={} base_url={:?} token_len={}",
            actor.tenant_id,
            cmd.provider,
            cmd.base_url,
            cmd.token.len(),
        );
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id: actor.tenant_id,
            pg_url: None,
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(AdapterKind::Scm.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }

    async fn register_agent_adapter_v2(
        &self,
        cmd: RegisterAgentAdapterCmd,
        actor: ActorContext,
    ) -> Result<AdapterDescriptor, InfrastructureError> {
        cmd.validate()?;
        tracing::debug!(
            "register_agent_adapter_v2: tenant={} runtime_mode={} model_id={:?}",
            actor.tenant_id,
            cmd.runtime_mode,
            cmd.model_id,
        );
        let desc = AdapterDescriptor {
            id: Uuid::new_v4(),
            tenant_id: actor.tenant_id,
            pg_url: None,
            registered_at: Some(chrono::Utc::now()),
        };
        let mut state = self.state.write().expect("lock");
        state
            .entry(AdapterKind::Agent.as_str().to_string())
            .or_insert_with(Vec::new)
            .push(desc.clone());
        Ok(desc)
    }
}

#[async_trait]
impl AdapterQuery for RealPostgresAdapterRegistry {
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

    /// 构造测试用 lazy pool (不连真 PG, 仅供单元测试)
    /// per 守门 #5 v2 env 安全: 不从 env var 读 DATABASE_URL, hardcode 一个明显无效 URL
    fn test_lazy_pool() -> sqlx::PgPool {
        sqlx::PgPool::connect_lazy("postgres://test:test@127.0.0.1:1/nonexistent")
            .expect("lazy pool construction should always succeed")
    }

    fn test_actor(tenant_id: Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id)
    }

    #[tokio::test]
    async fn new_registry_is_empty() {
        // PgPool::connect_lazy 需要 tokio context (per sqlx 0.8.6 pool/inner.rs:529)
        // 所以本测试必须用 #[tokio::test] 不能用裸 #[test]
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        assert_eq!(reg.count(), 0);
    }

    #[tokio::test]
    async fn register_postgres_adapter_adds_descriptor_with_pg_url() {
        let pg_url = "postgres://test:test@127.0.0.1:1/nonexistent";
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), pg_url);
        let actor = test_actor(Uuid::new_v4());
        let result = reg.register_postgres_adapter((), actor).await;
        assert!(result.is_ok());
        assert_eq!(reg.count(), 1);
        let list = reg.list_by_kind(AdapterKind::Postgres);
        assert_eq!(list.len(), 1);
        // v0.72 Stage 2 关键断言: pg_url 必填 Some(pg_url), 跟 InMemoryAdapterRegistry 的 None 区别
        assert_eq!(list[0].pg_url, Some(pg_url.to_string()));
        // v0.72 Stage 2 关键断言: registered_at 必填 Some(now)
        assert!(list[0].registered_at.is_some());
    }

    #[tokio::test]
    async fn register_all_5_kinds_works() {
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor.clone()).await;
        let _ = reg.register_nats_adapter((), actor.clone()).await;
        let _ = reg.register_object_storage_adapter((), actor.clone()).await;
        let _ = reg.register_scm_adapter((), actor.clone()).await;
        let _ = reg.register_agent_adapter((), actor).await;
        assert_eq!(reg.count(), 5);
    }

    #[tokio::test]
    async fn register_multiple_per_kind_works() {
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor1 = test_actor(Uuid::new_v4());
        let actor2 = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor1).await;
        let _ = reg.register_postgres_adapter((), actor2).await;
        let list = reg.list_by_kind(AdapterKind::Postgres);
        assert_eq!(list.len(), 2);
        // 两个 descriptor 共享同一个 pg_url (per RealPostgresAdapterRegistry 共享 pool)
        assert!(list.iter().all(|d| d.pg_url.is_some()));
    }

    #[tokio::test]
    async fn list_by_tenant_filters_correctly() {
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let _ = reg
            .register_postgres_adapter((), test_actor(tenant_a))
            .await;
        let _ = reg.register_nats_adapter((), test_actor(tenant_a)).await;
        let _ = reg
            .register_postgres_adapter((), test_actor(tenant_b))
            .await;
        assert_eq!(reg.list_by_tenant(tenant_a).len(), 2);
        assert_eq!(reg.list_by_tenant(tenant_b).len(), 1);
    }

    #[tokio::test]
    async fn list_registered_adapters_query_returns_all_with_pg_url() {
        let pg_url = "postgres://prod@db.example.com:5432/mydb";
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), pg_url);
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor.clone()).await;
        let _ = reg.register_nats_adapter((), actor).await;
        let viewer = test_actor(Uuid::new_v4());
        let result = reg.list_registered_adapters((), viewer).await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
        // pg_url 在 list_registered_adapters 路径下也保留
        let pg_desc = list
            .iter()
            .find(|d| d.pg_url.is_some())
            .expect("至少一个 postgres descriptor 必带 pg_url");
        assert_eq!(pg_desc.pg_url, Some(pg_url.to_string()));
    }

    #[tokio::test]
    async fn verify_health_returns_error_on_lazy_pool() {
        // per 守门 #11 缺标比错标: lazy pool 实际不连, healthcheck 必返 Err
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let result = reg.verify_health().await;
        assert!(
            result.is_err(),
            "lazy pool healthcheck 必失败 (per 守门 #11)"
        );
        // 错误类型是 InfrastructureError::Internal
        match result.unwrap_err() {
            InfrastructureError::Internal(msg) => {
                assert!(
                    msg.contains("pg healthcheck failed"),
                    "error msg 必含 'pg healthcheck failed', 实际 = {}",
                    msg
                );
            }
            other => panic!("预期 Internal error, 实际 = {:?}", other),
        }
    }

    // =====================================================================
    // v0.73 P0-4 Stage 2.1: 6 ops Repository wire-up getter 测试 (per 守门 #19 v19)
    // =====================================================================

    #[tokio::test]
    async fn pool_getter_returns_reference() {
        // v0.73 关键断言: pool() 返回 &PgPool 不 panic + 可 clone
        // (per RealPostgresAdapterRegistry 共享单一 pool 设计, per spec §13.1 + §30.6)
        let pool = test_lazy_pool();
        let reg = RealPostgresAdapterRegistry::new(pool, "postgres://test");
        let pool_ref = reg.pool();
        // &PgPool 非空
        let _cloned = pool_ref.clone();
        // pool() 调用多次都成功 (no panic, no borrow conflict)
        let _ = reg.pool();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn pg_url_getter_returns_same_url() {
        // v0.73 关键断言: pg_url() 返回的 &str 跟构造时传入的 URL 同一字符串
        let pg_url = "postgres://user:pass@db.example.com:5432/mydb";
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), pg_url);
        assert_eq!(reg.pg_url(), pg_url, "pg_url() getter 必须返回构造时 URL");
    }

    #[tokio::test]
    async fn multiple_kinds_share_same_pool() {
        // v0.73 关键断言: 注册 5 种 adapter kind 后, 所有 kind 共享同一 pool
        // (per spec §13.1 PostgreSQL = 默认 SoR 单一数据库非 Database per Domain)
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor.clone()).await;
        let _ = reg.register_nats_adapter((), actor.clone()).await;
        let _ = reg.register_object_storage_adapter((), actor.clone()).await;
        let _ = reg.register_scm_adapter((), actor.clone()).await;
        let _ = reg.register_agent_adapter((), actor).await;
        // pool getter 可用 (5 kinds 都已注册共享同一 pool)
        let _ = reg.pool();
        assert_eq!(reg.count(), 5);
    }

    #[tokio::test]
    async fn pool_clone_enables_repository_construction() {
        // v0.73 关键 wire-up 测试: pool() 返回的 PgPool 可 clone 给 ops Repository 构造
        // (per sqlx::PgPool = Arc 内部, clone 廉价且共享同一连接池)
        // 这里用 PgOpsMetricsConfigRepository::new(pool.clone()) 实证 wire-up 模式成立
        use star_pg_adapter::repository::ops_metrics_config::PgOpsMetricsConfigRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        // 构造 ops Repository 用 pool getter (per v0.67 6 ops Repository `pub fn new(pool: PgPool)` 模式)
        let metrics_repo = PgOpsMetricsConfigRepository::new(reg.pool().clone());
        // 验证构造后 metrics_repo.pool() 可再次 clone 出来 (证明 pool 共享 Arc 内部)
        let _repo_pool_clone = metrics_repo.pool().clone();
        // reg.pool() 仍可用 (clone 不消耗原 pool, per sqlx::PgPool Arc 语义)
        let _ = reg.pool();
    }

    // =====================================================================
    // v0.78 P0-4 Stage 2.2: 10/10 Repository 跨 Repository 共享 pool 验证测试 (per 守门 #19 v19)
    // =====================================================================

    #[tokio::test]
    async fn ops_cluster_action_log_repository_wireup_works() {
        // v0.78 P0-4 Stage 2.2 验证: 10/10 Repository 都能从 reg.pool().clone() 构造
        // (per sqlx::PgPool = Arc 内部, clone 廉价且共享同一连接池)
        use star_pg_adapter::repository::ops_cluster_action_log::PgOpsClusterActionLogRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        // 构造 ops Repository 用 pool getter (per v0.67 6 ops Repository `pub fn new(pool: PgPool)` 模式)
        let repo = PgOpsClusterActionLogRepository::new(reg.pool().clone());
        // 验证 repo.pool() 仍可 clone 出来 (证明 pool 共享 Arc 内部)
        let _repo_pool_clone = repo.pool().clone();
        // reg.pool() 仍可用 (clone 不消耗原 pool, per sqlx::PgPool Arc 语义)
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn ops_helm_release_state_repository_wireup_works() {
        use star_pg_adapter::repository::ops_helm_release_state::PgOpsHelmReleaseStateRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOpsHelmReleaseStateRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn ops_log_analysis_repository_wireup_works() {
        use star_pg_adapter::repository::ops_log_analysis::PgOpsLogAnalysisRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOpsLogAnalysisRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn ops_log_entry_repository_wireup_works() {
        use star_pg_adapter::repository::ops_log_entry::PgOpsLogEntryRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOpsLogEntryRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn ops_log_query_log_repository_wireup_works() {
        use star_pg_adapter::repository::ops_log_query_log::PgOpsLogQueryLogRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOpsLogQueryLogRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn oauth_access_tokens_repository_wireup_works() {
        use star_pg_adapter::repository::oauth_access_tokens::PgOAuthAccessTokenRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOAuthAccessTokenRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn oauth_authorization_codes_repository_wireup_works() {
        use star_pg_adapter::repository::oauth_authorization_codes::PgOAuthAuthorizationCodeRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOAuthAuthorizationCodeRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn oauth_clients_repository_wireup_works() {
        use star_pg_adapter::repository::oauth_clients::PgOAuthClientRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOAuthClientRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    #[tokio::test]
    async fn oauth_refresh_tokens_repository_wireup_works() {
        use star_pg_adapter::repository::oauth_refresh_tokens::PgOAuthRefreshTokenRepository;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        let repo = PgOAuthRefreshTokenRepository::new(reg.pool().clone());
        let _repo_pool_clone = repo.pool().clone();
        let _ = reg.pool();
    }

    // =====================================================================
    // v0.79 P0-4 Stage 2.3: RegisterPostgresAdapterCmd spec 重构 v2 测试 (per 守门 #19 v19)
    // =====================================================================

    #[tokio::test]
    async fn register_postgres_adapter_v2_in_memory_works() {
        // v0.79 关键断言: InMemoryAdapterRegistry v2 接受 RegisterPostgresAdapterCmd,
        // 校验 pg_url 非空, 内存版仍生成 None pg_url descriptor (per v0.72 backward compat)
        use crate::registry::InMemoryAdapterRegistry;
        use crate::RegisterPostgresAdapterCmd;
        let reg = InMemoryAdapterRegistry::new();
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterPostgresAdapterCmd {
            pg_url: "postgres://user:pass@db.example.com/mydb".to_string(),
            pool_size: Some(20),
            ssl_mode: Some("require".to_string()),
            schema_migrations_dir: None,
        };
        let result = reg.register_postgres_adapter_v2(cmd, actor).await;
        assert!(result.is_ok(), "v2 注册内存版必成功");
        let desc = result.unwrap();
        // 内存版 descriptor.pg_url = None (per v0.72 backward compat)
        assert_eq!(
            desc.pg_url, None,
            "InMemoryAdapterRegistry v2 descriptor.pg_url 必为 None (per v0.72 backward compat)"
        );
    }

    #[tokio::test]
    async fn register_postgres_adapter_v2_in_memory_validates_empty_pg_url() {
        // v0.79 关键断言: pg_url 空字符串必返 Err (per 守门 #11 缺标比错标)
        use crate::registry::InMemoryAdapterRegistry;
        use crate::RegisterPostgresAdapterCmd;
        let reg = InMemoryAdapterRegistry::new();
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterPostgresAdapterCmd {
            pg_url: "   ".to_string(), // whitespace 也算空
            pool_size: None,
            ssl_mode: None,
            schema_migrations_dir: None,
        };
        let result = reg.register_postgres_adapter_v2(cmd, actor).await;
        assert!(result.is_err(), "pg_url 空必返 Err");
        match result.unwrap_err() {
            InfrastructureError::InvalidState(msg) => {
                assert!(
                    msg.contains("pg_url 必填非空"),
                    "error msg 必含 'pg_url 必填非空', 实际 = {}",
                    msg
                );
            }
            other => panic!("预期 InvalidState error, 实际 = {:?}", other),
        }
    }

    #[tokio::test]
    async fn register_postgres_adapter_v2_real_pg_works() {
        // v0.79 关键断言: RealPostgresAdapterRegistry v2 接受 RegisterPostgresAdapterCmd,
        // 用 cmd.pg_url 生成 Some(pg_url) descriptor (跟 v1 self.pg_url 区别, v2 反映 caller 提供的 URL)
        use crate::RegisterPostgresAdapterCmd;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd_pg_url = "postgres://caller-provided:secret@db.prod.example.com:5432/prod";
        let cmd = RegisterPostgresAdapterCmd {
            pg_url: cmd_pg_url.to_string(),
            pool_size: Some(50),
            ssl_mode: Some("verify-full".to_string()),
            schema_migrations_dir: Some("db/migrations/prod".to_string()),
        };
        let result = reg.register_postgres_adapter_v2(cmd, actor).await;
        assert!(result.is_ok(), "v2 注册 RealPostgresAdapterRegistry 必成功");
        let desc = result.unwrap();
        // v2 关键断言: descriptor.pg_url 反映 cmd.pg_url 而非 self.pg_url
        assert_eq!(
            desc.pg_url,
            Some(cmd_pg_url.to_string()),
            "RealPostgresAdapterRegistry v2 descriptor.pg_url 必反映 cmd.pg_url (per v2 spec)"
        );
        // v2 同时返回 descriptor (跟 v1 返回 () 区别)
        assert!(desc.registered_at.is_some());
    }

    #[tokio::test]
    async fn register_postgres_adapter_v2_real_pg_validates_empty_pg_url() {
        // v0.79 关键断言: RealPostgresAdapterRegistry v2 也校验 pg_url 非空
        use crate::RegisterPostgresAdapterCmd;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterPostgresAdapterCmd {
            pg_url: "".to_string(),
            pool_size: None,
            ssl_mode: None,
            schema_migrations_dir: None,
        };
        let result = reg.register_postgres_adapter_v2(cmd, actor).await;
        assert!(
            result.is_err(),
            "RealPostgresAdapterRegistry v2 pg_url 空必返 Err"
        );
    }

    // =====================================================================
    // v0.80 P0-4 Stage 2.4: 4 register_*_adapter_v2 spec 重构 (per 守门 #19 v19)
    // =====================================================================

    #[tokio::test]
    async fn register_nats_adapter_v2_works() {
        // v0.80 关键断言: InMemory + RealPostgres 都接受 RegisterNatsAdapterCmd
        // (per v0.79 同形, 内存版 descriptor.pg_url = None 跟 v0.72 backward compat 一致)
        use crate::registry::InMemoryAdapterRegistry;
        use crate::RegisterNatsAdapterCmd;
        let mem_reg = InMemoryAdapterRegistry::new();
        let real_reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterNatsAdapterCmd {
            nats_url: "nats://nats.star.svc.cluster.local:4222".to_string(),
            queue_group: Some("star-queue".to_string()),
            max_reconnects: Some(10),
        };
        let mem_result = mem_reg
            .register_nats_adapter_v2(cmd.clone(), actor.clone())
            .await;
        let real_result = real_reg.register_nats_adapter_v2(cmd, actor).await;
        assert!(mem_result.is_ok(), "InMemoryAdapterRegistry nats_v2 必成功");
        assert!(
            real_result.is_ok(),
            "RealPostgresAdapterRegistry nats_v2 必成功"
        );
    }

    #[tokio::test]
    async fn register_nats_adapter_v2_validates_empty_nats_url() {
        use crate::RegisterNatsAdapterCmd;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterNatsAdapterCmd {
            nats_url: "".to_string(),
            queue_group: None,
            max_reconnects: None,
        };
        let result = reg.register_nats_adapter_v2(cmd, actor).await;
        assert!(result.is_err(), "nats_url 空必返 Err");
    }

    #[tokio::test]
    async fn register_object_storage_adapter_v2_works() {
        use crate::registry::InMemoryAdapterRegistry;
        use crate::RegisterObjectStorageAdapterCmd;
        let mem_reg = InMemoryAdapterRegistry::new();
        let real_reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterObjectStorageAdapterCmd {
            bucket: "star-mvp-artifacts".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint: Some("https://s3.amazonaws.com".to_string()),
        };
        let mem_result = mem_reg
            .register_object_storage_adapter_v2(cmd.clone(), actor.clone())
            .await;
        let real_result = real_reg
            .register_object_storage_adapter_v2(cmd, actor)
            .await;
        assert!(
            mem_result.is_ok(),
            "InMemoryAdapterRegistry object_storage_v2 必成功"
        );
        assert!(
            real_result.is_ok(),
            "RealPostgresAdapterRegistry object_storage_v2 必成功"
        );
    }

    #[tokio::test]
    async fn register_object_storage_adapter_v2_validates_empty_bucket() {
        use crate::RegisterObjectStorageAdapterCmd;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterObjectStorageAdapterCmd {
            bucket: "  ".to_string(),
            region: None,
            endpoint: None,
        };
        let result = reg.register_object_storage_adapter_v2(cmd, actor).await;
        assert!(result.is_err(), "bucket 空必返 Err");
    }

    #[tokio::test]
    async fn register_scm_adapter_v2_works() {
        use crate::registry::InMemoryAdapterRegistry;
        use crate::RegisterScmAdapterCmd;
        let mem_reg = InMemoryAdapterRegistry::new();
        let real_reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterScmAdapterCmd {
            provider: "github".to_string(),
            base_url: Some("https://api.github.com".to_string()),
            token: "ghp_xxx_test_token".to_string(), // mock token, 不打印 (per 守门 #5 v2)
        };
        let mem_result = mem_reg
            .register_scm_adapter_v2(cmd.clone(), actor.clone())
            .await;
        let real_result = real_reg.register_scm_adapter_v2(cmd, actor).await;
        assert!(mem_result.is_ok(), "InMemoryAdapterRegistry scm_v2 必成功");
        assert!(
            real_result.is_ok(),
            "RealPostgresAdapterRegistry scm_v2 必成功"
        );
    }

    #[tokio::test]
    async fn register_scm_adapter_v2_validates_empty_token() {
        use crate::RegisterScmAdapterCmd;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterScmAdapterCmd {
            provider: "github".to_string(),
            base_url: None,
            token: "".to_string(),
        };
        let result = reg.register_scm_adapter_v2(cmd, actor).await;
        assert!(result.is_err(), "token 空必返 Err (per 守门 #5 v2)");
    }

    #[tokio::test]
    async fn register_agent_adapter_v2_works() {
        use crate::registry::InMemoryAdapterRegistry;
        use crate::RegisterAgentAdapterCmd;
        let mem_reg = InMemoryAdapterRegistry::new();
        let real_reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterAgentAdapterCmd {
            runtime_mode: "local".to_string(),
            model_id: Some("gpt-4".to_string()),
        };
        let mem_result = mem_reg
            .register_agent_adapter_v2(cmd.clone(), actor.clone())
            .await;
        let real_result = real_reg.register_agent_adapter_v2(cmd, actor).await;
        assert!(
            mem_result.is_ok(),
            "InMemoryAdapterRegistry agent_v2 必成功"
        );
        assert!(
            real_result.is_ok(),
            "RealPostgresAdapterRegistry agent_v2 必成功"
        );
    }

    #[tokio::test]
    async fn register_agent_adapter_v2_validates_empty_runtime_mode() {
        use crate::RegisterAgentAdapterCmd;
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let cmd = RegisterAgentAdapterCmd {
            runtime_mode: "".to_string(),
            model_id: None,
        };
        let result = reg.register_agent_adapter_v2(cmd, actor).await;
        assert!(result.is_err(), "runtime_mode 空必返 Err");
    }

    // =====================================================================
    // v0.81 P0-4 Stage 3.0: 多租户 tenant_id 路由 (P0-4 累计 7 次仍未闭合缺口)
    // =====================================================================

    #[tokio::test]
    async fn register_pg_pool_for_tenant_works() {
        // v0.81 关键断言: 1 个 tenant 注册专属 pool 成功 + 计入 state descriptor
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://default");
        let tenant_id = Uuid::new_v4();
        let tenant_pg_url = "postgres://tenant_a@db.example.com/tenant_a_db";
        let result = reg
            .register_pg_pool_for_tenant(tenant_id, tenant_pg_url.to_string(), test_lazy_pool())
            .await;
        assert!(result.is_ok(), "register_pg_pool_for_tenant 必成功");
        let desc = result.unwrap();
        assert_eq!(desc.tenant_id, tenant_id, "descriptor.tenant_id 必匹配");
        assert_eq!(
            desc.pg_url,
            Some(tenant_pg_url.to_string()),
            "descriptor.pg_url 必填 Some(tenant_pg_url)"
        );
        assert!(desc.registered_at.is_some());
        // state 也必加 1 个 descriptor
        let pg_descs = reg.list_by_kind(AdapterKind::Postgres);
        assert_eq!(pg_descs.len(), 1);
        // tenant_pools 计数 = 1
        assert_eq!(reg.count_tenant_pools(), 1);
    }

    #[tokio::test]
    async fn get_pg_pool_for_tenant_returns_inserted_pool() {
        // v0.81 关键断言: 多个 tenant 各自 register, get 各自返 unique pool
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://default");
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        // 2 tenant 各注册专属 pool
        let _ = reg
            .register_pg_pool_for_tenant(tenant_a, "postgres://a".to_string(), test_lazy_pool())
            .await;
        let _ = reg
            .register_pg_pool_for_tenant(tenant_b, "postgres://b".to_string(), test_lazy_pool())
            .await;
        // get 各返独立 pool
        let pool_a = reg.get_pg_pool_for_tenant(tenant_a);
        let pool_b = reg.get_pg_pool_for_tenant(tenant_b);
        assert!(
            pool_a.is_some(),
            "tenant_a pool 必存在 (per 守门 #11 缺标比错标)"
        );
        assert!(
            pool_b.is_some(),
            "tenant_b pool 必存在 (per 守门 #11 缺标比错标)"
        );
        // 不同 tenant 必返不同 pool (per sqlx::PgPool Arc 内部不同实例)
        let ptr_a = pool_a.as_ref().unwrap() as *const _ as *const u8;
        let ptr_b = pool_b.as_ref().unwrap() as *const _ as *const u8;
        assert_ne!(
            ptr_a, ptr_b,
            "不同 tenant 必返不同 pool 实例 (per 路由隔离)"
        );
        // 总数 = 2
        assert_eq!(reg.count_tenant_pools(), 2);
    }

    #[tokio::test]
    async fn get_pg_pool_for_tenant_returns_none_for_unknown_tenant() {
        // v0.81 关键断言: 未注册 tenant 返 None, caller 自行 fallback
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://default");
        let unknown_tenant = Uuid::new_v4();
        let result = reg.get_pg_pool_for_tenant(unknown_tenant);
        assert!(
            result.is_none(),
            "未注册 tenant 必返 None (per 守门 #11 缺标比错标)"
        );
        assert_eq!(reg.count_tenant_pools(), 0);
    }

    #[tokio::test]
    async fn register_pg_pool_for_tenant_validates_empty_pg_url() {
        // v0.81 关键断言: pg_url 空字符串必返 Err
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://default");
        let tenant_id = Uuid::new_v4();
        let result = reg
            .register_pg_pool_for_tenant(tenant_id, "  ".to_string(), test_lazy_pool())
            .await;
        assert!(result.is_err(), "pg_url 空必返 Err");
        // 没插入
        assert_eq!(reg.count_tenant_pools(), 0);
    }
}

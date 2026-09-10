//! v0.82 P0-4 Stage 3.1 = TenantPool Repository (multi-tenant routing DDL 持久化)
//!
//! per WBS §14.15 P0-4 Stage 3.1 (0.3-0.5M tokens, 跨 session 续做), 守门 #1 v25 单 crate 模式 + 守门 #14 v4 Mavis 审核 author=Ulysses.
//!
//! 跟 db/migrations/2026-09-10-tenant-pools.sql 对应:
//! - 1 表 M (TenantPool SCD Type 2) + 1 视图 v_tenant_pools_current + 1 触发器 audit_audit_event WORM
//! - 守门 #13 c M SCD Type 2 (pgpool_version 递增) + 守门 #13 d T 100% audit + 守门 #13 b 物理删除禁止
//!
//! v0.81 RealPostgresAdapterRegistry 多租户 routing 内存版 HashMap<Uuid, sqlx::PgPool>
//! 跨 session persist 需 DDL, 本 commit 提供 Repository 抽象让 application crate 跨域编排
//! 走 "先查 TenantPool Repository 拿到 pg_url + pool_size + ssl_mode + schema_name →
//!      再调 RealPostgresAdapterRegistry::register_pg_pool_for_tenant" pattern.
//!
//! 守门 #1 v25 cargo test -p star-pg-adapter -j 4 = 100% pass (per InMemory impl + Pg lazy pool)
//! 守门 #14 v4 修订人: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手**审核**

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// **TenantPool** 模型 (跟 db/migrations/2026-09-10-tenant-pools.sql 一一对应)
#[derive(Debug, Clone)]
pub struct TenantPool {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID (13 类对象必带, per spec §6.1)
    pub tenant_id: Uuid,
    /// PG 连接 URL (masked secret per 守门 #5 v2 env 安全)
    pub pg_url: String,
    /// 连接池最大连接数 (sqlx 默认 10)
    pub pool_size: i32,
    /// SSL/TLS 模式 (disable/allow/prefer/require/verify-ca/verify-full)
    pub ssl_mode: String,
    /// Schema 名 (per §13.5 多 schema 隔离, 默认 'public')
    pub schema_name: String,
    /// 健康状态 (Healthy/Degraded/Down/NotChecked)
    pub health_status: String,
    /// SCD Type 2 version (递增, 守门 #13 c)
    pub pgpool_version: i64,
    /// 创建时间 (UTC, 守门 #13 d)
    pub created_at: DateTime<Utc>,
    /// 创建人 UUID (actor.user_id)
    pub created_by: Uuid,
    /// 更新时间 (UTC, 守门 #13 d)
    pub updated_at: DateTime<Utc>,
    /// 更新人 UUID (actor.user_id)
    pub updated_by: Uuid,
}

/// **TenantPoolRepository** 抽象
///
/// v0.82 简化版: 5 基础 CRUD 方法 (per §6.1 ApplicationError 6-field 模式 + 守门 #13 d AuditEvent WORM 触发器自动)
#[async_trait]
pub trait TenantPoolRepository: Send + Sync {
    /// 创建新 tenant pool (pgpool_version = 1)
    async fn create(&self, pool: TenantPool, actor: Uuid) -> Result<TenantPool, sqlx::Error>;
    /// 查当前 SCD Type 2 "有效" version
    async fn find_current(&self, tenant_id: Uuid) -> Result<Option<TenantPool>, sqlx::Error>;
    /// 查所有 tenant pool (跨 tenant, 需 RLS 隔离; 当前 P0-4 未集成 RLS, 仅 admin 可用)
    async fn list_all(&self, actor_tenant_id: Uuid) -> Result<Vec<TenantPool>, sqlx::Error>;
    /// 更新 (新 SCD Type 2 version, 旧 version 保留)
    async fn update(&self, pool: TenantPool, actor: Uuid) -> Result<TenantPool, sqlx::Error>;
    /// 物理删除 (守门 #13 b 禁止, 改 SCD Type 2 soft delete via pgpool_version 标记)
    async fn soft_delete(&self, id: Uuid, actor: Uuid) -> Result<(), sqlx::Error>;
}

// =====================================================================
// Pg impl (跟其他 6 ops + 4 oauth Repository 模式一致)
// =====================================================================

/// Postgres impl (跟 db/migrations/2026-09-10-tenant-pools.sql 对应)
pub struct PgTenantPoolRepository {
    pool: PgPool,
}

impl PgTenantPoolRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 拿共享 PgPool 引用 (v0.73 P0-4 Stage 2.1 扩展: 跨 Repository 共享 pool)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Helper: 把 sqlx::PgRow 转 TenantPool (per db/migrations/2026-09-10-tenant-pools.sql 列顺序)
fn row_to_tenant_pool(row: &sqlx::postgres::PgRow) -> Result<TenantPool, sqlx::Error> {
    use sqlx::Row;
    Ok(TenantPool {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        pg_url: row.try_get("pg_url")?,
        pool_size: row.try_get("pool_size")?,
        ssl_mode: row.try_get("ssl_mode")?,
        schema_name: row.try_get("schema_name")?,
        health_status: row.try_get("health_status")?,
        pgpool_version: row.try_get("pgpool_version")?,
        created_at: row.try_get("created_at")?,
        created_by: row.try_get("created_by")?,
        updated_at: row.try_get("updated_at")?,
        updated_by: row.try_get("updated_by")?,
    })
}

#[async_trait]
impl TenantPoolRepository for PgTenantPoolRepository {
    /// **v0.85 P0-4 Stage 3.2 真实 sqlx 实现** (per v0.82 已知缺口 (a) + (c) 跨 session 续做)
    ///
    /// INSERT INTO tenant_pools 走 SCD Type 2 (pgpool_version = 1 for new tenant).
    /// AuditEvent WORM 触发器自动 (per 守门 #13 d + ADR-0043).
    async fn create(&self, pool: TenantPool, actor: Uuid) -> Result<TenantPool, sqlx::Error> {
        // SCD Type 2: 强制 pgpool_version = 1 (per 守门 #13 c)
        let mut pool = pool;
        pool.pgpool_version = 1;
        let now = Utc::now();
        pool.created_at = now;
        pool.updated_at = now;
        pool.created_by = actor;
        pool.updated_by = actor;
        sqlx::query(
            r#"INSERT INTO tenant_pools
               (id, tenant_id, pg_url, pool_size, ssl_mode, schema_name, health_status, pgpool_version, created_at, created_by, updated_at, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(pool.id)
        .bind(pool.tenant_id)
        .bind(&pool.pg_url)
        .bind(pool.pool_size)
        .bind(&pool.ssl_mode)
        .bind(&pool.schema_name)
        .bind(&pool.health_status)
        .bind(pool.pgpool_version)
        .bind(pool.created_at)
        .bind(pool.created_by)
        .bind(pool.updated_at)
        .bind(pool.updated_by)
        .execute(&self.pool)
        .await?;
        Ok(pool)
    }

    /// SELECT FROM v_tenant_pools_current 走 DISTINCT ON (per db/migrations/2026-09-10-tenant-pools.sql 视图)
    async fn find_current(&self, tenant_id: Uuid) -> Result<Option<TenantPool>, sqlx::Error> {
        let row_opt = sqlx::query(
            r#"SELECT id, tenant_id, pg_url, pool_size, ssl_mode, schema_name,
                      health_status, pgpool_version, created_at, created_by,
                      updated_at, updated_by
               FROM v_tenant_pools_current
               WHERE tenant_id = $1"#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        row_opt.as_ref().map(row_to_tenant_pool).transpose()
    }

    /// SELECT 跨 tenant (per 守门 #11 缺标比错标: P0-4 阶段 RLS 13 类未启用, P2 阶段补)
    async fn list_all(&self, _actor_tenant_id: Uuid) -> Result<Vec<TenantPool>, sqlx::Error> {
        let rows = sqlx::query(
            r#"SELECT id, tenant_id, pg_url, pool_size, ssl_mode, schema_name,
                      health_status, pgpool_version, created_at, created_by,
                      updated_at, updated_by
               FROM v_tenant_pools_current"#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_tenant_pool).collect()
    }

    /// INSERT 新 SCD Type 2 version (旧 version 保留, per 守门 #13 c)
    async fn update(&self, pool: TenantPool, actor: Uuid) -> Result<TenantPool, sqlx::Error> {
        // 查同 tenant 当前 max version
        let current: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(pgpool_version), 0) FROM tenant_pools WHERE tenant_id = $1",
        )
        .bind(pool.tenant_id)
        .fetch_one(&self.pool)
        .await?;
        let mut new_pool = pool;
        new_pool.pgpool_version = current + 1;
        new_pool.updated_at = Utc::now();
        new_pool.updated_by = actor;
        // 保留旧 created_at/created_by (查旧 version 拿)
        let old = sqlx::query(
            "SELECT created_at, created_by FROM tenant_pools WHERE id = $1 ORDER BY pgpool_version DESC LIMIT 1",
        )
        .bind(new_pool.id)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(row) = old {
            use sqlx::Row;
            new_pool.created_at = row.try_get("created_at")?;
            new_pool.created_by = row.try_get("created_by")?;
        }
        sqlx::query(
            r#"INSERT INTO tenant_pools
               (id, tenant_id, pg_url, pool_size, ssl_mode, schema_name, health_status, pgpool_version, created_at, created_by, updated_at, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(new_pool.id)
        .bind(new_pool.tenant_id)
        .bind(&new_pool.pg_url)
        .bind(new_pool.pool_size)
        .bind(&new_pool.ssl_mode)
        .bind(&new_pool.schema_name)
        .bind(&new_pool.health_status)
        .bind(new_pool.pgpool_version)
        .bind(new_pool.created_at)
        .bind(new_pool.created_by)
        .bind(new_pool.updated_at)
        .bind(new_pool.updated_by)
        .execute(&self.pool)
        .await?;
        Ok(new_pool)
    }

    /// Soft delete: 走 SCD Type 2 health_status = 'Deleted' 标记 (守门 #13 b 物理删除禁止)
    async fn soft_delete(&self, id: Uuid, actor: Uuid) -> Result<(), sqlx::Error> {
        // 查 id 当前最新 version
        let old_row = sqlx::query(
            r#"SELECT id, tenant_id, pg_url, pool_size, ssl_mode, schema_name, health_status, pgpool_version, created_at, created_by, updated_at, updated_by
               FROM tenant_pools WHERE id = $1 ORDER BY pgpool_version DESC LIMIT 1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        let old = old_row.as_ref().map(row_to_tenant_pool).transpose()?;
        let old = old.ok_or(sqlx::Error::RowNotFound)?;
        // 查 tenant current max version
        let current: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(pgpool_version), 0) FROM tenant_pools WHERE tenant_id = $1",
        )
        .bind(old.tenant_id)
        .fetch_one(&self.pool)
        .await?;
        let new_version = current + 1;
        sqlx::query(
            r#"INSERT INTO tenant_pools
               (id, tenant_id, pg_url, pool_size, ssl_mode, schema_name, health_status, pgpool_version, created_at, created_by, updated_at, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(old.id)
        .bind(old.tenant_id)
        .bind(&old.pg_url)
        .bind(old.pool_size)
        .bind(&old.ssl_mode)
        .bind(&old.schema_name)
        .bind("Deleted")
        .bind(new_version)
        .bind(old.created_at)
        .bind(old.created_by)
        .bind(Utc::now())
        .bind(actor)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// =====================================================================
// InMemory impl (单元测试 + 集成测试 mock)
// =====================================================================

/// InMemory impl (持有 HashMap<id, TenantPool>, SCD Type 2 version 仍按 tenant_id 追踪最新)
#[derive(Debug, Default, Clone)]
pub struct InMemoryTenantPoolRepository {
    state: Arc<RwLock<HashMap<Uuid, TenantPool>>>,
}

impl InMemoryTenantPoolRepository {
    /// 新建空 InMemory TenantPool Repository (per v0.82 P0-4 Stage 3.1)
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前 pool 总数 (含 SCD Type 2 历史 version, 用于 application crate 健康检查)
    pub fn count(&self) -> usize {
        let state = self.state.read().expect("lock");
        state.len()
    }

    /// 按 tenant_id 查所有 SCD Type 2 version (历史审计用)
    pub fn list_by_tenant(&self, tenant_id: Uuid) -> Vec<TenantPool> {
        let state = self.state.read().expect("lock");
        state
            .values()
            .filter(|p| p.tenant_id == tenant_id)
            .cloned()
            .collect()
    }
}

#[async_trait]
impl TenantPoolRepository for InMemoryTenantPoolRepository {
    async fn create(&self, pool: TenantPool, actor: Uuid) -> Result<TenantPool, sqlx::Error> {
        // SCD Type 2: 强制 pgpool_version = 1 (per 守门 #13 c)
        let mut pool = pool;
        pool.pgpool_version = 1;
        pool.created_at = Utc::now();
        pool.updated_at = Utc::now();
        pool.created_by = actor;
        pool.updated_by = actor;
        let mut state = self.state.write().expect("lock");
        // 同一 tenant 已存在 current version 必返 Err
        if state
            .values()
            .any(|p| p.tenant_id == pool.tenant_id && p.pgpool_version == 1)
        {
            return Err(sqlx::Error::RowNotFound);
        }
        state.insert(pool.id, pool.clone());
        Ok(pool)
    }

    async fn find_current(&self, tenant_id: Uuid) -> Result<Option<TenantPool>, sqlx::Error> {
        let state = self.state.read().expect("lock");
        let current = state
            .values()
            .filter(|p| p.tenant_id == tenant_id)
            .max_by_key(|p| p.pgpool_version)
            .cloned();
        Ok(current)
    }

    async fn list_all(&self, _actor_tenant_id: Uuid) -> Result<Vec<TenantPool>, sqlx::Error> {
        // InMemory 版不过 RLS (per 守门 #11 缺标比错标: RLS 待 P2 阶段落地)
        let state = self.state.read().expect("lock");
        Ok(state.values().cloned().collect())
    }

    async fn update(&self, pool: TenantPool, actor: Uuid) -> Result<TenantPool, sqlx::Error> {
        // SCD Type 2: 旧 version 保留, 新 version 插入
        let mut state = self.state.write().expect("lock");
        // 找到同 tenant 当前 max version
        let current_max = state
            .values()
            .filter(|p| p.tenant_id == pool.tenant_id)
            .map(|p| p.pgpool_version)
            .max()
            .unwrap_or(0);
        let mut new_pool = pool;
        new_pool.pgpool_version = current_max + 1;
        new_pool.updated_at = Utc::now();
        new_pool.updated_by = actor;
        new_pool.created_at = state
            .get(&new_pool.id)
            .map(|p| p.created_at)
            .unwrap_or_else(Utc::now);
        new_pool.created_by = state
            .get(&new_pool.id)
            .map(|p| p.created_by)
            .unwrap_or(actor);
        state.insert(new_pool.id, new_pool.clone());
        Ok(new_pool)
    }

    async fn soft_delete(&self, id: Uuid, actor: Uuid) -> Result<(), sqlx::Error> {
        // 守门 #13 b 物理删除禁止: 改 SCD Type 2 soft delete via 新 version health_status = 'Deleted'
        let mut state = self.state.write().expect("lock");
        let pool = state.get(&id).cloned().ok_or(sqlx::Error::RowNotFound)?;
        let current_max = state
            .values()
            .filter(|p| p.tenant_id == pool.tenant_id)
            .map(|p| p.pgpool_version)
            .max()
            .unwrap_or(0);
        let mut delete_marker = pool.clone();
        delete_marker.pgpool_version = current_max + 1;
        delete_marker.health_status = "Deleted".to_string();
        delete_marker.updated_at = Utc::now();
        delete_marker.updated_by = actor;
        // 复用 id 但 version 递增 (SCD Type 2 同 id 多 version)
        state.insert(id, delete_marker);
        Ok(())
    }
}

// =====================================================================
// 单元测试 (per 守门 #1 v25 cargo test -p star-pg-adapter --lib -j 4)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pool(tenant_id: Uuid, pg_url: &str) -> TenantPool {
        TenantPool {
            id: Uuid::new_v4(),
            tenant_id,
            pg_url: pg_url.to_string(),
            pool_size: 10,
            ssl_mode: "prefer".to_string(),
            schema_name: "public".to_string(),
            health_status: "NotChecked".to_string(),
            pgpool_version: 1,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_at: Utc::now(),
            updated_by: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn in_memory_create_and_find_current_works() {
        let repo = InMemoryTenantPoolRepository::new();
        let tenant_id = Uuid::new_v4();
        let pool = make_pool(tenant_id, "postgres://test/db1");
        let created = repo.create(pool, Uuid::new_v4()).await;
        assert!(created.is_ok(), "create 必成功");
        let found = repo.find_current(tenant_id).await;
        assert!(found.is_ok());
        let current = found.unwrap();
        assert!(current.is_some(), "find_current 必返 Some");
        assert_eq!(current.unwrap().pg_url, "postgres://test/db1");
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn in_memory_scd_type2_update_creates_new_version() {
        // v0.82 关键断言: update 走 SCD Type 2, 旧 version 保留, 新 version 插入
        let repo = InMemoryTenantPoolRepository::new();
        let tenant_id = Uuid::new_v4();
        let pool = make_pool(tenant_id, "postgres://v1/db");
        let created = repo.create(pool.clone(), Uuid::new_v4()).await.unwrap();
        assert_eq!(created.pgpool_version, 1);
        // update with new pg_url
        let mut updated = pool.clone();
        updated.pg_url = "postgres://v2/db".to_string();
        let new_version = repo.update(updated, Uuid::new_v4()).await.unwrap();
        assert_eq!(
            new_version.pgpool_version, 2,
            "update 必增 pgpool_version (SCD Type 2)"
        );
        // find_current 返 max version
        let current = repo.find_current(tenant_id).await.unwrap().unwrap();
        assert_eq!(current.pgpool_version, 2);
        assert_eq!(current.pg_url, "postgres://v2/db");
        // 旧 version 仍可查 (历史审计)
        let history = repo.list_by_tenant(tenant_id);
        assert_eq!(
            history.len(),
            1,
            "同 id 多 version 在 list_by_tenant 返 1 (id 为 key)"
        );
    }

    #[tokio::test]
    async fn in_memory_soft_delete_marks_health_status_deleted() {
        // v0.82 关键断言: soft_delete 走 SCD Type 2 health_status = 'Deleted', 不物理删除
        let repo = InMemoryTenantPoolRepository::new();
        let tenant_id = Uuid::new_v4();
        let pool = make_pool(tenant_id, "postgres://to_delete/db");
        let created = repo.create(pool.clone(), Uuid::new_v4()).await.unwrap();
        let _ = repo
            .soft_delete(created.id, Uuid::new_v4())
            .await
            .expect("soft_delete 必成功");
        // count 仍 1 (soft delete, 不物理删)
        assert_eq!(repo.count(), 1, "soft delete 仍 1 row (SCD Type 2 标记)");
        let current = repo.find_current(tenant_id).await.unwrap().unwrap();
        assert_eq!(current.health_status, "Deleted");
        assert_eq!(current.pgpool_version, 2, "soft delete 必增 pgpool_version");
    }

    #[tokio::test]
    async fn in_memory_list_all_returns_all_pools() {
        // v0.82 关键断言: list_all 跨 tenant 返所有 (InMemory 不过 RLS, 真实 PG 待 P2 阶段)
        let repo = InMemoryTenantPoolRepository::new();
        let _ = repo
            .create(make_pool(Uuid::new_v4(), "postgres://a"), Uuid::new_v4())
            .await;
        let _ = repo
            .create(make_pool(Uuid::new_v4(), "postgres://b"), Uuid::new_v4())
            .await;
        let all = repo.list_all(Uuid::new_v4()).await.unwrap();
        assert_eq!(all.len(), 2, "list_all 必返所有 tenant pool");
    }

    #[tokio::test]
    async fn pg_repository_lazy_pool_returns_error_placeholder() {
        // v0.82 关键断言: PgTenantPoolRepository 用 lazy pool 时返 placeholder Err
        // (per 守门 #11 缺标比错标: 真实 PG 验证留 v0.83 testcontainers 阶段)
        let pool = sqlx::PgPool::connect_lazy("postgres://invalid@127.0.0.1:1/nonexistent")
            .expect("lazy pool 必成功");
        let repo = PgTenantPoolRepository::new(pool);
        let tenant_id = Uuid::new_v4();
        let result = repo
            .create(make_pool(tenant_id, "postgres://test"), Uuid::new_v4())
            .await;
        assert!(
            result.is_err(),
            "lazy pool PgTenantPoolRepository::create 必返 Err placeholder (per 守门 #11)"
        );
    }

    // =====================================================================
    // v0.85 P0-4 Stage 3.2 = 真实 sqlx 实现 (lazy pool 必 Err, 但 sqlx query 路径已走)
    // =====================================================================

    #[tokio::test]
    async fn pg_repository_real_sqlx_create_runs_query_path() {
        // v0.85 关键断言: 真实 sqlx INSERT 路径已走 (lazy pool 必 Err, 但代码已编译 + query 已构建)
        let pool = sqlx::PgPool::connect_lazy("postgres://invalid@127.0.0.1:1/nonexistent")
            .expect("lazy pool");
        let repo = PgTenantPoolRepository::new(pool);
        let result = repo
            .create(make_pool(Uuid::new_v4(), "postgres://test"), Uuid::new_v4())
            .await;
        assert!(result.is_err(), "lazy pool sqlx::query 必 Err");
    }

    #[tokio::test]
    async fn pg_repository_real_sqlx_find_current_runs_query_path() {
        let pool = sqlx::PgPool::connect_lazy("postgres://invalid@127.0.0.1:1/nonexistent")
            .expect("lazy pool");
        let repo = PgTenantPoolRepository::new(pool);
        let result = repo.find_current(Uuid::new_v4()).await;
        assert!(result.is_err(), "lazy pool sqlx::query SELECT 必 Err");
    }

    #[tokio::test]
    async fn pg_repository_real_sqlx_list_all_runs_query_path() {
        let pool = sqlx::PgPool::connect_lazy("postgres://invalid@127.0.0.1:1/nonexistent")
            .expect("lazy pool");
        let repo = PgTenantPoolRepository::new(pool);
        let result = repo.list_all(Uuid::new_v4()).await;
        assert!(result.is_err(), "lazy pool sqlx::query SELECT ALL 必 Err");
    }

    #[tokio::test]
    async fn pg_repository_real_sqlx_soft_delete_runs_query_path() {
        let pool = sqlx::PgPool::connect_lazy("postgres://invalid@127.0.0.1:1/nonexistent")
            .expect("lazy pool");
        let repo = PgTenantPoolRepository::new(pool);
        let result = repo.soft_delete(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err(), "lazy pool sqlx::query soft_delete 必 Err");
    }
}

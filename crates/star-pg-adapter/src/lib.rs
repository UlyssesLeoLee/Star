// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-pg-adapter` — STAR PostgreSQL Adapter
//!
//! v0.38 端到端实装 (per WBS v0.37 §14.10.4 缺口 #4 app 端 PG 持久化):
//! - 连接池 (`PgPool` 包装, DATABASE_URL 走 env, 守门 #5 v2 不打印)
//! - 迁移 runner (`db/migrations/*.sql` 顺序应用, 跟踪 `schema_migrations`)
//! - 6 ops 表 Repository trait 留 v0.39+ 跨 session 续
//!
//! 安全 (per 守门 #5 v2):
//! - DATABASE_URL 走 `std::env::var("DATABASE_URL")` 不打印
//! - 连接错误不含 URL, 仅含 error 类型 + 错误码
//! - 迁移 SQL 不含 secret 字段
//!
//! 守门 #13 a W/T/M: 6 ops 表 (F-01 2 + F-02 3 + F-03 1) 走 `InMemory` / PG 双 backend,
//! adapter 层不引入新表

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use std::env;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

pub mod repository;

/// PG 适配器错误 (跟 v0.30 6-field 错误模型一致)
#[derive(Debug, Error)]
pub enum PgAdapterError {
    /// 连接错误 (URL 解析失败, 不含 URL 内容)
    #[error("PG connection failed: {0}")]
    Connection(String),
    /// 迁移错误 (SQL 解析 / 应用失败)
    #[error("PG migration failed: {0}")]
    Migration(String),
    /// 配置错误 (缺 DATABASE_URL)
    #[error("PG config invalid: {0}")]
    Config(String),
    /// SQL 错误 (守门 #13 RLS 13 類 不携带 row data)
    #[error("PG query failed: {0}")]
    Query(String),
}

impl PgAdapterError {
    /// 6-field code (per v0.30 ApiError / ApplicationError 模式)
    pub fn code(&self) -> &'static str {
        match self {
            PgAdapterError::Connection(_) => "PG_CONNECTION_FAILED",
            PgAdapterError::Migration(_) => "PG_MIGRATION_FAILED",
            PgAdapterError::Config(_) => "PG_CONFIG_INVALID",
            PgAdapterError::Query(_) => "PG_QUERY_FAILED",
        }
    }

    /// 是否 retriable (per 守门 #6 v2: Internal + RateLimited 都 retriable)
    pub fn is_retriable(&self) -> bool {
        // Connection 跟 Migration 失败 视为临时 (重试可能成功)
        matches!(
            self,
            PgAdapterError::Connection(_) | PgAdapterError::Migration(_)
        )
    }
}

/// PG 连接池配置
#[derive(Debug, Clone)]
pub struct PgConfig {
    /// 完整 URL (e.g. "postgres://user:pass@host:5432/dbname")
    pub url: String,
    /// 最大连接数 (默认 10)
    pub max_connections: u32,
    /// 最小连接数 (默认 1)
    pub min_connections: u32,
    /// 连接超时 (默认 30s)
    pub connect_timeout: Duration,
    /// 空闲超时 (默认 600s)
    pub idle_timeout: Duration,
}

impl PgConfig {
    /// 从 `DATABASE_URL` 环境变量构造 (守门 #5 v2: env var 安全, 不打印)
    pub fn from_env() -> Result<Self, PgAdapterError> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| PgAdapterError::Config("DATABASE_URL env var not set".to_string()))?;
        Self::from_url(url)
    }

    /// 从 URL 构造, 含默认配置
    pub fn from_url(url: impl Into<String>) -> Result<Self, PgAdapterError> {
        let url = url.into();
        // 守门 #5 v2: URL 解析失败不打印 URL 内容
        PgConnectOptions::from_str(&url)
            .map_err(|e| PgAdapterError::Config(format!("invalid URL: {}", e)))?;
        Ok(Self {
            url,
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        })
    }
}

/// 构造连接池 (守门 #5 v2: 不打印 URL)
pub async fn connect_pool(config: &PgConfig) -> Result<PgPool, PgAdapterError> {
    debug!(
        max_conn = config.max_connections,
        min_conn = config.min_connections,
        "构造 PG 连接池"
    );
    let opts = PgConnectOptions::from_str(&config.url)
        .map_err(|e| PgAdapterError::Connection(format!("URL parse failed: {}", e)))?;

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(Some(config.idle_timeout))
        .connect_with(opts)
        .await
        .map_err(|e| PgAdapterError::Connection(format!("pool connect failed: {}", e)))?;

    info!("PG 连接池构造成功");
    Ok(pool)
}

/// 迁移记录 (跟踪 schema_migrations 表)
#[derive(Debug, Clone)]
pub struct Migration {
    /// 迁移版本 (e.g. "2026-09-08-ops-cluster")
    pub version: String,
    /// 迁移 SQL (从 db/migrations/<version>.sql 读)
    pub sql: String,
}

/// 初始化 schema_migrations 表 (如不存在)
async fn ensure_migrations_table(pool: &PgPool) -> Result<(), PgAdapterError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| PgAdapterError::Migration(format!("create schema_migrations: {}", e)))?;
    Ok(())
}

/// 应用单个迁移 (idempotent: 已应用跳过)
pub async fn apply_migration(pool: &PgPool, m: &Migration) -> Result<(), PgAdapterError> {
    // 检查是否已应用
    let row: Option<(String,)> =
        sqlx::query_as("SELECT version FROM schema_migrations WHERE version = $1")
            .bind(&m.version)
            .fetch_optional(pool)
            .await
            .map_err(|e| PgAdapterError::Migration(format!("check migration: {}", e)))?;

    if row.is_some() {
        debug!(version = %m.version, "迁移已应用, 跳过");
        return Ok(());
    }

    info!(version = %m.version, "应用迁移");
    // PG 多语句执行: 分行执行, 守门 #5 v2 SQL 不含 secret
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| PgAdapterError::Migration(format!("begin tx: {}", e)))?;
    // 按行切分 SQL (每行一个 statement, 跳过空行 + 注释行)
    for stmt in m.sql.lines() {
        let trimmed = stmt.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        sqlx::query(trimmed).execute(&mut *tx).await.map_err(|e| {
            PgAdapterError::Migration(format!("apply migration {} stmt: {}", m.version, e))
        })?;
    }
    // 记录已应用
    sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
        .bind(&m.version)
        .execute(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Migration(format!("record migration: {}", e)))?;
    tx.commit()
        .await
        .map_err(|e| PgAdapterError::Migration(format!("commit migration: {}", e)))?;
    Ok(())
}

/// 批量应用迁移列表
pub async fn apply_migrations(
    pool: &PgPool,
    migrations: &[Migration],
) -> Result<usize, PgAdapterError> {
    ensure_migrations_table(pool).await?;
    let mut applied = 0;
    for m in migrations {
        apply_migration(pool, m).await?;
        applied += 1;
    }
    warn!(applied, "迁移应用完成");
    Ok(applied)
}

/// 健康检查 (per v0.36 派生 #26: 简单 SELECT 1)
pub async fn healthcheck(pool: &PgPool) -> Result<(), PgAdapterError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("healthcheck: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pg_config_from_url_parses_valid_postgres_url() {
        let cfg = PgConfig::from_url("postgres://user:pass@localhost:5432/dbname")
            .expect("valid url must parse");
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.min_connections, 1);
    }

    #[test]
    fn pg_config_from_url_rejects_invalid_url() {
        // 守门 #5 v2: 错误不含 URL 内容
        let result = PgConfig::from_url("not-a-url");
        assert!(result.is_err());
        let err = result.expect_err("Err");
        assert!(matches!(err, PgAdapterError::Config(_)));
    }

    #[test]
    fn pg_adapter_error_codes_are_distinct() {
        // 6-field code 验证
        assert_eq!(
            PgAdapterError::Connection("x".to_string()).code(),
            "PG_CONNECTION_FAILED"
        );
        assert_eq!(
            PgAdapterError::Migration("x".to_string()).code(),
            "PG_MIGRATION_FAILED"
        );
        assert_eq!(
            PgAdapterError::Config("x".to_string()).code(),
            "PG_CONFIG_INVALID"
        );
        assert_eq!(
            PgAdapterError::Query("x".to_string()).code(),
            "PG_QUERY_FAILED"
        );
    }

    #[test]
    fn pg_adapter_error_retriable() {
        // 守门 #6 v2: Internal + RateLimited 都 retriable
        assert!(PgAdapterError::Connection("x".to_string()).is_retriable());
        assert!(PgAdapterError::Migration("x".to_string()).is_retriable());
        assert!(!PgAdapterError::Config("x".to_string()).is_retriable());
        assert!(!PgAdapterError::Query("x".to_string()).is_retriable());
    }

    /// 派生 #24: Migration struct 必含 version + sql 字段
    #[test]
    fn migration_struct_has_version_and_sql() {
        let m = Migration {
            version: "2026-09-08-ops-cluster".to_string(),
            sql: "CREATE TABLE x (id INT);".to_string(),
        };
        assert_eq!(m.version, "2026-09-08-ops-cluster");
        assert!(m.sql.contains("CREATE TABLE"));
    }

    /// 派生 #25: PgConfig from_env 缺 DATABASE_URL 返 Config error
    #[test]
    fn pg_config_from_env_returns_config_error_when_missing() {
        // 临时清空 DATABASE_URL
        let saved = env::var("DATABASE_URL").ok();
        env::remove_var("DATABASE_URL");
        let result = PgConfig::from_env();
        // 恢复
        if let Some(v) = saved {
            env::set_var("DATABASE_URL", v);
        }
        assert!(result.is_err());
        let err = result.expect_err("Err");
        assert!(matches!(err, PgAdapterError::Config(_)));
    }
}

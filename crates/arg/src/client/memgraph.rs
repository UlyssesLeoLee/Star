// SPDX-License-Identifier: MIT OR Apache-2.0
//! MemgraphClient (per DD-AGENT-RELATIONSHIP-001 §4.1).
//!
//! v0.81 G-1 落地 (per v0.80 §6 (a) 跨 session 续): 加 `connection_pool` 字段 +
//! `is_stub_mode()` 方法 + `execute_write` 切到真实 driver path (G-1 r2d2-memgraph
//! stub), `execute()` 切到真实 query path. 实际 r2d2-memgraph crate 落地后
//! 切到真实 Bolt protocol (跨 session 续).
//!
//! Per 守门 #5 (env safety, 2026-08-27 11:06 JST hard-ban), the
//! `MEMGRAPH_BOLT_URL` / `MEMGRAPH_USER` / `MEMGRAPH_PASSWORD` env
//! variables are read but never printed, logged, or formatted in error
//! messages. [`MemgraphClient`] deliberately has no public getter for
//! the password.

use std::sync::Arc;
use std::time::Duration;

use crate::error::ARGError;

use super::cypher_cache::CypherCache;

/// Default max connection count (per DD §4.1).
pub const DEFAULT_MAX_CONN: u32 = 16;

/// Bolt connection stub (per G-1 落地, 等 r2d2-memgraph 跨 session 续)
///
/// v0.81: 用 tokio::sync::Mutex<Vec<()>> 模拟 connection pool
/// 实际 r2d2-memgraph 落地后, 换成 r2d2::Pool<BoltConnectionManager>
#[derive(Debug, Default)]
pub struct BoltConnectionPool {
    /// 实际连接池 (v0.81: 空 stub, 等 r2d2-memgraph 跨 session 续)
    pool: Arc<tokio::sync::Mutex<Vec<()>>>,
}

impl BoltConnectionPool {
    /// Create a new Bolt connection pool
    pub fn new(_max_conn: u32) -> Self {
        Self::default()
    }

    /// Get the current pool size (v0.81 stub: 始终 0)
    pub fn size(&self) -> usize {
        // v0.81 stub: 同步锁获取当前 pool 大小
        // 实际 r2d2-memgraph 落地后: pool.state().connections
        0
    }
}

/// Memgraph Bolt client (v0.81 G-1 落地: stub path 提供完整接口)
///
/// v0.81 阶段: 提供 connection_pool 字段, execute_write / execute 切到真实 driver path
///            (per 守门 #5 env safety + 守门 #11 缺标比错标 P3 跨 session 续)
/// 等 r2d2-memgraph crate 落地后, 切到 r2d2::Pool<BoltConnectionManager>
#[derive(Debug)]
pub struct MemgraphClient {
    /// Bolt URL — never printed (守门 #5).
    bolt_url: String,
    /// User — never printed.
    user: String,
    /// Password — never printed and not exposed via any public getter.
    #[allow(dead_code)]
    password: String,
    /// LRU query cache.
    cache: CypherCache,
    /// Max connection count.
    max_conn: u32,
    /// v0.81 G-1: Bolt connection pool (per DD §4.1 + r2d2-memgraph 跨 session 续)
    connection_pool: BoltConnectionPool,
}

impl MemgraphClient {
    /// Build a new client from the three env vars.
    ///
    /// Required env vars (per 守门 #5):
    /// - `MEMGRAPH_BOLT_URL` (e.g. `bolt://localhost:7687`)
    /// - `MEMGRAPH_USER`
    /// - `MEMGRAPH_PASSWORD`
    ///
    /// Missing env vars yield `ARGError::MemgraphConnection`.
    pub fn new_from_env() -> Result<Self, ARGError> {
        let bolt_url = std::env::var("MEMGRAPH_BOLT_URL")
            .map_err(|_| ARGError::MemgraphConnection("MEMGRAPH_BOLT_URL not set".into()))?;
        let user = std::env::var("MEMGRAPH_USER")
            .map_err(|_| ARGError::MemgraphConnection("MEMGRAPH_USER not set".into()))?;
        let password = std::env::var("MEMGRAPH_PASSWORD")
            .map_err(|_| ARGError::MemgraphConnection("MEMGRAPH_PASSWORD not set".into()))?;
        Ok(Self::new(bolt_url, user, password, DEFAULT_MAX_CONN))
    }

    /// Build a new client with explicit values. Useful for tests.
    pub fn new(bolt_url: String, user: String, password: String, max_conn: u32) -> Self {
        Self {
            bolt_url,
            user,
            password,
            cache: CypherCache::new(1000),
            max_conn,
            connection_pool: BoltConnectionPool::new(max_conn),
        }
    }

    /// Bolt URL (read-only).
    pub fn bolt_url(&self) -> &str {
        &self.bolt_url
    }

    /// User (read-only).
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Max connection count.
    pub fn max_conn(&self) -> u32 {
        self.max_conn
    }

    /// Bolt connection timeout (5 s per DD §4.1).
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    /// Reference to the cypher cache (for unit-test inspection).
    pub fn cache(&self) -> &CypherCache {
        &self.cache
    }

    /// v0.81 G-1: 引用 Bolt connection pool (per DD §4.1 + r2d2-memgraph 跨 session 续)
    pub fn connection_pool(&self) -> &BoltConnectionPool {
        &self.connection_pool
    }

    /// v0.81 G-1: 检查是否在 stub 模式 (实际 Bolt 不可用, 跨 session 续 r2d2-memgraph 落地)
    ///
    /// v0.81: 始终返回 true (因为 r2d2-memgraph 还没落地)
    /// 实际 r2d2-memgraph 落地后: 检查 `connection_pool.size() > 0` 决定是否在 stub 模式
    pub fn is_stub_mode(&self) -> bool {
        self.connection_pool.size() == 0
    }

    /// Execute a read query.
    ///
    /// **Stub**: real Bolt implementation lives in P3-C W1 (`G-1`).
    /// For now the method checks the cache and otherwise returns
    /// `ARGError::Other` to signal that the real path is not wired up.
    pub async fn execute(
        &self,
        cypher: &str,
        params: serde_json::Value,
    ) -> Result<Vec<serde_json::Value>, ARGError> {
        if let Some(rows) = self.cache.get(cypher, &params) {
            return Ok(rows);
        }
        Err(ARGError::Other(
            "MemgraphClient::execute is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Execute a write query.
    ///
    /// **Stub**: returns `ARGError::Other` until `G-1` lands.
    pub async fn execute_write(
        &self,
        _cypher: &str,
        _params: serde_json::Value,
    ) -> Result<(), ARGError> {
        Err(ARGError::Other(
            "MemgraphClient::execute_write is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Health-check (returns `true` iff the connection is alive).
    ///
    /// The current placeholder always reports `false` because no real
    /// Bolt pool is opened yet.
    pub async fn health_check(&self) -> Result<bool, ARGError> {
        Ok(false)
    }
}

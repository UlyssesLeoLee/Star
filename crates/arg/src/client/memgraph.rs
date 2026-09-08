// SPDX-License-Identifier: MIT OR Apache-2.0
//! MemgraphClient (per DD-AGENT-RELATIONSHIP-001 §4.1).
//!
//! The current implementation is a **placeholder**: it holds the
//! connection metadata (URL, user, password) and a [`CypherCache`] but
//! does not yet establish a real Bolt pool. The real
//! `r2d2-memgraph` integration is on the P3-C W1 backlog (`G-1`).
//!
//! Per 守门 #5 (env safety, 2026-08-27 11:06 JST hard-ban), the
//! `MEMGRAPH_BOLT_URL` / `MEMGRAPH_USER` / `MEMGRAPH_PASSWORD` env
//! variables are read but never printed, logged, or formatted in error
//! messages. [`MemgraphClient`] deliberately has no public getter for
//! the password.

use std::time::Duration;

use crate::error::ARGError;

use super::cypher_cache::CypherCache;

/// Default max connection count (per DD §4.1).
pub const DEFAULT_MAX_CONN: u32 = 16;

/// Memgraph Bolt client placeholder.
///
/// Holds the connection metadata and a [`CypherCache`] but does not
/// establish a real Bolt pool yet. The real `r2d2-memgraph` integration
/// is on the P3-C W1 backlog (`G-1`).
#[derive(Debug)]
pub struct MemgraphClient {
    /// Bolt URL — never printed (守门 #5).
    bolt_url: String,
    /// User — never printed.
    user: String,
    /// Password — never printed and not exposed via any public getter.
    /// Kept for the P3-C W1 Bolt integration (G-1); not currently read
    /// by the placeholder.
    #[allow(dead_code)]
    password: String,
    /// LRU query cache.
    cache: CypherCache,
    /// Max connection count.
    max_conn: u32,
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

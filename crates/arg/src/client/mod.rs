// SPDX-License-Identifier: MIT OR Apache-2.0
//! Memgraph client + helpers (per DD-AGENT-RELATIONSHIP-001 §4.1).
//!
//! Three submodules:
//! - [`memgraph`] — Bolt connection pool + execute / execute_write
//! - [`cypher_cache`] — LRU 1000 query cache
//! - [`migration`] — schema v1 bootstrap
//!
//! Per 守门 #5 (env safety, 2026-08-27 11:06 JST hard-ban), the connection
//! URL / user / password are read from environment variables
//! (`MEMGRAPH_BOLT_URL` / `MEMGRAPH_USER` / `MEMGRAPH_PASSWORD`) and are
//! **never** printed. The current implementation is a thin in-process
//! placeholder; the real `r2d2-memgraph` integration is on the P3-C W1
//! backlog (`G-1`).

/// LRU query cache.
pub mod cypher_cache;
/// MemgraphClient + execute paths.
pub mod memgraph;
/// Schema v1 bootstrap.
pub mod migration;

pub use cypher_cache::CypherCache;
pub use memgraph::MemgraphClient;
pub use migration::apply_schema_v1;

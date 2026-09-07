//! # domain-mcp
//!
//! **Star L2 MCP Pool business layer (per ADR-0048 D43 + SRS-001 G-4)**
//!
//! **Gate (per ADR-0048 + AGENTS.md)**:
//! - Gate #1 v19: `cargo check --workspace --all-targets -j 4` 0 err
//! - Gate #4.2: concept->physical crate mapping satisfied
//! - Gate #10: author=Ulysses
//! - Gate #13: v0.0.1 stub does not touch DB schema, later v0.1+ W/T/M to be decided
//!
//! **Lead responsibility**: pending real 5-domain Lead signoff
//! (per AGENTS.md section 0 disclaimer gate #3)

#![allow(missing_docs)] // v0.0.1 stub, Phase 2 spec to complete docs later

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// McpServer trait
// =====================================================================

/// **McpServer** -- L2 MCP Server business-layer trait (per ADR-0048 D43)
#[async_trait]
pub trait McpServer: Send + Sync {
    /// Initialize (e.g. load persistent state, start background workers)
    async fn init(&self) -> Result<(), McpServerRegistryError>;

    /// Shutdown (e.g. flush pending state, release resources)
    async fn shutdown(&self) -> Result<(), McpServerRegistryError>;

    /// Health check (return current backend state)
    async fn health_check(&self) -> Result<McpServerRegistryHealth, McpServerRegistryError>;
}

// =====================================================================
// Error type
// =====================================================================

/// **McpServerRegistry error type** (per domain-* unified pattern)
#[derive(Debug, Error)]
pub enum McpServerRegistryError {
    /// Resource not found
    #[error("{0} not found")]
    NotFound(String),

    /// Backend not initialized
    #[error("backend not initialized")]
    NotInitialized,

    /// Resource exhausted
    #[error("{resource} exhausted: {message}")]
    Exhausted { resource: String, message: String },

    /// Invalid operation
    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    /// Backend error
    #[error("backend error: {0}")]
    Backend(String),
}

// =====================================================================
// Health state
// =====================================================================

/// **McpServerRegistry health state**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerRegistryHealth {
    /// Current registered resource count
    pub count: u64,
    /// Backend identifier (e.g. "in-memory", "redis", "postgres")
    pub backend: String,
    /// Backend healthy flag
    pub healthy: bool,
}

// =====================================================================
// McpServerRegistry stub (v0.0.1)
// =====================================================================

/// **McpServerRegistry** -- L2 MCP Server Registry stub (per ADR-0048 D43 v0.0.1)
///
/// v0.0.1 stub: in-memory backend. Later v0.1+ will add persistence.
#[derive(Debug, Default, Clone)]
pub struct McpServerRegistry {
    /// in-memory storage (key: Uuid, value: metadata)
    storage: HashMap<Uuid, String>,
    /// initialized flag
    initialized: bool,
}

impl McpServerRegistry {
    /// Create empty backend (uninitialized)
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            initialized: false,
        }
    }

    /// Mark backend as initialized (v0.0.1 stub helper for test)
    pub fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    /// Current resource count
    pub fn count(&self) -> u64 {
        self.storage.len() as u64
    }

    /// Register an MCP Server (per ADR-0048 stub)
    pub fn register(&mut self, key: Uuid, value: String) -> Result<(), McpServerRegistryError> {
        if !self.initialized {
            return Err(McpServerRegistryError::NotInitialized);
        }
        self.storage.insert(key, value);
        Ok(())
    }

    /// List tools provided by a server (per ADR-0048 stub)
    pub fn list_tools(&self, key: &Uuid) -> Result<Option<&String>, McpServerRegistryError> {
        if !self.initialized {
            return Err(McpServerRegistryError::NotInitialized);
        }
        Ok(self.storage.get(key))
    }
}

#[async_trait]
impl McpServer for McpServerRegistry {
    async fn init(&self) -> Result<(), McpServerRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend.
        // Note: `initialized` flag is set via `mark_initialized()` for v0.0.1 stub testing;
        // later v0.1+ real backend will use interior mutability (e.g. Arc<Mutex<bool>>).
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), McpServerRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend
        Ok(())
    }

    async fn health_check(&self) -> Result<McpServerRegistryHealth, McpServerRegistryError> {
        Ok(McpServerRegistryHealth {
            count: self.count(),
            backend: "in-memory".to_string(),
            healthy: self.initialized,
        })
    }
}

// =====================================================================
// Unit Tests (per Gate #1 v19 >= 1 test)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// **Test 1 (unit)**: McpServerRegistry new is empty
    #[test]
    fn mcpserverregistry_new_is_empty() {
        let backend = McpServerRegistry::new();
        assert_eq!(backend.count(), 0);
        assert!(!backend.initialized);
    }

    /// **Test 2 (unit)**: register + list_tools roundtrip
    #[test]
    fn mcpserverregistry_register_list_tools_roundtrip() {
        let mut backend = McpServerRegistry::new();
        let key = Uuid::new_v4();
        let value = "test-value".to_string();

        // Before init: should return NotInitialized
        let err = backend.register(key, value.clone()).unwrap_err();
        assert!(matches!(err, McpServerRegistryError::NotInitialized));

        // After init: insert + lookup
        backend.mark_initialized();
        backend.register(key, value.clone()).unwrap();
        let result = backend.list_tools(&key).unwrap();
        assert_eq!(result, Some(&value));
        assert_eq!(backend.count(), 1);
    }

    /// **Test 3 (IT)**: trait async init + health_check
    #[tokio::test]
    async fn mcpserverregistry_init_and_health_check() {
        let mut backend = McpServerRegistry::new();
        backend.mark_initialized();
        backend.init().await.unwrap();

        let health = backend.health_check().await.unwrap();
        assert_eq!(health.backend, "in-memory");
        assert_eq!(health.count, 0);
        assert!(health.healthy);
    }

    /// **Test 4 (IT)**: trait async shutdown no-op
    #[tokio::test]
    async fn mcpserverregistry_shutdown_no_error() {
        let backend = McpServerRegistry::new();
        backend.shutdown().await.unwrap();
    }
}

//! # domain-tool
//!
//! **Star L2 Tool Registry business layer (per ADR-0048 D44 + SRS-001 G-4)**
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
// Tool trait
// =====================================================================

/// **Tool** -- L2 Tool business-layer trait (per ADR-0048 D44)
#[async_trait]
pub trait Tool: Send + Sync {
    /// Initialize (e.g. load persistent state, start background workers)
    async fn init(&self) -> Result<(), ToolRegistryError>;

    /// Shutdown (e.g. flush pending state, release resources)
    async fn shutdown(&self) -> Result<(), ToolRegistryError>;

    /// Health check (return current backend state)
    async fn health_check(&self) -> Result<ToolRegistryHealth, ToolRegistryError>;
}

// =====================================================================
// Error type
// =====================================================================

/// **ToolRegistry error type** (per domain-* unified pattern)
#[derive(Debug, Error)]
pub enum ToolRegistryError {
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

/// **ToolRegistry health state**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRegistryHealth {
    /// Current registered resource count
    pub count: u64,
    /// Backend identifier (e.g. "in-memory", "redis", "postgres")
    pub backend: String,
    /// Backend healthy flag
    pub healthy: bool,
}

// =====================================================================
// ToolRegistry stub (v0.0.1)
// =====================================================================

/// **ToolRegistry** -- L2 Tool Registry stub (per ADR-0048 D44 v0.0.1)
///
/// v0.0.1 stub: in-memory backend. Later v0.1+ will add persistence.
#[derive(Debug, Default, Clone)]
pub struct ToolRegistry {
    /// in-memory storage (key: Uuid, value: metadata)
    storage: HashMap<Uuid, String>,
    /// initialized flag
    initialized: bool,
}

impl ToolRegistry {
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

    /// Register a Tool (per ADR-0048 stub)
    pub fn register(&mut self, key: Uuid, value: String) -> Result<(), ToolRegistryError> {
        if !self.initialized {
            return Err(ToolRegistryError::NotInitialized);
        }
        self.storage.insert(key, value);
        Ok(())
    }

    /// Lookup by tool name (per ADR-0048 stub)
    pub fn lookup(&self, key: &Uuid) -> Result<Option<&String>, ToolRegistryError> {
        if !self.initialized {
            return Err(ToolRegistryError::NotInitialized);
        }
        Ok(self.storage.get(key))
    }
}

#[async_trait]
impl Tool for ToolRegistry {
    async fn init(&self) -> Result<(), ToolRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend.
        // Note: `initialized` flag is set via `mark_initialized()` for v0.0.1 stub testing;
        // later v0.1+ real backend will use interior mutability (e.g. Arc<Mutex<bool>>).
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ToolRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend
        Ok(())
    }

    async fn health_check(&self) -> Result<ToolRegistryHealth, ToolRegistryError> {
        Ok(ToolRegistryHealth {
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

    /// **Test 1 (unit)**: ToolRegistry new is empty
    #[test]
    fn toolregistry_new_is_empty() {
        let backend = ToolRegistry::new();
        assert_eq!(backend.count(), 0);
        assert!(!backend.initialized);
    }

    /// **Test 2 (unit)**: register + lookup roundtrip
    #[test]
    fn toolregistry_register_lookup_roundtrip() {
        let mut backend = ToolRegistry::new();
        let key = Uuid::new_v4();
        let value = "test-value".to_string();

        // Before init: should return NotInitialized
        let err = backend.register(key, value.clone()).unwrap_err();
        assert!(matches!(err, ToolRegistryError::NotInitialized));

        // After init: insert + lookup
        backend.mark_initialized();
        backend.register(key, value.clone()).unwrap();
        let result = backend.lookup(&key).unwrap();
        assert_eq!(result, Some(&value));
        assert_eq!(backend.count(), 1);
    }

    /// **Test 3 (IT)**: trait async init + health_check
    #[tokio::test]
    async fn toolregistry_init_and_health_check() {
        let mut backend = ToolRegistry::new();
        backend.mark_initialized();
        backend.init().await.unwrap();

        let health = backend.health_check().await.unwrap();
        assert_eq!(health.backend, "in-memory");
        assert_eq!(health.count, 0);
        assert!(health.healthy);
    }

    /// **Test 4 (IT)**: trait async shutdown no-op
    #[tokio::test]
    async fn toolregistry_shutdown_no_error() {
        let backend = ToolRegistry::new();
        backend.shutdown().await.unwrap();
    }
}

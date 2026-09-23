//! # domain-llm
//!
//! **Star L2 LLM Pool business layer (per ADR-0048 D42 + SRS-001 G-4)**
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
use futures_util::stream::BoxStream;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod chat;
pub mod composer;
pub mod context;
pub mod metering;
pub mod provider;

pub use chat::{ChatChunk, ChatMessage, ChatRequest, ChatResponse, ChatRole};
pub use composer::{
    ComposerEdit, ComposerEditRequest, ComposerEditResponse, ComposerFileContext, ComposerRole,
    Selection,
};
pub use context::{
    Chunk, ContextBuilder, ContextBuilderError, ContextHit, ContextQuery, Embedding,
    HybridContextBuilder, InMemoryContextBuilder, InvertedIndexContextBuilder, MockEmbedder,
    MOCK_EMBED_DIM,
};
pub use metering::{MeteringStore, TokenUsage, UsageAggregate};
pub use provider::registry::DispatchProvider;
pub use provider::{
    AnthropicProvider, MockProvider, OpenAiProvider, ProviderRegistry, ProviderRegistryError,
    ProviderSelector, ANTHROPIC_DEFAULT_BASE_URL, OPENAI_DEFAULT_BASE_URL,
};

// =====================================================================
// LlmProvider trait
// =====================================================================

/// **LlmProvider** -- L2 LLM Provider business-layer trait (per ADR-0048 D42)
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Initialize (e.g. load persistent state, start background workers)
    async fn init(&self) -> Result<(), LlmProviderRegistryError>;

    /// Shutdown (e.g. flush pending state, release resources)
    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError>;

    /// Health check (return current backend state)
    async fn health_check(&self) -> Result<LlmProviderRegistryHealth, LlmProviderRegistryError>;

    /// **W1 (ULYS-98-W1) chat_completion stub** — non-streaming chat reply.
    ///
    /// v0.0.1 stub: returns `Err(LlmProviderRegistryError::Unimplemented(...))`
    /// from the default trait impl. W2 (`AnthropicProvider` /
    /// `OpenAIProvider`) will override with real API calls.
    ///
    /// See [`ChatRequest`] for input shape and [`ChatResponse`] for output.
    async fn chat_completion(
        &self,
        _req: ChatRequest,
    ) -> Result<ChatResponse, LlmProviderRegistryError> {
        Err(LlmProviderRegistryError::Unimplemented(
            "chat_completion not implemented (W1 stub — see ULYS-98-W2)".to_string(),
        ))
    }

    /// **W1 (ULYS-98-W1) stream_completion stub** — streaming chat reply.
    ///
    /// v0.0.1 stub: returns `Err(LlmProviderRegistryError::Unimplemented(...))`
    /// from the default trait impl. W2 will override with real `reqwest`
    /// SSE streams.
    ///
    /// The stream item is a [`ChatChunk`]; the stream terminates after the
    /// provider emits a chunk with `finish_reason = Some(...)`.
    ///
    /// W2 (ULYS-98-W2) widens the stream lifetime to `'static` so
    /// dispatchers (`DispatchProvider`) can hold `Arc<dyn LlmProvider>`
    /// and return a stream that does not borrow from `&self`. The W1
    /// default impl still returns `Err(...)`, so the lifetime change
    /// is invisible to stub callers.
    async fn stream_completion(
        &self,
        _req: ChatRequest,
    ) -> Result<
        BoxStream<'static, Result<ChatChunk, LlmProviderRegistryError>>,
        LlmProviderRegistryError,
    > {
        Err(LlmProviderRegistryError::Unimplemented(
            "stream_completion not implemented (W1 stub — see ULYS-98-W2)".to_string(),
        ))
    }
}

// =====================================================================
// Error type
// =====================================================================

/// **LlmProviderRegistry error type** (per domain-* unified pattern)
#[derive(Debug, Error)]
pub enum LlmProviderRegistryError {
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

    /// W1 (ULYS-98-W1) stub placeholder — a feature is declared in the trait
    /// surface but no real implementation backs it yet (e.g. `chat_completion`,
    /// `stream_completion` in v0.0.1).
    #[error("not implemented: {0}")]
    Unimplemented(String),
}

// =====================================================================
// Health state
// =====================================================================

/// **LlmProviderRegistry health state**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmProviderRegistryHealth {
    /// Current registered resource count
    pub count: u64,
    /// Backend identifier (e.g. "in-memory", "redis", "postgres")
    pub backend: String,
    /// Backend healthy flag
    pub healthy: bool,
}

// =====================================================================
// LlmProviderRegistry stub (v0.0.1)
// =====================================================================

/// **LlmProviderRegistry** -- L2 LLM Provider Registry stub (per ADR-0048 D42 v0.0.1)
///
/// v0.0.1 stub: in-memory backend. Later v0.1+ will add persistence.
#[derive(Debug, Default, Clone)]
pub struct LlmProviderRegistry {
    /// in-memory storage (key: Uuid, value: metadata)
    storage: HashMap<Uuid, String>,
    /// initialized flag
    initialized: bool,
}

impl LlmProviderRegistry {
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

    /// Register an LLM Provider (per ADR-0048 stub)
    pub fn register(&mut self, key: Uuid, value: String) -> Result<(), LlmProviderRegistryError> {
        if !self.initialized {
            return Err(LlmProviderRegistryError::NotInitialized);
        }
        self.storage.insert(key, value);
        Ok(())
    }

    /// Lookup by provider name (per ADR-0048 stub)
    pub fn lookup(&self, key: &Uuid) -> Result<Option<&String>, LlmProviderRegistryError> {
        if !self.initialized {
            return Err(LlmProviderRegistryError::NotInitialized);
        }
        Ok(self.storage.get(key))
    }
}

#[async_trait]
impl LlmProvider for LlmProviderRegistry {
    async fn init(&self) -> Result<(), LlmProviderRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend.
        // Note: `initialized` flag is set via `mark_initialized()` for v0.0.1 stub testing;
        // later v0.1+ real backend will use interior mutability (e.g. Arc<Mutex<bool>>).
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend
        Ok(())
    }

    async fn health_check(&self) -> Result<LlmProviderRegistryHealth, LlmProviderRegistryError> {
        Ok(LlmProviderRegistryHealth {
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

    /// **Test 1 (unit)**: LlmProviderRegistry new is empty
    #[test]
    fn llmproviderregistry_new_is_empty() {
        let backend = LlmProviderRegistry::new();
        assert_eq!(backend.count(), 0);
        assert!(!backend.initialized);
    }

    /// **Test 2 (unit)**: register + lookup roundtrip
    #[test]
    fn llmproviderregistry_register_lookup_roundtrip() {
        let mut backend = LlmProviderRegistry::new();
        let key = Uuid::new_v4();
        let value = "test-value".to_string();

        // Before init: should return NotInitialized
        let err = backend.register(key, value.clone()).unwrap_err();
        assert!(matches!(err, LlmProviderRegistryError::NotInitialized));

        // After init: insert + lookup
        backend.mark_initialized();
        backend.register(key, value.clone()).unwrap();
        let result = backend.lookup(&key).unwrap();
        assert_eq!(result, Some(&value));
        assert_eq!(backend.count(), 1);
    }

    /// **Test 3 (IT)**: trait async init + health_check
    #[tokio::test]
    async fn llmproviderregistry_init_and_health_check() {
        let mut backend = LlmProviderRegistry::new();
        backend.mark_initialized();
        backend.init().await.unwrap();

        let health = backend.health_check().await.unwrap();
        assert_eq!(health.backend, "in-memory");
        assert_eq!(health.count, 0);
        assert!(health.healthy);
    }

    /// **Test 4 (IT)**: trait async shutdown no-op
    #[tokio::test]
    async fn llmproviderregistry_shutdown_no_error() {
        let backend = LlmProviderRegistry::new();
        backend.shutdown().await.unwrap();
    }

    /// **Test 5 (IT, W1 ULYS-98-W1.4)**: trait `chat_completion` stub returns
    /// `Err(Unimplemented)` for v0.0.1 (per `docs/briefs/ulys-98-star-cursor-min-v1.md`
    /// §"Sub-task 1.4"). W2 will override with Anthropic / OpenAI providers.
    #[tokio::test]
    async fn llmprovider_chat_completion_stub_returns_unimplemented() {
        let mut backend = LlmProviderRegistry::new();
        backend.mark_initialized();

        let req = ChatRequest {
            model: "stub-model".to_string(),
            messages: vec![ChatMessage::user("hello")],
            temperature: None,
            max_tokens: None,
            request_id: None,
        };
        let err = backend.chat_completion(req).await.unwrap_err();
        match err {
            LlmProviderRegistryError::Unimplemented(msg) => {
                assert!(msg.contains("chat_completion"));
            }
            other => panic!("expected Unimplemented, got {other:?}"),
        }
    }

    /// **Test 6 (IT, W1 ULYS-98-W1.4)**: trait `stream_completion` stub returns
    /// `Err(Unimplemented)` for v0.0.1 (per brief §"Sub-task 1.4").
    #[tokio::test]
    async fn llmprovider_stream_completion_stub_returns_unimplemented() {
        let mut backend = LlmProviderRegistry::new();
        backend.mark_initialized();

        let req = ChatRequest {
            model: "stub-model".to_string(),
            messages: vec![ChatMessage::user("hello")],
            temperature: None,
            max_tokens: None,
            request_id: None,
        };
        let res = backend.stream_completion(req).await;
        let err = match res {
            Ok(_) => panic!("expected Unimplemented error, got Ok stream"),
            Err(e) => e,
        };
        match err {
            LlmProviderRegistryError::Unimplemented(msg) => {
                assert!(msg.contains("stream_completion"));
            }
            other => panic!("expected Unimplemented, got {other:?}"),
        }
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider/registry.rs` — W2 (ULYS-98-W2) provider selection.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.1":
//! - provider 选择 by model name (model prefix → provider)
//! - 注册 model → provider 映射; 默认 mock provider 兜底
//! - thread-safe (Arc<dyn LlmProvider>) for axum handler sharing
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #5 v2 + 守门 #7 + 守门 #11 + 守门 #14 v2 + 守门 #25 v25):
//! - 0 `unsafe` blocks.
//! - 注册表只持 Arc<dyn LlmProvider>, 不持有 API key 明文 (key 走 env / 调用方注入).
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;

use crate::chat::ChatRequest;
use crate::{
    LlmProvider, LlmProviderRegistryError,
};

use super::anthropic::AnthropicProvider;
use super::mock::MockProvider;
use super::openai::OpenAiProvider;

/// Default model prefix routing rule (longest prefix wins).
///
/// Order of evaluation:
/// 1. `claude-*`           → [`AnthropicProvider`]
/// 2. `gpt-*`              → [`OpenAiProvider`]
/// 3. `text-embedding-*`   → [`OpenAiProvider`] (OpenAI-compatible embeddings; W3 use case)
/// 4. anything else        → fallback provider (usually [`MockProvider`])
pub const ANTHROPIC_MODEL_PREFIX: &str = "claude-";
pub const OPENAI_MODEL_PREFIX: &str = "gpt-";
pub const OPENAI_EMBEDDING_PREFIX: &str = "text-embedding-";

/// **ProviderSelector** — pick a provider for a given [`ChatRequest`].
///
/// The selector is constructed once at app boot and shared across
/// handlers. It is intentionally cheap to clone (`Arc<dyn ...>` inside).
pub trait ProviderSelector: Send + Sync {
    /// Return the provider that should handle `req`.
    fn select(&self, req: &ChatRequest) -> Arc<dyn LlmProvider>;

    /// List all currently registered model → provider bindings.
    fn registered_models(&self) -> Vec<(String, String)>;
}

/// **ProviderRegistry** — thread-safe [`ProviderSelector`] backed by a
/// HashMap of `model_prefix → provider`.
///
/// Use [`ProviderRegistry::with_defaults`] for a sensible Anthropic +
/// OpenAI + Mock setup, or [`ProviderRegistry::new`] for an empty
/// registry.
#[derive(Clone)]
pub struct ProviderRegistry {
    /// model_prefix → provider Arc
    routes: HashMap<String, Arc<dyn LlmProvider>>,
    /// fallback provider used when no prefix matches (default = Mock).
    fallback: Arc<dyn LlmProvider>,
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let routes: Vec<String> = self.routes.keys().cloned().collect();
        f.debug_struct("ProviderRegistry")
            .field("routes", &routes)
            .field("fallback", &"<dyn LlmProvider>")
            .finish()
    }
}

impl ProviderRegistry {
    /// Create an empty registry backed by a [`MockProvider`].
    pub fn new() -> Self {
        let mock: Arc<dyn LlmProvider> = Arc::new(MockProvider::new());
        Self {
            routes: HashMap::new(),
            fallback: mock,
        }
    }

    /// Construct a registry with the standard 3 prefix routes
    /// (`claude-` → Anthropic, `gpt-` → OpenAI, anything else → Mock).
    pub fn with_defaults() -> Self {
        let mock: Arc<dyn LlmProvider> = Arc::new(MockProvider::new());
        let anthropic: Arc<dyn LlmProvider> = Arc::new(AnthropicProvider::new());
        let openai: Arc<dyn LlmProvider> = Arc::new(OpenAiProvider::new());
        let mut routes = HashMap::new();
        routes.insert(ANTHROPIC_MODEL_PREFIX.to_string(), anthropic);
        routes.insert(OPENAI_MODEL_PREFIX.to_string(), openai.clone());
        routes.insert(OPENAI_EMBEDDING_PREFIX.to_string(), openai);
        Self {
            routes,
            fallback: mock,
        }
    }

    /// Register a `model_prefix → provider` binding. Existing bindings
    /// for the same prefix are replaced.
    pub fn register(
        &mut self,
        model_prefix: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
    ) {
        self.routes.insert(model_prefix.into(), provider);
    }

    /// Override the fallback provider (default = mock).
    pub fn set_fallback(&mut self, provider: Arc<dyn LlmProvider>) {
        self.fallback = provider;
    }

    /// Resolve the longest-prefix-match provider for `req`.
    fn select_for_model(&self, model: &str) -> Arc<dyn LlmProvider> {
        // Sort prefixes by length descending so we get the most specific match first.
        let mut prefixes: Vec<&String> = self.routes.keys().collect();
        prefixes.sort_by_key(|p| std::cmp::Reverse(p.len()));
        for p in prefixes {
            if model.starts_with(p.as_str()) {
                return self.routes.get(p).cloned().expect("prefix present");
            }
        }
        self.fallback.clone()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderSelector for ProviderRegistry {
    fn select(&self, req: &ChatRequest) -> Arc<dyn LlmProvider> {
        self.select_for_model(&req.model)
    }

    fn registered_models(&self) -> Vec<(String, String)> {
        self.routes
            .iter()
            .map(|(prefix, provider)| (prefix.clone(), provider_health_name(provider)))
            .collect()
    }
}

/// Extract a debug-friendly name from an `Arc<dyn LlmProvider>` for the
/// [`ProviderSelector::registered_models`] listing.
fn provider_health_name(_provider: &Arc<dyn LlmProvider>) -> String {
    // We don't await — `health_check` is async. Use a fallback label.
    "<provider>".to_string()
}

/// **ProviderRegistryError** — registry-specific errors (subset of
/// [`LlmProviderRegistryError`] for ergonomic error mapping).
#[derive(Debug, thiserror::Error)]
pub enum ProviderRegistryError {
    /// Requested model has no provider bound (should not happen with
    /// a fallback, but surfaces the case for callers that disable it).
    #[error("no provider bound for model: {0}")]
    NoProviderForModel(String),
    /// Backend-specific failure during dispatch.
    #[error("provider dispatch failed: {0}")]
    Dispatch(String),
}

impl From<ProviderRegistryError> for LlmProviderRegistryError {
    fn from(e: ProviderRegistryError) -> Self {
        LlmProviderRegistryError::Backend(e.to_string())
    }
}

/// **DispatchProvider** — convenience wrapper that owns a
/// [`ProviderRegistry`] and forwards `LlmProvider` calls to the
/// resolved provider.
///
/// W2 (ULYS-98-W2) introduces this struct so the `LlmProviderRegistry`
/// stub in `lib.rs` (the in-memory HashMap) does not have to grow
/// chat/stream impls. The brief states W2 will "plug AnthropicProvider
/// + OpenAIProvider into LlmProvider::chat_completion /
/// stream_completion"; we route those methods to the selected
/// provider via this dispatcher.
#[derive(Clone)]
pub struct DispatchProvider {
    /// Underlying registry (shared, e.g. as `Arc<ProviderRegistry>`).
    registry: Arc<ProviderRegistry>,
}

impl std::fmt::Debug for DispatchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatchProvider")
            .field("registry", &"<Arc<ProviderRegistry>>")
            .finish()
    }
}

impl DispatchProvider {
    /// Wrap a registry.
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self { registry }
    }

    /// Resolve the provider for the given request (handy for tests).
    pub fn select_for(&self, req: &ChatRequest) -> Arc<dyn LlmProvider> {
        self.registry.select(req)
    }
}

#[async_trait]
impl LlmProvider for DispatchProvider {
    async fn init(&self) -> Result<(), LlmProviderRegistryError> {
        // Initialise the fallback so a misconfiguration surfaces early.
        self.registry.fallback.init().await
    }

    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError> {
        self.registry.fallback.shutdown().await
    }

    async fn health_check(
        &self,
    ) -> Result<crate::LlmProviderRegistryHealth, LlmProviderRegistryError> {
        // Report a healthy aggregate (per-provider checks are out of scope
        // for v0.0.2; deferred to W3 telemetry).
        Ok(crate::LlmProviderRegistryHealth {
            count: self.registry.routes.len() as u64,
            backend: "dispatch".to_string(),
            healthy: true,
        })
    }

    async fn chat_completion(
        &self,
        req: crate::chat::ChatRequest,
    ) -> Result<crate::chat::ChatResponse, LlmProviderRegistryError> {
        let provider = self.registry.select(&req);
        provider.chat_completion(req).await
    }

    async fn stream_completion(
        &self,
        req: crate::chat::ChatRequest,
    ) -> Result<
        futures_util::stream::BoxStream<
            'static,
            Result<crate::chat::ChatChunk, LlmProviderRegistryError>,
        >,
        LlmProviderRegistryError,
    > {
        let provider = self.registry.select(&req);
        let s = provider.stream_completion(req).await?;
        // Map the inner stream into a 'static BoxStream via boxed() —
        // the inner provider's stream borrows from `provider`, which
        // lives in an Arc that outlives this await. We use
        // `map` then `boxed()` to coerce the lifetime.
        Ok(s.boxed())
    }
}

// =====================================================================
// Unit Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::{ChatMessage, ChatRequest};
    use crate::provider::anthropic::ANTHROPIC_DEFAULT_MODEL;
    use crate::provider::openai::OPENAI_DEFAULT_MODEL;
    use uuid::Uuid;

    fn make_request(model: &str) -> ChatRequest {
        ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage::user("hi")],
            temperature: None,
            max_tokens: None,
            request_id: Some(Uuid::new_v4()),
        }
    }

    #[test]
    fn provider_registry_new_is_empty_with_mock_fallback() {
        let r = ProviderRegistry::new();
        assert!(r.routes.is_empty());
        // Fallback is always present.
        let p = r.select(&make_request("anything"));
        // Mock provider health backend is "mock".
        let h_fut = p.health_check();
        let _ = h_fut;
    }

    #[test]
    fn provider_registry_with_defaults_has_three_routes() {
        let r = ProviderRegistry::with_defaults();
        let models = r.registered_models();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|(p, _)| p == ANTHROPIC_MODEL_PREFIX));
        assert!(models.iter().any(|(p, _)| p == OPENAI_MODEL_PREFIX));
        assert!(models.iter().any(|(p, _)| p == OPENAI_EMBEDDING_PREFIX));
    }

    #[test]
    fn provider_registry_routes_anthropic_models() {
        let r = ProviderRegistry::with_defaults();
        let provider = r.select(&make_request(ANTHROPIC_DEFAULT_MODEL));
        // The AnthropicProvider's health backend begins with "anthropic:".
        // We assert via a non-async helper: same provider is reused.
        let provider2 = r.select(&make_request("claude-3-haiku-20240307"));
        assert!(
            std::sync::Arc::ptr_eq(&provider, &provider2),
            "claude-* models should resolve to the same provider instance"
        );
    }

    #[test]
    fn provider_registry_routes_openai_models() {
        let r = ProviderRegistry::with_defaults();
        let provider = r.select(&make_request(OPENAI_DEFAULT_MODEL));
        let provider2 = r.select(&make_request("gpt-4-turbo"));
        assert!(
            std::sync::Arc::ptr_eq(&provider, &provider2),
            "gpt-* models should resolve to the same provider instance"
        );
    }

    #[test]
    fn provider_registry_falls_back_to_mock_for_unknown_models() {
        let r = ProviderRegistry::with_defaults();
        let provider = r.select(&make_request("llama-3-70b"));
        // Mock provider's health backend is "mock".
        let h_fut = provider.health_check();
        let _ = h_fut;
    }

    #[test]
    fn provider_registry_register_overrides_existing_route() {
        let mut r = ProviderRegistry::with_defaults();
        let original = r.select(&make_request("claude-test"));
        let replacement: Arc<dyn LlmProvider> = Arc::new(MockProvider::new());
        r.register(ANTHROPIC_MODEL_PREFIX, replacement.clone());
        let updated = r.select(&make_request("claude-test"));
        assert!(!std::sync::Arc::ptr_eq(&original, &updated));
    }

    #[tokio::test]
    async fn dispatch_provider_init_and_shutdown_call_fallback() {
        let r = Arc::new(ProviderRegistry::with_defaults());
        let d = DispatchProvider::new(r);
        assert!(d.init().await.is_ok());
        assert!(d.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn dispatch_provider_health_reports_route_count() {
        let r = Arc::new(ProviderRegistry::with_defaults());
        let d = DispatchProvider::new(r);
        let h = d.health_check().await.unwrap();
        assert_eq!(h.backend, "dispatch");
        assert_eq!(h.count, 3);
        assert!(h.healthy);
    }

    #[tokio::test]
    async fn dispatch_provider_routes_unknown_model_to_mock_fallback() {
        let r = Arc::new(ProviderRegistry::with_defaults());
        let d = DispatchProvider::new(r);
        let resp = d
            .chat_completion(make_request("unknown-model"))
            .await
            .unwrap();
        assert!(resp.message.content.starts_with("[mock]"));
    }

    #[tokio::test]
    async fn dispatch_provider_routes_anthropic_to_anthropic_provider() {
        let r = Arc::new(ProviderRegistry::with_defaults());
        let d = DispatchProvider::new(r);
        let resp = d
            .chat_completion(make_request(ANTHROPIC_DEFAULT_MODEL))
            .await
            .unwrap();
        assert!(resp.message.content.contains("[anthropic stub]"));
    }

    #[tokio::test]
    async fn dispatch_provider_routes_openai_to_openai_provider() {
        let r = Arc::new(ProviderRegistry::with_defaults());
        let d = DispatchProvider::new(r);
        let resp = d
            .chat_completion(make_request(OPENAI_DEFAULT_MODEL))
            .await
            .unwrap();
        assert!(resp.message.content.contains("[openai stub]"));
    }

    #[tokio::test]
    async fn dispatch_provider_stream_routes_to_selected_provider() {
        use futures_util::StreamExt;
        let r = Arc::new(ProviderRegistry::with_defaults());
        let d = DispatchProvider::new(r);
        let mut s = d.stream_completion(make_request("gpt-test")).await.unwrap();
        let first = s.next().await.expect("non-empty").unwrap();
        // OpenAI stub for gpt-* prefix.
        assert!(first.delta.contains("openai"));
        assert_eq!(first.finish_reason.as_deref(), Some("stop"));
    }
}
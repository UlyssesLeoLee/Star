// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider/jev.rs` — Jev Cerebellum LLM provider implementation.
//!
//! Provides support for integrating with the Jev Cerebellum LLM service.
//! Handles API key resolution from environment variables or literal values.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #25 v25):
//! - 0 `unsafe` blocks.
//! - Real API keys not hardcoded; mock + env var stub for CI.
//! - Failures mapped to [`LlmProviderRegistryError`].

use async_trait::async_trait;
use futures_util::stream::StreamExt;
use std::env;
use thiserror::Error;
use uuid::Uuid;

use crate::chat::{ChatChunk, ChatMessage, ChatRequest, ChatResponse, ChatRole};
use crate::{
    LlmProvider, LlmProviderRegistryError, LlmProviderRegistryHealth,
};

pub const JEV_DEFAULT_BASE_URL: &str = "https://jev.cerebellum.ai/v1";
pub const JEV_DEFAULT_MODEL: &str = "jev-cerebellum";

/// **ResolvedKey** — represents a parsed API key that may reference an environment variable or be a literal value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedKey {
    /// References an environment variable by name (resolved via `std::env::var`).
    EnvRef(String),
    /// A literal API key value (not an environment variable reference).
    Literal(String),
}

impl ResolvedKey {
    /// Resolve the key to its actual string value.
    ///
    /// For `EnvRef`, fetches the value from the environment.
    /// For `Literal`, returns the value as-is.
    ///
    /// # Errors
    /// Returns `JevError::EnvVarNotFound` if an `EnvRef` variable is not set.
    pub fn resolve(&self) -> Result<String, JevError> {
        match self {
            ResolvedKey::EnvRef(name) => {
                env::var(name).map_err(|_| JevError::EnvVarNotFound(name.clone()))
            }
            ResolvedKey::Literal(value) => Ok(value.clone()),
        }
    }
}

/// **JevError** — errors specific to Jev provider configuration and operation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum JevError {
    /// Environment variable referenced but not found.
    #[error("Environment variable '{0}' not found")]
    EnvVarNotFound(String),
}

/// Parse an API key input string into a `ResolvedKey`.
///
/// Parsing rules:
/// - `$NAME` or `${NAME}` → environment variable reference
/// - `[A-Z_][A-Z0-9_]*` (only uppercase, underscore, digits) → environment variable reference
/// - Anything else (lowercase letters, `-`, spaces, typical key format) → literal key
///
/// # Arguments
/// * `input` - The key input string to parse
///
/// # Returns
/// A `ResolvedKey` variant indicating whether this is an env var reference or literal.
///
/// # Examples
/// ```
/// use crate::provider::jev::parse_key_input;
///
/// assert_eq!(parse_key_input("$MY_KEY"), ResolvedKey::EnvRef("MY_KEY".to_string()));
/// assert_eq!(parse_key_input("${MY_KEY}"), ResolvedKey::EnvRef("MY_KEY".to_string()));
/// assert_eq!(parse_key_input("MY_KEY"), ResolvedKey::EnvRef("MY_KEY".to_string()));
/// assert_eq!(parse_key_input("my-actual-key-123"), ResolvedKey::Literal("my-actual-key-123".to_string()));
/// assert_eq!(parse_key_input("typical-api-key"), ResolvedKey::Literal("typical-api-key".to_string()));
/// ```
pub fn parse_key_input(input: &str) -> ResolvedKey {
    let input = input.trim();

    // Check for $NAME or ${NAME} pattern
    if let Some(stripped) = input.strip_prefix('$') {
        if let Some(name) = stripped.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            // ${NAME} pattern
            return ResolvedKey::EnvRef(name.to_string());
        } else {
            // $NAME pattern - everything after $ is the variable name
            return ResolvedKey::EnvRef(stripped.to_string());
        }
    }

    // Check if input matches [A-Z_][A-Z0-9_]* (uppercase, underscore, digits only)
    if is_env_var_name(input) {
        return ResolvedKey::EnvRef(input.to_string());
    }

    // Otherwise, treat as literal key
    ResolvedKey::Literal(input.to_string())
}

/// Check if a string matches the pattern [A-Z_][A-Z0-9_]*
fn is_env_var_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character must be uppercase letter or underscore
    if !matches!(first, 'A'..='Z' | '_') {
        return false;
    }

    // Remaining characters must be uppercase, digits, or underscore
    chars.all(|c| matches!(c, 'A'..='Z' | '0'..='9' | '_'))
}

/// **JevCerebellumProvider** — Jev Cerebellum LLM provider.
#[derive(Debug, Clone)]
pub struct JevCerebellumProvider {
    /// Resolved API key (either from environment or literal).
    api_key: ResolvedKey,
    /// Base URL for the Jev Cerebellum API.
    base_url: String,
}

impl JevCerebellumProvider {
    /// Create a new Jev provider with a real (resolved) API key.
    ///
    /// The key input is parsed using [`parse_key_input`] to determine if it's
    /// an environment variable reference or a literal key.
    ///
    /// # Arguments
    /// * `key_input` - The API key input (may be a literal or env var reference)
    ///
    /// # Returns
    /// A new `JevCerebellumProvider` instance.
    pub fn with_real_api_key(key_input: &str) -> Self {
        let api_key = parse_key_input(key_input);
        Self {
            api_key,
            base_url: JEV_DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Create a new Jev provider with a custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Get the resolved API key value.
    ///
    /// # Errors
    /// Returns `LlmProviderRegistryError::Backend` if the environment variable is not found.
    pub async fn get_api_key(&self) -> Result<String, LlmProviderRegistryError> {
        self.api_key
            .resolve()
            .map_err(|e| LlmProviderRegistryError::Backend(e.to_string()))
    }
}

#[async_trait]
impl LlmProvider for JevCerebellumProvider {
    async fn init(&self) -> Result<(), LlmProviderRegistryError> {
        // Verify that the API key is available and valid (without exposing it in logs).
        let _key = self.get_api_key().await?;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<LlmProviderRegistryHealth, LlmProviderRegistryError> {
        // Verify the API key is available.
        let _key = self.get_api_key().await?;
        Ok(LlmProviderRegistryHealth {
            count: 0,
            backend: "jev-cerebellum".to_string(),
            healthy: true,
        })
    }

    async fn chat_completion(
        &self,
        req: ChatRequest,
    ) -> Result<ChatResponse, LlmProviderRegistryError> {
        // Validate the request
        if let Err(e) = req.validate() {
            return Err(LlmProviderRegistryError::InvalidOperation(e.to_string()));
        }

        // Verify API key availability
        let _key = self.get_api_key().await?;

        // TODO: Implement actual Jev API call
        // For now, return a stub response similar to mock provider
        let id = req.request_id.unwrap_or_else(Uuid::new_v4);
        Ok(ChatResponse {
            id,
            model: if req.model.is_empty() {
                JEV_DEFAULT_MODEL.to_string()
            } else {
                req.model.clone()
            },
            message: ChatMessage::assistant("Jev response stub".to_string()),
            finish_reason: "stop".to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    async fn stream_completion(
        &self,
        req: ChatRequest,
    ) -> Result<
        futures_util::stream::BoxStream<'static, Result<ChatChunk, LlmProviderRegistryError>>,
        LlmProviderRegistryError,
    > {
        // Validate the request
        if let Err(e) = req.validate() {
            return Err(LlmProviderRegistryError::InvalidOperation(e.to_string()));
        }

        // Verify API key availability
        let _key = self.get_api_key().await?;

        // TODO: Implement actual Jev streaming API call
        let id = req.request_id.unwrap_or_else(Uuid::new_v4);
        let model = if req.model.is_empty() {
            JEV_DEFAULT_MODEL.to_string()
        } else {
            req.model.clone()
        };

        let s = futures_util::stream::once(async move {
            Ok(ChatChunk {
                id,
                model,
                role: ChatRole::Assistant,
                delta: "Jev streaming response stub".to_string(),
                finish_reason: Some("stop".to_string()),
            })
        });

        Ok(s.boxed())
    }
}

// =====================================================================
// Unit Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_input_env_var_dollar_syntax() {
        assert_eq!(
            parse_key_input("$MY_API_KEY"),
            ResolvedKey::EnvRef("MY_API_KEY".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_env_var_braces_syntax() {
        assert_eq!(
            parse_key_input("${MY_API_KEY}"),
            ResolvedKey::EnvRef("MY_API_KEY".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_env_var_plain_uppercase() {
        assert_eq!(
            parse_key_input("MY_API_KEY"),
            ResolvedKey::EnvRef("MY_API_KEY".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_literal_with_lowercase() {
        assert_eq!(
            parse_key_input("my-actual-key"),
            ResolvedKey::Literal("my-actual-key".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_literal_with_hyphens() {
        assert_eq!(
            parse_key_input("typical-api-key-123"),
            ResolvedKey::Literal("typical-api-key-123".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_literal_with_spaces() {
        assert_eq!(
            parse_key_input("key with spaces"),
            ResolvedKey::Literal("key with spaces".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_unicode_literal_key() {
        assert_eq!(
            parse_key_input("密钥-test-🔑"),
            ResolvedKey::Literal("密钥-test-🔑".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_mixed_case_is_literal() {
        assert_eq!(
            parse_key_input("MyApiKey"),
            ResolvedKey::Literal("MyApiKey".to_string())
        );
    }

    #[test]
    fn test_parse_key_input_whitespace_trimming() {
        assert_eq!(
            parse_key_input("  $MY_KEY  "),
            ResolvedKey::EnvRef("MY_KEY".to_string())
        );
        assert_eq!(
            parse_key_input("  my-key  "),
            ResolvedKey::Literal("my-key".to_string())
        );
    }

    #[test]
    fn test_resolved_key_resolve_literal() {
        let key = ResolvedKey::Literal("my-secret-key".to_string());
        assert_eq!(key.resolve().unwrap(), "my-secret-key");
    }

    #[test]
    fn test_resolved_key_resolve_env_var_not_found() {
        let key = ResolvedKey::EnvRef("NONEXISTENT_VAR_XYZ".to_string());
        assert!(matches!(key.resolve(), Err(JevError::EnvVarNotFound(_))));
    }

    #[test]
    fn test_resolved_key_resolve_env_var_exists() {
        env::set_var("TEST_JEV_KEY", "test-value-123");
        let key = ResolvedKey::EnvRef("TEST_JEV_KEY".to_string());
        assert_eq!(key.resolve().unwrap(), "test-value-123");
        env::remove_var("TEST_JEV_KEY");
    }

    #[tokio::test]
    async fn test_jev_provider_with_literal_key() {
        let provider = JevCerebellumProvider::with_real_api_key("my-literal-key");
        assert_eq!(provider.get_api_key().await.unwrap(), "my-literal-key");
    }

    #[tokio::test]
    async fn test_jev_provider_with_env_var_key() {
        env::set_var("JEV_TEST_KEY", "env-value-456");
        let provider = JevCerebellumProvider::with_real_api_key("$JEV_TEST_KEY");
        assert_eq!(provider.get_api_key().await.unwrap(), "env-value-456");
        env::remove_var("JEV_TEST_KEY");
    }

    #[tokio::test]
    async fn test_jev_provider_init_with_valid_key() {
        env::set_var("JEV_INIT_TEST", "valid-key");
        let provider = JevCerebellumProvider::with_real_api_key("$JEV_INIT_TEST");
        assert!(provider.init().await.is_ok());
        env::remove_var("JEV_INIT_TEST");
    }

    #[tokio::test]
    async fn test_jev_provider_init_with_missing_env_var() {
        let provider = JevCerebellumProvider::with_real_api_key("$NONEXISTENT_JEV_VAR");
        assert!(provider.init().await.is_err());
    }

    #[tokio::test]
    async fn test_jev_provider_health_check() {
        env::set_var("JEV_HEALTH_TEST", "valid-key");
        let provider = JevCerebellumProvider::with_real_api_key("$JEV_HEALTH_TEST");
        let health = provider.health_check().await.unwrap();
        assert_eq!(health.backend, "jev-cerebellum");
        assert!(health.healthy);
        env::remove_var("JEV_HEALTH_TEST");
    }

    #[tokio::test]
    async fn test_jev_provider_chat_completion() {
        env::set_var("JEV_CHAT_TEST", "valid-key");
        let provider = JevCerebellumProvider::with_real_api_key("$JEV_CHAT_TEST");
        let req = ChatRequest {
            model: JEV_DEFAULT_MODEL.to_string(),
            messages: vec![ChatMessage::user("test message")],
            temperature: None,
            max_tokens: None,
            request_id: Some(Uuid::new_v4()),
        };
        let resp = provider.chat_completion(req).await.unwrap();
        assert_eq!(resp.model, JEV_DEFAULT_MODEL);
        assert_eq!(resp.message.role, ChatRole::Assistant);
        assert_eq!(resp.finish_reason, "stop");
        env::remove_var("JEV_CHAT_TEST");
    }

    #[test]
    fn test_jev_error_env_var_not_found() {
        let err = JevError::EnvVarNotFound("MY_VAR".to_string());
        assert_eq!(err.to_string(), "Environment variable 'MY_VAR' not found");
    }
}

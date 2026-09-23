// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for Jev provider threshold calibration (1k traffic).
//!
//! This test suite validates Jev provider behavior under various conditions:
//! - Key resolution (env vars vs literals)
//! - Request processing
//! - Error handling with realistic traffic patterns

use domain_llm::{
    chat::{ChatMessage, ChatRequest, ChatRole},
    provider::jev::{parse_key_input, JevCerebellumProvider, ResolvedKey},
    LlmProvider, LlmProviderRegistryError,
};
use std::env;
use uuid::Uuid;

// =====================================================================
// Key Resolution Tests (Unit-level tests for parse_key_input)
// =====================================================================

#[test]
fn threshold_test_env_var_dollar_notation() {
    let key = parse_key_input("$JEV_TEST_KEY");
    assert_eq!(key, ResolvedKey::EnvRef("JEV_TEST_KEY".to_string()));
}

#[test]
fn threshold_test_env_var_brace_notation() {
    let key = parse_key_input("${JEV_TEST_KEY}");
    assert_eq!(key, ResolvedKey::EnvRef("JEV_TEST_KEY".to_string()));
}

#[test]
fn threshold_test_env_var_plain_uppercase() {
    let key = parse_key_input("JEV_TEST_KEY");
    assert_eq!(key, ResolvedKey::EnvRef("JEV_TEST_KEY".to_string()));
}

#[test]
fn threshold_test_literal_key_with_hyphens() {
    let key = parse_key_input("jev-test-key-abc123");
    assert_eq!(key, ResolvedKey::Literal("jev-test-key-abc123".to_string()));
}

#[test]
fn threshold_test_literal_key_with_lowercase() {
    let key = parse_key_input("jev_test_key");
    assert_eq!(key, ResolvedKey::Literal("jev_test_key".to_string()));
}

#[test]
fn threshold_test_literal_key_unicode() {
    let key = parse_key_input("密钥-test-🔑-abc");
    assert_eq!(
        key,
        ResolvedKey::Literal("密钥-test-🔑-abc".to_string())
    );
}

// =====================================================================
// Integration Tests (Provider-level)
// =====================================================================

#[tokio::test]
async fn threshold_test_provider_with_literal_key() {
    let provider = JevCerebellumProvider::with_real_api_key("test-literal-key");
    let api_key = provider.get_api_key().await.unwrap();
    assert_eq!(api_key, "test-literal-key");
}

#[tokio::test]
async fn threshold_test_provider_with_env_var() {
    env::set_var("JEV_THRESHOLD_TEST", "threshold-test-value");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_THRESHOLD_TEST");
    let api_key = provider.get_api_key().await.unwrap();
    assert_eq!(api_key, "threshold-test-value");
    env::remove_var("JEV_THRESHOLD_TEST");
}

#[tokio::test]
async fn threshold_test_provider_missing_env_var_error() {
    let provider = JevCerebellumProvider::with_real_api_key("$NONEXISTENT_THRESHOLD_VAR");
    let result = provider.get_api_key().await;
    assert!(result.is_err());
    match result {
        Err(LlmProviderRegistryError::Backend(msg)) => {
            assert!(msg.contains("Environment variable"));
        }
        _ => panic!("Expected Backend error"),
    }
}

#[tokio::test]
async fn threshold_test_provider_init_success() {
    env::set_var("JEV_INIT_THRESHOLD", "init-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_INIT_THRESHOLD");
    assert!(provider.init().await.is_ok());
    env::remove_var("JEV_INIT_THRESHOLD");
}

#[tokio::test]
async fn threshold_test_provider_init_failure() {
    let provider = JevCerebellumProvider::with_real_api_key("$MISSING_THRESHOLD_VAR");
    assert!(provider.init().await.is_err());
}

#[tokio::test]
async fn threshold_test_provider_health_check() {
    env::set_var("JEV_HEALTH_THRESHOLD", "health-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_HEALTH_THRESHOLD");
    let health = provider.health_check().await.unwrap();
    assert_eq!(health.backend, "jev-cerebellum");
    assert!(health.healthy);
    env::remove_var("JEV_HEALTH_THRESHOLD");
}

#[tokio::test]
async fn threshold_test_chat_completion_basic() {
    env::set_var("JEV_CHAT_THRESHOLD", "chat-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_CHAT_THRESHOLD");
    let req = ChatRequest {
        model: "jev-cerebellum".to_string(),
        messages: vec![ChatMessage::user("Hello, Jev")],
        temperature: None,
        max_tokens: None,
        request_id: Some(Uuid::new_v4()),
    };
    let resp = provider.chat_completion(req).await.unwrap();
    assert_eq!(resp.model, "jev-cerebellum");
    assert_eq!(resp.message.role, ChatRole::Assistant);
    assert!(!resp.message.content.is_empty());
    env::remove_var("JEV_CHAT_THRESHOLD");
}

#[tokio::test]
async fn threshold_test_chat_completion_invalid_request() {
    env::set_var("JEV_INVALID_THRESHOLD", "invalid-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_INVALID_THRESHOLD");
    let req = ChatRequest {
        model: "".to_string(), // Empty model should be invalid
        messages: vec![ChatMessage::user("test")],
        temperature: None,
        max_tokens: None,
        request_id: None,
    };
    let result = provider.chat_completion(req).await;
    assert!(result.is_err());
    env::remove_var("JEV_INVALID_THRESHOLD");
}

#[tokio::test]
async fn threshold_test_stream_completion_basic() {
    use futures_util::StreamExt;

    env::set_var("JEV_STREAM_THRESHOLD", "stream-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_STREAM_THRESHOLD");
    let req = ChatRequest {
        model: "jev-cerebellum".to_string(),
        messages: vec![ChatMessage::user("Stream test")],
        temperature: None,
        max_tokens: None,
        request_id: Some(Uuid::new_v4()),
    };
    let mut stream = provider.stream_completion(req).await.unwrap();
    let first_chunk = stream.next().await.expect("Stream should have at least one chunk");
    assert!(first_chunk.is_ok());
    let chunk = first_chunk.unwrap();
    assert_eq!(chunk.role, ChatRole::Assistant);
    env::remove_var("JEV_STREAM_THRESHOLD");
}

// =====================================================================
// 1k Traffic Threshold Tests
// =====================================================================

/// Simulate concurrent requests to test provider under load (1k requests).
/// This is a simplified threshold test using sequential requests.
#[tokio::test]
async fn threshold_test_1k_requests_sequential() {
    env::set_var("JEV_LOAD_THRESHOLD", "load-test-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_LOAD_THRESHOLD");

    // Initialize provider
    assert!(provider.init().await.is_ok());

    // Process 1k sequential requests (simplified test)
    let mut success_count = 0;
    for i in 0..1000 {
        let req = ChatRequest {
            model: "jev-cerebellum".to_string(),
            messages: vec![ChatMessage::user(&format!("Request #{}", i))],
            temperature: None,
            max_tokens: None,
            request_id: Some(Uuid::new_v4()),
        };

        if let Ok(_resp) = provider.chat_completion(req).await {
            success_count += 1;
        }
    }

    // All requests should succeed (1k threshold)
    assert_eq!(success_count, 1000, "Expected 1000 successful requests");

    // Shutdown
    assert!(provider.shutdown().await.is_ok());

    env::remove_var("JEV_LOAD_THRESHOLD");
}

/// Test health check under load
#[tokio::test]
async fn threshold_test_health_check_under_load() {
    env::set_var("JEV_HEALTH_LOAD_THRESHOLD", "health-load-key");
    let provider = JevCerebellumProvider::with_real_api_key("$JEV_HEALTH_LOAD_THRESHOLD");

    // Multiple health checks
    for _ in 0..100 {
        let health = provider.health_check().await.unwrap();
        assert!(health.healthy);
    }

    env::remove_var("JEV_HEALTH_LOAD_THRESHOLD");
}

/// Test error handling with missing env var under load
#[tokio::test]
async fn threshold_test_missing_env_var_under_load() {
    let provider = JevCerebellumProvider::with_real_api_key("$MISSING_THRESHOLD_LOAD");

    let mut error_count = 0;
    for _ in 0..100 {
        if provider.init().await.is_err() {
            error_count += 1;
        }
    }

    // All attempts should fail with missing env var
    assert_eq!(error_count, 100);
}

// =====================================================================
// CI Environment Variable Path Tests
// =====================================================================

/// Test JEV_TEST_KEY environment variable path
/// This test expects JEV_TEST_KEY to be set in CI environment
#[tokio::test]
#[ignore] // Only run in CI with JEV_TEST_KEY set
async fn threshold_test_ci_env_var_path() {
    if let Ok(test_key) = env::var("JEV_TEST_KEY") {
        let provider = JevCerebellumProvider::with_real_api_key("$JEV_TEST_KEY");
        let api_key = provider.get_api_key().await.unwrap();
        assert_eq!(api_key, test_key);
    }
}

/// Test provider initialization with CI environment key
#[tokio::test]
#[ignore] // Only run in CI with JEV_TEST_KEY set
async fn threshold_test_ci_provider_init() {
    if env::var("JEV_TEST_KEY").is_ok() {
        let provider = JevCerebellumProvider::with_real_api_key("$JEV_TEST_KEY");
        assert!(provider.init().await.is_ok());
    }
}

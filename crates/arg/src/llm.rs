// SPDX-License-Identifier: MIT OR Apache-2.0
//! LLM abstraction (per DD-AGENT-RELATIONSHIP-001 §3.2.5).
//!
//! `LLMClient` is the trait implemented by real LLM adapters (Anthropic,
//! OpenAI, …) and by the local mock. Per 守门 #5 v2 + #23 the mock is
//! the only implementation that ships in this crate — production
//! adapters live in `arg-effect` or downstream crates and are NOT
//! wired up in ARG.1.

use async_trait::async_trait;

use crate::error::ARGError;
use crate::models::decision::Verdict;

/// LLM client trait (per DD §3.2.5).
///
/// The `call` method takes a free-form prompt and a JSON-serialisable
/// input payload, and returns a structured [`LLMResponse`].
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Run the prompt against the LLM and return a structured response.
    async fn call(&self, prompt: &str, input: &serde_json::Value) -> Result<LLMResponse, ARGError>;
}

/// LLM response payload.
#[derive(Debug, Clone, PartialEq)]
pub struct LLMResponse {
    /// Plain-text content returned by the model.
    pub content: String,
    /// Optional verdict for challenge / peer-review rounds.
    pub verdict: Option<Verdict>,
    /// Optional numeric score in `[0.0, 1.0]`.
    pub score: Option<f32>,
    /// Token count reported by the adapter (best effort).
    pub token_used: u32,
}

impl LLMResponse {
    /// Build a new response with the given content and metadata.
    pub fn new(
        content: String,
        verdict: Option<Verdict>,
        score: Option<f32>,
        token_used: u32,
    ) -> Self {
        Self {
            content,
            verdict,
            score,
            token_used,
        }
    }
}

/// Local mock implementation of [`LLMClient`].
///
/// Per 守门 #23 the mock **always** returns a low-confidence response
/// (`score = 0.42`, `verdict = None`) and the prompt is reflected in the
/// `content` so unit tests can assert what was sent.
#[derive(Debug, Default, Clone)]
pub struct MockLLMClient;

impl MockLLMClient {
    /// Build a new mock client.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn call(&self, prompt: &str, input: &serde_json::Value) -> Result<LLMResponse, ARGError> {
        Ok(LLMResponse::new(
            format!("[mock] prompt={} input={}", prompt, input),
            None,
            Some(0.42),
            0,
        ))
    }
}

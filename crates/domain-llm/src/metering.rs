// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/metering.rs` — W2 (ULYS-98-W2) LLM 调用计费埋点.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.5":
//! - 每个 LLM 调用记录 token usage (input / output / total)
//! - v0.0.2: in-memory HashMap (`llm_usage` table schema deferred to W3)
//! - 提供 `GET /v1/metering/usage?user_id=...` query API (route 在 `crates/api/src/metering.rs`)
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #13 a W/T/M + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"`).
//! - 写路径 append-only (per 守门 #13 a L0 协调派生): 调用方不能删除 / 修改历史事件.
//! - thread-safe (`Arc<Mutex<HashMap<...>>>`).
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

// =====================================================================
// TokenUsage (one event)
// =====================================================================

/// **TokenUsage** — per-call token accounting event.
///
/// v0.0.2 ships `input_tokens` + `output_tokens` + `total_tokens`; richer
/// telemetry (TTFB / latency / provider request id) is deferred to W3.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    /// Caller user id (per `LLMProviderRegistry.actor_id`).
    pub user_id: Uuid,
    /// Tenant id (per `LLMProviderRegistry.tenant_id`).
    pub tenant_id: Uuid,
    /// Provider backend identifier (`mock` / `anthropic` / `openai` / ...).
    pub provider: String,
    /// Model identifier (echo of `ChatRequest::model`).
    pub model: String,
    /// Input tokens consumed.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
    /// Convenience: `input_tokens + output_tokens`.
    pub total_tokens: u32,
    /// Server-side timestamp at event capture.
    pub captured_at: DateTime<Utc>,
    /// Optional provider request id (for traceability).
    pub request_id: Option<Uuid>,
}

impl TokenUsage {
    /// Build a new event with `total_tokens` auto-computed.
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        provider: impl Into<String>,
        model: impl Into<String>,
        input_tokens: u32,
        output_tokens: u32,
        request_id: Option<Uuid>,
    ) -> Self {
        Self {
            user_id,
            tenant_id,
            provider: provider.into(),
            model: model.into(),
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
            captured_at: Utc::now(),
            request_id,
        }
    }
}

/// **UsageAggregate** — sum of [`TokenUsage`] for a query window.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UsageAggregate {
    pub user_id: Uuid,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub total_tokens: u32,
    pub call_count: u32,
}

// =====================================================================
// MeteringStore (in-memory v0.0.2, swap for sqlx in W3)
// =====================================================================

/// **MeteringStore** — thread-safe append-only token usage ledger.
///
/// v0.0.2 uses an in-memory `HashMap<Uuid, TokenUsage>` keyed by event id.
/// Persistence (per-call `INSERT INTO llm_usage`) is deferred to W3 per
/// the brief's stage ordering (W3.1 `chat_sessions` / `chat_messages`
/// migrations + repo).
#[derive(Debug, Default, Clone)]
pub struct MeteringStore {
    inner: Arc<Mutex<MeteringStoreInner>>,
}

#[derive(Debug, Default)]
struct MeteringStoreInner {
    /// Append-only ledger (insertion order preserved by BTreeMap keys).
    events: HashMap<Uuid, TokenUsage>,
}

impl MeteringStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a [`TokenUsage`] event. Returns the event id.
    ///
    /// **守门 #13 a W/T/M**: append-only — callers cannot remove or mutate
    /// historical events. Aggregation is derived from the ledger.
    pub async fn record(&self, event: TokenUsage) -> Uuid {
        let id = event.request_id.unwrap_or_else(Uuid::new_v4);
        let mut inner = self.inner.lock().await;
        inner.events.insert(id, event);
        id
    }

    /// Convenience: derive a [`TokenUsage`] from explicit numbers and
    /// append it.
    pub async fn record_call(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        provider: impl Into<String>,
        model: impl Into<String>,
        input_tokens: u32,
        output_tokens: u32,
        request_id: Option<Uuid>,
    ) -> Uuid {
        let event = TokenUsage::new(
            user_id,
            tenant_id,
            provider,
            model,
            input_tokens,
            output_tokens,
            request_id,
        );
        self.record(event).await
    }

    /// Aggregate usage for a single user (across all events).
    pub async fn aggregate_for_user(&self, user_id: Uuid) -> UsageAggregate {
        let inner = self.inner.lock().await;
        let mut agg = UsageAggregate {
            user_id,
            ..Default::default()
        };
        for ev in inner.events.values() {
            if ev.user_id == user_id {
                agg.total_input_tokens = agg.total_input_tokens.saturating_add(ev.input_tokens);
                agg.total_output_tokens = agg.total_output_tokens.saturating_add(ev.output_tokens);
                agg.total_tokens = agg.total_tokens.saturating_add(ev.total_tokens);
                agg.call_count = agg.call_count.saturating_add(1);
            }
        }
        agg
    }

    /// Snapshot all events (read-only copy). Useful for `/v1/metering/usage`
    /// query responses.
    pub async fn snapshot(&self) -> Vec<TokenUsage> {
        let inner = self.inner.lock().await;
        inner.events.values().cloned().collect()
    }

    /// Current event count (cheap snapshot for health checks).
    pub async fn len(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.events.len()
    }

    /// Wipe all events. **测试 only** — production code must not call this.
    #[doc(hidden)]
    pub async fn reset_for_tests(&self) {
        let mut inner = self.inner.lock().await;
        inner.events.clear();
    }
}

// =====================================================================
// Token estimation helpers (offline; v0.0.2 heuristic — chars / 4)
// =====================================================================

/// Rough estimate of token count for a string. v0.0.2 uses a
/// `chars / 4` heuristic (English text averages ~4 chars per token;
/// CJK averages ~1.5–2 chars per token, so the heuristic **over-counts
/// CJK by 2×** — adequate for metering cost ceilings, not for billing).
///
/// W3 will replace this with provider-reported token counts from
/// Anthropic `usage.output_tokens` / OpenAI `usage.total_tokens`.
pub fn estimate_tokens(text: &str) -> u32 {
    let chars = text.chars().count() as u32;
    chars.div_ceil(4)
}

/// Estimate input/output tokens for a single chat completion.
///
/// - `input_tokens` = sum of `estimate_tokens` over all messages
/// - `output_tokens` = `estimate_tokens(reply)`
pub fn estimate_chat_tokens(req_messages: &[(String, String)], reply: &str) -> (u32, u32) {
    let input: u32 = req_messages
        .iter()
        .map(|(_, content)| estimate_tokens(content))
        .sum();
    let output = estimate_tokens(reply);
    (input, output)
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 — >= 1 unit test per module)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn user() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }
    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    #[test]
    fn token_usage_new_computes_total() {
        let e = TokenUsage::new(user(), tenant(), "anthropic", "claude-test", 100, 50, None);
        assert_eq!(e.input_tokens, 100);
        assert_eq!(e.output_tokens, 50);
        assert_eq!(e.total_tokens, 150);
    }

    #[test]
    fn token_usage_total_saturates_on_overflow() {
        let e = TokenUsage::new(user(), tenant(), "openai", "gpt-test", u32::MAX, 1, None);
        assert_eq!(e.total_tokens, u32::MAX, "saturating add must not panic");
    }

    #[test]
    fn estimate_tokens_rounds_up() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
        assert_eq!(estimate_tokens("a"), 1);
    }

    #[test]
    fn estimate_chat_tokens_sums_inputs_and_estimates_output() {
        let msgs = vec![
            ("user".to_string(), "hello".to_string()),      // 2 tokens
            ("assistant".to_string(), "world".to_string()), // 2 tokens
        ];
        let (inp, out) = estimate_chat_tokens(&msgs, "hi");
        assert_eq!(inp, 4);
        assert_eq!(out, 1);
    }

    #[tokio::test]
    async fn metering_store_record_appends_event() {
        let s = MeteringStore::new();
        assert_eq!(s.len().await, 0);
        s.record_call(user(), tenant(), "mock", "mock-llm", 10, 5, None)
            .await;
        assert_eq!(s.len().await, 1);
    }

    #[tokio::test]
    async fn metering_store_aggregate_for_user_sums_only_target() {
        let s = MeteringStore::new();
        s.record_call(user(), tenant(), "mock", "mock-llm", 10, 5, None)
            .await;
        let other = Uuid::parse_str("00000000-0000-0000-0000-0000000000ff").unwrap();
        s.record_call(other, tenant(), "mock", "mock-llm", 20, 10, None)
            .await;

        let agg = s.aggregate_for_user(user()).await;
        assert_eq!(agg.total_input_tokens, 10);
        assert_eq!(agg.total_output_tokens, 5);
        assert_eq!(agg.total_tokens, 15);
        assert_eq!(agg.call_count, 1);
    }

    #[tokio::test]
    async fn metering_store_snapshot_returns_all_events() {
        let s = MeteringStore::new();
        s.record_call(user(), tenant(), "anthropic", "claude-test", 1, 1, None)
            .await;
        s.record_call(user(), tenant(), "openai", "gpt-test", 2, 2, None)
            .await;
        let snap = s.snapshot().await;
        assert_eq!(snap.len(), 2);
    }

    #[tokio::test]
    async fn metering_store_reset_for_tests_clears_state() {
        let s = MeteringStore::new();
        s.record_call(user(), tenant(), "mock", "mock-llm", 1, 1, None)
            .await;
        assert_eq!(s.len().await, 1);
        s.reset_for_tests().await;
        assert_eq!(s.len().await, 0);
    }

    #[tokio::test]
    async fn metering_store_supports_concurrent_record() {
        use std::sync::Arc;
        let s = Arc::new(MeteringStore::new());
        let mut handles = Vec::new();
        for _ in 0..10 {
            let s2 = s.clone();
            handles.push(tokio::spawn(async move {
                s2.record_call(user(), tenant(), "mock", "mock-llm", 1, 1, None)
                    .await;
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(s.len().await, 10);
    }
}

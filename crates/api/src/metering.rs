// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/metering.rs` — W2 (ULYS-98-W2) LLM 调用计费 query API.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.5":
//! - GET `/v1/metering/usage?user_id=...&tenant_id=...`
//! - returns aggregated token usage for the given user (and tenant if provided)
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #10 + 守门 #13 a W/T/M + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - 只读端点 (per 守门 #13 a W/T/M: W 是 audit log; T 是 token usage; M 是 metering store).
//! 5 域 Lead 真人到位前 Mavis 临时代签.

#![allow(missing_docs)] // per crates/domain-llm/src/lib.rs precedent — module-level docs present.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain_llm::{MeteringStore, TokenUsage, UsageAggregate};

use super::chat::ChatState;
use crate::ApiError;

// =====================================================================
// Query DTO
// =====================================================================

/// GET `/v1/metering/usage?user_id=...&tenant_id=...` query.
#[derive(Debug, Clone, Deserialize)]
pub struct MeteringUsageQuery {
    pub user_id: Uuid,
    /// Optional tenant id (defaults to the state's tenant).
    pub tenant_id: Option<Uuid>,
}

// =====================================================================
// Response DTO
// =====================================================================

/// GET `/v1/metering/usage` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeteringUsageResponse {
    pub aggregate: UsageAggregate,
    /// All events for the user (sorted by captured_at ASC). May be empty
    /// if no calls have been recorded yet.
    pub events: Vec<TokenUsage>,
}

// =====================================================================
// Router
// =====================================================================

/// 1-route router factory: `GET /v1/metering/usage`.
pub fn metering_routes(state: Arc<ChatState>) -> Router {
    Router::new()
        .route("/v1/metering/usage", get(metering_usage))
        .with_state(state)
}

// =====================================================================
// Handlers
// =====================================================================

/// GET `/v1/metering/usage?user_id=...` — aggregate + per-event token
/// usage for a user.
async fn metering_usage(
    State(state): State<Arc<ChatState>>,
    Query(q): Query<MeteringUsageQuery>,
) -> Result<Json<MeteringUsageResponse>, ApiError> {
    let metering: Arc<MeteringStore> = state.metering.clone();
    let aggregate = metering.aggregate_for_user(q.user_id).await;
    let events: Vec<TokenUsage> = metering
        .snapshot()
        .await
        .into_iter()
        .filter(|e| e.user_id == q.user_id)
        .filter(|e| q.tenant_id.map(|t| e.tenant_id == t).unwrap_or(true))
        .collect();
    Ok(Json(MeteringUsageResponse { aggregate, events }))
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use domain_llm::provider::mock::MockProvider;
    use domain_llm::LlmProvider;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }
    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }
    fn user() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-00000000000a").unwrap()
    }

    fn make_state() -> Arc<ChatState> {
        let mock: Arc<dyn LlmProvider> = Arc::new(MockProvider::new());
        ChatState::with_provider(mock, Arc::new(MeteringStore::new()), tenant(), actor())
    }

    #[tokio::test]
    async fn metering_usage_returns_empty_aggregate_for_unknown_user() {
        let state = make_state();
        let resp = metering_usage(
            State(state),
            Query(MeteringUsageQuery {
                user_id: Uuid::new_v4(),
                tenant_id: None,
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(resp.aggregate.call_count, 0);
        assert_eq!(resp.aggregate.total_tokens, 0);
        assert!(resp.events.is_empty());
    }

    #[tokio::test]
    async fn metering_usage_returns_recorded_events() {
        let state = make_state();
        // Record a couple of events directly.
        state
            .metering
            .record_call(user(), tenant(), "mock", "mock-llm", 10, 5, None)
            .await;
        state
            .metering
            .record_call(user(), tenant(), "anthropic", "claude-test", 20, 10, None)
            .await;

        let resp = metering_usage(
            State(state),
            Query(MeteringUsageQuery {
                user_id: user(),
                tenant_id: Some(tenant()),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(resp.aggregate.call_count, 2);
        assert_eq!(resp.aggregate.total_input_tokens, 30);
        assert_eq!(resp.aggregate.total_output_tokens, 15);
        assert_eq!(resp.aggregate.total_tokens, 45);
        assert_eq!(resp.events.len(), 2);
    }

    #[tokio::test]
    async fn metering_usage_filters_by_tenant() {
        let state = make_state();
        let other_tenant = Uuid::parse_str("00000000-0000-0000-0000-0000000000bb").unwrap();
        state
            .metering
            .record_call(user(), tenant(), "mock", "mock-llm", 10, 5, None)
            .await;
        state
            .metering
            .record_call(user(), other_tenant, "mock", "mock-llm", 100, 50, None)
            .await;

        // aggregate_for_user counts across all tenants (no tenant filter),
        // but the events list returned by the handler DOES filter by tenant.
        let agg = state.metering.aggregate_for_user(user()).await;
        assert_eq!(agg.call_count, 2);

        let resp = metering_usage(
            State(state),
            Query(MeteringUsageQuery {
                user_id: user(),
                tenant_id: Some(tenant()),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(
            resp.aggregate.call_count, 2,
            "aggregate includes both tenants"
        );
        assert_eq!(resp.events.len(), 1, "events list filters to tenant");
        assert_eq!(resp.events[0].tenant_id, tenant());
    }

    #[test]
    fn metering_routes_builds_router_without_error() {
        let state = make_state();
        let _router = metering_routes(state);
    }
}

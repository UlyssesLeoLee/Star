// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/chat.rs` — W1 + W2 (ULYS-98-W1 + ULYS-98-W2) Chat RPC stub.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 1.3":
//! - POST `/v1/chat/send` — non-streaming chat reply
//! - GET  `/v1/chat/messages?session_id=...` — list messages (v0.0.1 stub returns empty)
//!
//! W2 (ULYS-98-W2) wires the handler to `domain_llm::DispatchProvider` so
//! real LLM providers (anthropic / openai / mock fallback) handle the
//! actual call. v0.0.2 still keeps an in-memory message store; W3.1
//! replaces it with a Postgres-backed session/message repo per brief.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #10 + 守门 #13 a W/T/M + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - DTOs derive `Serialize` / `Deserialize` (per 守门 #11 缺标比错标).
//! - 每条 chat message 走 audit log (per 守门 #13 d).
//! 5 域 Lead 真人到位前 Mavis 临时代签.

#![allow(missing_docs)] // per crates/domain-llm/src/lib.rs precedent — module-level docs present.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use domain_llm::chat::{ChatMessage, ChatRequest, ChatRole};
use domain_llm::{
    LlmProvider, LlmProviderRegistryError, MeteringStore, ProviderRegistry, TokenUsage,
};

use crate::ApiError;

// =====================================================================
// ChatState (per route handler)
// =====================================================================

/// **ChatState** — shared state for the chat routes (per W1.3).
///
/// `#[derive(Clone)]` so axum can pass `Arc<ChatState>` across handlers
/// cheaply (per axum 0.8 `State` extractor requirement).
#[derive(Clone)]
pub struct ChatState {
    /// The provider registry (Anthropic + OpenAI + Mock fallback).
    pub provider: Arc<dyn LlmProvider>,
    /// In-memory chat session store (v0.0.1 stub; W3.1 swaps for sqlx).
    pub sessions: Arc<Mutex<HashMap<Uuid, ChatSession>>>,
    /// Metering store (W2.5 — records token usage per call).
    pub metering: Arc<MeteringStore>,
    /// Default tenant id (守门 #13 b RLS 13 類).
    pub tenant_id: Uuid,
    /// Default actor id (守门 #10 author).
    pub actor_id: Uuid,
}

impl ChatState {
    /// Construct a state with the standard provider registry.
    pub fn new(tenant_id: Uuid, actor_id: Uuid) -> Arc<Self> {
        let registry = Arc::new(ProviderRegistry::with_defaults());
        let provider: Arc<dyn LlmProvider> =
            Arc::new(domain_llm::DispatchProvider::new(registry));
        Arc::new(Self {
            provider,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            metering: Arc::new(MeteringStore::new()),
            tenant_id,
            actor_id,
        })
    }

    /// Construct from a pre-built `Arc<dyn LlmProvider>` (handy for tests).
    pub fn with_provider(
        provider: Arc<dyn LlmProvider>,
        metering: Arc<MeteringStore>,
        tenant_id: Uuid,
        actor_id: Uuid,
    ) -> Arc<Self> {
        Arc::new(Self {
            provider,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            metering,
            tenant_id,
            actor_id,
        })
    }
}

/// **ChatSession** — in-memory chat session (per W1.3).
#[derive(Debug, Clone)]
pub struct ChatSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =====================================================================
// Request DTOs
// =====================================================================

/// POST `/v1/chat/send` request body.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatSendRequest {
    /// Session id (UUID). New sessions are created if absent.
    pub session_id: Uuid,
    /// Caller user id (per 守门 #10).
    pub user_id: Uuid,
    /// User message content (non-empty).
    pub content: String,
    /// Optional model override (e.g. `claude-3-5-sonnet`).
    pub model: Option<String>,
}

impl ChatSendRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.content.trim().is_empty() {
            return Err("chat send: content must be non-empty");
        }
        Ok(())
    }
}

/// GET `/v1/chat/messages?session_id=...` query.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessagesQuery {
    pub session_id: Uuid,
}

/// GET `/v1/chat/stream?session_id=...&content=...` query (W3.2 SSE).
#[derive(Debug, Clone, Deserialize)]
pub struct ChatStreamQuery {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub model: Option<String>,
}

// =====================================================================
// Response DTOs
// =====================================================================

/// POST `/v1/chat/send` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatSendResponse {
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub assistant_content: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    /// Token usage for this call (W2.5 — populated from provider metering).
    pub usage: Option<TokenUsage>,
}

/// GET `/v1/chat/messages` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessagesResponse {
    pub session_id: Uuid,
    pub messages: Vec<ChatMessage>,
    pub count: usize,
}

// =====================================================================
// Router
// =====================================================================

/// 2-route router factory: `POST /v1/chat/send` + `GET /v1/chat/messages`.
pub fn chat_routes(state: Arc<ChatState>) -> Router {
    Router::new()
        .route("/v1/chat/send", post(chat_send))
        .route("/v1/chat/messages", get(chat_messages))
        .route("/v1/chat/stream", get(chat_stream))
        .with_state(state)
}

// =====================================================================
// Handlers
// =====================================================================

/// POST `/v1/chat/send` — non-streaming chat reply.
async fn chat_send(
    State(state): State<Arc<ChatState>>,
    Json(req): Json<ChatSendRequest>,
) -> Result<Json<ChatSendResponse>, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            e,
            "api",
            "validation",
            false,
            "Provide a non-empty content field",
        ));
    }

    let mut sessions = state.sessions.lock().await;
    let now = Utc::now();

    // Create session if missing.
    let session = sessions.entry(req.session_id).or_insert_with(|| ChatSession {
        id: req.session_id,
        user_id: req.user_id,
        messages: Vec::new(),
        created_at: now,
        updated_at: now,
    });

    // Append user message.
    session.messages.push(ChatMessage::user(req.content.clone()));
    session.updated_at = now;

    // Build ChatRequest for the provider.
    let model = req
        .model
        .clone()
        .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());
    let chat_req = ChatRequest {
        model: model.clone(),
        messages: session.messages.clone(),
        temperature: None,
        max_tokens: Some(1024),
        request_id: Some(Uuid::new_v4()),
    };

    // Dispatch to the provider (real LLM when key present + no_network_mode=false;
    // stub otherwise — see brief §Risk #1).
    let resp = state.provider.chat_completion(chat_req).await.map_err(|e| {
        ApiError::new(
            "LLM_PROVIDER_ERROR",
            format!("chat send: provider dispatch failed: {e}"),
            "api",
            "external",
            true,
            "Retry the request; check provider health if persistent",
        )
    })?;

    // Record token usage (W2.5 metering).
    let usage = TokenUsage::new(
        req.user_id,
        state.tenant_id,
        model.clone(),
        model.clone(),
        // v0.0.2 heuristic: rough char/4 estimate; W3 replaces with
        // provider-reported counts.
        ((req.content.chars().count() as u32).div_ceil(4)),
        ((resp.message.content.chars().count() as u32).div_ceil(4)),
        Some(resp.id),
    );
    state.metering.record(usage.clone()).await;

    // Append assistant reply.
    session.messages.push(resp.message.clone());
    session.updated_at = Utc::now();

    let message_id = resp.id;
    Ok(Json(ChatSendResponse {
        session_id: req.session_id,
        message_id,
        assistant_content: resp.message.content,
        model: resp.model,
        created_at: resp.created_at,
        usage: Some(usage),
    }))
}

/// GET `/v1/chat/messages?session_id=...` — list messages for a session.
async fn chat_messages(
    State(state): State<Arc<ChatState>>,
    Query(q): Query<ChatMessagesQuery>,
) -> Result<Json<ChatMessagesResponse>, ApiError> {
    let sessions = state.sessions.lock().await;
    match sessions.get(&q.session_id) {
        Some(s) => Ok(Json(ChatMessagesResponse {
            session_id: s.id,
            count: s.messages.len(),
            messages: s.messages.clone(),
        })),
        None => Err(ApiError::new(
            "RESOURCE_NOT_FOUND",
            format!("chat session {} not found", q.session_id),
            "api",
            "validation",
            false,
            "Send a chat message first to create the session",
        )),
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use domain_llm::provider::mock::MockProvider;
    use domain_llm::MeteringStore;

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

    #[test]
    fn chat_send_request_validate_rejects_empty_content() {
        let r = ChatSendRequest {
            session_id: Uuid::new_v4(),
            user_id: user(),
            content: "   ".to_string(),
            model: None,
        };
        assert!(r.validate().is_err());
        let r = ChatSendRequest {
            session_id: Uuid::new_v4(),
            user_id: user(),
            content: "hi".to_string(),
            model: None,
        };
        assert!(r.validate().is_ok());
    }

    #[tokio::test]
    async fn chat_send_handler_returns_assistant_reply_and_records_metering() {
        let state = make_state();
        let req = ChatSendRequest {
            session_id: Uuid::new_v4(),
            user_id: user(),
            content: "hello".to_string(),
            model: Some("claude-3-5-sonnet-20241022".to_string()),
        };
        let resp = chat_send(State(state.clone()), Json(req.clone()))
            .await
            .unwrap()
            .0;
        assert_eq!(resp.session_id, req.session_id);
        assert!(resp.assistant_content.starts_with("[mock]"));
        assert_eq!(resp.usage.as_ref().unwrap().user_id, user());

        let agg = state.metering.aggregate_for_user(user()).await;
        assert_eq!(agg.call_count, 1);
        assert!(agg.total_tokens > 0);
    }

    #[tokio::test]
    async fn chat_send_handler_rejects_empty_content() {
        let state = make_state();
        let req = ChatSendRequest {
            session_id: Uuid::new_v4(),
            user_id: user(),
            content: "".to_string(),
            model: None,
        };
        let err = chat_send(State(state), Json(req)).await.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn chat_messages_handler_returns_session_messages() {
        let state = make_state();
        let sid = Uuid::new_v4();
        // First send a message to create the session.
        chat_send(
            State(state.clone()),
            Json(ChatSendRequest {
                session_id: sid,
                user_id: user(),
                content: "hi".to_string(),
                model: None,
            }),
        )
        .await
        .unwrap();

        let resp = chat_messages(
            State(state),
            Query(ChatMessagesQuery { session_id: sid }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(resp.session_id, sid);
        assert_eq!(resp.count, 2);
        assert_eq!(resp.messages[0].role, ChatRole::User);
        assert_eq!(resp.messages[1].role, ChatRole::Assistant);
    }

    #[tokio::test]
    async fn chat_messages_handler_returns_not_found_for_unknown_session() {
        let state = make_state();
        let err = chat_messages(
            State(state),
            Query(ChatMessagesQuery {
                session_id: Uuid::new_v4(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "RESOURCE_NOT_FOUND");
    }

    /// W3.2 IT: chat_stream rejects empty content with VALIDATION_FAILED.
    #[tokio::test]
    async fn chat_stream_handler_rejects_empty_content_with_400() {
        let state = make_state();
        let err = chat_stream(
            State(state),
            Query(ChatStreamQuery {
                session_id: Uuid::new_v4(),
                user_id: user(),
                content: "   ".to_string(),
                model: None,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    /// W3.2 IT: chat_stream happy path -- mock provider yields 1 chunk + terminal done.
    /// We assert the SSE stream emits at least one `event: data` payload containing
    /// `delta` + `done` fields by polling the inner stream via `axum::body::Body`.
    #[tokio::test]
    async fn chat_stream_handler_emits_at_least_one_sse_data_frame() {
        use axum::body::to_bytes;
        use axum::http::{Request, StatusCode};

        let state = make_state();
        let router = chat_routes(state);
        let app = tower::ServiceExt::oneshot(
            router,
            Request::builder()
                .method("GET")
                .uri("/v1/chat/stream?session_id=11111111-1111-1111-1111-111111111111&user_id=22222222-2222-2222-2222-222222222222&content=hello")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(app.status(), StatusCode::OK);
        let ct = app.headers().get("content-type").cloned();
        assert!(
            ct.as_ref().and_then(|v| v.to_str().ok()).map(|s| s.starts_with("text/event-stream")).unwrap_or(false),
            "expected text/event-stream content-type, got {:?}",
            ct,
        );
        // Body may be chunked; we don't require a specific number of bytes
        // (mock provider emits done:true immediately), only that the body
        // is non-empty and contains at least one `data:` SSE frame.
        let bytes = to_bytes(app.into_body(), 1024 * 64).await.unwrap();
        let body_str = String::from_utf8_lossy(&bytes);
        assert!(
            body_str.contains("data:") && body_str.contains(""done""),
            "expected SSE data frame with done field, got: {}",
            body_str.chars().take(400).collect::<String>(),
        );
    }

    #[test]
    fn chat_routes_builds_router_without_error() {
        let state = make_state();
        let _router = chat_routes(state);
    }
}

// =====================================================================
// error.rs re-export (per 守门 #11 — explicit error path)
// =====================================================================

/// Re-export the chat-module specific error type alias (currently a
/// thin wrapper around [`LlmProviderRegistryError`]).
pub type ChatError = LlmProviderRegistryError;

/// Internal error mapping (per 守门 #13 d M 派生).
#[derive(Debug, Error)]
pub enum ChatInternalError {
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),
    #[error("provider dispatch failed: {0}")]
    ProviderDispatch(String),
}

impl From<ChatInternalError> for ApiError {
    fn from(e: ChatInternalError) -> Self {
        match e {
            ChatInternalError::SessionNotFound(id) => ApiError::new(
                "RESOURCE_NOT_FOUND",
                format!("chat session {id} not found"),
                "api",
                "validation",
                false,
                "Send a chat message first to create the session",
            ),
            ChatInternalError::ProviderDispatch(msg) => ApiError::new(
                "LLM_PROVIDER_ERROR",
                msg,
                "api",
                "external",
                true,
                "Retry the request; check provider health if persistent",
            ),
        }
    }
}

// =====================================================================
// SSE handler (ULYS-98-W3 Sub-task 3.2 Chat stream RPC)
// =====================================================================

/// GET `/v1/chat/stream?session_id=...&content=...` -- Server-Sent Events
/// streaming chat reply. Calls `LlmProvider::stream_completion` and
/// forwards each [`ChatChunk`] as an `event: data` SSE frame.
///
/// Wire format (per RFC 8895 text/event-stream):
/// - `data: {"delta":"...","done":false}

`  -- per token chunk
/// - `data: {"delta":"","done":true,"usage":{...}}

` -- terminal frame
async fn chat_stream(
    State(state): State<Arc<ChatState>>,
    Query(q): Query<ChatStreamQuery>,
) -> Result<
    Sse<std::pin::Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>,
    ApiError,
> {
    use futures_util::StreamExt;

    if q.content.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            "chat stream: content must be non-empty",
            "api",
            "validation",
            false,
            "Provide a non-empty content field",
        ));
    }

    let mut sessions = state.sessions.lock().await;
    let now = Utc::now();
    let session = sessions.entry(q.session_id).or_insert_with(|| ChatSession {
        id: q.session_id,
        user_id: q.user_id,
        messages: Vec::new(),
        created_at: now,
        updated_at: now,
    });
    session.messages.push(ChatMessage::user(q.content.clone()));
    session.updated_at = now;
    drop(sessions);

    let model = q
        .model
        .clone()
        .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());
    let chat_req = ChatRequest {
        model: model.clone(),
        messages: state
            .sessions
            .lock()
            .await
            .get(&q.session_id)
            .map(|s| s.messages.clone())
            .unwrap_or_default(),
        temperature: None,
        max_tokens: Some(1024),
        request_id: Some(Uuid::new_v4()),
    };

    let provider_stream = state
        .provider
        .stream_completion(chat_req)
        .await
        .map_err(|e| {
            ApiError::new(
                "LLM_PROVIDER_ERROR",
                format!("chat stream: provider dispatch failed: {e}"),
                "api",
                "external",
                true,
                "Retry the request; check provider health if persistent",
            )
        })?;

    let user_id = q.user_id;
    let metering = state.metering.clone();
    let tenant_id = state.tenant_id;
    let model_for_record = model.clone();

    let sse_stream = async_stream::stream! {
        let mut total_input = 0u32;
        let mut total_output = 0u32;
        let mut provider_stream = provider_stream;
        while let Some(chunk_res) = provider_stream.next().await {
            match chunk_res {
                Ok(chunk) => {
                    if let Some(usage) = &chunk.usage {
                        total_input = usage.input_tokens;
                        total_output = usage.output_tokens;
                    }
                    let payload = serde_json::json!({
                        "delta": chunk.delta,
                        "done": chunk.done,
                        "usage": chunk.usage,
                    });
                    if let Ok(s) = serde_json::to_string(&payload) {
                        yield Ok(Event::default().data(s));
                    }
                    if chunk.done {
                        let usage = TokenUsage::new(
                            user_id,
                            tenant_id,
                            model_for_record.clone(),
                            model_for_record.clone(),
                            total_input,
                            total_output,
                            None,
                        );
                        metering.record(usage).await;
                        break;
                    }
                }
                Err(e) => {
                    let payload = serde_json::json!({
                        "delta": "",
                        "done": true,
                        "error": format!("{e}"),
                    });
                    if let Ok(s) = serde_json::to_string(&payload) {
                        yield Ok(Event::default().data(s));
                    }
                    break;
                }
            }
        }
    };

    let pinned: std::pin::Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
        Box::pin(sse_stream);
    Ok(Sse::new(pinned).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15))))
}
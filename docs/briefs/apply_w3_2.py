"""Apply W3.2 SSE handler to crates/api/src/chat.rs (Windows path)."""
import sys

path = r"D:\Star\crates\api\src\chat.rs"
with open(path, encoding="utf-8") as f:
    src = f.read()
print(f"loaded {len(src)} bytes", flush=True)

old_imp = '''use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;'''

new_imp = '''use axum::{
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
use uuid::Uuid;'''

assert old_imp in src, "OLD_IMP_NOT_FOUND"
src = src.replace(old_imp, new_imp)
print("OK imports", flush=True)

old_router = '''pub fn chat_routes(state: Arc<ChatState>) -> Router {
    Router::new()
        .route("/v1/chat/send", post(chat_send))
        .route("/v1/chat/messages", get(chat_messages))
        .with_state(state)
}'''

new_router = '''pub fn chat_routes(state: Arc<ChatState>) -> Router {
    Router::new()
        .route("/v1/chat/send", post(chat_send))
        .route("/v1/chat/messages", get(chat_messages))
        .route("/v1/chat/stream", get(chat_stream))
        .with_state(state)
}'''

assert old_router in src, "OLD_ROUTER_NOT_FOUND"
src = src.replace(old_router, new_router)
print("OK router", flush=True)

old_msgs = '''/// GET `/v1/chat/messages?session_id=...` query.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessagesQuery {
    pub session_id: Uuid,
}'''

new_msgs = '''/// GET `/v1/chat/messages?session_id=...` query.
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
}'''

assert old_msgs in src, "OLD_MSGS_NOT_FOUND"
src = src.replace(old_msgs, new_msgs)
print("OK ChatStreamQuery", flush=True)

sentinel = '''/// Sentinel re-export so route handlers compile without unused warnings
/// when [`axum::response::IntoResponse`] is referenced for future
/// streaming replies (W3.2).
#[allow(dead_code)]
const _: fn() = || {
    let _: Option<StatusCode> = None;
    let _: Option<Box<dyn IntoResponse>> = None;
};'''

new_handler = '''// =====================================================================
// SSE handler (ULYS-98-W3 Sub-task 3.2 Chat stream RPC)
// =====================================================================

/// GET `/v1/chat/stream?session_id=...&content=...` -- Server-Sent Events
/// streaming chat reply. Calls `LlmProvider::stream_completion` and
/// forwards each [`ChatChunk`] as an `event: data` SSE frame.
///
/// Wire format (per RFC 8895 text/event-stream):
/// - `data: {"delta":"...","done":false}\\n\\n`  -- per token chunk
/// - `data: {"delta":"","done":true,"usage":{...}}\\n\\n` -- terminal frame
async fn chat_stream(
    State(state): State<Arc<ChatState>>,
    Query(q): Query<ChatStreamQuery>,
) -> Result<
    Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>,
    ApiError,
> {
    use futures_util::StreamExt;
    use std::pin::Pin;

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

    let pinned: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
        Box::pin(sse_stream);
    Ok(Sse::new(pinned).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15))))
}'''

assert sentinel in src, "SENTINEL_NOT_FOUND"
src = src.replace(sentinel, new_handler)
print("OK chat_stream handler", flush=True)

with open(path, "w", encoding="utf-8") as f:
    f.write(src)
print(f"DONE {len(src)} bytes", flush=True)

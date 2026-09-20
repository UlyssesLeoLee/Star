// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/routes/chat.rs` — W1 (ULYS-98-W1) Chat RPC stub (per
//! `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 1.3 backend Chat RPC stub").
//!
//! **Endpoints**:
//! - `POST /v1/chat/send`     — send a user message, get back the stub assistant reply.
//! - `GET  /v1/chat/messages` — fetch the full transcript for a session.
//!
//! **W1 stub behavior**: assistant content is the literal string
//! `"AI 暂未接入"` (per brief §"Sub-task 1.3" — "stub: 'AI 暂未接入' (W2 改真 LLM)").
//! W2 will replace the hardcoded reply by calling
//! `domain_llm::LlmProvider::chat_completion` against an Anthropic / OpenAI
//! provider — at that point the handler signature stays identical, callers
//! are insulated.
//!
//! **State**: in-memory `Arc<Mutex<HashMap<session_id, Vec<ChatMessageDto>>>>`
//! — fine for W1 unit tests + single-process demo; W2 will swap for a real
//! persistence layer (or `LlmProviderRegistry` keyed by session).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 所有 handler 返 `Result<Json<T>, (StatusCode, Json<ApiError>)>` — 显式
//!   错误路径, 不 `unwrap` / 不 `panic` (per 守门 #11).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// API error type (5-variant, 跟 collaboration::dto::ApiError 对齐独立不共享)
// =====================================================================

/// **ChatApiError** -- 5 变体 error type for the chat RPC surface (W1 stub).
///
/// 跟 [`crate::collaboration::dto::ApiError`] 同形但独立 (per 守门 #11 缺标比错标):
/// BFF chat 跟 collab 是平级 0 反向依赖, 0 共享 error type. W2 接真 LLM 时
/// 引入第 6 变体 `Upstream(_)`.
#[derive(Debug, Error)]
pub enum ChatApiError {
    /// 400 — request payload validation failed (empty session_id, empty content, etc).
    #[error("bad request: {0}")]
    BadRequest(String),
    /// 404 — referenced session does not exist (GET /v1/chat/messages on
    /// unknown session_id).
    #[error("not found: {0}")]
    NotFound(String),
    /// 500 — internal error (mutex poisoned, serialization failure).
    #[error("internal: {0}")]
    Internal(String),
}

impl ChatApiError {
    /// HTTP status code that this variant maps to.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ChatApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": self.to_string(),
            }
        }));
        (status, body).into_response()
    }
}

// =====================================================================
// ChatState (per-handler state injected via axum::State)
// =====================================================================

/// **ChatState** -- W1 stub in-memory store.
///
/// Single field: `sessions` keyed by `session_id` (UUID), value is the
/// ordered list of messages (user + assistant). `Mutex` is sufficient for
/// the stub; W2 will move to a connection pool / `LlmProviderRegistry`.
#[derive(Debug, Default, Clone)]
pub struct ChatState {
    /// session_id -> ordered transcript
    pub sessions: Arc<Mutex<HashMap<Uuid, Vec<ChatMessageDto>>>>,
}

impl ChatState {
    /// Construct a new empty `ChatState`.
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Build the axum `Router` with the 2 chat routes mounted and `self` as
    /// state.
    pub fn into_router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/v1/chat/send", post(send_chat_message))
            .route("/v1/chat/messages", get(get_chat_messages))
            .with_state(self)
    }
}

// =====================================================================
// DTOs (per brief §"Sub-task 1.3" — 2 endpoints × req/resp shapes)
// =====================================================================

/// **ChatSendReq** — POST /v1/chat/send request body (per brief §1.3).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ChatSendReq {
    /// Client-generated session UUID. Empty string rejected.
    pub session_id: String,
    /// User id (extracted from JWT in production; W1: passed verbatim).
    pub user_id: String,
    /// UTF-8 message content. Whitespace-only rejected.
    pub content: String,
}

impl ChatSendReq {
    fn validate(&self) -> Result<Uuid, ChatApiError> {
        if self.session_id.trim().is_empty() {
            return Err(ChatApiError::BadRequest(
                "session_id must be non-empty".to_string(),
            ));
        }
        let parsed = Uuid::parse_str(&self.session_id).map_err(|_| {
            ChatApiError::BadRequest(format!(
                "session_id must be a valid UUID, got {:?}",
                self.session_id
            ))
        })?;
        if self.user_id.trim().is_empty() {
            return Err(ChatApiError::BadRequest(
                "user_id must be non-empty".to_string(),
            ));
        }
        if self.content.trim().is_empty() {
            return Err(ChatApiError::BadRequest(
                "content must be non-empty".to_string(),
            ));
        }
        Ok(parsed)
    }
}

/// **ChatSendResp** — POST /v1/chat/send response body (per brief §1.3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatSendResp {
    /// Echo of the request session_id (canonical UUID form).
    pub session_id: Uuid,
    /// Server-generated message id (UUID v4).
    pub message_id: Uuid,
    /// Assistant reply text. W1 stub: literal `"AI 暂未接入"`.
    /// W2: real LLM reply from `domain_llm::LlmProvider::chat_completion`.
    pub assistant_content: String,
    /// Server-side timestamp at reply finalization.
    pub created_at: DateTime<Utc>,
}

/// **ChatMessagesQuery** — GET /v1/chat/messages query string.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ChatMessagesQuery {
    /// Session UUID to fetch. Empty string rejected.
    pub session_id: String,
}

impl ChatMessagesQuery {
    fn validate(&self) -> Result<Uuid, ChatApiError> {
        if self.session_id.trim().is_empty() {
            return Err(ChatApiError::BadRequest(
                "session_id must be non-empty".to_string(),
            ));
        }
        Uuid::parse_str(&self.session_id).map_err(|_| {
            ChatApiError::BadRequest(format!(
                "session_id must be a valid UUID, got {:?}",
                self.session_id
            ))
        })
    }
}

/// **ChatMessageDto** — one transcript entry (user or assistant).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessageDto {
    /// Speaker role (`user` / `assistant`; `system` deferred to W2).
    pub role: String,
    /// UTF-8 content text.
    pub content: String,
    /// Server-side timestamp.
    pub created_at: DateTime<Utc>,
}

/// **ChatMessagesResp** — GET /v1/chat/messages response body (per brief §1.3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessagesResp {
    /// Echo of the requested session_id (canonical UUID form).
    pub session_id: Uuid,
    /// Ordered transcript (oldest first).
    pub messages: Vec<ChatMessageDto>,
}

// =====================================================================
// Route handlers (per brief §1.3 — POST + GET)
// =====================================================================

/// **POST /v1/chat/send** — append a user message, append a stub assistant
/// reply, return the assistant message envelope.
///
/// W1 stub behavior: assistant content is the literal `"AI 暂未接入"` (per
/// brief §"Sub-task 1.3" — "W2 改真 LLM"). The session is created lazily on
/// first send, so the handler does not 404 on a fresh session.
async fn send_chat_message(
    State(state): State<Arc<ChatState>>,
    Json(req): Json<ChatSendReq>,
) -> Result<Json<ChatSendResp>, ChatApiError> {
    let session_id = req.validate()?;

    let now = Utc::now();
    let user_msg = ChatMessageDto {
        role: "user".to_string(),
        content: req.content.clone(),
        created_at: now,
    };
    // Brief §1.3: stub assistant content is `"AI 暂未接入"` (W2 → real LLM).
    let assistant_msg = ChatMessageDto {
        role: "assistant".to_string(),
        content: "AI 暂未接入".to_string(),
        created_at: now,
    };
    let message_id = Uuid::new_v4();

    // Lock + append (recover from poisoned mutex — surface as 500).
    let mut sessions = state.sessions.lock().map_err(|e| {
        ChatApiError::Internal(format!("chat sessions mutex poisoned: {e}"))
    })?;
    let transcript = sessions.entry(session_id).or_default();
    transcript.push(user_msg);
    transcript.push(assistant_msg);

    Ok(Json(ChatSendResp {
        session_id,
        message_id,
        assistant_content: "AI 暂未接入".to_string(),
        created_at: now,
    }))
}

/// **GET /v1/chat/messages?session_id=…** — fetch the ordered transcript for a
/// session. Returns 404 if the session was never written to (no prior send).
async fn get_chat_messages(
    State(state): State<Arc<ChatState>>,
    Query(q): Query<ChatMessagesQuery>,
) -> Result<Json<ChatMessagesResp>, ChatApiError> {
    let session_id = q.validate()?;

    let sessions = state.sessions.lock().map_err(|e| {
        ChatApiError::Internal(format!("chat sessions mutex poisoned: {e}"))
    })?;
    let messages = sessions
        .get(&session_id)
        .cloned()
        .ok_or_else(|| ChatApiError::NotFound(format!("session {session_id}")))?;

    Ok(Json(ChatMessagesResp {
        session_id,
        messages,
    }))
}

// =====================================================================
// Router factory (per-handler — used by lib.rs build_router)
// =====================================================================

/// **chat_routes** — build the 2-route axum router with a fresh `ChatState`.
///
/// Caller (typically `crate::build_router`) merges the result into the
/// top-level application router. The freshly-allocated `ChatState` lives
/// for the router's lifetime — fine for W1 stub.
pub fn chat_routes() -> Router {
    ChatState::new().into_router()
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 — >= 1 unit test per module, brief §1.3 要 3 tests)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    fn app() -> Router {
        chat_routes()
    }

    /// **Test 1 (unit)**: POST /v1/chat/send happy — returns 200 + assistant_content
    /// `"AI 暂未接入"` (per brief §"Sub-task 1.3" W1 stub behavior).
    #[tokio::test]
    async fn post_chat_send_returns_stub_assistant_reply() {
        let app = app();
        let session_id = Uuid::new_v4();
        let body = serde_json::json!({
            "session_id": session_id.to_string(),
            "user_id": "user-1",
            "content": "hello",
        });
        let req = Request::builder()
            .method("POST")
            .uri("/v1/chat/send")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        let parsed: ChatSendResp = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.session_id, session_id);
        assert_eq!(parsed.assistant_content, "AI 暂未接入");
        assert_ne!(parsed.message_id, Uuid::nil());
    }

    /// **Test 2 (unit)**: POST /v1/chat/send validation — empty content
    /// rejected with 400.
    #[tokio::test]
    async fn post_chat_send_rejects_empty_content_with_400() {
        let app = app();
        let session_id = Uuid::new_v4();
        let body = serde_json::json!({
            "session_id": session_id.to_string(),
            "user_id": "user-1",
            "content": "   ",
        });
        let req = Request::builder()
            .method("POST")
            .uri("/v1/chat/send")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// **Test 3 (unit)**: GET /v1/chat/messages happy — send then fetch returns
    /// the full transcript (user + assistant).
    #[tokio::test]
    async fn get_chat_messages_returns_transcript_after_send() {
        let app = app();
        let session_id = Uuid::new_v4();
        // 1. POST first.
        let post_body = serde_json::json!({
            "session_id": session_id.to_string(),
            "user_id": "user-1",
            "content": "ping",
        });
        let post_req = Request::builder()
            .method("POST")
            .uri("/v1/chat/send")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&post_body).unwrap()))
            .unwrap();
        let post_resp = app.clone().oneshot(post_req).await.unwrap();
        assert_eq!(post_resp.status(), StatusCode::OK);

        // 2. GET the transcript.
        let get_req = Request::builder()
            .method("GET")
            .uri(format!("/v1/chat/messages?session_id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let get_resp = app.oneshot(get_req).await.unwrap();
        assert_eq!(get_resp.status(), StatusCode::OK);
        let bytes = to_bytes(get_resp.into_body(), 4096).await.unwrap();
        let parsed: ChatMessagesResp = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.session_id, session_id);
        assert_eq!(parsed.messages.len(), 2);
        assert_eq!(parsed.messages[0].role, "user");
        assert_eq!(parsed.messages[0].content, "ping");
        assert_eq!(parsed.messages[1].role, "assistant");
        assert_eq!(parsed.messages[1].content, "AI 暂未接入");
    }

    /// **Test 4 (unit, bonus)**: GET on unknown session returns 404.
    #[tokio::test]
    async fn get_chat_messages_unknown_session_returns_404() {
        let app = app();
        let session_id = Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/v1/chat/messages?session_id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
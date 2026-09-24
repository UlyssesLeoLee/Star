// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/completion.rs` — W2 (ULYS-98-W2) AI 补全 RPC.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.2":
//! - POST `/v1/completion/inline` — inline (cursor-position) code completion
//! - input: file_path + cursor position + prefix/suffix (8KB max) + language
//! - output: completion + token_usage
//! - calls `domain_llm::DispatchProvider::chat_completion` with a
//!   purpose-built system prompt for code completion.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #10 + 守门 #13 a W/T/M + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - DTOs derive `Serialize` / `Deserialize` (per 守门 #11).
//! - audit log every completion call (per 守门 #13 d).
//! 5 域 Lead 真人到位前 Mavis 临时代签.

#![allow(missing_docs)] // per crates/domain-llm/src/lib.rs precedent — module-level docs present.

use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use domain_llm::chat::{ChatMessage, ChatRequest, ChatRole};
use domain_llm::TokenUsage;

use super::chat::ChatState;
use crate::ApiError;

// =====================================================================
// Limits
// =====================================================================

/// Maximum prefix / suffix length accepted by `/v1/completion/inline`
/// (per brief §"Sub-task 2.2"). Inputs above this are rejected with
/// `VALIDATION_FAILED` rather than silently truncated.
pub const MAX_PREFIX_BYTES: usize = 8 * 1024;
pub const MAX_SUFFIX_BYTES: usize = 8 * 1024;

/// Default model used for inline completion when caller omits `model`.
pub const DEFAULT_COMPLETION_MODEL: &str = "claude-3-5-sonnet-20241022";

/// System prompt for inline completion — instructs the model to return
/// **only the missing text** between `prefix` and `suffix`, no markdown
/// fences, no commentary.
pub const COMPLETION_SYSTEM_PROMPT: &str = "\
You are an inline code completion assistant. The user shows you the text \
before the cursor (`prefix`) and after the cursor (`suffix`). Reply with \
ONLY the code that should be inserted at the cursor position, without \
markdown fences, without commentary, without leading or trailing newlines \
unless they are syntactically required.";

// =====================================================================
// Request DTOs
// =====================================================================

/// POST `/v1/completion/inline` request body.
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionInlineRequest {
    /// File path (relative to worktree root).
    pub file_path: String,
    /// Cursor line (1-indexed).
    pub cursor_line: u32,
    /// Cursor column (1-indexed, byte offset within the line).
    pub cursor_col: u32,
    /// Text immediately before the cursor (max 8 KiB).
    pub prefix: String,
    /// Text immediately after the cursor (max 8 KiB).
    pub suffix: String,
    /// Language id (e.g. `rust`, `typescript`, `python`).
    pub language: String,
    /// Caller user id (per 守门 #10).
    pub user_id: Uuid,
    /// Optional model override.
    pub model: Option<String>,
}

impl CompletionInlineRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.file_path.trim().is_empty() {
            return Err("completion: file_path must be non-empty");
        }
        if self.prefix.len() > MAX_PREFIX_BYTES {
            return Err("completion: prefix exceeds 8 KiB");
        }
        if self.suffix.len() > MAX_SUFFIX_BYTES {
            return Err("completion: suffix exceeds 8 KiB");
        }
        Ok(())
    }
}

// =====================================================================
// Response DTOs
// =====================================================================

/// POST `/v1/completion/inline` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionInlineResponse {
    /// The completion text (may be empty if the model declines).
    pub completion: String,
    /// Model used (echo of input or default).
    pub model: String,
    /// Token usage for this call.
    pub token_usage: TokenUsage,
    /// Server-side timestamp.
    pub created_at: chrono::DateTime<Utc>,
}

// =====================================================================
// Router
// =====================================================================

/// 1-route router factory: `POST /v1/completion/inline`.
pub fn completion_routes(state: Arc<ChatState>) -> Router {
    Router::new()
        .route("/v1/completion/inline", post(completion_inline))
        .with_state(state)
}

// =====================================================================
// Handlers
// =====================================================================

/// POST `/v1/completion/inline` — inline code completion.
async fn completion_inline(
    State(state): State<Arc<ChatState>>,
    Json(req): Json<CompletionInlineRequest>,
) -> Result<Json<CompletionInlineResponse>, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            e,
            "api",
            "validation",
            false,
            "Trim prefix/suffix to <= 8 KiB and provide a non-empty file_path",
        ));
    }

    let model = req
        .model
        .clone()
        .unwrap_or_else(|| DEFAULT_COMPLETION_MODEL.to_string());

    // Build the chat request. The prompt intentionally omits the
    // `prefix`/`suffix` from the user content — we ship them verbatim so
    // providers without a dedicated `/completions` endpoint can still
    // answer (Anthropic Messages API requires messages[] — no
    // /completions route as of 2026-09-20).
    let user_content = format!(
        "Language: {lang}\nFile: {path}\nLine: {line}, Col: {col}\n\n\
         <<<PREFIX>>>\n{prefix}\n<<<END_PREFIX>>>\n\n\
         <<<SUFFIX>>>\n{suffix}\n<<<END_SUFFIX>>>\n\n\
         Continue the code at the cursor.",
        lang = req.language,
        path = req.file_path,
        line = req.cursor_line,
        col = req.cursor_col,
        prefix = req.prefix,
        suffix = req.suffix,
    );

    let chat_req = ChatRequest {
        model: model.clone(),
        messages: vec![
            ChatMessage {
                role: ChatRole::System,
                content: COMPLETION_SYSTEM_PROMPT.to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: user_content,
            },
        ],
        temperature: Some(0.2),
        max_tokens: Some(256),
        request_id: Some(Uuid::new_v4()),
        thinking_level: None,
    };

    let resp = state
        .provider
        .chat_completion(chat_req)
        .await
        .map_err(|e| {
            ApiError::new(
                "LLM_PROVIDER_ERROR",
                format!("completion: provider dispatch failed: {e}"),
                "api",
                "external",
                true,
                "Retry the request; check provider health if persistent",
            )
        })?;

    let input_tokens = ((req.prefix.chars().count()
        + req.suffix.chars().count()
        + COMPLETION_SYSTEM_PROMPT.chars().count()) as u32)
        .div_ceil(4);
    let output_tokens = (resp.message.content.chars().count() as u32).div_ceil(4);

    let usage = TokenUsage::new(
        req.user_id,
        state.tenant_id,
        model.clone(),
        model.clone(),
        input_tokens,
        output_tokens,
        Some(resp.id),
    );
    state.metering.record(usage.clone()).await;

    Ok(Json(CompletionInlineResponse {
        completion: resp.message.content,
        model: resp.model,
        token_usage: usage,
        created_at: Utc::now(),
    }))
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use domain_llm::provider::mock::MockProvider;
    use domain_llm::{LlmProvider, MeteringStore};

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

    fn sample_request() -> CompletionInlineRequest {
        CompletionInlineRequest {
            file_path: "src/lib.rs".into(),
            cursor_line: 12,
            cursor_col: 8,
            prefix: "fn hello() {\n    prin".into(),
            suffix: "\n}".into(),
            language: "rust".into(),
            user_id: user(),
            model: None,
        }
    }

    #[test]
    fn completion_request_validate_rejects_empty_path() {
        let mut r = sample_request();
        r.file_path = "".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn completion_request_validate_rejects_oversized_prefix() {
        let mut r = sample_request();
        r.prefix = "x".repeat(MAX_PREFIX_BYTES + 1);
        assert!(r.validate().is_err());
    }

    #[test]
    fn completion_request_validate_rejects_oversized_suffix() {
        let mut r = sample_request();
        r.suffix = "y".repeat(MAX_SUFFIX_BYTES + 1);
        assert!(r.validate().is_err());
    }

    #[tokio::test]
    async fn completion_handler_returns_assistant_reply_and_records_metering() {
        let state = make_state();
        let req = sample_request();
        let resp = completion_inline(State(state.clone()), Json(req))
            .await
            .unwrap()
            .0;
        // Mock provider echoes the user content — completion contains
        // the prompt we built.
        assert!(resp.completion.contains("[mock]"));
        assert_eq!(resp.model, DEFAULT_COMPLETION_MODEL);
        assert_eq!(resp.token_usage.user_id, user());
        assert!(resp.token_usage.total_tokens > 0);

        let agg = state.metering.aggregate_for_user(user()).await;
        assert_eq!(agg.call_count, 1);
    }

    #[tokio::test]
    async fn completion_handler_rejects_invalid_request() {
        let state = make_state();
        let mut r = sample_request();
        r.file_path = "".into();
        let err = completion_inline(State(state), Json(r)).await.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn completion_handler_uses_provided_model_override() {
        let state = make_state();
        let mut r = sample_request();
        r.model = Some("gpt-4o-mini".to_string());
        let resp = completion_inline(State(state), Json(r)).await.unwrap().0;
        assert_eq!(resp.model, "gpt-4o-mini");
    }

    #[test]
    fn completion_routes_builds_router_without_error() {
        let state = make_state();
        let _router = completion_routes(state);
    }
}

// =====================================================================
// Internal error
// =====================================================================

#[derive(Debug, Error)]
pub enum CompletionInternalError {
    #[error("provider dispatch failed: {0}")]
    ProviderDispatch(String),
}

impl From<CompletionInternalError> for ApiError {
    fn from(e: CompletionInternalError) -> Self {
        ApiError::new(
            "LLM_PROVIDER_ERROR",
            e.to_string(),
            "api",
            "external",
            true,
            "Retry the request; check provider health if persistent",
        )
    }
}
// =====================================================================
// Composer routes (ULYS-98-W3.4 backend)
// =====================================================================
// Per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 3.4 Composer
// 多文件 diff RPC" + §"Sub-task 4.2 action-engine 加 3 类 AI Action".
//
// v0.0.1 stub: returns 1 stub diff per file. W4.2 wires real LLM dispatch
// via DispatchProvider + action-engine AiComposerEdit ActionType.

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerEditApiRequest {
    pub files: Vec<ComposerFileApi>,
    pub instruction: String,
    pub user_id: Uuid,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerFileApi {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEditOut {
    pub path: String,
    pub diff: String,
    pub new_content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposerEditApiResponse {
    pub session_id: Uuid,
    pub edits: Vec<FileEditOut>,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerApplyApiRequest {
    pub session_id: Uuid,
    pub paths: Vec<String>,
    #[allow(dead_code)]
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposerApplyApiResponse {
    pub applied_paths: Vec<String>,
    pub skipped_paths: Vec<String>,
}

/// Append `/v1/composer/edit` + `/v1/composer/apply` routes to an existing
/// Router (shares ChatState for metering + provider).
pub fn extend_with_composer(router: Router, state: Arc<ChatState>) -> Router {
    router
        .route(
            "/v1/composer/edit",
            post(composer_edit_handler).with_state(state.clone()),
        )
        .route(
            "/v1/composer/apply",
            post(composer_apply_handler).with_state(state.clone()),
        )
}

async fn composer_edit_handler(
    State(state): State<Arc<ChatState>>,
    Json(req): Json<ComposerEditApiRequest>,
) -> Result<Json<ComposerEditApiResponse>, ApiError> {
    if req.files.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            "composer edit: files must be non-empty",
            "api",
            "validation",
            false,
            "Provide at least one file in the selection",
        ));
    }
    if req.instruction.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            "composer edit: instruction must be non-empty",
            "api",
            "validation",
            false,
            "Provide a non-empty instruction",
        ));
    }
    let session_id = req.session_id.unwrap_or_else(Uuid::new_v4);
    let model = req
        .model
        .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());
    let edits: Vec<FileEditOut> = req
        .files
        .into_iter()
        .map(|f| FileEditOut {
            path: f.path,
            diff: format!("# composer stub (instruction: {})", req.instruction),
            new_content: f.content,
        })
        .collect();
    let _ = state; // v0.0.1 no-op; W4.2 wires provider + metering
    Ok(Json(ComposerEditApiResponse {
        session_id,
        edits,
        model,
    }))
}

async fn composer_apply_handler(
    State(state): State<Arc<ChatState>>,
    Json(req): Json<ComposerApplyApiRequest>,
) -> Result<Json<ComposerApplyApiResponse>, ApiError> {
    let _ = state;
    Ok(Json(ComposerApplyApiResponse {
        applied_paths: req.paths,
        skipped_paths: vec![],
    }))
}

#[cfg(test)]
mod composer_tests {
    use super::*;

    #[test]
    fn composer_routes_helper_builds_router_without_error() {
        let _ = extend_with_composer(
            Router::new(),
            crate::chat::ChatState::new(
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
            ),
        );
    }
}

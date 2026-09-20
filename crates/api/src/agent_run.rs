// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/agent_run.rs` -- ULYS-98 v1.0.1 follow-up: /v1/agent/run backend.
//!
//! Per docs/specs/star-cursor-min-spec.md section 1.4 Agent API:
//! POST /v1/agent/run {session_id, user_id, prompt, model?, max_steps?}
//!   -> {session_id, steps: AiStep[], final_answer: string?}
//!
//! This module wires domain-agent::ai_session_inline::run_agent_loop
//! (W4.3) with agent-bridge::tool::dispatch (W4.1) into an axum 0.8
//! handler. v1.0.0 had the frontend AgentRunner + ai_session_inline
//! stub but no HTTP endpoint; v1.0.1 adds the full backend.

#![allow(missing_docs)]

use std::sync::Arc;

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use agent_bridge::tool::ToolCall;
use domain_agent::ai_session_inline::{run_agent_loop, AiSessionError};
use domain_llm::LlmProvider;

use crate::ApiError;
use crate::chat::ChatState;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentRunApiRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
}

fn default_max_steps() -> u32 {
    5
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentRunApiResponse {
    pub session_id: Uuid,
    pub steps: Vec<AgentStepApi>,
    pub final_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStepApi {
    pub step_id: Uuid,
    pub thought: String,
    pub tool_call: Option<ToolCallApi>,
    pub observation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallApi {
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum AgentRunError {
    #[error("agent session error: {0}")]
    Session(String),
    #[error("tool dispatch error: {0}")]
    Tool(String),
    #[error("max steps exceeded")]
    MaxStepsExceeded,
}

impl From<AiSessionError> for AgentRunError {
    fn from(e: AiSessionError) -> Self {
        match e {
            AiSessionError::MaxStepsExceeded => Self::MaxStepsExceeded,
            AiSessionError::Provider(p) => Self::Session(format!("provider: {p}")),
            AiSessionError::Tool(t) => Self::Tool(t),
            AiSessionError::InvalidState(s) => Self::Session(s),
        }
    }
}

pub fn agent_routes(state: Arc<ChatState>) -> Router {
    Router::new()
        .route("/v1/agent/run", post(agent_run))
        .with_state(state)
}

async fn agent_run(
    State(state): State<Arc<ChatState>>,
    Json(req): Json<AgentRunApiRequest>,
) -> Result<Json<AgentRunApiResponse>, ApiError> {
    if req.prompt.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            "agent run: prompt must be non-empty",
            "api",
            "validation",
            false,
            "Provide a non-empty prompt",
        ));
    }
    if req.max_steps == 0 || req.max_steps > 50 {
        return Err(ApiError::new(
            "VALIDATION_FAILED",
            format!("agent run: max_steps must be in 1..=50, got {}", req.max_steps),
            "api",
            "validation",
            false,
            "Provide max_steps in 1..=50",
        ));
    }

    let provider: Arc<dyn LlmProvider> = state.provider.clone();
    let session = run_agent_loop(provider, req.prompt.clone(), req.max_steps)
        .await
        .map_err(|e| {
            let agent_err: AgentRunError = e.into();
            let (code, retry) = match &agent_err {
                AgentRunError::MaxStepsExceeded => ("MAX_STEPS_EXCEEDED", false),
                AgentRunError::Session(_) => ("AGENT_SESSION_ERROR", true),
                AgentRunError::Tool(_) => ("AGENT_TOOL_ERROR", true),
            };
            ApiError::new(
                code,
                format!("agent run: {agent_err}"),
                "api",
                "external",
                retry,
                "Retry with smaller prompt or fewer steps",
            )
        })?;

    let steps = session
        .steps
        .into_iter()
        .map(|s| AgentStepApi {
            step_id: s.step_id,
            thought: s.thought,
            tool_call: s.tool_call.and_then(|v| {
                serde_json::from_value::<ToolCall>(v)
                    .ok()
                    .map(|tc| ToolCallApi {
                        tool: tool_name_str(&tc),
                        args: serde_json::to_value(&tc).unwrap_or(serde_json::Value::Null),
                    })
            }),
            observation: s.observation,
        })
        .collect();

    Ok(Json(AgentRunApiResponse {
        session_id: req.session_id,
        steps,
        final_answer: session.final_answer,
    }))
}

fn tool_name_str(tc: &ToolCall) -> String {
    match tc {
        ToolCall::ReadFile { .. } => "read_file".to_string(),
        ToolCall::EditFile { .. } => "edit_file".to_string(),
        ToolCall::RunCmd { .. } => "run_cmd".to_string(),
        ToolCall::WebSearch { .. } => "web_search".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::ChatState;

    fn state() -> Arc<ChatState> {
        ChatState::new(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
        )
    }

    #[test]
    fn agent_routes_builds_router_without_error() {
        let s = state();
        let _ = agent_routes(s);
    }

    #[tokio::test]
    async fn agent_run_rejects_empty_prompt() {
        let s = state();
        let err = agent_run(
            State(s),
            Json(AgentRunApiRequest {
                session_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                prompt: "   ".to_string(),
                model: None,
                max_steps: 3,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn agent_run_rejects_zero_max_steps() {
        let s = state();
        let err = agent_run(
            State(s),
            Json(AgentRunApiRequest {
                session_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                prompt: "hi".to_string(),
                model: None,
                max_steps: 0,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn agent_run_with_mock_returns_state() {
        let s = state();
        let out = agent_run(
            State(s),
            Json(AgentRunApiRequest {
                session_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                prompt: "hello".to_string(),
                model: None,
                max_steps: 3,
            }),
        )
        .await
        .unwrap();
        assert!(!out.steps.is_empty(), "expected at least one step");
    }

    #[test]
    fn tool_name_str_maps_all_4_tools() {
        let r = ToolCall::ReadFile { path: "/x".into() };
        assert_eq!(tool_name_str(&r), "read_file");
        let e = ToolCall::EditFile { path: "/x".into(), new_content: "y".into() };
        assert_eq!(tool_name_str(&e), "edit_file");
        let c = ToolCall::RunCmd { cmd: "ls".into(), timeout_secs: Some(5) };
        assert_eq!(tool_name_str(&c), "run_cmd");
        let w = ToolCall::WebSearch { query: "x".into(), max_results: Some(3) };
        assert_eq!(tool_name_str(&w), "web_search");
    }
}

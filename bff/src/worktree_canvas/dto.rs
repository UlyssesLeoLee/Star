// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/dto.rs` — Request/Response/Event/SSE message types
//! for the AI Worktree Graph Canvas BFF module (per DD-WORKTREE-CANVAS-001 §20-§21
//! + WORKTREE-CANVAS-IMPL-PLAN-001 §4.13 + spec §4.4 §4.5).
//!
//! 5 类 types:
//! - **REST request/response**: `WorktreeResponse` / `CreateWorktreeRequest` /
//!   `ActionRequest` / `ActionResponse` / `SearchRequest` / `SearchResponse` /
//!   `ExplainResponse` / `WorktreeGraphResponse` (4 详细 schema per DD §20 + 7
//!   衍生 per IMPL-PLAN §4.13.2).
//! - **SSE event**: `WorktreeSseEvent` enum (15 变体, 镜像 spec §4.5 `WorktreeEvent`).
//! - **WebSocket message**: `WsClientMessage` + `WsServerMessage` (UI ↔ BFF realtime).
//! - **Health/Risk/Risk**: `HealthResponse` / `RiskResponse` (per DD §12-§14).
//! - **API error**: `WorktreeApiError` (5 变体, 跟 V0.1 collab `ApiError` 同形独立).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #6 v2 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 6-field error (per 守门 #6 v2): code / message / source / location / context / trace_id.
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// ID re-exports (跟 graph-core 共享 ID 类型 via re-export, 0 重新定义)
// =====================================================================

pub use graph_core::types::{
    AgentId, BranchId, IssueId, PRId, RepoId, RiskScore, TaskId, TestId, UserId, WorktreeId,
};

/// Branch name string (per DD §7.1, max 255 chars per git ref).
pub type BranchName = String;

/// Commit SHA hex string (per DD §7.6, 7-40 hex chars).
pub type CommitSha = String;

/// Symbol kind (per graph-core::state::SymbolKind re-export, 7 变体).
pub use graph_core::state::{
    AgentResultState, AgentStatus, HumanState, IssueState, MachineState, MergeStrategy, PRState,
    SymbolKind, TaskStatus, TestState, TestStatus, TokenUsage,
};

// =====================================================================
// Health Score (per DD §12 + IMPL-PLAN §3 13 因素)
// =====================================================================

/// Health score 0-100 + deduction breakdown (per DD §12 + spec §3.4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthScoreDto {
    /// 0-100 (clamped, per DD §12).
    pub value: u8,
    /// 扣分项 (per factor), 解释性.
    pub deductions: Vec<HealthDeduction>,
    /// 重算时间 (UTC).
    pub computed_at: DateTime<Utc>,
}

/// 单条扣分项 (per DD §12.3 解释性).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthDeduction {
    /// 扣分因素 (per DD §12 表).
    pub factor: String,
    /// 扣分值 (负数).
    pub points: i8,
    /// 解释 (per DD §12 R8 Explainable State).
    pub reason: String,
}

// =====================================================================
// Worktree Response (per DD §20.1)
// =====================================================================

/// `GET /v1/worktree-canvas/worktrees/:id` 响应 (per DD §20.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeResponse {
    pub id: WorktreeId,
    pub repo_id: RepoId,
    pub name: String,
    pub branch: BranchName,
    pub human_state: HumanState,
    pub machine_state: MachineState,
    pub ahead: u32,
    pub behind: u32,
    pub dirty: bool,
    pub health_score: HealthScoreDto,
    pub last_activity: DateTime<Utc>,
    pub last_activity_relative: String,
    pub agent: Option<AgentSummary>,
    pub task: Option<TaskSummary>,
    pub risk_count: u32,
    pub test_state: TestState,
    pub locked: bool,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: AgentId,
    pub agent_type: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: TaskId,
    pub title: String,
}

// =====================================================================
// Create / Action / Search / Explain / Health / Risk (per IMPL-PLAN §4.13.2 + DD §20.2-§20.4)
// =====================================================================

/// `POST /v1/worktree-canvas/worktrees` 创建请求 (per IMPL-PLAN §4.13.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorktreeRequest {
    pub repo_id: RepoId,
    pub branch: BranchName,
    pub base_branch: Option<BranchName>,
    pub task_id: Option<TaskId>,
    pub idempotency_key: Uuid,
}

/// `POST /v1/worktree-canvas/worktrees/:id/actions/:action_type` (18 Action per spec §4.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub confirm: bool,
    pub idempotency_key: Uuid,
}

/// Action 响应 (per DD §20.2 + spec §4.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    pub success: bool,
    pub worktree_id: WorktreeId,
    pub new_state: HumanState,
    pub duration_ms: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// 确认对话 (per DD §20.2 409 响应, Destructive Action).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmDialog {
    pub title: String,
    pub description: String,
    pub impact: ImpactDto,
    pub confirm_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactDto {
    pub affected_count: u32,
    pub affected_paths: Vec<String>,
    pub reversible: bool,
    pub estimated_duration: String,
}

/// `POST /v1/worktree-canvas/search` (per DD §20.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_query_type")]
    pub query_type: String, // "dsl" | "nl"
}

fn default_query_type() -> String {
    "dsl".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub worktree_ids: Vec<WorktreeId>,
    pub total: u32,
    pub duration_ms: u64,
}

/// `GET /v1/worktree-canvas/nl-query` (per IMPL-PLAN §4.13.2 + spec §20 NL query).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlQueryRequest {
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlQueryResponse {
    pub worktree_ids: Vec<WorktreeId>,
    pub translated_query: String,
    pub explanation: String,
}

/// `GET /v1/worktree-canvas/repositories` 响应 (per IMPL-PLAN §4.13.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDto {
    pub id: RepoId,
    pub name: String,
    pub default_branch: BranchName,
    pub worktree_count: u32,
}

/// `GET /v1/worktree-canvas/worktrees` 列表响应 (per IMPL-PLAN §4.13.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorktreesResponse {
    pub items: Vec<WorktreeResponse>,
    pub total: u32,
    pub next_cursor: Option<String>,
}

/// `GET /v1/worktree-canvas/health` 响应 (per IMPL-PLAN §4.13.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummaryDto {
    pub overall_score: u8,
    pub healthy: u32,
    pub at_risk: u32,
    pub diverged: u32,
    pub conflict: u32,
    pub stale: u32,
}

/// `GET /v1/worktree-canvas/risks` 响应 (per IMPL-PLAN §4.13.2 + DD §13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResponse {
    pub worktree_id: WorktreeId,
    pub risk_score: RiskScore,
    pub risk_type: String,
    pub shared_files: Vec<String>,
    pub detected_at: DateTime<Utc>,
    pub reason: String,
    pub confidence: f32,
}

/// `GET /v1/worktree-canvas/health` 健康检查响应 (per brief §"15 SSE 端点").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BffHealthDto {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub worktree_canvas_enabled: bool,
}

// =====================================================================
// Graph response (per IMPL-PLAN §4.13.2 + DD §8 Graph Schema)
// =====================================================================

/// `GET /v1/worktree-canvas/worktrees/:id/graph` 响应 (focus N-hop subgraph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeGraphResponse {
    pub center: WorktreeId,
    pub hop: u8,
    pub nodes: Vec<GraphNodeDto>,
    pub edges: Vec<GraphEdgeDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeDto {
    pub id: Uuid, // node ID, not worktree ID
    pub kind: String, // 11 Node kinds per DD §7
    pub label: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeDto {
    pub source: Uuid,
    pub target: Uuid,
    pub kind: String, // 13 Edge kinds per DD §8
    pub risk_score: Option<RiskScore>,
    pub metadata: serde_json::Value,
}

// =====================================================================
// SSE Event (15 变体, per spec §4.5 + DD §19 + IMPL-PLAN §4.13.3)
// =====================================================================

/// 15 SSE event 变体 (per spec §4.5 WorktreeEvent enum).
///
/// SSE wire format: `event: <PascalCase>\ndata: <json>\n\n`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum WorktreeSseEvent {
    WorktreeCreated {
        worktree_id: WorktreeId,
        repository_id: RepoId,
        branch: BranchName,
    },
    WorktreeDeleted {
        worktree_id: WorktreeId,
    },
    WorktreeChanged {
        worktree_id: WorktreeId,
        changes: Vec<WorktreeFieldChange>,
    },
    WorktreeMerged {
        worktree_id: WorktreeId,
        target_branch: BranchName,
    },
    BranchUpdated {
        branch: BranchName,
        repository_id: RepoId,
        new_head: CommitSha,
    },
    MainUpdated {
        repository_id: RepoId,
        new_head: CommitSha,
    },
    AgentStarted {
        agent_session_id: Uuid,
        worktree_id: WorktreeId,
        agent_type: String,
    },
    AgentStopped {
        agent_session_id: Uuid,
        reason: String,
    },
    TaskChanged {
        task_id: TaskId,
        status: TaskStatus,
    },
    TestCompleted {
        worktree_id: WorktreeId,
        run_id: Uuid,
        result: TestState,
    },
    RiskDetected {
        worktree_id: WorktreeId,
        risk_type: String,
        risk_score: RiskScore,
    },
    RiskResolved {
        worktree_id: WorktreeId,
        risk_type: String,
    },
    HealthChanged {
        worktree_id: WorktreeId,
        old_score: u8,
        new_score: u8,
    },
    RelationCreated {
        edge_id: Uuid,
        kind: String,
        source: Uuid,
        target: Uuid,
    },
    RelationRemoved {
        edge_id: Uuid,
    },
}

/// Worktree field change (per spec §4.5 WorktreeChanged.changes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeFieldChange {
    pub field: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

impl WorktreeSseEvent {
    /// SSE event name (PascalCase, per spec §4.5 wire format).
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::WorktreeCreated { .. } => "WorktreeCreated",
            Self::WorktreeDeleted { .. } => "WorktreeDeleted",
            Self::WorktreeChanged { .. } => "WorktreeChanged",
            Self::WorktreeMerged { .. } => "WorktreeMerged",
            Self::BranchUpdated { .. } => "BranchUpdated",
            Self::MainUpdated { .. } => "MainUpdated",
            Self::AgentStarted { .. } => "AgentStarted",
            Self::AgentStopped { .. } => "AgentStopped",
            Self::TaskChanged { .. } => "TaskChanged",
            Self::TestCompleted { .. } => "TestCompleted",
            Self::RiskDetected { .. } => "RiskDetected",
            Self::RiskResolved { .. } => "RiskResolved",
            Self::HealthChanged { .. } => "HealthChanged",
            Self::RelationCreated { .. } => "RelationCreated",
            Self::RelationRemoved { .. } => "RelationRemoved",
        }
    }

    /// 全部 15 变体名 (per spec §4.5).
    pub const ALL_NAMES: &'static [&'static str] = &[
        "WorktreeCreated",
        "WorktreeDeleted",
        "WorktreeChanged",
        "WorktreeMerged",
        "BranchUpdated",
        "MainUpdated",
        "AgentStarted",
        "AgentStopped",
        "TaskChanged",
        "TestCompleted",
        "RiskDetected",
        "RiskResolved",
        "HealthChanged",
        "RelationCreated",
        "RelationRemoved",
    ];
}

// =====================================================================
// WebSocket message (per DD §21 + IMPL-PLAN §4.13.4)
// =====================================================================

/// UI → BFF WebSocket 消息 (per DD §21 WS protocol).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Subscribe to specific event types (empty = all).
    Subscribe { events: Vec<String> },
    /// Unsubscribe.
    Unsubscribe { events: Vec<String> },
    /// Ping for keepalive (per IMPL-PLAN §4.13.4 心跳 30s).
    Ping,
}

/// BFF → UI WebSocket 消息.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// Subscribe ack.
    Subscribed { events: Vec<String> },
    /// SSE event fanout (same payload as SSE).
    Event { payload: WorktreeSseEvent },
    /// Pong (response to Ping).
    Pong,
    /// Error.
    Error { code: String, message: String },
}

// =====================================================================
// API Error (per 守门 #6 v2: 6-field)
// =====================================================================

/// 6-field Worktree API Error (per 守门 #6 v2 + 跟 V0.1 collab `ApiError` 同形独立).
///
/// Fields: code / message / source / location / context / trace_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeApiError {
    pub code: String,
    pub message: String,
    pub source: String,
    pub location: String,
    #[serde(default)]
    pub context: serde_json::Value,
    pub trace_id: Uuid,
}

impl std::fmt::Display for WorktreeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} (source={}, location={}, trace_id={})",
            self.code, self.message, self.source, self.location, self.trace_id
        )
    }
}

impl std::error::Error for WorktreeApiError {}

impl WorktreeApiError {
    /// Construct a new error.
    pub fn new(code: impl Into<String>, message: impl Into<String>, source: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source: source.into(),
            location: location.into(),
            context: serde_json::Value::Null,
            trace_id: Uuid::new_v4(),
        }
    }

    /// 4 常用快捷构造函数.
    pub fn bad_request(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message, "bff.worktree_canvas", location)
    }

    pub fn not_found(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message, "bff.worktree_canvas", location)
    }

    pub fn forbidden(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::new("FORBIDDEN", message, "bff.worktree_canvas", location)
    }

    pub fn conflict(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::new("CONFLICT_REQUIRED", message, "bff.worktree_canvas", location)
    }

    /// 6-field 完整性守门 (per 守门 #6 v2).
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.code.is_empty() {
            return Err("code is empty");
        }
        if self.message.is_empty() {
            return Err("message is empty");
        }
        if self.source.is_empty() {
            return Err("source is empty");
        }
        if self.location.is_empty() {
            return Err("location is empty");
        }
        // trace_id 不查 nil — 一定非 nil 由 new() 保证.
        if self.trace_id.is_nil() {
            return Err("trace_id is nil");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_event_names_count_is_fifteen() {
        // 15 SSE event 变体 per spec §4.5.
        assert_eq!(WorktreeSseEvent::ALL_NAMES.len(), 15);
        assert_eq!(
            WorktreeSseEvent::WorktreeCreated {
                worktree_id: Uuid::new_v4(),
                repository_id: Uuid::new_v4(),
                branch: "main".into(),
            }
            .event_name(),
            "WorktreeCreated"
        );
    }

    #[test]
    fn worktree_api_error_six_fields_present() {
        let e = WorktreeApiError::bad_request("test", "loc");
        assert_eq!(e.code, "BAD_REQUEST");
        assert!(!e.message.is_empty());
        assert_eq!(e.source, "bff.worktree_canvas");
        assert!(!e.location.is_empty());
        assert!(!e.trace_id.is_nil());
        assert!(e.validate().is_ok());
    }

    #[test]
    fn worktree_api_error_validate_rejects_empty_code() {
        let mut e = WorktreeApiError::bad_request("test", "loc");
        e.code = String::new();
        assert!(e.validate().is_err());
    }
}

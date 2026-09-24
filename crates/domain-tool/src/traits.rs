//! # Tool supporting types — PI-4 Tool trait 5 方法
//!
//! **PI-4 (per SRS-PI-BORROW-001 §4 FR-19~23 + BD-PI-BORROW-001 §2.4 + §3.3)**
//!
//! Defines the supporting DTOs that travel through the new 5-method
//! `Tool` trait:
//!
//! - [`ToolArgs`] — validated LLM-supplied arguments (per FR-21, **Transaction**)
//! - [`ToolOutput`] — execution result (per FR-23, **Transaction**)
//! - [`ToolExecutionMode`] — schedule hint (per FR-22, value type, embedded in tool metadata)
//!
//! W/T/M classification (per 守门 #13):
//!
//! | DTO | 分类 | 理由 |
//! |---|---|---|
//! | `ToolArgs` | **Transaction** | 执行参数, audit log |
//! | `ToolOutput` | **Transaction** | 执行结果, audit log |
//! | `ToolExecutionMode` | (value) | enum, 嵌入 tool registry metadata |
//!
//! **Sprint 2 vs Sprint 3 boundary**:
//! - Sprint 2 (this issue, ULYS-205): types land here as v0.0.2 stubs.
//! - Sprint 3 (PI-4 later): real Bash/Read/Write tools wire JSON Schema
//!   validation via `schemars` + `jsonschema` crates (not yet in workspace;
//!   per "no new dep without PI-4 child confirmation" gate).
//!
//! **Lead responsibility**: domain-tool Lead (per AGENTS.md §0 gate #3).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// =====================================================================
// ToolArgs — validated LLM-supplied arguments (FR-21, Transaction)
// =====================================================================

/// **ToolArgs** -- validated arguments for a single tool invocation
/// (per SRS-PI-BORROW-001 §4 FR-21 + BD §2.1 + §2.4)
///
/// Lifecycle:
/// 1. LLM emits raw JSON arguments.
/// 2. `Tool::prepare_arguments(&self, raw_json)` validates against `Tool::schema()`.
/// 3. On `Ok(args)`, agent loop dispatches `Tool::execute(args)`.
/// 4. On `Err(ToolRegistryError::InvalidArguments)`, agent loop retries
///    the tool call (per FR-21 NFR-6).
///
/// W/T/M: **Transaction** (per BD §2.2 + 守门 #13) — `audit_log` row,
/// `event_type='tool_call'`, `payload=args_json`.
///
/// **Sprint 2 stub**: wraps `serde_json::Value` plus the LLM-supplied
/// tool-call id (so a single LLM response with N tool calls can be
/// correlated). Real schema validation lands in Sprint 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolArgs {
    /// Tool-call id assigned by the LLM/agent loop (used for replay correlation).
    pub tool_call_id: Uuid,
    /// Validated JSON arguments (validated against `Tool::schema()` by `prepare_arguments`).
    pub arguments: Value,
}

impl ToolArgs {
    /// Create a new `ToolArgs` with explicit tool-call id + arguments.
    ///
    /// Callers normally obtain this via `Tool::prepare_arguments` rather
    /// than constructing it directly — direct construction is provided
    /// for test fixtures and replay paths.
    pub fn new(tool_call_id: Uuid, arguments: Value) -> Self {
        Self {
            tool_call_id,
            arguments,
        }
    }
}

// =====================================================================
// ToolOutput — execution result (FR-23, Transaction)
// =====================================================================

/// **ToolOutput** -- result of a single tool execution
/// (per SRS-PI-BORROW-001 §4 FR-23 + BD §2.1 + §2.4)
///
/// W/T/M: **Transaction** — `audit_log` row, `event_type='tool_result'`,
/// `payload=output_json`.
///
/// **Replay**: `Tool::replay(execution_id)` returns this same shape
/// reconstructed from `audit_log` (per FR-23).
///
/// Sprint 2 stub carries the JSON-serialized result + execution id +
/// the tool name (for replay). Tool-specific rich variants (e.g.
/// `BashOutput { stdout, stderr, exit_code }`) will live in Sprint 3
/// tool impls as thin newtypes over `ToolOutput::Structured`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Execution id (one tool invocation = one execution id).
    pub execution_id: Uuid,
    /// Logical tool name (matches the registered tool).
    pub tool_name: String,
    /// JSON-encoded result body. Sprint 3 may switch to a `Structured(Value)` variant
    /// if the agent loop needs typed access; v0.0.2 keeps it simple.
    pub result: Value,
}

impl ToolOutput {
    /// Create a new `ToolOutput` (used by `Tool::execute` impls).
    pub fn new(execution_id: Uuid, tool_name: impl Into<String>, result: Value) -> Self {
        Self {
            execution_id,
            tool_name: tool_name.into(),
            result,
        }
    }
}

// =====================================================================
// ToolExecutionMode — schedule hint (FR-22, value type)
// =====================================================================

/// **ToolExecutionMode** -- scheduling hint returned by `Tool::execution_mode()`
/// (per SRS-PI-BORROW-001 §4 FR-22 + BD §2.1 + §3.3)
///
/// Agent loop dispatches tool calls according to this enum:
///
/// - `Synchronous` — block the agent loop until the tool returns.
///   Used for fast, deterministic tools (e.g. `read_file`, `grep`).
/// - `Async` — submit and return a handle; agent loop polls for completion.
///   Used for tools whose completion is expected but not blocking UX (e.g. `web_search`).
/// - `Stream` — produces a stream of partial results (e.g. `tail_log`).
///   Agent loop integrates chunks into context incrementally.
/// - `Background` — fire-and-forget; result arrives via event bus (e.g. `schedule_job`).
///   Agent loop continues without waiting.
///
/// **Sprint 2 stub**: enum is complete; agent loop dispatch logic
/// referenced by `execution_mode()` lands in Sprint 3 (per BD §8 BD-G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionMode {
    /// Block the agent loop until the tool returns.
    Synchronous,
    /// Submit and return a handle; agent loop polls for completion.
    Async,
    /// Stream partial results; agent loop integrates chunks incrementally.
    Stream,
    /// Fire-and-forget; result arrives via event bus.
    Background,
}

impl ToolExecutionMode {
    /// Stable string representation (for logging / persistence).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Synchronous => "synchronous",
            Self::Async => "async",
            Self::Stream => "stream",
            Self::Background => "background",
        }
    }
}

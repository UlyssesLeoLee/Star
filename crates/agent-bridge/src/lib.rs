//! `agent-bridge` — AI Worktree Graph Canvas Agent Bridge crate.
//!
//! Per ULYS-57.2 T6 (2026-09-16):
//! - `AgentRuntime` Trait + 8 方法 (per BD D-AGENT-001)
//! - Multica adapter + AgentSession 14 状态同步 (per DD §15)
//! - INV-WC-10: LLM 不修改 Source of Truth, 只消费数据
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 `AgentError` 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod annotate;
pub mod error;
pub mod runtime;
pub mod sandbox;
pub mod tool;

pub use annotate::{AnnotationAuthor, AnnotationRegistry, AnnotationRegistryError, DiffAnnotation};
pub use error::AgentError;
pub use runtime::{
    AgentEvent, AgentFilter, AgentMetrics, AgentRuntime, AgentSession, InMemoryAgentRuntime,
    TokenUsage,
};

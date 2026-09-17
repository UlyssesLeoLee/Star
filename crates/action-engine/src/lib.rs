//! `action-engine` — AI Worktree Graph Canvas Action Engine crate.
//!
//! Per ULYS-57.3 T8 (2026-09-16):
//! - `ActionEngine` + 18 Action + 3 类别 (Safe 6 + Warning 5 + Destructive 7,
//!   per DD-WORKTREE-CANVAS-001 §12.1 自审修正 `Merge = Destructive`)
//! - 二次确认机制 (Destructive 必走, per FR-ACTION-003)
//! - Idempotency 24h TTL (per INV-WC-11, DD §42)
//! - Audit SCD Type 2 不可篡改 (per INV-WC-12, DD §43)
//! - RBAC 5 角色 (per DD §44, FR-ACTION-004): Owner / SRE / Lead / PM / Viewer
//! - Retry 3 档 (per DD §39): Network / Graph Conn / LLM Timeout
//!
//! 守门:
//! - #1 v25 `cargo test -p action-engine --lib` 6/6 pass
//! - #6 v2 错误码 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]`
//! - #13 W/T/M 100% 覆盖 (action_audit T + idempotency W, 见 `db/migrations/`)
//!
//! 阶段 1 scope (per WORKTREE-CANVAS-IMPL-PLAN-001 §4.8):
//! - `ActionEngine::dispatch()` + 18 Action registry
//! - 危险分类 (Safe / Warning / Destructive) + 二次确认
//! - Idempotency Store 内存版 (24h TTL), PG/Redis 落点保留接口
//! - Audit Log SCD Type 2 (append-only)
//! - RBAC 5 角色权限矩阵
//! - Retry 3 档策略 (exponential backoff 1s/2s/4s)

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
// 守门 #6 v2: ActionError 6-field 设计 (code / message / source / location / context / trace_id)
#![allow(clippy::result_large_err)]

pub mod action;
pub mod audit_writer;
pub mod danger;
pub mod engine;
pub mod error;
pub mod idempotency;
pub mod rbac;
pub mod retry;
pub mod validator;

pub use action::{
    Action, ActionClass, ActionContext, ActionMetadata, ActionRequest, ActionResult, ActionType,
    ActionWarning,
};
pub use audit_writer::{AuditRecord, AuditWriter, AuditWriterError};
pub use danger::{classify, Classification};
pub use engine::{ActionEngine, ActionRegistry};
pub use error::ActionError;
pub use idempotency::{IdempotencyRecord, IdempotencyStore};
pub use rbac::{Role, User};
pub use retry::{retry_with_backoff, RetryDecision, RetryPolicy};

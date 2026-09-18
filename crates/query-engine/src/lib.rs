//! `query-engine` — AI Worktree Graph Canvas Query Engine crate.
//!
//! Per ULYS-57.3 T11 (2026-09-17):
//! - `QueryEngine` Trait + 3 方法 (execute_dsl / nl_to_dsl / dsl_to_cypher, per DD §17)
//! - Search DSL Parser (6 关键字 + AND/OR/NOT, per BD §0.2 D-SEARCH-001)
//! - NL Translator (LLM → DSL + Git Filter, per FR-SEARCH-003, MVP stub)
//! - Cypher injection guard (per NFR-SEC-002)
//!
//! 6 关键字 (per ULYS-57.3 acceptance + spec §8.7):
//! - `show` (HumanState: conflict / unmerged / ready / stale / running / waiting / merged)
//! - `agent` (agent type: codex / claude-code / opencode)
//! - `behind` (op + num)
//! - `ahead` (op + num)
//! - `health` (op + num)
//! - `modified` (file path)
//!
//! 注: DD §17.1 / spec §8.7 还提到 `task` 和 `inactive`; ULYS-57.3 acceptance 限定 6 关键字,
//! 即 `show / agent / behind / ahead / health / modified`.
//!
//! 守门:
//! - #1 v25 `cargo test -p query-engine --lib` 4/4 pass
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #10 Cypher 注入防护 (per NFR-SEC-002, 字符串净化)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]`

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
// 守门 #6 v2: QueryError 6-field 设计 (code / message / source / location / context / trace_id)
// 字段超过 100 字节是 intentional — TraceID + 完整 audit context
#![allow(clippy::result_large_err)]

pub mod cypher_guard;
pub mod dsl_parser;
pub mod engine;
pub mod error;
pub mod keywords;
pub mod nl_translator;

pub use cypher_guard::{is_safe_cypher_param, sanitize_cypher_param, CypherGuardError};
pub use dsl_parser::{DslParser, Filter, Operator, ShowFilterValue};
pub use engine::{CypherQuery, QueryEngine, QueryEnginePure, QueryResult};
pub use error::QueryError;
pub use keywords::{Keyword, KEYWORD_COUNT};
pub use nl_translator::{nl_to_dsl, nl_to_filters};

//! `graph-core` — AI Worktree Graph Canvas Graph Core crate.
//!
//! Per ULYS-57.1 T1 (2026-09-16):
//! - `GraphRepository` Trait + 11 方法 (per BD-WORKTREE-CANVAS-001 D-GRAPH-001)
//! - Neo4j 5.x adapter + Memory adapter (testcontainers / E2E)
//! - 11 Node 类型 enum + struct (per DD-WORKTREE-CANVAS-001 §7, INV-WC-05)
//! - 13 Edge 类型 enum + struct (per DD-WORKTREE-CANVAS-001 §8, INV-WC-04)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//! - #13 W/T/M 100% 覆盖 (5 表 DDL 落档 `db/migrations/`, per 守门 #13)
//! - #14 v4 5 域 Lead review (Mavis 临时代签 per 9/3 11:35 JST 反转)
//!
//! 阶段 1 scope (per WORKTREE-CANVAS-IMPL-PLAN-001.md §4.1):
//! - 11 Node 类型 enum/struct 完整覆盖 SRS §7 核心 11 类
//! - 13 Edge 类型 enum/struct 完整覆盖 SRS §8 核心 13 类
//! - `GraphRepository` trait 11 方法: 5 Node CRUD + 4 Edge CRUD + 2 traversal
//! - 6-field `GraphError` (per 守门 #6 v2)
//! - Memory adapter 完整实现 (test/E2E 用, Neo4j adapter 接口对齐)
//! - Neo4j adapter 暴露 Cypher 字符串 + schema DDL, 不连接真实库 (阶段 2 实装)

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
// workspace.lints
// 阶段 1 简化: GraphError 6-field 设计 (per 守门 #6 v2 + DD §38)
// 字段超过 100 字节是 intentional — TraceID + 完整 audit context
#![allow(clippy::result_large_err)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod edge;
pub mod error;
pub mod memory_repo;
pub mod neo4j_repo;
pub mod node;
pub mod repository;
pub mod state;
pub mod types;

pub use edge::{EdgeKind, EdgePayload};
pub use error::GraphError;
pub use memory_repo::InMemoryGraphRepository;
pub use neo4j_repo::{Neo4jConfig, Neo4jGraphRepository, NEO4J_CONSTRAINTS, NEO4J_INDEXES};
pub use node::{NodeKind, NodePayload};
pub use repository::{
    EdgeFilter, GraphQueryResult, GraphRepository, NodeFilter, TraversalDirection,
};
pub use state::{
    AgentResultState, AgentStatus, HumanState, IssueState, MachineState, MergeStrategy, PRState,
    SymbolKind, TaskStatus, TestState, TestStatus, TokenUsage,
};
pub use types::{
    AgentId, BranchId, IssueId, PRId, RepoId, RiskScore, TaskId, TestId, UserId, WorktreeId,
};

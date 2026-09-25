//! `ai-vault` — AI Vault domain crate (per ULYS-157 + FR-ORCA-023..024 + §5 v1.0)
//!
//! **目的**: 桌面 AI Vault 对话历史搜索的 local-only settings + clear index.
//!
//! **MVP v0 范围 (本 issue ULYS-157 P0-C MVP)**:
//! - FR-ORCA-023 (per spec line 347-349): `SearchPolicy{enabled, history_days}` local-only 设置
//! - FR-ORCA-024 (per spec line 351-353): `clear_search_index()` 删除 SQLite + sidecars, 保留 raw
//!
//! **不在本 MVP 范围 (P1 followup, per description §5 工时估 5-6 周)**:
//! - FR-ORCA-019 (跨 Agent Session 搜索) — 需 tantivy / meilisearch 选型
//! - FR-ORCA-020 (Cursor / Generation 双向 fence)
//! - FR-ORCA-021 (Redact by Transport)
//! - FR-ORCA-022 (Status Endpoint sentinel-aware)
//! - 实际 indexer 生命周期 (per description "实装期走原型对比")
//! - Settings UI 渲染 (frontend M2)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod clear;
pub mod policy;

pub use clear::{
    clear_search_index, ClearIndexError, ClearSearchIndexResult, VaultSearchIndexLayout,
};
pub use policy::{PolicyError, SearchPolicy};

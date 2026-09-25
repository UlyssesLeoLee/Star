//! `worktree-service` — AI Worktree Graph Canvas Worktree Service crate.
//!
//! Per ULYS-57.2 T3 (2026-09-16):
//! - `WorktreeService` Trait + 9 方法 (per DD-WORKTREE-CANVAS-001 §12)
//! - 7 State 状态机迁移 (per DD §23, INV-WC-01)
//! - CRUD + create/read/update/delete (per INV-WC-09, NFR-SEC-001)
//! - Health + Risk 集成 (per INV-WC-02/03)
//! - Cleanup 批量 + Archive (保留 Provenance, per INV-WC-08)
//! - `WorktreeStatusObserved` Projection 写 30 天热数据 (per DD §12.5)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 `ServiceError` 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//! - #13 a/d W/T/M 100% 覆盖 (worktree_status_observed W + worktree_conflict T)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod error;
pub mod external_worktree_import;
pub mod lifecycle;
pub mod projection;
pub mod service;
pub mod service_impl;
pub mod start_from_picker;

pub use error::ServiceError;
pub use external_worktree_import::{
    diff_external, map_to_worktree, parse_porcelain, scan_external_worktrees, ExternalWorktree,
    ImportError, ImportOutcome,
};
pub use lifecycle::{transition, TransitionError};
pub use projection::{StatusObservedPoint, WorktreeStatusObserved};
pub use service::{SyncResult, Worktree, WorktreeFilter, WorktreeService, WorktreeUpdate};
pub use service_impl::InMemoryWorktreeService;
// ULYS-194 Start-from Picker (FR-ORCA-009): re-export start_from_picker 模块符号
// 让 integration test (`start_from_picker_integration.rs`) + 外部 caller 直接 use.
// 注: NoopPickerSource 在 service_impl.rs (per ULYS-194 跟 PickerSource trait impl 一起).
pub use service_impl::NoopPickerSource;
pub use start_from_picker::{
    pick_start_from_candidates, select_by_id, worktree_to_existing_candidate,
    DefaultStartFromPickerSource, PickerCandidate, PickerCandidateKind, PickerCandidates,
    StartFromPickerSource,
};
// 注: per-workspace multica-config/config.json 的 shared-dir reader 已在
// crates/worktree-shared-dir/ (FileBackedConfigSource) 实装, 不在本 crate.
// ULYS-177 历史 commit ef025c8d 的 `shared_dir_sources.rs` 模块已被替代.

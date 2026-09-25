//! `lib.rs` — `worktree-shared-dir` crate root
//!
//! Per ULYS-104.3 (ULYS-158 + ULYS-177) P0-D, 9/22 D-Boy 启动发令 + 11:36 JST 路径 C 自决:
//! - `SharedDirResolver` Trait (per FR-ORCA-007 3 机制聚合)
//! - 3 Source: `PerUserSource` (file-backed, 实装 in-crate, ULYS-158.2 P0)
//!             + `WorkspaceSource` (PG, P1 followup)
//!             + `ConfigSource` (file-backed, 实装 in-crate)
//! - 5 `SharedMountStrategy` + 3 `SharedDirPriority` + 4 `PickerCandidateKind` (FR-ORCA-009 P2)
//! - `SharedDirectory` / `PickerCandidate` / `SharedDirConfig`
//! - 6-field `SharedDirError` (per 守门 #6 v2)
//!
//! ## 关键决策 (per D-Boy 11:36 JST 2026-09-22 comment `01a0c8e7-5df4-7b4c-af8d-528351eb4a22`)
//!
//! - **不依赖 Multica CLI**: ConfigSource 直接读 `<workspace>/multica-config/config.json`,
//!   不走 `multica config set/get`. 即使 CLI v0.4.42 的 16-key whitelist 不含
//!   `worktree_shared_directories`, 本 crate 仍可工作 (运维手写 config.json 即可).
//! - **PerUserSource 不依赖 CLI**: 直读 `~/.star/worktree_shared_dirs.txt`,
//!   纯文本 (一行一路径 + `#` 注释), 同样不依赖 CLI (per ULYS-158.2 §2).
//! - **复用 Rust 生态**: 用 `serde_json` (workspace dep) 解析 config.json, 不引新依赖.
//! - **零失败 fallback**: 任何 IO / parse 错误降级为 empty, 不阻断 worktree 创建 (per FR-ORCA-007).
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 `SharedDirError` 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]`
//! - #13 a/d 配合 PG 迁移 (M 100% audit) — PG impl 留 P1 followup

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod branch_naming;
pub mod error;
pub mod external_worktree_import;
pub mod shared_dir_resolver;
pub mod shared_dir_types;
pub mod start_from_picker;
pub mod worktree_create_async;

pub use branch_naming::{
    BranchNamer, BranchNamingInput, BranchNamingResult, BranchNamingSource, DefaultBranchNamer,
    EMOJI_SHORTCODE_TABLE, MAX_BRANCH_LENGTH, normalize_branch, replace_emoji_shortcodes,
    sanitize_branch_chars, truncate_branch,
};
pub use error::{SharedDirError, SharedDirResult};
pub use external_worktree_import::{
    DynExternalWorktreeImport, ExternalWorktree, ExternalWorktreeImport,
    ExternalWorktreeImportError, ExternalWorktreeImportRegistry,
    ExternalWorktreeImportResult, InMemoryExternalWorktreeImport, NoopPostImportHook,
    PostImportHook, RealExternalWorktreeImport, WorktreeId,
};
pub use shared_dir_resolver::{
    default_per_user_config_path, ConfigSource, FileBackedConfigSource, FileBackedPerUserSource,
    InMemorySharedDirConfigSource, InMemorySharedDirPerUserSource, InMemorySharedDirResolver,
    InMemorySharedDirWorkspaceSource, NoopConfigSource, NoopPerUserSource, NoopWorkspaceSource,
    PerUserSource, ResolvedSharedDirs, SharedDirResolver, WorkspaceSource, CLI_CONFIG_DIR_NAME,
    CLI_CONFIG_FIELD, CLI_CONFIG_FILE_NAME, PERUSER_CONFIG_DIR_NAME, PERUSER_CONFIG_FILE_NAME,
    SOURCE_CLI_CONFIG,
};
pub use shared_dir_types::{
    PickerCandidate, PickerCandidateKind, SharedDirConfig, SharedDirPriority, SharedDirSource,
    SharedDirectory, SharedMountStrategy,
};
pub use start_from_picker::{
    InMemoryStartFromPicker, RealStartFromPicker, RemoteFetch, StartFrom,
    StartFromPickerError, StartFromPickerRegistry, describe_start_from, parse_candidate_id,
};
pub use worktree_create_async::{
    InMemoryWorktreeCreateAsync, RealWorktreeCreateAsync, WorktreeCreateAsync,
    WorktreeCreateHandle, WorktreeCreateRequest, WorktreeEvent,
};

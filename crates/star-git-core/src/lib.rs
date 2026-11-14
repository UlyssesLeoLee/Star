//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/GitGit (AGPL-3.0) with the author's
//! dual-license permission (MIT OR Apache-2.0) for Star,
//! per 2026-11-14 Ulysses 拍板 ("gitgit 我电脑里就有，直接用").
//! Source: D:\GitGit @ 69770e6
//! ═══════════════════════════════════════════════════════════════════

//! star-git-core — Git 冲突区域内核 (Star)
//!
//! 从 UlyssesLeoLee/GitGit 移植 (作者双授权 MIT OR Apache-2.0):
//! - repo::conflict: 冲突 REGION 引擎 (ADR-0024 hunk 仲裁)
//! - repo::status: 路径校验
//! - subprocess: git CLI 同步包装
#![allow(missing_docs, unreachable_pub)]

pub mod error;
pub mod repo;
pub mod subprocess;

pub use error::{Result, StarGitError};
pub use repo::conflict::{
    ConflictSet, Disposition, LineRange, ParsedMerge, ParsedRegion, Region, RegionChoice,
    RegionConfidence, Scope, ScopeConfidence, ScopeKind, ScopeMap, ScopeRef, Segment,
};

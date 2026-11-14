//! ide-collab-core — 多 Agent 协同编辑内核 (Star)
//!
//! 从 UlyssesLeoLee/IDE1.0 `ide-agent-router` 移植的纯内核模块:
//! - identity: 语义单元模型与稳定身份指纹 (DD-ROUTER-SEM)
//! - parser:   多语言语义单元抽取 (Rust/Python/TS/JS/C/PowerShell)
//! - patch:    AST-aware Patch 重定位引擎
//! - coordinator: 文件级一致性协调器 + Fencing 租约 (DD-ROUTER-DB)
//! - collaboration: 多人/多 Agent 实时协同编辑引擎 (光标/选区/归属矩阵)
//!
//! 许可: 源为 AGPL-3.0, 作者 Ulysses 双授权 MIT OR Apache-2.0 用于 Star
//! (per 2026-11-14 拍板)。移植范围仅纯内核 (无 editcore/ide-kernel 依赖)。
#![allow(missing_docs, unreachable_pub, clippy::too_many_arguments)]

pub mod collaboration;
pub mod coordinator;
pub mod identity;
pub mod parser;
pub mod patch;

pub use collaboration::{
    AnchorOffsetTable, CollaborationHub, CollaborationSession, CollaboratorColor, CollaboratorRole,
    KeystrokeBroadcast, KeystrokeOp, RemoteCursor, ZoneAnchor,
};
pub use coordinator::{
    CoordinatorError, FileCommitCoordinator, FileState, OutboxEvent, PatchCommitResponse,
    TaskState, TaskStatus,
};
pub use identity::{SemanticUnit, SupportedLanguage};
pub use parser::SemanticParser;
pub use patch::{PatchEngine, PatchError, PatchProposal, PatchResult};

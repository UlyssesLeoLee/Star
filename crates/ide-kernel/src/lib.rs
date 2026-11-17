/*
@cypher schema=1 source_sha256=b0f9c1b55774cd918c76681be97df6ccbdb31d606bf1dc42b426f2068bb8e157
MERGE (self:File {path:"crates/ide-kernel/src/lib.rs"})
MERGE (buffer:Symbol {id:"crates/ide-kernel/src/lib.rs::buffer",kind:"module"})
MERGE (cursor:Symbol {id:"crates/ide-kernel/src/lib.rs::cursor",kind:"module"})
MERGE (undo:Symbol {id:"crates/ide-kernel/src/lib.rs::undo",kind:"module"})
MERGE (document:Symbol {id:"crates/ide-kernel/src/lib.rs::document",kind:"module"})
MERGE (session:Symbol {id:"crates/ide-kernel/src/lib.rs::session",kind:"module"})
MERGE (text_ops:Symbol {id:"crates/ide-kernel/src/lib.rs::text_ops",kind:"module"})
MERGE (doc_store:Symbol {id:"crates/ide-kernel/src/lib.rs::doc_store",kind:"module"})
MERGE (type_config:Symbol {id:"crates/ide-kernel/src/lib.rs::type_config",kind:"module"})
MERGE (kernel_version:Symbol {id:"crates/ide-kernel/src/lib.rs::KERNEL_VERSION",kind:"const"})
MERGE (self)-[:DEFINES]->(buffer)
MERGE (self)-[:DEFINES]->(cursor)
MERGE (self)-[:DEFINES]->(undo)
MERGE (self)-[:DEFINES]->(document)
MERGE (self)-[:DEFINES]->(session)
MERGE (self)-[:DEFINES]->(text_ops)
MERGE (self)-[:DEFINES]->(doc_store)
MERGE (self)-[:DEFINES]->(type_config)
MERGE (self)-[:DEFINES]->(kernel_version)
@endcypher
*/
// =====================================================================
// 移植自 IDE1.0 (E:\IDE1.0, HEAD a200e5d)
//   源文件:
//     crates/ide-kernel/src/{lib,document,session,text_ops,doc_store}.rs
//     crates/editcore/src/{buffer,cursor,undo}.rs
//     crates/editcore/src/{type_config,types,outline_langs}.rs
//   许可: 权利人授权按 Star MIT 复用
//     (docs/architecture/2026-11-14-ops-console/REUSE-MAP-COLLAB-SANDBOX-001.md §4 v0.2)
//   差异: 仅将 `editcore::` 前缀改写为 `crate::` (buffer/cursor/undo/type_config/
//     types/outline_langs 随本 crate 一并移植), 其余保持原样以利 diff 对齐。
// Star workspace lints 要求 missing_docs/unreachable_pub deny — 移植代码保持
// 原样以利 diff 对齐, 本 crate 内豁免这两项 (其余 lint 照常)。
// =====================================================================

//! `ide-kernel` — IDE1.0 唯一编辑内核的 Star 移植.
//!
//! # 为什么存在
//!
//! IDE1.0 曾有两个互不相通的编辑引擎 (前端 JS 引擎 ≈4400 行 / `editcore` Rust 库),
//! 修复方式是把 `editcore` 已验证的部件聚合成**会话级编辑内核**: 内容 + 光标 +
//! undo/redo + 选区 + 多光标 + 查找替换 + 编码 + 行尾符, 通过 `Document` 单一入口
//! 对外暴露。三个外壳 (Web HTTP / Tauri command / TUI) 只做渲染。
//!
//! 本移植抽取内核及其最小依赖 (buffer/cursor/undo + 语义 outline 三模块),
//! 供 Star 后端 (star-api-rest `/api/v1/ide/*`) 复用同一份编辑语义。
//!
//! # 分层
//!
//! ```text
//!   外壳 (web / desktop / tui)   ← 只渲染 + 转发按键
//!        ↓
//!   ide_kernel::DocStore         ← 多文档会话的传输无关实现
//!        ↓
//!   ide_kernel::Session          ← 会话: 多标签 + 窗口状态
//!        ↓
//!   ide_kernel::Document         ← 单文档: 本 crate 的核心
//!        ↓
//!   ide_kernel::{buffer, cursor, undo}  ← 自 editcore 移植的基础部件
//! ```
//!
//! # 设计约束
//!
//! - **不引入新依赖**。只用 serde / serde_json。
//! - **零 unsafe** (`#![forbid(unsafe_code)]`, 与上游 editcore 一致)。
//! - **所有编辑走 undo**: 每个修改点必须经 `Document::apply`, 否则 undo 栈失真。
//! - **char 语义**: 行列按 char 计数, 不按 byte (否则多字节字符编辑会错位)。

#![forbid(unsafe_code)]
// 移植代码保持与上游 diff 对齐 (per nema-codec 同款豁免, 见文件头注释)。
#![allow(missing_docs, unreachable_pub)]

pub mod buffer;
pub mod cursor;
pub mod doc_store;
pub mod document;
pub mod outline_langs;
pub mod session;
pub mod text_ops;
pub mod type_config;
pub mod types;
pub mod undo;

pub use doc_store::{DocStore, DocView, FindHit, ReplaceStat};
pub use document::{Document, DocumentSnapshot, Encoding, LineEnding, Pos, Selection};
pub use session::{Session, TabState};
pub use text_ops::{FindOptions, FindResult, ReplaceOutcome};
pub use type_config::{OutlineEntry, TypeConfig, TypeRegistry};

/// 内核版本 —— 前端/外壳用于确认协议兼容.
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

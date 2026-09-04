//! `star-vcs` crate (R-007 落点 + Phase D 填实)
//!
//! 公开 API:
//! - [`cache::VcsCache`]     — R-007 cache 抽象占位 (Phase D 填实 `Cache<K, V>` trait + `InMemoryCache` impl)
//! - [`cache::CacheError`]   — cache 错误类型占位
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 0 新依赖 (当前占位, Phase D 填实时再 `use serde/tokio/thiserror` 等 workspace 已有依赖)
//! - 占位 + 显式 TODO 注释 (per R-012 缺标比错标安全守门)
//! - workspace.members 已注册 (per T1.3, RF-001 WBS §1)
//!
//! ## 触发
//!
//! - 8/27 commit `48610ff2` R-007 cache 层落点 (Mavis 接手 DEC-008, 仅占位 + TODO)
//! - 9/3 RF-001 T1.3 拍板 A 注册 (per `docs/plans/PHASE-EXEC-PLAN-2026-09-03.md` §6.2 #1)
//! - 9/3 本 commit 新建 `Cargo.toml` + 注册 workspace.members + 本 `lib.rs` 包 mod cache

/// R-007 cache 层占位 (Phase D 填实)
/// 详见 [`cache::VcsCache`] (当前空 struct, Phase D 替换为 `Cache<K, V>` trait + `InMemoryCache` impl)
pub mod cache;

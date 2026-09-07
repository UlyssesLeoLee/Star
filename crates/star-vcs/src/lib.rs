//! `star-vcs` crate (R-007 落点 + Phase D 完整实装)
//!
//! 公开 API:
//! - [`cache::VcsCache`]          — R-007 cache 抽象 (async trait: get/put/invalidate/clear)
//! - [`cache::InMemoryVcsCache`]  — MVP 进程内后端 (tokio RwLock<HashMap> + TTL + CacheMetrics)
//! - [`cache::ProviderCache`]     — Read-through wrapper 集成 [`cache::VcsProvider`] (30s 默认 TTL)
//! - [`cache::VcsProvider`]       — VCS Provider 最小 trait (Phase F star-sa 引用点)
//! - [`cache::CacheError`]        — cache 错误类型 (thiserror enum, 4 变体)
//! - [`cache::CacheMetrics`]      — hit/miss/put/evict 计数 + 命中率
//!
//! ## 守门规则
//!
//! - 0 unsafe (`unsafe_code = "forbid"`)
//! - 仅 workspace 已有依赖 (tokio / async-trait / thiserror)
//! - 占位 → 实装 过渡: 0 unknown TODO, 4 类 test 覆盖 (basic/ttl/eviction/provider)
//! - workspace.members 已注册 (per T1.3, RF-001 WBS §1)
//!
//! ## 触发
//!
//! - 8/27 commit `48610ff2` R-007 cache 层落点 (Mavis 接手 DEC-008, 仅占位 + TODO)
//! - 9/3 RF-001 T1.3 拍板 A 注册 (per `docs/plans/PHASE-EXEC-PLAN-2026-09-03.md` §6.2 #1)
//! - 9/3 本 commit 新建 `Cargo.toml` + 注册 workspace.members + 本 `lib.rs` 包 mod cache
//! - 9/7 brief OPT-WORKER-02-vcs-cache 完整实装 cache.rs (per OPT-CODE-68 + STAR-P4-OPT-WBS §3.2 #3)

/// R-007 cache 层实装 (Phase D 完整)
pub mod cache;

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/arg-effect` — STAR Agent Relationship Graph (ARG) Effect Tier
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) v0.1.1
//! + [`docs/architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md`](../../docs/architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md) v0.50
//!
//! This crate is the **Effect Tier** of the ARG subsystem. It exposes
//! 5 submodules that consume the Data Tier ([`star_arg`]) and the
//! Bridge Tier ([`star_arg_bridge`]):
//!
//! - [`dispatch_router`] — `ARGDispatchRouter` (per DD §4.5). 3 effect 维度 #1.
//! - [`context_injector`] — `ARGContextInjector` (per DD §4.6). 3 effect 维度 #2.
//! - [`trust_engine`] — `ARGTrustEngine` (per DD §4.7). 3 effect 维度 #3.
//! - [`output_evaluator`] — `ARGOutputEvaluator` (per DD §4.8). 3 effect 维度 #4 + 10 challenges prompt.
//! - [`achievement_engine`] — `ARGAchievementEngine` (per DD §4.9). 8 拓扑成就 + 7 行为 + 5 产出.
//! - [`prompts`] — 10 套 challenges 双向论证 prompt 模板 (per DD §7).
//! - [`error`] — `EffectError` (8 variants per DD §9.1).
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).
//! - Env-bound secrets are read via [`star_arg::client::MemgraphClient::new_from_env`]
//!   and never printed (守门 #5 env-safety, 2026-08-27 11:06 JST hard-ban).
//! - All public types derive `Debug` + `Clone` where possible to ease
//!   the `Arc`-sharable design (per arch §3.1).
//!
//! 缺标比错标安全: the real L0↔L1 PyO3 binding (per DD §5.3) and the
//! production LLMClient adapter (per 守门 #5 v2 + #23) are still on
//! the P3-C W3 backlog; the current surface accepts the G-1 stubs
//! from `star_arg` and the local `MockLLMClient`.

#![forbid(unsafe_code)]

/// `ARGAchievementEngine` — 8 拓扑 + 7 行为 + 5 产出 (per DD §4.9 + arch §3.1).
pub mod achievement_engine;
/// `ARGContextInjector` — 3 effect 维度 #2 (per DD §4.6 + arch §3.1).
pub mod context_injector;
/// `ARGDispatchRouter` — 3 effect 维度 #1 (per DD §4.5 + arch §3.1).
pub mod dispatch_router;
/// Error types used across the ARG Effect Tier (per DD §9.1 — 8 variants).
pub mod error;
/// `ARGOutputEvaluator` — 3 effect 维度 #4 (per DD §4.8 + arch §3.1).
pub mod output_evaluator;
/// 10 套 challenges 双向论证 prompt 模板 (per DD §7 + arch §3.1).
pub mod prompts;
/// `ARGTrustEngine` — 3 effect 维度 #3 (per DD §4.7 + arch §3.1).
pub mod trust_engine;

pub use achievement_engine::{
    ARGAchievementEngine, BehaviorEvaluator, OutputEvaluator, StubTopologyBackend, TopologyBackend,
    TopologyEvaluator,
};
pub use context_injector::{
    ARGContextInjector, ContextProvider, DecisionRow, EventRow, InMemoryContextProvider,
    DEFAULT_INJECTED_CAP_BYTES,
};
pub use dispatch_router::{ARGDispatchRouter, DispatchRoute, InMemoryEdgeStore};
pub use error::EffectError;
pub use output_evaluator::{ARGOutputEvaluator, UtcDateTime, PEER_REVIEW_ACCEPT_THRESHOLD};
pub use prompts::{challenge_prompts, TrustTier};
pub use trust_engine::{
    ARGTrustEngine, TrustStore, SKIP_VERIFY_EDGE_WEIGHT_THRESHOLD, SKIP_VERIFY_SCORE_THRESHOLD,
};

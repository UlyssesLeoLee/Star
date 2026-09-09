// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/arg-bridge` — STAR Agent Relationship Graph (ARG) Bridge Tier
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) v0.1.1
//! + [`docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md`](../../docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) v0.50
//!
//! This crate is the **Bridge Tier** of the ARG subsystem. It bridges the
//! ARG Data Tier ([`star_arg`]) and the in-process LangGraph `TopAgentState`
//! via four submodules:
//!
//! - [`memgraph_listener`] — `MemgraphEventListener` (per DD §4.10). Holds
//!   a `tokio::sync::broadcast::Sender<ARGEvent>` and re-emits Memgraph
//!   edge.changed events into in-process subscribers.
//! - [`langgraph_updater`] — `LangGraphStateUpdater` (per DD §4.11 + §5.3).
//!   Receives `ARGEvent` and dispatches into the 5 Reducer functions
//!   (`merge_arg_agents` / `merge_arg_edges` / `merge_trust_scores` /
//!   `merge_dispatch` / `add`) on the LangGraph `TopAgentState`.
//! - [`period_flush`] — `PeriodFlushWorker` (per BD §7.2). 30 s default
//!   `tokio` task that batches in-process state changes and writes them
//!   back to Memgraph with versioned idempotency.
//! - [`offline_queue`] — `OfflineQueue` (per BD §7.2 + 守门 #12 已知缺口).
//!   `sled`-backed embedded queue, capacity 10 000 entries, 7-day TTL,
//!   consumed on reconnect.
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).
//! - Env-bound secrets (`MEMGRAPH_BOLT_URL` etc.) are read via
//!   [`star_arg::client::MemgraphClient::new_from_env`] and never printed
//!   (守门 #5 env-safety, 2026-08-27 11:06 JST hard-ban).
//!
//! 缺标比错标安全: real Bolt subscriptions / PyO3 bindings / sled pub-sub
//! across processes are not yet wired up. The current implementation
//! accepts the P3-C W1 stub returned by `MemgraphClient` (G-1) and uses
//! a placeholder `update_state` sink that records dispatch events in
//! memory. The real LangGraph integration lands in ARG.3.

#![forbid(unsafe_code)]

/// Error types used across the ARG Bridge Tier (per DD §9.1 — 6 variants).
pub mod error;
/// `LangGraphStateUpdater` — `ARGEvent` → 5 Reducer (per DD §4.11 + §5.3).
pub mod langgraph_updater;
/// `MemgraphEventListener` — Bolt subscription → in-process broadcast (per DD §4.10).
pub mod memgraph_listener;
/// `OfflineQueue` — `sled`-backed offline persistence (per BD §7.2 + 守门 #12).
pub mod offline_queue;
/// `PeriodFlushWorker` — 30 s 周期 flush (per BD §7.2).
pub mod period_flush;
/// 5 内部协议 schema (per arch 2026-09-03-arg/06-arg-02-arg-bridge.md §2).
pub mod protocol;

pub use error::BridgeError;
pub use langgraph_updater::{LangGraphStateUpdater, ReducerKind};
pub use memgraph_listener::MemgraphEventListener;
pub use offline_queue::OfflineQueue;
pub use period_flush::PeriodFlushWorker;
pub use protocol::{BridgeEnvelope, BridgeEnvelopeKind};

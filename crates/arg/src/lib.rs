// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/arg` — STAR Agent Relationship Graph (ARG) Data Tier
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) v0.1.1
//!
//! This crate is the **Data Tier** of the ARG subsystem. It exposes:
//!
//! - [`models`] — domain types (Agent, Edge, TeamTemplate, Achievement, …)
//! - [`client`] — Memgraph Bolt client + LRU cypher cache + schema migration
//! - [`ops`] — CRUD operations for agents, edges, templates, events and achievements
//! - [`query`] — Cypher templates for topology / behavior / output evaluations
//! - [`llm`] — `LLMClient` trait + a local mock implementation (守门 #5 v2 + #23)
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).
//! - All env-bound secrets (Memgraph URL / user / password) are read via
//!   [`client::memgraph::MemgraphClient::new_from_env`] and never printed
//!   (守门 #5 env-safety, 2026-08-27 11:06 JST hard-ban).
//!
//! 缺标比错标安全: many operations in [`ops`] currently return a
//! `MemgraphConnection` or `Other` error because the real `r2d2-memgraph`
//! integration is still on the P3-C W1 backlog (`G-1` in
//! `DD-AGENT-RELATIONSHIP-001.md` §13). Surface area is intentionally
//! narrow so the higher-tier crates (`arg-bridge`, `arg-effect`) can grow
//! against a stable API.

#![forbid(unsafe_code)]

/// Memgraph client + cypher cache + schema migration helpers.
pub mod client;
/// Error types used across the ARG Data Tier (per DD §9.1 — 10 variants).
pub mod error;
/// LLM abstraction trait + local mock implementation.
pub mod llm;
/// Domain models: agents, edges, templates, achievements, events, decisions.
pub mod models;
/// CRUD operation layer (agent nodes, edges, templates, events, achievements).
pub mod ops;
/// Cypher / event-pattern query templates.
pub mod query;

pub use error::ARGError;

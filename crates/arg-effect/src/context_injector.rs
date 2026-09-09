// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ARGContextInjector` (per DD-AGENT-RELATIONSHIP-001 §4.6).
//!
//! Injects mentor / shadow edge context into a base prompt. The
//! injector reads incoming edges via the in-memory store and produces
//! a single string that the LangGraph prompt builder appends to the
//! agent's base prompt.
//!
//! Per DD §4.6:
//! - MENTORS  edges contribute historical decision context
//! - SHADOWS  edges contribute recent event context
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::sync::Arc;

use star_arg::models::edge::{Edge, RelationshipType};
use uuid::Uuid;

use crate::dispatch_router::InMemoryEdgeStore;
use crate::error::EffectError;

/// Default per-call byte cap (per DD §11 NFR: 4 effect reload < 50ms).
pub const DEFAULT_INJECTED_CAP_BYTES: usize = 32 * 1024;

/// A single decision row used in the mentor context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRow {
    /// Decision description.
    pub description: String,
    /// When the decision was made.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DecisionRow {
    /// Build a new decision row.
    pub fn new(description: impl Into<String>, created_at: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            description: description.into(),
            created_at,
        }
    }
}

/// A single event row used in the shadow context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRow {
    /// Event description.
    pub description: String,
    /// When the event was emitted.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EventRow {
    /// Build a new event row.
    pub fn new(description: impl Into<String>, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            description: description.into(),
            timestamp,
        }
    }
}

/// Provider trait for cross-crate decision / event lookups (per DD §4.6
/// `query_recent_decisions` / `query_recent_events`).
///
/// The real implementation hits `decision_audit` and `relationship_events`
/// tables in the respective `domain-*` crates (per DD §4.6 + G-3).
/// During ARG.3 the trait is satisfied by an in-memory mock.
pub trait ContextProvider: Send + Sync {
    /// Return the N most recent decision rows for the given agent.
    fn query_recent_decisions(
        &self,
        agent_id: Uuid,
        limit: u32,
    ) -> Result<Vec<DecisionRow>, EffectError>;
    /// Return the N most recent event rows for the given agent.
    fn query_recent_events(&self, agent_id: Uuid, limit: u32)
        -> Result<Vec<EventRow>, EffectError>;
}

/// In-memory mock used by tests and by ARG.3 until G-3 lands.
#[derive(Debug, Default, Clone)]
pub struct InMemoryContextProvider {
    decisions: std::collections::HashMap<Uuid, Vec<DecisionRow>>,
    events: std::collections::HashMap<Uuid, Vec<EventRow>>,
}

impl InMemoryContextProvider {
    /// Build a new empty provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-seed decisions for an agent.
    pub fn seed_decisions(&mut self, agent_id: Uuid, rows: Vec<DecisionRow>) {
        self.decisions.insert(agent_id, rows);
    }

    /// Pre-seed events for an agent.
    pub fn seed_events(&mut self, agent_id: Uuid, rows: Vec<EventRow>) {
        self.events.insert(agent_id, rows);
    }
}

impl ContextProvider for InMemoryContextProvider {
    fn query_recent_decisions(
        &self,
        agent_id: Uuid,
        limit: u32,
    ) -> Result<Vec<DecisionRow>, EffectError> {
        let mut out: Vec<DecisionRow> = self.decisions.get(&agent_id).cloned().unwrap_or_default();
        out.sort_by_key(|d| std::cmp::Reverse(d.created_at));
        out.truncate(limit as usize);
        Ok(out)
    }

    fn query_recent_events(
        &self,
        agent_id: Uuid,
        limit: u32,
    ) -> Result<Vec<EventRow>, EffectError> {
        let mut out: Vec<EventRow> = self.events.get(&agent_id).cloned().unwrap_or_default();
        out.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        out.truncate(limit as usize);
        Ok(out)
    }
}

/// `ARGContextInjector` (per DD §4.6).
pub struct ARGContextInjector {
    /// Shared edge store.
    store: Arc<InMemoryEdgeStore>,
    /// Cross-crate context provider (mocked during ARG.3).
    provider: Arc<dyn ContextProvider>,
    /// Per-call byte cap (default 32 KiB).
    cap_bytes: usize,
}

impl std::fmt::Debug for ARGContextInjector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ARGContextInjector")
            .field("store", &self.store)
            .field("provider", &"<dyn ContextProvider>")
            .field("cap_bytes", &self.cap_bytes)
            .finish()
    }
}

impl Clone for ARGContextInjector {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            provider: self.provider.clone(),
            cap_bytes: self.cap_bytes,
        }
    }
}

impl ARGContextInjector {
    /// Build a new injector with the default byte cap.
    pub fn new(store: Arc<InMemoryEdgeStore>, provider: Arc<dyn ContextProvider>) -> Self {
        Self::with_cap(store, provider, DEFAULT_INJECTED_CAP_BYTES)
    }

    /// Build a new injector with a custom byte cap.
    pub fn with_cap(
        store: Arc<InMemoryEdgeStore>,
        provider: Arc<dyn ContextProvider>,
        cap_bytes: usize,
    ) -> Self {
        Self {
            store,
            provider,
            cap_bytes,
        }
    }

    /// Inject mentor / shadow context into the given base prompt.
    ///
    /// Layout:
    /// ```text
    /// {base_prompt}
    ///
    /// --- ARG Context ---
    /// === Mentor 历史决策参考 ===
    /// Mentor {uuid} (weight=...):
    ///   - {decision description}
    ///   - ...
    /// === Shadow 观察对象最近 Event ===
    /// Shadow {uuid} (weight=...):
    ///   - {event description}
    ///   - ...
    /// ```
    ///
    /// Returns `EffectError::ContextTooLarge` when the final string
    /// exceeds the configured cap.
    pub fn inject_context(
        &self,
        agent_id: Uuid,
        base_prompt: &str,
        tenant_id: Uuid,
    ) -> Result<String, EffectError> {
        let incoming = self.store.incoming(agent_id, tenant_id);
        let mentor_block = self.collect_mentor_context(&incoming)?;
        let shadow_block = self.collect_shadow_context(&incoming)?;

        let mut out =
            String::with_capacity(base_prompt.len() + mentor_block.len() + shadow_block.len() + 32);
        out.push_str(base_prompt);
        if !mentor_block.is_empty() || !shadow_block.is_empty() {
            out.push_str("\n\n--- ARG Context ---\n");
        }
        if !mentor_block.is_empty() {
            out.push_str(&mentor_block);
            if !shadow_block.is_empty() {
                out.push('\n');
            }
        }
        if !shadow_block.is_empty() {
            out.push_str(&shadow_block);
        }

        if out.len() > self.cap_bytes {
            return Err(EffectError::ContextTooLarge {
                size: out.len(),
                cap: self.cap_bytes,
            });
        }
        Ok(out)
    }

    /// Return just the mentor context block (per DD §4.6
    /// `collect_mentor_context`).
    pub fn collect_mentor_context(&self, edges: &[Edge]) -> Result<String, EffectError> {
        let mentors: Vec<&Edge> = edges
            .iter()
            .filter(|e| e.edge_type == RelationshipType::Mentors && !e.archived)
            .collect();
        if mentors.is_empty() {
            return Ok(String::new());
        }
        let mut ctx = String::from("=== Mentor 历史决策参考 ===\n");
        for m in mentors {
            let decisions = self.provider.query_recent_decisions(m.from_agent, 10)?;
            ctx.push_str(&format!(
                "Mentor {} (weight={:.3}):\n",
                m.from_agent, m.weight
            ));
            for d in decisions {
                ctx.push_str(&format!("  - {}\n", d.description));
            }
        }
        Ok(ctx)
    }

    /// Return just the shadow context block (per DD §4.6
    /// `collect_shadow_context`).
    pub fn collect_shadow_context(&self, edges: &[Edge]) -> Result<String, EffectError> {
        let shadows: Vec<&Edge> = edges
            .iter()
            .filter(|e| e.edge_type == RelationshipType::Shadows && !e.archived)
            .collect();
        if shadows.is_empty() {
            return Ok(String::new());
        }
        let mut ctx = String::from("=== Shadow 观察对象最近 Event ===\n");
        for s in shadows {
            let events = self.provider.query_recent_events(s.from_agent, 20)?;
            ctx.push_str(&format!(
                "Shadow {} (weight={:.3}):\n",
                s.from_agent, s.weight
            ));
            for e in events {
                ctx.push_str(&format!("  - {}\n", e.description));
            }
        }
        Ok(ctx)
    }

    /// Reference to the underlying edge store.
    pub fn store(&self) -> &Arc<InMemoryEdgeStore> {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use star_arg::models::edge::EdgeDirection;

    fn edge(from: Uuid, to: Uuid, t: RelationshipType, weight: f32, tenant: Uuid) -> Edge {
        let now = Utc::now();
        Edge {
            id: Uuid::new_v4(),
            from_agent: from,
            to_agent: to,
            edge_type: t,
            weight,
            direction: if t.is_directed() {
                EdgeDirection::Directed
            } else {
                EdgeDirection::Undirected
            },
            archived: false,
            metadata: serde_json::Value::Null,
            tenant_id: tenant,
            created_at: now,
            updated_at: now,
            version: 1,
            created_by: Uuid::new_v4(),
        }
    }

    #[test]
    fn inject_returns_base_prompt_when_no_incoming() {
        let store = Arc::new(InMemoryEdgeStore::default());
        let provider = Arc::new(InMemoryContextProvider::new());
        let injector = ARGContextInjector::new(store, provider);
        let out = injector
            .inject_context(Uuid::new_v4(), "base", Uuid::new_v4())
            .expect("ok");
        assert_eq!(out, "base");
    }

    #[test]
    fn inject_appends_mentor_block() {
        let tenant = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let mentor = Uuid::new_v4();

        let mut store = InMemoryEdgeStore::default();
        store.add(edge(mentor, agent, RelationshipType::Mentors, 0.7, tenant));

        let mut provider = InMemoryContextProvider::new();
        provider.seed_decisions(
            mentor,
            vec![DecisionRow::new("choose async runtime", Utc::now())],
        );

        let injector = ARGContextInjector::new(Arc::new(store), Arc::new(provider));
        let out = injector
            .inject_context(agent, "you are", tenant)
            .expect("ok");
        assert!(out.contains("Mentor "));
        assert!(out.contains("choose async runtime"));
        assert!(out.contains("--- ARG Context ---"));
    }

    #[test]
    fn inject_appends_shadow_block() {
        let tenant = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let target = Uuid::new_v4();

        let mut store = InMemoryEdgeStore::default();
        store.add(edge(target, agent, RelationshipType::Shadows, 0.6, tenant));

        let mut provider = InMemoryContextProvider::new();
        provider.seed_events(target, vec![EventRow::new("rebuild schema", Utc::now())]);

        let injector = ARGContextInjector::new(Arc::new(store), Arc::new(provider));
        let out = injector
            .inject_context(agent, "you are", tenant)
            .expect("ok");
        assert!(out.contains("Shadow "));
        assert!(out.contains("rebuild schema"));
    }

    #[test]
    fn inject_rejects_oversized() {
        let tenant = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let store = Arc::new(InMemoryEdgeStore::default());
        let provider = Arc::new(InMemoryContextProvider::new());
        // cap = 3 bytes — the literal "base" (4 bytes) exceeds the cap.
        let injector = ARGContextInjector::with_cap(store, provider, 3);
        let res = injector.inject_context(agent, "base", tenant);
        assert!(matches!(res, Err(EffectError::ContextTooLarge { .. })));
    }
}

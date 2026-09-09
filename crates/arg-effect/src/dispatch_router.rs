// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ARGDispatchRouter` (per DD-AGENT-RELATIONSHIP-001 §4.5 + arch §3.1).
//!
//! The router computes a [`DispatchRoute`] for a given agent by reading
//! the in-process edge store. The ARG.1 `EdgeOps` write paths already
//! accept the in-process state; the read paths are stubs (`G-1`) so
//! during the ARG.3 implementation the router accepts a pre-seeded
//! in-memory list of edges and a [`DecisionType`] matrix.
//!
//! The public surface mirrors the DD spec:
//! - [`ARGDispatchRouter::route`] returns a [`DispatchRoute`]
//!   with `delegates` / `stand_ins` / `collaborators`.
//! - [`ARGDispatchRouter::new`] takes shared dependencies.
//!
//! Cross-cutting safety nets (per `AGENTS.md` §4):
//! - No `unsafe` is allowed (`unsafe_code = "forbid"` at workspace level, 守门 #7).
//! - Every public item must have documentation (`missing_docs = "deny"`, 守门 #1 v1).

use std::collections::HashSet;
use std::sync::Arc;

use star_arg::models::edge::{Edge, RelationshipType};
use uuid::Uuid;

use crate::error::EffectError;

/// A single route computed by [`ARGDispatchRouter::route`].
///
/// 3 Vec fields per DD §4.5:
/// - `delegates`    — outgoing `DELEGATES_TO` targets
/// - `stand_ins`    — incoming `STAND_IN_FOR` sources (fallback candidates)
/// - `collaborators`— `COLLABORATES_WITH` peers (deduplicated)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchRoute {
    /// Outgoing `DELEGATES_TO` target agent ids.
    pub delegates: Vec<Uuid>,
    /// Incoming `STAND_IN_FOR` source agent ids (fallback candidates).
    pub stand_ins: Vec<Uuid>,
    /// `COLLABORATES_WITH` peer agent ids (deduplicated, undirected).
    pub collaborators: Vec<Uuid>,
}

impl DispatchRoute {
    /// Build a new empty route.
    pub fn empty() -> Self {
        Self {
            delegates: Vec::new(),
            stand_ins: Vec::new(),
            collaborators: Vec::new(),
        }
    }

    /// True iff all 3 vectors are empty (no route possible).
    pub fn is_empty(&self) -> bool {
        self.delegates.is_empty() && self.stand_ins.is_empty() && self.collaborators.is_empty()
    }

    /// Total number of agents in the route (sum across all 3 lists).
    pub fn total(&self) -> usize {
        self.delegates.len() + self.stand_ins.len() + self.collaborators.len()
    }
}

/// In-process store that mirrors the `EdgeOps` read path (per DD §4.3
/// and §4.5). The real Memgraph-backed implementation lands in P3-C
/// W1 when G-1 is closed; the stub here accepts a list of edges on
/// construction so the Effect Tier can be exercised end-to-end without
/// Memgraph.
#[derive(Debug, Default, Clone)]
pub struct InMemoryEdgeStore {
    /// All edges (per tenant).
    edges: Vec<Edge>,
}

impl InMemoryEdgeStore {
    /// Build a new in-memory store with the given edges.
    pub fn new(edges: Vec<Edge>) -> Self {
        Self { edges }
    }

    /// Add a single edge (used by tests and by the listener integration).
    pub fn add(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Number of edges currently in the store.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// True iff the store has no edges.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// List the outgoing edges of an agent (for a given tenant).
    pub fn outgoing(&self, agent_id: Uuid, _tenant_id: Uuid) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|e| e.from_agent == agent_id && !e.archived)
            .cloned()
            .collect()
    }

    /// List the incoming edges of an agent.
    pub fn incoming(&self, agent_id: Uuid, _tenant_id: Uuid) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|e| e.to_agent == agent_id && !e.archived)
            .cloned()
            .collect()
    }

    /// List all `COLLABORATES_WITH` edges for an agent (undirected, so
    /// we look at both `from_agent` and `to_agent`).
    pub fn collaborates_with(&self, agent_id: Uuid, _tenant_id: Uuid) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|e| {
                e.edge_type == RelationshipType::CollaboratesWith
                    && !e.archived
                    && (e.from_agent == agent_id || e.to_agent == agent_id)
            })
            .cloned()
            .collect()
    }

    /// All edges in the store (for tests / inspection).
    pub fn all(&self) -> &[Edge] {
        &self.edges
    }
}

/// `ARGDispatchRouter` (per DD §4.5 + arch §3.1).
///
/// The router wraps a shared [`InMemoryEdgeStore`] and computes a
/// [`DispatchRoute`] for any given agent. Construction is infallible;
/// routes that find no edges return an empty [`DispatchRoute`].
#[derive(Debug, Clone)]
pub struct ARGDispatchRouter {
    /// Shared in-memory edge store.
    store: Arc<InMemoryEdgeStore>,
}

impl ARGDispatchRouter {
    /// Build a new router wrapping the given store.
    pub fn new(store: Arc<InMemoryEdgeStore>) -> Self {
        Self { store }
    }

    /// Compute a [`DispatchRoute`] for the given agent.
    ///
    /// Returns an empty route (not an error) when no edges match — the
    /// caller can decide whether an empty route is acceptable. To get
    /// an `EffectError::DispatchNoRoute` use [`Self::route_or_error`].
    pub fn route(&self, current_agent_id: Uuid, tenant_id: Uuid) -> DispatchRoute {
        let outgoing = self.store.outgoing(current_agent_id, tenant_id);
        let delegates: Vec<Uuid> = outgoing
            .iter()
            .filter(|e| e.edge_type == RelationshipType::DelegatesTo)
            .map(|e| e.to_agent)
            .collect();

        let incoming = self.store.incoming(current_agent_id, tenant_id);
        let stand_ins: Vec<Uuid> = incoming
            .iter()
            .filter(|e| e.edge_type == RelationshipType::StandInFor)
            .map(|e| e.from_agent)
            .collect();

        let collab_edges = self.store.collaborates_with(current_agent_id, tenant_id);
        let mut seen: HashSet<Uuid> = HashSet::new();
        let mut collaborators: Vec<Uuid> = Vec::new();
        for e in &collab_edges {
            let other = if e.from_agent == current_agent_id {
                e.to_agent
            } else {
                e.from_agent
            };
            if seen.insert(other) {
                collaborators.push(other);
            }
        }

        DispatchRoute {
            delegates,
            stand_ins,
            collaborators,
        }
    }

    /// Like [`Self::route`] but returns `EffectError::DispatchNoRoute`
    /// when the route is empty.
    pub fn route_or_error(
        &self,
        current_agent_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<DispatchRoute, EffectError> {
        let route = self.route(current_agent_id, tenant_id);
        if route.is_empty() {
            Err(EffectError::DispatchNoRoute(current_agent_id))
        } else {
            Ok(route)
        }
    }

    /// Reference to the underlying store (for tests / inspection).
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
    fn route_returns_3_vec_for_full_graph() {
        let tenant = Uuid::new_v4();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        let mut store = InMemoryEdgeStore::default();
        // a delegates to b and c
        store.add(edge(a, b, RelationshipType::DelegatesTo, 0.7, tenant));
        store.add(edge(a, c, RelationshipType::DelegatesTo, 0.6, tenant));
        // d stands in for a
        store.add(edge(d, a, RelationshipType::StandInFor, 0.8, tenant));
        // a collaborates with c (undirected, so we test the seen-set)
        store.add(edge(a, c, RelationshipType::CollaboratesWith, 0.5, tenant));

        let router = ARGDispatchRouter::new(Arc::new(store));
        let route = router.route(a, tenant);
        assert_eq!(route.delegates, vec![b, c]);
        assert_eq!(route.stand_ins, vec![d]);
        assert_eq!(route.collaborators, vec![c]);
        assert_eq!(route.total(), 4);
    }

    #[test]
    fn route_empty_when_no_edges() {
        let store = InMemoryEdgeStore::default();
        let router = ARGDispatchRouter::new(Arc::new(store));
        let route = router.route(Uuid::new_v4(), Uuid::new_v4());
        assert!(route.is_empty());
    }

    #[test]
    fn route_filters_archived() {
        let tenant = Uuid::new_v4();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut e = edge(a, b, RelationshipType::DelegatesTo, 0.5, tenant);
        e.archived = true;
        let store = InMemoryEdgeStore::new(vec![e]);
        let router = ARGDispatchRouter::new(Arc::new(store));
        let route = router.route(a, tenant);
        assert!(route.delegates.is_empty());
    }

    #[test]
    fn route_or_error_when_empty() {
        let store = InMemoryEdgeStore::default();
        let router = ARGDispatchRouter::new(Arc::new(store));
        let agent = Uuid::new_v4();
        let res = router.route_or_error(agent, Uuid::new_v4());
        assert!(matches!(res, Err(EffectError::DispatchNoRoute(_))));
    }
}

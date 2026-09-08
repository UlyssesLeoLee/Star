// SPDX-License-Identifier: MIT OR Apache-2.0
//! `EdgeOps` (per DD-AGENT-RELATIONSHIP-001 §4.3).
//!
//! 7 methods: `create` / `get` / `list` / `update` / `archive` /
//! `outgoing_edges` / `incoming_edges` / `find_edge`. The `create` path
//! runs `Edge::validate()` first so it can be unit-tested without a
//! live Memgraph.

use std::sync::Arc;
use uuid::Uuid;

use crate::client::MemgraphClient;
use crate::error::ARGError;
use crate::models::edge::{Edge, RelationshipType};

use super::event_writer::EventWriter;

/// Filter used by [`EdgeOps::list`].
#[derive(Debug, Clone, Default)]
pub struct EdgeFilter {
    /// Optional type filter.
    pub edge_type: Option<RelationshipType>,
    /// Optional source filter.
    pub from_agent: Option<Uuid>,
    /// Optional target filter.
    pub to_agent: Option<Uuid>,
}

/// Patch used by [`EdgeOps::update`]. Only `weight` / `metadata` are
/// mutable; `from` / `to` / `edge_type` are immutable per DD §4.3.
#[derive(Debug, Clone, Default)]
pub struct EdgePatch {
    /// New weight.
    pub weight: Option<f32>,
    /// Replacement metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Edge CRUD + traversal operations.
#[derive(Debug, Clone)]
pub struct EdgeOps {
    client: Arc<MemgraphClient>,
    event_writer: Arc<EventWriter>,
}

impl EdgeOps {
    /// Build a new ops struct.
    pub fn new(client: Arc<MemgraphClient>, event_writer: Arc<EventWriter>) -> Self {
        Self {
            client,
            event_writer,
        }
    }

    /// Validate the edge, write it and append an `EdgeCreated` event.
    pub async fn create(&self, edge: Edge) -> Result<Edge, ARGError> {
        edge.validate()?;
        let cypher = format!(
            "MATCH (a:Agent {{id: $from}}), (b:Agent {{id: $to}}) \
             CREATE (a)-[r:{} {{id: $id, weight: $weight, ...}}]->(b) \
             RETURN r",
            edge.edge_type.cypher_label()
        );
        self.client
            .execute_write(&cypher, serde_json::json!({}))
            .await?;
        self.event_writer
            .append(crate::models::ARGEvent::EdgeCreated(edge.clone()))
            .await?;
        Ok(edge)
    }

    /// Look up an edge by id.
    pub async fn get(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Edge>, ARGError> {
        let _ = (id, tenant_id);
        Err(ARGError::Other(
            "EdgeOps::get is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// List edges, filtered.
    pub async fn list(&self, filter: EdgeFilter) -> Result<Vec<Edge>, ARGError> {
        let _ = filter;
        Err(ARGError::Other(
            "EdgeOps::list is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Apply a patch with SCD Type 2 versioning.
    pub async fn update(&self, id: Uuid, patch: EdgePatch, actor: Uuid) -> Result<Edge, ARGError> {
        let _ = (id, patch, actor);
        Err(ARGError::Other(
            "EdgeOps::update is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Soft-archive an edge.
    pub async fn archive(&self, id: Uuid, actor: Uuid) -> Result<(), ARGError> {
        let _ = (id, actor);
        Err(ARGError::Other(
            "EdgeOps::archive is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// List the outgoing edges of an agent.
    pub async fn outgoing_edges(
        &self,
        agent_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Vec<Edge>, ARGError> {
        let _ = (agent_id, tenant_id);
        Err(ARGError::Other(
            "EdgeOps::outgoing_edges is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// List the incoming edges of an agent.
    pub async fn incoming_edges(
        &self,
        agent_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Vec<Edge>, ARGError> {
        let _ = (agent_id, tenant_id);
        Err(ARGError::Other(
            "EdgeOps::incoming_edges is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Find an edge by `(from, to, type)` triple (used by the Effect
    /// Tier to look up `CHALLENGES` / `PEER_REVIEWS` edges).
    pub async fn find_edge(
        &self,
        from: Uuid,
        to: Uuid,
        edge_type: RelationshipType,
        tenant_id: Uuid,
    ) -> Result<Option<Edge>, ARGError> {
        let _ = (from, to, edge_type, tenant_id);
        Err(ARGError::Other(
            "EdgeOps::find_edge is a P3-C W1 stub (G-1)".into(),
        ))
    }
}

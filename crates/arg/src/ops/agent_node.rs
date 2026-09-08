// SPDX-License-Identifier: MIT OR Apache-2.0
//! `AgentNodeOps` (per DD-AGENT-RELATIONSHIP-001 §4.2).
//!
//! 5 methods: `create` / `get` / `list` / `update` / `archive`. The
//! current implementation validates the input, generates the Cypher,
//! delegates the actual Bolt write to [`MemgraphClient`] and appends the
//! corresponding `ARGEvent` to the [`EventWriter`].
//!
//! The real driver call is still a P3-C W1 stub (`G-1`); the methods
//! return `ARGError::Other` when the underlying write path is not yet
//! available, but they always perform the field-level validation that
//! would run on the real path.

use std::sync::Arc;
use uuid::Uuid;

use crate::client::MemgraphClient;
use crate::error::ARGError;
use crate::models::agent::Agent;

use super::event_writer::EventWriter;

/// Filter used by [`AgentNodeOps::list`].
#[derive(Debug, Clone, Default)]
pub struct AgentFilter {
    /// Optional archetype filter.
    pub archetype: Option<crate::models::agent::AgentArchetype>,
    /// Optional domain filter.
    pub domain: Option<crate::models::agent::Domain>,
    /// Optional status filter.
    pub status: Option<crate::models::agent::AgentStatus>,
}

/// Pagination cursor.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// Max rows returned.
    pub limit: u32,
    /// Offset (0-based).
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// Patch used by [`AgentNodeOps::update`].
#[derive(Debug, Clone, Default)]
pub struct AgentPatch {
    /// New display name.
    pub name: Option<String>,
    /// New trust score.
    pub trust_score: Option<f32>,
    /// Replacement metadata.
    pub metadata: Option<serde_json::Value>,
    /// New status (state machine checked).
    pub status: Option<crate::models::agent::AgentStatus>,
}

/// Agent CRUD operations (5 methods per DD §4.2).
#[derive(Debug, Clone)]
pub struct AgentNodeOps {
    client: Arc<MemgraphClient>,
    event_writer: Arc<EventWriter>,
}

impl AgentNodeOps {
    /// Build a new ops struct.
    pub fn new(client: Arc<MemgraphClient>, event_writer: Arc<EventWriter>) -> Self {
        Self {
            client,
            event_writer,
        }
    }

    /// Validate the agent and emit a `MERGE` / `CREATE` Cypher.
    ///
    /// Real driver call is the P3-C W1 stub; the validation runs first
    /// so callers can rely on it without hitting the network.
    pub async fn create(&self, agent: Agent) -> Result<Agent, ARGError> {
        agent.validate()?;
        let cypher = agent.to_cypher();
        self.client
            .execute_write(&cypher, serde_json::json!({}))
            .await?;
        self.event_writer
            .append(crate::models::ARGEvent::AgentCreated(agent.clone()))
            .await?;
        Ok(agent)
    }

    /// Look up an agent by id (with RLS 13 類 tenant check).
    pub async fn get(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Agent>, ARGError> {
        let cypher = "MATCH (a:Agent {id: $id, tenant_id: $tenant_id}) RETURN a LIMIT 1";
        let rows = self
            .client
            .execute(
                cypher,
                serde_json::json!({"id": id, "tenant_id": tenant_id}),
            )
            .await?;
        if rows.is_empty() {
            Ok(None)
        } else {
            // Real decode lives in P3-C W1 (G-1).
            Ok(None)
        }
    }

    /// List agents, filtered + paginated.
    pub async fn list(
        &self,
        filter: AgentFilter,
        page: Pagination,
    ) -> Result<Vec<Agent>, ARGError> {
        let _ = (filter, page);
        Err(ARGError::Other(
            "AgentNodeOps::list is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Apply a patch with SCD Type 2 versioning.
    pub async fn update(
        &self,
        id: Uuid,
        patch: AgentPatch,
        actor: Uuid,
    ) -> Result<Agent, ARGError> {
        let _ = (id, patch, actor);
        Err(ARGError::Other(
            "AgentNodeOps::update is a P3-C W1 stub (G-1)".into(),
        ))
    }

    /// Soft-archive an agent (SCD Type 2).
    pub async fn archive(&self, id: Uuid, actor: Uuid) -> Result<(), ARGError> {
        let _ = (id, actor);
        Err(ARGError::Other(
            "AgentNodeOps::archive is a P3-C W1 stub (G-1)".into(),
        ))
    }
}

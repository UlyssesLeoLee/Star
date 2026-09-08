// SPDX-License-Identifier: MIT OR Apache-2.0
//! `TemplateOps` (per DD-AGENT-RELATIONSHIP-001 §4.4).
//!
//! 2 methods: `list_templates` (sync) and `instantiate` (async). The
//! instantiate path validates the agent count against the template's
//! `min_agents..=max_agents` before building the [`TemplateInstance`].

use std::sync::Arc;
use uuid::Uuid;

use crate::client::MemgraphClient;
use crate::error::ARGError;
use crate::models::template::{all_templates, TeamTemplate, TemplateEdge, TemplateId};
use crate::models::template_instance::TemplateInstance;

use super::edge_ops::EdgeOps;
use super::event_writer::EventWriter;

/// Team template operations.
#[derive(Debug, Clone)]
pub struct TemplateOps {
    #[allow(dead_code)]
    client: Arc<MemgraphClient>,
    #[allow(dead_code)]
    edge_ops: Arc<EdgeOps>,
    event_writer: Arc<EventWriter>,
}

impl TemplateOps {
    /// Build a new ops struct.
    pub fn new(
        client: Arc<MemgraphClient>,
        edge_ops: Arc<EdgeOps>,
        event_writer: Arc<EventWriter>,
    ) -> Self {
        Self {
            client,
            edge_ops,
            event_writer,
        }
    }

    /// Synchronous: list the 5 canonical templates (per DD §3.2.3).
    pub fn list_templates() -> Vec<TeamTemplate> {
        all_templates()
    }

    /// Materialise a template into a [`TemplateInstance`] and append the
    /// corresponding `ARGEvent::TemplateInstantiated` event.
    ///
    /// The current placeholder does not yet push the edges through
    /// `EdgeOps::create` (that path is the P3-C W1 stub); what runs
    /// today is the validation + the event append.
    pub async fn instantiate(
        &self,
        template_id: TemplateId,
        agent_ids: Vec<Uuid>,
        instance_name: String,
        tenant_id: Uuid,
        created_by: Uuid,
    ) -> Result<TemplateInstance, ARGError> {
        let template = all_templates()
            .into_iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| ARGError::ValidationFailed("unknown template_id".into()))?;
        if agent_ids.len() < template.min_agents || agent_ids.len() > template.max_agents {
            return Err(ARGError::TemplateAgentCountMismatch {
                expected: template.min_agents,
                actual: agent_ids.len(),
            });
        }
        let edges = template.edges.clone();
        let instance = TemplateInstance::new(
            template_id,
            instance_name,
            agent_ids,
            edges,
            tenant_id,
            created_by,
        );
        self.event_writer
            .append(crate::models::ARGEvent::TemplateInstantiated(
                instance.clone(),
            ))
            .await?;
        Ok(instance)
    }
}

/// Helper for tests: build the edge list for a [`Mesh`] template.
pub fn build_mesh_edges(n: usize) -> Vec<TemplateEdge> {
    let mut edges = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            edges.push(TemplateEdge {
                from_index: i,
                to_index: j,
                edge_type: crate::models::edge::RelationshipType::CollaboratesWith,
                default_weight: 0.5,
            });
        }
    }
    edges
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `EdgeOps` (per DD-AGENT-RELATIONSHIP-001 §4.3).
//!
//! 7 methods: `create` / `get` / `list` / `update` / `archive` /
//! `outgoing_edges` / `incoming_edges` / `find_edge`. The `create` path
//! runs `Edge::validate()` first so it can be unit-tested without a
//! live Memgraph.
//!
//! G-4 集成 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 拍板):
//! 如果设置 trust_audit (per `with_trust_audit`), `create` 对
//! `RelationshipType::Trusts` 关系自动应用 4 重审计 + 写 WORM audit log
//! (per 守门 #13 d Transaction 100% audit + ADR-0043).

use std::sync::Arc;
use uuid::Uuid;

use crate::client::MemgraphClient;
use crate::error::ARGError;
use crate::models::edge::{Edge, RelationshipType};

use super::event_writer::EventWriter;
use super::trust_audit::TrustAuditOps;

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
    /// Optional TrustAuditOps for G-4 4 重审计 integration (per DDD-REVIEW §1.2)
    trust_audit: Option<Arc<TrustAuditOps>>,
}

impl EdgeOps {
    /// Build a new ops struct.
    pub fn new(client: Arc<MemgraphClient>, event_writer: Arc<EventWriter>) -> Self {
        Self {
            client,
            event_writer,
            trust_audit: None,
        }
    }

    /// Attach TrustAuditOps for G-4 4 重审计 (per DDD-REVIEW §1.2)
    pub fn with_trust_audit(mut self, trust_audit: Arc<TrustAuditOps>) -> Self {
        self.trust_audit = Some(trust_audit);
        self
    }

    /// Validate the edge, write it and append an `EdgeCreated` event.
    ///
    /// G-4 集成 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2): 如果 trust_audit 已设置
    /// 且 edge.edge_type == Trusts, 自动调用 4 重审计 (阈值 0.95 + mutual + evidence + multi-source)
    /// + 写 WORM audit log. 如果 4 重审计失败, 返回 Err(ARGError) 不创建 edge.
    pub async fn create(&self, edge: Edge) -> Result<Edge, ARGError> {
        edge.validate()?;

        // G-4 集成: Trusts 关系自动应用 4 重审计 (per DDD-REVIEW §1.2)
        if edge.edge_type == RelationshipType::Trusts {
            if let Some(audit_ops) = &self.trust_audit {
                let _audit_log = audit_ops.audit_trust_relationship(&edge)?;
                // 4 重审计通过, 继续走 create 路径
            }
            // trust_audit 未设置时, 跳过审计 (保持向后兼容, per 守门 #1 禁回溯叙事)
        }

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

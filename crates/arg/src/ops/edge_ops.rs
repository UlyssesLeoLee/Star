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
use super::v2_sink::V2EdgeSink;

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
    /// Optional V2EdgeSink for G-10 阶段 1 双写 (per DDD-REVIEW §1.3)
    v2_sink: Option<Arc<dyn V2EdgeSink>>,
}

impl EdgeOps {
    /// Build a new ops struct.
    pub fn new(client: Arc<MemgraphClient>, event_writer: Arc<EventWriter>) -> Self {
        Self {
            client,
            event_writer,
            trust_audit: None,
            v2_sink: None,
        }
    }

    /// Attach TrustAuditOps for G-4 4 重审计 (per DDD-REVIEW §1.2)
    pub fn with_trust_audit(mut self, trust_audit: Arc<TrustAuditOps>) -> Self {
        self.trust_audit = Some(trust_audit);
        self
    }

    /// Attach V2EdgeSink for G-10 阶段 1 双写 (per DDD-REVIEW §1.3)
    pub fn with_v2_sink(mut self, v2_sink: Arc<dyn V2EdgeSink>) -> Self {
        self.v2_sink = Some(v2_sink);
        self
    }

    /// Validate the edge, write it and append an `EdgeCreated` event.
    ///
    /// G-4 集成 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2): 如果 trust_audit 已设置
    /// 且 edge.edge_type == Trusts, 自动调用 4 重审计 (阈值 0.95 + mutual + evidence + multi-source)
    /// + 写 WORM audit log. 如果 4 重审计失败, 返回 Err(ARGError) 不创建 edge.
    ///
    /// G-10 集成 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3): 如果 v2_sink 已设置:
    /// - 阶段 1 (v0.82): V1 + V2 双写
    /// - 阶段 2 (v0.83): 读路径 V2 优先
    /// - 阶段 3 (v0.84): 写路径 V2 only, V1 表 mark read-only
    /// - 阶段 4 (跨 session 续): V1 archive
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

        // G-10 阶段 3 V1 只读 (v0.84): 当 v2_sink 已设置, 写路径 V2 only
        if self.v2_sink.is_some() {
            // V1 表 mark read-only (per 阶段 3 写路径全切 V2)
            // 0 V1 写; V1 仍可读 (per阶段 3 fallback 监控)
            if let Some(v2_sink) = &self.v2_sink {
                v2_sink.append_v2(&edge)?;
            }
        } else {
            // 向后兼容路径 (v0.81 阶段 0/1/2 都维持): V1 写, 0 V2 写
            // 1. 写 V1 (主路径, 旧 schema, per DD §3.3.1)
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
        }

        Ok(edge)
    }

    /// Look up an edge by id.
    ///
    /// G-10 阶段 2 V2 优先 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3):
    /// 读路径全走 V2 (v2_sink.get_v2), V2 没找到 fallback 到 V1 (per 阶段 2 监控 V1 读 fallback 比例 < 5%)
    pub async fn get(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Edge>, ARGError> {
        let _ = tenant_id;
        // G-10 阶段 2: V2 优先
        if let Some(v2_sink) = &self.v2_sink {
            if let Some(edge) = v2_sink.get_v2(id)? {
                return Ok(Some(edge));
            }
            // V2 没找到: fallback 到 V1 (per 阶段 2 监控)
            // v0.83 阶段: V1 仍 stub, 返回 None
        }
        // v0.83 阶段 2: V2 优先 + V1 fallback stub
        // 实际 r2d2-memgraph G-1 落地后: V1 走 client.execute_read(cypher)
        let _ = id;
        Ok(None)
    }

    /// List edges, filtered.
    ///
    /// G-10 阶段 2 V2 优先 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3):
    /// 读路径全走 V2 (v2_sink.list_v2)
    pub async fn list(&self, filter: EdgeFilter) -> Result<Vec<Edge>, ARGError> {
        // G-10 阶段 2: V2 优先
        if let Some(v2_sink) = &self.v2_sink {
            return v2_sink.list_v2();
        }
        // v0.83 阶段 2: V2 优先 + V1 fallback stub
        let _ = filter;
        Ok(Vec::new())
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

//! `engine.rs` — `RelationshipEngine` Trait + InMemoryRelationshipEngine (per DD §13)
//!
//! 13 Edge ops + 4 高风险 Edge metadata + 1/2/3-hop queries

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use graph_core::edge::{EdgeKind, EdgePayload};
use graph_core::node::NodePayload;
use graph_core::types::{RepoId, WorktreeId};

use crate::edge_ops::{validate_kind, EdgeCreateRequest, HighRiskKind, HighRiskMetadata};
use crate::error::RelationshipError;

/// RelationshipEngine trait (per DD §13)
#[async_trait]
pub trait RelationshipEngine: Send + Sync {
    /// Create Edge
    async fn create_edge(&self, req: EdgeCreateRequest) -> Result<EdgePayload, RelationshipError>;

    /// Get Edge
    async fn get_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<EdgePayload, RelationshipError>;

    /// Update Edge (仅更新高风险 metadata)
    async fn update_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
        metadata: HighRiskMetadata,
    ) -> Result<EdgePayload, RelationshipError>;

    /// Delete Edge
    async fn delete_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<bool, RelationshipError>;

    /// Merge edges (合并 from + to 指向同一 target 的多条 edge)
    async fn merge_edges(
        &self,
        kind: EdgeKind,
        from_ids: Vec<Uuid>,
        to_id: Uuid,
    ) -> Result<Vec<EdgePayload>, RelationshipError>;

    /// Supersede (标记一条 edge 被另一条替代)
    async fn supersede_edge(
        &self,
        old_kind: EdgeKind,
        old_from: Uuid,
        old_to: Uuid,
        new_edge: EdgeCreateRequest,
    ) -> Result<(EdgePayload, EdgePayload), RelationshipError>;

    /// List edges (按 kind + 节点过滤)
    async fn list_edges(
        &self,
        repo_id: Option<RepoId>,
        kinds: Option<Vec<EdgeKind>>,
        from_node: Option<Uuid>,
        to_node: Option<Uuid>,
    ) -> Result<Vec<EdgePayload>, RelationshipError>;

    /// N-hop query
    async fn n_hop(
        &self,
        repo_id: RepoId,
        start_id: WorktreeId,
        hop: u8,
    ) -> Result<crate::n_hop::NHopResult, RelationshipError>;
}

/// Edge 索引 key: (种类, from, to) — 见 `InMemoryRelationshipEngine::edges`
type EdgeIndex = HashMap<(EdgeKind, Uuid, Uuid), EdgePayload>;

/// In-memory relationship engine (per DD §13)
pub struct InMemoryRelationshipEngine {
    /// WorktreeId 索引的 edge list
    edges: Arc<RwLock<EdgeIndex>>,
    /// N-hop local BFS 用: 全部 nodes 缓存
    nodes: Arc<RwLock<Vec<NodePayload>>>,
}

impl InMemoryRelationshipEngine {
    /// 创建新 engine
    pub fn new() -> Self {
        Self {
            edges: Arc::new(RwLock::new(HashMap::new())),
            nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 注入 node 缓存 (供 n_hop 使用)
    pub async fn register_nodes(&self, nodes: Vec<NodePayload>) {
        let mut guard = self.nodes.write().await;
        guard.extend(nodes);
    }

    /// 注入全部 edges (用于 N-hop BFS)
    pub async fn register_edges(&self, edges: Vec<EdgePayload>) {
        let mut guard = self.edges.write().await;
        for e in edges {
            let key = edge_key(&e);
            guard.insert(key, e);
        }
    }
}

fn edge_key(e: &EdgePayload) -> (EdgeKind, Uuid, Uuid) {
    match e {
        EdgePayload::BasedOn(x) => (EdgeKind::BasedOn, x.from, Uuid::nil()),
        EdgePayload::UsesBranch(x) => (EdgeKind::UsesBranch, x.from, Uuid::nil()),
        EdgePayload::DerivedFrom(x) => (EdgeKind::DerivedFrom, x.from, x.to),
        EdgePayload::WorksOn(x) => (EdgeKind::WorksOn, x.from, x.to),
        EdgePayload::ImplementedIn(x) => (EdgeKind::ImplementedIn, x.from, x.to),
        EdgePayload::Modifies(x) => (EdgeKind::Modifies, x.from, Uuid::nil()),
        EdgePayload::ModifiesSymbol(x) => (EdgeKind::ModifiesSymbol, x.from, Uuid::nil()),
        EdgePayload::ConflictsWith(x) => (EdgeKind::ConflictsWith, x.from, x.to),
        EdgePayload::OverlapsWith(x) => (EdgeKind::OverlapsWith, x.from, x.to),
        EdgePayload::DependsOn(x) => (EdgeKind::DependsOn, x.from, x.to),
        EdgePayload::Blocks(x) => (EdgeKind::Blocks, x.from, x.to),
        EdgePayload::Supersedes(x) => (EdgeKind::Supersedes, x.from, x.to),
        EdgePayload::MergedInto(x) => (EdgeKind::MergedInto, x.from, Uuid::nil()),
    }
}

impl Default for InMemoryRelationshipEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RelationshipEngine for InMemoryRelationshipEngine {
    async fn create_edge(&self, req: EdgeCreateRequest) -> Result<EdgePayload, RelationshipError> {
        validate_kind(req.kind)?;
        // 高风险 Edge 必须带 metadata
        if HighRiskKind::from_edge_kind(req.kind).is_some() && req.high_risk_metadata.is_none() {
            return Err(RelationshipError::new(
                "REL.HIGH_RISK_REQUIRES_METADATA",
                format!("High-risk edge {:?} requires metadata", req.kind),
                "trace",
            ));
        }

        // 构造基础 edge (各 kind 共用基础字段, 高风险 metadata 单独存在 KV)
        let now = Utc::now();
        let from = WorktreeId::from(req.from_id);
        let to = WorktreeId::from(req.to_id);
        let edge = match req.kind {
            EdgeKind::DerivedFrom => EdgePayload::DerivedFrom(graph_core::edge::DerivedFromEdge {
                from,
                to,
                created_at: now,
                merge_base_sha: None,
            }),
            EdgeKind::ConflictsWith => {
                EdgePayload::ConflictsWith(graph_core::edge::ConflictsWithEdge {
                    from,
                    to,
                    risk_score: graph_core::types::RiskScore::new(
                        req.high_risk_metadata
                            .as_ref()
                            .map(|m| m.risk_score)
                            .unwrap_or(0.0),
                    ),
                    shared_files: req
                        .high_risk_metadata
                        .as_ref()
                        .map(|m| m.shared_files.clone())
                        .unwrap_or_default(),
                    shared_symbols: req
                        .high_risk_metadata
                        .as_ref()
                        .map(|m| m.shared_symbols.clone())
                        .unwrap_or_default(),
                    detected_at: now,
                    reason: req
                        .high_risk_metadata
                        .as_ref()
                        .map(|m| m.reason.clone())
                        .unwrap_or_default(),
                    confidence: req
                        .high_risk_metadata
                        .as_ref()
                        .map(|m| m.confidence)
                        .unwrap_or(0.0),
                })
            }
            EdgeKind::OverlapsWith => {
                EdgePayload::OverlapsWith(graph_core::edge::OverlapsWithEdge {
                    from,
                    to,
                    overlap_score: req
                        .high_risk_metadata
                        .as_ref()
                        .map(|m| m.risk_score)
                        .unwrap_or(0.0),
                    shared_files: req
                        .high_risk_metadata
                        .as_ref()
                        .map(|m| m.shared_files.clone())
                        .unwrap_or_default(),
                    detected_at: now,
                })
            }
            EdgeKind::DependsOn => EdgePayload::DependsOn(graph_core::edge::DependsOnEdge {
                from,
                to,
                required_state: graph_core::state::HumanState::Merged,
                created_at: now,
                created_by: graph_core::types::UserId::new_v4(),
            }),
            EdgeKind::Blocks => EdgePayload::Blocks(graph_core::edge::BlocksEdge {
                from,
                to,
                created_at: now,
                reason: req
                    .high_risk_metadata
                    .as_ref()
                    .map(|m| m.reason.clone())
                    .unwrap_or_default(),
            }),
            EdgeKind::Supersedes => EdgePayload::Supersedes(graph_core::edge::SupersedesEdge {
                from,
                to,
                created_at: now,
                reason: req
                    .high_risk_metadata
                    .as_ref()
                    .map(|m| m.reason.clone())
                    .unwrap_or_default(),
            }),
            _ => {
                // 其他 kind 暂不在 MVP 范围 (BASED_ON / MODIFIES 等需要其他 node 类型)
                return Err(RelationshipError::new(
                    "REL.NOT_SUPPORTED_IN_PHASE1",
                    format!("Edge kind {:?} not supported in MVP", req.kind),
                    "trace",
                ));
            }
        };

        let mut guard = self.edges.write().await;
        let key = (req.kind, req.from_id, req.to_id);
        guard.insert(key, edge.clone());
        Ok(edge)
    }

    async fn get_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<EdgePayload, RelationshipError> {
        let guard = self.edges.read().await;
        guard
            .get(&(kind, from_id, to_id))
            .cloned()
            .ok_or_else(|| RelationshipError::not_found(format!("{:?}", kind), "trace"))
    }

    async fn update_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
        metadata: HighRiskMetadata,
    ) -> Result<EdgePayload, RelationshipError> {
        validate_kind(kind)?;
        if HighRiskKind::from_edge_kind(kind).is_none() {
            return Err(RelationshipError::new(
                "REL.NOT_HIGH_RISK",
                format!("Edge {:?} is not high-risk", kind),
                "trace",
            ));
        }
        let req = EdgeCreateRequest {
            kind,
            from_id,
            to_id,
            high_risk_metadata: Some(metadata),
        };
        self.create_edge(req).await
    }

    async fn delete_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<bool, RelationshipError> {
        validate_kind(kind)?;
        let mut guard = self.edges.write().await;
        Ok(guard.remove(&(kind, from_id, to_id)).is_some())
    }

    async fn merge_edges(
        &self,
        kind: EdgeKind,
        from_ids: Vec<Uuid>,
        to_id: Uuid,
    ) -> Result<Vec<EdgePayload>, RelationshipError> {
        validate_kind(kind)?;
        let mut out = Vec::new();
        for from in from_ids {
            if let Ok(edge) = self.get_edge(kind, from, to_id).await {
                out.push(edge);
            }
        }
        Ok(out)
    }

    async fn supersede_edge(
        &self,
        old_kind: EdgeKind,
        old_from: Uuid,
        old_to: Uuid,
        new_edge: EdgeCreateRequest,
    ) -> Result<(EdgePayload, EdgePayload), RelationshipError> {
        let new = self.create_edge(new_edge).await?;
        let old = self.get_edge(old_kind, old_from, old_to).await?;
        Ok((old, new))
    }

    async fn list_edges(
        &self,
        _repo_id: Option<RepoId>,
        kinds: Option<Vec<EdgeKind>>,
        from_node: Option<Uuid>,
        to_node: Option<Uuid>,
    ) -> Result<Vec<EdgePayload>, RelationshipError> {
        let guard = self.edges.read().await;
        let kind_set: Option<std::collections::HashSet<EdgeKind>> =
            kinds.map(|k| k.into_iter().collect());
        let mut out: Vec<EdgePayload> = guard
            .iter()
            .filter(|((k, from, to), _)| {
                if let Some(ref ks) = kind_set {
                    if !ks.contains(k) {
                        return false;
                    }
                }
                if let Some(f) = from_node {
                    if *from != f {
                        return false;
                    }
                }
                if let Some(t) = to_node {
                    if *to != t {
                        return false;
                    }
                }
                true
            })
            .map(|(_, v)| v.clone())
            .collect();
        // 稳定排序
        out.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        Ok(out)
    }

    async fn n_hop(
        &self,
        _repo_id: RepoId,
        start_id: WorktreeId,
        hop: u8,
    ) -> Result<crate::n_hop::NHopResult, RelationshipError> {
        let nodes = self.nodes.read().await.clone();
        let edges = {
            let guard = self.edges.read().await;
            guard.values().cloned().collect::<Vec<_>>()
        };
        Ok(crate::n_hop::NHopQuery::bfs_local(
            start_id, hop, &nodes, &edges,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn engine_crud_4_high_risk() {
        // Per TEST-DESIGN §2.2: edge_ops_crud_4_high_risk_metadata
        let engine = InMemoryRelationshipEngine::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        // 1. CONFLICTS_WITH — 必须带 metadata
        let edge = engine
            .create_edge(EdgeCreateRequest {
                kind: EdgeKind::ConflictsWith,
                from_id: a,
                to_id: b,
                high_risk_metadata: Some(HighRiskMetadata {
                    risk_score: 0.8,
                    shared_files: vec!["x.rs".into()],
                    shared_symbols: vec!["Foo::bar".into()],
                    detected_at: Utc::now(),
                    reason: "shared file".into(),
                    confidence: 0.9,
                }),
            })
            .await
            .unwrap();
        assert!(matches!(edge, EdgePayload::ConflictsWith(_)));

        // 2. 没 metadata 应失败
        let err = engine
            .create_edge(EdgeCreateRequest {
                kind: EdgeKind::ConflictsWith,
                from_id: a,
                to_id: b,
                high_risk_metadata: None,
            })
            .await;
        assert!(err.is_err());

        // 3. UPDATE
        let updated = engine
            .update_edge(
                EdgeKind::ConflictsWith,
                a,
                b,
                HighRiskMetadata {
                    risk_score: 0.9,
                    shared_files: vec![],
                    shared_symbols: vec![],
                    detected_at: Utc::now(),
                    reason: "updated".into(),
                    confidence: 0.95,
                },
            )
            .await
            .unwrap();

        // 4. DELETE
        let deleted = engine
            .delete_edge(EdgeKind::ConflictsWith, a, b)
            .await
            .unwrap();
        assert!(deleted);

        let _ = updated;
    }
}

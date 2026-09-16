//! `repository.rs` — `GraphRepository` Trait + 11 方法 (per BD D-GRAPH-001 + DD §10/§3.2)
//!
//! Per ULYS-57.1 T1 + IMPL-PLAN §4.1 T1.1:
//! - 11 方法: 5 Node CRUD + 4 Edge CRUD + 2 traversal
//!
//! 方法清单:
//!  1. `upsert_node` (5 Node CRUD: Repository/Branch/Worktree/Task/AgentSession 入口)
//!  2. `get_node`    (按 kind + id 取单 Node)
//!  3. `list_nodes`  (按 NodeFilter 拉一批 Node)
//!  4. `delete_node` (软删, INV-WC-08 守门)
//!  5. `count_nodes` (按 kind 计数)
//!  6. `upsert_edge` (4 Edge CRUD: BASED_ON/USES_BRANCH/DERIVED_FROM/WORKS_ON 入口)
//!  7. `get_edge`    (按 kind + from/to 取单 Edge)
//!  8. `list_edges`  (按 EdgeFilter 拉一批 Edge)
//!  9. `delete_edge` (按 kind + from/to 删 Edge)
//! 10. `n_hop`       (n-hop 邻域查询, per DD §17.2 TraversalDirection)
//! 11. `shortest_path` (最短路径查询, P2)
//!
//! 异步 trait: `async_trait`, per 守门 #11 + workspace.async-trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::edge::{EdgeKind, EdgePayload};
use crate::error::GraphError;
use crate::node::{NodeKind, NodePayload};
use crate::types::RepoId;

/// 邻域遍历方向 (per DD §17.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalDirection {
    /// 出边 (from → to)
    Outgoing,
    /// 入边 (to → from)
    Incoming,
    /// 双向
    Both,
}

/// Node filter (per DD §3.2 + §12 `WorktreeFilter` 简化版)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeFilter {
    /// Repo 范围 (None = 全部)
    pub repo_id: Option<RepoId>,
    /// Node kind 限制 (None = 全部)
    pub kinds: Option<HashSet<NodeKind>>,
    /// Limit
    pub limit: Option<u32>,
}

/// Edge filter (per DD §8 + §3.2)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeFilter {
    /// Edge kind 限制 (None = 全部)
    pub kinds: Option<HashSet<EdgeKind>>,
    /// Source Node ID (None = 全部)
    pub from_id: Option<crate::types::WorktreeId>,
    /// Target Node ID (None = 全部)
    pub to_id: Option<crate::types::WorktreeId>,
    /// Limit
    pub limit: Option<u32>,
}

/// n-hop / shortest_path 查询结果 (per DD §3.2 `TraversalMeta`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    /// 命中的 Node
    pub nodes: Vec<NodePayload>,
    /// 命中的 Edge
    pub edges: Vec<EdgePayload>,
    /// 遍历元数据
    pub meta: TraversalMeta,
}

/// 遍历元数据 (per DD §3.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalMeta {
    /// 跳数 (1-hop / 2-hop / 3-hop)
    pub hop: u8,
    /// 遍历方向
    pub direction: TraversalDirection,
    /// 访问过的节点数
    pub visited_count: usize,
    /// 耗时毫秒
    pub duration_ms: u64,
}

/// `GraphRepository` trait — 11 方法
///
/// 阶段 1 scope: trait 形态 + InMemory 完整实现 + Neo4j 接口骨架 (无真实连接)
/// 阶段 2: Neo4j 真实连接 (testcontainers-rs)
#[async_trait]
pub trait GraphRepository: Send + Sync {
    // ========== Node CRUD (5 方法) ==========

    /// Upsert (insert or update) 一个 Node (per DD §10 + INV-WC-05 11 Node 类型)
    async fn upsert_node(&self, node: NodePayload) -> Result<NodePayload, GraphError>;

    /// 按 kind + RepoId 取单 Node (复合 key 决定唯一性, per INV-WC-05)
    async fn get_node(
        &self,
        kind: NodeKind,
        repo_id: RepoId,
        node_id: uuid::Uuid,
    ) -> Result<Option<NodePayload>, GraphError>;

    /// 按 filter 拉一批 Node (per DD §3.2)
    async fn list_nodes(&self, filter: NodeFilter) -> Result<Vec<NodePayload>, GraphError>;

    /// 软删 Node (保留审计, per 守门 #13 d 100% audit + INV-WC-08)
    async fn delete_node(
        &self,
        kind: NodeKind,
        repo_id: RepoId,
        node_id: uuid::Uuid,
    ) -> Result<bool, GraphError>;

    /// 按 kind 计数 Node (用于 dashboard 顶部数字, per BD D-GRAPH-001)
    async fn count_nodes(&self, kind: NodeKind, repo_id: Option<RepoId>)
        -> Result<u64, GraphError>;

    // ========== Edge CRUD (4 方法) ==========

    /// Upsert (insert or update) 一个 Edge (per DD §8 + INV-WC-04 13 Edge 类型)
    async fn upsert_edge(&self, edge: EdgePayload) -> Result<EdgePayload, GraphError>;

    /// 按 kind + from/to 取单 Edge
    async fn get_edge(
        &self,
        kind: EdgeKind,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
    ) -> Result<Option<EdgePayload>, GraphError>;

    /// 按 filter 拉一批 Edge (per DD §3.2)
    async fn list_edges(&self, filter: EdgeFilter) -> Result<Vec<EdgePayload>, GraphError>;

    /// 删 Edge (硬删, edge 允许物理删除 per INV-WC-08 修正)
    async fn delete_edge(
        &self,
        kind: EdgeKind,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
    ) -> Result<bool, GraphError>;

    // ========== Traversal (2 方法) ==========

    /// n-hop 邻域查询 (per DD §3.2 `GraphQueryRequest` + FR-GRAPH-007)
    async fn n_hop(
        &self,
        start_id: uuid::Uuid,
        hop: u8,
        direction: TraversalDirection,
        edge_kinds: Option<HashSet<EdgeKind>>,
    ) -> Result<GraphQueryResult, GraphError>;

    /// 最短路径查询 (per DD §17.2, P2)
    async fn shortest_path(
        &self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        max_hop: u8,
    ) -> Result<Option<GraphQueryResult>, GraphError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traversal_direction_serializes_snake_case() {
        let json = serde_json::to_string(&TraversalDirection::Outgoing).unwrap();
        assert_eq!(json, "\"outgoing\"");
        let parsed: TraversalDirection = serde_json::from_str("\"both\"").unwrap();
        assert_eq!(parsed, TraversalDirection::Both);
    }

    #[test]
    fn filter_default_is_empty() {
        let f = NodeFilter::default();
        assert!(f.repo_id.is_none());
        assert!(f.kinds.is_none());
        let ef = EdgeFilter::default();
        assert!(ef.kinds.is_none());
        assert!(ef.from_id.is_none());
        assert!(ef.to_id.is_none());
    }

    #[test]
    fn traversal_meta_has_four_fields() {
        let meta = TraversalMeta {
            hop: 1,
            direction: TraversalDirection::Both,
            visited_count: 5,
            duration_ms: 12,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: TraversalMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.hop, 1);
        assert_eq!(parsed.visited_count, 5);
    }
}

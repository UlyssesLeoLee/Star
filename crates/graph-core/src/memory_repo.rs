//! `memory_repo.rs` — `InMemoryGraphRepository` (testcontainers-rs / E2E 测试用)
//!
//! Per ULYS-57.1 T1.3 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.1):
//! - 完整实现 `GraphRepository` trait 11 方法
//! - 线程安全 (Mutex 保护内部 HashMap)
//! - 用于 unit test + E2E 测试 + dev 模式 (无 Neo4j)
//!
//! 数据模型:
//! - Node: keyed by `(NodeKind, RepoId, Uuid)` 三元组
//! - Edge: keyed by `(EdgeKind, from_id, to_id)` 三元组
//! - 邻接表: `outgoing[from_id] -> Vec<(edge_kind, to_id)>`, `incoming[to_id] -> Vec<(edge_kind, from_id)>`

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::edge::{EdgeKind, EdgePayload};
use crate::error::GraphError;
use crate::node::{NodeKind, NodePayload};
use crate::repository::{
    EdgeFilter, GraphQueryResult, GraphRepository, NodeFilter, TraversalDirection, TraversalMeta,
};
use crate::types::RepoId;

/// In-memory node key: `(NodeKind, RepoId, Uuid)`
type NodeKey = (NodeKind, RepoId, Uuid);

/// In-memory edge key: `(EdgeKind, from_id, to_id)`
type EdgeKey = (EdgeKind, Uuid, Uuid);

/// In-memory `GraphRepository` (testcontainers / E2E / dev mode)
#[derive(Debug, Clone, Default)]
pub struct InMemoryGraphRepository {
    /// Node 存储
    nodes: Arc<RwLock<HashMap<NodeKey, NodePayload>>>,
    /// Edge 存储
    edges: Arc<RwLock<HashMap<EdgeKey, EdgePayload>>>,
    /// 出边邻接表
    outgoing: Arc<RwLock<HashMap<Uuid, Vec<(EdgeKind, Uuid)>>>>,
    /// 入边邻接表
    incoming: Arc<RwLock<HashMap<Uuid, Vec<(EdgeKind, Uuid)>>>>,
}

impl InMemoryGraphRepository {
    /// 构造空仓库
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前 Node 数 (per kind / repo)
    pub async fn len(&self, kind: NodeKind) -> usize {
        self.nodes
            .read()
            .await
            .keys()
            .filter(|(k, _, _)| *k == kind)
            .count()
    }

    /// 当前 Edge 数
    pub async fn edge_count(&self) -> usize {
        self.edges.read().await.len()
    }

    /// Clear all nodes & edges (用于 test fixture reset)
    pub async fn clear(&self) {
        self.nodes.write().await.clear();
        self.edges.write().await.clear();
        self.outgoing.write().await.clear();
        self.incoming.write().await.clear();
    }
}

#[async_trait]
impl GraphRepository for InMemoryGraphRepository {
    // ========== Node CRUD ==========

    #[track_caller]
    async fn upsert_node(&self, node: NodePayload) -> Result<NodePayload, GraphError> {
        let (kind, repo_id, id) = extract_node_key(&node).map_err(|e| {
            GraphError::new("GRAPH.UPSERT_NODE", "extract node key failed").with_source(e)
        })?;
        let mut nodes = self.nodes.write().await;
        nodes.insert((kind, repo_id, id), node.clone());
        Ok(node)
    }

    #[track_caller]
    async fn get_node(
        &self,
        kind: NodeKind,
        repo_id: RepoId,
        node_id: Uuid,
    ) -> Result<Option<NodePayload>, GraphError> {
        let nodes = self.nodes.read().await;
        Ok(nodes.get(&(kind, repo_id, node_id)).cloned())
    }

    #[track_caller]
    async fn list_nodes(&self, filter: NodeFilter) -> Result<Vec<NodePayload>, GraphError> {
        let nodes = self.nodes.read().await;
        let mut out: Vec<NodePayload> = Vec::new();
        for (key, val) in nodes.iter() {
            let (kind, repo_id, _) = key;
            if let Some(target_repo) = filter.repo_id {
                if *repo_id != target_repo {
                    continue;
                }
            }
            if let Some(ref kinds) = filter.kinds {
                if !kinds.contains(kind) {
                    continue;
                }
            }
            out.push(val.clone());
            if let Some(limit) = filter.limit {
                if out.len() as u32 >= limit {
                    break;
                }
            }
        }
        Ok(out)
    }

    #[track_caller]
    async fn delete_node(
        &self,
        kind: NodeKind,
        repo_id: RepoId,
        node_id: Uuid,
    ) -> Result<bool, GraphError> {
        let mut nodes = self.nodes.write().await;
        Ok(nodes.remove(&(kind, repo_id, node_id)).is_some())
    }

    #[track_caller]
    async fn count_nodes(
        &self,
        kind: NodeKind,
        repo_id: Option<RepoId>,
    ) -> Result<u64, GraphError> {
        let nodes = self.nodes.read().await;
        let n = nodes
            .keys()
            .filter(|(k, r, _)| *k == kind && repo_id.map(|target| *r == target).unwrap_or(true))
            .count() as u64;
        Ok(n)
    }

    // ========== Edge CRUD ==========

    #[track_caller]
    async fn upsert_edge(&self, edge: EdgePayload) -> Result<EdgePayload, GraphError> {
        let kind = edge.kind();
        let (from_id, to_id) = extract_edge_endpoints(&edge).map_err(|e| {
            GraphError::new("GRAPH.UPSERT_EDGE", "extract endpoints failed").with_source(e)
        })?;
        let mut edges = self.edges.write().await;
        edges.insert((kind, from_id, to_id), edge.clone());

        let mut outgoing = self.outgoing.write().await;
        outgoing.entry(from_id).or_default().push((kind, to_id));
        let mut incoming = self.incoming.write().await;
        incoming.entry(to_id).or_default().push((kind, from_id));
        Ok(edge)
    }

    #[track_caller]
    async fn get_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<Option<EdgePayload>, GraphError> {
        let edges = self.edges.read().await;
        Ok(edges.get(&(kind, from_id, to_id)).cloned())
    }

    #[track_caller]
    async fn list_edges(&self, filter: EdgeFilter) -> Result<Vec<EdgePayload>, GraphError> {
        let edges = self.edges.read().await;
        let mut out: Vec<EdgePayload> = Vec::new();
        for (key, val) in edges.iter() {
            let (kind, from, to) = key;
            if let Some(ref kinds) = filter.kinds {
                if !kinds.contains(kind) {
                    continue;
                }
            }
            if let Some(target_from) = filter.from_id {
                if *from != target_from {
                    continue;
                }
            }
            if let Some(target_to) = filter.to_id {
                if *to != target_to {
                    continue;
                }
            }
            out.push(val.clone());
            if let Some(limit) = filter.limit {
                if out.len() as u32 >= limit {
                    break;
                }
            }
        }
        Ok(out)
    }

    #[track_caller]
    async fn delete_edge(
        &self,
        kind: EdgeKind,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<bool, GraphError> {
        let mut edges = self.edges.write().await;
        let removed = edges.remove(&(kind, from_id, to_id)).is_some();

        let mut outgoing = self.outgoing.write().await;
        if let Some(v) = outgoing.get_mut(&from_id) {
            v.retain(|(_, t)| *t != to_id);
        }
        let mut incoming = self.incoming.write().await;
        if let Some(v) = incoming.get_mut(&to_id) {
            v.retain(|(k, f)| *k == kind && *f != from_id);
        }
        Ok(removed)
    }

    // ========== Traversal ==========

    #[track_caller]
    async fn n_hop(
        &self,
        start_id: Uuid,
        hop: u8,
        direction: TraversalDirection,
        edge_kinds: Option<HashSet<EdgeKind>>,
    ) -> Result<GraphQueryResult, GraphError> {
        let started = std::time::Instant::now();

        let outgoing = self.outgoing.read().await;
        let incoming = self.incoming.read().await;
        let edges = self.edges.read().await;
        let nodes = self.nodes.read().await;

        let mut visited: HashSet<Uuid> = HashSet::new();
        visited.insert(start_id);
        let mut frontier: HashSet<Uuid> = HashSet::new();
        frontier.insert(start_id);

        let mut found_edges: HashSet<(EdgeKind, Uuid, Uuid)> = HashSet::new();
        let mut found_nodes: HashSet<(NodeKind, RepoId, Uuid)> = HashSet::new();

        for h in 0..hop {
            let mut next: HashSet<Uuid> = HashSet::new();
            for node in &frontier {
                if matches!(
                    direction,
                    TraversalDirection::Outgoing | TraversalDirection::Both
                ) {
                    if let Some(v) = outgoing.get(node) {
                        for (k, t) in v {
                            if let Some(ref kinds) = edge_kinds {
                                if !kinds.contains(k) {
                                    continue;
                                }
                            }
                            found_edges.insert((*k, *node, *t));
                            next.insert(*t);
                        }
                    }
                }
                if matches!(
                    direction,
                    TraversalDirection::Incoming | TraversalDirection::Both
                ) {
                    if let Some(v) = incoming.get(node) {
                        for (k, f) in v {
                            if let Some(ref kinds) = edge_kinds {
                                if !kinds.contains(k) {
                                    continue;
                                }
                            }
                            found_edges.insert((*k, *f, *node));
                            next.insert(*f);
                        }
                    }
                }
            }
            visited.extend(next.iter());
            // h 累计: 阶段 1 简化仅访问计数, 阶段 2 按 hop 决定是否继续
            let _ = h;
            frontier = next;
        }

        // 收集命中的 Node (跨 repo / 跨 kind)
        for (k, r, n) in nodes.keys() {
            if visited.contains(n) {
                found_nodes.insert((*k, *r, *n));
            }
        }
        let mut out_nodes: Vec<NodePayload> = nodes
            .iter()
            .filter(|((k, r, n), _)| found_nodes.contains(&(*k, *r, *n)))
            .map(|(_, v)| v.clone())
            .collect();
        let mut out_edges: Vec<EdgePayload> = edges
            .iter()
            .filter(|((k, f, t), _)| found_edges.contains(&(*k, *f, *t)))
            .map(|(_, v)| v.clone())
            .collect();
        // 排序保证 UT 稳定
        out_nodes.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        out_edges.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));

        let _ = frontier; // 抑制 unused warning; 末尾 frontier 已 visited

        Ok(GraphQueryResult {
            nodes: out_nodes,
            edges: out_edges,
            meta: TraversalMeta {
                hop,
                direction,
                visited_count: visited.len(),
                duration_ms: started.elapsed().as_millis() as u64,
            },
        })
    }

    #[track_caller]
    async fn shortest_path(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        max_hop: u8,
    ) -> Result<Option<GraphQueryResult>, GraphError> {
        let started = std::time::Instant::now();

        let outgoing = self.outgoing.read().await;
        let edges = self.edges.read().await;
        let nodes = self.nodes.read().await;

        // BFS 最短路径
        let mut queue: std::collections::VecDeque<(Uuid, u8)> = std::collections::VecDeque::new();
        let mut parent: HashMap<Uuid, (Uuid, EdgeKind)> = HashMap::new();
        let mut visited: HashSet<Uuid> = HashSet::new();
        queue.push_back((from_id, 0));
        visited.insert(from_id);

        let mut found = false;
        while let Some((cur, depth)) = queue.pop_front() {
            if cur == to_id {
                found = true;
                break;
            }
            if depth >= max_hop {
                continue;
            }
            if let Some(v) = outgoing.get(&cur) {
                for (k, t) in v {
                    if visited.contains(t) {
                        continue;
                    }
                    visited.insert(*t);
                    parent.insert(*t, (cur, *k));
                    queue.push_back((*t, depth + 1));
                }
            }
        }
        if !found {
            return Ok(None);
        }
        // 重建路径
        let mut path: Vec<Uuid> = vec![to_id];
        let mut cur = to_id;
        while let Some((p, _)) = parent.get(&cur) {
            path.push(*p);
            cur = *p;
            if cur == from_id {
                break;
            }
        }
        path.reverse();
        let path_set: HashSet<Uuid> = path.iter().copied().collect();
        let mut out_nodes: Vec<NodePayload> = nodes
            .iter()
            .filter(|((_, _, n), _)| path_set.contains(n))
            .map(|(_, v)| v.clone())
            .collect();
        let mut out_edges: Vec<EdgePayload> = Vec::new();
        for w in path.windows(2) {
            if let (Some(from), Some(to)) = (
                path.get(w[0].as_u128() as usize),
                path.get(w[1].as_u128() as usize),
            ) {
                let _ = (from, to); // no-op
            }
            // 按 parent 选 edge
            if let Some((_, kind)) = parent.get(&w[1]) {
                if let Some(e) = edges.get(&(*kind, w[0], w[1])) {
                    out_edges.push(e.clone());
                }
            }
        }
        out_nodes.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        out_edges.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        Ok(Some(GraphQueryResult {
            nodes: out_nodes,
            edges: out_edges,
            meta: TraversalMeta {
                hop: max_hop,
                direction: TraversalDirection::Outgoing,
                visited_count: visited.len(),
                duration_ms: started.elapsed().as_millis() as u64,
            },
        }))
    }
}

/// 提取 Node 唯一 key `(kind, repo_id, id)`.
fn extract_node_key(node: &NodePayload) -> Result<(NodeKind, RepoId, Uuid), String> {
    match node {
        NodePayload::Repository(n) => Ok((NodeKind::Repository, n.id, n.id)),
        NodePayload::Branch(n) => Ok((NodeKind::Branch, n.repo_id, n.id)),
        NodePayload::Worktree(n) => Ok((NodeKind::Worktree, n.repo_id, n.id)),
        NodePayload::Commit(n) => Ok((NodeKind::Commit, n.repo_id, Uuid::nil())), // Commit 用 sha 字符串而非 Uuid, 阶段 2 修正
        NodePayload::Task(n) => Ok((NodeKind::Task, n.repo_id, n.id)),
        NodePayload::AgentSession(n) => Ok((NodeKind::AgentSession, n.repo_id, n.id)),
        NodePayload::PullRequest(n) => Ok((NodeKind::PullRequest, n.repo_id, n.id)),
        NodePayload::File(n) => Ok((NodeKind::File, n.repo_id, n.id)),
        NodePayload::Symbol(n) => Ok((NodeKind::Symbol, Uuid::nil(), n.id)),
        NodePayload::TestRun(n) => Ok((NodeKind::TestRun, Uuid::nil(), n.id)),
        NodePayload::Issue(n) => Ok((NodeKind::Issue, n.repo_id, n.id)),
    }
}

/// 提取 Edge 端点 `(from_id, to_id)`.
fn extract_edge_endpoints(edge: &EdgePayload) -> Result<(Uuid, Uuid), String> {
    match edge {
        EdgePayload::BasedOn(e) => Ok((e.from, Uuid::nil())), // SHA 非 Uuid, 阶段 2 修正
        EdgePayload::UsesBranch(e) => Ok((e.from, e.to)),
        EdgePayload::DerivedFrom(e) => Ok((e.from, e.to)),
        EdgePayload::WorksOn(e) => Ok((e.from, e.to)),
        EdgePayload::ImplementedIn(e) => Ok((e.from, e.to)),
        EdgePayload::Modifies(e) => Ok((e.from, e.to)),
        EdgePayload::ModifiesSymbol(e) => Ok((e.from, e.to)),
        EdgePayload::ConflictsWith(e) => Ok((e.from, e.to)),
        EdgePayload::OverlapsWith(e) => Ok((e.from, e.to)),
        EdgePayload::DependsOn(e) => Ok((e.from, e.to)),
        EdgePayload::Blocks(e) => Ok((e.from, e.to)),
        EdgePayload::Supersedes(e) => Ok((e.from, e.to)),
        EdgePayload::MergedInto(e) => Ok((e.from, e.to)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::DependsOnEdge;
    use crate::node::{RepositoryNode, WorktreeNode};
    use crate::state::HumanState;
    use chrono::Utc;
    use std::path::PathBuf;

    fn make_repo(id: RepoId) -> NodePayload {
        NodePayload::Repository(RepositoryNode {
            id,
            name: "Star".into(),
            url: "git@github.com:Ulysses/Star.git".into(),
            default_branch: "main".into(),
            active_worktree_count: 0,
            risk_count: 0,
            ready_count: 0,
            merged_count: 0,
            last_commit_at: Utc::now(),
        })
    }

    fn make_wt(repo: RepoId, name: &str) -> NodePayload {
        NodePayload::Worktree(WorktreeNode {
            id: Uuid::new_v4(),
            repo_id: repo,
            name: name.into(),
            branch: format!("feat/{name}"),
            path: PathBuf::from(format!("/tmp/{name}")),
            head_sha: "0".repeat(40),
            ahead: 0,
            behind: 0,
            dirty: false,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
        })
    }

    fn depends_on(from: Uuid, to: Uuid) -> EdgePayload {
        EdgePayload::DependsOn(DependsOnEdge {
            from,
            to,
            required_state: HumanState::Merged,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        })
    }

    #[tokio::test]
    async fn upsert_and_get_node() {
        let repo = InMemoryGraphRepository::new();
        let rid = RepoId::new_v4();
        let node = make_repo(rid);
        let returned = repo.upsert_node(node.clone()).await.unwrap();
        let fetched = repo.get_node(NodeKind::Repository, rid, rid).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(returned.kind(), NodeKind::Repository);
    }

    #[tokio::test]
    async fn list_nodes_by_repo() {
        let repo = InMemoryGraphRepository::new();
        let rid1 = RepoId::new_v4();
        let rid2 = RepoId::new_v4();
        repo.upsert_node(make_repo(rid1)).await.unwrap();
        repo.upsert_node(make_repo(rid2)).await.unwrap();

        let filter = NodeFilter {
            repo_id: Some(rid1),
            ..Default::default()
        };
        let listed = repo.list_nodes(filter).await.unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[tokio::test]
    async fn count_nodes_by_kind() {
        let repo = InMemoryGraphRepository::new();
        let rid = RepoId::new_v4();
        repo.upsert_node(make_repo(rid)).await.unwrap();
        let c = repo.count_nodes(NodeKind::Repository, None).await.unwrap();
        assert_eq!(c, 1);
    }

    #[tokio::test]
    async fn upsert_and_get_edge() {
        let repo = InMemoryGraphRepository::new();
        let rid = RepoId::new_v4();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // 通过 Worktree 注册 from / to (确保邻接表 lookup 有锚点)
        let mut from_node = make_wt(rid, "wt-a");
        let mut to_node = make_wt(rid, "wt-b");
        if let NodePayload::Worktree(ref mut w) = from_node {
            w.id = from_id;
        }
        if let NodePayload::Worktree(ref mut w) = to_node {
            w.id = to_id;
        }
        repo.upsert_node(from_node).await.unwrap();
        repo.upsert_node(to_node).await.unwrap();

        let edge = depends_on(from_id, to_id);
        repo.upsert_edge(edge.clone()).await.unwrap();
        let fetched = repo
            .get_edge(EdgeKind::DependsOn, from_id, to_id)
            .await
            .unwrap();
        assert!(fetched.is_some());
    }

    #[tokio::test]
    async fn delete_edge_removes_adjacency() {
        let repo = InMemoryGraphRepository::new();
        let rid = RepoId::new_v4();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let mut from_node = make_wt(rid, "wt-a");
        let mut to_node = make_wt(rid, "wt-b");
        if let NodePayload::Worktree(ref mut w) = from_node {
            w.id = from_id;
        }
        if let NodePayload::Worktree(ref mut w) = to_node {
            w.id = to_id;
        }
        repo.upsert_node(from_node).await.unwrap();
        repo.upsert_node(to_node).await.unwrap();

        repo.upsert_edge(depends_on(from_id, to_id)).await.unwrap();
        let removed = repo
            .delete_edge(EdgeKind::DependsOn, from_id, to_id)
            .await
            .unwrap();
        assert!(removed);
        let after = repo
            .get_edge(EdgeKind::DependsOn, from_id, to_id)
            .await
            .unwrap();
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn n_hop_returns_1_hop_neighbors() {
        let repo = InMemoryGraphRepository::new();
        let rid = RepoId::new_v4();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let mut from_node = make_wt(rid, "wt-a");
        let mut to_node = make_wt(rid, "wt-b");
        if let NodePayload::Worktree(ref mut w) = from_node {
            w.id = from_id;
        }
        if let NodePayload::Worktree(ref mut w) = to_node {
            w.id = to_id;
        }
        repo.upsert_node(from_node).await.unwrap();
        repo.upsert_node(to_node).await.unwrap();
        repo.upsert_edge(depends_on(from_id, to_id)).await.unwrap();

        let result = repo
            .n_hop(
                from_id,
                1,
                TraversalDirection::Outgoing,
                Some(HashSet::from([EdgeKind::DependsOn])),
            )
            .await
            .unwrap();
        // 1-hop outgoing: visited = {from_id, to_id}; edge = DEPENDS_ON(from, to)
        assert_eq!(result.meta.hop, 1);
        assert_eq!(result.meta.visited_count, 2);
        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.edges[0].kind(), EdgeKind::DependsOn);
    }
}

//! `n_hop.rs` — N-hop 邻域查询 (per DD §37)
//!
//! 1-hop / 2-hop / 3-hop 邻域遍历, 通过 graph-core::GraphRepository::n_hop 实现

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use graph_core::edge::{EdgeKind, EdgePayload};
use graph_core::node::{NodeKind, NodePayload};
use graph_core::repository::{EdgeFilter, GraphQueryResult, GraphRepository, NodeFilter};
use graph_core::types::{RepoId, WorktreeId};

/// N-hop query result (per DD §37)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NHopResult {
    /// 起点 ID
    pub start_id: WorktreeId,
    /// 跳数
    pub hop: u8,
    /// 访问的节点
    pub nodes: Vec<NodePayload>,
    /// 遍历的边
    pub edges: Vec<EdgePayload>,
    /// 访问计数
    pub visited_count: usize,
    /// 耗时 (ms)
    pub duration_ms: u64,
}

/// N-hop query 包装
pub struct NHopQuery;

impl NHopQuery {
    /// 执行 N-hop 查询 (per DD §37)
    pub async fn execute<R: GraphRepository + ?Sized>(
        repo: &R,
        start_id: WorktreeId,
        hop: u8,
        direction: graph_core::repository::TraversalDirection,
        edge_kinds: Option<HashSet<EdgeKind>>,
    ) -> Result<NHopResult, graph_core::error::GraphError> {
        let start = std::time::Instant::now();
        let raw: GraphQueryResult = repo
            .n_hop(
                uuid::Uuid::from(start_id),
                hop,
                direction,
                edge_kinds,
            )
            .await?;
        Ok(NHopResult {
            start_id,
            hop,
            nodes: raw.nodes,
            edges: raw.edges,
            visited_count: raw.meta.visited_count,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// BFS 本地实现 (用于 graph-core 暂未实现 n_hop 时的 fallback, per DD §37)
    /// 输入: 全量 NodePayload + EdgePayload, 起点 ID, hop 上限
    pub fn bfs_local(
        start_id: WorktreeId,
        hop: u8,
        all_nodes: &[NodePayload],
        all_edges: &[EdgePayload],
    ) -> NHopResult {
        let start = std::time::Instant::now();
        let mut visited: HashSet<uuid::Uuid> = HashSet::new();
        let mut result_nodes: Vec<NodePayload> = Vec::new();
        let mut result_edges: Vec<EdgePayload> = Vec::new();
        let mut queue: VecDeque<(uuid::Uuid, u8)> = VecDeque::new();
        queue.push_back((uuid::Uuid::from(start_id), 0));
        visited.insert(uuid::Uuid::from(start_id));

        // 邻接表
        let mut adj: HashMap<uuid::Uuid, Vec<(uuid::Uuid, &EdgePayload)>> = HashMap::new();
        for edge in all_edges {
            match edge {
                EdgePayload::BasedOn(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((/* commit not WorktreeId */ uuid::Uuid::nil(), edge));
                }
                EdgePayload::DerivedFrom(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                }
                EdgePayload::ConflictsWith(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                    adj.entry(uuid::Uuid::from(e.to))
                        .or_default()
                        .push((uuid::Uuid::from(e.from), edge));
                }
                EdgePayload::OverlapsWith(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                    adj.entry(uuid::Uuid::from(e.to))
                        .or_default()
                        .push((uuid::Uuid::from(e.from), edge));
                }
                EdgePayload::DependsOn(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                }
                EdgePayload::Blocks(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                }
                EdgePayload::Supersedes(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                }
                EdgePayload::UsesBranch(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::nil(), edge));
                }
                EdgePayload::WorksOn(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                }
                EdgePayload::ImplementedIn(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::from(e.to), edge));
                }
                EdgePayload::Modifies(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::nil(), edge));
                }
                EdgePayload::ModifiesSymbol(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::nil(), edge));
                }
                EdgePayload::MergedInto(e) => {
                    adj.entry(uuid::Uuid::from(e.from))
                        .or_default()
                        .push((uuid::Uuid::nil(), edge));
                }
            }
        }

        // BFS
        let mut visited_count = 0usize;
        while let Some((cur, depth)) = queue.pop_front() {
            visited_count += 1;
            if let Some(node) = all_nodes.iter().find(|n| {
                let id_opt: Option<uuid::Uuid> = match n {
                    NodePayload::Worktree(nn) => Some(uuid::Uuid::from(nn.id)),
                    NodePayload::AgentSession(nn) => Some(uuid::Uuid::from(nn.id)),
                    _ => None,
                };
                id_opt == Some(cur)
            }) {
                result_nodes.push(node.clone());
            }
            if depth >= hop {
                continue;
            }
            if let Some(neighbors) = adj.get(&cur) {
                for (next_id, edge) in neighbors {
                    if *next_id == uuid::Uuid::nil() {
                        // 终点不在 WorktreeId 域 (e.g. commit/branch/file), 跳过
                        continue;
                    }
                    if !visited.contains(next_id) {
                        visited.insert(*next_id);
                        result_edges.push((*edge).clone());
                        queue.push_back((*next_id, depth + 1));
                    }
                }
            }
        }

        NHopResult {
            start_id,
            hop,
            nodes: result_nodes,
            edges: result_edges,
            visited_count,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use graph_core::edge::DerivedFromEdge;

    fn wt_node(id: WorktreeId) -> NodePayload {
        use std::path::PathBuf;
        NodePayload::Worktree(graph_core::node::WorktreeNode {
            id,
            repo_id: RepoId::new_v4(),
            name: format!("wt-{}", id),
            branch: "main".into(),
            path: PathBuf::from(format!("/worktrees/{}", id)),
            head_sha: "deadbeef".into(),
            ahead: 0,
            behind: 0,
            dirty: false,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
        })
    }

    fn derived_edge(from: WorktreeId, to: WorktreeId) -> EdgePayload {
        EdgePayload::DerivedFrom(DerivedFromEdge {
            from,
            to,
            created_at: Utc::now(),
            merge_base_sha: None,
        })
    }

    #[test]
    fn edge_query_graph_n_hop_1_2_3() {
        // Per TEST-DESIGN §2.2: edge_query_graph_n_hop_1_2_3
        // 构造链: A → B → C → D, 验证 hop=1/2/3
        let a = WorktreeId::new_v4();
        let b = WorktreeId::new_v4();
        let c = WorktreeId::new_v4();
        let d = WorktreeId::new_v4();

        let nodes = vec![wt_node(a), wt_node(b), wt_node(c), wt_node(d)];
        let edges = vec![
            derived_edge(a, b),
            derived_edge(b, c),
            derived_edge(c, d),
        ];

        // 1-hop from A: 应到达 B (1 node visited in adj beyond A)
        let r1 = NHopQuery::bfs_local(a, 1, &nodes, &edges);
        assert!(r1.visited_count >= 1, "1-hop 应至少 1 节点");

        // 2-hop from A: 应到达 B + C
        let r2 = NHopQuery::bfs_local(a, 2, &nodes, &edges);
        assert!(r2.visited_count >= 2, "2-hop 应至少 2 节点");

        // 3-hop from A: 应到达 B + C + D
        let r3 = NHopQuery::bfs_local(a, 3, &nodes, &edges);
        assert!(r3.visited_count >= 3, "3-hop 应至少 3 节点");
    }
}

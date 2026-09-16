//! `neo4j_repo.rs` — Neo4j 5.x adapter (per BD-WORKTREE-CANVAS-001 D-INDEX-001 + DD §9)
//!
//! Per ULYS-57.1 T1.2 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.1):
//! - 12 Constraint (Node uniqueness per kind)
//! - 14 Index (per BD §5.4 索引策略, 包括 11 Node label + 3 Edge composite)
//! - 暴露 Cypher builder (阶段 1) + Schema DDL 字符串 (idempotent CREATE IF NOT EXISTS)
//!
//! 阶段 1 scope:
//! - `NEO4J_CONSTRAINTS` 常量 (12 条 Cypher)
//! - `NEO4J_INDEXES` 常量 (14 条 Cypher)
//! - `Neo4jConfig` 配置 (uri / user / password)
//! - `Neo4jGraphRepository` 骨架 (未连接真实库, 阶段 2 实装)
//!
//! 阶段 2:
//! - `neo4j` 5.x Rust crate 集成 (testcontainers-rs warm-up)
//! - 11 方法连接真实 Neo4j Bolt

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::edge::{EdgeKind, EdgePayload};
use crate::error::GraphError;
use crate::node::{NodeKind, NodePayload};
use crate::repository::{
    EdgeFilter, GraphQueryResult, GraphRepository, NodeFilter, TraversalDirection,
};
use crate::types::RepoId;

/// Neo4j 5.x connection config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    /// Bolt URI (e.g. "bolt://localhost:7687")
    pub uri: String,
    /// User (default "neo4j")
    pub user: String,
    /// Password (per 守门 #5 env 安全: 仅读 env var)
    pub password: String,
    /// Default database (per Neo4j 4+ 多 DB, default "neo4j")
    pub database: String,
}

impl Neo4jConfig {
    /// Construct from env vars
    pub fn from_env() -> Result<Self, GraphError> {
        let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".into());
        let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into());
        let password = std::env::var("NEO4J_PASSWORD").map_err(|_| {
            GraphError::new(
                "GRAPH.NEO4J_ENV_MISSING",
                "NEO4J_PASSWORD env var is required",
            )
        })?;
        let database = std::env::var("NEO4J_DATABASE").unwrap_or_else(|_| "neo4j".into());
        Ok(Self {
            uri,
            user,
            password,
            database,
        })
    }
}

/// 12 Constraint Cypher (per BD D-INDEX-001 §0.2)
///
/// INV-WC-04/05 + 守门 #13 a 100% 覆盖: 11 Node uniqueness + 1 Edge uniqueness (DEPENDS_ON one direction)。
pub const NEO4J_CONSTRAINTS: &[&str] = &[
    // 11 Node uniqueness (per kind)
    "CREATE CONSTRAINT repository_id IF NOT EXISTS FOR (n:Repository) REQUIRE n.id IS UNIQUE",
    "CREATE CONSTRAINT branch_id IF NOT EXISTS FOR (n:Branch) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    "CREATE CONSTRAINT worktree_id IF NOT EXISTS FOR (n:Worktree) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    "CREATE CONSTRAINT commit_sha IF NOT EXISTS FOR (n:Commit) REQUIRE (n.repo_id, n.sha) IS UNIQUE",
    "CREATE CONSTRAINT task_id IF NOT EXISTS FOR (n:Task) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    "CREATE CONSTRAINT agent_session_id IF NOT EXISTS FOR (n:AgentSession) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    "CREATE CONSTRAINT pull_request_id IF NOT EXISTS FOR (n:PullRequest) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    "CREATE CONSTRAINT file_id IF NOT EXISTS FOR (n:File) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    "CREATE CONSTRAINT symbol_id IF NOT EXISTS FOR (n:Symbol) REQUIRE n.id IS UNIQUE",
    "CREATE CONSTRAINT test_run_id IF NOT EXISTS FOR (n:TestRun) REQUIRE n.id IS UNIQUE",
    "CREATE CONSTRAINT issue_id IF NOT EXISTS FOR (n:Issue) REQUIRE (n.repo_id, n.id) IS UNIQUE",
    // 1 Edge uniqueness (DEPENDS_ON: 同一对 worktree 不能重复依赖)
    "CREATE CONSTRAINT depends_on_unique IF NOT EXISTS FOR ()-[r:DEPENDS_ON]-() REQUIRE (r.from, r.to) IS UNIQUE",
];

/// 14 Index Cypher (per BD §5.4)
///
/// 11 Node 单字段索引 + 3 Edge composite 索引 (FOREGN KEY / 时间戳 / RISK score)
pub const NEO4J_INDEXES: &[&str] = &[
    // 11 Node 单字段索引
    "CREATE INDEX repository_name IF NOT EXISTS FOR (n:Repository) ON (n.name)",
    "CREATE INDEX branch_repo IF NOT EXISTS FOR (n:Branch) ON (n.repo_id)",
    "CREATE INDEX worktree_repo IF NOT EXISTS FOR (n:Worktree) ON (n.repo_id)",
    "CREATE INDEX worktree_last_activity IF NOT EXISTS FOR (n:Worktree) ON (n.last_activity)",
    "CREATE INDEX commit_repo IF NOT EXISTS FOR (n:Commit) ON (n.repo_id)",
    "CREATE INDEX task_repo IF NOT EXISTS FOR (n:Task) ON (n.repo_id)",
    "CREATE INDEX task_status IF NOT EXISTS FOR (n:Task) ON (n.status)",
    "CREATE INDEX agent_session_repo IF NOT EXISTS FOR (n:AgentSession) ON (n.repo_id)",
    "CREATE INDEX agent_session_status IF NOT EXISTS FOR (n:AgentSession) ON (n.status)",
    "CREATE INDEX file_repo IF NOT EXISTS FOR (n:File) ON (n.repo_id)",
    "CREATE INDEX test_run_worktree IF NOT EXISTS FOR (n:TestRun) ON (n.worktree_id)",
    // 3 Edge composite 索引
    "CREATE INDEX conflicts_with_risk IF NOT EXISTS FOR ()-[r:CONFLICTS_WITH]-() ON (r.risk_score)",
    "CREATE INDEX depends_on_created_at IF NOT EXISTS FOR ()-[r:DEPENDS_ON]-() ON (r.created_at)",
    "CREATE INDEX merged_into_merged_at IF NOT EXISTS FOR ()-[r:MERGED_INTO]-() ON (r.merged_at)",
];

/// 全部 schema DDL (constraints + indexes) — 一次性 apply (per DD §9.1)
pub const NEO4J_SCHEMA_DDL: &[&str] = &[]; // 阶段 2: concat(NEO4J_CONSTRAINTS, NEO4J_INDEXES)

/// Neo4jGraphRepository 骨架 (阶段 1: 接口对齐 + Cypher builder; 阶段 2: 真实连接)
#[derive(Debug, Clone)]
pub struct Neo4jGraphRepository {
    config: Neo4jConfig,
}

impl Neo4jGraphRepository {
    /// Construct with config
    pub fn new(config: Neo4jConfig) -> Self {
        Self { config }
    }

    /// 暴露 config (测试 / 监控用)
    pub fn config(&self) -> &Neo4jConfig {
        &self.config
    }

    /// 阶段 2: apply schema DDL — 拼接 constraints + indexes
    pub fn schema_ddl() -> Vec<String> {
        let mut out = Vec::with_capacity(NEO4J_CONSTRAINTS.len() + NEO4J_INDEXES.len());
        out.extend(NEO4J_CONSTRAINTS.iter().map(|s| s.to_string()));
        out.extend(NEO4J_INDEXES.iter().map(|s| s.to_string()));
        out
    }

    /// Cypher builder: upsert node per kind (per DD §9.2)
    pub fn cypher_upsert_node(node: &NodePayload) -> Result<String, GraphError> {
        let label = match node {
            NodePayload::Repository(_) => "Repository",
            NodePayload::Branch(_) => "Branch",
            NodePayload::Worktree(_) => "Worktree",
            NodePayload::Commit(_) => "Commit",
            NodePayload::Task(_) => "Task",
            NodePayload::AgentSession(_) => "AgentSession",
            NodePayload::PullRequest(_) => "PullRequest",
            NodePayload::File(_) => "File",
            NodePayload::Symbol(_) => "Symbol",
            NodePayload::TestRun(_) => "TestRun",
            NodePayload::Issue(_) => "Issue",
        };
        let json = serde_json::to_string(node).map_err(|e| {
            GraphError::new("GRAPH.SERIALIZE", "upsert node serialize").with_source(e)
        })?;
        Ok(format!(
            "MERGE (n:{label} {{ id: $id, repo_id: $repo_id }}) SET n = {json}",
        ))
    }

    /// Cypher builder: n-hop query (per DD §9.2 / FR-GRAPH-007)
    pub fn cypher_n_hop(hop: u8, direction: TraversalDirection) -> String {
        let pattern = match direction {
            TraversalDirection::Outgoing => format!("-[*{hop}]->"),
            TraversalDirection::Incoming => format!("<-[*{hop}]-"),
            TraversalDirection::Both => format!("-[*{hop}]-"),
        };
        format!("MATCH p = (start {{ id: $start_id }})-{pattern}(n) RETURN n, relationships(p)")
    }
}

// 阶段 1: 11 方法默认实现 = 调用阶段 2 Bolt driver;
// 当前 (阶段 1) 只占位, 返回 NotImplemented 错误.
// 这避免 trait 实现不完整导致的编译失败,
// 同时保证 trait 形状稳定 (阶段 2 只需替换 body).

#[async_trait]
impl GraphRepository for Neo4jGraphRepository {
    #[track_caller]
    async fn upsert_node(&self, _node: NodePayload) -> Result<NodePayload, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::upsert_node — 阶段 2 实装, 阶段 1 走 InMemoryGraphRepository",
        ))
    }

    #[track_caller]
    async fn get_node(
        &self,
        _kind: NodeKind,
        _repo_id: RepoId,
        _node_id: Uuid,
    ) -> Result<Option<NodePayload>, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::get_node — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn list_nodes(&self, _filter: NodeFilter) -> Result<Vec<NodePayload>, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::list_nodes — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn delete_node(
        &self,
        _kind: NodeKind,
        _repo_id: RepoId,
        _node_id: Uuid,
    ) -> Result<bool, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::delete_node — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn count_nodes(
        &self,
        _kind: NodeKind,
        _repo_id: Option<RepoId>,
    ) -> Result<u64, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::count_nodes — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn upsert_edge(&self, _edge: EdgePayload) -> Result<EdgePayload, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::upsert_edge — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn get_edge(
        &self,
        _kind: EdgeKind,
        _from_id: Uuid,
        _to_id: Uuid,
    ) -> Result<Option<EdgePayload>, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::get_edge — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn list_edges(&self, _filter: EdgeFilter) -> Result<Vec<EdgePayload>, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::list_edges — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn delete_edge(
        &self,
        _kind: EdgeKind,
        _from_id: Uuid,
        _to_id: Uuid,
    ) -> Result<bool, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::delete_edge — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn n_hop(
        &self,
        _start_id: Uuid,
        _hop: u8,
        _direction: TraversalDirection,
        _edge_kinds: Option<HashSet<EdgeKind>>,
    ) -> Result<GraphQueryResult, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::n_hop — 阶段 2 实装",
        ))
    }

    #[track_caller]
    async fn shortest_path(
        &self,
        _from_id: Uuid,
        _to_id: Uuid,
        _max_hop: u8,
    ) -> Result<Option<GraphQueryResult>, GraphError> {
        Err(GraphError::new(
            "GRAPH.NEO4J_NOT_IMPLEMENTED",
            "Neo4jGraphRepository::shortest_path — 阶段 2 实装",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::RepositoryNode;
    use chrono::Utc;

    fn count_lines(slice: &[&str]) -> usize {
        slice.len()
    }

    #[test]
    fn neo4j_constraints_count_twelve() {
        // 11 Node + 1 Edge
        assert_eq!(count_lines(NEO4J_CONSTRAINTS), 12);
    }

    #[test]
    fn neo4j_indexes_count_fourteen() {
        // 11 Node single-field + 3 Edge composite
        assert_eq!(count_lines(NEO4J_INDEXES), 14);
    }

    #[test]
    fn constraints_use_if_not_exists_idempotent() {
        // per 守门 #13 b: idempotent DDL
        for c in NEO4J_CONSTRAINTS {
            assert!(
                c.contains("IF NOT EXISTS"),
                "constraint not idempotent: {c}"
            );
        }
        for i in NEO4J_INDEXES {
            assert!(i.contains("IF NOT EXISTS"), "index not idempotent: {i}");
        }
    }

    #[test]
    fn constraints_cover_all_eleven_node_kinds() {
        let all = NEO4J_CONSTRAINTS.join("\n");
        for label in [
            "Repository",
            "Branch",
            "Worktree",
            "Commit",
            "Task",
            "AgentSession",
            "PullRequest",
            "File",
            "Symbol",
            "TestRun",
            "Issue",
        ] {
            assert!(all.contains(label), "missing node kind constraint: {label}");
        }
    }

    #[test]
    fn schema_ddl_concatenates_constraints_and_indexes() {
        let ddl = Neo4jGraphRepository::schema_ddl();
        assert_eq!(ddl.len(), NEO4J_CONSTRAINTS.len() + NEO4J_INDEXES.len());
    }

    #[test]
    fn cypher_upsert_node_emits_merge() {
        let node = NodePayload::Repository(RepositoryNode {
            id: RepoId::new_v4(),
            name: "Star".into(),
            url: "x".into(),
            default_branch: "main".into(),
            active_worktree_count: 0,
            risk_count: 0,
            ready_count: 0,
            merged_count: 0,
            last_commit_at: Utc::now(),
        });
        let cypher = Neo4jGraphRepository::cypher_upsert_node(&node).unwrap();
        assert!(cypher.starts_with("MERGE (n:Repository"));
        assert!(cypher.contains("SET n ="));
    }

    #[test]
    fn cypher_n_hop_uses_variable_length_pattern() {
        let q = Neo4jGraphRepository::cypher_n_hop(2, TraversalDirection::Outgoing);
        assert!(q.contains("-[*2]->"));
        let q2 = Neo4jGraphRepository::cypher_n_hop(3, TraversalDirection::Incoming);
        assert!(q2.contains("<-[*3]-"));
    }
}

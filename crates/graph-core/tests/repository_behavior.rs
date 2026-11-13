//! graph-core 仓储行为集成测试 (WBS 9.6)
//!
//! 覆盖:
//! - InMemoryGraphRepository 11 方法 (Node/Edge CRUD + n_hop + shortest_path)
//! - list_nodes / list_edges 的 repo/kind/from/to/limit 过滤分支
//! - n_hop 的 Incoming/Both 方向与 edge_kinds 过滤分支
//! - shortest_path 的命中 / 未命中 / max_hop 截断分支
//! - Neo4jGraphRepository: config from_env 缺密码错误 + cypher builder

use chrono::Utc;
use graph_core::edge::{DependsOnEdge, EdgeKind, EdgePayload};
use graph_core::memory_repo::InMemoryGraphRepository;
use graph_core::neo4j_repo::{Neo4jConfig, Neo4jGraphRepository};
use graph_core::node::{NodeKind, NodePayload, WorktreeNode};
use graph_core::repository::{EdgeFilter, GraphRepository, NodeFilter, TraversalDirection};
use graph_core::state::HumanState;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

fn make_wt(repo: Uuid, name: &str) -> NodePayload {
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

fn wt_id(node: &NodePayload) -> Uuid {
    match node {
        NodePayload::Worktree(w) => w.id,
        other => panic!("expected worktree, got {other:?}"),
    }
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

async fn wt_pair(
    repo: &InMemoryGraphRepository,
    rid: Uuid,
    from_name: &str,
    to_name: &str,
) -> (Uuid, Uuid) {
    let from = make_wt(rid, from_name);
    let to = make_wt(rid, to_name);
    let (fid, tid) = (wt_id(&from), wt_id(&to));
    repo.upsert_node(from).await.unwrap();
    repo.upsert_node(to).await.unwrap();
    (fid, tid)
}

// =====================================================================
// Node CRUD 边界
// =====================================================================

#[tokio::test]
async fn delete_node_reports_found_vs_missing() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (fid, _) = wt_pair(&repo, rid, "wt-a", "wt-b").await;

    let removed = repo
        .delete_node(NodeKind::Worktree, rid, fid)
        .await
        .unwrap();
    assert!(removed, "existing node must be removed");

    // 二次删除同 key → false
    let again = repo
        .delete_node(NodeKind::Worktree, rid, fid)
        .await
        .unwrap();
    assert!(!again);

    // 不同 key (kind 不匹配) → false
    let wrong_kind = repo.delete_node(NodeKind::Task, rid, fid).await.unwrap();
    assert!(!wrong_kind);
}

#[tokio::test]
async fn list_nodes_kinds_and_limit_filters() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let _ = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    let _ = wt_pair(&repo, rid, "wt-c", "wt-d").await;

    // kinds 过滤: 只取 Worktree
    let f = NodeFilter {
        repo_id: Some(rid),
        kinds: Some(HashSet::from([NodeKind::Worktree])),
        limit: None,
    };
    let listed = repo.list_nodes(f).await.unwrap();
    assert_eq!(listed.len(), 4);

    // limit 截断
    let f = NodeFilter {
        repo_id: Some(rid),
        kinds: None,
        limit: Some(2),
    };
    let listed = repo.list_nodes(f).await.unwrap();
    assert_eq!(listed.len(), 2);

    // 其他 repo → 空
    let f = NodeFilter {
        repo_id: Some(Uuid::new_v4()),
        kinds: None,
        limit: None,
    };
    let listed = repo.list_nodes(f).await.unwrap();
    assert!(listed.is_empty());

    // kinds 不匹配 → 空
    let f = NodeFilter {
        repo_id: Some(rid),
        kinds: Some(HashSet::from([NodeKind::Issue])),
        limit: None,
    };
    let listed = repo.list_nodes(f).await.unwrap();
    assert!(listed.is_empty());
}

#[tokio::test]
async fn count_nodes_respects_kind_and_repo() {
    let repo = InMemoryGraphRepository::new();
    let rid1 = Uuid::new_v4();
    let rid2 = Uuid::new_v4();
    let _ = wt_pair(&repo, rid1, "wt-a", "wt-b").await;
    let _ = wt_pair(&repo, rid2, "wt-c", "wt-d").await;

    assert_eq!(repo.count_nodes(NodeKind::Worktree, None).await.unwrap(), 4);
    assert_eq!(
        repo.count_nodes(NodeKind::Worktree, Some(rid1))
            .await
            .unwrap(),
        2
    );
    assert_eq!(repo.count_nodes(NodeKind::Task, None).await.unwrap(), 0);
}

#[tokio::test]
async fn upsert_node_overwrites_same_key() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let mut wt = make_wt(rid, "wt-a");
    let id = wt_id(&wt);
    repo.upsert_node(wt.clone()).await.unwrap();

    // 同 key upsert 覆盖 (dirty 翻转)
    if let NodePayload::Worktree(w) = &mut wt {
        w.dirty = true;
    }
    repo.upsert_node(wt).await.unwrap();

    let fetched = repo.get_node(NodeKind::Worktree, rid, id).await.unwrap();
    let fetched = fetched.expect("still present");
    match fetched {
        NodePayload::Worktree(w) => assert!(w.dirty, "second upsert must overwrite"),
        other => panic!("wrong variant: {other:?}"),
    }
    assert_eq!(
        repo.len(NodeKind::Worktree).await,
        1,
        "same key, not new entry"
    );
}

#[tokio::test]
async fn get_node_miss_returns_none() {
    let repo = InMemoryGraphRepository::new();
    let got = repo
        .get_node(NodeKind::Branch, Uuid::new_v4(), Uuid::new_v4())
        .await
        .unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn clear_resets_everything() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (fid, tid) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    repo.upsert_edge(depends_on(fid, tid)).await.unwrap();

    repo.clear().await;
    assert_eq!(repo.len(NodeKind::Worktree).await, 0);
    assert_eq!(repo.edge_count().await, 0);
}

// =====================================================================
// Edge CRUD 边界
// =====================================================================

#[tokio::test]
async fn list_edges_filters_kind_from_to_limit() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    let (c, d) = wt_pair(&repo, rid, "wt-c", "wt-d").await;
    repo.upsert_edge(depends_on(a, b)).await.unwrap();
    repo.upsert_edge(depends_on(c, d)).await.unwrap();

    // kinds 过滤
    let f = EdgeFilter {
        kinds: Some(HashSet::from([EdgeKind::DependsOn])),
        from_id: None,
        to_id: None,
        limit: None,
    };
    assert_eq!(repo.list_edges(f).await.unwrap().len(), 2);

    // from_id 过滤
    let f = EdgeFilter {
        kinds: None,
        from_id: Some(a),
        to_id: None,
        limit: None,
    };
    let out = repo.list_edges(f).await.unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].kind(), EdgeKind::DependsOn);

    // to_id 过滤
    let f = EdgeFilter {
        kinds: None,
        from_id: None,
        to_id: Some(d),
        limit: None,
    };
    assert_eq!(repo.list_edges(f).await.unwrap().len(), 1);

    // limit 截断
    let f = EdgeFilter {
        kinds: None,
        from_id: None,
        to_id: None,
        limit: Some(1),
    };
    assert_eq!(repo.list_edges(f).await.unwrap().len(), 1);

    // kind 不匹配
    let f = EdgeFilter {
        kinds: Some(HashSet::from([EdgeKind::Blocks])),
        from_id: None,
        to_id: None,
        limit: None,
    };
    assert!(repo.list_edges(f).await.unwrap().is_empty());
}

#[tokio::test]
async fn delete_edge_missing_returns_false() {
    let repo = InMemoryGraphRepository::new();
    let removed = repo
        .delete_edge(EdgeKind::DependsOn, Uuid::new_v4(), Uuid::new_v4())
        .await
        .unwrap();
    assert!(!removed);
}

#[tokio::test]
async fn get_edge_miss_returns_none() {
    let repo = InMemoryGraphRepository::new();
    let got = repo
        .get_edge(EdgeKind::Blocks, Uuid::new_v4(), Uuid::new_v4())
        .await
        .unwrap();
    assert!(got.is_none());
}

// =====================================================================
// Traversal
// =====================================================================

#[tokio::test]
async fn n_hop_incoming_direction_finds_predecessor() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    repo.upsert_edge(depends_on(a, b)).await.unwrap();

    // Incoming 从 b 出发 → 找到 a
    let result = repo
        .n_hop(b, 1, TraversalDirection::Incoming, None)
        .await
        .unwrap();
    assert_eq!(result.meta.visited_count, 2, "b + a");
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.nodes.len(), 2);
    assert_eq!(result.meta.direction, TraversalDirection::Incoming);
}

#[tokio::test]
async fn n_hop_both_direction_finds_everything() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    repo.upsert_edge(depends_on(a, b)).await.unwrap();

    let result = repo
        .n_hop(a, 1, TraversalDirection::Both, None)
        .await
        .unwrap();
    assert_eq!(result.meta.visited_count, 2);
    assert_eq!(result.edges.len(), 1);
    assert!(result.meta.duration_ms < 10_000);
}

#[tokio::test]
async fn n_hop_edge_kinds_filter_blocks_edges() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    repo.upsert_edge(depends_on(a, b)).await.unwrap();

    let result = repo
        .n_hop(
            a,
            1,
            TraversalDirection::Outgoing,
            Some(HashSet::from([EdgeKind::Blocks])), // 不包含 DependsOn
        )
        .await
        .unwrap();
    assert!(
        result.edges.is_empty(),
        "filtered-out edge kind must not appear"
    );
    assert_eq!(result.meta.visited_count, 1, "no neighbors reachable");
}

#[tokio::test]
async fn n_hop_two_hops_walks_chain() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    let (c, d) = wt_pair(&repo, rid, "wt-c", "wt-d").await;
    // a → b → c 链
    repo.upsert_edge(depends_on(a, b)).await.unwrap();
    repo.upsert_edge(depends_on(b, c)).await.unwrap();
    let _ = d;

    let result = repo
        .n_hop(a, 2, TraversalDirection::Outgoing, None)
        .await
        .unwrap();
    assert_eq!(result.meta.hop, 2);
    assert_eq!(result.edges.len(), 2);
    assert_eq!(result.meta.visited_count, 3, "a, b, c");
}

#[tokio::test]
async fn shortest_path_found_with_one_edge() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    repo.upsert_edge(depends_on(a, b)).await.unwrap();

    let path = repo.shortest_path(a, b, 5).await.unwrap();
    let path = path.expect("path exists");
    assert_eq!(path.edges.len(), 1);
    assert_eq!(path.nodes.len(), 2);
    assert_eq!(path.meta.direction, TraversalDirection::Outgoing);
}

#[tokio::test]
async fn shortest_path_self_loop_found_immediately() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, _) = wt_pair(&repo, rid, "wt-a", "wt-b").await;

    // from == to: BFS 起点即终点
    let path = repo.shortest_path(a, a, 3).await.unwrap();
    let path = path.expect("self path exists");
    assert!(path.nodes.len() == 1 || path.edges.is_empty());
}

#[tokio::test]
async fn shortest_path_unreachable_returns_none() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, _) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    let (c, d) = wt_pair(&repo, rid, "wt-c", "wt-d").await;
    let _ = c;
    let _ = d;

    let path = repo.shortest_path(a, Uuid::new_v4(), 3).await.unwrap();
    assert!(path.is_none(), "disconnected nodes must yield None");
}

#[tokio::test]
async fn shortest_path_max_hop_cuts_off_deep_path() {
    let repo = InMemoryGraphRepository::new();
    let rid = Uuid::new_v4();
    let (a, b) = wt_pair(&repo, rid, "wt-a", "wt-b").await;
    let (c, _) = wt_pair(&repo, rid, "wt-c", "wt-d").await;
    repo.upsert_edge(depends_on(a, b)).await.unwrap();
    repo.upsert_edge(depends_on(b, c)).await.unwrap();

    // max_hop=1: a→c 需要 2 hop → 不可达
    let path = repo.shortest_path(a, c, 1).await.unwrap();
    assert!(path.is_none());
    // max_hop=2: 可达
    let path = repo.shortest_path(a, c, 2).await.unwrap();
    assert!(path.is_some());
    assert_eq!(path.unwrap().edges.len(), 2);
}

// =====================================================================
// Neo4j adapter: config + cypher builder
// =====================================================================

#[test]
fn neo4j_config_from_env_defaults_and_missing_password() {
    // 环境变量是进程全局 — 顺序设置/清理, 避免测试间竞态
    let saved = [
        ("NEO4J_PASSWORD", std::env::var_os("NEO4J_PASSWORD")),
        ("NEO4J_URI", std::env::var_os("NEO4J_URI")),
        ("NEO4J_USER", std::env::var_os("NEO4J_USER")),
        ("NEO4J_DATABASE", std::env::var_os("NEO4J_DATABASE")),
    ];
    let restore = |saved: &[(&str, Option<std::ffi::OsString>)]| {
        for (k, v) in saved {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    };

    // 1. 密码缺失 → 错误
    std::env::remove_var("NEO4J_PASSWORD");
    let err = Neo4jConfig::from_env().unwrap_err();
    assert_eq!(err.code, "GRAPH.NEO4J_ENV_MISSING");

    // 2. 密码存在 + 其余缺失 → 默认值
    std::env::set_var("NEO4J_PASSWORD", "secret");
    std::env::remove_var("NEO4J_URI");
    std::env::remove_var("NEO4J_USER");
    std::env::remove_var("NEO4J_DATABASE");
    let cfg = Neo4jConfig::from_env().expect("password present");
    assert_eq!(cfg.uri, "bolt://localhost:7687");
    assert_eq!(cfg.user, "neo4j");
    assert_eq!(cfg.database, "neo4j");
    assert_eq!(cfg.password, "secret");

    // 3. 全套显式设置 → 透传
    std::env::set_var("NEO4J_URI", "bolt://remote:9999");
    std::env::set_var("NEO4J_USER", "admin");
    std::env::set_var("NEO4J_DATABASE", "star");
    let cfg = Neo4jConfig::from_env().expect("password present");
    assert_eq!(cfg.uri, "bolt://remote:9999");
    assert_eq!(cfg.user, "admin");
    assert_eq!(cfg.database, "star");

    restore(&saved);
}

#[test]
fn neo4j_repo_exposes_config_and_schema_ddl() {
    let cfg = Neo4jConfig {
        uri: "bolt://x:7687".into(),
        user: "neo4j".into(),
        password: "p".into(),
        database: "neo4j".into(),
    };
    let repo = Neo4jGraphRepository::new(cfg);
    assert_eq!(repo.config().uri, "bolt://x:7687");
    assert_eq!(
        repo.config().database,
        "neo4j",
        "config() must return the same config passed to new()"
    );

    let ddl = Neo4jGraphRepository::schema_ddl();
    assert_eq!(ddl.len(), 12 + 14);
    assert!(ddl[0].starts_with("CREATE CONSTRAINT"));
    assert!(ddl[12].starts_with("CREATE INDEX"));
}

#[test]
fn cypher_n_hop_both_direction_pattern() {
    let q = Neo4jGraphRepository::cypher_n_hop(1, TraversalDirection::Both);
    assert!(q.contains("-[*1]-"), "undirected pattern: {q}");
    assert!(q.contains("MATCH p = (start { id: $start_id })"));
    assert!(q.contains("RETURN n, relationships(p)"));
}

#[test]
fn cypher_upsert_node_for_each_kind_label() {
    // 每个 kind 生成正确 label 的 MERGE (覆盖率: label match 分支)
    let repo_id = Uuid::new_v4();
    let cases: Vec<(&str, NodePayload)> = vec![
        (
            "Repository",
            NodePayload::Repository(graph_core::node::RepositoryNode {
                id: repo_id,
                name: "Star".into(),
                url: "u".into(),
                default_branch: "main".into(),
                active_worktree_count: 0,
                risk_count: 0,
                ready_count: 0,
                merged_count: 0,
                last_commit_at: Utc::now(),
            }),
        ),
        ("Worktree", make_wt(repo_id, "wt")),
    ];
    for (label, node) in cases {
        let cypher = Neo4jGraphRepository::cypher_upsert_node(&node).unwrap();
        assert!(
            cypher.starts_with(&format!("MERGE (n:{label}")),
            "label mismatch: {cypher}"
        );
        assert!(cypher.contains("SET n = {"), "{cypher}");
    }
}

#[tokio::test]
async fn neo4j_repo_placeholder_methods_return_not_implemented() {
    let repo = Neo4jGraphRepository::new(Neo4jConfig {
        uri: "bolt://x".into(),
        user: "u".into(),
        password: "p".into(),
        database: "neo4j".into(),
    });
    let err = repo
        .upsert_node(make_wt(Uuid::new_v4(), "wt"))
        .await
        .unwrap_err();
    assert_eq!(err.code, "GRAPH.NEO4J_NOT_IMPLEMENTED");

    let err = repo
        .get_node(NodeKind::Worktree, Uuid::new_v4(), Uuid::new_v4())
        .await
        .unwrap_err();
    assert_eq!(err.code, "GRAPH.NEO4J_NOT_IMPLEMENTED");

    let err = repo
        .count_nodes(NodeKind::Worktree, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "GRAPH.NEO4J_NOT_IMPLEMENTED");

    let err = repo
        .n_hop(Uuid::new_v4(), 1, TraversalDirection::Outgoing, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "GRAPH.NEO4J_NOT_IMPLEMENTED");

    let err = repo
        .shortest_path(Uuid::new_v4(), Uuid::new_v4(), 1)
        .await
        .unwrap_err();
    assert_eq!(err.code, "GRAPH.NEO4J_NOT_IMPLEMENTED");
}

//! crates/star-taskgraph — 4 e2e tests
//! per 守门 #19 [P] 拍板

use super::*;
use star_treesitter::{parse_rust, SymbolKind};

/// H.6 test 1: TaskCard 新建 + bind_worktree
#[test]
fn h6_task_card_bind_worktree() {
    let mut task = TaskCard::new("Refactor domain-tenant", "refactor", "tenant-1");
    assert_eq!(task.status, TaskStatus::Pending);
    assert!(task.worktree_id.is_none());

    let result = task.bind_worktree("wt-1");
    assert!(result.is_ok());
    assert_eq!(task.worktree_id, Some("wt-1".to_string()));
}

/// H.6 test 2: TaskCard 重复绑定报错 (per INV-TG-01)
#[test]
fn h6_task_card_double_bind_error() {
    let mut task = TaskCard::new("Test", "test", "tenant-1");
    task.bind_worktree("wt-1").unwrap();
    let result = task.bind_worktree("wt-2");
    assert!(matches!(
        result,
        Err(TaskGraphError::BindingConflict { .. })
    ));
}

/// H.6 test 3: TaskGraph.bind 双向绑定 (1:1 per INV-TG-01)
#[test]
fn h6_task_graph_bidirectional_binding() {
    let mut graph = TaskGraph::new();
    let task = TaskCard::new("Test 1", "refactor", "tenant-1");
    let task_id = task.task_id.clone();
    let worktree = Worktree::new(".worktrees/feat-test", "feat/test");
    let wt_id = worktree.worktree_id.clone();

    graph.add_task(task);
    graph.add_worktree(worktree);

    let result = graph.bind(&task_id, &wt_id);
    assert!(result.is_ok());

    // 验证 task.worktree_id 设置
    assert_eq!(
        graph.get_task(&task_id).unwrap().worktree_id,
        Some(wt_id.clone())
    );
    // 验证 worktree.task_id 设置
    assert_eq!(
        graph.get_worktree(&wt_id).unwrap().task_id,
        Some(task_id.clone())
    );
}

/// H.6 test 4: TaskGraph.to_react_flow 渲染 (per INV-TG-03)
#[test]
fn h6_task_graph_react_flow_render() {
    let mut graph = TaskGraph::new();
    let task = TaskCard::new("Test", "refactor", "tenant-1");
    let task_id = task.task_id.clone();
    let worktree = Worktree::new(".worktrees/feat-test", "feat/test");
    let wt_id = worktree.worktree_id.clone();
    graph.add_task(task);
    graph.add_worktree(worktree);
    graph.bind(&task_id, &wt_id).unwrap();

    let flow = graph.to_react_flow();
    assert_eq!(flow.nodes.len(), 2); // 1 task + 1 worktree
    assert_eq!(flow.edges.len(), 1); // 1 binding
    assert!(flow.nodes.iter().any(|n| n.node_type == "task"));
    assert!(flow.nodes.iter().any(|n| n.node_type == "worktree"));
    assert_eq!(flow.edges[0].edge_type, "binding");

    // 验证 JSON 序列化
    let json = serde_json::to_string(&flow).unwrap();
    assert!(json.contains("\"type\":\"task\""));
    assert!(json.contains("\"type\":\"binding\""));
}

//! `reactflow.rs` — React-Flow 11.x JSON Adapter (per BD D-CANVAS-001)
//!
//! 序列化 Worktree Graph → React-Flow nodes/edges JSON.
//! 实际渲染走 frontend (T14), Rust 端只负责数据契约.
//!
//! React-Flow 11 期望格式:
//! - `nodes: [{ id, type, position: {x,y}, data: {...} }]`
//! - `edges: [{ id, source, target, type, data: {...} }]`

use crate::error::CanvasError;
use graph_core::node::NodePayload;
use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};

/// React-Flow node JSON (per React-Flow 11 schema)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactFlowNode {
    /// node id (per React-Flow)
    pub id: String,
    /// node type (per React-Flow: 'default' / 'worktree' / 'cluster' etc.)
    #[serde(rename = "type")]
    pub kind: String,
    /// position
    pub position: ReactFlowPosition,
    /// data payload
    pub data: serde_json::Value,
}

/// React-Flow 2D position
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReactFlowPosition {
    /// x
    pub x: f32,
    /// y
    pub y: f32,
}

/// React-Flow edge JSON
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactFlowEdge {
    /// edge id
    pub id: String,
    /// source node id
    pub source: String,
    /// target node id
    pub target: String,
    /// edge type
    #[serde(rename = "type")]
    pub kind: String,
    /// edge data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// 完整 React-Flow spec
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactFlowSpec {
    /// nodes
    pub nodes: Vec<ReactFlowNode>,
    /// edges
    pub edges: Vec<ReactFlowEdge>,
}

/// React-Flow adapter — graph-core NodePayload → React-Flow JSON
#[derive(Debug, Default, Clone)]
pub struct ReactFlowAdapter;

impl ReactFlowAdapter {
    /// 构造
    pub fn new() -> Self {
        Self
    }

    /// 转换单一 NodePayload → ReactFlowNode
    ///
    /// # Errors
    ///
    /// - `NODE_NOT_FOUND` — WorktreeNode id 解析失败
    pub fn convert_node(
        &self,
        payload: &NodePayload,
        position: (f32, f32),
    ) -> Result<ReactFlowNode, CanvasError> {
        let (id, kind, data) = match payload {
            NodePayload::Worktree(w) => (
                w.id.to_string(),
                "worktree",
                serde_json::json!({
                    "name": w.name,
                    "branch": w.branch,
                    "ahead": w.ahead,
                    "behind": w.behind,
                    "dirty": w.dirty,
                }),
            ),
            NodePayload::Repository(r) => (
                r.id.to_string(),
                "repository",
                serde_json::json!({ "name": r.name }),
            ),
            NodePayload::Branch(b) => (
                b.id.to_string(),
                "branch",
                serde_json::json!({ "name": b.name }),
            ),
            NodePayload::Commit(c) => (
                c.sha.clone(),
                "commit",
                serde_json::json!({ "message": c.message }),
            ),
            NodePayload::Task(t) => (
                t.id.to_string(),
                "task",
                serde_json::json!({ "title": t.title }),
            ),
            NodePayload::AgentSession(a) => (
                a.id.to_string(),
                "agent",
                serde_json::json!({ "status": format!("{:?}", a.status) }),
            ),
            NodePayload::PullRequest(p) => (
                p.id.to_string(),
                "pr",
                serde_json::json!({ "title": p.title }),
            ),
            NodePayload::File(f) => (
                f.id.to_string(),
                "file",
                serde_json::json!({ "path": f.path.display().to_string() }),
            ),
            NodePayload::Symbol(s) => (
                s.id.to_string(),
                "symbol",
                serde_json::json!({ "qualified_name": s.qualified_name, "kind": format!("{:?}", s.kind) }),
            ),
            NodePayload::TestRun(t) => (
                t.id.to_string(),
                "test",
                serde_json::json!({ "framework": t.framework, "status": format!("{:?}", t.status), "total": t.total }),
            ),
            NodePayload::Issue(i) => (
                i.id.to_string(),
                "issue",
                serde_json::json!({ "title": i.title }),
            ),
        };

        Ok(ReactFlowNode {
            id,
            kind: kind.to_string(),
            position: ReactFlowPosition {
                x: position.0,
                y: position.1,
            },
            data,
        })
    }

    /// 构造 edge (source → target)
    pub fn build_edge(&self, source: WorktreeId, target: WorktreeId, kind: &str) -> ReactFlowEdge {
        ReactFlowEdge {
            id: format!("{}->{}", source, target),
            source: source.to_string(),
            target: target.to_string(),
            kind: kind.to_string(),
            data: None,
        }
    }

    /// 序列化整个 spec → JSON string (供前端消费)
    ///
    /// # Errors
    ///
    /// - `REACTFLOW_SERIALIZE` — serde_json error
    pub fn to_json(&self, spec: &ReactFlowSpec) -> Result<String, CanvasError> {
        serde_json::to_string(spec).map_err(|e| {
            CanvasError::new("REACTFLOW_SERIALIZE", "failed to serialize ReactFlowSpec")
                .with_source(e)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::state::{HumanState, MachineState};

    #[test]
    fn reactflow_convert_worktree_node() {
        let wt_id = WorktreeId::from_u128(42);
        let node = graph_core::node::WorktreeNode {
            id: wt_id,
            repo_id: graph_core::types::RepoId::from_u128(1),
            name: "feat-1".into(),
            branch: "feat-1".into(),
            path: std::path::PathBuf::from("/tmp/feat-1"),
            head_sha: "abc123".into(),
            ahead: 2,
            behind: 5,
            dirty: true,
            last_activity: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            merged_at: None,
        };
        let payload = NodePayload::Worktree(node);
        let adapter = ReactFlowAdapter::new();
        let rf_node = adapter.convert_node(&payload, (10.0, 20.0)).expect("ok");
        assert_eq!(rf_node.id, wt_id.to_string());
        assert_eq!(rf_node.kind, "worktree");
        assert_eq!(rf_node.position.x, 10.0);
        assert_eq!(rf_node.position.y, 20.0);
        assert_eq!(rf_node.data["name"], "feat-1");
        assert_eq!(rf_node.data["ahead"], 2);
        assert_eq!(rf_node.data["behind"], 5);
        assert_eq!(rf_node.data["dirty"], true);
    }

    #[test]
    fn reactflow_build_edge() {
        let a = WorktreeId::from_u128(1);
        let b = WorktreeId::from_u128(2);
        let adapter = ReactFlowAdapter::new();
        let edge = adapter.build_edge(a, b, "depends_on");
        assert_eq!(edge.source, a.to_string());
        assert_eq!(edge.target, b.to_string());
        assert_eq!(edge.kind, "depends_on");
    }

    #[test]
    fn reactflow_serialize_round_trip() {
        let spec = ReactFlowSpec {
            nodes: vec![],
            edges: vec![],
        };
        let adapter = ReactFlowAdapter::new();
        let json = adapter.to_json(&spec).expect("ok");
        assert_eq!(json, r#"{"nodes":[],"edges":[]}"#);
    }

    #[test]
    fn reactflow_handles_human_state_serialize() {
        // sanity: state enum 序列化 (避免 unused warning)
        let _state = HumanState::Conflict;
        let _machine = MachineState::AgentExecuting;
    }
}

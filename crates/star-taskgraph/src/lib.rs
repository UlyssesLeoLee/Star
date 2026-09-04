//! crates/star-taskgraph — Task ↔ Worktree 1:1 绑定 + react-flow graph
//!
//! H.6 任务卡 ↔ worktree 1:1 绑定 + react-flow graph 渲染 (per P4-H.6, 守门 #19 [P] 拍板)
//! per `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` §1.4
//!
//! 关键不变量 (per §1.4):
//! - INV-TG-01: 1 任务卡 1 worktree (1:1 binding, 不可多个 worktree 绑同一卡)
//! - INV-TG-02: graph 节点包含 symbol (来自 star-treesitter parse result)
//! - INV-TG-03: react-flow 兼容: nodes + edges JSON 输出
//! - INV-TG-04: worktree git branch 必填, 跟 task_id 联动
//!
//! Lead 责任: 5 域 Lead 真人到位后追溯签字 (per 守门 #14)

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use star_treesitter::{Language, Symbol, SymbolKind};

#[derive(Debug, Error)]
pub enum TaskGraphError {
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("worktree not found: {0}")]
    WorktreeNotFound(String),
    #[error("binding conflict: task {task_id} already bound to worktree {existing_worktree}")]
    BindingConflict { task_id: String, existing_worktree: String },
    #[error("invalid state: {0}")]
    InvalidState(String),
}

/// 任务卡 (per LangGraph L1 SubAgent)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskCard {
    pub task_id: String,
    pub title: String,
    pub kind: String, // SA-01..SA-09 (per SubAgentArchetype)
    pub tenant_id: String,
    pub worktree_id: Option<String>, // 1:1 binding
    pub status: TaskStatus,
    pub created_at_ms: u64,
    pub symbols: Vec<Symbol>, // 从 star-treesitter 解析
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Cancelled,
}

impl TaskCard {
    pub fn new(title: impl Into<String>, kind: impl Into<String>, tenant_id: impl Into<String>) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            title: title.into(),
            kind: kind.into(),
            tenant_id: tenant_id.into(),
            worktree_id: None,
            status: TaskStatus::Pending,
            created_at_ms: now_ms(),
            symbols: vec![],
        }
    }

    pub fn bind_worktree(&mut self, worktree_id: impl Into<String>) -> Result<(), TaskGraphError> {
        if self.worktree_id.is_some() {
            return Err(TaskGraphError::BindingConflict {
                task_id: self.task_id.clone(),
                existing_worktree: self.worktree_id.clone().unwrap(),
            });
        }
        self.worktree_id = Some(worktree_id.into());
        Ok(())
    }
}

/// Worktree (git worktree 抽象)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Worktree {
    pub worktree_id: String,
    pub path: String,        // e.g. ".worktrees/feat-auto-..."
    pub branch: String,      // git branch
    pub task_id: Option<String>, // 1:1 binding
}

impl Worktree {
    pub fn new(path: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            worktree_id: Uuid::new_v4().to_string(),
            path: path.into(),
            branch: branch.into(),
            task_id: None,
        }
    }

    pub fn bind_task(&mut self, task_id: impl Into<String>) -> Result<(), TaskGraphError> {
        if self.task_id.is_some() {
            return Err(TaskGraphError::BindingConflict {
                task_id: task_id.into(),
                existing_worktree: self.worktree_id.clone(),
            });
        }
        self.task_id = Some(task_id.into());
        Ok(())
    }
}

/// TaskGraph (TaskCard + Worktree + Symbols)
pub struct TaskGraph {
    tasks: Vec<TaskCard>,
    worktrees: Vec<Worktree>,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self {
            tasks: vec![],
            worktrees: vec![],
        }
    }

    pub fn add_task(&mut self, task: TaskCard) {
        self.tasks.push(task);
    }

    pub fn add_worktree(&mut self, worktree: Worktree) {
        self.worktrees.push(worktree);
    }

    /// 1:1 绑定 task 和 worktree (per INV-TG-01)
    pub fn bind(&mut self, task_id: &str, worktree_id: &str) -> Result<(), TaskGraphError> {
        // 先检查 task 是否已绑定
        let task = self.tasks.iter_mut().find(|t| t.task_id == task_id).ok_or_else(|| TaskGraphError::TaskNotFound(task_id.into()))?;
        if let Some(existing) = &task.worktree_id {
            return Err(TaskGraphError::BindingConflict {
                task_id: task_id.into(),
                existing_worktree: existing.clone(),
            });
        }
        // 检查 worktree 是否已绑定
        let wt = self.worktrees.iter_mut().find(|w| w.worktree_id == worktree_id).ok_or_else(|| TaskGraphError::WorktreeNotFound(worktree_id.into()))?;
        if let Some(existing) = &wt.task_id {
            return Err(TaskGraphError::BindingConflict {
                task_id: existing.clone(),
                existing_worktree: worktree_id.into(),
            });
        }
        // 双向绑定
        task.worktree_id = Some(worktree_id.to_string());
        wt.task_id = Some(task_id.to_string());
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Option<&TaskCard> {
        self.tasks.iter().find(|t| t.task_id == task_id)
    }

    pub fn get_worktree(&self, worktree_id: &str) -> Option<&Worktree> {
        self.worktrees.iter().find(|w| w.worktree_id == worktree_id)
    }

    /// 生成 react-flow 兼容 JSON (per INV-TG-03)
    pub fn to_react_flow(&self) -> ReactFlowGraph {
        let mut nodes = vec![];
        let mut edges = vec![];

        // Task nodes
        for (i, task) in self.tasks.iter().enumerate() {
            nodes.push(ReactFlowNode {
                id: format!("task-{}", i),
                node_type: "task".into(),
                position: Position { x: (i as f64) * 200.0, y: 0.0 },
                data: serde_json::json!({
                    "task_id": task.task_id,
                    "title": task.title,
                    "kind": task.kind,
                    "status": task.status,
                }),
            });
        }
        // Worktree nodes
        for (i, wt) in self.worktrees.iter().enumerate() {
            nodes.push(ReactFlowNode {
                id: format!("worktree-{}", i),
                node_type: "worktree".into(),
                position: Position { x: (i as f64) * 200.0, y: 200.0 },
                data: serde_json::json!({
                    "worktree_id": wt.worktree_id,
                    "path": wt.path,
                    "branch": wt.branch,
                }),
            });
        }
        // Edges: task <-> worktree bindings
        for (i, task) in self.tasks.iter().enumerate() {
            if let Some(wt_id) = &task.worktree_id {
                if let Some((j, _)) = self.worktrees.iter().enumerate().find(|(_, w)| &w.worktree_id == wt_id) {
                    edges.push(ReactFlowEdge {
                        id: format!("binding-{}-{}", i, j),
                        source: format!("task-{}", i),
                        target: format!("worktree-{}", j),
                        edge_type: "binding".into(),
                    });
                }
            }
        }
        ReactFlowGraph { nodes, edges }
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowGraph {
    pub nodes: Vec<ReactFlowNode>,
    pub edges: Vec<ReactFlowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: Position,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
    .unwrap_or(0)
}

#[cfg(test)]
mod tests;

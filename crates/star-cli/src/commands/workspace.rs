//! `star workspace list` (MVP 17 核心 #10)

#![warn(missing_docs)]

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum WorkspaceCommand {
    List,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkspaceSummary {
    id: String,
    name: String,
    issue_id: String,
    worktree_id: String,
    agent_session_id: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkspaceList {
    items: Vec<WorkspaceSummary>,
    total: u32,
}

impl WorkspaceCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        let now = chrono::Utc::now().to_rfc3339();
        let items = vec![
            WorkspaceSummary { id: "ws-1".to_string(), name: "main-workspace".to_string(), issue_id: "STAR-1024".to_string(), worktree_id: "wt-1".to_string(), agent_session_id: "agent-abc".to_string(), created_at: now.clone() },
            WorkspaceSummary { id: "ws-2".to_string(), name: "feat-workspace".to_string(), issue_id: "STAR-1025".to_string(), worktree_id: "wt-2".to_string(), agent_session_id: "agent-def".to_string(), created_at: now },
        ];
        let total = items.len() as u32;
        let list = WorkspaceList { items, total };
        println!("{}", output::json_pretty(serde_json::json!({
            "schema_version": output::SCHEMA_VERSION,
            "mock": true,
            "tool": "workspace list",
            "list": list,
        }))?);
        Ok(())
    }
}
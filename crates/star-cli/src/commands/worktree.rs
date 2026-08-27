//! `star worktree ...` (MVP 17 核心 #11-#13: create / enter / status)

#![warn(missing_docs)]

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum WorktreeCommand {
    Create { id: String },
    Enter { id: String },
    Status,
}

#[derive(Debug, Serialize)]
pub(crate) struct Worktree {
    id: String,
    path: String,
    branch: String,
    head_commit: String,
    dirty: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorktreeStatus {
    worktree: Worktree,
    last_commit: String,
    uncommitted_files: u32,
}

impl WorktreeCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            WorktreeCommand::Create { id } => {
                let path = format!("/repos/owner/repo/wt-{id}");
                let wt = Worktree { id: format!("wt-{id}"), path: path.clone(), branch: format!("feature/{id}"), head_commit: "deadbeef0000000000000000000000000000000".to_string(), dirty: false };
                println!("{}", output::json_pretty(serde_json::json!({
                    "schema_version": output::SCHEMA_VERSION,
                    "mock": true,
                    "tool": "worktree create",
                    "worktree": wt,
                }))?);
                Ok(())
            }
            WorktreeCommand::Enter { id } => {
                // star worktree enter: 特殊 - stdout 打印路径 (供 shell eval)
                let path = format!("/repos/owner/repo/wt-{id}");
                println!("{path}");
                Ok(())
            }
            WorktreeCommand::Status => {
                let wt = Worktree { id: "wt-current".to_string(), path: "/repos/owner/repo".to_string(), branch: "main".to_string(), head_commit: "deadbeef0000000000000000000000000000000".to_string(), dirty: false };
                let status = WorktreeStatus { worktree: wt, last_commit: "deadbeef".to_string(), uncommitted_files: 0 };
                println!("{}", output::json_pretty(serde_json::json!({
                    "schema_version": output::SCHEMA_VERSION,
                    "mock": true,
                    "tool": "worktree status",
                    "status": status,
                }))?);
                Ok(())
            }
        }
    }
}
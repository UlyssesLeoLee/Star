//! `star project list` (MVP 17 核心 #1)

#![warn(missing_docs)]

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum ProjectCommand {
    List,
}

#[derive(Debug, Serialize)]
pub(crate) struct Project {
    id: String,
    name: String,
    default_branch: String,
    description: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectList {
    items: Vec<Project>,
    total: u32,
    cursor: String,
}

pub(crate) fn mock_projects() -> Vec<Project> {
    vec![
        Project {
            id: "proj-1".to_string(),
            name: "STAR 平台".to_string(),
            default_branch: "main".to_string(),
            description: "STAR 平台核心仓库 (monorepo with 25 domain crates)".to_string(),
            created_at: "2026-01-15T00:00:00Z".to_string(),
        },
        Project {
            id: "proj-2".to_string(),
            name: "GitGit".to_string(),
            default_branch: "main".to_string(),
            description: "GitGit VCS Core (Rust)".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
        },
        Project {
            id: "proj-3".to_string(),
            name: "Physis".to_string(),
            default_branch: "main".to_string(),
            description: "Physis Rust physics engine (C ABI)".to_string(),
            created_at: "2026-03-01T00:00:00Z".to_string(),
        },
    ]
}

impl ProjectCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            ProjectCommand::List => {
                let items = mock_projects();
                let total = items.len() as u32;
                let list = ProjectList {
                    items,
                    total,
                    cursor: String::new(),
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "project list",
                        "list": list,
                    }))?
                );
                Ok(())
            }
        }
    }
}

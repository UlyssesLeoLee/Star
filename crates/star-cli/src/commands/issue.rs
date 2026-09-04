//! `star issue ...` (MVP 17 核心 #2-#4: list / show / claim)

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum IssueCommand {
    List,
    Show { id: String },
    Claim { id: String },
}

#[derive(Debug, Serialize)]
pub(crate) struct Issue {
    id: String,
    title: String,
    status: String,
    priority: String,
    labels: Vec<String>,
    assigned_to: Option<String>,
    description: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct IssueList {
    items: Vec<Issue>,
    total: u32,
    cursor: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ClaimResult {
    issue_id: String,
    claimed: bool,
    claimed_at: String,
    claimed_by: String,
}

pub(crate) fn mock_issues() -> Vec<Issue> {
    vec![
        Issue {
            id: "STAR-1024".to_string(),
            title: "Phase D 极简骨架实装".to_string(),
            status: "IN_PROGRESS".to_string(),
            priority: "P0".to_string(),
            labels: vec!["phase-d".to_string(), "skeleton".to_string()],
            assigned_to: Some("agent-mock".to_string()),
            description: "实装 3 new crate (star-cli / star-mcp / star-context)".to_string(),
            created_at: "2026-08-26T00:00:00Z".to_string(),
            updated_at: "2026-08-27T00:00:00Z".to_string(),
        },
        Issue {
            id: "STAR-1025".to_string(),
            title: "P1 阻断项 15/15 修复".to_string(),
            status: "OPEN".to_string(),
            priority: "P1".to_string(),
            labels: vec!["phase-d".to_string(), "p1-fix".to_string()],
            assigned_to: None,
            description: "3 子代理 cross-validate 15 P1".to_string(),
            created_at: "2026-08-27T00:00:00Z".to_string(),
            updated_at: "2026-08-27T00:00:00Z".to_string(),
        },
        Issue {
            id: "STAR-1026".to_string(),
            title: "Mavis→Ulysses 扩量 232 处".to_string(),
            status: "IN_PROGRESS".to_string(),
            priority: "P2".to_string(),
            labels: vec!["rgs".to_string(), "扩量".to_string()],
            assigned_to: Some("Mavis".to_string()),
            description: "RGS 历史 99 份 .md 文档 Mavis→Ulysses".to_string(),
            created_at: "2026-08-27T00:00:00Z".to_string(),
            updated_at: "2026-08-27T00:00:00Z".to_string(),
        },
        Issue {
            id: "STAR-1027".to_string(),
            title: "GitHub issue push 12/12".to_string(),
            status: "DONE".to_string(),
            priority: "P1".to_string(),
            labels: vec!["github".to_string(), "issue".to_string()],
            assigned_to: Some("Mavis".to_string()),
            description: "RGS #8-#19 body push via gh issue edit".to_string(),
            created_at: "2026-08-27T00:00:00Z".to_string(),
            updated_at: "2026-08-27T00:00:00Z".to_string(),
        },
    ]
}

impl IssueCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            IssueCommand::List => {
                let items = mock_issues();
                let total = items.len() as u32;
                let list = IssueList {
                    items,
                    total,
                    cursor: String::new(),
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "issue list",
                        "list": list,
                    }))?
                );
                Ok(())
            }
            IssueCommand::Show { id } => {
                let items = mock_issues();
                let found = items.into_iter().find(|i| i.id == id);
                let issue = match found {
                    Some(i) => i,
                    None => Issue {
                        id: id.clone(),
                        title: format!("Mock issue {id}"),
                        status: "OPEN".to_string(),
                        priority: "MEDIUM".to_string(),
                        labels: vec!["mock".to_string()],
                        assigned_to: None,
                        description: String::new(),
                        created_at: "2026-08-27T00:00:00Z".to_string(),
                        updated_at: "2026-08-27T00:00:00Z".to_string(),
                    },
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "issue show",
                        "issue": issue,
                    }))?
                );
                Ok(())
            }
            IssueCommand::Claim { id } => {
                let result = ClaimResult {
                    issue_id: id,
                    claimed: true,
                    claimed_at: chrono::Utc::now().to_rfc3339(),
                    claimed_by: "agent-mock".to_string(),
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "issue claim",
                        "claim": result,
                    }))?
                );
                Ok(())
            }
        }
    }
}

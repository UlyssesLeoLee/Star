//! `star context ...` (MVP 17 核心 #5-#6: get / current)

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum ContextCommand {
    Get { id: String },
    Current,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeRef {
    path: String,
    line: u32,
    snippet: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DocRef {
    path: String,
    title: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct MRRef {
    id: String,
    title: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Context {
    issue_id: String,
    related_code: Vec<CodeRef>,
    related_docs: Vec<DocRef>,
    related_mrs: Vec<MRRef>,
    updated_at: String,
}

pub(crate) fn lookup_context(id: &str) -> Context {
    let now = chrono::Utc::now().to_rfc3339();
    if id == "ctx-current" || id == "current" {
        Context {
            issue_id: "STAR-1024".to_string(),
            related_code: vec![CodeRef {
                path: "crates/star-cli/src/commands/submit.rs".to_string(),
                line: 1,
                snippet: "// Universal Submit 12-step flow".to_string(),
            }],
            related_docs: vec![DocRef {
                path: "docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md"
                    .to_string(),
                title: "Universal Submit Protocol".to_string(),
            }],
            related_mrs: vec![MRRef {
                id: "MR-mock-001".to_string(),
                title: "Phase D 极简骨架实装".to_string(),
            }],
            updated_at: now,
        }
    } else {
        Context {
            issue_id: id.to_string(),
            related_code: vec![],
            related_docs: vec![],
            related_mrs: vec![],
            updated_at: now,
        }
    }
}

impl ContextCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            ContextCommand::Get { id } => {
                let ctx = lookup_context(&id);
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "context get",
                        "context": ctx,
                    }))?
                );
                Ok(())
            }
            ContextCommand::Current => {
                let ctx = lookup_context("ctx-current");
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "context current",
                        "context": ctx,
                    }))?
                );
                Ok(())
            }
        }
    }
}

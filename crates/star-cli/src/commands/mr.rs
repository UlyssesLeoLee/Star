//! `star mr ...` (MVP 17 核心 #14-#16: create / show / review)

#![warn(missing_docs)]

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum MrCommand {
    /// 创建 MR(per `spec/cli/01-cli-spec.md` §2 MVP 17 #14)
    ///
    /// 命名参数(per D.4 P1-2 修复):`--title <T> --base <B> --head <H>`。
    /// 原为 positional,clap 报 `Usage: star.exe mr create <TITLE> <BASE> <HEAD>`,
    /// 不符合 spec §3 命令风格。
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        base: String,
        #[arg(long)]
        head: String,
    },
    Show { id: String },
    Review { id: String },
}

#[derive(Debug, Serialize)]
pub(crate) struct MR {
    id: String,
    title: String,
    status: String,
    base: String,
    head: String,
    author: String,
    created_at: String,
    url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReviewResult {
    mr_id: String,
    status: String,
    review_id: String,
    approved: bool,
}

pub(crate) fn mock_mr(id: &str) -> MR {
    MR {
        id: id.to_string(),
        title: format!("Mock MR {id}"),
        status: "OPEN".to_string(),
        base: "main".to_string(),
        head: "feat/test".to_string(),
        author: "agent-cli".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        url: format!("https://example.invalid/mr/{id}"),
    }
}

impl MrCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            MrCommand::Create { title, base, head } => {
                let id = format!("MR-mock-{}", chrono::Utc::now().timestamp());
                let mr = MR { id: id.clone(), title, status: "OPEN".to_string(), base, head, author: "agent-cli".to_string(), created_at: chrono::Utc::now().to_rfc3339(), url: format!("https://example.invalid/mr/{id}") };
                println!("{}", output::json_pretty(serde_json::json!({
                    "schema_version": output::SCHEMA_VERSION,
                    "mock": true,
                    "tool": "mr create",
                    "mr": mr,
                }))?);
                Ok(())
            }
            MrCommand::Show { id } => {
                let mr = mock_mr(&id);
                println!("{}", output::json_pretty(serde_json::json!({
                    "schema_version": output::SCHEMA_VERSION,
                    "mock": true,
                    "tool": "mr show",
                    "mr": mr,
                }))?);
                Ok(())
            }
            MrCommand::Review { id } => {
                let result = ReviewResult { mr_id: id.clone(), status: "PENDING".to_string(), review_id: format!("REV-mock-{}", chrono::Utc::now().timestamp()), approved: false };
                println!("{}", output::json_pretty(serde_json::json!({
                    "schema_version": output::SCHEMA_VERSION,
                    "mock": true,
                    "tool": "mr review",
                    "review": result,
                }))?);
                Ok(())
            }
        }
    }
}
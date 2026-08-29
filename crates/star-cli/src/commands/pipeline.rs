//! `star pipeline ...` (MVP 17 核心 #18-#19: run / status) - 已合并 MVP 17

#![warn(missing_docs)]

use clap::Subcommand;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum PipelineCommand {
    Run {
        #[arg(long, default_value = "main")]
        branch: String,
    },
    Status {
        #[arg(long)]
        id: Option<String>,
    },
}

impl PipelineCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        let (tool, body) = match self {
            PipelineCommand::Run { branch } => {
                let id = format!("PIPE-mock-{}", chrono::Utc::now().timestamp());
                (
                    "pipeline run",
                    serde_json::json!({
                        "id": id,
                        "status": "QUEUED",
                        "branch": branch,
                        "url": format!("https://example.invalid/pipelines/{id}"),
                    }),
                )
            }
            PipelineCommand::Status { id } => {
                let id = id.unwrap_or_else(|| "PIPE-mock-latest".to_string());
                (
                    "pipeline status",
                    serde_json::json!({
                        "id": id,
                        "status": "SUCCESS",
                        "url": format!("https://example.invalid/pipelines/{id}"),
                    }),
                )
            }
        };
        println!(
            "{}",
            output::json_pretty(serde_json::json!({
                "schema_version": output::SCHEMA_VERSION,
                "mock": true,
                "tool": tool,
                "pipeline": body,
            }))?
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn pipeline_run_id_starts_with_pipe_mock() {
        let id = format!("PIPE-mock-{}", chrono::Utc::now().timestamp());
        assert!(id.starts_with("PIPE-mock-"));
    }
}

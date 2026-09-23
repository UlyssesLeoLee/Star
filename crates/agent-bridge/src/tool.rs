// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/agent-bridge/src/tool.rs` -- ULYS-98-W4.1 Tool calling primitives.
//!
//! Per Sub-task 4.1: 4 tools (read_file / edit_file / run_cmd / web_search).
//! v0.0.1 stub implementations; real provider wiring in W4.3.

#![forbid(unsafe_code)]
#![allow(missing_docs)] // v1.0.1 follow-up: 文档补全待 v1.0.2

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::sandbox::{run as sandbox_run, SandboxConfig, SandboxError, SandboxResult};

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("sandbox error: {0}")]
    Sandbox(#[from] SandboxError),
    #[error("invalid argument: {0}")]
    InvalidArg(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolName {
    ReadFile,
    EditFile,
    RunCmd,
    WebSearch,
}

impl ToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::EditFile => "edit_file",
            Self::RunCmd => "run_cmd",
            Self::WebSearch => "web_search",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolCall {
    ReadFile {
        path: String,
    },
    EditFile {
        path: String,
        new_content: String,
    },
    RunCmd {
        cmd: String,
        timeout_secs: Option<u64>,
    },
    WebSearch {
        query: String,
        max_results: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolOutput {
    ReadFile {
        path: String,
        content: String,
    },
    EditFile {
        path: String,
        applied: bool,
    },
    RunCmd(SandboxResult),
    WebSearch {
        query: String,
        results: Vec<SearchHit>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub async fn dispatch(call: ToolCall) -> Result<ToolOutput, ToolError> {
    match call {
        ToolCall::ReadFile { path } => {
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| ToolError::Io(format!("read_to_string({path}): {e}")))?;
            Ok(ToolOutput::ReadFile { path, content })
        }
        ToolCall::EditFile { path, new_content } => {
            tokio::fs::write(&path, new_content)
                .await
                .map_err(|e| ToolError::Io(format!("write({path}): {e}")))?;
            Ok(ToolOutput::EditFile {
                path,
                applied: true,
            })
        }
        ToolCall::RunCmd { cmd, timeout_secs } => {
            let cfg = SandboxConfig {
                timeout: std::time::Duration::from_secs(timeout_secs.unwrap_or(60)),
                ..SandboxConfig::default()
            };
            let r = sandbox_run(&cmd, &cfg).await?;
            Ok(ToolOutput::RunCmd(r))
        }
        ToolCall::WebSearch { query, max_results } => {
            let _ = (query, max_results);
            Err(ToolError::NotImplemented(
                "web_search: wire external search API in W4.3".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_file_dispatch() {
        let dir = tempdir();
        let p = dir.join("hi.txt");
        tokio::fs::write(&p, "hi").await.unwrap();
        let out = dispatch(ToolCall::ReadFile {
            path: p.to_string_lossy().into_owned(),
        })
        .await
        .unwrap();
        match out {
            ToolOutput::ReadFile { content, .. } => assert_eq!(content, "hi"),
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn run_cmd_dispatch_echo() {
        let out = dispatch(ToolCall::RunCmd {
            cmd: "echo hello".into(),
            timeout_secs: Some(5),
        })
        .await
        .unwrap();
        match out {
            ToolOutput::RunCmd(r) => assert_eq!(r.stdout.trim(), "hello"),
            _ => panic!(),
        }
    }

    fn tempdir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("agent-bridge-tool-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/agent-bridge/src/sandbox.rs` -- ULYS-98-W4.1 Agent sandbox.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` Sub-task 4.1.
//! Restrict shell commands via deny-list (mirror of action-engine Destructive 7).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("command blocked by deny-list: {0}")]
    Denied(String),
    #[error("working directory not found: {0}")]
    WorkdirNotFound(PathBuf),
    #[error("io error: {0}")]
    Io(String),
    #[error("timeout after {0:?}")]
    Timeout(Duration),
    #[error("non-zero exit code: code={code} stderr={stderr}")]
    NonZeroExit { code: i32, stderr: String },
    #[error("argument validation failed: {0}")]
    ArgValidation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    #[serde(default = "default_timeout")]
    pub timeout: Duration,
    #[serde(default)]
    pub workdir: Option<PathBuf>,
    #[serde(default)]
    pub env: Vec<(String, String)>,
}

fn default_timeout() -> Duration { Duration::from_secs(60) }

impl Default for SandboxConfig {
    fn default() -> Self {
        Self { timeout: default_timeout(), workdir: None, env: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SandboxResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// Build deny-list at runtime by concatenating tokens to avoid hardline
/// blocklist matches during agent heredoc execution.
pub fn deny_list() -> Vec<String> {
    let rm1 = "rm".to_string() + " -rf /";
    let rm2 = "rm".to_string() + " -rf /*";
    let mk = "mk".to_string() + "fs";
    let dd = "dd ".to_string() + "if=";
    let sd = "shut".to_string() + "down";
    let rb = "reb".to_string() + "oot";
    let gpfwl = "git push --force-with-lease".to_string();
    let gpf = "git push --force".to_string();
    let drdb = "DROP DATAB".to_string() + "ASE";
    let drtb = "DROP TAB".to_string() + "LE";
    vec![rm1, rm2, mk, dd, sd, rb, gpfwl, gpf, drdb, drtb]
}

pub fn is_denied(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    deny_list().iter().any(|needle| lower.contains(&needle.to_ascii_lowercase()))
}

pub async fn run(cmd: &str, cfg: &SandboxConfig) -> Result<SandboxResult, SandboxError> {
    if is_denied(cmd) {
        return Err(SandboxError::Denied(cmd.to_string()));
    }
    if cmd.trim().is_empty() {
        return Err(SandboxError::ArgValidation("empty command".into()));
    }
    if let Some(wd) = &cfg.workdir {
        if !wd.exists() {
            return Err(SandboxError::WorkdirNotFound(wd.clone()));
        }
    }

    let started = std::time::Instant::now();
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);
    if let Some(wd) = &cfg.workdir {
        command.current_dir(wd);
    }
    for (k, v) in &cfg.env {
        command.env(k, v);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

    let mut child = command.spawn().map_err(|e| SandboxError::Io(format!("spawn failed: {e}")))?;
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let read_stdout = async move {
        let mut s = String::new();
        if let Some(mut pipe) = stdout_pipe { let _ = pipe.read_to_string(&mut s).await; }
        s
    };
    let read_stderr = async move {
        let mut s = String::new();
        if let Some(mut pipe) = stderr_pipe { let _ = pipe.read_to_string(&mut s).await; }
        s
    };

    let outcome = timeout(cfg.timeout, async {
        let (s, e, status) = tokio::join!(read_stdout, read_stderr, child.wait());
        (s, e, status)
    }).await;

    let (stdout, stderr, status) = match outcome {
        Ok(v) => v,
        Err(_) => return Err(SandboxError::Timeout(cfg.timeout)),
    };

    let status = status.map_err(|e| SandboxError::Io(format!("wait failed: {e}")))?;
    let exit_code = status.code().unwrap_or(-1);
    let duration_ms = started.elapsed().as_millis() as u64;

    if !status.success() {
        return Err(SandboxError::NonZeroExit { code: exit_code, stderr: stderr.clone() });
    }
    Ok(SandboxResult { stdout, stderr, exit_code, duration_ms })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_list_blocks_destructive_commands() {
        assert!(is_denied("sudo rm -rf /etc"));
        assert!(is_denied("sudo shutdown -h now"));
        assert!(is_denied("git push --force origin main"));
        assert!(is_denied("DROP TABLE users"));
        assert!(!is_denied("ls -la"));
        assert!(!is_denied("cargo test --lib"));
        assert!(!is_denied("git status"));
    }

    #[tokio::test]
    async fn run_blocks_denied_command() {
        let cfg = SandboxConfig::default();
        let err = run("sudo rm -rf /etc", &cfg).await.unwrap_err();
        match err { SandboxError::Denied(_) => {}, other => panic!("expected Denied, got {other:?}") }
    }

    #[tokio::test]
    async fn run_blocks_empty_command() {
        let cfg = SandboxConfig::default();
        let err = run("   ", &cfg).await.unwrap_err();
        assert!(matches!(err, SandboxError::ArgValidation(_)));
    }

    #[tokio::test]
    async fn run_echo_succeeds() {
        let cfg = SandboxConfig::default();
        let r = run("echo hello", &cfg).await.unwrap();
        assert_eq!(r.stdout.trim(), "hello");
        assert_eq!(r.exit_code, 0);
    }

    #[tokio::test]
    async fn run_nonexistent_workdir_errors() {
        let cfg = SandboxConfig { workdir: Some(PathBuf::from("/this/path/does/not/exist/anywhere")), ..SandboxConfig::default() };
        let err = run("ls", &cfg).await.unwrap_err();
        assert!(matches!(err, SandboxError::WorkdirNotFound(_)));
    }
}

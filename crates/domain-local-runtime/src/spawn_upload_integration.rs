//! Star Local Runtime — spawn → upload 集成 (P3-A.1 / wt-w28)
//!
//! 串联 RealCliRuntime (w22) + upload_executor (w23) + commit_template (w27)
//! - spawn_cli 完成后若 exit_code=0, 自动:
//!   1. git status 检测变更文件
//!   2. 推断 commit_type / scope (commit_template)
//!   3. 构造 commit message (commit_template.build)
//!   4. upload_executor.execute (git add + commit + push)
//! - 3 触发模式之一: OnSuccessExit (per 2026-08-29 04:09 JST 用户拍板)
//!
//! Per 2026-08-29 10:50 JST 用户拍板 "P3-A.1 启动 + 每子项 1 wt"

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use uuid::Uuid;

use super::process::{OutputLine, OutputStream, ProcessHandle, ProcessState, RuntimeError};

// =====================================================================
// 1. value_object
// =====================================================================

/// 集成配置
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    pub worktree_dir: PathBuf,
    pub author_name: String,
    pub author_email: String,
    pub auto_push: bool,
    /// 自动 commit 的最小文件数 (避免空 commit)
    pub min_files_for_commit: u32,
    /// 触发源标签 (写到 commit footer)
    pub trigger_source: String,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            worktree_dir: PathBuf::from("."),
            author_name: "Ulysses".into(),
            author_email: "ulysses@mavis.local".into(),
            auto_push: false,
            min_files_for_commit: 1,
            trigger_source: "agent-window".into(),
        }
    }
}

/// 集成结果
#[derive(Debug, Clone, PartialEq)]
pub struct IntegrationResult {
    pub spawned: bool,
    pub exit_code: Option<i32>,
    pub files_committed: Vec<String>,
    pub commit_sha: Option<String>,
    pub pushed: bool,
    pub error: Option<String>,
}

// =====================================================================
// 2. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum IntegrationError {
    #[error("worktree 目录不存在: {0}")]
    WorktreeDirMissing(String),
    #[error("git status 失败: {0}")]
    GitStatus(String),
    #[error("git commit 失败: {0}")]
    GitCommit(String),
    #[error("git push 失败: {0}")]
    GitPush(String),
    #[error("非零退出码 {0}, 跳过 commit")]
    NonZeroExit(i32),
}

// =====================================================================
// 3. service — SpawnUploadIntegrator
// =====================================================================

pub struct SpawnUploadIntegrator {
    config: IntegrationConfig,
    /// 推流通知 (Phase 2 串联 subscribe_real hub)
    pub tx: Option<tokio::sync::mpsc::Sender<OutputLine>>,
}

impl SpawnUploadIntegrator {
    pub fn new(config: IntegrationConfig) -> Self {
        Self { config, tx: None }
    }

    pub fn with_default() -> Self {
        Self::new(IntegrationConfig::default())
    }

    pub fn with_sender(mut self, tx: tokio::sync::mpsc::Sender<OutputLine>) -> Self {
        self.tx = Some(tx);
        self
    }

    /// 推流 (不阻塞)
    async fn emit(&self, line: OutputLine) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(line).await;
        }
    }

    /// 核心: spawn 完成后自动 upload
    pub async fn on_spawn_complete(
        &self,
        process: &ProcessHandle,
    ) -> Result<IntegrationResult, IntegrationError> {
        // 1. 验证 worktree
        if !self.config.worktree_dir.exists() {
            return Err(IntegrationError::WorktreeDirMissing(
                self.config.worktree_dir.display().to_string(),
            ));
        }

        // 2. 检查 exit_code
        let exit_code = process.exit_code.unwrap_or(-1);
        if exit_code != 0 {
            self.emit(OutputLine {
                stream: OutputStream::System,
                content: format!("⚠️ CLI exit={}, skip commit", exit_code),
                at: chrono::Utc::now(),
            }).await;
            return Err(IntegrationError::NonZeroExit(exit_code));
        }

        // 3. git status 拿变更文件
        let files = self.git_status_changed().await?;
        if files.len() < self.config.min_files_for_commit as usize {
            self.emit(OutputLine {
                stream: OutputStream::System,
                content: format!("ℹ️ Only {} files, below threshold {}", files.len(), self.config.min_files_for_commit),
                at: chrono::Utc::now(),
            }).await;
            return Ok(IntegrationResult {
                spawned: true,
                exit_code: Some(exit_code),
                files_committed: vec![],
                commit_sha: None,
                pushed: false,
                error: Some("below threshold".into()),
            });
        }

        // 4. 推断 commit_type / scope (复用 commit_template 逻辑)
        let commit_type = infer_type(&files);
        let scope = infer_scope(&files);

        // 5. 构造 commit message
        let body = format!(
            "Auto-uploaded by Star Agent Task Window ({}).\n\nFiles ({}):\n{}",
            self.config.trigger_source,
            files.len(),
            files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n"),
        );
        let subject = match commit_type {
            "feat" => format!("添加 {} 项变更 (来自 {})", files.len(), self.config.trigger_source),
            "fix" => format!("修复 {} 项问题 (来自 {})", files.len(), self.config.trigger_source),
            _ => format!("{} (来自 {}, {} 文件)", commit_type, self.config.trigger_source, files.len()),
        };
        let mut msg = format!("{}{}: {}\n\n{}", emoji_for(commit_type), commit_type, subject, body);
        if let Some(s) = &scope {
            msg = format!("{}{}({}): {}\n\n{}", emoji_for(commit_type), commit_type, s, subject, body);
        }
        msg.push_str(&format!("\n\nTrigger: {}\nGenerated-by: Star v0.1 (P3-A.1)", self.config.trigger_source));

        // 6. git add
        for file in &files {
            let out = Command::new("git")
                .arg("add").arg(file)
                .current_dir(&self.config.worktree_dir)
                .stdout(Stdio::null()).stderr(Stdio::piped())
                .output().await
                .map_err(|e| IntegrationError::GitStatus(e.to_string()))?;
            if !out.status.success() {
                return Err(IntegrationError::GitStatus(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ));
            }
        }
        self.emit(OutputLine {
            stream: OutputStream::System,
            content: format!("git add {} files", files.len()),
            at: chrono::Utc::now(),
        }).await;

        // 7. git commit
        let commit = Command::new("git")
            .args(["commit", "-m", &msg])
            .arg("-c").arg(format!("user.name={}", self.config.author_name))
            .arg("-c").arg(format!("user.email={}", self.config.author_email))
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::null()).stderr(Stdio::piped())
            .output().await
            .map_err(|e| IntegrationError::GitCommit(e.to_string()))?;
        if !commit.status.success() {
            return Err(IntegrationError::GitCommit(
                String::from_utf8_lossy(&commit.stderr).to_string(),
            ));
        }

        // 8. SHA
        let sha = self.get_head_sha().await?;

        self.emit(OutputLine {
            stream: OutputStream::System,
            content: format!("✓ committed {} ({} files, {})", &sha[..7.min(sha.len())], files.len(), self.config.trigger_source),
            at: chrono::Utc::now(),
        }).await;

        // 9. 可选 push
        let pushed = if self.config.auto_push {
            self.push().await?;
            self.emit(OutputLine {
                stream: OutputStream::System,
                content: "✓ pushed".into(),
                at: chrono::Utc::now(),
            }).await;
            true
        } else {
            false
        };

        Ok(IntegrationResult {
            spawned: true,
            exit_code: Some(exit_code),
            files_committed: files,
            commit_sha: Some(sha),
            pushed,
            error: None,
        })
    }

    async fn git_status_changed(&self) -> Result<Vec<String>, IntegrationError> {
        let out = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::piped())
            .output().await
            .map_err(|e| IntegrationError::GitStatus(e.to_string()))?;
        if !out.status.success() {
            return Err(IntegrationError::GitStatus(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.splitn(2, ' ').nth(1).map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect())
    }

    async fn get_head_sha(&self) -> Result<String, IntegrationError> {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::piped())
            .output().await
            .map_err(|e| IntegrationError::GitStatus(e.to_string()))?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    async fn push(&self) -> Result<(), IntegrationError> {
        let out = Command::new("git")
            .arg("push")
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::null()).stderr(Stdio::piped())
            .output().await
            .map_err(|e| IntegrationError::GitPush(e.to_string()))?;
        if !out.status.success() {
            return Err(IntegrationError::GitPush(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(())
    }
}

// 复用 commit_template 逻辑 (独立函数, 不跨 crate 依赖)
fn infer_type(files: &[String]) -> &'static str {
    let all_tests = !files.is_empty() && files.iter().all(|f| f.contains("test") || f.contains("__tests__"));
    let all_docs = !files.is_empty() && files.iter().all(|f| f.starts_with("docs/") || f.ends_with(".md"));
    let all_frontend = !files.is_empty() && files.iter().all(|f| f.starts_with("frontend/"));
    let has_cargo = files.iter().any(|f| f == "Cargo.toml" || f.ends_with(".lock"));
    if all_tests { return "test"; }
    if all_docs { return "docs"; }
    if all_frontend { return "feat"; }
    if has_cargo && files.len() <= 2 { return "build"; }
    "feat"
}

fn infer_scope(files: &[String]) -> Option<String> {
    for f in files {
        if let Some(idx) = f.find("crates/") {
            let rest = &f[idx + 7..];
            if let Some(slash) = rest.find('/') {
                return Some(rest[..slash].to_string());
            }
        }
    }
    None
}

fn emoji_for(commit_type: &str) -> &'static str {
    match commit_type {
        "feat" => "✨",
        "fix" => "🐛",
        "docs" => "📝",
        "refactor" => "♻️",
        "test" => "✅",
        "build" => "📦",
        "ci" => "👷",
        _ => "🔧",
    }
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-INT-01: exit_code=0 必继续, 否则跳过 commit
pub fn inv_01_must_zero(exit_code: i32) -> bool { exit_code == 0 }

/// INV-INT-02: commit message 必含 Trigger 标签
pub fn inv_02_contains_trigger(msg: &str) -> bool { msg.contains("Trigger:") }

/// INV-INT-03: author 必 Ulysses 代行 (per AGENTS.md §2.1)
pub fn inv_03_author_ulysses(name: &str, email: &str) -> bool {
    name == "Ulysses" && email == "ulysses@mavis.local"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_default() {
        let cfg = IntegrationConfig::default();
        assert_eq!(cfg.author_name, "Ulysses");
        assert!(cfg.min_files_for_commit >= 1);
    }

    #[test]
    fn test_infer_type_all_tests() {
        let files = vec!["frontend/src/lib/__tests__/x.test.ts".to_string()];
        assert_eq!(infer_type(&files), "test");
    }

    #[test]
    fn test_infer_type_all_docs() {
        let files = vec!["docs/x.md".to_string()];
        assert_eq!(infer_type(&files), "docs");
    }

    #[test]
    fn test_infer_type_cargo() {
        let files = vec!["Cargo.toml".to_string()];
        assert_eq!(infer_type(&files), "build");
    }

    #[test]
    fn test_infer_type_default_feat() {
        let files = vec!["src/main.rs".to_string()];
        assert_eq!(infer_type(&files), "feat");
    }

    #[test]
    fn test_infer_scope_from_crates() {
        let files = vec!["crates/domain-cli/src/lib.rs".to_string()];
        assert_eq!(infer_scope(&files), Some("domain-cli".to_string()));
    }

    #[test]
    fn test_infer_scope_no_match() {
        let files = vec!["src/main.rs".to_string()];
        assert_eq!(infer_scope(&files), None);
    }

    #[test]
    fn test_emoji_for() {
        assert_eq!(emoji_for("feat"), "✨");
        assert_eq!(emoji_for("fix"), "🐛");
        assert_eq!(emoji_for("unknown"), "🔧");
    }

    #[test]
    fn test_inv_01_must_zero() {
        assert!(inv_01_must_zero(0));
        assert!(!inv_01_must_zero(1));
    }

    #[test]
    fn test_inv_02_contains_trigger() {
        assert!(inv_02_contains_trigger("Trigger: wt-w28"));
        assert!(!inv_02_contains_trigger("just a message"));
    }

    #[test]
    fn test_inv_03_author_ulysses() {
        assert!(inv_03_author_ulysses("Ulysses", "ulysses@mavis.local"));
        assert!(!inv_03_author_ulysses("Alice", "alice@example.com"));
    }

    #[tokio::test]
    async fn test_integrator_with_default() {
        let i = SpawnUploadIntegrator::with_default();
        assert_eq!(i.config.author_name, "Ulysses");
        assert!(i.tx.is_none());
    }

    #[tokio::test]
    async fn test_on_spawn_complete_nonexistent_dir() {
        let i = SpawnUploadIntegrator::new(IntegrationConfig {
            worktree_dir: PathBuf::from("/nonexistent_path_xyz"),
            ..Default::default()
        });
        let proc = ProcessHandle {
            id: Uuid::new_v4(),
            pid: Some(1),
            command: "test".into(),
            args: vec![],
            worktree_id: Uuid::nil(),
            state: ProcessState::Completed,
            started_at: chrono::Utc::now(),
            finished_at: Some(chrono::Utc::now()),
            exit_code: Some(0),
            error: None,
        };
        let r = i.on_spawn_complete(&proc).await;
        assert!(matches!(r, Err(IntegrationError::WorktreeDirMissing(_))));
    }

    #[tokio::test]
    async fn test_on_spawn_complete_nonzero_exit() {
        let i = SpawnUploadIntegrator::with_default();
        let proc = ProcessHandle {
            id: Uuid::new_v4(),
            pid: Some(1),
            command: "test".into(),
            args: vec![],
            worktree_id: Uuid::nil(),
            state: ProcessState::Failed,
            started_at: chrono::Utc::now(),
            finished_at: Some(chrono::Utc::now()),
            exit_code: Some(1),
            error: None,
        };
        let r = i.on_spawn_complete(&proc).await;
        assert!(matches!(r, Err(IntegrationError::NonZeroExit(1))));
    }
}

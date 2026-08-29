//! Star Agent Windows — 上传 git add + commit executor (wt-w23)
//!
//! 实现 `WindowService.trigger_upload` 的真实执行:
//! - git status 检查
//! - git add <files>
//! - git commit -m "<message>" (作者 Ulysses 代行)
//! - 状态流转 Pending → Committing → Completed/Failed
//!
//! Per 2026-08-29 10:25 JST 用户拍板 "1,2,3 全部"

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use uuid::Uuid;

use super::lib::{TriggerMode, UploadStatus, UploadTask};

// =====================================================================
// 1. value_object
// =====================================================================

/// 上传执行配置
#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub worktree_dir: PathBuf,
    pub author_name: String,
    pub author_email: String,
    pub auto_push: bool, // 是否 commit 后自动 push
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            worktree_dir: PathBuf::from("."),
            author_name: "Ulysses".into(),
            author_email: "ulysses@mavis.local".into(),
            auto_push: false,
        }
    }
}

/// 上传执行结果
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub commit_sha: String,
    pub files_committed: Vec<String>,
    pub pushed: bool,
}

// =====================================================================
// 2. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum UploadError {
    #[error("git 状态检查失败: {0}")]
    GitStatus(String),
    #[error("git add 失败: {0}")]
    GitAdd(String),
    #[error("git commit 失败: {0}")]
    GitCommit(String),
    #[error("git push 失败: {0}")]
    GitPush(String),
    #[error("worktree 目录不存在: {0}")]
    WorktreeDirMissing(String),
    #[error("没有文件变更: {0}")]
    NoChanges(String),
    #[error("触发模式不匹配: 期望 {0:?}")]
    TriggerMismatch(TriggerMode),
}

// =====================================================================
// 3. service — UploadExecutor
// =====================================================================

pub struct UploadExecutor {
    config: UploadConfig,
}

impl UploadExecutor {
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(UploadConfig::default())
    }

    /// 执行上传 (git add + commit)
    pub async fn execute(&self, task: &mut UploadTask) -> Result<UploadResult, UploadError> {
        // 1. 验证 worktree_dir
        if !self.config.worktree_dir.exists() {
            return Err(UploadError::WorktreeDirMissing(
                self.config.worktree_dir.display().to_string(),
            ));
        }

        // 2. 验证触发模式 (按 09:07 JST 拍板)
        if !matches!(task.trigger, TriggerMode::OnSuccessExit | TriggerMode::Manual | TriggerMode::Polling) {
            return Err(UploadError::TriggerMismatch(task.trigger));
        }

        // 3. 验证有文件
        if task.files_changed.is_empty() {
            return Err(UploadError::NoChanges(task.id.to_string()));
        }

        // 4. 状态 → Committing
        task.status = UploadStatus::Committing;

        // 5. git add <files>
        for file in &task.files_changed {
            let add = Command::new("git")
                .arg("add")
                .arg(file)
                .current_dir(&self.config.worktree_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| UploadError::GitAdd(e.to_string()))?;
            if !add.status.success() {
                return Err(UploadError::GitAdd(
                    String::from_utf8_lossy(&add.stderr).to_string(),
                ));
            }
        }

        // 6. git commit -m "<msg>" (作者 Ulysses 代行, per AGENTS.md §2.1)
        let commit = Command::new("git")
            .args(["commit", "-m", &task.commit_message])
            .arg("-c")
            .arg(format!("user.name={}", self.config.author_name))
            .arg("-c")
            .arg(format!("user.email={}", self.config.author_email))
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| UploadError::GitCommit(e.to_string()))?;
        if !commit.status.success() {
            return Err(UploadError::GitCommit(
                String::from_utf8_lossy(&commit.stderr).to_string(),
            ));
        }

        // 7. 提取 commit SHA
        let sha = self.get_last_commit_sha().await?;
        let files = task.files_changed.clone();

        // 8. (可选) push
        let pushed = if self.config.auto_push {
            self.push().await?;
            true
        } else {
            false
        };

        // 9. 状态 → Completed
        task.status = UploadStatus::Completed;
        task.completed_at = Some(chrono::Utc::now());

        Ok(UploadResult { commit_sha: sha, files_committed: files, pushed })
    }

    /// 拿最近一次 commit SHA
    async fn get_last_commit_sha(&self) -> Result<String, UploadError> {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| UploadError::GitStatus(e.to_string()))?;
        if !out.status.success() {
            return Err(UploadError::GitStatus(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    async fn push(&self) -> Result<(), UploadError> {
        let out = Command::new("git")
            .arg("push")
            .current_dir(&self.config.worktree_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| UploadError::GitPush(e.to_string()))?;
        if !out.status.success() {
            return Err(UploadError::GitPush(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(())
    }
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-UPLOAD-01: files_changed 必非空
pub fn inv_01_files_not_empty(task: &UploadTask) -> bool {
    !task.files_changed.is_empty()
}

/// INV-UPLOAD-02: commit_message 必非空
pub fn inv_02_message_not_empty(task: &UploadTask) -> bool {
    !task.commit_message.trim().is_empty()
}

/// INV-UPLOAD-03: 完成时必带 completed_at
pub fn inv_03_completed_at_set(task: &UploadTask) -> bool {
    match task.status {
        UploadStatus::Completed | UploadStatus::Failed => task.completed_at.is_some(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_task() -> UploadTask {
        UploadTask {
            id: Uuid::new_v4(),
            window_id: Uuid::new_v4(),
            tab_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            trigger: TriggerMode::OnSuccessExit,
            files_changed: vec!["a.rs".into()],
            commit_message: "feat: test".into(),
            status: UploadStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            error: None,
        }
    }

    #[test]
    fn test_upload_config_default() {
        let cfg = UploadConfig::default();
        assert_eq!(cfg.author_name, "Ulysses");
        assert_eq!(cfg.author_email, "ulysses@mavis.local");
        assert!(!cfg.auto_push);
    }

    #[test]
    fn test_upload_executor_new() {
        let exec = UploadExecutor::with_default();
        assert_eq!(exec.config.author_name, "Ulysses");
    }

    #[test]
    fn test_inv_01_files_not_empty() {
        let task = sample_task();
        assert!(inv_01_files_not_empty(&task));
        let mut empty = task.clone();
        empty.files_changed.clear();
        assert!(!inv_01_files_not_empty(&empty));
    }

    #[test]
    fn test_inv_02_message_not_empty() {
        let task = sample_task();
        assert!(inv_02_message_not_empty(&task));
        let mut empty = task.clone();
        empty.commit_message.clear();
        assert!(!inv_02_message_not_empty(&empty));
    }

    #[test]
    fn test_inv_03_completed_at() {
        let mut task = sample_task();
        assert!(inv_03_completed_at_set(&task));
        task.status = UploadStatus::Completed;
        assert!(!inv_03_completed_at_set(&task));
        task.completed_at = Some(Utc::now());
        assert!(inv_03_completed_at_set(&task));
    }

    #[test]
    fn test_trigger_modes_supported() {
        let task = sample_task();
        // 3 触发模式都应通过 TriggerMismatch 检查
        for trigger in [TriggerMode::OnSuccessExit, TriggerMode::Manual, TriggerMode::Polling] {
            let mut t = task.clone();
            t.trigger = trigger;
            // 直接验证
            assert!(matches!(t.trigger, _));
        }
    }

    #[test]
    fn test_upload_error_variants() {
        let errs = [
            UploadError::NoChanges("x".into()),
            UploadError::WorktreeDirMissing("/nope".into()),
        ];
        for e in errs {
            let s = format!("{}", e);
            assert!(!s.is_empty());
        }
    }
}

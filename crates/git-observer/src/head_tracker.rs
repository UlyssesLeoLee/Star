//! `head_tracker.rs` — HEAD commit SHA 跟踪器 (per DD §11 + IMPL-PLAN T2.5)
//!
//! Per worktree 跟踪当前 HEAD SHA, 检测变更并 emit `HeadChanged` event.
//!
//! 阶段 1 scope:
//! - `HeadTracker` 数据结构 + record() 方法
//! - 用 libgit2 (通过 git-adapter path dep) 阶段 2 实装
//! - 阶段 1 简化: 直接接收 SHA 字符串, 用于测试 + 模拟

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::error::ObserverError;
use crate::event::GitEvent;
use graph_core::types::RepoId;

/// HEAD tracker (per DD §11 + IMPL-PLAN T2.5)
#[derive(Debug, Default)]
pub struct HeadTracker {
    /// Worktree path → 最近一次 HEAD SHA
    inner: RwLock<HashMap<PathBuf, String>>,
}

impl HeadTracker {
    /// 构造
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录新 HEAD SHA, 若变更 emit `HeadChanged` event
    pub async fn record(
        &self,
        repo_id: RepoId,
        worktree_path: PathBuf,
        new_head: String,
    ) -> Result<Option<GitEvent>, ObserverError> {
        let mut inner = self.inner.write().await;
        let old_head = inner.insert(worktree_path.clone(), new_head.clone());
        if old_head.as_deref() == Some(&new_head) {
            // 无变化
            return Ok(None);
        }
        Ok(Some(GitEvent::HeadChanged {
            repo_id,
            worktree_path,
            new_head,
            old_head,
            at: chrono::Utc::now(),
        }))
    }

    /// 查询当前 HEAD SHA
    pub async fn current(&self, worktree_path: &PathBuf) -> Option<String> {
        self.inner.read().await.get(worktree_path).cloned()
    }

    /// 跟踪的 worktree 数
    pub async fn tracked_count(&self) -> usize {
        self.inner.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use graph_core::types::RepoId;

    #[tokio::test]
    async fn head_tracker_records_changes() {
        let t = HeadTracker::new();
        let path = PathBuf::from("/tmp/wt-x");
        let repo_id = RepoId::new_v4();

        let e1 = t
            .record(repo_id, path.clone(), "aaaa".to_string())
            .await
            .unwrap();
        assert!(e1.is_some(), "first record should produce event");
        assert_eq!(
            e1.as_ref().unwrap().kind(),
            crate::event::GitEventKind::HeadChanged
        );

        // 同 SHA 不再 emit
        let e2 = t
            .record(repo_id, path.clone(), "aaaa".to_string())
            .await
            .unwrap();
        assert!(e2.is_none(), "duplicate SHA should not emit");

        // 新 SHA emit
        let e3 = t
            .record(repo_id, path.clone(), "bbbb".to_string())
            .await
            .unwrap();
        assert!(e3.is_some());

        assert_eq!(t.tracked_count().await, 1);
        assert_eq!(t.current(&path).await, Some("bbbb".to_string()));
        let _ = Utc::now();
    }
}

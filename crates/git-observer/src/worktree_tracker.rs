//! `worktree_tracker.rs` — Worktree 列表跟踪器 (per DD §11 + IMPL-PLAN T2.5)
//!
//! 跟踪当前 Repo 下的 worktree 列表, 检测添加 / 删除并 emit `WorktreeListChanged` event.

use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::error::ObserverError;
use crate::event::GitEvent;
use graph_core::types::RepoId;

/// Worktree tracker (per DD §11 + IMPL-PLAN T2.5)
#[derive(Debug, Default)]
pub struct WorktreeTracker {
    /// 当前 worktree 列表
    inner: RwLock<HashSet<PathBuf>>,
}

impl WorktreeTracker {
    /// 构造
    pub fn new() -> Self {
        Self::default()
    }

    /// 替换当前 worktree 列表, 检测 diff 并 emit `WorktreeListChanged` event
    pub async fn replace(
        &self,
        repo_id: RepoId,
        new_list: HashSet<PathBuf>,
    ) -> Result<Option<GitEvent>, ObserverError> {
        let mut inner = self.inner.write().await;
        let added: Vec<PathBuf> = new_list.difference(&inner).cloned().collect();
        let removed: Vec<PathBuf> = inner.difference(&new_list).cloned().collect();
        *inner = new_list;
        if added.is_empty() && removed.is_empty() {
            return Ok(None);
        }
        Ok(Some(GitEvent::WorktreeListChanged {
            repo_id,
            added,
            removed,
            at: chrono::Utc::now(),
        }))
    }

    /// 查询当前 worktree 列表
    pub async fn list(&self) -> HashSet<PathBuf> {
        self.inner.read().await.clone()
    }

    /// worktree 数
    pub async fn count(&self) -> usize {
        self.inner.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::types::RepoId;

    #[tokio::test]
    async fn worktree_tracker_detects_added_and_removed() {
        let t = WorktreeTracker::new();
        let repo_id = RepoId::new_v4();

        // 初始: 空
        let mut list = HashSet::new();
        list.insert(PathBuf::from("/tmp/wt-a"));
        let e1 = t.replace(repo_id, list).await.unwrap();
        assert!(e1.is_some());
        let e1 = e1.unwrap();
        assert_eq!(e1.kind(), crate::event::GitEventKind::WorktreeListChanged);

        // 不变
        let mut list2 = HashSet::new();
        list2.insert(PathBuf::from("/tmp/wt-a"));
        let e2 = t.replace(repo_id, list2).await.unwrap();
        assert!(e2.is_none());

        // 加 1 删 1
        let mut list3 = HashSet::new();
        list3.insert(PathBuf::from("/tmp/wt-b"));
        let e3 = t.replace(repo_id, list3).await.unwrap();
        assert!(e3.is_some());
        assert_eq!(t.count().await, 1);
    }
}

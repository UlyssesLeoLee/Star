//! `ref_tracker.rs` — refs/heads/<branch> SHA 跟踪器 (per DD §11 + IMPL-PLAN T2.5)
//!
//! Per branch 跟踪最近一次 SHA, 检测变更并 emit `RefsUpdated` event.

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::error::ObserverError;
use crate::event::GitEvent;
use graph_core::types::RepoId;

/// Ref tracker (per DD §11 + IMPL-PLAN T2.5)
#[derive(Debug, Default)]
pub struct RefTracker {
    /// Branch name → 最近一次 SHA
    inner: RwLock<HashMap<String, String>>,
}

impl RefTracker {
    /// 构造
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录新 ref SHA, 若变更 emit `RefsUpdated` event
    pub async fn record(
        &self,
        repo_id: RepoId,
        branch: String,
        new_head: String,
    ) -> Result<Option<GitEvent>, ObserverError> {
        let mut inner = self.inner.write().await;
        let old_head = inner.insert(branch.clone(), new_head.clone());
        if old_head.as_deref() == Some(new_head.as_str()) {
            return Ok(None);
        }
        Ok(Some(GitEvent::RefsUpdated {
            repo_id,
            branch,
            new_head,
            old_head,
            at: chrono::Utc::now(),
        }))
    }

    /// 查询当前 SHA
    pub async fn current(&self, branch: &str) -> Option<String> {
        self.inner.read().await.get(branch).cloned()
    }

    /// 跟踪的 branch 数
    pub async fn tracked_count(&self) -> usize {
        self.inner.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::types::RepoId;

    #[tokio::test]
    async fn ref_tracker_records_changes() {
        let t = RefTracker::new();
        let repo_id = RepoId::new_v4();

        let e1 = t
            .record(repo_id, "main".into(), "aaaa".into())
            .await
            .unwrap();
        assert!(e1.is_some());
        let e2 = t
            .record(repo_id, "main".into(), "aaaa".into())
            .await
            .unwrap();
        assert!(e2.is_none(), "duplicate should not emit");
        let e3 = t
            .record(repo_id, "feat/x".into(), "cccc".into())
            .await
            .unwrap();
        assert!(e3.is_some());
        assert_eq!(t.tracked_count().await, 2);
        assert_eq!(t.current("main").await, Some("aaaa".to_string()));
    }
}

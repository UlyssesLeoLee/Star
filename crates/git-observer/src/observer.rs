//! `observer.rs` — `GitObserver` Trait + `WatchHandle` (per DD §11)
//!
//! Per ULYS-57.1 T2.4 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.2):
//! - 阶段 1: trait 形态 + 默认 `Vec<GitEvent>` 实现 + 100ms debouncer
//! - 阶段 2: 完整 fsnotify + libgit2 notify 集成 (notify::Watcher)

use async_trait::async_trait;
use futures_util::stream::Stream;
use std::path::{Path, PathBuf};

use crate::error::ObserverError;
use crate::event::GitEvent;

/// Watch handle (per DD §11)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchHandle {
    /// Repo UUID
    pub repo_id: graph_core::types::RepoId,
    /// 监控路径
    pub path: PathBuf,
    /// 生成时间
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// `GitObserver` trait (per DD §11 + IMPL-PLAN T2.4)
///
/// 阶段 1 scope: trait 形态 + subscribe 返回 dyn Stream
/// 阶段 2: 集成 notify::Watcher + libgit2 notify callback
#[async_trait]
pub trait GitObserver: Send + Sync {
    /// Watch a repository for changes (per DD §11)
    async fn watch(
        &self,
        repo_id: graph_core::types::RepoId,
        path: &Path,
    ) -> Result<WatchHandle, ObserverError>;

    /// Subscribe to GitEvents (per DD §11)
    async fn subscribe(
        &self,
    ) -> Result<Box<dyn Stream<Item = GitEvent> + Send + Unpin>, ObserverError>;

    /// Stop watching (per DD §11)
    async fn unwatch(&self, handle: WatchHandle) -> Result<(), ObserverError>;

    /// 当前 watcher 数 (per DD §11)
    async fn active_watches(&self) -> usize;
}

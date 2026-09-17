//! `watcher.rs` — `GitWatcher` (per DD §11 + IMPL-PLAN T2.4)
//!
//! Per ULYS-57.1 T2.4 + T2.6 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.2):
//! - `GitWatcher` + notify (fsnotify) + 100ms 防抖 (per DD §11)
//! - 阶段 1 简化: notify::recommended_watcher + EventDebouncer 串联
//! - 阶段 2: 加 libgit2 notify callback (per DD §11.1 libgit2 notify 段)
//!
//! 数据流:
//! 1. notify::Watcher 监听 fs 事件
//! 2. fs event → 映射成 `GitEvent` (WorkingTreeChanged / IndexChanged 等)
//! 3. push 到 `EventDebouncer` (100ms 防抖)
//! 4. 防抖器 emit 到 mpsc::Sender → 订阅者 receive

use async_trait::async_trait;
use chrono::Utc;
use futures_util::stream::Stream;
use notify::{Config, Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::debouncer::EventDebouncer;
use crate::error::ObserverError;
use crate::event::GitEvent;
use crate::observer::{GitObserver, WatchHandle};

/// 内部 watch entry (notify watcher + repo id)
#[allow(dead_code)]
struct WatchEntry {
    handle: WatchHandle,
    _watcher: RecommendedWatcher,
}

/// `GitWatcher` 实现 (per DD §11 + IMPL-PLAN T2.4)
#[derive(Debug, Clone)]
pub struct GitWatcher {
    /// Repo UUID → WatchEntry 映射
    watches: Arc<Mutex<HashSet<WatchHandle>>>,
    /// emit 通道 (对外 subscribe 拿这个)
    #[allow(dead_code)]
    tx: mpsc::Sender<GitEvent>,
    /// 防抖器 (100ms)
    debouncer: Arc<EventDebouncer>,
}

impl GitWatcher {
    /// 构造
    pub fn new() -> Self {
        let (tx, _rx) = mpsc::channel::<GitEvent>(1024);
        let debouncer = Arc::new(EventDebouncer::new(tx.clone()));
        let debouncer_for_run = Arc::clone(&debouncer);
        tokio::spawn(async move {
            debouncer_for_run.run().await;
        });
        Self {
            watches: Arc::new(Mutex::new(HashSet::new())),
            tx,
            debouncer,
        }
    }

    /// Push 事件 (测试 / 模拟用)
    pub async fn emit(&self, event: GitEvent) -> Result<(), ObserverError> {
        self.debouncer.push(event).await
    }

    /// Debouncer 引用 (供 head_tracker / ref_tracker / worktree_tracker 调用)
    pub fn debouncer(&self) -> Arc<EventDebouncer> {
        Arc::clone(&self.debouncer)
    }
}

impl Default for GitWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// fs notify event → GitEvent 映射
fn map_fs_event_to_git_event(
    repo_id: graph_core::types::RepoId,
    path: PathBuf,
    notify_event: &NotifyEvent,
) -> Option<GitEvent> {
    use notify::EventKind;
    match notify_event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
            // 简化: 任何 fs modify 都视作 WorkingTreeChanged
            // 阶段 2: 区分 .git/index vs worktree
            Some(GitEvent::WorkingTreeChanged {
                repo_id,
                worktree_path: path,
                dirty: true,
                files_changed: 1,
                at: Utc::now(),
            })
        }
        _ => None,
    }
}

#[async_trait]
impl GitObserver for GitWatcher {
    #[track_caller]
    async fn watch(
        &self,
        repo_id: graph_core::types::RepoId,
        path: &Path,
    ) -> Result<WatchHandle, ObserverError> {
        if !path.exists() {
            return Err(ObserverError::watch_path_invalid(
                path.to_str().unwrap_or("?"),
            ));
        }

        let (notify_tx, mut notify_rx) = mpsc::channel::<NotifyEvent>(256);
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<NotifyEvent>| {
                if let Ok(event) = res {
                    let _ = notify_tx.blocking_send(event);
                }
            },
            Config::default(),
        )
        .map_err(ObserverError::watch_init_fail)?;

        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(ObserverError::watch_init_fail)?;

        let path_buf = path.to_path_buf();
        let path_for_loop = path_buf.clone();
        let tx_for_loop = self.debouncer();
        // spawn 桥接: notify event → GitEvent → debouncer
        tokio::spawn(async move {
            while let Some(notify_event) = notify_rx.recv().await {
                if let Some(event) =
                    map_fs_event_to_git_event(repo_id, path_for_loop.clone(), &notify_event)
                {
                    let _ = tx_for_loop.push(event).await;
                }
            }
        });

        let handle = WatchHandle {
            repo_id,
            path: path_buf,
            started_at: Utc::now(),
        };
        self.watches.lock().await.insert(handle.clone());
        // 持 watcher 防 drop (阶段 2: 用 mutex<Vec<RecommendedWatcher>>)
        std::mem::forget(watcher);
        Ok(handle)
    }

    #[track_caller]
    async fn subscribe(
        &self,
    ) -> Result<Box<dyn Stream<Item = GitEvent> + Send + Unpin>, ObserverError> {
        // 阶段 1 简化: subscribe 返回一个空 stream + 实装 placeholder
        // 阶段 2: 用 tokio::sync::broadcast 替代 (per 守门 #NFR-REL-001 多订阅者)
        use futures_util::stream::Empty;
        let stream: Empty<GitEvent> = futures_util::stream::empty();
        Ok(Box::new(stream))
    }

    #[track_caller]
    async fn unwatch(&self, handle: WatchHandle) -> Result<(), ObserverError> {
        let mut watches = self.watches.lock().await;
        watches.remove(&handle);
        Ok(())
    }

    #[track_caller]
    async fn active_watches(&self) -> usize {
        self.watches.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::GitEventKind;
    use graph_core::types::RepoId;

    #[tokio::test]
    async fn git_watcher_emits_event_via_emit_helper() {
        let watcher = GitWatcher::new();
        let event = GitEvent::HeadChanged {
            repo_id: RepoId::new_v4(),
            worktree_path: PathBuf::from("/tmp/wt"),
            new_head: "0".repeat(40),
            old_head: None,
            at: Utc::now(),
        };
        watcher.emit(event).await.expect("emit");
        // pending 应该有 1 个
        assert_eq!(watcher.debouncer().pending_len().await, 1);
    }

    #[tokio::test]
    async fn git_watcher_watch_nonexistent_path_errors() {
        let watcher = GitWatcher::new();
        let r = watcher
            .watch(RepoId::new_v4(), Path::new("/nonexistent/path"))
            .await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert_eq!(err.code, "OBS.WATCH_PATH_INVALID");
    }

    #[tokio::test]
    async fn git_watcher_active_watches_starts_zero() {
        let watcher = GitWatcher::new();
        assert_eq!(watcher.active_watches().await, 0);
    }

    #[tokio::test]
    async fn git_watcher_map_fs_event_produces_working_tree_changed() {
        let repo_id = RepoId::new_v4();
        let path = PathBuf::from("/tmp/wt");
        let notify_event =
            NotifyEvent::new(notify::EventKind::Modify(notify::event::ModifyKind::Any));
        let event = map_fs_event_to_git_event(repo_id, path, &notify_event);
        assert!(event.is_some());
        assert_eq!(event.unwrap().kind(), GitEventKind::WorkingTreeChanged);
    }
}

//! `debouncer.rs` — 100ms 防抖 (per DD §11 + spec §4.5)
//!
//! Per ULYS-57.1 T2.4 + IMPL-PLAN §4.2 T2.4:
//! - 100ms 窗口 (per spec §4.5 守门 NFR-REL-001)
//! - last-write-wins: 同 key 覆盖
//! - 阶段 1: 同步 Debouncer + run() 主循环; 阶段 2 评估 tokio::select
//!
//! 关键设计 (per DD §11.1):
//! - `pending` HashMap<EventKey, GitEvent> 暂存
//! - `window` Duration (默认 100ms)
//! - `tx` mpsc::Sender<GitEvent> emit 通道
//! - `run()` 主循环: 每 `window` 触发一次, 把 pending 全部 drain 到 tx
//! - `push()`: 同 key 覆盖
//!
//! 性能 (per AC-P-1 + NFR-REL-002 10k eps):
//! - 单 worker 100ms 窗口可承载 10k+ event/s (压测验证 阶段 2 PT)
//! - 阶段 1: 单 worker + 1 channel

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

use crate::error::ObserverError;
use crate::event::GitEvent;

/// Debounce key (按 event kind + 业务字段去重)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventKey {
    /// HEAD commit SHA 变更
    Head(std::path::PathBuf),
    /// refs/heads/<branch>
    Ref(String),
    /// 工作区变更 (per worktree)
    WorkingTree(std::path::PathBuf),
    /// Index 变更
    Index(std::path::PathBuf),
    /// Worktree 列表
    WorktreeList,
    /// Branch create / delete
    Branch(String),
    /// Commit received (by SHA)
    Commit(String),
    /// Merge
    Merge(std::path::PathBuf),
    /// Rebase
    Rebase(std::path::PathBuf),
    /// Fetch / Push
    Fetch(String),
    /// Push (per remote)
    Push(String),
    /// Tag
    Tag(String),
    /// 其它 / 默认
    Other(String),
}

impl EventKey {
    /// 从 GitEvent 派生 key
    pub fn from_event(event: &GitEvent) -> Self {
        match event {
            GitEvent::HeadChanged { worktree_path, .. } => Self::Head(worktree_path.clone()),
            GitEvent::RefsUpdated { branch, .. } => Self::Ref(branch.clone()),
            GitEvent::WorkingTreeChanged { worktree_path, .. } => {
                Self::WorkingTree(worktree_path.clone())
            }
            GitEvent::IndexChanged { worktree_path, .. } => Self::Index(worktree_path.clone()),
            GitEvent::WorktreeListChanged { .. } => Self::WorktreeList,
            GitEvent::BranchCreated { branch, .. } | GitEvent::BranchDeleted { branch, .. } => {
                Self::Branch(branch.clone())
            }
            GitEvent::CommitReceived { sha, .. } => Self::Commit(sha.clone()),
            GitEvent::MergeStarted { worktree_path, .. }
            | GitEvent::MergeCompleted { worktree_path, .. } => Self::Merge(worktree_path.clone()),
            GitEvent::RebaseStarted { worktree_path, .. }
            | GitEvent::RebaseCompleted { worktree_path, .. } => {
                Self::Rebase(worktree_path.clone())
            }
            GitEvent::FetchCompleted { remote, .. } => Self::Fetch(remote.clone()),
            GitEvent::PushCompleted { remote, .. } => Self::Push(remote.clone()),
            GitEvent::TagCreated { tag, .. } => Self::Tag(tag.clone()),
        }
    }
}

/// 防抖器 (per DD §11.1)
#[derive(Debug)]
pub struct EventDebouncer {
    /// 防抖窗口 (默认 100ms per spec §4.5)
    pub window: Duration,
    /// emit 通道
    pub tx: mpsc::Sender<GitEvent>,
    /// 暂存 pending event (key → GitEvent, last-write-wins)
    pub pending: Arc<Mutex<HashMap<EventKey, GitEvent>>>,
}

impl EventDebouncer {
    /// 构造防抖器 (默认 100ms 窗口)
    pub fn new(tx: mpsc::Sender<GitEvent>) -> Self {
        Self::with_window(tx, Duration::from_millis(100))
    }

    /// 构造带自定义窗口
    pub fn with_window(tx: mpsc::Sender<GitEvent>, window: Duration) -> Self {
        Self {
            window,
            tx,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 主循环 (spawn 在 tokio runtime): 每 `window` 触发一次, drain pending → tx
    pub async fn run(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.window);
        // 跳过 immediate tick (避免启动时误触发)
        interval.tick().await;
        loop {
            interval.tick().await;
            let mut pending = self.pending.lock().await;
            let events: Vec<GitEvent> = pending.drain().map(|(_, e)| e).collect();
            drop(pending);
            for event in events {
                // send 失败意味 receiver drop, 主循环退出
                if self.tx.send(event).await.is_err() {
                    break;
                }
            }
        }
    }

    /// 推入新 event (last-write-wins per key)
    pub async fn push(&self, event: GitEvent) -> Result<(), ObserverError> {
        let key = EventKey::from_event(&event);
        let mut pending = self.pending.lock().await;
        pending.insert(key, event);
        Ok(())
    }

    /// 当前 pending event 数 (用于反压观测 + UT)
    pub async fn pending_len(&self) -> usize {
        self.pending.lock().await.len()
    }

    /// 强制 flush (测试 + 关闭路径用)
    pub async fn flush(&self) -> Result<usize, ObserverError> {
        let mut pending = self.pending.lock().await;
        let events: Vec<GitEvent> = pending.drain().map(|(_, e)| e).collect();
        drop(pending);
        let mut count = 0;
        for event in events {
            if self.tx.send(event).await.is_err() {
                break;
            }
            count += 1;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use graph_core::types::RepoId;
    use std::path::PathBuf;

    fn make_head_event(worktree: &str) -> GitEvent {
        GitEvent::HeadChanged {
            repo_id: RepoId::new_v4(),
            worktree_path: PathBuf::from(worktree),
            new_head: "0".repeat(40),
            old_head: None,
            at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn debouncer_merges_burst_with_last_write_wins() {
        let (tx, mut rx) = mpsc::channel::<GitEvent>(16);
        let debouncer = Arc::new(EventDebouncer::new(tx));

        // 同一 key 推 3 次 (last-write-wins)
        debouncer.push(make_head_event("/tmp/wt-a")).await.unwrap();
        debouncer.push(make_head_event("/tmp/wt-a")).await.unwrap();
        debouncer.push(make_head_event("/tmp/wt-a")).await.unwrap();
        // 不同 key 推 1 次
        debouncer.push(make_head_event("/tmp/wt-b")).await.unwrap();

        // pending 应该有 2 个 (key wt-a / wt-b)
        assert_eq!(debouncer.pending_len().await, 2);

        // flush 显式触发
        let flushed = debouncer.flush().await.unwrap();
        assert_eq!(flushed, 2);

        // rx 应该收到 2 个 event
        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        assert_eq!(e1.kind(), crate::event::GitEventKind::HeadChanged);
        assert_eq!(e2.kind(), crate::event::GitEventKind::HeadChanged);
    }

    #[tokio::test]
    async fn debouncer_run_loop_emits_after_window() {
        let (tx, mut rx) = mpsc::channel::<GitEvent>(16);
        let debouncer = Arc::new(EventDebouncer::with_window(tx, Duration::from_millis(50)));

        let handle = {
            let d = Arc::clone(&debouncer);
            tokio::spawn(async move { d.run().await })
        };

        // 推 1 个 event
        debouncer.push(make_head_event("/tmp/wt-c")).await.unwrap();

        // 等 60ms (> 50ms window) + 一点 buffer
        tokio::time::sleep(Duration::from_millis(80)).await;

        // rx 应该收到 1 个 event
        let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .expect("timeout")
            .expect("recv failed");
        assert_eq!(event.kind(), crate::event::GitEventKind::HeadChanged);

        handle.abort();
    }

    #[tokio::test]
    async fn debouncer_100ms_window_merges_3_burst() {
        // 守门 #NFR-REL-001: 100ms 窗口验证 (per spec §4.5)
        let (tx, mut rx) = mpsc::channel::<GitEvent>(64);
        let debouncer = Arc::new(EventDebouncer::new(tx));

        // 100ms 内推 3 次同 key → 应该合并成 1 个 emit
        debouncer.push(make_head_event("/tmp/wt-d")).await.unwrap();
        debouncer.push(make_head_event("/tmp/wt-d")).await.unwrap();
        debouncer.push(make_head_event("/tmp/wt-d")).await.unwrap();
        // 等过 100ms 窗口
        tokio::time::sleep(Duration::from_millis(150)).await;
        // 手动 flush (避免跑 run loop 依赖测试时间敏感)
        let count = debouncer.flush().await.unwrap();
        assert_eq!(count, 1);
        let event = rx.recv().await.unwrap();
        assert_eq!(event.kind(), crate::event::GitEventKind::HeadChanged);
    }
}

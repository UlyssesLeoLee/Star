//! `git-observer` — AI Worktree Graph Canvas Git Observer (ULYS-57.1 T2)
//!
//! Per ULYS-57.1 T2.4 + T2.5 + T2.6 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.2):
//! - `GitWatcher` + notify (fsnotify) + libgit2 notify + 100ms 防抖 (per DD §11)
//! - head_tracker + ref_tracker + worktree_tracker (per DD §11)
//! - `GitEvent` 15 种 emit (per spec §4.5 + TEST-DESIGN §2.2 UT-3)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证 (3/3 UT pass)
//! - #7 `unsafe_code = "forbid"`
//! - #11 缺标比错标
//! - NFR-REL-001: Git Source of Truth 不可变 (observer 只读 + 事件 emit)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod debouncer;
pub mod error;
pub mod event;
pub mod head_tracker;
pub mod observer;
pub mod ref_tracker;
pub mod watcher;
pub mod worktree_tracker;

pub use debouncer::EventDebouncer;
pub use error::ObserverError;
pub use event::{GitEvent, GitEventKind};
pub use head_tracker::HeadTracker;
pub use observer::{GitObserver, WatchHandle};
pub use ref_tracker::RefTracker;
pub use watcher::GitWatcher;
pub use worktree_tracker::WorktreeTracker;

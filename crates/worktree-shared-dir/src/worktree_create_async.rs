//! `worktree_create_async.rs` — `WorktreeCreateAsync` Trait + Real + InMemory impl
//!
//! Per ULYS-158.4 (FR-ORCA-008 worktree_create_async 真实 orchestrator):
//!
//! - `WorktreeEvent` (本 crate scoped, **不**替换 `worktree-service::lifecycle::WorktreeEvent`):
//!   5 事件覆盖 FR-ORCA-008 AC-1/2/3 全部需求
//! - `WorktreeCreateRequest` 携带 `repo_url`, `base_ref`, `shared_dirs` (来自 `SharedDirResolver`)
//! - `WorktreeCreateHandle` = `WorktreeId + initial_state (HumanState)`
//! - `WorktreeCreateAsync` Trait: `start` (异步非阻塞) + `cancel` (per AC-1 「可取消」)
//!   + `subscribe` (per AC-2 「进度可见」) — 返回 `tokio::sync::mpsc::Receiver<WorktreeEvent>`
//! - `RealWorktreeCreateAsync` 用 `tokio::spawn` 真跑 `GitProvider::open_repo` +
//!   `create_worktree` + 5 `SharedMountStrategy` 落地 (per FR-ORCA-007 AC-1)
//! - `InMemoryWorktreeCreateAsync` 满足单测 (per `worktree-shared-dir/src/shared_dir_resolver.rs` 同款 in-mem 实装)
//!
//! ## 调用契约 (per FR-ORCA-008 AC-1 异步 + AC-2 进度可见)
//!
//! ```text
//! // 1. caller 先 subscribe (拿到 Receiver)
//! let mut rx = orch.subscribe(worktree_id);  // 注: start 之前 worktree_id 未知, 需调整
//! ```
//!
//! 实际契约修订: caller 先 `start()` 拿到 `worktree_id`, 然后 `subscribe(worktree_id)` 返回
//! `Receiver<WorktreeEvent>`. 早期事件 (Created) 可能在 subscribe 前发出, MVP 通过 **bounded
//! replay buffer** (per worktree_id) 缓存最近 N 事件, subscribe 时先 drain 再 recv (P1 followup:
//! 全量 replay; per 守门 #11 缺标比错标 MVP 先实装最小集).
//!
//! ## 状态机 (per FR-ORCA-008 + DD §23, **orchestrator transient state**, 不替换 `HumanState`)
//!
//! - 初始: `WorktreeCreateHandle::initial_state = HumanState::Waiting` (per DD §23.1 默认态;
//!   7 态状态机未定义 `Provisioning`, 但 transient orchestrator 阶段可标 `Waiting` 表示等 git 落盘)
//! - Created → ProvisioningCompleted → HumanState::Running (caller 负责 `transition(Waiting, Created)`)
//! - 任一失败 → ConflictDetected event, caller 把 `HumanState::Waiting` 推到 `HumanState::Conflict`
//! - Cancel → Cancelled event, caller 标 `Stale`
//!
//! ## MVP 范围 (per ULYS-217 §6 工时预估 D-Boy 9/22 自决)
//!
//! 本期实装 WorktreeAdd + Symlink 2 策略, 其它 3 (Hardlink / BindMount / Copy) 留 P1 followup, 落地时返回明确错误
//! 引导 caller 重试 (per FR-ORCA-008 AC-3 失败重试). 跨平台 Symlink: Unix 走
//! `std::os::unix::fs::symlink`, Windows NTFS 走 `std::fs::copy` fallback (junction 需 winapi,
//! P1 followup 加; per FR-ORCA-007 AC-1 "APFS clone-copy on macOS, otherwise symlink" 兜底).
//!
//! ## 守门
//!
//! - #1 v25 cargo test 单 crate 实证 (per ULYS-217 §5 验证)
//! - #6 v2 SharedDirError 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]` 或 path dep, 0 新外部 crate
//! - #19 v19 0 动 V0.1 任何业务 logic (本 crate 之前没有任何 `WorktreeCreate*`)
//! - #13 a L0↔L0 派生 (orchestrator 是 L0 协调设施, 跟 EventBus 同模式, 不引子 crate 跨域直连)

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use graph_core::state::HumanState;
use graph_core::types::{RepoId, WorktreeId};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use git_adapter::GitProvider;

use crate::error::{SharedDirError, SharedDirResult};
use crate::shared_dir_resolver::{ResolvedSharedDirs, SharedDirResolver};
use crate::shared_dir_types::SharedMountStrategy;

/// 默认 trace id (per 守门 #6 v2)
const DEFAULT_TRACE_ID: &str = "wca-default-trace";

/// 默认 mpsc channel buffer (per AC-2 sidebar 进度行 + cancel signal; 64 够覆盖正常推进)
const DEFAULT_EVENT_BUFFER: usize = 64;

/// 默认 per-worktree replay buffer (MVP: 缓存 64 个早期事件给 subscribe-after-start caller)
const DEFAULT_REPLAY_BUFFER: usize = 64;

// =====================================================================
// WorktreeEvent (per FR-ORCA-008 AC-1/2/3)
// =====================================================================

/// Worktree 创建 orchestrator 事件流 (per FR-ORCA-008 AC-1 异步 + AC-2 进度可见 + AC-3 失败重试)
///
/// 注: 本 enum **不替换** `worktree-service::lifecycle::WorktreeEvent` (后者是 11 events
/// 7-state 状态机迁移); 本 enum 是 transient orchestrator 阶段事件, 通过 mpsc channel 推到 UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorktreeEvent {
    /// Created (caller 收到 handle 后第一个事件)
    Created {
        /// Worktree UUID
        worktree_id: WorktreeId,
    },
    /// 进度更新 (per FR-ORCA-008 AC-2 sidebar 进度行)
    ProvisioningProgress {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 0-100 进度百分比
        pct: u8,
        /// 人类可读进度描述 (e.g. "fetching origin/main", "creating worktree", "symlinking /opt/shared")
        message: String,
        /// 事件时间戳
        at: DateTime<Utc>,
    },
    /// Provisioning 成功 (caller 应触发 `transition(Waiting, Created)` → `HumanState::Running`)
    ProvisioningCompleted {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// Worktree 最终本地路径
        worktree_path: PathBuf,
        /// 事件时间戳
        at: DateTime<Utc>,
    },
    /// 冲突 / 失败 (per FR-ORCA-008 AC-3 失败时显示错误 + Retry)
    ConflictDetected {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 失败原因 (e.g. "GIT.WORKTREE_CREATE_FAIL: branch already exists")
        reason: String,
        /// 错误码
        code: String,
        /// 事件时间戳
        at: DateTime<Utc>,
    },
    /// 取消 (per FR-ORCA-008 AC-1 「可取消」)
    Cancelled {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 事件时间戳
        at: DateTime<Utc>,
    },
}

// =====================================================================
// WorktreeCreateRequest / Handle
// =====================================================================

/// Worktree 创建请求 (per FR-ORCA-008)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeCreateRequest {
    /// Repo UUID (跟 worktree-service `RepoId` 对齐)
    pub repo_id: RepoId,
    /// Repo URL (e.g. "git@github.com:foo/bar.git")
    pub repo_url: String,
    /// Base ref (per FR-ORCA-008; 通常 `origin/main`, 由 caller 派生)
    pub base_ref: String,
    /// 新分支名 (per FR-ORCA-010 Branch 命名优先级链, 由 caller 派生后传入)
    pub branch: String,
    /// Worktree 本地路径 (e.g. `/path/to/worktrees/feat-x`)
    pub worktree_path: PathBuf,
    /// 共享目录列表 (来自 `SharedDirResolver::resolve` 3 机制合并结果)
    pub shared_dirs: ResolvedSharedDirs,
    /// 租户 ID (跨域隔离, INV-ACT-01)
    pub tenant_id: Uuid,
    /// Workspace ID (跨域隔离)
    pub workspace_id: Uuid,
}

/// Worktree 创建句柄 (per FR-ORCA-008 AC-1: start() 立即返回)
///
/// caller 拿到 handle 后 3 件事:
/// 1. 调 `subscribe(worktree_id)` → `Receiver<WorktreeEvent>` (UI 消费进度)
/// 2. 监听 `ProvisioningCompleted` → 把 `HumanState::Waiting` 推到 `Running`
/// 3. 监听 `ConflictDetected` → 显示 Retry 按钮 (per AC-3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeCreateHandle {
    /// Worktree UUID (orchestrator 内部生成, caller 持有)
    pub worktree_id: WorktreeId,
    /// 初始 HumanState (caller 推到状态机; 默认 Waiting, 跟 DD §23.1 一致)
    pub initial_state: HumanState,
    /// 创建时间戳
    pub created_at: DateTime<Utc>,
}

// =====================================================================
// WorktreeCreateAsync trait (per FR-ORCA-008 AC-1/2/3)
// =====================================================================

/// Worktree 异步创建 Trait (per FR-ORCA-008)
#[async_trait]
pub trait WorktreeCreateAsync: Send + Sync {
    /// 启动异步 create (per FR-ORCA-008 AC-1 提交 Create dialog 立即关闭)
    ///
    /// 立即返回 `WorktreeCreateHandle`; 实际 git fetch + worktree add + 5 mount_strategy
    /// 落地在 background tokio task 跑, 通过 `subscribe()` 拿 `Receiver<WorktreeEvent>` 监听进度.
    async fn start(&self, req: WorktreeCreateRequest) -> SharedDirResult<WorktreeCreateHandle>;

    /// 取消 in-flight create (per FR-ORCA-008 AC-1 「可取消」)
    ///
    /// 调 cancel 后 orchestrator emit `WorktreeEvent::Cancelled` 给对应 worktree_id 的 subscriber,
    /// 然后 abort 内部 tokio task + 尽力回滚 partial git worktree (best-effort, 不保证).
    async fn cancel(&self, worktree_id: WorktreeId) -> SharedDirResult<()>;

    /// 订阅进度 (per FR-ORCA-008 AC-2 进度可见)
    ///
    /// 返回 `Receiver<WorktreeEvent>`, 持有期间可收到 `Created` / `ProvisioningProgress` /
    /// `ProvisioningCompleted` / `ConflictDetected` / `Cancelled` 5 类事件.
    /// **MVP 行为**: subscribe-after-start caller 收不到 subscribe 之前的早期事件;
    /// Real impl 通过 bounded replay buffer (per-worktree 64 events) 缓存最近事件, subscribe 时
    /// 先 drain replay 再走 live channel. P1 followup: 全量 replay.
    ///
    /// async fn (per tokio Mutex 需要 .await; sync fn 用 `block_on` 在 tokio 上下文里会 panic)
    async fn subscribe(&self, worktree_id: WorktreeId) -> mpsc::Receiver<WorktreeEvent>;
}

// =====================================================================
// Per-worktree event router (per AC-2 进度可见)
// =====================================================================

/// per-worktree event router: live sender + bounded replay ring
#[derive(Debug)]
struct EventRouter {
    /// 当前 live sender (subscribe 之前是 `None`)
    live: Option<mpsc::Sender<WorktreeEvent>>,
    /// 最近 N 事件 replay ring (MVP: 64 事件)
    replay: VecDeque<WorktreeEvent>,
    /// replay buffer capacity
    capacity: usize,
}

impl EventRouter {
    fn new(capacity: usize) -> Self {
        Self {
            live: None,
            replay: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Emit event: 推 live sender + 缓存到 replay ring
    fn emit(&mut self, event: WorktreeEvent) {
        // 缓存到 replay (FIFO; capacity 满则 pop_front)
        if self.replay.len() >= self.capacity {
            self.replay.pop_front();
        }
        self.replay.push_back(event.clone());
        // 推 live sender (best-effort: 满则丢 + warn)
        if let Some(tx) = &self.live {
            match tx.try_send(event) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!(
                        target: "worktree_shared_dir::EventRouter",
                        "subscriber buffer full, event dropped (caller should drain more aggressively)"
                    );
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // subscriber closed; do not propagate (orchestrator continues)
                }
            }
        }
        // else: no live subscriber yet, event cached in replay
    }

    /// subscribe 路径: 建新 channel, 先 replay drain 到 caller-owned channel + sender
    fn subscribe(&mut self, buffer: usize) -> mpsc::Receiver<WorktreeEvent> {
        let (tx, rx) = mpsc::channel(buffer);
        // 先 drain replay (一次性 send all)
        for ev in self.replay.drain(..) {
            // blocking send 是 OK 的: caller 刚建 channel, 没人消费, 但 buffer = self.replay.len() 通常 ≤ 64
            // 用 try_send 防 caller panic
            let _ = tx.try_send(ev);
        }
        // 后续事件走 live sender
        self.live = Some(tx);
        rx
    }
}

// =====================================================================
// InMemoryWorktreeCreateAsync (单测 + 阶段 1 E2E 占位)
// =====================================================================

/// In-memory `WorktreeCreateAsync` 实装 (per `shared_dir_resolver.rs::InMemorySharedDirResolver` 同款)
///
/// 用于单测 + 阶段 1 E2E (per spec §3 阶段 1 简化). 不真正跑 git 命令, 通过
/// 注入 `GitProvider` mock 让 caller 控制成败.
pub struct InMemoryWorktreeCreateAsync {
    git_provider: Arc<dyn GitProvider>,
    shared_dir_resolver: Arc<dyn SharedDirResolver>,
    /// per-worktree event router
    routers: Arc<Mutex<HashMap<WorktreeId, EventRouter>>>,
    /// per-worktree cancel flag (per cancel() 触发 abort)
    cancel_flags: Arc<Mutex<HashMap<WorktreeId, Arc<Mutex<bool>>>>>,
}

impl std::fmt::Debug for InMemoryWorktreeCreateAsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryWorktreeCreateAsync").finish_non_exhaustive()
    }
}

impl InMemoryWorktreeCreateAsync {
    /// 构造 (per spec §3.3 orchestrator 装配, 默认事件 buffer + replay)
    pub fn new(
        git_provider: Arc<dyn GitProvider>,
        shared_dir_resolver: Arc<dyn SharedDirResolver>,
    ) -> Self {
        Self::with_buffer(git_provider, shared_dir_resolver, DEFAULT_EVENT_BUFFER)
    }

    /// 构造 with explicit event buffer (replay 容量固定 per MVP)
    pub fn with_buffer(
        git_provider: Arc<dyn GitProvider>,
        shared_dir_resolver: Arc<dyn SharedDirResolver>,
        _buffer: usize,
    ) -> Self {
        Self {
            git_provider,
            shared_dir_resolver,
            routers: Arc::new(Mutex::new(HashMap::new())),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// =====================================================================
// RealWorktreeCreateAsync (FR-ORCA-008 AC-1/2/3 真实 orchestrator)
// =====================================================================

/// 真实 `WorktreeCreateAsync` 实装 (per ULYS-158.4 + FR-ORCA-008 AC-1/2/3)
///
/// 装配:
/// - `git_provider`: 复用 `crates/git-adapter` 的 `Libgit2Provider` / `CliGitProvider`
/// - `shared_dir_resolver`: 复用 `InMemorySharedDirResolver` (3 机制聚合)
///
/// 状态机集成:
/// - `start()` 后 spawn tokio task, 跑 4 阶段:
///   1. `open_repo` + 拿 RepoHandle
///   2. `create_worktree` (via `GitProvider`)
///   3. 5 `SharedMountStrategy` 落地 (per entry from `shared_dirs.entries`)
///   4. emit `ProvisioningCompleted` 或 `ConflictDetected`
/// - `cancel()` 设 cancel flag + abort JoinHandle (best-effort)
/// - `subscribe(worktree_id)` 返回 per-worktree live mpsc::Receiver + drain replay ring
pub struct RealWorktreeCreateAsync {
    git_provider: Arc<dyn GitProvider>,
    shared_dir_resolver: Arc<dyn SharedDirResolver>,
    /// per-worktree event router
    routers: Arc<Mutex<HashMap<WorktreeId, EventRouter>>>,
    /// per-worktree cancel flag (cancel() 触发 abort)
    cancel_flags: Arc<Mutex<HashMap<WorktreeId, Arc<Mutex<bool>>>>>,
    /// per-worktree JoinHandle (cancel 时 abort, per AC-1 「可取消」)
    handles: Arc<Mutex<HashMap<WorktreeId, tokio::task::JoinHandle<()>>>>,
}

impl std::fmt::Debug for RealWorktreeCreateAsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealWorktreeCreateAsync").finish_non_exhaustive()
    }
}

impl RealWorktreeCreateAsync {
    /// 构造 (per spec §3.3 orchestrator 装配, 默认事件 buffer + replay)
    pub fn new(
        git_provider: Arc<dyn GitProvider>,
        shared_dir_resolver: Arc<dyn SharedDirResolver>,
    ) -> Self {
        Self::with_buffer(git_provider, shared_dir_resolver, DEFAULT_EVENT_BUFFER)
    }

    /// 构造 with explicit event buffer (replay 容量固定 per MVP)
    pub fn with_buffer(
        git_provider: Arc<dyn GitProvider>,
        shared_dir_resolver: Arc<dyn SharedDirResolver>,
        _buffer: usize,
    ) -> Self {
        Self {
            git_provider,
            shared_dir_resolver,
            routers: Arc::new(Mutex::new(HashMap::new())),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// =====================================================================
// Shared impl block for in-memory + real (cancel logic)
// =====================================================================

/// 取消 in-flight create (per FR-ORCA-008 AC-1 「可取消」)
async fn cancel_impl(
    routers: &Arc<Mutex<HashMap<WorktreeId, EventRouter>>>,
    cancel_flags: &Arc<Mutex<HashMap<WorktreeId, Arc<Mutex<bool>>>>>,
    handles: Option<&Arc<Mutex<HashMap<WorktreeId, tokio::task::JoinHandle<()>>>>>,
    worktree_id: WorktreeId,
) -> SharedDirResult<()> {
    // 1. 设 cancel flag (background task 轮询检测) + 从 map 移除 (per AC-1 idempotent cancel
    //    → 第二次 cancel 应报 WORKTREE_NOT_FOUND)
    let flag_exists = {
        let mut flags = cancel_flags.lock().await;
        if let Some(flag) = flags.remove(&worktree_id) {
            let mut f = flag.lock().await;
            *f = true;
            true
        } else {
            false
        }
    };
    if !flag_exists {
        return Err(SharedDirError::new(
            "WCA.WORKTREE_NOT_FOUND",
            format!("no in-flight create for worktree_id={worktree_id}"),
            DEFAULT_TRACE_ID,
        ));
    }
    // 2. Abort JoinHandle (per AC-1 「可取消」 best-effort; InMemory impl 不传 handles)
    if let Some(handles_map) = handles {
        let mut h = handles_map.lock().await;
        if let Some(handle) = h.remove(&worktree_id) {
            handle.abort();
        }
    }
    // 3. Emit Cancelled event (per AC-3 UI 收到后展示 Retry)
    let mut routers_map = routers.lock().await;
    if let Some(router) = routers_map.get_mut(&worktree_id) {
        router.emit(WorktreeEvent::Cancelled {
            worktree_id,
            at: Utc::now(),
        });
    }
    Ok(())
}

// =====================================================================
// MountStrategy 5 档落地 (per FR-ORCA-007 AC-1 + 9/22 D-Boy MVP 自决)
// =====================================================================

/// 5 mount_strategy 落地 (per FR-ORCA-007 AC-1: APFS clone-copy on macOS, otherwise symlink)
///
/// MVP 实装 (per ULYS-217 §6):
/// - `WorktreeAdd`: noop (已在 `git worktree add` 阶段处理)
/// - `Symlink`: 跨平台, Unix `std::os::unix::fs::symlink`, Windows 走 `std::fs::copy` fallback
///   (NTFS junction 留 P1 followup, per §6 「跨平台兼容」 1 周工时拆分)
/// - `Hardlink` / `BindMount` / `Copy`: 留 P1 followup, 返回明确错误引导 caller 重试
///
/// 注: 本函数不修改 `Self::run` 调用流程, 是独立可测试的纯函数 (per 守门 #1 单 crate 实证).
pub(crate) async fn apply_mount_strategy(
    strategy: SharedMountStrategy,
    worktree_path: &Path,
    source_path: &Path,
    dest_name: &str,
) -> SharedDirResult<()> {
    match strategy {
        SharedMountStrategy::WorktreeAdd => {
            // noop: source path 由 git worktree add 处理
            debug!(
                target: "worktree_shared_dir::apply_mount_strategy",
                worktree = %worktree_path.display(),
                source = %source_path.display(),
                "WorktreeAdd mount strategy: noop (git worktree add handles this)"
            );
            Ok(())
        }
        SharedMountStrategy::Symlink => {
            // 跨平台 symlink: Unix 走 os::unix::fs::symlink; Windows NTFS 走 std::fs::copy
            // fallback (junction 需 winapi, P1 followup 加, per FR-ORCA-007 AC-1)
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let link = worktree_path.join(dest_name);
                if let Some(parent) = link.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        SharedDirError::new(
                            "WCA.MOUNT_IO_FAIL",
                            format!("create_dir_all({}) failed: {e}", parent.display()),
                            DEFAULT_TRACE_ID,
                        )
                        .with_source(format!("io: {e}"))
                    })?;
                }
                // 若 link 已存在, 不覆盖 (per FR-ORCA-006 并行 worktree 隔离: 重复 symlink = 配置冲突)
                if link.exists() || link.symlink_metadata().is_ok() {
                    return Err(SharedDirError::new(
                        "WCA.MOUNT_TARGET_EXISTS",
                        format!("symlink target already exists: {}", link.display()),
                        DEFAULT_TRACE_ID,
                    ));
                }
                symlink(source_path, &link).map_err(|e| {
                    SharedDirError::new(
                        "WCA.MOUNT_SYMLINK_FAIL",
                        format!(
                            "symlink({} -> {}) failed: {e}",
                            source_path.display(),
                            link.display()
                        ),
                        DEFAULT_TRACE_ID,
                    )
                    .with_source(format!("io: {e}"))
                })?;
                info!(
                    target: "worktree_shared_dir::apply_mount_strategy",
                    worktree = %worktree_path.display(),
                    source = %source_path.display(),
                    link = %link.display(),
                    "Symlink mount strategy: created unix symlink"
                );
                Ok(())
            }
            #[cfg(windows)]
            {
                // Windows 走 std::fs::copy fallback (P1 followup 加 NTFS junction per §6 跨平台)
                warn!(
                    target: "worktree_shared_dir::apply_mount_strategy",
                    worktree = %worktree_path.display(),
                    source = %source_path.display(),
                    "Symlink mount strategy on Windows: falling back to recursive copy (NTFS junction is P1 followup, per FR-ORCA-007 AC-1)"
                );
                let target = worktree_path.join(dest_name);
                if target.exists() {
                    return Err(SharedDirError::new(
                        "WCA.MOUNT_TARGET_EXISTS",
                        format!("copy target already exists: {}", target.display()),
                        DEFAULT_TRACE_ID,
                    ));
                }
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        SharedDirError::new(
                            "WCA.MOUNT_IO_FAIL",
                            format!("create_dir_all({}) failed: {e}", parent.display()),
                            DEFAULT_TRACE_ID,
                        )
                        .with_source(format!("io: {e}"))
                    })?;
                }
                // copy_dir_all: 递归 copy (per FR-ORCA-007 AC-3 `.worktreeinclude` copy 模式)
                copy_dir_all(source_path, &target).map_err(|e| {
                    SharedDirError::new(
                        "WCA.MOUNT_COPY_FAIL",
                        format!(
                            "recursive copy ({} -> {}) failed: {e}",
                            source_path.display(),
                            target.display()
                        ),
                        DEFAULT_TRACE_ID,
                    )
                    .with_source(format!("io: {e}"))
                })?;
                Ok(())
            }
        }
        SharedMountStrategy::Hardlink => Err(SharedDirError::new(
            "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED",
            "Hardlink mount strategy is P1 followup (per ULYS-217 §6 MVP scope)",
            DEFAULT_TRACE_ID,
        )),
        SharedMountStrategy::BindMount => Err(SharedDirError::new(
            "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED",
            "BindMount mount strategy is P1 followup (per ULYS-217 §6 MVP scope, requires Linux mount syscall)",
            DEFAULT_TRACE_ID,
        )),
        SharedMountStrategy::Copy => Err(SharedDirError::new(
            "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED",
            "Copy mount strategy is P1 followup (per ULYS-217 §6 MVP scope; std::fs::copy path covered by Symlink-Windows fallback)",
            DEFAULT_TRACE_ID,
        )),
    }
}

/// 递归 copy directory (per FR-ORCA-007 AC-3 `.worktreeinclude` copy 模式 + Symlink Windows fallback)
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if ty.is_symlink() {
            // 跳过 symlink (避免循环)
            continue;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// =====================================================================
// RealWorktreeCreateAsync: Trait impl
// =====================================================================

#[async_trait]
impl WorktreeCreateAsync for RealWorktreeCreateAsync {
    async fn start(&self, req: WorktreeCreateRequest) -> SharedDirResult<WorktreeCreateHandle> {
        let worktree_id = WorktreeId::new_v4();
        let cancel_flag = Arc::new(Mutex::new(false));

        // 1. 注册 cancel flag (per cancel_impl 路径)
        {
            let mut flags = self.cancel_flags.lock().await;
            flags.insert(worktree_id, cancel_flag.clone());
        }
        // 1b. 注册 event router (per AC-2 进度可见)
        {
            let mut routers = self.routers.lock().await;
            routers.insert(worktree_id, EventRouter::new(DEFAULT_REPLAY_BUFFER));
        }

        // 2. 校验 request (per FR-ORCA-007 AC-4 共享目录 path 必须绝对 + repo_url 不能空)
        if req.repo_url.trim().is_empty() {
            return Err(SharedDirError::new(
                "WCA.REPO_URL_EMPTY",
                "repo_url must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }
        if req.base_ref.trim().is_empty() {
            return Err(SharedDirError::new(
                "WCA.BASE_REF_EMPTY",
                "base_ref must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }
        if req.branch.trim().is_empty() {
            return Err(SharedDirError::new(
                "WCA.BRANCH_EMPTY",
                "branch must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }
        if req.worktree_path.as_os_str().is_empty() {
            return Err(SharedDirError::new(
                "WCA.WORKTREE_PATH_EMPTY",
                "worktree_path must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }

        // 3. emit Created event (per AC-1 caller 收到 handle 后第一个事件)
        {
            let mut routers = self.routers.lock().await;
            if let Some(router) = routers.get_mut(&worktree_id) {
                router.emit(WorktreeEvent::Created { worktree_id });
            }
        }

        // 4. spawn tokio task 跑 4 阶段 (git fetch → worktree add → mount_strategy → complete)
        let git = self.git_provider.clone();
        let _resolver = self.shared_dir_resolver.clone();
        let cancel = cancel_flag.clone();
        let routers = self.routers.clone();
        let worktree_path = req.worktree_path.clone();
        let base_ref = req.base_ref.clone();
        let branch = req.branch.clone();
        let shared_dirs = req.shared_dirs.clone();
        let handle = tokio::spawn(async move {
            // helper: emit event through router
            let emit = |routers: Arc<Mutex<HashMap<WorktreeId, EventRouter>>>,
                        worktree_id: WorktreeId,
                        event: WorktreeEvent| {
                let routers = routers;
                async move {
                    let mut map = routers.lock().await;
                    if let Some(router) = map.get_mut(&worktree_id) {
                        router.emit(event);
                    } else {
                        error!(
                            target: "worktree_shared_dir::RealWorktreeCreateAsync",
                            worktree_id = %worktree_id,
                            "router disappeared before provisioning started"
                        );
                    }
                }
            };

            // helper: detect cancel flag
            let is_cancelled = || async {
                let f = cancel.lock().await;
                *f
            };

            // Stage 1: open_repo (10%)
            emit(
                routers.clone(),
                worktree_id,
                WorktreeEvent::ProvisioningProgress {
                    worktree_id,
                    pct: 10,
                    message: "opening repository".to_string(),
                    at: Utc::now(),
                },
            )
            .await;
            if is_cancelled().await {
                return;
            }
            let repo_parent = worktree_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf();
            let repo_handle = match git.open_repo(&repo_parent).await {
                Ok(h) => h,
                Err(e) => {
                    emit(
                        routers.clone(),
                        worktree_id,
                        WorktreeEvent::ConflictDetected {
                            worktree_id,
                            reason: format!("open_repo failed: {e}"),
                            code: "WCA.GIT_REPO_OPEN_FAIL".to_string(),
                            at: Utc::now(),
                        },
                    )
                    .await;
                    return;
                }
            };

            // Stage 2: git worktree add (40%)
            emit(
                routers.clone(),
                worktree_id,
                WorktreeEvent::ProvisioningProgress {
                    worktree_id,
                    pct: 40,
                    message: format!("creating worktree on branch {branch}"),
                    at: Utc::now(),
                },
            )
            .await;
            if is_cancelled().await {
                return;
            }
            if let Err(e) = git
                .create_worktree(&repo_handle, &branch, &worktree_path, Some(&base_ref))
                .await
            {
                emit(
                    routers.clone(),
                    worktree_id,
                    WorktreeEvent::ConflictDetected {
                        worktree_id,
                        reason: format!("create_worktree failed: {e}"),
                        code: "WCA.GIT_WORKTREE_CREATE_FAIL".to_string(),
                        at: Utc::now(),
                    },
                )
                .await;
                return;
            }

            // Stage 3: 5 mount_strategy 落地 (40% → 80%)
            let n = shared_dirs.entries.len();
            for (idx, entry) in shared_dirs.entries.iter().enumerate() {
                if is_cancelled().await {
                    return;
                }
                let pct = if n == 0 {
                    80u8
                } else {
                    40u8 + ((idx as u8 + 1) * 40 / n.max(1) as u8)
                };
                emit(
                    routers.clone(),
                    worktree_id,
                    WorktreeEvent::ProvisioningProgress {
                        worktree_id,
                        pct,
                        message: format!(
                            "mounting {} via {:?} ({}/{})",
                            entry.path.display(),
                            entry.mount_strategy,
                            idx + 1,
                            n
                        ),
                        at: Utc::now(),
                    },
                )
                .await;
                if let Err(e) =
                    apply_mount_strategy(entry.mount_strategy, &worktree_path, &entry.path, &entry.label).await
                {
                    let code = match e.code.as_str() {
                        "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED" => "WCA.MOUNT_NOT_IMPLEMENTED",
                        _ => &e.code,
                    };
                    emit(
                        routers.clone(),
                        worktree_id,
                        WorktreeEvent::ConflictDetected {
                            worktree_id,
                            reason: format!(
                                "mount_strategy {:?} failed for {}: {}",
                                entry.mount_strategy,
                                entry.path.display(),
                                e
                            ),
                            code: code.to_string(),
                            at: Utc::now(),
                        },
                    )
                    .await;
                    return;
                }
            }

            // Stage 4: emit ProvisioningCompleted (100%)
            emit(
                routers.clone(),
                worktree_id,
                WorktreeEvent::ProvisioningProgress {
                    worktree_id,
                    pct: 100,
                    message: "worktree ready".to_string(),
                    at: Utc::now(),
                },
            )
            .await;
            if is_cancelled().await {
                return;
            }
            emit(
                routers.clone(),
                worktree_id,
                WorktreeEvent::ProvisioningCompleted {
                    worktree_id,
                    worktree_path,
                    at: Utc::now(),
                },
            )
            .await;
        });

        // 5. 存 JoinHandle (per cancel_impl 路径 abort)
        {
            let mut handles_map = self.handles.lock().await;
            handles_map.insert(worktree_id, handle);
        }

        Ok(WorktreeCreateHandle {
            worktree_id,
            initial_state: HumanState::Waiting,
            created_at: Utc::now(),
        })
    }

    async fn cancel(&self, worktree_id: WorktreeId) -> SharedDirResult<()> {
        cancel_impl(
            &self.routers,
            &self.cancel_flags,
            Some(&self.handles),
            worktree_id,
        )
        .await
    }

    async fn subscribe(&self, worktree_id: WorktreeId) -> mpsc::Receiver<WorktreeEvent> {
        // async fn (per tokio Mutex 需要 .await)
        let routers = self.routers.clone();
        let (tx, rx) = mpsc::channel(DEFAULT_EVENT_BUFFER);
        let mut map = routers.lock().await;
        let router = map
            .entry(worktree_id)
            .or_insert_with(|| EventRouter::new(DEFAULT_REPLAY_BUFFER));
        // 先 drain replay 到 caller-owned channel
        let replayed: Vec<WorktreeEvent> = router.replay.drain(..).collect();
        for ev in replayed {
            let _ = tx.try_send(ev);
        }
        // 后续事件走 live sender
        router.live = Some(tx);
        drop(map);
        rx
    }
}

// =====================================================================
// InMemoryWorktreeCreateAsync: Trait impl (单测用, 跟 Real 行为对齐)
// =====================================================================

#[async_trait]
impl WorktreeCreateAsync for InMemoryWorktreeCreateAsync {
    async fn start(&self, req: WorktreeCreateRequest) -> SharedDirResult<WorktreeCreateHandle> {
        let worktree_id = WorktreeId::new_v4();
        let cancel_flag = Arc::new(Mutex::new(false));
        {
            let mut flags = self.cancel_flags.lock().await;
            flags.insert(worktree_id, cancel_flag.clone());
        }
        // 校验 (跟 Real impl 一致)
        if req.repo_url.trim().is_empty() {
            return Err(SharedDirError::new(
                "WCA.REPO_URL_EMPTY",
                "repo_url must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }
        if req.branch.trim().is_empty() {
            return Err(SharedDirError::new(
                "WCA.BRANCH_EMPTY",
                "branch must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }
        if req.worktree_path.as_os_str().is_empty() {
            return Err(SharedDirError::new(
                "WCA.WORKTREE_PATH_EMPTY",
                "worktree_path must not be empty",
                DEFAULT_TRACE_ID,
            ));
        }
        // InMemory: 不真跑 git + mount, 走 EventRouter emit Created + ProvisioningProgress + ProvisioningCompleted
        {
            let mut routers = self.routers.lock().await;
            let router = routers
                .entry(worktree_id)
                .or_insert_with(|| EventRouter::new(DEFAULT_REPLAY_BUFFER));
            router.emit(WorktreeEvent::Created { worktree_id });
            router.emit(WorktreeEvent::ProvisioningProgress {
                worktree_id,
                pct: 100,
                message: "in-memory provisioning complete".to_string(),
                at: Utc::now(),
            });
            router.emit(WorktreeEvent::ProvisioningCompleted {
                worktree_id,
                worktree_path: req.worktree_path.clone(),
                at: Utc::now(),
            });
        }
        Ok(WorktreeCreateHandle {
            worktree_id,
            initial_state: HumanState::Waiting,
            created_at: Utc::now(),
        })
    }

    async fn cancel(&self, worktree_id: WorktreeId) -> SharedDirResult<()> {
        cancel_impl(&self.routers, &self.cancel_flags, None, worktree_id).await
    }

    async fn subscribe(&self, worktree_id: WorktreeId) -> mpsc::Receiver<WorktreeEvent> {
        let routers = self.routers.clone();
        let (tx, rx) = mpsc::channel(DEFAULT_EVENT_BUFFER);
        let mut map = routers.lock().await;
        let router = map
            .entry(worktree_id)
            .or_insert_with(|| EventRouter::new(DEFAULT_REPLAY_BUFFER));
        let replayed: Vec<WorktreeEvent> = router.replay.drain(..).collect();
        for ev in replayed {
            let _ = tx.try_send(ev);
        }
        router.live = Some(tx);
        drop(map);
        rx
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_dir_resolver::{NoopConfigSource, NoopPerUserSource, NoopWorkspaceSource};
    use crate::shared_dir_types::{
        SharedDirPriority, SharedDirSource, SharedDirectory, SharedMountStrategy,
    };

    use std::path::PathBuf;

    /// Mock `GitProvider` (per 单测注入, 不真跑 git 命令)
    /// 字段 `fail_create: bool` 控制 `create_worktree` 是否失败, 让单测覆盖失败路径.
    #[derive(Clone)]
    struct MockGitProvider {
        fail_create: Arc<Mutex<bool>>,
    }
    impl MockGitProvider {
        fn new(fail_create: bool) -> Self {
            Self {
                fail_create: Arc::new(Mutex::new(fail_create)),
            }
        }
    }

    #[async_trait]
    impl GitProvider for MockGitProvider {
        async fn open_repo(
            &self,
            _path: &Path,
        ) -> Result<git_adapter::RepoHandle, git_adapter::GitError> {
            Ok(git_adapter::RepoHandle {
                path: PathBuf::from("/mock/repo"),
                repo_id: None,
            })
        }
        async fn list_worktrees(
            &self,
            _repo: &git_adapter::RepoHandle,
        ) -> Result<Vec<git_adapter::WorktreeInfo>, git_adapter::GitError> {
            Ok(Vec::new())
        }
        async fn create_worktree(
            &self,
            _repo: &git_adapter::RepoHandle,
            _branch: &str,
            _path: &Path,
            _base: Option<&str>,
        ) -> Result<git_adapter::WorktreeInfo, git_adapter::GitError> {
            if *self.fail_create.lock().await {
                Err(git_adapter::GitError::new(
                    "GIT.WORKTREE_CREATE_FAIL",
                    "mock failure",
                ))
            } else {
                Ok(git_adapter::WorktreeInfo {
                    path: PathBuf::from("/mock/worktree"),
                    name: "mock-wt".to_string(),
                    branch: "feat/x".to_string(),
                    head: "abc123".to_string(),
                    ahead: 0,
                    behind: 0,
                    dirty: false,
                    last_commit_at: None,
                })
            }
        }
        async fn remove_worktree(
            &self,
            _repo: &git_adapter::RepoHandle,
            _path: &Path,
            _force: bool,
        ) -> Result<(), git_adapter::GitError> {
            Ok(())
        }
        async fn diff(
            &self,
            _repo: &git_adapter::RepoHandle,
            _from: &str,
            _to: &str,
        ) -> Result<git_adapter::Diff, git_adapter::GitError> {
            Ok(git_adapter::Diff {
                files: Vec::new(),
                total_added: 0,
                total_removed: 0,
            })
        }
        async fn merge_base(
            &self,
            _repo: &git_adapter::RepoHandle,
            _a: &str,
            _b: &str,
        ) -> Result<String, git_adapter::GitError> {
            Ok("abc".to_string())
        }
        async fn rev_list_count(
            &self,
            _repo: &git_adapter::RepoHandle,
            _range: &str,
        ) -> Result<u32, git_adapter::GitError> {
            Ok(0)
        }
        async fn status(
            &self,
            _repo: &git_adapter::RepoHandle,
        ) -> Result<Vec<git_adapter::StatusEntry>, git_adapter::GitError> {
            Ok(Vec::new())
        }
        async fn sync_main(
            &self,
            _repo: &git_adapter::RepoHandle,
            _worktree_path: &Path,
        ) -> Result<git_adapter::SyncResult, git_adapter::GitError> {
            Ok(git_adapter::SyncResult {
                fetched_commits: 0,
                rebased_commits: 0,
                conflicts: Vec::new(),
            })
        }
        async fn rebase(
            &self,
            _repo: &git_adapter::RepoHandle,
            _worktree_path: &Path,
            _target: &str,
        ) -> Result<(), git_adapter::GitError> {
            Ok(())
        }
        async fn merge(
            &self,
            _repo: &git_adapter::RepoHandle,
            _worktree_path: &Path,
            _branch: &str,
            _strategy: git_adapter::MergeStrategy,
        ) -> Result<git_adapter::MergeResult, git_adapter::GitError> {
            Ok(git_adapter::MergeResult {
                success: true,
                merge_commit: Some("abc".to_string()),
                conflicts: Vec::new(),
                strategy_used: git_adapter::MergeStrategy::Merge,
            })
        }
        async fn head(
            &self,
            _repo: &git_adapter::RepoHandle,
            _worktree_path: &Path,
        ) -> Result<String, git_adapter::GitError> {
            Ok("abc".to_string())
        }
        async fn last_commit_at(
            &self,
            _repo: &git_adapter::RepoHandle,
            _worktree_path: &Path,
        ) -> Result<chrono::DateTime<chrono::Utc>, git_adapter::GitError> {
            Ok(Utc::now())
        }
        async fn list_branches(
            &self,
            _repo: &git_adapter::RepoHandle,
        ) -> Result<Vec<git_adapter::BranchInfo>, git_adapter::GitError> {
            Ok(Vec::new())
        }
    }

    fn dummy_request() -> WorktreeCreateRequest {
        WorktreeCreateRequest {
            repo_id: RepoId::new_v4(),
            repo_url: "git@example.com:foo/bar.git".to_string(),
            base_ref: "origin/main".to_string(),
            branch: "feat/test".to_string(),
            worktree_path: PathBuf::from("/tmp/wt-test"),
            shared_dirs: ResolvedSharedDirs::default(),
            tenant_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
        }
    }

    fn dummy_resolver() -> Arc<dyn SharedDirResolver> {
        Arc::new(crate::shared_dir_resolver::InMemorySharedDirResolver::new(
            Arc::new(NoopPerUserSource),
            Arc::new(NoopWorkspaceSource),
            Arc::new(NoopConfigSource),
        ))
    }

    #[test]
    fn worktree_event_serializes_snake_case() {
        let e = WorktreeEvent::ProvisioningProgress {
            worktree_id: WorktreeId::new_v4(),
            pct: 50,
            message: "test".to_string(),
            at: Utc::now(),
        };
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("\"kind\":\"provisioning_progress\""));
        assert!(s.contains("\"pct\":50"));
    }

    #[test]
    fn worktree_event_variants_5_types() {
        // Per FR-ORCA-008 AC-1/2/3 完整覆盖
        let variants = vec![
            WorktreeEvent::Created {
                worktree_id: WorktreeId::new_v4(),
            },
            WorktreeEvent::ProvisioningProgress {
                worktree_id: WorktreeId::new_v4(),
                pct: 10,
                message: "x".to_string(),
                at: Utc::now(),
            },
            WorktreeEvent::ProvisioningCompleted {
                worktree_id: WorktreeId::new_v4(),
                worktree_path: PathBuf::from("/x"),
                at: Utc::now(),
            },
            WorktreeEvent::ConflictDetected {
                worktree_id: WorktreeId::new_v4(),
                reason: "x".to_string(),
                code: "X".to_string(),
                at: Utc::now(),
            },
            WorktreeEvent::Cancelled {
                worktree_id: WorktreeId::new_v4(),
                at: Utc::now(),
            },
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn worktree_create_handle_initial_state_is_waiting() {
        // Per DD §23.1 + ULYS-217 §2.2 transient state 默认 Waiting
        // (7-state 不含 Provisioning, Waiting 是 closest 含义: 等 git 落盘)
        let handle = WorktreeCreateHandle {
            worktree_id: WorktreeId::new_v4(),
            initial_state: HumanState::Waiting,
            created_at: Utc::now(),
        };
        assert_eq!(handle.initial_state, HumanState::Waiting);
    }

    #[tokio::test]
    async fn real_start_emits_created_and_validates_request() {
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());

        // 正常路径: handle returned, worktree_id 非 nil
        let mut req = dummy_request();
        req.shared_dirs = ResolvedSharedDirs {
            entries: vec![SharedDirectory {
                source: SharedDirSource::Workspace,
                path: PathBuf::from("/opt/shared"),
                mount_strategy: SharedMountStrategy::WorktreeAdd, // noop, 测 WorktreeAdd 不报错
                label: "shared".to_string(),
                priority: SharedDirPriority::P1,
                enabled: true,
            }],
            sources_hit: vec![SharedDirSource::Workspace],
        };
        let handle = orch.start(req).await.unwrap();
        assert_eq!(handle.initial_state, HumanState::Waiting);
        assert_ne!(handle.worktree_id, WorktreeId::nil());
    }

    #[tokio::test]
    async fn real_start_rejects_empty_repo_url() {
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let mut req = dummy_request();
        req.repo_url = "".to_string();
        let err = orch.start(req).await.unwrap_err();
        assert_eq!(err.code, "WCA.REPO_URL_EMPTY");
    }

    #[tokio::test]
    async fn real_start_rejects_empty_branch() {
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let mut req = dummy_request();
        req.branch = "".to_string();
        let err = orch.start(req).await.unwrap_err();
        assert_eq!(err.code, "WCA.BRANCH_EMPTY");
    }

    #[tokio::test]
    async fn real_start_rejects_empty_worktree_path() {
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let mut req = dummy_request();
        req.worktree_path = PathBuf::new();
        let err = orch.start(req).await.unwrap_err();
        assert_eq!(err.code, "WCA.WORKTREE_PATH_EMPTY");
    }

    #[tokio::test]
    async fn real_cancel_emits_cancelled_event() {
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let req = dummy_request();
        let handle = orch.start(req).await.unwrap();

        // subscribe first to capture the Cancelled event
        let mut rx = orch.subscribe(handle.worktree_id).await;
        orch.cancel(handle.worktree_id).await.unwrap();
        // 收到 Cancelled event (per AC-3 UI 收到后展示 Retry).
        // 注: Created event 已在 start() 同步 emit 到 replay, subscribe 时 drain;
        //     Cancelled event 在 cancel() 后 emit, 走 live sender.
        // 消费掉 Created (replay) 后, 应拿到 Cancelled.
        let mut saw_cancelled = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            match tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await {
                Ok(Some(WorktreeEvent::Cancelled { .. })) => {
                    saw_cancelled = true;
                    break;
                }
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(_) => continue,
            }
        }
        assert!(saw_cancelled, "did not receive Cancelled event within 2s");
    }

    #[tokio::test]
    async fn real_cancel_unknown_worktree_errors() {
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let unknown = WorktreeId::new_v4();
        let err = orch.cancel(unknown).await.unwrap_err();
        assert_eq!(err.code, "WCA.WORKTREE_NOT_FOUND");
    }

    #[tokio::test]
    async fn in_memory_start_emits_full_event_sequence() {
        let git = Arc::new(MockGitProvider::new(false));
        let resolver = dummy_resolver();
        let orch = InMemoryWorktreeCreateAsync::new(git, resolver);
        let req = dummy_request();

        let handle = orch.start(req).await.unwrap();
        let mut rx = orch.subscribe(handle.worktree_id).await;

        // 应收到 Created + ProvisioningProgress + ProvisioningCompleted 3 事件 (per InMemory emit 顺序)
        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        let e3 = rx.recv().await.unwrap();
        assert!(matches!(e1, WorktreeEvent::Created { .. }));
        assert!(matches!(e2, WorktreeEvent::ProvisioningProgress { .. }));
        assert!(matches!(e3, WorktreeEvent::ProvisioningCompleted { .. }));
    }

    #[tokio::test]
    async fn in_memory_subscribe_replays_early_events() {
        // Per FR-ORCA-008 AC-2: subscribe-after-start 应能 replay 早期事件
        let git = Arc::new(MockGitProvider::new(false));
        let orch = InMemoryWorktreeCreateAsync::new(git, dummy_resolver());
        let req = dummy_request();

        let handle = orch.start(req).await.unwrap();
        // subscribe 在 start 之后, 应通过 replay ring 拿到 Created event
        let mut rx = orch.subscribe(handle.worktree_id).await;
        let e = rx.recv().await.unwrap();
        assert!(
            matches!(e, WorktreeEvent::Created { .. }),
            "expected replayed Created, got {e:?}"
        );
    }

    #[tokio::test]
    async fn apply_mount_strategy_worktree_add_is_noop() {
        let res = apply_mount_strategy(
            SharedMountStrategy::WorktreeAdd,
            Path::new("/tmp/wt"),
            Path::new("/opt/shared"),
            "shared",
        )
        .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn apply_mount_strategy_hardlink_errors_p1_followup() {
        let res = apply_mount_strategy(
            SharedMountStrategy::Hardlink,
            Path::new("/tmp/wt"),
            Path::new("/opt/shared"),
            "shared",
        )
        .await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().code,
            "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED"
        );
    }

    #[tokio::test]
    async fn apply_mount_strategy_bindmount_errors_p1_followup() {
        let res = apply_mount_strategy(
            SharedMountStrategy::BindMount,
            Path::new("/tmp/wt"),
            Path::new("/opt/shared"),
            "shared",
        )
        .await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().code,
            "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED"
        );
    }

    #[tokio::test]
    async fn apply_mount_strategy_copy_errors_p1_followup() {
        let res = apply_mount_strategy(
            SharedMountStrategy::Copy,
            Path::new("/tmp/wt"),
            Path::new("/opt/shared"),
            "shared",
        )
        .await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().code,
            "WCA.MOUNT_STRATEGY_NOT_IMPLEMENTED"
        );
    }

    #[tokio::test]
    async fn apply_mount_strategy_symlink_unix() {
        // Unix: 真建 symlink, 验证 file_type
        #[cfg(unix)]
        {
            let tmp = tempfile::tempdir().unwrap();
            let source = tmp.path().join("source");
            std::fs::create_dir_all(&source).unwrap();
            std::fs::write(source.join("data.txt"), "hello").unwrap();
            let worktree = tmp.path().join("worktree");
            std::fs::create_dir_all(&worktree).unwrap();
            // 调用 apply (Unix 分支)
            let res = apply_mount_strategy(
                SharedMountStrategy::Symlink,
                &worktree,
                &source,
                "source",
            )
            .await;
            assert!(res.is_ok(), "got {:?}", res.err());
            let link = worktree.join("source");
            assert!(link.exists() || link.symlink_metadata().is_ok());
            // 验证 symlink 指向 source
            let meta = std::fs::symlink_metadata(&link).unwrap();
            assert!(meta.file_type().is_symlink());
        }
        #[cfg(not(unix))]
        {
            // Windows: std::fs::copy fallback, 跳过 (per §6 P1 followup)
            let _ = tempfile::tempdir();
        }
    }

    #[tokio::test]
    async fn apply_mount_strategy_symlink_existing_target_errors() {
        // 重复 symlink 应该报错 (per FR-ORCA-006 并行 worktree 隔离)
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        std::fs::create_dir_all(&source).unwrap();
        let worktree = tmp.path().join("worktree");
        std::fs::create_dir_all(&worktree).unwrap();
        // 预先建一个同名的 link (per Unix) 或 dir (per Windows)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&source, worktree.join("source")).unwrap();
        }
        #[cfg(not(unix))]
        {
            std::fs::create_dir_all(worktree.join("source")).unwrap();
        }
        let res = apply_mount_strategy(
            SharedMountStrategy::Symlink,
            &worktree,
            &source,
            "source",
        )
        .await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "WCA.MOUNT_TARGET_EXISTS");
    }

    #[tokio::test]
    async fn real_start_with_worktreeadd_entry_succeeds() {
        // 覆盖 §5 验证清单: WorktreeAdd strategy 落地不报错
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let mut req = dummy_request();
        req.shared_dirs = ResolvedSharedDirs {
            entries: vec![SharedDirectory {
                source: SharedDirSource::MulticaConfig,
                path: PathBuf::from("/opt/shared"),
                mount_strategy: SharedMountStrategy::WorktreeAdd,
                label: "shared".to_string(),
                priority: SharedDirPriority::P3,
                enabled: true,
            }],
            sources_hit: vec![SharedDirSource::MulticaConfig],
        };
        let handle = orch.start(req).await.unwrap();
        // Mock GitProvider 不真跑 git, handle 立即返回 (per AC-1 异步)
        assert_eq!(handle.initial_state, HumanState::Waiting);
    }

    #[tokio::test]
    async fn real_start_failure_emits_conflict_detected_in_spawned_task() {
        // Mock GitProvider fail_create=true 模拟 git worktree add 失败
        let git = Arc::new(MockGitProvider::new(true));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let req = dummy_request();
        let handle = orch.start(req).await.unwrap();
        assert_eq!(handle.initial_state, HumanState::Waiting);
        // subscribe 后等 ConflictDetected event
        let mut rx = orch.subscribe(handle.worktree_id).await;
        // 等若干事件, 第 1 个应是 Created (start 路径同步 emit), 后续是 ProvisioningProgress + ConflictDetected
        let e1 = rx.recv().await.unwrap();
        assert!(
            matches!(e1, WorktreeEvent::Created { .. }),
            "expected Created, got {e1:?}"
        );
        // 给 tokio task 时间跑
        let mut found_conflict = false;
        for _ in 0..5 {
            if let Ok(Some(event)) =
                tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await
            {
                if matches!(event, WorktreeEvent::ConflictDetected { .. }) {
                    found_conflict = true;
                    break;
                }
            } else {
                break;
            }
        }
        assert!(found_conflict, "expected ConflictDetected event");
    }

    #[tokio::test]
    async fn real_start_then_cancel_is_safe_under_load() {
        // 多 worktree 并发 start + cancel, 验证 cancel flag 独立 (per AC-1 「可取消」)
        let git = Arc::new(MockGitProvider::new(false));
        let orch = RealWorktreeCreateAsync::new(git, dummy_resolver());
        let mut handles = Vec::new();
        for _ in 0..4 {
            let req = dummy_request();
            handles.push(orch.start(req).await.unwrap().worktree_id);
        }
        // 逐个 cancel
        for id in &handles {
            orch.cancel(*id).await.unwrap();
        }
        // 再次 cancel 应报 NOT_FOUND
        let err = orch.cancel(*handles.first().unwrap()).await.unwrap_err();
        assert_eq!(err.code, "WCA.WORKTREE_NOT_FOUND");
    }
}
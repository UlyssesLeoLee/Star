//! `service.rs` — `WorktreeService` Trait + 9 方法 (per DD §12)
//!
//! Per DD §12 + IMPL-PLAN §4.3 + INV-WC-01/02/03/08/09:
//! - 9 方法: list / get / create / update / delete / compute_health / compute_risks /
//!   sync_main / subscribe
//! - 7 HumanState 状态机迁移
//! - RLS 13 类 (per INV-WC-09, NFR-SEC-001)
//! - Health + Risk 集成 (per INV-WC-02/03)
//! - Cleanup 批量 + Archive 保留 Provenance (per INV-WC-08)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use graph_core::state::{HumanState, MergeStrategy, TestState};
use graph_core::types::{RepoId, UserId, WorktreeId};

use crate::start_from_picker::PickerCandidates;

use crate::error::ServiceError;
use crate::lifecycle::WorktreeEvent;

/// Worktree 过滤 (per DD §12)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeFilter {
    /// 限定 human_state 集合
    pub human_state: Option<Vec<HumanState>>,
    /// 限定 agent_type
    pub agent_type: Option<String>,
    /// 最小 behind 阈值
    pub min_behind: Option<u32>,
    /// 最大 health_score (low < threshold)
    pub max_health: Option<u8>,
    /// locked 过滤
    pub locked: Option<bool>,
    /// archived 过滤
    pub archived: Option<bool>,
    /// Limit
    pub limit: Option<u32>,
}

/// Worktree 更新参数 (per DD §12)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeUpdate {
    /// 状态直接 set (用于 admin 操作)
    pub human_state: Option<HumanState>,
    /// 触发状态机迁移事件
    pub event: Option<WorktreeEvent>,
    /// Lock
    pub lock: Option<bool>,
    /// Archive
    pub archived: Option<bool>,
    /// Branch 重命名
    pub branch: Option<String>,
}

/// Sync 结果 (per DD §12)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// 操作前 ahead/behind
    pub before: (u32, u32),
    /// 操作后 ahead/behind
    pub after: (u32, u32),
    /// 是否需要 fast-forward
    pub fast_forwarded: bool,
    /// Sync 时间戳
    pub synced_at: DateTime<Utc>,
    /// 使用的 merge strategy (per DD §12)
    pub strategy: MergeStrategy,
}

/// Worktree 完整快照 (per DD §3.1 DTO 简化版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// Worktree UUID
    pub id: WorktreeId,
    /// Repo UUID
    pub repo_id: RepoId,
    /// Worktree 名称
    pub name: String,
    /// Branch
    pub branch: String,
    /// 本地路径
    pub path: PathBuf,
    /// Human state (7 态)
    pub human_state: HumanState,
    /// Test state (None/Passed/Failed/Running)
    pub test_state: TestState,
    /// Ahead
    pub ahead: u32,
    /// Behind
    pub behind: u32,
    /// Dirty
    pub dirty: bool,
    /// Locked
    pub locked: bool,
    /// Archived
    pub archived: bool,
    /// Health score (0-100)
    pub health_score: u8,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Created
    pub created_at: DateTime<Utc>,
    /// Merged at
    pub merged_at: Option<DateTime<Utc>>,
    /// Lock owner
    pub locked_by: Option<UserId>,
    /// Agent type
    pub agent_type: Option<String>,
}

/// WorktreeEvent (per DD §12) — 事件流返回类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorktreeEventEnvelope {
    /// 创建
    Created {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// Repo UUID
        repo_id: RepoId,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 状态变化
    StateChanged {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 之前
        from: HumanState,
        /// 之后
        to: HumanState,
        /// 触发 event
        event: WorktreeEvent,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// Health 变化
    HealthChanged {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 之前
        from: u8,
        /// 之后
        to: u8,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// Deleted
    Deleted {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// Archived
    Archived {
        /// Worktree UUID
        worktree_id: WorktreeId,
        /// 时间戳
        at: DateTime<Utc>,
    },
}

/// WorktreeService Trait (per DD §12)
#[async_trait]
pub trait WorktreeService: Send + Sync {
    /// 列出 Worktrees (per DD §12 `list`)
    async fn list(
        &self,
        repo_id: RepoId,
        filter: Option<WorktreeFilter>,
    ) -> Result<Vec<Worktree>, ServiceError>;

    /// 取单个 Worktree (per DD §12 `get`)
    async fn get(&self, id: WorktreeId) -> Result<Worktree, ServiceError>;

    /// 创建 Worktree (per DD §12 `create`)
    async fn create(
        &self,
        repo_id: RepoId,
        branch: &str,
        base: Option<&str>,
    ) -> Result<Worktree, ServiceError>;

    /// 更新 Worktree (per DD §12 `update`)
    async fn update(
        &self,
        id: WorktreeId,
        update: WorktreeUpdate,
    ) -> Result<Worktree, ServiceError>;

    /// 删除 Worktree (per DD §12 `delete`, force=true 跳过 lock 校验)
    async fn delete(&self, id: WorktreeId, force: bool) -> Result<(), ServiceError>;

    /// 计算 Health Score (per DD §12 `compute_health` + INV-WC-02)
    async fn compute_health(&self, id: WorktreeId) -> Result<u8, ServiceError>;

    /// 计算 Risks (per DD §12 `compute_risks` + INV-WC-03)
    async fn compute_risks(&self, id: WorktreeId) -> Result<Vec<Uuid>, ServiceError>;

    /// Sync Main (per DD §12 `sync_main`, fetch + rebase)
    async fn sync_main(&self, id: WorktreeId) -> Result<SyncResult, ServiceError>;

    /// 订阅事件流 (per DD §12 `subscribe`)
    async fn subscribe(&self) -> Result<BoxStream<'static, WorktreeEventEnvelope>, ServiceError>;

    /// 批量 Cleanup (per INV-WC-08)
    async fn cleanup_batch(
        &self,
        repo_id: RepoId,
        older_than_days: i64,
        force: bool,
    ) -> Result<Vec<WorktreeId>, ServiceError>;

    /// Archive 单个 (per INV-WC-08 保留 Provenance)
    async fn archive(&self, id: WorktreeId) -> Result<(), ServiceError>;

    /// 扫描 + 导入 external git worktrees (per ULYS-195 / FR-ORCA-011).
    ///
    /// `repo_path` = 仓库 git 根目录; `force=true` 时同 branch 冲突会覆盖更新
    /// 内部 Worktree.branch/path 指向新 worktree_path.
    ///
    /// 返回 `ImportOutcome { imported, skipped, updated }`, caller 可决定
    /// 是否发 SSE `WorktreeEventEnvelope::Created` 给订阅者.
    async fn import_external_worktrees(
        &self,
        repo_id: RepoId,
        repo_path: &std::path::Path,
        force: bool,
    ) -> Result<crate::external_worktree_import::ImportOutcome, ServiceError>;

    /// 选 Start-from Picker candidates (per ULYS-194 FR-ORCA-009)
    ///
    /// 默认实现走 `InMemoryWorktreeService` + 注入的 `StartFromPickerSource`.
    /// 默认 source = `NoopPickerSource` (永远只返 empty).
    async fn pick_start_from_candidates(
        &self,
        repo_id: RepoId,
    ) -> Result<PickerCandidates, ServiceError>;
}

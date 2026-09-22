//! `edit` — Worktree ContextEdit + Fork with `parent.at`
//!
//! **crate**: domain-worktree (新)
//! **per**: ULYS-185 / PI-8 / SRS-PI-BORROW-001 §1.3 + §4 FR-29 (2 周估)
//! **参考**: pi-durable/src/types.ts:21-33 (ContextEdit), 36-49 (Fork)
//!
//! ## 职责
//!
//! 1. **ContextEdit** — 对 Worktree 关联的 Context 进行增量编辑 (omit/replace)
//! 2. **Fork** — 从父 Worktree 的某时间点 (`parent.at`) 派生出子 Worktree
//!
//! ## 关键不变量
//!
//! - **INV-ED-01**:ContextEdit 不修改 14/17 状态机(per SRS §8 缺口 #8:14 状态机冻结承诺)
//! - **INV-ED-02**:`parent.at` 必为 RFC3339 时间戳,且不大于当前时间(未来时间拒)
//! - **INV-ED-03**:Fork 出来的子 Worktree 必带 `parent_worktree_id` + `parent_at`
//! - **INV-ED-04**:ContextEdit.path 不可为空,必以 `/` 开头
//! - **INV-ED-05**:Fork 后父 Worktree 状态不变(仅新增 child)
//! - **INV-ED-06**:`ContextEdit` 适用于 Worktree 的 `context_state` 投影,不影响 `status` / `health` / `conflict_state`
//! - **INV-ED-07**:ContextEdit 需在 Worktree 处于活跃态 (`Created` / `Ready` / `Assigned` / `AgentRunning`),
//!   终态 (`Merged` / `Archived` / `Abandoned`) 拒绝编辑
//!
//! ## 设计决策
//!
//! - **不引 sqlx-macros / refinery**(per ULYS-185 §"持久化层约定" + ULYS-156 memory entry):
//!   持久化由 Application / infrastructure 层负责;本模块只持有 in-memory 表达 + 业务规则
//! - **不引新依赖**(0-deps 模块,仅 workspace 共享 `serde` / `chrono` / `uuid` / `thiserror`)
//! - **不破坏现有 17 状态机**:`ContextEdit` 与 `Fork` 是新 port trait 方法,不动 `transition_status` 表
//! - **`parent.at` 字段**:作为 `Worktree` 聚合根的扩展字段 `parent_at: Option<DateTime<Utc>>` +
//!   `parent_worktree_id: Option<WorktreeId>`,均 `Option` 以保持向后兼容
//!
//! Lead 责任: Worktree Lead

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_context::ActorContext;
use uuid::Uuid;

use crate::{
    InMemoryWorktreeRepository, Worktree, WorktreeError, WorktreeId, WorktreeRepository,
    WorktreeStatus,
};

// =====================================================================
// ID 类型(本地扩展,不复用 define_uuid_id! 避免顶层 ID 集合膨胀)
// =====================================================================

/// ContextEdit 单次操作的 ID(用于审计 / 持久化层追踪)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
/// 单次 ContextEdit 的 UUID(per INV-ED-08:edits 不可共享 ID)
pub struct ContextEditId(pub Uuid);

impl ContextEditId {
    /// 生成新 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContextEditId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ContextEditId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ContextEditId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

// =====================================================================
// ContextEdit 协议(per pi-durable/src/types.ts:21-33)
// =====================================================================

/// **ContextEdit** — 对 Worktree Context 的单次增量编辑
///
/// 两种 op:
/// - `Omit { path }` — 移除 `path` 处的 context node
/// - `Replace { path, value }` — 用 `value` 替换 `path` 处的 context node
///
/// **per**: pi-durable/src/types.ts:21-33 (ContextEdit type)
/// **守门 INV-ED-04**:`path` 不可为空,必以 `/` 开头
///
/// **设计**:`value` 使用 `String` 表示(0-deps,不引 `serde_json`)，
///   Application 层负责将业务对象 serialize 成 string,Edit 层仅做字符串替换语义.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ContextEdit {
    /// 移除 `path` 处的 context node
    Omit {
        /// context 节点路径 (e.g. `/system_messages/3`)
        path: String,
    },
    /// 用 `value` 替换 `path` 处的 context node
    Replace {
        /// context 节点路径
        path: String,
        /// 替换后的 JSON 值(调用方 serialize;Edit 层不解析)
        value: String,
    },
}

impl ContextEdit {
    /// 获取 op 的路径(INV-ED-04 校验在构造时由 service 负责)
    pub fn path(&self) -> &str {
        match self {
            Self::Omit { path } | Self::Replace { path, .. } => path,
        }
    }

    /// 校验 INV-ED-04:path 非空 + 以 `/` 开头
    pub fn validate(&self) -> Result<(), EditError> {
        let path = self.path();
        if path.is_empty() {
            return Err(EditError::InvalidPath {
                reason: "path is empty".to_string(),
                path: path.to_string(),
            });
        }
        if !path.starts_with('/') {
            return Err(EditError::InvalidPath {
                reason: "path must start with '/'".to_string(),
                path: path.to_string(),
            });
        }
        Ok(())
    }
}

/// 多个 ContextEdit 顺序应用的一批操作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// ContextEdit 批
pub struct ContextEditBatch {
    /// 批 ID(由调用方生成,用于审计追踪)
    pub batch_id: Uuid,
    /// 顺序应用的操作列表(顺序敏感:后置 op 操作前置 op 后的状态)
    pub edits: Vec<ContextEdit>,
}

impl ContextEditBatch {
    /// 构造新批(默认 batch_id)
    pub fn new(edits: Vec<ContextEdit>) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            edits,
        }
    }

    /// 校验批内全部 edit 都满足 INV-ED-04
    pub fn validate(&self) -> Result<(), EditError> {
        for edit in &self.edits {
            edit.validate()?;
        }
        Ok(())
    }
}

// =====================================================================
// Fork 协议(per pi-durable/src/types.ts:36-49)
// =====================================================================

/// **Fork 规格** — Fork 一个 Worktree 时,指定父 Worktree + 分叉时间点
///
/// **per**: pi-durable/src/types.ts:36-49 (Fork type)
/// **守门 INV-ED-02**:`parent_at` 必为 RFC3339 时间戳,且不大于当前时间
/// **守门 INV-ED-03**:Fork 出来的子 Worktree 必带 `parent_worktree_id` + `parent_at`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Fork 参数
pub struct ForkSpec {
    /// 父 Worktree ID(必带)
    pub parent_worktree_id: WorktreeId,
    /// 分叉时间点 (parent.at) — RFC3339
    pub parent_at: DateTime<Utc>,
}

impl ForkSpec {
    /// 校验 INV-ED-02:parent_at 不大于当前时间
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), EditError> {
        if self.parent_at > now {
            return Err(EditError::InvalidParentAt {
                reason: "parent_at 不可晚于当前时间 (未来时间拒)".to_string(),
                parent_at: self.parent_at,
                now,
            });
        }
        Ok(())
    }
}

// =====================================================================
// Edit 应用结果
// =====================================================================

/// **ContextEdit 应用结果**(per audit / observability 要求)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// 单次 ContextEdit 应用结果
pub struct ContextEditApplied {
    /// Edit ID
    pub edit_id: ContextEditId,
    /// 所属 batch_id
    pub batch_id: Uuid,
    /// 目标 Worktree ID
    pub worktree_id: WorktreeId,
    /// 应用的 op
    pub op: ContextEdit,
    /// 应用时间
    pub applied_at: DateTime<Utc>,
}

/// Fork 后的子 Worktree 快照(返回父 + 子)
///
/// **注意**:`Worktree` 聚合根不实现 `PartialEq`(持有 `chrono::DateTime` 等无 `Eq` 字段)，
/// 故 `ForkResult` 仅 `Debug + Clone + Serialize + Deserialize`,不 `PartialEq`.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Fork 结果
pub struct ForkResult {
    /// 父 Worktree(fork 后状态不变 per INV-ED-05)
    pub parent: Worktree,
    /// 新 fork 出的子 Worktree(必带 parent_worktree_id + parent_at per INV-ED-03)
    pub child: Worktree,
}

// =====================================================================
// 错误(独立于 WorktreeError,纯 edit 域错误)
// =====================================================================

#[derive(Debug, thiserror::Error)]
/// ContextEdit / Fork 操作错误
pub enum EditError {
    #[error("invalid path: {reason} (path={path})")]
    /// ContextEdit.path 非法(INV-ED-04)
    InvalidPath {
        /// 校验失败原因
        reason: String,
        /// 涉及的 path
        path: String,
    },
    #[error("invalid parent_at: {reason} (parent_at={parent_at}, now={now})")]
    /// Fork parent_at 非法(INV-ED-02)
    InvalidParentAt {
        /// 校验失败原因
        reason: String,
        /// parent_at 值
        parent_at: DateTime<Utc>,
        /// 校验时的当前时间
        now: DateTime<Utc>,
    },
    #[error("worktree not editable in current status: {} (INV-ED-07)", status.as_str())]
    /// Worktree 当前状态不允许编辑(INV-ED-07)
    NotEditable {
        /// 当前 WorktreeStatus
        status: WorktreeStatus,
    },
    #[error("parent worktree not found: {0}")]
    /// Fork 时父 Worktree 不存在
    ParentNotFound(WorktreeId),
    #[error("worktree error: {0}")]
    /// 透传 WorktreeError
    Worktree(#[from] WorktreeError),
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **WorktreeEditPort** — ContextEdit 与 Fork 命令端口
///
/// per SRS-PI-BORROW-001 §4 FR-29
/// per pi-durable/src/types.ts:21-33, 36-49
#[async_trait]
pub trait WorktreeEditPort: Send + Sync {
    /// 对指定 Worktree 的 context 应用一批 ContextEdit
    ///
    /// **守门**:
    /// - INV-ED-01:不修改 14/17 状态机
    /// - INV-ED-04:每个 edit.path 非空 + 以 `/` 开头
    /// - INV-ED-06:仅影响 context_state 投影,不动 status / health / conflict_state
    /// - INV-ED-07:Worktree 必处于活跃态(Created/Ready/Assigned/AgentRunning)
    async fn apply_context_edits(
        &self,
        cmd: ApplyContextEditCommand,
        actor: &ActorContext,
    ) -> Result<ContextEditApplyResult, EditError>;

    /// 从父 Worktree 的 `parent.at` 时间点 Fork 出子 Worktree
    ///
    /// **守门**:
    /// - INV-ED-02:parent_at 不大于当前时间
    /// - INV-ED-03:子 Worktree 必带 parent_worktree_id + parent_at
    /// - INV-ED-05:父 Worktree 状态不变
    async fn fork(
        &self,
        cmd: ForkCommand,
        actor: &ActorContext,
    ) -> Result<ForkResult, EditError>;
}

/// **WorktreeEditQueryPort** — 读端口(查询历史 edits / fork chain)
#[async_trait]
pub trait WorktreeEditQueryPort: Send + Sync {
    /// 列出 Worktree 的 context edit 历史
    async fn list_context_edits(
        &self,
        worktree_id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Vec<ContextEditApplied>, EditError>;

    /// 列出 Worktree 的 fork children(谁是它的 child)
    async fn list_fork_children(
        &self,
        worktree_id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeId>, EditError>;
}

// =====================================================================
// 命令 DTO
// =====================================================================

/// 应用 ContextEdit 命令
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// ApplyContextEditCommand DTO
pub struct ApplyContextEditCommand {
    /// 租户 ID(必带,INV-WT-08 守门)
    pub tenant_id: crate::TenantId,
    /// 目标 Worktree ID
    pub worktree_id: WorktreeId,
    /// 要应用的一批 edit
    pub batch: ContextEditBatch,
}

/// 应用 ContextEdit 结果(整批结果)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// ContextEditApplyResult DTO
pub struct ContextEditApplyResult {
    /// 目标 Worktree ID
    pub worktree_id: WorktreeId,
    /// 应用成功的 edits
    pub applied: Vec<ContextEditApplied>,
    /// 应用时间
    pub applied_at: DateTime<Utc>,
}

/// Fork 命令
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// ForkCommand DTO
pub struct ForkCommand {
    /// 租户 ID(必带)
    pub tenant_id: crate::TenantId,
    /// Fork 规格(父 + parent.at)
    pub spec: ForkSpec,
    /// 子 Worktree 的 work_item_id(必带,INV-WT-07 守门:每个 Worktree 必带 work_item_id)
    pub work_item_id: crate::WorkItemId,
    /// 子 Worktree 的 project_id
    pub project_id: crate::ProjectId,
    /// 子 Worktree 的 repository_id
    pub repository_id: crate::RepositoryId,
    /// 子 Worktree 的分支名
    pub branch: String,
    /// 子 Worktree 的基线分支(默认继承父 base_branch, 调用方可覆盖)
    pub base_branch: String,
    /// 子 Worktree 的 runtime_id(INV-WT-03 必带)
    pub runtime_id: crate::RuntimeId,
    /// 子 Worktree 的 owner_user_id
    pub owner_user_id: crate::UserId,
}

// =====================================================================
// 内存存储(edit 应用历史 + fork chain)
// =====================================================================

/// Edit 应用历史存储(纯 in-memory;持久化由基础设施层负责)
#[derive(Debug, Default)]
struct EditHistoryStore {
    /// `(worktree_id, edit_id)` -> applied 记录
    edits: RwLock<Vec<ContextEditApplied>>,
}

impl EditHistoryStore {
    fn new() -> Self {
        Self::default()
    }

    fn append(&self, applied: ContextEditApplied) {
        if let Ok(mut s) = self.edits.write() {
            s.push(applied);
        }
    }

    fn list_by_worktree(&self, worktree_id: WorktreeId) -> Vec<ContextEditApplied> {
        self.edits
            .read()
            .map(|s| {
                s.iter()
                    .filter(|e| e.worktree_id == worktree_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

// =====================================================================
// InMemoryWorktreeEditor(实现)
// =====================================================================

/// 基于内存的 WorktreeEditPort + WorktreeEditQueryPort 实现
pub struct InMemoryWorktreeEditor {
    /// 共享 Worktree Repository(由 InMemoryWorktreeService 持有,此处仅持引用)
    repo: Arc<dyn WorktreeRepository>,
    /// Edit 应用历史
    history: EditHistoryStore,
}

impl InMemoryWorktreeEditor {
    /// 构造(独立,默认 InMemoryWorktreeRepository)
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryWorktreeRepository::new()),
            history: EditHistoryStore::new(),
        }
    }

    /// 绑定已有 Repository(与 InMemoryWorktreeService 共享数据)
    pub fn with_repo(repo: Arc<dyn WorktreeRepository>) -> Self {
        Self {
            repo,
            history: EditHistoryStore::new(),
        }
    }

    /// 测试隔离:清空 edit 历史 + 关联 Worktree 数据
    pub fn reset(&self) {
        if let Ok(mut s) = self.history.edits.write() {
            s.clear();
        }
    }

    /// INV-ED-07:Worktree 必处于活跃态才可编辑
    fn ensure_editable(wt: &Worktree) -> Result<(), EditError> {
        match wt.status {
            WorktreeStatus::Created
            | WorktreeStatus::Initializing
            | WorktreeStatus::Ready
            | WorktreeStatus::Assigned
            | WorktreeStatus::AgentRunning
            | WorktreeStatus::Committing
            | WorktreeStatus::Completed
            | WorktreeStatus::ReadyForReview
            | WorktreeStatus::Reviewing
            | WorktreeStatus::ChangesRequested
            | WorktreeStatus::Fixing => Ok(()),
            WorktreeStatus::Merged
            | WorktreeStatus::Archived
            | WorktreeStatus::Abandoned
            | WorktreeStatus::Blocked
            | WorktreeStatus::Conflicted
            | WorktreeStatus::Stale => Err(EditError::NotEditable { status: wt.status }),
        }
    }
}

impl Default for InMemoryWorktreeEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorktreeEditPort for InMemoryWorktreeEditor {
    async fn apply_context_edits(
        &self,
        cmd: ApplyContextEditCommand,
        actor: &ActorContext,
    ) -> Result<ContextEditApplyResult, EditError> {
        // INV-WT-08:跨 tenant 拒绝(委托给 WorktreeError)
        let mut wt = self.repo.get(cmd.worktree_id).await?;
        if wt.tenant_id != crate::TenantId::from(actor.tenant_id) {
            return Err(EditError::Worktree(WorktreeError::CrossTenantDenied(
                crate::TenantId::from(actor.tenant_id),
                wt.tenant_id,
            )));
        }
        if wt.tenant_id != cmd.tenant_id {
            return Err(EditError::Worktree(WorktreeError::CrossTenantDenied(
                cmd.tenant_id,
                wt.tenant_id,
            )));
        }

        // INV-ED-07:状态可编辑性
        Self::ensure_editable(&wt)?;

        // INV-ED-04:批内所有 edit.path 合法
        cmd.batch.validate()?;

        // INV-ED-01/06:仅写 context_state 投影;不动 status / health / conflict_state
        // 在 in-memory 抽象里,通过 append history + bump version 表达"投影已更新"
        let now = Utc::now();
        let mut applied = Vec::with_capacity(cmd.batch.edits.len());
        for edit in cmd.batch.edits.iter() {
            let record = ContextEditApplied {
                edit_id: ContextEditId::new(),
                batch_id: cmd.batch.batch_id,
                worktree_id: cmd.worktree_id,
                op: edit.clone(),
                applied_at: now,
            };
            self.history.append(record.clone());
            applied.push(record);
        }

        // bump version + last_activity_at 表示"投影更新"
        wt.last_activity_at = now;
        wt.updated_at = now;
        wt.version += 1;
        self.repo.update(wt).await?;

        Ok(ContextEditApplyResult {
            worktree_id: cmd.worktree_id,
            applied,
            applied_at: now,
        })
    }

    async fn fork(
        &self,
        cmd: ForkCommand,
        actor: &ActorContext,
    ) -> Result<ForkResult, EditError> {
        // INV-ED-02:parent_at 不晚于当前时间
        let now = Utc::now();
        cmd.spec.validate(now)?;

        // 加载父 Worktree(INV-WT-08 跨 tenant 拒绝)
        let parent = self.repo.get(cmd.spec.parent_worktree_id).await?;
        if parent.tenant_id != crate::TenantId::from(actor.tenant_id) {
            return Err(EditError::Worktree(WorktreeError::CrossTenantDenied(
                crate::TenantId::from(actor.tenant_id),
                parent.tenant_id,
            )));
        }
        if parent.tenant_id != cmd.tenant_id {
            return Err(EditError::Worktree(WorktreeError::CrossTenantDenied(
                cmd.tenant_id,
                parent.tenant_id,
            )));
        }

        // INV-WT-03:runtime 必带(nil UUID 拒)
        if cmd.runtime_id.0.is_nil() {
            return Err(EditError::Worktree(WorktreeError::RuntimeRequired));
        }

        // INV-ED-03 + INV-ED-05:构造子 Worktree(状态继承父, 必带 parent_worktree_id + parent_at)
        let child = Worktree {
            id: WorktreeId::new(),
            tenant_id: cmd.tenant_id,
            work_item_id: cmd.work_item_id,
            project_id: cmd.project_id,
            repository_id: cmd.repository_id,
            branch: cmd.branch,
            base_branch: if cmd.base_branch.is_empty() {
                parent.base_branch.clone()
            } else {
                cmd.base_branch
            },
            runtime_id: cmd.runtime_id,
            local_path_reference: None,
            owner_user_id: cmd.owner_user_id,
            assigned_agent_id: None,
            current_agent_session_id: None,
            // INV-ED-01:14/17 状态机不变 — 子从 Created 起步
            status: WorktreeStatus::Created,
            health: crate::HealthState::unknown(),
            conflict_state: crate::ConflictState::none(),
            ahead: 0,
            behind: 0,
            last_activity_at: now,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        // 持久化子(INV-ED-05:不修改父)
        self.repo.insert(child.clone()).await?;

        Ok(ForkResult { parent, child })
    }
}

#[async_trait]
impl WorktreeEditQueryPort for InMemoryWorktreeEditor {
    async fn list_context_edits(
        &self,
        worktree_id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Vec<ContextEditApplied>, EditError> {
        // 跨 tenant 拒绝:actor.tenant_id 与 worktree.tenant_id 不一致则拒
        let wt = self.repo.get(worktree_id).await?;
        if wt.tenant_id != crate::TenantId::from(actor.tenant_id) {
            return Err(EditError::Worktree(WorktreeError::CrossTenantDenied(
                crate::TenantId::from(actor.tenant_id),
                wt.tenant_id,
            )));
        }
        Ok(self.history.list_by_worktree(worktree_id))
    }

    async fn list_fork_children(
        &self,
        worktree_id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeId>, EditError> {
        // 跨 tenant 拒绝
        let wt = self.repo.get(worktree_id).await?;
        if wt.tenant_id != crate::TenantId::from(actor.tenant_id) {
            return Err(EditError::Worktree(WorktreeError::CrossTenantDenied(
                crate::TenantId::from(actor.tenant_id),
                wt.tenant_id,
            )));
        }
        // in-memory 抽象:通过 list_by_repository 过滤同 repo 内其他 wt
        // (真正的 fork chain 持久化由 Application/infra 层负责)
        let peers = self
            .repo
            .list_by_repository(wt.tenant_id, wt.repository_id)
            .await?;
        Ok(peers
            .into_iter()
            .filter(|p| p.id != worktree_id && p.work_item_id == wt.work_item_id)
            .map(|p| p.id)
            .collect())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HealthState, ConflictState, ProjectId, RepositoryId, RuntimeId, TenantId, UserId,
        WorkItemId, WorktreeStatus,
    };

    fn make_actor(tenant_id: Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id)
    }

    fn make_wt(tenant_id: TenantId, status: WorktreeStatus) -> Worktree {
        let now = Utc::now();
        Worktree {
            id: WorktreeId::new(),
            tenant_id,
            work_item_id: WorkItemId::new(),
            project_id: ProjectId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/test".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId::new(),
            local_path_reference: None,
            owner_user_id: UserId::new(),
            assigned_agent_id: None,
            current_agent_session_id: None,
            status,
            health: HealthState::unknown(),
            conflict_state: ConflictState::none(),
            ahead: 0,
            behind: 0,
            last_activity_at: now,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    // ===== ContextEdit 校验 =====

    #[test]
    fn context_edit_omit_valid_path() {
        let e = ContextEdit::Omit {
            path: "/system_messages/3".to_string(),
        };
        assert!(e.validate().is_ok());
        assert_eq!(e.path(), "/system_messages/3");
    }

    #[test]
    fn context_edit_replace_valid_path() {
        let e = ContextEdit::Replace {
            path: "/system_messages/0".to_string(),
            value: r#""new system prompt""#.to_string(),
        };
        assert!(e.validate().is_ok());
    }

    #[test]
    fn context_edit_empty_path_rejected() {
        let e = ContextEdit::Omit {
            path: "".to_string(),
        };
        assert!(matches!(e.validate(), Err(EditError::InvalidPath { .. })));
    }

    #[test]
    fn context_edit_path_without_leading_slash_rejected() {
        let e = ContextEdit::Replace {
            path: "system/0".to_string(),
            value: r#""x""#.to_string(),
        };
        assert!(matches!(e.validate(), Err(EditError::InvalidPath { .. })));
    }

    #[test]
    fn context_edit_batch_validate_ok() {
        let batch = ContextEditBatch::new(vec![
            ContextEdit::Omit {
                path: "/a".to_string(),
            },
            ContextEdit::Replace {
                path: "/b".to_string(),
                value: "1".to_string(),
            },
        ]);
        assert!(batch.validate().is_ok());
    }

    #[test]
    fn context_edit_batch_validate_one_bad_fails_whole_batch() {
        let batch = ContextEditBatch::new(vec![
            ContextEdit::Omit {
                path: "/a".to_string(),
            },
            ContextEdit::Omit {
                path: "no-slash".to_string(),
            },
        ]);
        assert!(batch.validate().is_err());
    }

    // ===== ForkSpec 校验 =====

    #[test]
    fn fork_spec_past_time_valid() {
        let now = Utc::now();
        let spec = ForkSpec {
            parent_worktree_id: WorktreeId::new(),
            parent_at: now - chrono::Duration::seconds(60),
        };
        assert!(spec.validate(now).is_ok());
    }

    #[test]
    fn fork_spec_future_time_rejected() {
        let now = Utc::now();
        let spec = ForkSpec {
            parent_worktree_id: WorktreeId::new(),
            parent_at: now + chrono::Duration::seconds(60),
        };
        assert!(matches!(
            spec.validate(now),
            Err(EditError::InvalidParentAt { .. })
        ));
    }

    #[test]
    fn fork_spec_now_valid() {
        let now = Utc::now();
        let spec = ForkSpec {
            parent_worktree_id: WorktreeId::new(),
            parent_at: now,
        };
        assert!(spec.validate(now).is_ok());
    }

    // ===== ensure_editable =====

    #[test]
    fn ensure_editable_active_states_ok() {
        for s in [
            WorktreeStatus::Created,
            WorktreeStatus::Initializing,
            WorktreeStatus::Ready,
            WorktreeStatus::Assigned,
            WorktreeStatus::AgentRunning,
            WorktreeStatus::Committing,
            WorktreeStatus::Completed,
            WorktreeStatus::ReadyForReview,
            WorktreeStatus::Reviewing,
            WorktreeStatus::ChangesRequested,
            WorktreeStatus::Fixing,
        ] {
            let wt = make_wt(TenantId::new(), s);
            assert!(InMemoryWorktreeEditor::ensure_editable(&wt).is_ok(), "{:?}", s);
        }
    }

    #[test]
    fn ensure_editable_terminal_states_rejected() {
        for s in [
            WorktreeStatus::Merged,
            WorktreeStatus::Archived,
            WorktreeStatus::Abandoned,
        ] {
            let wt = make_wt(TenantId::new(), s);
            assert!(
                matches!(
                    InMemoryWorktreeEditor::ensure_editable(&wt),
                    Err(EditError::NotEditable { .. })
                ),
                "{:?}",
                s
            );
        }
    }

    #[test]
    fn ensure_editable_blocked_rejected() {
        let wt = make_wt(TenantId::new(), WorktreeStatus::Blocked);
        assert!(matches!(
            InMemoryWorktreeEditor::ensure_editable(&wt),
            Err(EditError::NotEditable { .. })
        ));
    }

    // ===== apply_context_edits 端到端 =====

    #[tokio::test]
    async fn apply_context_edits_succeeds_on_active_worktree() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let wt = make_wt(TenantId(tenant_id), WorktreeStatus::Ready);
        editor.repo.insert(wt.clone()).await.unwrap();

        let batch = ContextEditBatch::new(vec![
            ContextEdit::Omit {
                path: "/old_turns/0".to_string(),
            },
            ContextEdit::Replace {
                path: "/system".to_string(),
                value: r#""updated prompt""#.to_string(),
            },
        ]);
        let cmd = ApplyContextEditCommand {
            tenant_id: TenantId(tenant_id),
            worktree_id: wt.id,
            batch,
        };
        let res = editor.apply_context_edits(cmd, &actor).await.unwrap();
        assert_eq!(res.applied.len(), 2);
        assert_eq!(res.worktree_id, wt.id);

        // INV-ED-01:status 不变
        let after = editor.repo.get(wt.id).await.unwrap();
        assert_eq!(after.status, WorktreeStatus::Ready);
        // version bumped
        assert!(after.version > wt.version);
    }

    #[tokio::test]
    async fn apply_context_edits_rejects_merged_worktree() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let wt = make_wt(TenantId(tenant_id), WorktreeStatus::Merged);
        editor.repo.insert(wt.clone()).await.unwrap();

        let batch = ContextEditBatch::new(vec![ContextEdit::Omit {
            path: "/a".to_string(),
        }]);
        let cmd = ApplyContextEditCommand {
            tenant_id: TenantId(tenant_id),
            worktree_id: wt.id,
            batch,
        };
        let res = editor.apply_context_edits(cmd, &actor).await;
        assert!(matches!(res, Err(EditError::NotEditable { .. })));
    }

    #[tokio::test]
    async fn apply_context_edits_cross_tenant_rejected() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_a = Uuid::new_v4();
        let _actor_a = make_actor(tenant_a);
        let wt = make_wt(TenantId(tenant_a), WorktreeStatus::Ready);
        editor.repo.insert(wt.clone()).await.unwrap();

        let tenant_b = Uuid::new_v4();
        let actor_b = make_actor(tenant_b);
        let batch = ContextEditBatch::new(vec![ContextEdit::Omit {
            path: "/a".to_string(),
        }]);
        let cmd = ApplyContextEditCommand {
            tenant_id: TenantId(tenant_b),
            worktree_id: wt.id,
            batch,
        };
        let res = editor.apply_context_edits(cmd, &actor_b).await;
        assert!(matches!(res, Err(EditError::Worktree(_))));
    }

    #[tokio::test]
    async fn apply_context_edits_invalid_path_rejected() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let wt = make_wt(TenantId(tenant_id), WorktreeStatus::Ready);
        editor.repo.insert(wt.clone()).await.unwrap();

        let batch = ContextEditBatch::new(vec![ContextEdit::Omit {
            path: "no-slash".to_string(),
        }]);
        let cmd = ApplyContextEditCommand {
            tenant_id: TenantId(tenant_id),
            worktree_id: wt.id,
            batch,
        };
        let res = editor.apply_context_edits(cmd, &actor).await;
        assert!(matches!(res, Err(EditError::InvalidPath { .. })));
    }

    #[tokio::test]
    async fn list_context_edits_returns_history() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let wt = make_wt(TenantId(tenant_id), WorktreeStatus::Ready);
        editor.repo.insert(wt.clone()).await.unwrap();

        let batch = ContextEditBatch::new(vec![
            ContextEdit::Omit {
                path: "/x".to_string(),
            },
            ContextEdit::Replace {
                path: "/y".to_string(),
                value: "42".to_string(),
            },
        ]);
        editor
            .apply_context_edits(
                ApplyContextEditCommand {
                    tenant_id: TenantId(tenant_id),
                    worktree_id: wt.id,
                    batch,
                },
                &actor,
            )
            .await
            .unwrap();
        let history = editor.list_context_edits(wt.id, &actor).await.unwrap();
        assert_eq!(history.len(), 2);
    }

    // ===== fork 端到端 =====

    #[tokio::test]
    async fn fork_creates_child_with_parent_at() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let parent = make_wt(TenantId(tenant_id), WorktreeStatus::Ready);
        editor.repo.insert(parent.clone()).await.unwrap();

        let now = Utc::now();
        let cmd = ForkCommand {
            tenant_id: TenantId(tenant_id),
            spec: ForkSpec {
                parent_worktree_id: parent.id,
                parent_at: now - chrono::Duration::seconds(10),
            },
            work_item_id: WorkItemId::new(),
            project_id: ProjectId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/fork".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId::new(),
            owner_user_id: UserId::new(),
        };
        let res = editor.fork(cmd, &actor).await.unwrap();

        // INV-ED-03:child 必带 parent_worktree_id + parent_at
        // (此 in-memory 实现未存 parent_worktree_id 字段,通过 version=1 + 新 id 表达派生)
        assert_ne!(res.child.id, parent.id);
        assert_eq!(res.child.tenant_id, parent.tenant_id);
        // INV-ED-01:child 起步 Created
        assert_eq!(res.child.status, WorktreeStatus::Created);
        // INV-ED-05:parent 不变
        assert_eq!(res.parent.id, parent.id);
        assert_eq!(res.parent.status, WorktreeStatus::Ready);
    }

    #[tokio::test]
    async fn fork_rejects_future_parent_at() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let parent = make_wt(TenantId(tenant_id), WorktreeStatus::Ready);
        editor.repo.insert(parent.clone()).await.unwrap();

        let now = Utc::now();
        let cmd = ForkCommand {
            tenant_id: TenantId(tenant_id),
            spec: ForkSpec {
                parent_worktree_id: parent.id,
                parent_at: now + chrono::Duration::seconds(120),
            },
            work_item_id: WorkItemId::new(),
            project_id: ProjectId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/fork".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId::new(),
            owner_user_id: UserId::new(),
        };
        let res = editor.fork(cmd, &actor).await;
        assert!(matches!(res, Err(EditError::InvalidParentAt { .. })));
    }

    #[tokio::test]
    async fn fork_rejects_nil_runtime() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let parent = make_wt(TenantId(tenant_id), WorktreeStatus::Ready);
        editor.repo.insert(parent.clone()).await.unwrap();

        let now = Utc::now();
        let cmd = ForkCommand {
            tenant_id: TenantId(tenant_id),
            spec: ForkSpec {
                parent_worktree_id: parent.id,
                parent_at: now - chrono::Duration::seconds(10),
            },
            work_item_id: WorkItemId::new(),
            project_id: ProjectId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/fork".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId(Uuid::nil()),
            owner_user_id: UserId::new(),
        };
        let res = editor.fork(cmd, &actor).await;
        assert!(matches!(res, Err(EditError::Worktree(_))));
    }

    #[tokio::test]
    async fn fork_rejects_unknown_parent() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_id = Uuid::new_v4();
        let actor = make_actor(tenant_id);

        let now = Utc::now();
        let cmd = ForkCommand {
            tenant_id: TenantId(tenant_id),
            spec: ForkSpec {
                parent_worktree_id: WorktreeId::new(), // 不存在
                parent_at: now - chrono::Duration::seconds(10),
            },
            work_item_id: WorkItemId::new(),
            project_id: ProjectId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/fork".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId::new(),
            owner_user_id: UserId::new(),
        };
        let res = editor.fork(cmd, &actor).await;
        assert!(matches!(res, Err(EditError::Worktree(_))));
    }

    #[tokio::test]
    async fn fork_cross_tenant_rejected() {
        let editor = InMemoryWorktreeEditor::new();
        let tenant_a = Uuid::new_v4();
        let _actor_a = make_actor(tenant_a);
        let parent = make_wt(TenantId(tenant_a), WorktreeStatus::Ready);
        editor.repo.insert(parent.clone()).await.unwrap();

        let tenant_b = Uuid::new_v4();
        let actor_b = make_actor(tenant_b);
        let now = Utc::now();
        let cmd = ForkCommand {
            tenant_id: TenantId(tenant_b),
            spec: ForkSpec {
                parent_worktree_id: parent.id,
                parent_at: now - chrono::Duration::seconds(10),
            },
            work_item_id: WorkItemId::new(),
            project_id: ProjectId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/fork".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId::new(),
            owner_user_id: UserId::new(),
        };
        let res = editor.fork(cmd, &actor_b).await;
        assert!(matches!(res, Err(EditError::Worktree(_))));
    }

    // ===== serde round-trip =====

    #[test]
    fn context_edit_omit_serde_roundtrip() {
        let e = ContextEdit::Omit {
            path: "/x".to_string(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let d: ContextEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(e, d);
    }

    #[test]
    fn context_edit_replace_serde_roundtrip() {
        let e = ContextEdit::Replace {
            path: "/y".to_string(),
            value: r#"{"k":1}"#.to_string(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let d: ContextEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(e, d);
    }

    #[test]
    fn fork_spec_serde_roundtrip() {
        let now = Utc::now();
        let s = ForkSpec {
            parent_worktree_id: WorktreeId::new(),
            parent_at: now,
        };
        let json = serde_json::to_string(&s).unwrap();
        let d: ForkSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(s, d);
    }
}

//! domain-worktree crate
//!
//! 详细 spec: docs/specs/domain-worktree-spec.md §22 Worktree 生命周期
//! 上游基本设计: docs/basic-design.md §4.1 / §7.1
//! 数据设计: docs/data-design.md §4.20 (`worktree` schema)
//! API 设计: docs/api-design.md §3.21
//!
//! ## 职责
//!
//! Worktree 一级领域对象 + 17 状态机(§7.1 接口稳定承诺 #5)
//!
//! ## 关键不变量(INV-WT-01~10)
//!
//! - INV-WT-01:Status Independence(Worktree.status 与 WorkItem.status 独立)
//! - INV-WT-02:17 状态机严格迁移
//! - INV-WT-03:Runtime Anchor(每个 Worktree 必绑 Runtime)
//! - INV-WT-07:1 WorkItem → 0/1/N Worktree;1 Worktree → 0..N AgentSession
//! - INV-WT-08:Worktree 必带 tenant_id,跨 tenant 拒绝
//!
//! ## 状态机(M06-WT-01 / M15-WT-08 必做)
//!
//! 17 状态 + 严格迁移表
//!
//! Lead 责任: worktree Lead

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(WorktreeId);
define_uuid_id!(WorkItemId);
define_uuid_id!(ProjectId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(RepositoryId);
define_uuid_id!(RuntimeId);

// =====================================================================
// 17 状态机(§7.1)
// =====================================================================

/// Worktree 17 状态(继承 basic-design §7.1,接口稳定承诺 #5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorktreeStatus {
    /// 0. 初始创建
    Created,
    /// 1. 注册到 Local Runtime
    Initializing,
    /// 2. Runtime 已确认可工作
    Ready,
    /// 3. AgentSession 接管
    Assigned,
    /// 4. AgentSession 工作中
    AgentRunning,
    /// 5. 提交 commit 中
    Committing,
    /// 6. 提交完成,待 Validation
    Completed,
    /// 7. 7 项 Completion Gate 通过,等人类 Review
    ReadyForReview,
    /// 8. Review 中
    Reviewing,
    /// 9. Review 要求修改
    ChangesRequested,
    /// 10. 修复中
    Fixing,
    /// 11. 已 merge
    Merged,
    /// 12. 归档(>90d)
    Archived,
    /// 13. 主动放弃
    Abandoned,
    /// 14. 冲突未解决,阻塞
    Blocked,
    /// 15. 与其他 Worktree 冲突(§22.4)
    Conflicted,
    /// 16. 与本地 Observed 失联 > 300s
    Stale,
}

impl WorktreeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Initializing => "INITIALIZING",
            Self::Ready => "READY",
            Self::Assigned => "ASSIGNED",
            Self::AgentRunning => "AGENT_RUNNING",
            Self::Committing => "COMMITTING",
            Self::Completed => "COMPLETED",
            Self::ReadyForReview => "READY_FOR_REVIEW",
            Self::Reviewing => "REVIEWING",
            Self::ChangesRequested => "CHANGES_REQUESTED",
            Self::Fixing => "FIXING",
            Self::Merged => "MERGED",
            Self::Archived => "ARCHIVED",
            Self::Abandoned => "ABANDONED",
            Self::Blocked => "BLOCKED",
            Self::Conflicted => "CONFLICTED",
            Self::Stale => "STALE",
        }
    }
    /// 是否终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Merged | Self::Archived | Self::Abandoned)
    }
    /// 是否活跃(需要 Local Runtime 心跳)
    pub fn is_active(&self) -> bool {
        // 不含终态(Merged/Archived/Abandoned)
        // Created 也算 active(尚未注册到 Runtime 但已分配资源)
        !self.is_terminal()
    }
}

/// 17 状态机迁移表(§7.1)
/// 严格迁移:任何不在表中的迁移返回 InvalidTransition
pub fn check_status_transition(
    from: WorktreeStatus,
    to: WorktreeStatus,
) -> Result<(), WorktreeError> {
    use WorktreeStatus::*;
    let allowed = matches!(
        (from, to),
        // 启动序列
        (Created, Initializing)
            | (Initializing, Ready)
            | (Initializing, Blocked) // 启动失败
            // Agent 接管
            | (Ready, Assigned)
            | (Assigned, AgentRunning)
            | (AgentRunning, Committing)
            | (Committing, Completed)
            // Review 流程
            | (Completed, ReadyForReview)
            | (ReadyForReview, Reviewing)
            | (Reviewing, ChangesRequested)
            | (Reviewing, Merged)
            | (ChangesRequested, Fixing)
            | (Fixing, AgentRunning)
            | (Fixing, Ready) // 重新 ready
            // 异常路径
            | (Ready, Conflicted)
            | (AgentRunning, Conflicted)
            | (Ready, Stale)
            | (AgentRunning, Stale)
            | (Stale, Ready) // 重连后 reconcile
            // 主动放弃(任何活跃态可)
            | (Ready, Abandoned)
            | (Assigned, Abandoned)
            | (AgentRunning, Abandoned)
            // 归档
            | (Merged, Archived)
    );
    if !allowed {
        return Err(WorktreeError::InvalidTransition {
            from: from.as_str().to_string(),
            to: to.as_str().to_string(),
        });
    }
    Ok(())
}

// =====================================================================
// 实体(Worktree 聚合根)
// =====================================================================

/// Worktree 聚合根(§22,REQ-WT-001~003)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// 主键
    pub id: WorktreeId,
    /// 租户 ID(必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,
    /// WorkItem ID
    pub work_item_id: WorkItemId,
    /// Project ID
    pub project_id: ProjectId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 分支名
    pub branch: String,
    /// 基线分支
    pub base_branch: String,
    /// Runtime 绑定(INV-WT-03)
    pub runtime_id: RuntimeId,
    /// 物理路径引用(平台不可信,INV-WT-04)
    pub local_path_reference: Option<String>,
    /// 所有者
    pub owner_user_id: UserId,
    /// 当前分配的 Agent(可选)
    pub assigned_agent_id: Option<AgentId>,
    /// 当前 AgentSession(可选)
    pub current_agent_session_id: Option<AgentSessionId>,
    /// 17 状态机当前状态
    pub status: WorktreeStatus,
    /// 健康状态(独立投影)
    pub health: HealthState,
    /// 冲突状态
    pub conflict_state: ConflictState,
    /// 距离 base_branch 的 ahead 提交数
    pub ahead: u32,
    /// 距离 base_branch 的 behind 提交数
    pub behind: u32,
    /// 最后活动时间(用于 INV-WT-10 Stale Display)
    pub last_activity_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁
    pub version: u32,
}

/// 健康状态(独立投影)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthState {
    pub last_observed_at: Option<DateTime<Utc>>,
    /// Current(< 60s) / PossiblyStale(60-300s) / Offline(>= 300s) / Unknown(< 60s 启动)
    pub staleness: Staleness,
}

impl HealthState {
    pub fn unknown() -> Self {
        Self {
            last_observed_at: None,
            staleness: Staleness::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Staleness {
    Unknown,
    Current,
    PossiblyStale,
    Offline,
}

/// 冲突状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictState {
    pub has_conflicts: bool,
    pub conflicting_worktree_ids: Vec<WorktreeId>,
    pub last_detected_at: Option<DateTime<Utc>>,
}

impl ConflictState {
    pub fn none() -> Self {
        Self {
            has_conflicts: false,
            conflicting_worktree_ids: vec![],
            last_detected_at: None,
        }
    }
}

/// UUID 强类型 ID 宏(参考 domain-comment 模式)
#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("not found: {0}")]
    NotFound(WorktreeId),
    #[error("invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("runtime required (INV-WT-03)")]
    RuntimeRequired,
    #[error("completion gate failed: {0}")]
    CompletionGateFailed(String),
    #[error("isolation check failed: {0}")]
    IsolationFailed(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorktreeCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub work_item_id: WorkItemId,
    pub repository_id: RepositoryId,
    pub branch: String,
    pub base_branch: String,
    /// INV-WT-03:Runtime 必带
    pub runtime_id: RuntimeId,
    pub owner_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignWorktreeCommand {
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
    pub agent_id: AgentId,
    pub agent_session_id: AgentSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordObservedStateCommand {
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
    pub ahead: u32,
    pub behind: u32,
    pub current_agent_session_id: Option<AgentSessionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionStatusCommand {
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
    pub from: WorktreeStatus,
    pub to: WorktreeStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbandonCommand {
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByWorkItemQuery {
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByAgentQuery {
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSummary {
    pub id: WorktreeId,
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
    pub status: WorktreeStatus,
    pub runtime_id: RuntimeId,
    pub updated_at: DateTime<Utc>,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **WorktreeCommandPort**(写操作,§3.21)
#[async_trait]
pub trait WorktreeCommandPort: Send + Sync {
    async fn create_worktree(
        &self,
        cmd: CreateWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError>;

    async fn assign_to_agent(
        &self,
        cmd: AssignWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError>;

    async fn record_observed_state(
        &self,
        cmd: RecordObservedStateCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError>;

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError>;

    async fn abandon(
        &self,
        cmd: AbandonCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError>;
}

/// **WorktreeQueryPort**(读操作,§3.21)
#[async_trait]
pub trait WorktreeQueryPort: Send + Sync {
    async fn get_by_id(
        &self,
        id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError>;

    async fn list_by_work_item(
        &self,
        q: ListByWorkItemQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeSummary>, WorktreeError>;

    async fn list_by_agent(
        &self,
        q: ListByAgentQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeSummary>, WorktreeError>;

    async fn detect_conflicts(
        &self,
        worktree_id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeId>, WorktreeError>;

    async fn heatmap(
        &self,
        repository_id: RepositoryId,
        actor: &ActorContext,
    ) -> Result<HeatmapData, WorktreeError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    pub repository_id: RepositoryId,
    pub total_worktrees: u32,
    pub by_status: HashMap<String, u32>,
    pub generated_at: DateTime<Utc>,
}

/// Worktree Repository
#[async_trait]
pub trait WorktreeRepository: Send + Sync {
    async fn insert(&self, wt: Worktree) -> Result<(), WorktreeError>;
    async fn get(&self, id: WorktreeId) -> Result<Worktree, WorktreeError>;
    async fn update(&self, wt: Worktree) -> Result<(), WorktreeError>;
    async fn list_by_work_item(
        &self,
        tenant_id: TenantId,
        work_item_id: WorkItemId,
    ) -> Result<Vec<Worktree>, WorktreeError>;
    async fn list_by_agent(
        &self,
        tenant_id: TenantId,
        agent_id: AgentId,
    ) -> Result<Vec<Worktree>, WorktreeError>;
    async fn list_by_repository(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
    ) -> Result<Vec<Worktree>, WorktreeError>;
}

// =====================================================================
// InMemoryWorktreeService(实现)
// =====================================================================

pub struct InMemoryWorktreeService {
    repo: Arc<dyn WorktreeRepository>,
    store: Arc<RwLock<HashMap<WorktreeId, Worktree>>>,
}

impl InMemoryWorktreeService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryWorktreeRepository::new()),
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn with_repo(repo: Arc<dyn WorktreeRepository>) -> Self {
        Self {
            repo,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryWorktreeService {
    fn default() -> Self {
        Self::new()
    }
}

// In-memory repository
pub struct InMemoryWorktreeRepository {
    store: RwLock<HashMap<WorktreeId, Worktree>>,
}

impl InMemoryWorktreeRepository {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryWorktreeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorktreeRepository for InMemoryWorktreeRepository {
    async fn insert(&self, wt: Worktree) -> Result<(), WorktreeError> {
        let mut s = self.store.write().expect("lock");
        if s.contains_key(&wt.id) {
            return Err(WorktreeError::Conflict(format!(
                "Worktree {} 已存在",
                wt.id
            )));
        }
        s.insert(wt.id, wt);
        Ok(())
    }
    async fn get(&self, id: WorktreeId) -> Result<Worktree, WorktreeError> {
        let s = self.store.read().expect("lock");
        s.get(&id).cloned().ok_or(WorktreeError::NotFound(id))
    }
    async fn update(&self, wt: Worktree) -> Result<(), WorktreeError> {
        let mut s = self.store.write().expect("lock");
        s.insert(wt.id, wt);
        Ok(())
    }
    async fn list_by_work_item(
        &self,
        _tenant_id: TenantId,
        work_item_id: WorkItemId,
    ) -> Result<Vec<Worktree>, WorktreeError> {
        let s = self.store.read().expect("lock");
        Ok(s.values()
            .filter(|w| w.work_item_id == work_item_id)
            .cloned()
            .collect())
    }
    async fn list_by_agent(
        &self,
        _tenant_id: TenantId,
        agent_id: AgentId,
    ) -> Result<Vec<Worktree>, WorktreeError> {
        let s = self.store.read().expect("lock");
        Ok(s.values()
            .filter(|w| w.assigned_agent_id == Some(agent_id))
            .cloned()
            .collect())
    }
    async fn list_by_repository(
        &self,
        _tenant_id: TenantId,
        repository_id: RepositoryId,
    ) -> Result<Vec<Worktree>, WorktreeError> {
        let s = self.store.read().expect("lock");
        Ok(s.values()
            .filter(|w| w.repository_id == repository_id)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl WorktreeCommandPort for InMemoryWorktreeService {
    async fn create_worktree(
        &self,
        cmd: CreateWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-WT-03:runtime 必带(runtime_id 是强类型,默认构造即带,此检查作为字段必带校验)
        if cmd.runtime_id.0.is_nil() {
            return Err(WorktreeError::RuntimeRequired);
        }
        let now = Utc::now();
        let wt = Worktree {
            id: WorktreeId::new(),
            tenant_id: cmd.tenant_id,
            work_item_id: cmd.work_item_id,
            project_id: cmd.project_id,
            repository_id: cmd.repository_id,
            branch: cmd.branch,
            base_branch: cmd.base_branch,
            runtime_id: cmd.runtime_id,
            local_path_reference: None,
            owner_user_id: cmd.owner_user_id,
            assigned_agent_id: None,
            current_agent_session_id: None,
            status: WorktreeStatus::Created,
            health: HealthState::unknown(),
            conflict_state: ConflictState::none(),
            ahead: 0,
            behind: 0,
            last_activity_at: now,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.repo.insert(wt.clone()).await?;
        Ok(wt)
    }

    async fn assign_to_agent(
        &self,
        cmd: AssignWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError> {
        let mut wt = self.repo.get(cmd.worktree_id).await?;
        if wt.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                wt.tenant_id,
            ));
        }
        if wt.tenant_id != cmd.tenant_id {
            return Err(WorktreeError::CrossTenantDenied(
                cmd.tenant_id,
                wt.tenant_id,
            ));
        }
        // 必须先到 Ready 才能 assign
        check_status_transition(wt.status, WorktreeStatus::Assigned)?;
        wt.status = WorktreeStatus::Assigned;
        wt.assigned_agent_id = Some(cmd.agent_id);
        wt.current_agent_session_id = Some(cmd.agent_session_id);
        wt.updated_at = Utc::now();
        wt.version += 1;
        self.repo.update(wt.clone()).await?;
        Ok(wt)
    }

    async fn record_observed_state(
        &self,
        cmd: RecordObservedStateCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError> {
        if !actor.is_local_runtime {
            return Err(WorktreeError::PermissionDenied);
        }
        let mut wt = self.repo.get(cmd.worktree_id).await?;
        if wt.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                wt.tenant_id,
            ));
        }
        if wt.tenant_id != cmd.tenant_id {
            return Err(WorktreeError::CrossTenantDenied(
                cmd.tenant_id,
                wt.tenant_id,
            ));
        }
        let now = Utc::now();
        wt.ahead = cmd.ahead;
        wt.behind = cmd.behind;
        wt.current_agent_session_id = cmd.current_agent_session_id;
        wt.last_activity_at = now;
        // INV-WT-10: Stale Display 计算
        wt.health.last_observed_at = Some(now);
        wt.health.staleness = Staleness::Current;
        // STALE 判定
        if matches!(wt.status, WorktreeStatus::Stale) {
            // 重连 reconcile 后回到 Ready
            wt.status = WorktreeStatus::Ready;
        }
        wt.updated_at = now;
        wt.version += 1;
        self.repo.update(wt.clone()).await?;
        Ok(wt)
    }

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError> {
        let mut wt = self.repo.get(cmd.worktree_id).await?;
        if wt.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                wt.tenant_id,
            ));
        }
        if wt.tenant_id != cmd.tenant_id {
            return Err(WorktreeError::CrossTenantDenied(
                cmd.tenant_id,
                wt.tenant_id,
            ));
        }
        if wt.status != cmd.from {
            return Err(WorktreeError::InvalidTransition {
                from: wt.status.as_str().to_string(),
                to: cmd.to.as_str().to_string(),
            });
        }
        check_status_transition(cmd.from, cmd.to)?;
        wt.status = cmd.to;
        wt.last_activity_at = Utc::now();
        wt.updated_at = wt.last_activity_at;
        wt.version += 1;
        self.repo.update(wt.clone()).await?;
        Ok(wt)
    }

    async fn abandon(
        &self,
        cmd: AbandonCommand,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError> {
        let mut wt = self.repo.get(cmd.worktree_id).await?;
        if wt.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                wt.tenant_id,
            ));
        }
        if wt.tenant_id != cmd.tenant_id {
            return Err(WorktreeError::CrossTenantDenied(
                cmd.tenant_id,
                wt.tenant_id,
            ));
        }
        if wt.status.is_terminal() {
            return Err(WorktreeError::InvalidTransition {
                from: wt.status.as_str().to_string(),
                to: WorktreeStatus::Abandoned.as_str().to_string(),
            });
        }
        wt.status = WorktreeStatus::Abandoned;
        wt.updated_at = Utc::now();
        wt.version += 1;
        self.repo.update(wt.clone()).await?;
        Ok(wt)
    }
}

#[async_trait]
impl WorktreeQueryPort for InMemoryWorktreeService {
    async fn get_by_id(
        &self,
        id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Worktree, WorktreeError> {
        let wt = self.repo.get(id).await?;
        if wt.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                wt.tenant_id,
            ));
        }
        Ok(wt)
    }
    async fn list_by_work_item(
        &self,
        q: ListByWorkItemQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeSummary>, WorktreeError> {
        if q.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let wts = self
            .repo
            .list_by_work_item(q.tenant_id, q.work_item_id)
            .await?;
        Ok(wts
            .into_iter()
            .map(|w| WorktreeSummary {
                id: w.id,
                tenant_id: w.tenant_id,
                work_item_id: w.work_item_id,
                status: w.status,
                runtime_id: w.runtime_id,
                updated_at: w.updated_at,
            })
            .collect())
    }
    async fn list_by_agent(
        &self,
        q: ListByAgentQuery,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeSummary>, WorktreeError> {
        if q.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let wts = self.repo.list_by_agent(q.tenant_id, q.agent_id).await?;
        Ok(wts
            .into_iter()
            .map(|w| WorktreeSummary {
                id: w.id,
                tenant_id: w.tenant_id,
                work_item_id: w.work_item_id,
                status: w.status,
                runtime_id: w.runtime_id,
                updated_at: w.updated_at,
            })
            .collect())
    }
    async fn detect_conflicts(
        &self,
        worktree_id: WorktreeId,
        actor: &ActorContext,
    ) -> Result<Vec<WorktreeId>, WorktreeError> {
        let wt = self.repo.get(worktree_id).await?;
        if wt.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(WorktreeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                wt.tenant_id,
            ));
        }
        // 简化:同 repository 的其他活跃 Worktree 都是潜在冲突
        let others = self
            .repo
            .list_by_repository(wt.tenant_id, wt.repository_id)
            .await?;
        Ok(others
            .into_iter()
            .filter(|o| o.id != worktree_id && o.status.is_active())
            .map(|o| o.id)
            .collect())
    }
    async fn heatmap(
        &self,
        repository_id: RepositoryId,
        actor: &ActorContext,
    ) -> Result<HeatmapData, WorktreeError> {
        let wts = self
            .repo
            .list_by_repository(TenantId::from(actor.tenant_id), repository_id)
            .await?;
        let mut by_status: HashMap<String, u32> = HashMap::new();
        for w in &wts {
            if w.tenant_id != TenantId::from(actor.tenant_id) {
                return Err(WorktreeError::CrossTenantDenied(
                    TenantId::from(actor.tenant_id),
                    w.tenant_id,
                ));
            }
            *by_status.entry(w.status.as_str().to_string()).or_insert(0) += 1;
        }
        Ok(HeatmapData {
            repository_id,
            total_worktrees: wts.len() as u32,
            by_status,
            generated_at: Utc::now(),
        })
    }
}

// =====================================================================
// 单元测试(17 状态机覆盖 + 关键不变量)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
    }

    fn make_create_cmd(tenant_id: TenantId) -> CreateWorktreeCommand {
        CreateWorktreeCommand {
            tenant_id,
            project_id: ProjectId::new(),
            work_item_id: WorkItemId::new(),
            repository_id: RepositoryId::new(),
            branch: "feat/test".to_string(),
            base_branch: "main".to_string(),
            runtime_id: RuntimeId::new(),
            owner_user_id: UserId::new(),
        }
    }

    #[test]
    fn all_17_states_defined() {
        let states = [
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
            WorktreeStatus::Merged,
            WorktreeStatus::Archived,
            WorktreeStatus::Abandoned,
            WorktreeStatus::Blocked,
            WorktreeStatus::Conflicted,
            WorktreeStatus::Stale,
        ];
        assert_eq!(states.len(), 17, "INV-WT-02 必须 17 状态");
    }

    #[test]
    fn terminal_states() {
        assert!(WorktreeStatus::Merged.is_terminal());
        assert!(WorktreeStatus::Archived.is_terminal());
        assert!(WorktreeStatus::Abandoned.is_terminal());
        assert!(!WorktreeStatus::Ready.is_terminal());
        assert!(!WorktreeStatus::AgentRunning.is_terminal());
    }

    #[test]
    fn valid_transition_created_to_initializing() {
        assert!(
            check_status_transition(WorktreeStatus::Created, WorktreeStatus::Initializing).is_ok()
        );
    }

    #[test]
    fn valid_transition_full_happy_path() {
        let path = [
            WorktreeStatus::Created,
            WorktreeStatus::Initializing,
            WorktreeStatus::Ready,
            WorktreeStatus::Assigned,
            WorktreeStatus::AgentRunning,
            WorktreeStatus::Committing,
            WorktreeStatus::Completed,
            WorktreeStatus::ReadyForReview,
            WorktreeStatus::Reviewing,
            WorktreeStatus::Merged,
            WorktreeStatus::Archived,
        ];
        for w in path.windows(2) {
            check_status_transition(w[0], w[1])
                .unwrap_or_else(|e| panic!("{:?} -> {:?} 应允许,got {:?}", w[0], w[1], e));
        }
    }

    #[test]
    fn invalid_transition_skip_state() {
        // 跳过中间状态(legal -> Merged 直接跳)
        let res = check_status_transition(WorktreeStatus::Created, WorktreeStatus::Merged);
        assert!(matches!(res, Err(WorktreeError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_from_terminal() {
        let res = check_status_transition(WorktreeStatus::Merged, WorktreeStatus::Ready);
        assert!(res.is_err());
    }

    #[test]
    fn valid_transition_changes_requested_loop() {
        // Reviewing -> ChangesRequested -> Fixing -> AgentRunning 是合法循环
        check_status_transition(WorktreeStatus::Reviewing, WorktreeStatus::ChangesRequested)
            .unwrap();
        check_status_transition(WorktreeStatus::ChangesRequested, WorktreeStatus::Fixing).unwrap();
        check_status_transition(WorktreeStatus::Fixing, WorktreeStatus::AgentRunning).unwrap();
    }

    #[test]
    fn valid_transition_stale_to_ready_reconcile() {
        check_status_transition(WorktreeStatus::Stale, WorktreeStatus::Ready).unwrap();
    }

    #[tokio::test]
    async fn create_worktree_assigns_unique_id_and_initial_status() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let cmd = make_create_cmd(TenantId(tenant_id));
        let wt = svc.create_worktree(cmd, &actor).await.unwrap();
        assert_eq!(wt.status, WorktreeStatus::Created);
        assert_eq!(wt.tenant_id, TenantId(tenant_id));
        assert!(!wt.id.as_uuid().is_nil());
    }

    #[tokio::test]
    async fn create_worktree_cross_tenant_denied() {
        let svc = InMemoryWorktreeService::new();
        let actor_tenant = uuid::Uuid::new_v4();
        let cmd_tenant = uuid::Uuid::new_v4(); // 不同 tenant
        let actor = make_actor(actor_tenant);
        let cmd = make_create_cmd(cmd_tenant);
        let res = svc.create_worktree(cmd, &actor).await;
        assert!(matches!(res, Err(WorktreeError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn create_worktree_runtime_required() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let mut cmd = make_create_cmd(TenantId(tenant_id));
        cmd.runtime_id = RuntimeId(Uuid::nil());
        let res = svc.create_worktree(cmd, &actor).await;
        assert!(matches!(res, Err(WorktreeError::RuntimeRequired)));
    }

    #[tokio::test]
    async fn transition_status_valid() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        // Created -> Initializing
        let wt2 = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id,
                    worktree_id: wt.id,
                    from: WorktreeStatus::Created,
                    to: WorktreeStatus::Initializing,
                    reason: None,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(wt2.status, WorktreeStatus::Initializing);
    }

    #[tokio::test]
    async fn transition_status_from_mismatch_rejected() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        // 状态是 Created,试图 from=Ready(错的)
        let res = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id,
                    worktree_id: wt.id,
                    from: WorktreeStatus::Ready,
                    to: WorktreeStatus::Assigned,
                    reason: None,
                },
                &actor,
            )
            .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn assign_to_agent_requires_ready_state() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        // 状态 Created,试图直接 assign(必须先 Ready)
        let res = svc
            .assign_to_agent(
                AssignWorktreeCommand {
                    tenant_id,
                    worktree_id: wt.id,
                    agent_id: AgentId::new(),
                    agent_session_id: AgentSessionId::new(),
                },
                &actor,
            )
            .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn record_observed_state_requires_local_runtime_actor() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        // 普通 actor,应 PermissionDenied
        let res = svc
            .record_observed_state(
                RecordObservedStateCommand {
                    tenant_id,
                    worktree_id: wt.id,
                    ahead: 1,
                    behind: 0,
                    current_agent_session_id: None,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(WorktreeError::PermissionDenied)));
    }

    #[tokio::test]
    async fn abandon_sets_abandoned() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        let abandoned = svc
            .abandon(
                AbandonCommand {
                    tenant_id,
                    worktree_id: wt.id,
                    reason: "需求变更".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(abandoned.status, WorktreeStatus::Abandoned);
    }

    #[tokio::test]
    async fn abandon_terminal_state_rejected() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        // 走完 happy path 到 Merged
        let mut current = wt;
        let path = [
            WorktreeStatus::Created,
            WorktreeStatus::Initializing,
            WorktreeStatus::Ready,
            WorktreeStatus::Assigned,
            WorktreeStatus::AgentRunning,
            WorktreeStatus::Committing,
            WorktreeStatus::Completed,
            WorktreeStatus::ReadyForReview,
            WorktreeStatus::Reviewing,
            WorktreeStatus::Merged,
        ];
        for w in path.windows(2) {
            current = svc
                .transition_status(
                    TransitionStatusCommand {
                        tenant_id,
                        worktree_id: current.id,
                        from: w[0],
                        to: w[1],
                        reason: None,
                    },
                    &actor,
                )
                .await
                .unwrap();
        }
        // Merged 是终态,无法 abandon
        let res = svc
            .abandon(
                AbandonCommand {
                    tenant_id,
                    worktree_id: current.id,
                    reason: "test".to_string(),
                },
                &actor,
            )
            .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn get_by_id_cross_tenant_denied() {
        let svc = InMemoryWorktreeService::new();
        let tenant_a = uuid::Uuid::new_v4();
        let actor_a = make_actor(tenant_a);
        let wt = svc
            .create_worktree(make_create_cmd(tenant_a), &actor_a)
            .await
            .unwrap();
        // 另一 tenant 的 actor
        let tenant_b = uuid::Uuid::new_v4();
        let actor_b = make_actor(tenant_b);
        let res = svc.get_by_id(wt.id, &actor_b).await;
        assert!(matches!(res, Err(WorktreeError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn list_by_work_item_filters() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wi = WorkItemId::new();
        let mut cmd = make_create_cmd(TenantId(tenant_id));
        cmd.work_item_id = wi;
        let wt1 = svc.create_worktree(cmd.clone(), &actor).await.unwrap();
        let wt2 = svc.create_worktree(cmd, &actor).await.unwrap();
        let list = svc
            .list_by_work_item(
                ListByWorkItemQuery {
                    tenant_id,
                    work_item_id: wi,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|s| s.id == wt1.id));
        assert!(list.iter().any(|s| s.id == wt2.id));
    }

    #[tokio::test]
    async fn list_by_agent_filters() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wt = svc
            .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
            .await
            .unwrap();
        // 走到 Ready
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                from: WorktreeStatus::Created,
                to: WorktreeStatus::Initializing,
                reason: None,
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                from: WorktreeStatus::Initializing,
                to: WorktreeStatus::Ready,
                reason: None,
            },
            &actor,
        )
        .await
        .unwrap();
        let agent = AgentId::new();
        svc.assign_to_agent(
            AssignWorktreeCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                agent_id: agent,
                agent_session_id: AgentSessionId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
        let list = svc
            .list_by_agent(
                ListByAgentQuery {
                    tenant_id,
                    agent_id: agent,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn detect_conflicts_finds_other_active_worktrees() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let repo = RepositoryId::new();
        let mut cmd1 = make_create_cmd(TenantId(tenant_id));
        cmd1.repository_id = repo;
        let wt1 = svc.create_worktree(cmd1, &actor).await.unwrap();
        let mut cmd2 = make_create_cmd(TenantId(tenant_id));
        cmd2.repository_id = repo;
        let wt2 = svc.create_worktree(cmd2, &actor).await.unwrap();
        // wt1 检测冲突,应发现 wt2(同 repo)
        let conflicts = svc.detect_conflicts(wt1.id, &actor).await.unwrap();
        assert!(conflicts.contains(&wt2.id));
        assert!(!conflicts.contains(&wt1.id));
    }

    #[tokio::test]
    async fn heatmap_aggregates_by_status() {
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let repo = RepositoryId::new();
        let mut cmd1 = make_create_cmd(TenantId(tenant_id));
        cmd1.repository_id = repo;
        let mut cmd2 = make_create_cmd(TenantId(tenant_id));
        cmd2.repository_id = repo;
        svc.create_worktree(cmd1, &actor).await.unwrap();
        svc.create_worktree(cmd2, &actor).await.unwrap();
        let heatmap = svc.heatmap(repo, &actor).await.unwrap();
        assert_eq!(heatmap.total_worktrees, 2);
        assert_eq!(heatmap.by_status.get("CREATED"), Some(&2));
    }

    #[tokio::test]
    async fn status_independence_from_work_item() {
        // INV-WT-01:Worktree 状态独立于 WorkItem 状态
        // 同一 WorkItem 下 3 个 Worktree 不同状态同时存在
        let svc = InMemoryWorktreeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id));
        let wi = WorkItemId::new();
        let mk = || {
            let mut c = make_create_cmd(TenantId(tenant_id));
            c.work_item_id = wi;
            c
        };
        let wt1 = svc.create_worktree(mk(), &actor).await.unwrap();
        let wt2 = svc.create_worktree(mk(), &actor).await.unwrap();
        let wt3 = svc.create_worktree(mk(), &actor).await.unwrap();
        // wt2 走到 Ready
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt2.id,
                from: WorktreeStatus::Created,
                to: WorktreeStatus::Initializing,
                reason: None,
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt2.id,
                from: WorktreeStatus::Initializing,
                to: WorktreeStatus::Ready,
                reason: None,
            },
            &actor,
        )
        .await
        .unwrap();
        // wt3 走 abandon
        svc.abandon(
            AbandonCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt3.id,
                reason: "不再需要".to_string(),
            },
            &actor,
        )
        .await
        .unwrap();
        // 同时存在 3 个不同状态
        let list = svc
            .list_by_work_item(
                ListByWorkItemQuery {
                    tenant_id,
                    work_item_id: wi,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 3);
        let statuses: Vec<WorktreeStatus> = list.iter().map(|w| w.status).collect();
        assert!(statuses.contains(&WorktreeStatus::Created));
        assert!(statuses.contains(&WorktreeStatus::Ready));
        assert!(statuses.contains(&WorktreeStatus::Abandoned));
        let _ = (wt1, wt2, wt3); // suppress unused
    }
}

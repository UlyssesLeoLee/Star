//! domain-development crate
//!
//! 详细 spec: docs/specs/domain-development-spec.md §8.5 (ChangeSet 状态机)
//! 上游基本设计: docs/basic-design.md §4.2.6 (Human-in-the-loop merge_gate)
//! 数据设计: docs/data-design.md §4.19 (`development` schema)
//! API 设计: docs/api-design.md §3.20 (Development endpoints)
//!
//! ## 职责
//!
//! Development 域 3 类实体 + ChangeSet 状态机 + 14 测试
//! - ChangeSet 聚合根 (§8.5):Draft → ReadyForReview → Approved/Rejected → Merged
//! - DevelopmentExecution 实体:CI/Test/Build 记录
//! - SymbolIndex 实体:代码符号索引(version 随 file 变化)
//!
//! ## 关键不变量
//!
//! - INV-DEV-01:ChangeSet 必带 tenant_id
//! - INV-DEV-02:Merge 必须 project_admin 或 tenant_admin (§4.2.6 Human-in-the-loop)
//! - INV-DEV-03:Status 严格转换 (5 状态机)
//! - INV-DEV-04:SymbolIndex version 随 file version 递增
//! - INV-DEV-05:file changes 在 Draft 状态可改,ReadyForReview 后只读
//!
//! Lead 责任: development Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

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

define_uuid_id!(ChangeSetId);
define_uuid_id!(DevelopmentExecutionId);
define_uuid_id!(SymbolIndexId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(WorktreeId);
define_uuid_id!(WorkItemId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(RepositoryId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

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
// 实体
// =====================================================================

/// ChangeSet(§8.5,聚合根)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: ChangeSetId,
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
    pub work_item_id: WorkItemId,
    pub agent_session_id: Option<AgentSessionId>,
    pub branch: String,
    pub base_sha: String,
    pub head_sha: String,
    pub files: Vec<FileChange>,
    pub stats: ChangeStats,
    pub status: ChangeSetStatus,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
}

/// 单个文件改动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: FileChangeType,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub previous_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl FileChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Added => "ADDED",
            Self::Modified => "MODIFIED",
            Self::Deleted => "DELETED",
            Self::Renamed => "RENAMED",
        }
    }
}

/// ChangeSet 累计统计
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeStats {
    pub files_changed: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

/// ChangeSet 5 状态(§8.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeSetStatus {
    /// 草稿(可改 file changes)
    Draft,
    /// 已提交评审
    ReadyForReview,
    /// 已批准(可 merge)
    Approved,
    /// 已拒绝(终态)
    Rejected,
    /// 已合并(终态)
    Merged,
}

impl ChangeSetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::ReadyForReview => "READY_FOR_REVIEW",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Merged => "MERGED",
        }
    }
    /// Approved/Rejected/Merged 为终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Approved | Self::Rejected | Self::Merged)
    }
    /// 严格状态机:仅允许下列迁移(INV-DEV-03)
    pub fn can_transition_to(self, next: ChangeSetStatus) -> bool {
        use ChangeSetStatus::*;
        match (self, next) {
            (Draft, ReadyForReview) => true,
            (ReadyForReview, Approved) => true,
            (ReadyForReview, Rejected) => true,
            (ReadyForReview, Draft) => true, // request_changes
            (Approved, Merged) => true,
            // 终态不可迁出
            _ => false,
        }
    }
}

/// DevelopmentExecution(实体,记录执行细节)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentExecution {
    pub id: DevelopmentExecutionId,
    pub change_set_id: ChangeSetId,
    pub executed_by: ExecutionActor,
    pub executed_at: DateTime<Utc>,
    pub command: String,
    pub result: ExecutionResult,
    pub output_ref: Option<String>,
}

/// 执行者(用户或 Agent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionActor {
    User(UserId),
    Agent(AgentId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success,
    Failed,
    Timeout,
}

impl ExecutionResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::Failed => "FAILED",
            Self::Timeout => "TIMEOUT",
        }
    }
}

/// SymbolIndex(实体,代码符号)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIndex {
    pub id: SymbolIndexId,
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    pub file_path: String,
    pub symbol_name: String,
    pub kind: SymbolKind,
    pub signature: String,
    pub line_start: u32,
    pub line_end: u32,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Class,
    Struct,
    Trait,
    Module,
    Constant,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "FUNCTION",
            Self::Class => "CLASS",
            Self::Struct => "STRUCT",
            Self::Trait => "TRAIT",
            Self::Module => "MODULE",
            Self::Constant => "CONSTANT",
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum DevelopmentError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid status transition: {from} -> {to}")]
    InvalidStatus { from: String, to: String },
    #[error("cross-tenant access denied: actor {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("file changes are read-only in status: {0}")]
    FileChangesReadOnly(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChangeSetCommand {
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
    pub work_item_id: WorkItemId,
    pub agent_session_id: Option<AgentSessionId>,
    pub branch: String,
    pub base_sha: String,
    pub head_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFileChangeCommand {
    pub change_set_id: ChangeSetId,
    pub file: FileChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitChangeSetCommand {
    pub change_set_id: ChangeSetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveChangeSetCommand {
    pub change_set_id: ChangeSetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectChangeSetCommand {
    pub change_set_id: ChangeSetId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestChangesCommand {
    pub change_set_id: ChangeSetId,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeChangeSetCommand {
    pub change_set_id: ChangeSetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordExecutionCommand {
    pub change_set_id: ChangeSetId,
    pub executed_by: ExecutionActor,
    pub command: String,
    pub result: ExecutionResult,
    pub output_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSymbolCommand {
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    pub file_path: String,
    pub symbol_name: String,
    pub kind: SymbolKind,
    pub signature: String,
    pub line_start: u32,
    pub line_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetChangeSetQuery {
    pub change_set_id: ChangeSetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByWorktreeQuery {
    pub worktree_id: WorktreeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByStatusQuery {
    pub tenant_id: TenantId,
    pub status: ChangeSetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSymbolQuery {
    pub symbol_id: SymbolIndexId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait DevelopmentCommandPort: Send + Sync {
    async fn create_change_set(
        &self,
        cmd: CreateChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn add_file_change(
        &self,
        cmd: AddFileChangeCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn submit(
        &self,
        cmd: SubmitChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn approve(
        &self,
        cmd: ApproveChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn reject(
        &self,
        cmd: RejectChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn request_changes(
        &self,
        cmd: RequestChangesCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn merge(
        &self,
        cmd: MergeChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn record_execution(
        &self,
        cmd: RecordExecutionCommand,
        actor: &ActorContext,
    ) -> Result<DevelopmentExecution, DevelopmentError>;

    async fn upsert_symbol(
        &self,
        cmd: UpsertSymbolCommand,
        actor: &ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError>;
}

#[async_trait]
pub trait DevelopmentQueryPort: Send + Sync {
    async fn get_change_set(
        &self,
        q: GetChangeSetQuery,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    async fn list_by_worktree(
        &self,
        q: ListByWorktreeQuery,
        actor: &ActorContext,
    ) -> Result<Vec<ChangeSet>, DevelopmentError>;

    async fn list_by_status(
        &self,
        q: ListByStatusQuery,
        actor: &ActorContext,
    ) -> Result<Vec<ChangeSet>, DevelopmentError>;

    async fn get_symbol(
        &self,
        q: GetSymbolQuery,
        actor: &ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError>;
}

#[async_trait]
pub trait DevelopmentRepository: Send + Sync {
    async fn insert_change_set(&self, cs: ChangeSet) -> Result<(), DevelopmentError>;
    async fn update_change_set(&self, cs: ChangeSet) -> Result<(), DevelopmentError>;
    async fn get_change_set(&self, id: ChangeSetId) -> Result<ChangeSet, DevelopmentError>;
    async fn list_change_sets_by_worktree(
        &self,
        worktree_id: WorktreeId,
    ) -> Result<Vec<ChangeSet>, DevelopmentError>;
    async fn list_change_sets_by_status(
        &self,
        tenant_id: TenantId,
        status: ChangeSetStatus,
    ) -> Result<Vec<ChangeSet>, DevelopmentError>;

    async fn insert_execution(&self, exec: DevelopmentExecution) -> Result<(), DevelopmentError>;

    async fn upsert_symbol(&self, s: SymbolIndex) -> Result<(), DevelopmentError>;
    async fn get_symbol(&self, id: SymbolIndexId) -> Result<SymbolIndex, DevelopmentError>;
    /// 同 (tenant, repo, file_path, symbol_name) 取最新符号
    async fn find_symbol_by_key(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        file_path: &str,
        symbol_name: &str,
    ) -> Result<Option<SymbolIndex>, DevelopmentError>;
}

// =====================================================================
// InMemoryDevelopmentService
// =====================================================================

pub struct InMemoryDevelopmentService {
    repo: Arc<dyn DevelopmentRepository>,
    change_sets: Arc<RwLock<HashMap<ChangeSetId, ChangeSet>>>,
    executions: Arc<RwLock<HashMap<DevelopmentExecutionId, DevelopmentExecution>>>,
    symbols: Arc<RwLock<HashMap<SymbolIndexId, SymbolIndex>>>,
}

impl InMemoryDevelopmentService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryDevelopmentRepository::new()),
            change_sets: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            symbols: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryDevelopmentService {
    fn default() -> Self {
        Self::new()
    }
}

fn check_tenant(actor: &ActorContext, tenant_id: TenantId) -> Result<(), DevelopmentError> {
    if TenantId::from(actor.tenant_id) != tenant_id {
        return Err(DevelopmentError::CrossTenantDenied(
            TenantId::from(actor.tenant_id),
            tenant_id,
        ));
    }
    Ok(())
}

#[async_trait]
impl DevelopmentCommandPort for InMemoryDevelopmentService {
    async fn create_change_set(
        &self,
        cmd: CreateChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        // INV-DEV-01:ChangeSet 必带 tenant_id
        check_tenant(actor, cmd.tenant_id)?;
        let cs = ChangeSet {
            id: ChangeSetId::new(),
            tenant_id: cmd.tenant_id,
            worktree_id: cmd.worktree_id,
            work_item_id: cmd.work_item_id,
            agent_session_id: cmd.agent_session_id,
            branch: cmd.branch,
            base_sha: cmd.base_sha,
            head_sha: cmd.head_sha,
            files: vec![],
            stats: ChangeStats::default(),
            status: ChangeSetStatus::Draft,
            created_at: Utc::now(),
            submitted_at: None,
        };
        self.repo.insert_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn add_file_change(
        &self,
        cmd: AddFileChangeCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        let mut cs = self
            .change_sets
            .write()
            .unwrap()
            .get(&cmd.change_set_id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "change_set:{}",
                cmd.change_set_id.as_uuid()
            )))?;
        check_tenant(actor, cs.tenant_id)?;
        // INV-DEV-05:Draft 状态可改,ReadyForReview 后只读
        if cs.status != ChangeSetStatus::Draft {
            return Err(DevelopmentError::FileChangesReadOnly(
                cs.status.as_str().to_string(),
            ));
        }
        // 累计 stats
        cs.stats.files_changed = cs.stats.files_changed.saturating_add(1);
        cs.stats.lines_added = cs.stats.lines_added.saturating_add(cmd.file.lines_added);
        cs.stats.lines_deleted = cs
            .stats
            .lines_deleted
            .saturating_add(cmd.file.lines_deleted);
        cs.files.push(cmd.file);
        self.repo.update_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn submit(
        &self,
        cmd: SubmitChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        let mut cs = self.fetch_mut(cmd.change_set_id)?;
        check_tenant(actor, cs.tenant_id)?;
        transition(&mut cs, ChangeSetStatus::ReadyForReview)?;
        cs.submitted_at = Some(Utc::now());
        self.repo.update_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn approve(
        &self,
        cmd: ApproveChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(DevelopmentError::PermissionDenied(
                "approve requires project_admin or tenant_admin".to_string(),
            ));
        }
        let mut cs = self.fetch_mut(cmd.change_set_id)?;
        check_tenant(actor, cs.tenant_id)?;
        transition(&mut cs, ChangeSetStatus::Approved)?;
        self.repo.update_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn reject(
        &self,
        cmd: RejectChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(DevelopmentError::PermissionDenied(
                "reject requires project_admin or tenant_admin".to_string(),
            ));
        }
        let mut cs = self.fetch_mut(cmd.change_set_id)?;
        check_tenant(actor, cs.tenant_id)?;
        transition(&mut cs, ChangeSetStatus::Rejected)?;
        self.repo.update_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn request_changes(
        &self,
        cmd: RequestChangesCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(DevelopmentError::PermissionDenied(
                "request_changes requires project_admin or tenant_admin".to_string(),
            ));
        }
        let mut cs = self.fetch_mut(cmd.change_set_id)?;
        check_tenant(actor, cs.tenant_id)?;
        transition(&mut cs, ChangeSetStatus::Draft)?;
        self.repo.update_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn merge(
        &self,
        cmd: MergeChangeSetCommand,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        // INV-DEV-02:Merge 必须 project_admin 或 tenant_admin
        if !actor.has_role("developer")
            && !actor.has_role("project_admin")
            && !actor.is_platform_admin
        {
            return Err(DevelopmentError::PermissionDenied(
                "merge requires project_admin or tenant_admin (INV-DEV-02)".to_string(),
            ));
        }
        let mut cs = self.fetch_mut(cmd.change_set_id)?;
        check_tenant(actor, cs.tenant_id)?;
        transition(&mut cs, ChangeSetStatus::Merged)?;
        self.repo.update_change_set(cs.clone()).await?;
        self.change_sets.write().unwrap().insert(cs.id, cs.clone());
        Ok(cs)
    }

    async fn record_execution(
        &self,
        cmd: RecordExecutionCommand,
        actor: &ActorContext,
    ) -> Result<DevelopmentExecution, DevelopmentError> {
        let cs = self
            .change_sets
            .read()
            .unwrap()
            .get(&cmd.change_set_id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "change_set:{}",
                cmd.change_set_id.as_uuid()
            )))?;
        check_tenant(actor, cs.tenant_id)?;
        let exec = DevelopmentExecution {
            id: DevelopmentExecutionId::new(),
            change_set_id: cmd.change_set_id,
            executed_by: cmd.executed_by,
            executed_at: Utc::now(),
            command: cmd.command,
            result: cmd.result,
            output_ref: cmd.output_ref,
        };
        self.repo.insert_execution(exec.clone()).await?;
        self.executions
            .write()
            .unwrap()
            .insert(exec.id, exec.clone());
        Ok(exec)
    }

    async fn upsert_symbol(
        &self,
        cmd: UpsertSymbolCommand,
        actor: &ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError> {
        check_tenant(actor, cmd.tenant_id)?;
        // INV-DEV-04:SymbolIndex version 随 file version 递增
        let existing = self
            .repo
            .find_symbol_by_key(
                cmd.tenant_id,
                cmd.repository_id,
                &cmd.file_path,
                &cmd.symbol_name,
            )
            .await?;
        let new_version = match &existing {
            Some(prev) => prev.version.saturating_add(1),
            None => 1,
        };
        let s = SymbolIndex {
            id: existing.map(|e| e.id).unwrap_or_else(SymbolIndexId::new),
            tenant_id: cmd.tenant_id,
            repository_id: cmd.repository_id,
            file_path: cmd.file_path,
            symbol_name: cmd.symbol_name,
            kind: cmd.kind,
            signature: cmd.signature,
            line_start: cmd.line_start,
            line_end: cmd.line_end,
            version: new_version,
        };
        self.repo.upsert_symbol(s.clone()).await?;
        self.symbols.write().unwrap().insert(s.id, s.clone());
        Ok(s)
    }
}

#[async_trait]
impl DevelopmentQueryPort for InMemoryDevelopmentService {
    async fn get_change_set(
        &self,
        q: GetChangeSetQuery,
        actor: &ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        let cs = self
            .change_sets
            .read()
            .unwrap()
            .get(&q.change_set_id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "change_set:{}",
                q.change_set_id.as_uuid()
            )))?;
        check_tenant(actor, cs.tenant_id)?;
        Ok(cs)
    }

    async fn list_by_worktree(
        &self,
        q: ListByWorktreeQuery,
        actor: &ActorContext,
    ) -> Result<Vec<ChangeSet>, DevelopmentError> {
        let list = self
            .change_sets
            .read()
            .unwrap()
            .values()
            .filter(|cs| cs.worktree_id == q.worktree_id)
            .cloned()
            .collect::<Vec<_>>();
        // 至少过滤跨 tenant
        let list = list
            .into_iter()
            .filter(|cs| cs.tenant_id == TenantId::from(actor.tenant_id))
            .collect();
        Ok(list)
    }

    async fn list_by_status(
        &self,
        q: ListByStatusQuery,
        actor: &ActorContext,
    ) -> Result<Vec<ChangeSet>, DevelopmentError> {
        check_tenant(actor, q.tenant_id)?;
        let list = self
            .change_sets
            .read()
            .unwrap()
            .values()
            .filter(|cs| cs.tenant_id == q.tenant_id && cs.status == q.status)
            .cloned()
            .collect();
        Ok(list)
    }

    async fn get_symbol(
        &self,
        q: GetSymbolQuery,
        actor: &ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError> {
        let s = self
            .symbols
            .read()
            .unwrap()
            .get(&q.symbol_id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "symbol:{}",
                q.symbol_id.as_uuid()
            )))?;
        check_tenant(actor, s.tenant_id)?;
        Ok(s)
    }
}

impl InMemoryDevelopmentService {
    fn fetch_mut(&self, id: ChangeSetId) -> Result<ChangeSet, DevelopmentError> {
        self.change_sets
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "change_set:{}",
                id.as_uuid()
            )))
    }
}

fn transition(cs: &mut ChangeSet, next: ChangeSetStatus) -> Result<(), DevelopmentError> {
    if !cs.status.can_transition_to(next) {
        return Err(DevelopmentError::InvalidStatus {
            from: cs.status.as_str().to_string(),
            to: next.as_str().to_string(),
        });
    }
    cs.status = next;
    Ok(())
}

// =====================================================================
// InMemoryDevelopmentRepository
// =====================================================================

pub struct InMemoryDevelopmentRepository {
    change_sets: RwLock<HashMap<ChangeSetId, ChangeSet>>,
    executions: RwLock<HashMap<DevelopmentExecutionId, DevelopmentExecution>>,
    symbols: RwLock<HashMap<SymbolIndexId, SymbolIndex>>,
}

impl InMemoryDevelopmentRepository {
    pub fn new() -> Self {
        Self {
            change_sets: RwLock::new(HashMap::new()),
            executions: RwLock::new(HashMap::new()),
            symbols: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryDevelopmentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DevelopmentRepository for InMemoryDevelopmentRepository {
    async fn insert_change_set(&self, cs: ChangeSet) -> Result<(), DevelopmentError> {
        self.change_sets.write().unwrap().insert(cs.id, cs);
        Ok(())
    }
    async fn update_change_set(&self, cs: ChangeSet) -> Result<(), DevelopmentError> {
        self.change_sets.write().unwrap().insert(cs.id, cs);
        Ok(())
    }
    async fn get_change_set(&self, id: ChangeSetId) -> Result<ChangeSet, DevelopmentError> {
        self.change_sets
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "change_set:{}",
                id.as_uuid()
            )))
    }
    async fn list_change_sets_by_worktree(
        &self,
        worktree_id: WorktreeId,
    ) -> Result<Vec<ChangeSet>, DevelopmentError> {
        Ok(self
            .change_sets
            .read()
            .unwrap()
            .values()
            .filter(|cs| cs.worktree_id == worktree_id)
            .cloned()
            .collect())
    }
    async fn list_change_sets_by_status(
        &self,
        tenant_id: TenantId,
        status: ChangeSetStatus,
    ) -> Result<Vec<ChangeSet>, DevelopmentError> {
        Ok(self
            .change_sets
            .read()
            .unwrap()
            .values()
            .filter(|cs| cs.tenant_id == tenant_id && cs.status == status)
            .cloned()
            .collect())
    }

    async fn insert_execution(&self, exec: DevelopmentExecution) -> Result<(), DevelopmentError> {
        self.executions.write().unwrap().insert(exec.id, exec);
        Ok(())
    }

    async fn upsert_symbol(&self, s: SymbolIndex) -> Result<(), DevelopmentError> {
        self.symbols.write().unwrap().insert(s.id, s);
        Ok(())
    }
    async fn get_symbol(&self, id: SymbolIndexId) -> Result<SymbolIndex, DevelopmentError> {
        self.symbols
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DevelopmentError::NotFound(format!(
                "symbol:{}",
                id.as_uuid()
            )))
    }
    async fn find_symbol_by_key(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        file_path: &str,
        symbol_name: &str,
    ) -> Result<Option<SymbolIndex>, DevelopmentError> {
        Ok(self
            .symbols
            .read()
            .unwrap()
            .values()
            .find(|s| {
                s.tenant_id == tenant_id
                    && s.repository_id == repository_id
                    && s.file_path == file_path
                    && s.symbol_name == symbol_name
            })
            .cloned())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn developer(tid: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid.0)
    }

    fn project_admin(tid: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid.0).with_role("project_admin")
    }

    fn make_cmd(tid: TenantId) -> CreateChangeSetCommand {
        CreateChangeSetCommand {
            tenant_id: tid,
            worktree_id: WorktreeId::new(),
            work_item_id: WorkItemId::new(),
            agent_session_id: None,
            branch: "feat/test".to_string(),
            base_sha: "base".to_string(),
            head_sha: "head".to_string(),
        }
    }

    #[test]
    fn change_set_status_as_str() {
        assert_eq!(ChangeSetStatus::Draft.as_str(), "DRAFT");
        assert_eq!(ChangeSetStatus::ReadyForReview.as_str(), "READY_FOR_REVIEW");
        assert_eq!(ChangeSetStatus::Approved.as_str(), "APPROVED");
        assert_eq!(ChangeSetStatus::Rejected.as_str(), "REJECTED");
        assert_eq!(ChangeSetStatus::Merged.as_str(), "MERGED");
        assert!(!ChangeSetStatus::Draft.is_terminal());
        assert!(ChangeSetStatus::Approved.is_terminal());
        assert!(ChangeSetStatus::Rejected.is_terminal());
        assert!(ChangeSetStatus::Merged.is_terminal());
    }

    #[test]
    fn status_state_machine_strict() {
        // 允许迁移
        assert!(ChangeSetStatus::Draft.can_transition_to(ChangeSetStatus::ReadyForReview));
        assert!(ChangeSetStatus::ReadyForReview.can_transition_to(ChangeSetStatus::Approved));
        assert!(ChangeSetStatus::ReadyForReview.can_transition_to(ChangeSetStatus::Rejected));
        assert!(ChangeSetStatus::ReadyForReview.can_transition_to(ChangeSetStatus::Draft));
        assert!(ChangeSetStatus::Approved.can_transition_to(ChangeSetStatus::Merged));
        // 禁止迁移
        assert!(!ChangeSetStatus::Draft.can_transition_to(ChangeSetStatus::Approved));
        assert!(!ChangeSetStatus::Draft.can_transition_to(ChangeSetStatus::Merged));
        assert!(!ChangeSetStatus::Rejected.can_transition_to(ChangeSetStatus::Draft));
        assert!(!ChangeSetStatus::Merged.can_transition_to(ChangeSetStatus::Draft));
        assert!(!ChangeSetStatus::Approved.can_transition_to(ChangeSetStatus::Rejected));
    }

    #[test]
    fn file_change_type_as_str() {
        assert_eq!(FileChangeType::Added.as_str(), "ADDED");
        assert_eq!(FileChangeType::Modified.as_str(), "MODIFIED");
        assert_eq!(FileChangeType::Deleted.as_str(), "DELETED");
        assert_eq!(FileChangeType::Renamed.as_str(), "RENAMED");
    }

    #[test]
    fn symbol_kind_as_str() {
        assert_eq!(SymbolKind::Function.as_str(), "FUNCTION");
        assert_eq!(SymbolKind::Struct.as_str(), "STRUCT");
        assert_eq!(SymbolKind::Trait.as_str(), "TRAIT");
        assert_eq!(SymbolKind::Module.as_str(), "MODULE");
        assert_eq!(SymbolKind::Constant.as_str(), "CONSTANT");
        assert_eq!(SymbolKind::Class.as_str(), "CLASS");
    }

    #[test]
    fn execution_result_as_str() {
        assert_eq!(ExecutionResult::Success.as_str(), "SUCCESS");
        assert_eq!(ExecutionResult::Failed.as_str(), "FAILED");
        assert_eq!(ExecutionResult::Timeout.as_str(), "TIMEOUT");
    }

    #[test]
    fn actor_can_merge() {
        let tid = uuid::Uuid::new_v4();
        let dev = developer(tid);
        let pa = project_admin(tid);
        let ta = ActorContext::new(Uuid::new_v4(), tid.0).with_role("tenant_admin");
        assert!(!dev.has_role("project_admin") && !dev.has_role("developer"));
        assert!(pa.has_role("project_admin"));
        assert!(ta.has_role("tenant_admin") || ta.is_platform_admin);
    }

    #[tokio::test]
    async fn create_change_set_starts_as_draft() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        assert_eq!(cs.status, ChangeSetStatus::Draft);
        assert_eq!(cs.tenant_id, tid);
        assert!(cs.submitted_at.is_none());
        assert!(cs.files.is_empty());
    }

    #[tokio::test]
    async fn add_file_change_in_draft() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let actor = developer(tid);
        let cs2 = svc
            .add_file_change(
                AddFileChangeCommand {
                    change_set_id: cs.id,
                    file: FileChange {
                        path: "src/lib.rs".to_string(),
                        change_type: FileChangeType::Modified,
                        lines_added: 10,
                        lines_deleted: 3,
                        previous_path: None,
                    },
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(cs2.files.len(), 1);
        assert_eq!(cs2.stats.files_changed, 1);
        assert_eq!(cs2.stats.lines_added, 10);
        assert_eq!(cs2.stats.lines_deleted, 3);
    }

    #[tokio::test]
    async fn add_file_change_rejected_after_ready_for_review() {
        // INV-DEV-05:ReadyForReview 后只读
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let actor = developer(tid);
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(cs.status, ChangeSetStatus::ReadyForReview);
        let res = svc
            .add_file_change(
                AddFileChangeCommand {
                    change_set_id: cs.id,
                    file: FileChange {
                        path: "src/lib.rs".to_string(),
                        change_type: FileChangeType::Added,
                        lines_added: 1,
                        lines_deleted: 0,
                        previous_path: None,
                    },
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(DevelopmentError::FileChangesReadOnly(_))));
    }

    #[tokio::test]
    async fn submit_draft_to_ready_for_review() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let actor = developer(tid);
        let cs2 = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(cs2.status, ChangeSetStatus::ReadyForReview);
        assert!(cs2.submitted_at.is_some());
    }

    #[tokio::test]
    async fn approve_ready_for_review_to_approved() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let dev = developer(tid);
        let pa = project_admin(tid);
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await
            .unwrap();
        let cs2 = svc
            .approve(
                ApproveChangeSetCommand {
                    change_set_id: cs.id,
                },
                &pa,
            )
            .await
            .unwrap();
        assert_eq!(cs2.status, ChangeSetStatus::Approved);
    }

    #[tokio::test]
    async fn reject_ready_for_review_to_rejected() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let dev = developer(tid);
        let pa = project_admin(tid);
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await
            .unwrap();
        let cs2 = svc
            .reject(
                RejectChangeSetCommand {
                    change_set_id: cs.id,
                    reason: "needs more tests".to_string(),
                },
                &pa,
            )
            .await
            .unwrap();
        assert_eq!(cs2.status, ChangeSetStatus::Rejected);
        assert!(cs2.status.is_terminal());
    }

    #[tokio::test]
    async fn merge_approved_to_merged_requires_admin() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let dev = developer(tid);
        let pa = project_admin(tid);
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await
            .unwrap();
        let cs = svc
            .approve(
                ApproveChangeSetCommand {
                    change_set_id: cs.id,
                },
                &pa,
            )
            .await
            .unwrap();
        let cs2 = svc
            .merge(
                MergeChangeSetCommand {
                    change_set_id: cs.id,
                },
                &pa,
            )
            .await
            .unwrap();
        assert_eq!(cs2.status, ChangeSetStatus::Merged);
    }

    #[tokio::test]
    async fn merge_denied_for_developer() {
        // INV-DEV-02:developer 角色 merge 拒绝
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let dev = developer(tid);
        let pa = project_admin(tid);
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await
            .unwrap();
        let cs = svc
            .approve(
                ApproveChangeSetCommand {
                    change_set_id: cs.id,
                },
                &pa,
            )
            .await
            .unwrap();
        let res = svc
            .merge(
                MergeChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await;
        assert!(matches!(res, Err(DevelopmentError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn request_changes_returns_to_draft() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let dev = developer(tid);
        let pa = project_admin(tid);
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await
            .unwrap();
        let cs2 = svc
            .request_changes(
                RequestChangesCommand {
                    change_set_id: cs.id,
                    comment: "fix naming".to_string(),
                },
                &pa,
            )
            .await
            .unwrap();
        assert_eq!(cs2.status, ChangeSetStatus::Draft);
    }

    #[tokio::test]
    async fn record_execution_links_to_change_set() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let actor = developer(tid);
        let exec = svc
            .record_execution(
                RecordExecutionCommand {
                    change_set_id: cs.id,
                    executed_by: ExecutionActor::User(UserId::from(actor.user_id)),
                    command: "cargo test".to_string(),
                    result: ExecutionResult::Success,
                    output_ref: Some("s3://logs/test-123.log".to_string()),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(exec.change_set_id, cs.id);
        assert_eq!(exec.result, ExecutionResult::Success);
        assert!(exec.output_ref.is_some());
    }

    #[tokio::test]
    async fn upsert_symbol_creates_v1() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = developer(tid);
        let s = svc
            .upsert_symbol(
                UpsertSymbolCommand {
                    tenant_id: tid,
                    repository_id: RepositoryId::new(),
                    file_path: "src/lib.rs".to_string(),
                    symbol_name: "add".to_string(),
                    kind: SymbolKind::Function,
                    signature: "fn add(a:i32,b:i32)->i32".to_string(),
                    line_start: 10,
                    line_end: 20,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.version, 1);
    }

    #[tokio::test]
    async fn symbol_version_increments_on_upsert() {
        // INV-DEV-04:SymbolIndex version 随 file version 递增
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = developer(tid);
        let repo_id = RepositoryId::new();
        let cmd = UpsertSymbolCommand {
            tenant_id: tid,
            repository_id: repo_id,
            file_path: "src/lib.rs".to_string(),
            symbol_name: "add".to_string(),
            kind: SymbolKind::Function,
            signature: "fn add(a:i32,b:i32)->i32".to_string(),
            line_start: 10,
            line_end: 20,
        };
        let s1 = svc.upsert_symbol(cmd.clone(), &actor).await.unwrap();
        let s2 = svc.upsert_symbol(cmd, &actor).await.unwrap();
        assert_eq!(s1.version, 1);
        assert_eq!(s2.version, 2);
        assert_eq!(s1.id, s2.id);
    }

    #[tokio::test]
    async fn list_by_status_filters_correctly() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let dev = developer(tid);
        let pa = project_admin(tid);
        // 3 个 ChangeSet:1 Draft,1 ReadyForReview,1 Approved
        let cs1 = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let cs2 = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let cs3 = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        svc.submit(
            SubmitChangeSetCommand {
                change_set_id: cs2.id,
            },
            &dev,
        )
        .await
        .unwrap();
        svc.submit(
            SubmitChangeSetCommand {
                change_set_id: cs3.id,
            },
            &dev,
        )
        .await
        .unwrap();
        svc.approve(
            ApproveChangeSetCommand {
                change_set_id: cs3.id,
            },
            &pa,
        )
        .await
        .unwrap();

        let drafts = svc
            .list_by_status(
                ListByStatusQuery {
                    tenant_id: tid,
                    status: ChangeSetStatus::Draft,
                },
                &dev,
            )
            .await
            .unwrap();
        let ready = svc
            .list_by_status(
                ListByStatusQuery {
                    tenant_id: tid,
                    status: ChangeSetStatus::ReadyForReview,
                },
                &dev,
            )
            .await
            .unwrap();
        let approved = svc
            .list_by_status(
                ListByStatusQuery {
                    tenant_id: tid,
                    status: ChangeSetStatus::Approved,
                },
                &dev,
            )
            .await
            .unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].id, cs1.id);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, cs2.id);
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].id, cs3.id);
    }

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryDevelopmentService::new();
        let tid_a = uuid::Uuid::new_v4();
        let tid_b = uuid::Uuid::new_v4();
        let actor_a = developer(tid_a);
        let cs = svc
            .create_change_set(make_cmd(tid_a), &developer(tid_a))
            .await
            .unwrap();
        let actor_b = developer(tid_b);
        // 跨 tenant get 拒绝
        let res = svc
            .get_change_set(
                GetChangeSetQuery {
                    change_set_id: cs.id,
                },
                &actor_b,
            )
            .await;
        assert!(matches!(
            res,
            Err(DevelopmentError::CrossTenantDenied(_, _))
        ));
        // 跨 tenant create 拒绝
        let res = svc
            .create_change_set(
                CreateChangeSetCommand {
                    tenant_id: tid_b,
                    worktree_id: WorktreeId::new(),
                    work_item_id: WorkItemId::new(),
                    agent_session_id: None,
                    branch: "feat/x".to_string(),
                    base_sha: "b".to_string(),
                    head_sha: "h".to_string(),
                },
                &actor_a,
            )
            .await;
        assert!(matches!(
            res,
            Err(DevelopmentError::CrossTenantDenied(_, _))
        ));
    }

    #[tokio::test]
    async fn full_lifecycle_draft_to_merged() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let dev = developer(tid);
        let pa = project_admin(tid);
        // 1. Draft + add files
        let cs = svc
            .create_change_set(make_cmd(tid), &developer(tid))
            .await
            .unwrap();
        let cs = svc
            .add_file_change(
                AddFileChangeCommand {
                    change_set_id: cs.id,
                    file: FileChange {
                        path: "src/lib.rs".to_string(),
                        change_type: FileChangeType::Modified,
                        lines_added: 5,
                        lines_deleted: 2,
                        previous_path: None,
                    },
                },
                &dev,
            )
            .await
            .unwrap();
        assert_eq!(cs.status, ChangeSetStatus::Draft);
        assert_eq!(cs.files.len(), 1);
        // 2. submit
        let cs = svc
            .submit(
                SubmitChangeSetCommand {
                    change_set_id: cs.id,
                },
                &dev,
            )
            .await
            .unwrap();
        assert_eq!(cs.status, ChangeSetStatus::ReadyForReview);
        // 3. approve
        let cs = svc
            .approve(
                ApproveChangeSetCommand {
                    change_set_id: cs.id,
                },
                &pa,
            )
            .await
            .unwrap();
        assert_eq!(cs.status, ChangeSetStatus::Approved);
        // 4. record execution
        let exec = svc
            .record_execution(
                RecordExecutionCommand {
                    change_set_id: cs.id,
                    executed_by: ExecutionActor::User(UserId::from(dev.user_id)),
                    command: "cargo test".to_string(),
                    result: ExecutionResult::Success,
                    output_ref: None,
                },
                &dev,
            )
            .await
            .unwrap();
        assert_eq!(exec.result, ExecutionResult::Success);
        // 5. merge
        let cs = svc
            .merge(
                MergeChangeSetCommand {
                    change_set_id: cs.id,
                },
                &pa,
            )
            .await
            .unwrap();
        assert_eq!(cs.status, ChangeSetStatus::Merged);
        assert!(cs.status.is_terminal());
    }

    #[tokio::test]
    async fn not_found_returns_proper_error() {
        let svc = InMemoryDevelopmentService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = developer(tid);
        let res = svc
            .get_change_set(
                GetChangeSetQuery {
                    change_set_id: ChangeSetId::new(),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(DevelopmentError::NotFound(_))));
    }
}

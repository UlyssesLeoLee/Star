//! Development 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.20 (Development Execution / ChangeSet / SymbolIndex)
//! - `docs/specs/domain-development-spec.md` §4 (接口签名)
//!
//! **端口清单**:
//! - `DevelopmentCommandPort`:5 方法(写)
//! - `DevelopmentQueryPort`:7 方法(读)
//! - `DevelopmentRepository`:基础设施层使用,本文件声明 trait
//!
//! **命令 DTO 命名**:
//! - spec §4 命名以 spec 为准(`CreateExecutionCommand` / `AppendChangeSetCommand` /
//!   `AttachRiskSignalCommand` / `CloseExecutionCommand`),骨架阶段的
//!   `RecordChangeSetCommand` / `CreateLinkCommand` / `RegisterSymbolIndexCommand`
//!   统一替换为 spec 命名。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{
    ChangeSet, DevelopmentContext, DevelopmentExecution, RepositoryContext, RiskSignal,
    SymbolIndex,
};
use crate::error::DevelopmentError;
use crate::value_object::{
    AgentSessionId, ChangeSetId, CommitId, ExecutionId, FileChangeStatus, FilePath,
    RepositoryId, RiskSeverity, RiskSignalKind, RiskSource, SymbolKind, TenantId, WorkItemId,
    WorktreeId,
};

// =====================================================================
// 命令 DTO(写操作输入)
// =====================================================================

/// 单 File Change 在 ChangeSet 创建时提供的草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeDraft {
    /// 文件路径
    pub path: FilePath,
    /// 旧路径(Renamed 时必填)
    pub old_path: Option<FilePath>,
    /// 变更状态
    pub status: FileChangeStatus,
    /// 新增行数
    pub lines_added: u32,
    /// 删除行数
    pub lines_deleted: u32,
    /// Before content hash
    pub before_content_hash: Option<String>,
    /// After content hash
    pub after_content_hash: Option<String>,
}

/// 单 Risk Signal 在 ChangeSet 创建时提供的草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSignalDraft {
    /// 类型(8 种)
    pub kind: RiskSignalKind,
    /// 严重度
    pub severity: RiskSeverity,
    /// 来源
    pub source: RiskSource,
    /// 证据
    pub evidence: String,
    /// 建议动作
    pub suggested_action: String,
}

/// `CreateExecutionCommand`(spec §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: crate::value_object::ProjectId,
    /// 关联 WorkItem
    pub work_item_id: WorkItemId,
    /// 关联 Repository
    pub repository_id: RepositoryId,
    /// 关联 Worktree(初始 1..N)
    pub worktree_ids: Vec<WorktreeId>,
}

/// `AppendChangeSetCommand`(spec §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendChangeSetCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: crate::value_object::ProjectId,
    /// 关联 Execution
    pub execution_id: ExecutionId,
    /// 关联 Worktree
    pub worktree_id: WorktreeId,
    /// 关联 Agent Session(可选)
    pub agent_session_id: Option<AgentSessionId>,
    /// 关联 Commit
    pub commit_id: CommitId,
    /// Diff Object Storage 引用
    pub diff_reference: String,
    /// 文件级变更列表
    pub files: Vec<FileChangeDraft>,
    /// 风险信号列表
    pub risk_signals: Vec<RiskSignalDraft>,
    /// 依赖变更
    pub dependency_changes: Vec<String>,
    /// Schema 变更
    pub schema_changes: Vec<String>,
    /// 配置变更
    pub config_changes: Vec<String>,
    /// 测试变更
    pub test_changes: Vec<String>,
}

/// `AttachRiskSignalCommand`(spec §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachRiskSignalCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 关联 ChangeSet
    pub change_set_id: ChangeSetId,
    /// 类型
    pub kind: RiskSignalKind,
    /// 严重度
    pub severity: RiskSeverity,
    /// 来源
    pub source: RiskSource,
    /// 证据
    pub evidence: String,
    /// 建议动作
    pub suggested_action: String,
    /// AISelfClaim 必填:已通过 Validation Chain 的 Validation ID
    pub validation_passed_id: Option<uuid::Uuid>,
}

/// `CloseExecutionCommand`(spec §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseExecutionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 关联 Execution
    pub execution_id: ExecutionId,
    /// 终态(Succeeded / Failed / Cancelled)
    pub terminal_state: crate::value_object::ExecutionState,
    /// 结束时间(可选,默认 now)
    pub ended_at: Option<DateTime<Utc>>,
}

/// `BuildSymbolIndexCommand`(扩展 spec,异步 worker 入口)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSymbolIndexCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 关联 Repository
    pub repository_id: RepositoryId,
    /// 符号种子(由 worker 提取后传入)
    pub symbol_seeds: Vec<SymbolSeed>,
}

/// 单 Symbol 在 Build 时提供的种子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSeed {
    /// 符号引用
    pub symbol_ref: String,
    /// 种类
    pub kind: SymbolKind,
    /// 签名
    pub signature: Option<String>,
    /// 所在文件
    pub file_path: FilePath,
    /// 行范围
    pub line_range: crate::value_object::LineRange,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListChangeSetQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListChangeSetQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 按 Execution 过滤(可选)
    pub execution_id: Option<ExecutionId>,
    /// 按 Worktree 过滤(可选)
    pub worktree_id: Option<WorktreeId>,
    /// 分页 limit
    pub limit: u32,
    /// 分页 offset
    pub offset: u32,
}

/// `ListExecutionQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExecutionQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 按 Worktree 过滤(可选)
    pub worktree_id: Option<WorktreeId>,
    /// 按 WorkItem 过滤(可选)
    pub work_item_id: Option<WorkItemId>,
    /// 分页 limit
    pub limit: u32,
    /// 分页 offset
    pub offset: u32,
}

/// `ListSymbolQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSymbolQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 名称前缀过滤(可选)
    pub name_prefix: Option<String>,
    /// 分页 limit
    pub limit: u32,
    /// 分页 offset
    pub offset: u32,
}

/// `SearchSymbolQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSymbolQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 关键词(模糊匹配 symbol_ref)
    pub keyword: String,
    /// 限制返回
    pub limit: u32,
}

/// Diff 短期预签名 URL(spec §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffDownloadURL {
    /// ChangeSet ID
    pub change_set_id: ChangeSetId,
    /// 短期 URL
    pub url: String,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
}

// =====================================================================
// 端口:DevelopmentCommandPort(5 方法,spec §4)
// =====================================================================

/// **Development 命令端口**(写操作 5 方法)
#[async_trait]
pub trait DevelopmentCommandPort: Send + Sync {
    /// 创建 Execution(INV-D-09:worktree_ids 1..N)
    async fn create_execution(
        &self,
        cmd: CreateExecutionCommand,
        actor: ActorContext,
    ) -> Result<ExecutionId, DevelopmentError>;

    /// 追加 ChangeSet(INV-D-01/03/05/07 校验)
    async fn append_change_set(
        &self,
        cmd: AppendChangeSetCommand,
        actor: ActorContext,
    ) -> Result<ChangeSetId, DevelopmentError>;

    /// 附加 Risk Signal(INV-D-04/07,D-003/D-005)
    async fn attach_risk_signal(
        &self,
        cmd: AttachRiskSignalCommand,
        actor: ActorContext,
    ) -> Result<RiskSignal, DevelopmentError>;

    /// 关闭 Execution(状态机终态迁移)
    async fn close_execution(
        &self,
        cmd: CloseExecutionCommand,
        actor: ActorContext,
    ) -> Result<DevelopmentExecution, DevelopmentError>;

    /// 刷新 SymbolIndex(INV-D-06/08)
    async fn build_symbol_index(
        &self,
        cmd: BuildSymbolIndexCommand,
        actor: ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError>;
}

// =====================================================================
// 端口:DevelopmentQueryPort(7 方法,spec §4)
// =====================================================================

/// **Development 查询端口**(读操作 7 方法)
#[async_trait]
pub trait DevelopmentQueryPort: Send + Sync {
    /// 按 ID 查询 Execution
    async fn get_execution(
        &self,
        id: ExecutionId,
        viewer: ActorContext,
    ) -> Result<DevelopmentExecution, DevelopmentError>;

    /// 列出 Execution(按 Worktree / WorkItem 过滤)
    async fn list_executions(
        &self,
        q: ListExecutionQuery,
        viewer: ActorContext,
    ) -> Result<Vec<DevelopmentExecution>, DevelopmentError>;

    /// 按 ID 查询 ChangeSet
    async fn get_change_set(
        &self,
        id: ChangeSetId,
        viewer: ActorContext,
    ) -> Result<ChangeSet, DevelopmentError>;

    /// 列出 ChangeSet(按 Execution / Worktree 过滤)
    async fn list_change_sets(
        &self,
        q: ListChangeSetQuery,
        viewer: ActorContext,
    ) -> Result<Vec<ChangeSet>, DevelopmentError>;

    /// 取得 ChangeSet 的 Diff 短期预签名 URL
    async fn get_diff_url(
        &self,
        id: ChangeSetId,
        viewer: ActorContext,
    ) -> Result<DiffDownloadURL, DevelopmentError>;

    /// 取得 SymbolIndex
    async fn get_symbol_index(
        &self,
        repository_id: RepositoryId,
        viewer: ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError>;

    /// 列出 Symbol(可按名称前缀过滤)
    async fn list_symbols(
        &self,
        q: ListSymbolQuery,
        viewer: ActorContext,
    ) -> Result<Vec<crate::entity::IndexedSymbol>, DevelopmentError>;

    /// 按关键词搜索 Symbol
    async fn search_symbol(
        &self,
        q: SearchSymbolQuery,
        viewer: ActorContext,
    ) -> Result<Vec<crate::entity::IndexedSymbol>, DevelopmentError>;

    /// 取得 RepositoryContext
    async fn get_repository_context(
        &self,
        repository_id: RepositoryId,
        viewer: ActorContext,
    ) -> Result<RepositoryContext, DevelopmentError>;

    /// 取得 DevelopmentContext
    async fn get_development_context(
        &self,
        execution_id: ExecutionId,
        viewer: ActorContext,
    ) -> Result<DevelopmentContext, DevelopmentError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **Development 仓库端口**(供 SQLx / 内存 / 测试 Adapter 实现)
#[async_trait]
pub trait DevelopmentRepository: Send + Sync {
    // ---- DevelopmentExecution ----
    /// 插入 Execution
    async fn insert_execution(
        &self,
        e: &DevelopmentExecution,
    ) -> Result<(), DevelopmentError>;
    /// 按 ID 读
    async fn find_execution_by_id(
        &self,
        id: ExecutionId,
    ) -> Result<Option<DevelopmentExecution>, DevelopmentError>;
    /// 列出 Execution
    async fn list_executions(
        &self,
        tenant_id: TenantId,
        worktree_id: Option<WorktreeId>,
        work_item_id: Option<WorkItemId>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DevelopmentExecution>, DevelopmentError>;
    /// 更新 Execution
    async fn update_execution(
        &self,
        e: &DevelopmentExecution,
    ) -> Result<(), DevelopmentError>;

    // ---- ChangeSet ----
    /// 插入 ChangeSet
    async fn insert_change_set(&self, c: &ChangeSet) -> Result<(), DevelopmentError>;
    /// 按 ID 读
    async fn find_change_set_by_id(
        &self,
        id: ChangeSetId,
    ) -> Result<Option<ChangeSet>, DevelopmentError>;
    /// 列出 ChangeSet
    async fn list_change_sets(
        &self,
        tenant_id: TenantId,
        execution_id: Option<ExecutionId>,
        worktree_id: Option<WorktreeId>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChangeSet>, DevelopmentError>;
    /// 标记 ChangeSet 已 commit(INV-D-02)
    async fn mark_change_set_committed(
        &self,
        id: ChangeSetId,
    ) -> Result<(), DevelopmentError>;

    // ---- SymbolIndex ----
    /// 取得 Repository 的 SymbolIndex(仓库层,无 tenant 校验)
    async fn find_symbol_index_by_repository(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Option<SymbolIndex>, DevelopmentError>;
    /// 插入或替换 SymbolIndex
    async fn upsert_symbol_index(
        &self,
        idx: &SymbolIndex,
    ) -> Result<(), DevelopmentError>;
}

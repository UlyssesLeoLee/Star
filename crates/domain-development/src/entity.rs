//! Development 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.19 (`development` schema)
//! - `docs/specs/domain-development-spec.md` §2 (实体清单)
//!
//! 包含 4 个核心实体 + 3 个投影实体:
//! - `DevelopmentExecution` — 主聚合根(§21)
//! - `ChangeSet` — 主聚合根(§21.1)
//! - `FileChange` — 子实体(ChangeSet 文件级变更)
//! - `RiskSignal` — 值对象(8 种类型)
//! - `SymbolIndex` — 投影(§20 / §21.2)
//! - `RepositoryContext` — 仓库元数据投影
//! - `DevelopmentContext` — 项目级上下文投影

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AgentSessionId, ChangeSetId, CommitId, ExecutionId, ExecutionState, FileChangeStatus,
    FilePath, LineRange, ProjectId, RepositoryId, RiskSeverity, RiskSignalId, RiskSignalKind,
    RiskSource, SymbolId, SymbolKind, TenantId, UserId, WorkItemId, WorktreeId,
};

// =====================================================================
// DevelopmentExecution 聚合根
// =====================================================================

/// **DevelopmentExecution 聚合根**(spec §21,17 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentExecution {
    /// 主键
    pub id: ExecutionId,

    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,

    /// Project ID
    pub project_id: ProjectId,

    /// 关联 WorkItem
    pub work_item_id: WorkItemId,

    /// 关联 Repository
    pub repository_id: RepositoryId,

    /// 关联的 Worktree 列表(1..N,INV-D-09)
    pub worktree_ids: Vec<WorktreeId>,

    /// 关联的 Agent Session 列表
    pub agent_session_ids: Vec<AgentSessionId>,

    /// 关联的 ChangeSet 列表
    pub change_set_ids: Vec<ChangeSetId>,

    /// 验证结果 ID 列表
    pub validation_result_ids: Vec<uuid::Uuid>,

    /// 反馈 ID 列表
    pub feedback_ids: Vec<uuid::Uuid>,

    /// Commit ID 列表
    pub commit_ids: Vec<CommitId>,

    /// PR ID 列表
    pub pull_request_ids: Vec<uuid::Uuid>,

    /// 启动时间
    pub started_at: DateTime<Utc>,

    /// 结束时间(终态填入)
    pub ended_at: Option<DateTime<Utc>>,

    /// 当前状态
    pub execution_state: ExecutionState,

    /// 乐观锁版本
    pub lock_version: u32,

    /// 创建者
    pub created_by_user_id: UserId,
}

impl DevelopmentExecution {
    /// 字段数
    pub const FIELD_COUNT: usize = 17;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
    }

    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.execution_state,
            ExecutionState::Succeeded | ExecutionState::Failed | ExecutionState::Cancelled
        )
    }

    /// 关闭 Execution(写入 ended_at)
    pub fn close(&mut self, terminal_state: ExecutionState, at: DateTime<Utc>) {
        self.execution_state = terminal_state;
        self.ended_at = Some(at);
        self.bump_version();
    }
}

// =====================================================================
// ChangeSet 聚合根
// =====================================================================

/// **ChangeSet 聚合根**(spec §21.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    /// 主键
    pub id: ChangeSetId,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// Project ID
    pub project_id: ProjectId,

    /// 关联 Worktree
    pub worktree_id: WorktreeId,

    /// 关联 Agent Session
    pub agent_session_id: Option<AgentSessionId>,

    /// 关联 Commit
    pub commit_id: CommitId,

    /// 文件级变更列表
    pub files: Vec<FileChange>,

    /// 符号级变更列表
    pub symbols: Vec<SymbolChange>,

    /// Diff Object Storage 引用(必带 tenant_id 前缀,INV-D-05)
    pub diff_reference: String,

    /// 计数
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub renamed_files: u32,
    pub generated_files: u32,

    /// 风险信号列表(必含 8 种基本类型,INV-D-04)
    pub risk_signals: Vec<RiskSignal>,

    /// 依赖变更
    pub dependency_changes: Vec<String>,
    pub schema_changes: Vec<String>,
    pub config_changes: Vec<String>,
    pub test_changes: Vec<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 乐观锁版本
    pub lock_version: u32,

    /// 是否已 commit(commit 后不可修改,INV-D-02)
    pub is_committed: bool,
}

impl ChangeSet {
    /// 字段数
    pub const FIELD_COUNT: usize = 19;

    /// 升级乐观锁版本
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
    }

    /// 文件总数
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// 高严重度(>=High)Risk Signal 数量(用于事件触发门槛)
    pub fn high_severity_signal_count(&self) -> usize {
        self.risk_signals
            .iter()
            .filter(|r| r.severity >= RiskSeverity::High)
            .count()
    }
}

// =====================================================================
// FileChange 子实体
// =====================================================================

/// **FileChange**(文件级变更,spec §21.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// 主键
    pub id: uuid::Uuid,

    /// 文件路径(INV-D-01:结构化)
    pub path: FilePath,

    /// 重命名前的路径(仅 Renamed 状态)
    pub old_path: Option<FilePath>,

    /// 变更状态
    pub status: FileChangeStatus,

    /// 新增行数
    pub lines_added: u32,

    /// 删除行数
    pub lines_deleted: u32,

    /// Before 内容 hash(可选)
    pub before_content_hash: Option<String>,

    /// After 内容 hash(可选)
    pub after_content_hash: Option<String>,
}

impl FileChange {
    /// 字段数
    pub const FIELD_COUNT: usize = 7;
}

// =====================================================================
// SymbolChange 子实体
// =====================================================================

/// **SymbolChange**(符号级变更,spec §21.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolChange {
    /// 符号引用(如 `crate::foo::bar`)
    pub symbol_ref: String,

    /// 符号种类
    pub kind: SymbolKind,

    /// 变更状态(Added / Modified / Removed)
    pub status: FileChangeStatus,

    /// 旧签名(用于 diff 渲染)
    pub old_signature: Option<String>,

    /// 新签名
    pub new_signature: Option<String>,

    /// 所在文件
    pub file_path: FilePath,

    /// 所在行范围
    pub line_range: LineRange,
}

impl SymbolChange {
    /// 字段数
    pub const FIELD_COUNT: usize = 7;
}

// =====================================================================
// RiskSignal 值对象(spec §2,INV-D-04 8 种类型)
// =====================================================================

/// **RiskSignal**(spec §2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSignal {
    /// 主键
    pub id: RiskSignalId,

    /// 所属 ChangeSet
    pub change_set_id: ChangeSetId,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// 类型(8 种基本类型之一)
    pub kind: RiskSignalKind,

    /// 严重度
    pub severity: RiskSeverity,

    /// 来源
    pub source: RiskSource,

    /// 证据
    pub evidence: String,

    /// 建议动作
    pub suggested_action: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl RiskSignal {
    /// 字段数
    pub const FIELD_COUNT: usize = 8;

    /// 是否需触发 Validation Chain(INV-D-07)
    pub fn requires_validation(&self) -> bool {
        self.kind == RiskSignalKind::AISelfClaim
    }
}

// =====================================================================
// SymbolIndex 投影(spec §20 / §21.2)
// =====================================================================

/// **SymbolIndex 投影**(spec §20,V1:File-level + Basic Symbol Detection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIndex {
    /// 主键
    pub id: uuid::Uuid,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// 关联 Repository
    pub repository_id: RepositoryId,

    /// 符号集
    pub symbols: Vec<IndexedSymbol>,

    /// 最后刷新时间
    pub last_refresh_at: DateTime<Utc>,

    /// 单调递增版本号(INV-D-06:跨 Repository 不合并)
    pub version: u32,
}

impl SymbolIndex {
    /// 字段数
    pub const FIELD_COUNT: usize = 6;

    /// 升级版本
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.last_refresh_at = Utc::now();
    }
}

/// **IndexedSymbol**(索引中的一项符号)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedSymbol {
    /// 主键
    pub id: SymbolId,

    /// 符号引用
    pub symbol_ref: String,

    /// 种类
    pub kind: SymbolKind,

    /// 签名
    pub signature: Option<String>,

    /// 所在文件
    pub file_path: FilePath,

    /// 行范围
    pub line_range: LineRange,
}

impl IndexedSymbol {
    /// 字段数
    pub const FIELD_COUNT: usize = 6;
}

// =====================================================================
// RepositoryContext 投影(spec §20)
// =====================================================================

/// **RepositoryContext 投影**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContext {
    /// 主键
    pub id: uuid::Uuid,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// 关联 Repository
    pub repository_id: RepositoryId,

    /// 主要语言(如 "rust")
    pub primary_language: Option<String>,

    /// 框架(如 "actix-web")
    pub framework: Option<String>,

    /// 构建系统(如 "cargo")
    pub build_system: Option<String>,

    /// 测试框架(如 "cargo test")
    pub test_framework: Option<String>,

    /// 文件总数
    pub total_files: u32,

    /// 代码总行数
    pub total_lines: u32,

    /// 最后索引时间
    pub last_indexed_at: DateTime<Utc>,
}

impl RepositoryContext {
    /// 字段数
    pub const FIELD_COUNT: usize = 10;
}

// =====================================================================
// DevelopmentContext 投影(spec §20)
// =====================================================================

/// **DevelopmentContext 投影**(spec §20,持久化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentContext {
    /// 主键
    pub id: uuid::Uuid,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// Project ID
    pub project_id: ProjectId,

    /// 关联 WorkItem
    pub work_item_id: WorkItemId,

    /// 关联 Execution
    pub execution_id: ExecutionId,

    /// 关联的 Relevant Symbol 列表
    pub relevant_symbols: Vec<String>,

    /// 关联的 Relevant File 列表
    pub relevant_files: Vec<FilePath>,

    /// 架构约束(由 Project Policy 提供)
    pub architecture_constraints: Vec<String>,

    /// 最后编译时间
    pub last_compiled_at: Option<DateTime<Utc>>,

    /// 缓存版本
    pub version: u32,
}

impl DevelopmentContext {
    /// 字段数
    pub const FIELD_COUNT: usize = 10;
}

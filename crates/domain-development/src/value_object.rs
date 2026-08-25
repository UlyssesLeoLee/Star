//! Development 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.19 (`development` schema)
//! - `docs/specs/domain-development-spec.md` §2 (实体清单) / §3 (RiskSignal 类型)
//!
//! 集中放置强类型 ID、文件路径、Risk Signal 枚举等。
//!
//! **8 种 Risk Signal 类型**(basic-design §4.8.5,接口稳定承诺 #4):
//! - LargeChange / GeneratedFile / SchemaChange / DependencyUpgrade
//! - SecurityHint / TestCoverageDrop / ConflictRisk / AISelfClaim

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

define_uuid_id!(ExecutionId);
define_uuid_id!(ChangeSetId);
define_uuid_id!(SymbolId);
define_uuid_id!(RiskSignalId);
define_uuid_id!(FileChangeId);
define_uuid_id!(RepositoryId);
define_uuid_id!(RepositoryContextId);
define_uuid_id!(SymbolIndexId);
define_uuid_id!(WorktreeId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(CommitId);
define_uuid_id!(ProjectId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 枚举:ExecutionState(开发执行状态机)
// =====================================================================

/// **ExecutionState**(开发执行状态,spec §21)
///
/// 状态机:`Pending → Running → Succeeded | Failed | Cancelled`
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionState {
    /// 等待启动
    #[default]
    Pending,
    /// 运行中
    Running,
    /// 成功完成
    Succeeded,
    /// 失败
    Failed,
    /// 取消
    Cancelled,
}

impl std::fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "PENDING",
            Self::Running => "RUNNING",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
        };
        f.write_str(s)
    }
}

/// 状态机合法迁移图(终态 Succeeded/Failed/Cancelled 不可再迁出)
pub fn is_valid_state_transition(from: ExecutionState, to: ExecutionState) -> bool {
    use ExecutionState::*;
    match (from, to) {
        (Pending, Running) => true,
        (Pending, Cancelled) => true,
        (Pending, Failed) => true,
        (Running, Succeeded) => true,
        (Running, Failed) => true,
        (Running, Cancelled) => true,
        // 终态不能迁出
        (Succeeded, _) | (Failed, _) | (Cancelled, _) => false,
        // 同态
        (a, b) if a == b => false,
        // 其他组合不允许
        _ => false,
    }
}

// =====================================================================
// 枚举:FileChangeStatus(文件级变更状态)
// =====================================================================

/// **FileChangeStatus**(spec §21.1)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileChangeStatus {
    /// 新增
    Added,
    /// 修改
    Modified,
    /// 删除
    Deleted,
    /// 重命名
    Renamed,
    /// 自动生成
    Generated,
}

impl std::fmt::Display for FileChangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Added => "ADDED",
            Self::Modified => "MODIFIED",
            Self::Deleted => "DELETED",
            Self::Renamed => "RENAMED",
            Self::Generated => "GENERATED",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:SymbolKind(符号种类,spec §20)
// =====================================================================

/// **SymbolKind**(spec §20 / §21.1,V1:File-level + Basic Symbol Detection)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolKind {
    /// 文件
    File,
    /// 函数
    Function,
    /// 方法
    Method,
    /// 类
    Class,
    /// 结构体
    Struct,
    /// 枚举
    Enum,
    /// Trait / Interface
    Trait,
    /// 模块 / Namespace
    Module,
    /// 变量
    Variable,
    /// 常量
    Constant,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::File => "FILE",
            Self::Function => "FUNCTION",
            Self::Method => "METHOD",
            Self::Class => "CLASS",
            Self::Struct => "STRUCT",
            Self::Enum => "ENUM",
            Self::Trait => "TRAIT",
            Self::Module => "MODULE",
            Self::Variable => "VARIABLE",
            Self::Constant => "CONSTANT",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:RiskSignalKind(8 种基本类型,INV-D-04 锁定)
// =====================================================================

/// **RiskSignalKind** 8 种基本类型(spec §4 / basic-design §4.8.5)
///
/// **INV-D-04**:8 种类型被基本设计锁定,后续 RFC 阶段不会变更(接口稳定承诺 #4)。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskSignalKind {
    /// 大量变更
    LargeChange,
    /// 自动生成文件
    GeneratedFile,
    /// Schema 变更
    SchemaChange,
    /// 依赖升级
    DependencyUpgrade,
    /// 安全提示
    SecurityHint,
    /// 测试覆盖率下降
    TestCoverageDrop,
    /// 合并冲突风险
    ConflictRisk,
    /// AI 自我声称(必走 Validation Chain,INV-D-07)
    AISelfClaim,
}

impl RiskSignalKind {
    /// 8 种类型集合(供 INV-D-04 校验)
    pub const ALL: &'static [RiskSignalKind] = &[
        Self::LargeChange,
        Self::GeneratedFile,
        Self::SchemaChange,
        Self::DependencyUpgrade,
        Self::SecurityHint,
        Self::TestCoverageDrop,
        Self::ConflictRisk,
        Self::AISelfClaim,
    ];

    /// 是否合法(在 8 种类型中)
    pub fn is_known(&self) -> bool {
        Self::ALL.contains(self)
    }
}

impl std::fmt::Display for RiskSignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::LargeChange => "LARGE_CHANGE",
            Self::GeneratedFile => "GENERATED_FILE",
            Self::SchemaChange => "SCHEMA_CHANGE",
            Self::DependencyUpgrade => "DEPENDENCY_UPGRADE",
            Self::SecurityHint => "SECURITY_HINT",
            Self::TestCoverageDrop => "TEST_COVERAGE_DROP",
            Self::ConflictRisk => "CONFLICT_RISK",
            Self::AISelfClaim => "AI_SELF_CLAIM",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:RiskSeverity
// =====================================================================

/// **RiskSeverity**(spec §2)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskSeverity {
    /// 信息
    Info,
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

impl std::fmt::Display for RiskSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Info => "INFO",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:RiskSource
// =====================================================================

/// **RiskSource**(spec §2)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskSource {
    /// 静态分析
    StaticAnalysis,
    /// Lint
    Lint,
    /// AI 分类器
    AIClassifier,
    /// 人工
    Human,
    /// 启发式
    Heuristic,
}

impl std::fmt::Display for RiskSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::StaticAnalysis => "STATIC_ANALYSIS",
            Self::Lint => "LINT",
            Self::AIClassifier => "AI_CLASSIFIER",
            Self::Human => "HUMAN",
            Self::Heuristic => "HEURISTIC",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 值对象:FilePath(typed path,INV-D-01:结构化数据基础)
// =====================================================================

/// **FilePath**(强类型文件路径)
///
/// 约束:
/// - 不允许空字符串
/// - 不允许以 `/` 开头(相对路径语义)
/// - 不允许包含 `..`(防止路径穿越)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FilePath(String);

impl FilePath {
    /// 构造 FilePath(校验)
    pub fn new(path: impl Into<String>) -> Result<Self, String> {
        let s: String = path.into();
        if s.trim().is_empty() {
            return Err("FilePath 不允许为空".to_string());
        }
        if s.starts_with('/') {
            return Err("FilePath 不允许以 '/' 开头(必须为相对路径)".to_string());
        }
        if s.contains("..") {
            return Err("FilePath 不允许包含 '..' 段".to_string());
        }
        Ok(Self(s))
    }

    /// unwrap 风格的构造(测试 / 已知合法值场景)
    pub fn new_unchecked(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// 取得内部字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 取得后缀
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.0).extension().and_then(|e| e.to_str())
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for FilePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// =====================================================================
// 值对象:LineRange
// =====================================================================

/// **LineRange**(行号范围,Symbol 定位)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineRange {
    /// 起始行(含)
    pub start: u32,
    /// 结束行(含)
    pub end: u32,
}

impl LineRange {
    /// 构造 LineRange
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
    /// 行数
    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start) + 1
    }
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }
}

// =====================================================================
// 标准角色(与 domain-workflow 对齐)
// =====================================================================

/// Development 相关标准角色常量
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 平台运营
    pub const PLATFORM_OPERATOR: &str = "platform_operator";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读观察者
    pub const VIEWER: &str = "viewer";
}

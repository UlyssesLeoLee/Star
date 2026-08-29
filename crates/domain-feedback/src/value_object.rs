//! Feedback 域值对象
//!
//! 来源:
//! - `docs/data-design.md` §4.22 (`feedback` schema)
//! - `docs/specs/domain-feedback-spec.md` §2
//! - `docs/api-design.md` §3.23
//!
//! **强类型 ID** 避免与 `domain-comment` 等 crate 的 ID 串扰;`Uuid` 是底层
//! 表示,Service/Repository 层只接触 `FeedbackId` 等强类型。

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(本 crate 自有)
// =====================================================================

define_uuid_id!(FeedbackId);
define_uuid_id!(FeedbackResolutionId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);
define_uuid_id!(WorkItemId);
define_uuid_id!(AcceptanceCriterionId);
define_uuid_id!(RequirementId);
define_uuid_id!(WorktreeId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(RepositoryId);
define_uuid_id!(CommitId);
define_uuid_id!(TestId);
define_uuid_id!(BuildId);
define_uuid_id!(DecisionId);
define_uuid_id!(SymbolId);

// =====================================================================
// 角色
// =====================================================================

/// 角色常量
pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
    pub const VIEWER: &str = "viewer";
}

// =====================================================================
// FeedbackStatus — 6 状态机(§7.3)
// =====================================================================

/// **Feedback 状态机 6 状态**(spec §2,§7.3,basic-design §10 接口稳定承诺 #7)
///
/// 状态迁移规则(完整状态机):
/// - `Open → Acknowledged`(Agent 拉取,INV-FB-01)
/// - `Acknowledged → Applied`(ChangeSet 提交,INV-FB-01)
/// - `Applied → Verified`(Validation 通过,INV-FB-01)
/// - `Open | Acknowledged | Applied → Rejected`(人工/系统拒绝)
/// - `Open | Acknowledged | Applied | Verified → Superseded`(必须有 successor,INV-FB-04)
/// - `Rejected / Superseded` 是终态,不可再迁移
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeedbackStatus {
    /// 刚创建
    Open,
    /// Agent 拉取(消费中)
    Acknowledged,
    /// 已应用(ChangeSet 已提交)
    Applied,
    /// 已验证(Validation 通过)
    Verified,
    /// 已拒绝(终态)
    Rejected,
    /// 已取代(终态,INV-FB-04 必须有 successor)
    Superseded,
}

impl Default for FeedbackStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for FeedbackStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Open => "OPEN",
            Self::Acknowledged => "ACKNOWLEDGED",
            Self::Applied => "APPLIED",
            Self::Verified => "VERIFIED",
            Self::Rejected => "REJECTED",
            Self::Superseded => "SUPERSEDED",
        };
        f.write_str(s)
    }
}

impl FeedbackStatus {
    /// 是否为终态(`Rejected` / `Superseded`)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Rejected | Self::Superseded)
    }

    /// 检查 `from -> to` 是否为合法迁移(INV-FB-01)
    pub fn can_transition_to(self, to: FeedbackStatus) -> bool {
        use FeedbackStatus::*;
        if self == to {
            return false;
        }
        match (self, to) {
            (Open, Acknowledged) => true,
            (Acknowledged, Applied) => true,
            (Applied, Verified) => true,
            (Open | Acknowledged | Applied, Rejected) => true,
            (Open | Acknowledged | Applied | Verified, Superseded) => true,
            _ => false,
        }
    }
}

// =====================================================================
// FeedbackType — 7 类(spec §7 Type)
// =====================================================================

/// **Feedback 类型**(spec §7:Question / Architecture / Review Finding /
/// Security / Conflict / Test Failure / Other)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeedbackType {
    /// 提问/澄清
    Question,
    /// 架构性意见
    Architecture,
    /// 评审发现(Review Finding)
    ReviewFinding,
    /// 安全相关
    Security,
    /// 冲突(目标之间)
    Conflict,
    /// 测试失败
    TestFailure,
    /// 其他
    Other,
}

impl Default for FeedbackType {
    fn default() -> Self {
        Self::Other
    }
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Question => "QUESTION",
            Self::Architecture => "ARCHITECTURE",
            Self::ReviewFinding => "REVIEW_FINDING",
            Self::Security => "SECURITY",
            Self::Conflict => "CONFLICT",
            Self::TestFailure => "TEST_FAILURE",
            Self::Other => "OTHER",
        };
        f.write_str(s)
    }
}

// =====================================================================
// Severity — P0-P3(spec §4.3.6 Intervention Queue 优先级)
// =====================================================================

/// **Feedback 严重程度**(P0-P3,§4.3.6 Intervention Queue 优先级)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// P0 — Security / 必须立即干预
    P0,
    /// P1 — Architecture
    P1,
    /// P2 — Review Finding
    P2,
    /// P3 — Other / Question
    P3,
}

impl Default for Severity {
    fn default() -> Self {
        Self::P3
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
        };
        f.write_str(s)
    }
}

// =====================================================================
// FeedbackTarget — 11 类(spec §7 Target,SOW 11 Target)
// =====================================================================

/// **Feedback 目标类型 11 种**(spec §7, SOW 任务范围 11 Target)
///
/// **注意**:spec §2 文档(§4.3.3)列出 13 种(含 RuntimeLog / PullRequest /
/// ReviewFinding),本 crate MVP 范围(SOW 必做范围)实现 11 种
/// `WorkItem / Requirement / AC / Worktree / AgentSession / File / Symbol /
/// DiffHunk / Test / Build / Decision`,MVP 不实现:RuntimeLog, PullRequest, ReviewFinding。
///
/// 任何新增 Target 都需同步 `TargetType::count()` 计数 + 工厂方法 + 转换。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeedbackTarget {
    /// WorkItem 目标
    WorkItem { work_item_id: WorkItemId },
    /// Requirement 目标
    Requirement { requirement_id: RequirementId },
    /// AcceptanceCriterion 目标
    AcceptanceCriterion { ac_id: AcceptanceCriterionId },
    /// Worktree 目标
    Worktree { worktree_id: WorktreeId },
    /// AgentSession 目标
    AgentSession { session_id: AgentSessionId },
    /// File 目标(必带 line_range,即使为 None 表示整个文件)
    File {
        repository_id: RepositoryId,
        path: String,
        line_range: Option<LineRange>,
    },
    /// Symbol 目标
    Symbol {
        symbol_id: SymbolId,
        ref_name: String,
    },
    /// DiffHunk 目标
    DiffHunk {
        commit_id: CommitId,
        hunk_index: u32,
    },
    /// Test 目标
    Test { test_id: TestId },
    /// Build 目标
    Build { build_id: BuildId },
    /// Decision 目标
    Decision { decision_id: DecisionId },
}

impl FeedbackTarget {
    /// 11 种 Target 计数(常量,SOW 任务范围)
    pub const COUNT: usize = 11;

    /// Target 类型标签(SCREAMING_SNAKE_CASE)
    pub fn kind(&self) -> &'static str {
        match self {
            Self::WorkItem { .. } => "WORK_ITEM",
            Self::Requirement { .. } => "REQUIREMENT",
            Self::AcceptanceCriterion { .. } => "ACCEPTANCE_CRITERION",
            Self::Worktree { .. } => "WORKTREE",
            Self::AgentSession { .. } => "AGENT_SESSION",
            Self::File { .. } => "FILE",
            Self::Symbol { .. } => "SYMBOL",
            Self::DiffHunk { .. } => "DIFF_HUNK",
            Self::Test { .. } => "TEST",
            Self::Build { .. } => "BUILD",
            Self::Decision { .. } => "DECISION",
        }
    }
}

/// **行范围**(用于 File 目标)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineRange {
    /// 起始行(1-based,包含)
    pub start: u32,
    /// 结束行(1-based,包含)
    pub end: u32,
}

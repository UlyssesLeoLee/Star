//! SCM 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.18 (`scm` schema)
//! - `docs/specs/domain-scm-spec.md` §2 (实体清单) / §3 (基本类型)
//!
//! 集中放置强类型 ID、ScmProvider 枚举、ScmOwnership 枚举、SyncStatus 枚举、PullRequestState 枚举等。
//!
//! **MVP 范围**:Connected 所有权(继承 §4.7.4,§30.6)
//! **PR 状态机**:7 状态 DRAFT / OPEN / REVIEWING / CHANGES_REQUESTED / APPROVED / MERGEABLE / MERGED / CLOSED
//! (继承 basic-design §7.5,§A.6;实际为 8 个字符串,Merageable 合并到 MERGED 前)

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

define_uuid_id!(ScmProviderId);
define_uuid_id!(RepositoryId);
define_uuid_id!(BranchId);
define_uuid_id!(CommitId);
define_uuid_id!(PullRequestId);
define_uuid_id!(ReviewId);
define_uuid_id!(PipelineId);
define_uuid_id!(WebhookEventId);

//// 标准 Tenant ID(避免依赖 domain-tenant)
define_uuid_id!(TenantId);

// 标准 Project ID
define_uuid_id!(ProjectId);

// 强类型 User ID
define_uuid_id!(UserId);

// 强类型 WorkItem ID(避免依赖 domain-work-item)
define_uuid_id!(WorkItemId);

/// 外部 Repository ID 引用(厂商侧 ID,如 GitHub 的 "acme/foo" 字符串)
/// 这里 newtype 包装字符串,避免与 `RepositoryId`(平台内 ID)混淆。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExternalRepositoryId(pub String);

impl ExternalRepositoryId {
    /// 构造
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// 内部字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// 取出字符串(consume)
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ExternalRepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =====================================================================
// 枚举:ScmProvider(§4.7.2 / §19)
// =====================================================================

/// **SCM Provider 类型**(继承 §4.7.2)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScmProvider {
    /// GitHub
    Github,
    /// GitLab
    Gitlab,
    /// Gitea(Self-hosted)
    Gitea,
    /// Bitbucket
    Bitbucket,
    /// 未来扩展占位
    Future,
}

impl ScmProvider {
    /// 字符串字面量(供 DDL `provider` 列匹配)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Gitea => "gitea",
            Self::Bitbucket => "bitbucket",
            Self::Future => "future",
        }
    }
}

impl std::fmt::Display for ScmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ScmProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "gitea" => Ok(Self::Gitea),
            "bitbucket" => Ok(Self::Bitbucket),
            "future" => Ok(Self::Future),
            other => Err(format!("unknown scm provider: {other}")),
        }
    }
}

// =====================================================================
// 枚举:RepositoryOwnership(§4.7.4 / §19.2)
// =====================================================================

/// **Repository Ownership 类型**(§4.7.4)
///
/// **MVP 范围**:仅 Connected(§30.6 强化:不自建 Git Server)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RepositoryOwnership {
    /// 外部 SCM 是 SoR,平台只读
    Connected,
    /// 平台单向镜像(读优化)
    Mirrored,
    /// 平台创建并管理,外部只读
    Managed,
    /// 仅 Local Runtime 可见(实验)
    LocalOnly,
}

impl RepositoryOwnership {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Connected => "CONNECTED",
            Self::Mirrored => "MIRRORED",
            Self::Managed => "MANAGED",
            Self::LocalOnly => "LOCAL_ONLY",
        }
    }
}

impl std::fmt::Display for RepositoryOwnership {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:SyncStatus(§4.7.6 / §19)
// =====================================================================

/// **Repository 同步状态**(§4.7.6)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncStatus {
    /// 与外部 SoR 一致
    InSync,
    /// 平台落后外部(需要 Pull)
    Behind,
    /// 平台领先外部(需 Push)
    Ahead,
    /// 双向冲突
    Conflict,
    /// 同步已禁用
    Disabled,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InSync => "IN_SYNC",
            Self::Behind => "BEHIND",
            Self::Ahead => "AHEAD",
            Self::Conflict => "CONFLICT",
            Self::Disabled => "DISABLED",
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:ConflictStrategy(§4.7.6)
// =====================================================================

/// **同步冲突策略**(§4.7.6)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictStrategy {
    /// 外部 SoR,平台服从
    LatestWins,
    /// 平台 First
    FirstWins,
    /// 创建人工 Conflict 任务
    ManualReview,
    /// 慎用,需 Loop 防护
    Bidirectional {
        /// 平台侧字段
        platform_field: String,
        /// 外部侧字段
        external_field: String,
    },
}

// =====================================================================
// 枚举:PullRequestState(§7.5,§4.18.4)
// =====================================================================

/// **PR 状态机**(8 态,继承 §7.5,§4.18.4 DDL `ck_pr_state`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PullRequestState {
    /// 草稿(未 Ready for Review)
    Draft,
    /// 已开放
    Open,
    /// 审查中
    Reviewing,
    /// 变更请求
    ChangesRequested,
    /// 已批准
    Approved,
    /// 可合并(CI 通过 + Branch 同步)
    Mergeable,
    /// 已合并
    Merged,
    /// 已关闭
    Closed,
}

impl PullRequestState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Open => "OPEN",
            Self::Reviewing => "REVIEWING",
            Self::ChangesRequested => "CHANGES_REQUESTED",
            Self::Approved => "APPROVED",
            Self::Mergeable => "MERGEABLE",
            Self::Merged => "MERGED",
            Self::Closed => "CLOSED",
        }
    }

    /// 状态机迁移是否合法(继承 §7.5)
    pub fn can_transition_to(self, next: PullRequestState) -> bool {
        use PullRequestState::*;
        match (self, next) {
            // DRAFT → OPEN
            (Draft, Open) => true,
            // OPEN → REVIEWING
            (Open, Reviewing) => true,
            // REVIEWING → CHANGES_REQUESTED / APPROVED
            (Reviewing, ChangesRequested) => true,
            (Reviewing, Approved) => true,
            // CHANGES_REQUESTED → OPEN(循环)
            (ChangesRequested, Open) => true,
            // APPROVED → MERGEABLE
            (Approved, Mergeable) => true,
            // MERGEABLE → MERGED
            (Mergeable, Merged) => true,
            // 任意 → CLOSED(包含 MERGED → CLOSED 终态后再关闭语义)
            (_, Closed) => true,
            // 其他均非法
            _ => false,
        }
    }

    /// 是否为终态(MERGED / CLOSED)
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Merged | Self::Closed)
    }
}

impl std::fmt::Display for PullRequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:ReviewState(§4.18.5)
// =====================================================================

/// **Review 状态**(§4.18.5 DDL `ck_review_state`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
}

impl ReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "APPROVED",
            Self::ChangesRequested => "CHANGES_REQUESTED",
            Self::Commented => "COMMENTED",
            Self::Dismissed => "DISMISSED",
        }
    }
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:PipelineStatus(§4.18.6)
// =====================================================================

/// **Pipeline(CI)状态**(§4.18.6 DDL `ck_pipeline_status`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PipelineStatus {
    Pending,
    Running,
    Success,
    Failed,
    Canceled,
}

impl PipelineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Running => "RUNNING",
            Self::Success => "SUCCESS",
            Self::Failed => "FAILED",
            Self::Canceled => "CANCELED",
        }
    }
}

impl std::fmt::Display for PipelineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:WebhookEventType(§3.19.4)
// =====================================================================

/// **Webhook 入站事件类型**(§3.19.4)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Push
    Push,
    /// Pull Request (created / synchronized)
    PullRequest,
    /// Issue 状态变化
    Issues,
    /// Pipeline 状态变化
    Pipeline,
    /// Ping(连通性测试)
    Ping,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::PullRequest => "pull_request",
            Self::Issues => "issues",
            Self::Pipeline => "pipeline",
            Self::Ping => "ping",
        }
    }
}

impl std::fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 标准角色
// =====================================================================

/// SCM 相关标准角色常量
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

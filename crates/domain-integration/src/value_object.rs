//! Integration 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.12 (`integration` schema)
//! - `docs/specs/domain-integration-spec.md` §2 (实体清单) / §3 (基本类型)
//!
//! 集中放置强类型 ID、4 类关系枚举(`IntegrationRelationType`)、6 态
//! (`IntegrationState`)、4 类 source(`IntegrationSource`)、`ConflictStrategy` 等。

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

define_uuid_id!(IntegrationId);
define_uuid_id!(SyncStateId);
define_uuid_id!(WebhookDeliveryId);

// 标准 Tenant / Project / User ID(避免跨 crate 依赖)
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);

/// **外部系统名**(Provider 名,例如 `github` / `gitlab` / `jira` / `slack` / `notion` 等)
///
/// 这里 newtype 包装字符串,用于 `Integration.external_system_name`。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExternalSystemName(pub String);

impl ExternalSystemName {
    /// 构造
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// 内部字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExternalSystemName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// **外部系统实体 ID**(厂商侧 ID,如 GitHub Issue 编号、Slack 频道 ID)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExternalEntityId(pub String);

impl ExternalEntityId {
    /// 构造
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// 内部字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExternalEntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =====================================================================
// 枚举:IntegrationRelationType(4 类关系,§4.7.5)
// =====================================================================

/// **Integration 关系分类**(§4.7.5,基本设计表 16)
///
/// 4 类关系必须明确区分,**禁止混用**:
/// - `Link` — 只读链接(WorkItem ↔ GitHub Issue,不反向同步)
/// - `Mirror` — 单向同步(平台 → 外部,或外部 → 平台)
/// - `Bidirectional` — 双向同步 + Loop 防护
/// - `PlatformOwned` — 平台作为真相源(外部只读镜像)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntegrationRelationType {
    /// 只读链接(不反向同步)
    Link,
    /// 单向同步
    Mirror,
    /// 双向同步 + Loop 防护
    Bidirectional,
    /// 平台作为真相源
    PlatformOwned,
}

impl IntegrationRelationType {
    /// 字符串字面量(供 DDL `relation_type` 列匹配)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Link => "LINK",
            Self::Mirror => "MIRROR",
            Self::Bidirectional => "BIDIRECTIONAL",
            Self::PlatformOwned => "PLATFORM_OWNED",
        }
    }

    /// 是否需要 Loop 防护(仅 Bidirectional 必须有 idempotency_key + sync_token)
    pub fn requires_loop_guard(&self) -> bool {
        matches!(self, Self::Bidirectional)
    }

    /// 是否需要 sync_token(Mirror / Bidirectional / PlatformOwned 需要;Link 只读不需要)
    pub fn requires_sync_token(&self) -> bool {
        !matches!(self, Self::Link)
    }
}

impl std::fmt::Display for IntegrationRelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:IntegrationState(6 态,§4.12)
// =====================================================================

/// **Integration 状态**(§4.12 DDL `ck_integration_state`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntegrationState {
    /// 初始化中(创建后第一次同步未完成)
    Initializing,
    /// 活跃(可同步)
    Active,
    /// 暂停(用户主动暂停)
    Paused,
    /// 错误(最近一次同步失败)
    Error,
    /// 禁用(管理员禁用)
    Disabled,
}

impl IntegrationState {
    /// 字符串字面量
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initializing => "INITIALIZING",
            Self::Active => "ACTIVE",
            Self::Paused => "PAUSED",
            Self::Error => "ERROR",
            Self::Disabled => "DISABLED",
        }
    }
}

impl std::fmt::Display for IntegrationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:IntegrationSource(4 类 source,§4.12)
// =====================================================================

/// **Integration 源系统分类**(§4.12)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntegrationSource {
    /// SCM(GitHub / GitLab / Gitea / Bitbucket)
    Scm,
    /// Project Management(Jira / Linear / Trello)
    ProjectManagement,
    /// Communication(Slack / Discord / Teams)
    Communication,
    /// 其它(Notion / Confluence / 自定义)
    Other,
}

impl IntegrationSource {
    /// 字符串字面量
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scm => "SCM",
            Self::ProjectManagement => "PROJECT_MANAGEMENT",
            Self::Communication => "COMMUNICATION",
            Self::Other => "OTHER",
        }
    }
}

impl std::fmt::Display for IntegrationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:ConflictStrategy(§4.7.6,继承 SCM 模式)
// =====================================================================

/// **Integration 同步冲突策略**(§4.7.6,4 种)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictStrategy {
    /// 外部 SoR,平台服从
    LatestWins,
    /// 平台 First
    FirstWins,
    /// 创建人工 Conflict 任务(默认)
    ManualReview,
    /// 慎用,需 Loop 防护,仅 Bidirectional 可用
    Bidirectional {
        /// 平台侧字段
        platform_field: String,
        /// 外部侧字段
        external_field: String,
    },
}

impl ConflictStrategy {
    /// 字符串字面量(供 DDL `conflict_strategy` 列匹配)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LatestWins => "LATEST_WINS",
            Self::FirstWins => "FIRST_WINS",
            Self::ManualReview => "MANUAL_REVIEW",
            Self::Bidirectional { .. } => "BIDIRECTIONAL",
        }
    }
}

impl std::fmt::Display for ConflictStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:SyncOutcome(§4.12,SyncState 实体)
// =====================================================================

/// **单次同步结果**(§4.12)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncOutcome {
    /// 成功
    Success,
    /// 部分成功(部分记录同步失败)
    PartialSuccess,
    /// 失败
    Failed,
    /// 跳过(如 Bidirectional Loop 防护命中)
    Skipped,
}

impl SyncOutcome {
    /// 字符串字面量
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::PartialSuccess => "PARTIAL_SUCCESS",
            Self::Failed => "FAILED",
            Self::Skipped => "SKIPPED",
        }
    }
}

impl std::fmt::Display for SyncOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 标准角色
// =====================================================================

/// Integration 相关标准角色常量
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

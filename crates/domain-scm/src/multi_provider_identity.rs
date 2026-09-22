//! Multi-Provider Identity Validation module
//!
//! 详细 spec: ULYS-172 / FR-ORCA-043 (P3, per 2026-09-22 D-Boy 决策 B — GitHub only)
//! 上游基本设计: docs/basic-design.md §15 (Multi-Provider Identity Validation)
//! 数据设计: docs/data-design.md §4.x (ProviderIdentity / IdentityLink schema)
//! API 设计: docs/api-design.md §3.x (ProviderIdentity CRUD + IdentityValidation 状态机)
//!
//! ## Scope (per 2026-09-22 D-Boy 决策 B)
//!
//! MVP 仅实装 **GitHub** provider;Linear / Jira / GitLab 留作 future-extension 占位
//! (enum variant 存在,validation/link 路径返回 `UnsupportedProvider` 错误)。
//!
//! ## 职责
//!
//! - **ProviderIdentity**:同一用户在 4 个 provider(GitHub / Linear / Jira / GitLab)
//!   的外部账号注册与存储
//! - **IdentityLink**:把多个 `ProviderIdentity` 关联到同一本地 `UserId`
//!   (1 个本地用户 ↔ N 个外部 provider 账号)
//! - **IdentityValidation**:对一组 linked identity 跑一致性验证(邮箱 / 登录名 / 显示名)
//!   产出 `Pending | Validated | Mismatch | Revoked` 状态
//! - **Cross-provider Consistency**:`propagate_identity_change` 把"主 identity 的变更"
//!   翻译成下游 provider 的"待同步 delta",记录到 IdentityPropagation 实体(待
//!   application 层 outbound adapter 真正调用 provider API)
//!
//! ## 关键不变量(INV-MPI-01~05,共 5 条)
//!
//! - **INV-MPI-01**:`IdentityLink` 必带 `tenant_id`,跨 tenant 拒绝(对齐 INV-SCM-04)
//! - **INV-MPI-02**:MVP 仅 `Github` provider 可实装;其他 provider 任何
//!   `link_identities` / `validate_identity` 操作必须返回 `UnsupportedProvider`
//! - **INV-MPI-03**:同一 `(ProviderIdentity.provider, external_id)` 在
//!   同一 `tenant_id` 下唯一(`register_provider_identity` 重复时返回 `Duplicate`)
//! - **INV-MPI-04**:`IdentityValidation` 状态机严格按 `Pending → Validated | Mismatch → Revoked`
//!   (Validated 不能直接 Revoked;Mismatch 可 Validated 也可 Revoked)
//! - **INV-MPI-05**:`propagate_identity_change` 不真正调用 provider API,
//!   只产出 `IdentityPropagation` 实体(delta + 状态 = `Pending`),由 application
//!   层 adapter 负责发送;防止 domain 层耦合 provider SDK(对齐 INV-SCM-01)
//!
//! ## 状态机(IdentityValidation,§15.4)
//!
//! ```text
//!        register_link → Pending
//!                          │
//!        validate_identity │
//!                          ▼
//!               ┌── Validated ──┐
//!               │               │
//!               ▼               ▼
//!            Mismatch       Revoked
//!               │
//!               ▼ (再 validate 可回 Validated)
//!            Validated
//! ```
//!
//! Lead 责任: scm Lead (FR-ORCA-043 / 2026-09-22)

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use tokio::sync::RwLock;
use crate::{TenantId, UserId};

// =====================================================================
// 强类型 ID(复用 lib.rs 的 macro_rules)
// =====================================================================

define_uuid_id!(ProviderIdentityId);
define_uuid_id!(IdentityLinkId);
define_uuid_id!(IdentityValidationId);
define_uuid_id!(IdentityPropagationId);

// =====================================================================
// Provider 枚举(对齐 ScmProvider 命名风格,INV-MPI-02)
// =====================================================================

/// **Identity Provider**(per FR-ORCA-043 / 2026-09-22 决策 B)
///
/// MVP 仅 `Github` 可实装;`Linear` / `Jira` / `Gitlab` 留作 future-extension
/// (enum variant 存在但实装返回 `UnsupportedProvider`)。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityProvider {
    /// GitHub(MVP,per 2026-09-22 D-Boy 决策 B)
    Github,
    /// Linear(future-extension 占位)
    Linear,
    /// Jira(future-extension 占位)
    Jira,
    /// GitLab(future-extension 占位,注:与 SCM Provider 不同 — 本 enum 专指 issue tracker 身份)
    Gitlab,
}

impl IdentityProvider {
    /// 是否 MVP 可实装(per 2026-09-22 D-Boy 决策 B = 仅 GitHub)
    pub fn is_mvp_supported(&self) -> bool {
        matches!(self, Self::Github)
    }

    /// 是否 future-extension(Linear / Jira / GitLab per 2026-09-22 D-Boy 决策 B)
    pub fn is_future_extension(&self) -> bool {
        !self.is_mvp_supported()
    }
}

impl Default for IdentityProvider {
    fn default() -> Self {
        Self::Github
    }
}

impl std::fmt::Display for IdentityProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Github => "github",
            Self::Linear => "linear",
            Self::Jira => "jira",
            Self::Gitlab => "gitlab",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 值对象
// =====================================================================

/// **ProviderIdentity**:1 条记录 = 1 个用户在 1 个 provider 上的外部账号
///
/// 由 `(tenant_id, provider, external_id)` 三元组唯一标识(INV-MPI-03)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderIdentity {
    /// 主键
    pub id: ProviderIdentityId,
    /// Tenant 隔离(必填,INV-MPI-01)
    pub tenant_id: TenantId,
    /// 关联本地用户(可为 None — 未 link 状态)
    pub local_user_id: Option<UserId>,
    /// 外部 provider
    pub provider: IdentityProvider,
    /// 厂商侧唯一 ID(如 GitHub user id "1234567" 的字符串形式)
    pub external_id: String,
    /// 厂商侧登录名(如 "octocat")
    pub external_login: String,
    /// 厂商侧显示名
    pub display_name: String,
    /// 邮箱(可空 — 部分 provider 不暴露)
    pub email: Option<String>,
    /// 厂商侧 profile URL
    pub url: String,
    /// 注册时间
    pub created_at: DateTime<Utc>,
    /// 最近一次 validated 时间
    pub last_validated_at: Option<DateTime<Utc>>,
    /// 锁定版本(乐观锁)
    pub lock_version: u32,
}

/// **IdentityLink**:把 N 个 `ProviderIdentity` 关联到 1 个本地 `UserId`
///
/// 同一 link 下 identity 必须共享 tenant(INV-MPI-01)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityLink {
    /// 主键
    pub id: IdentityLinkId,
    /// Tenant 隔离(必填,INV-MPI-01)
    pub tenant_id: TenantId,
    /// 本地用户
    pub local_user_id: UserId,
    /// 关联的 provider identities(至少 1 个)
    pub identity_ids: Vec<ProviderIdentityId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 锁定版本
    pub lock_version: u32,
}

/// **ValidationStatus**(状态机,INV-MPI-04)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// 已注册,尚未 validate
    Pending,
    /// 一致(email/login/display_name 全部匹配)
    Validated,
    /// 不一致(具体差异在 `mismatch_fields`)
    Mismatch,
    /// 已撤销(provider 端账号注销 / token 失效)
    Revoked,
}

impl Default for ValidationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Validated => "validated",
            Self::Mismatch => "mismatch",
            Self::Revoked => "revoked",
        };
        f.write_str(s)
    }
}

/// **IdentityValidation**:对一组 linked identity 跑一致性验证的结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityValidation {
    /// 主键
    pub id: IdentityValidationId,
    /// Tenant 隔离(必填)
    pub tenant_id: TenantId,
    /// 关联 link
    pub link_id: IdentityLinkId,
    /// 验证的 identities(快照,避免 link 改动后追溯)
    pub identity_ids: Vec<ProviderIdentityId>,
    /// 状态
    pub status: ValidationStatus,
    /// Mismatch 时具体差异字段(如 `["email", "display_name"]`)
    pub mismatch_fields: Vec<String>,
    /// 验证时间
    pub validated_at: DateTime<Utc>,
    /// 锁定版本
    pub lock_version: u32,
}

impl IdentityValidation {
    /// 是否终态(Revoked 是终态;Validated/Mismatch 可再 validate)
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, ValidationStatus::Revoked)
    }

    /// 状态机迁移是否合法(per INV-MPI-04)
    pub fn can_transition_to(&self, next: ValidationStatus) -> bool {
        use ValidationStatus::*;
        match (self.status, next) {
            // Pending → Validated | Mismatch | Revoked
            (Pending, Validated) => true,
            (Pending, Mismatch) => true,
            (Pending, Revoked) => true,
            // Validated → Mismatch | Revoked(发现新不一致 / 撤销)
            (Validated, Mismatch) => true,
            (Validated, Revoked) => true,
            // Mismatch → Validated | Revoked(修复后再 validate / 撤销)
            (Mismatch, Validated) => true,
            (Mismatch, Revoked) => true,
            // Revoked 是终态,不能迁出
            (Revoked, _) => false,
            // 同状态不变更合法(no-op)
            (a, b) if a == b => true,
            // 其他非法
            _ => false,
        }
    }
}

/// **IdentityPropagation**:`propagate_identity_change` 产出的下游同步 delta
///
/// domain 层不真正调用 provider API(INV-MPI-05);只产出此实体由 application 层 adapter 发送。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityPropagation {
    /// 主键
    pub id: IdentityPropagationId,
    /// Tenant 隔离
    pub tenant_id: TenantId,
    /// 关联 link
    pub link_id: IdentityLinkId,
    /// 源头 identity(主 identity,变更发生处)
    pub source_identity_id: ProviderIdentityId,
    /// 待同步的目标 identities(同 link 下其他 provider)
    pub target_identity_ids: Vec<ProviderIdentityId>,
    /// 变更字段(由 propagation plan 决定)
    pub changed_fields: Vec<PropagatedField>,
    /// 状态
    pub status: PropagationStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 锁定版本
    pub lock_version: u32,
}

/// **PropagatedField**:单个待同步字段的 delta
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropagatedField {
    /// 字段名(`display_name` / `email`)
    pub field: String,
    /// 源头新值
    pub source_value: String,
    /// 目标旧值(在 validate 时记录)
    pub target_old_value: Option<String>,
}

/// **PropagationStatus**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationStatus {
    /// 已产出,待 application 层 adapter 发送
    Pending,
    /// 已发送,等 provider ack
    InFlight,
    /// provider 确认成功
    Completed,
    /// provider 拒绝 / 失败(可重试)
    Failed,
}

impl Default for PropagationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for PropagationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::InFlight => "in_flight",
            Self::Completed => "completed",
            Self::Failed => "failed",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 命令 / 查询输入
// =====================================================================

/// `register_provider_identity` 命令输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterProviderIdentityCommand {
    /// Tenant(actor.tenant_id 必须等于此值,INV-MPI-01)
    pub tenant_id: TenantId,
    /// 本地用户(可为 None — 允许先注册再 link)
    pub local_user_id: Option<UserId>,
    /// Provider
    pub provider: IdentityProvider,
    /// 厂商侧 ID
    pub external_id: String,
    /// 厂商侧登录名
    pub external_login: String,
    /// 厂商侧显示名
    pub display_name: String,
    /// 邮箱(可空)
    pub email: Option<String>,
    /// profile URL
    pub url: String,
}

/// `link_identities` 命令输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkIdentitiesCommand {
    /// Tenant
    pub tenant_id: TenantId,
    /// 本地用户
    pub local_user_id: UserId,
    /// 要关联的 identities(至少 1 个)
    pub identity_ids: Vec<ProviderIdentityId>,
}

/// `validate_identity` 命令输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateIdentityCommand {
    /// Tenant
    pub tenant_id: TenantId,
    /// Link ID
    pub link_id: IdentityLinkId,
}

/// `propagate_identity_change` 命令输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagateIdentityChangeCommand {
    /// Tenant
    pub tenant_id: TenantId,
    /// Link ID
    pub link_id: IdentityLinkId,
    /// 源头 identity(主 identity)
    pub source_identity_id: ProviderIdentityId,
    /// 源头新值(display_name / email)
    pub new_display_name: Option<String>,
    /// 源头新 email(可选)
    pub new_email: Option<String>,
}

/// `revoke_identity` 命令输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeIdentityCommand {
    /// Tenant
    pub tenant_id: TenantId,
    /// Link ID
    pub link_id: IdentityLinkId,
    /// 撤销原因(进 audit log)
    pub reason: String,
}

// =====================================================================
// 错误(对齐 ScmError 命名风格)
// =====================================================================

/// Multi-Provider Identity 领域错误
#[derive(Debug, Error)]
pub enum MultiProviderIdentityError {
    /// 404 Identity / Link / Validation 不存在
    #[error("identity not found: {0}")]
    NotFound(String),
    /// 422 跨 tenant 拒绝(INV-MPI-01)
    #[error("tenant mismatch: {0}")]
    TenantMismatch(String),
    /// 409 `(provider, external_id)` 重复(INV-MPI-03)
    #[error("duplicate identity: provider={provider}, external_id={external_id}")]
    Duplicate {
        /// 重复的 provider
        provider: IdentityProvider,
        /// 重复的厂商侧 ID
        external_id: String,
    },
    /// 422 Provider 未实装(INV-MPI-02,2026-09-22 决策 B 仅 GitHub)
    #[error("provider not supported in MVP: {0} (per 2026-09-22 decision B, GitHub only)")]
    UnsupportedProvider(IdentityProvider),
    /// 422 状态机非法(INV-MPI-04)
    #[error("invalid status transition: {from} -> {to}")]
    InvalidTransition {
        /// 起始状态
        from: ValidationStatus,
        /// 目标状态
        to: ValidationStatus,
    },
    /// 422 缺少必填字段
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// 422 Link 为空
    #[error("link must contain at least 1 identity")]
    EmptyLink,
    /// 5xx
    #[error("internal: {0}")]
    Internal(String),
}

impl MultiProviderIdentityError {
    /// 返回错误对应的短代码(对齐 ScmError::code)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "MPI_NOT_FOUND",
            Self::TenantMismatch(_) => "MPI_TENANT_MISMATCH",
            Self::Duplicate { .. } => "MPI_DUPLICATE",
            Self::UnsupportedProvider(_) => "MPI_UNSUPPORTED_PROVIDER",
            Self::InvalidTransition { .. } => "MPI_INVALID_TRANSITION",
            Self::ValidationFailed(_) => "MPI_VALIDATION_FAILED",
            Self::EmptyLink => "MPI_EMPTY_LINK",
            Self::Internal(_) => "MPI_INTERNAL",
        }
    }
}

// =====================================================================
// 端口(Command / Query,对齐 ScmCommandPort / ScmQueryPort 风格)
// =====================================================================

/// 命令端口(M 类 Master 写操作)
#[async_trait]
pub trait MultiProviderIdentityCommandPort: Send + Sync {
    /// 注册 1 个 provider identity(返回新建 identity)
    async fn register_provider_identity(
        &self,
        cmd: RegisterProviderIdentityCommand,
        actor: ActorContext,
    ) -> Result<ProviderIdentity, MultiProviderIdentityError>;

    /// 把 N 个 identities 关联到 1 个本地用户(返回新建 link)
    async fn link_identities(
        &self,
        cmd: LinkIdentitiesCommand,
        actor: ActorContext,
    ) -> Result<IdentityLink, MultiProviderIdentityError>;

    /// 对 1 个 link 跑一致性验证(返回 validation 结果)
    async fn validate_identity(
        &self,
        cmd: ValidateIdentityCommand,
        actor: ActorContext,
    ) -> Result<IdentityValidation, MultiProviderIdentityError>;

    /// 把源头 identity 的变更产出 propagation plan(不真正调用 provider,INV-MPI-05)
    async fn propagate_identity_change(
        &self,
        cmd: PropagateIdentityChangeCommand,
        actor: ActorContext,
    ) -> Result<IdentityPropagation, MultiProviderIdentityError>;

    /// 撤销 1 个 link(INV-MPI-04 终态)
    async fn revoke_identity(
        &self,
        cmd: RevokeIdentityCommand,
        actor: ActorContext,
    ) -> Result<IdentityValidation, MultiProviderIdentityError>;
}

/// 查询端口(Q 类读操作)
#[async_trait]
pub trait MultiProviderIdentityQueryPort: Send + Sync {
    /// 按 ID 查 identity
    async fn get_identity(
        &self,
        tenant_id: TenantId,
        id: ProviderIdentityId,
        actor: ActorContext,
    ) -> Result<ProviderIdentity, MultiProviderIdentityError>;

    /// 按 `(provider, external_id)` 查 identity
    async fn find_identity_by_external(
        &self,
        tenant_id: TenantId,
        provider: IdentityProvider,
        external_id: &str,
        actor: ActorContext,
    ) -> Result<Option<ProviderIdentity>, MultiProviderIdentityError>;

    /// 按本地用户 ID 查所有 links
    async fn list_links_by_local_user(
        &self,
        tenant_id: TenantId,
        local_user_id: UserId,
        actor: ActorContext,
    ) -> Result<Vec<IdentityLink>, MultiProviderIdentityError>;

    /// 按 link ID 查所有 validations
    async fn list_validations_by_link(
        &self,
        tenant_id: TenantId,
        link_id: IdentityLinkId,
        actor: ActorContext,
    ) -> Result<Vec<IdentityValidation>, MultiProviderIdentityError>;
}

/// 复合端口(对齐 ScmPort)
pub trait MultiProviderIdentityPort: Send + Sync {
    /// 命令端口
    fn commands(&self) -> Arc<dyn MultiProviderIdentityCommandPort>;
    /// 查询端口
    fn queries(&self) -> Arc<dyn MultiProviderIdentityQueryPort>;
}

// =====================================================================
// In-Memory 实现(对齐 InMemoryScmService 风格,for test/dev)
// =====================================================================

/// In-Memory Multi-Provider Identity Service(测试 / 开发用,for IT/UT)
#[derive(Clone)]
pub struct InMemoryMultiProviderIdentityService {
    #[allow(missing_docs)]
    /// ProviderIdentity 表(主键 → 实体)
    identities: Arc<RwLock<HashMap<ProviderIdentityId, ProviderIdentity>>>,
    #[allow(missing_docs)]
    /// IdentityLink 表
    links: Arc<RwLock<HashMap<IdentityLinkId, IdentityLink>>>,
    #[allow(missing_docs)]
    /// IdentityValidation 表
    validations: Arc<RwLock<HashMap<IdentityValidationId, IdentityValidation>>>,
    #[allow(missing_docs)]
    /// IdentityPropagation 表
    propagations: Arc<RwLock<HashMap<IdentityPropagationId, IdentityPropagation>>>,
    #[allow(missing_docs)]
    /// `(tenant, provider, external_id)` → identity_id 索引(INV-MPI-03 去重)
    external_index: Arc<RwLock<HashMap<(TenantId, IdentityProvider, String), ProviderIdentityId>>>,
}

impl InMemoryMultiProviderIdentityService {
    /// 新建 in-memory service(测试 / 开发)
    pub fn new_for_test() -> Self {
        Self {
            identities: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            propagations: Arc::new(RwLock::new(HashMap::new())),
            external_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 校验 actor.tenant_id == cmd.tenant_id(INV-MPI-01)
    ///
    /// tenant_admin 角色可跨 tenant 操作(对齐 star-context `ActorContext::is_tenant_admin`,
    /// 与 INV-SCM-04 / INV-ID-01 一致)。
    fn check_tenant(
        &self,
        actor: &ActorContext,
        expected: TenantId,
    ) -> Result<(), MultiProviderIdentityError> {
        if actor.is_tenant_admin() {
            return Ok(());
        }
        if actor.tenant_id != expected.0 {
            return Err(MultiProviderIdentityError::TenantMismatch(format!(
                "actor.tenant_id={} != cmd.tenant_id={}",
                actor.tenant_id, expected.0
            )));
        }
        Ok(())
    }

    /// 校验 provider 在 MVP 可实装(INV-MPI-02)
    fn check_mvp_provider(p: IdentityProvider) -> Result<(), MultiProviderIdentityError> {
        if p.is_future_extension() {
            return Err(MultiProviderIdentityError::UnsupportedProvider(p));
        }
        Ok(())
    }
}

#[async_trait]
impl MultiProviderIdentityCommandPort for InMemoryMultiProviderIdentityService {
    async fn register_provider_identity(
        &self,
        cmd: RegisterProviderIdentityCommand,
        actor: ActorContext,
    ) -> Result<ProviderIdentity, MultiProviderIdentityError> {
        self.check_tenant(&actor, cmd.tenant_id)?;
        Self::check_mvp_provider(cmd.provider)?;

        // INV-MPI-03:同 `(tenant, provider, external_id)` 必须唯一
        let key = (cmd.tenant_id, cmd.provider, cmd.external_id.clone());
        {
            let idx = self.external_index.read().await;
            if idx.contains_key(&key) {
                return Err(MultiProviderIdentityError::Duplicate {
                    provider: cmd.provider,
                    external_id: cmd.external_id.clone(),
                });
            }
        }

        // 基础字段校验
        if cmd.external_id.trim().is_empty() {
            return Err(MultiProviderIdentityError::ValidationFailed(
                "external_id must not be empty".to_string(),
            ));
        }
        if cmd.external_login.trim().is_empty() {
            return Err(MultiProviderIdentityError::ValidationFailed(
                "external_login must not be empty".to_string(),
            ));
        }

        let identity = ProviderIdentity {
            id: ProviderIdentityId::new(),
            tenant_id: cmd.tenant_id,
            local_user_id: cmd.local_user_id,
            provider: cmd.provider,
            external_id: cmd.external_id.clone(),
            external_login: cmd.external_login,
            display_name: cmd.display_name,
            email: cmd.email,
            url: cmd.url,
            created_at: Utc::now(),
            last_validated_at: None,
            lock_version: 1,
        };

        let id = identity.id;
        {
            let mut store = self.identities.write().await;
            store.insert(id, identity.clone());
        }
        {
            let mut idx = self.external_index.write().await;
            idx.insert(key, id);
        }

        Ok(identity)
    }

    async fn link_identities(
        &self,
        cmd: LinkIdentitiesCommand,
        actor: ActorContext,
    ) -> Result<IdentityLink, MultiProviderIdentityError> {
        self.check_tenant(&actor, cmd.tenant_id)?;

        // INV-MPI-02:link 内任一 identity 是 future-extension provider → 整体拒绝
        if cmd.identity_ids.is_empty() {
            return Err(MultiProviderIdentityError::EmptyLink);
        }

        let identities = self.identities.read().await;
        let mut resolved_providers: Vec<IdentityProvider> = Vec::new();
        for id in &cmd.identity_ids {
            let identity = identities.get(id).ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!("identity:{}", id.as_uuid()))
            })?;
            // INV-MPI-01:link 内所有 identity 必须同 tenant
            if identity.tenant_id != cmd.tenant_id {
                return Err(MultiProviderIdentityError::TenantMismatch(format!(
                    "identity {} tenant mismatch",
                    id.as_uuid()
                )));
            }
            resolved_providers.push(identity.provider);
        }
        drop(identities);

        for p in &resolved_providers {
            Self::check_mvp_provider(*p)?;
        }

        let link = IdentityLink {
            id: IdentityLinkId::new(),
            tenant_id: cmd.tenant_id,
            local_user_id: cmd.local_user_id,
            identity_ids: cmd.identity_ids,
            created_at: Utc::now(),
            lock_version: 1,
        };

        let id = link.id;
        {
            let mut store = self.links.write().await;
            store.insert(id, link.clone());
        }
        // 同步更新 identities[].local_user_id
        let mut identities = self.identities.write().await;
        for pid in &link.identity_ids {
            if let Some(p) = identities.get_mut(pid) {
                if p.local_user_id.is_none() {
                    p.local_user_id = Some(link.local_user_id);
                    p.lock_version += 1;
                }
            }
        }

        Ok(link)
    }

    async fn validate_identity(
        &self,
        cmd: ValidateIdentityCommand,
        actor: ActorContext,
    ) -> Result<IdentityValidation, MultiProviderIdentityError> {
        self.check_tenant(&actor, cmd.tenant_id)?;

        let links = self.links.read().await;
        let link = links
            .get(&cmd.link_id)
            .ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!("link:{}", cmd.link_id.as_uuid()))
            })?
            .clone();
        drop(links);

        if link.tenant_id != cmd.tenant_id {
            return Err(MultiProviderIdentityError::TenantMismatch(format!(
                "link {} tenant mismatch",
                cmd.link_id.as_uuid()
            )));
        }

        // MVP 仅 GitHub:link 内任一 identity 是 future-extension provider → 拒绝(INV-MPI-02)
        let identities = self.identities.read().await;
        let mut snapshots: Vec<ProviderIdentity> = Vec::new();
        for pid in &link.identity_ids {
            let id = identities.get(pid).ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!("identity:{}", pid.as_uuid()))
            })?;
            if id.provider.is_future_extension() {
                return Err(MultiProviderIdentityError::UnsupportedProvider(id.provider));
            }
            snapshots.push(id.clone());
        }
        drop(identities);

        // 取最后一个 identity 的值作为"当前"值(简化模型;
        // 真实实现应按 provider 分别 query 最新 profile)
        let latest = snapshots
            .last()
            .ok_or(MultiProviderIdentityError::EmptyLink)?
            .clone();

        // 简单一致性检查:邮箱非空 → 必须匹配;display_name 非空 → 必须匹配
        let mut mismatch_fields: Vec<String> = Vec::new();
        for s in &snapshots {
            if let Some(em) = &s.email {
                if !em.eq_ignore_ascii_case(&latest.email.clone().unwrap_or_default()) {
                    if !mismatch_fields.contains(&"email".to_string()) {
                        mismatch_fields.push("email".to_string());
                    }
                }
            }
            if s.display_name != latest.display_name
                && !mismatch_fields.contains(&"display_name".to_string())
            {
                mismatch_fields.push("display_name".to_string());
            }
        }
        let new_status = if mismatch_fields.is_empty() {
            ValidationStatus::Validated
        } else {
            ValidationStatus::Mismatch
        };

        // 状态机迁移(INV-MPI-04):取最近一条 validation 作为 prev,若无则视为 Pending
        let validations = self.validations.read().await;
        let prev_status = validations
            .values()
            .filter(|v| v.link_id == cmd.link_id)
            .max_by_key(|v| v.validated_at)
            .map(|v| v.status);
        drop(validations);

        let prev_status = prev_status.unwrap_or(ValidationStatus::Pending);
        // 构造临时 prev 对象做 can_transition_to 校验
        let prev = IdentityValidation {
            id: IdentityValidationId::new(),
            tenant_id: cmd.tenant_id,
            link_id: cmd.link_id,
            identity_ids: link.identity_ids.clone(),
            status: prev_status,
            mismatch_fields: vec![],
            validated_at: Utc::now(),
            lock_version: 0,
        };
        if !prev.can_transition_to(new_status) {
            return Err(MultiProviderIdentityError::InvalidTransition {
                from: prev.status,
                to: new_status,
            });
        }

        let validation = IdentityValidation {
            id: IdentityValidationId::new(),
            tenant_id: cmd.tenant_id,
            link_id: cmd.link_id,
            identity_ids: link.identity_ids.clone(),
            status: new_status,
            mismatch_fields,
            validated_at: Utc::now(),
            lock_version: 1,
        };

        let id = validation.id;
        // 同步更新 identities[].last_validated_at
        let mut identities = self.identities.write().await;
        for pid in &validation.identity_ids {
            if let Some(p) = identities.get_mut(pid) {
                p.last_validated_at = Some(validation.validated_at);
                p.lock_version += 1;
            }
        }
        drop(identities);

        let mut validations = self.validations.write().await;
        validations.insert(id, validation.clone());

        Ok(validation)
    }

    async fn propagate_identity_change(
        &self,
        cmd: PropagateIdentityChangeCommand,
        actor: ActorContext,
    ) -> Result<IdentityPropagation, MultiProviderIdentityError> {
        self.check_tenant(&actor, cmd.tenant_id)?;

        let links = self.links.read().await;
        let link = links
            .get(&cmd.link_id)
            .ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!("link:{}", cmd.link_id.as_uuid()))
            })?
            .clone();
        drop(links);

        if link.tenant_id != cmd.tenant_id {
            return Err(MultiProviderIdentityError::TenantMismatch(format!(
                "link {} tenant mismatch",
                cmd.link_id.as_uuid()
            )));
        }

        let identities = self.identities.read().await;
        let source = identities
            .get(&cmd.source_identity_id)
            .ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!(
                    "source identity:{}",
                    cmd.source_identity_id.as_uuid()
                ))
            })?
            .clone();
        if source.tenant_id != cmd.tenant_id {
            return Err(MultiProviderIdentityError::TenantMismatch(
                "source identity tenant mismatch".to_string(),
            ));
        }
        if source.provider.is_future_extension() {
            return Err(MultiProviderIdentityError::UnsupportedProvider(source.provider));
        }

        // INV-MPI-05:仅产 delta,不真正调 provider
        let mut target_ids: Vec<ProviderIdentityId> = Vec::new();
        let mut changed_fields: Vec<PropagatedField> = Vec::new();

        for pid in &link.identity_ids {
            if *pid == cmd.source_identity_id {
                continue;
            }
            let target = identities.get(pid).ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!("target identity:{}", pid.as_uuid()))
            })?;
            // MVP 仅 GitHub;若目标是非 GitHub → 跳过(INV-MPI-02)
            if target.provider.is_future_extension() {
                continue;
            }
            target_ids.push(*pid);

            // 比较 display_name
            if let Some(new_dn) = &cmd.new_display_name {
                if &target.display_name != new_dn {
                    changed_fields.push(PropagatedField {
                        field: "display_name".to_string(),
                        source_value: new_dn.clone(),
                        target_old_value: Some(target.display_name.clone()),
                    });
                }
            }
            // 比较 email
            if let Some(new_em) = &cmd.new_email {
                if target.email.as_deref() != Some(new_em.as_str()) {
                    changed_fields.push(PropagatedField {
                        field: "email".to_string(),
                        source_value: new_em.clone(),
                        target_old_value: target.email.clone(),
                    });
                }
            }
        }
        drop(identities);

        let propagation = IdentityPropagation {
            id: IdentityPropagationId::new(),
            tenant_id: cmd.tenant_id,
            link_id: cmd.link_id,
            source_identity_id: cmd.source_identity_id,
            target_identity_ids: target_ids,
            changed_fields,
            status: PropagationStatus::Pending,
            created_at: Utc::now(),
            lock_version: 1,
        };

        let id = propagation.id;
        let mut store = self.propagations.write().await;
        store.insert(id, propagation.clone());

        Ok(propagation)
    }

    async fn revoke_identity(
        &self,
        cmd: RevokeIdentityCommand,
        actor: ActorContext,
    ) -> Result<IdentityValidation, MultiProviderIdentityError> {
        self.check_tenant(&actor, cmd.tenant_id)?;

        let links = self.links.read().await;
        let link = links
            .get(&cmd.link_id)
            .ok_or_else(|| {
                MultiProviderIdentityError::NotFound(format!("link:{}", cmd.link_id.as_uuid()))
            })?
            .clone();
        drop(links);

        if link.tenant_id != cmd.tenant_id {
            return Err(MultiProviderIdentityError::TenantMismatch(format!(
                "link {} tenant mismatch",
                cmd.link_id.as_uuid()
            )));
        }

        let validations = self.validations.read().await;
        let prev_status = validations
            .values()
            .filter(|v| v.link_id == cmd.link_id)
            .max_by_key(|v| v.validated_at)
            .map(|v| v.status)
            .unwrap_or(ValidationStatus::Pending);
        drop(validations);

        let prev = IdentityValidation {
            id: IdentityValidationId::new(),
            tenant_id: cmd.tenant_id,
            link_id: cmd.link_id,
            identity_ids: link.identity_ids.clone(),
            status: prev_status,
            mismatch_fields: vec![],
            validated_at: Utc::now(),
            lock_version: 0,
        };
        if !prev.can_transition_to(ValidationStatus::Revoked) {
            return Err(MultiProviderIdentityError::InvalidTransition {
                from: prev.status,
                to: ValidationStatus::Revoked,
            });
        }

        let validation = IdentityValidation {
            id: IdentityValidationId::new(),
            tenant_id: cmd.tenant_id,
            link_id: cmd.link_id,
            identity_ids: link.identity_ids.clone(),
            status: ValidationStatus::Revoked,
            mismatch_fields: vec![cmd.reason.clone()],
            validated_at: Utc::now(),
            lock_version: 1,
        };

        let id = validation.id;
        let mut store = self.validations.write().await;
        store.insert(id, validation.clone());

        Ok(validation)
    }
}

#[async_trait]
impl MultiProviderIdentityQueryPort for InMemoryMultiProviderIdentityService {
    async fn get_identity(
        &self,
        tenant_id: TenantId,
        id: ProviderIdentityId,
        _actor: ActorContext,
    ) -> Result<ProviderIdentity, MultiProviderIdentityError> {
        let store = self.identities.read().await;
        let identity = store
            .get(&id)
            .ok_or_else(|| MultiProviderIdentityError::NotFound(format!("identity:{}", id.as_uuid())))?
            .clone();
        if identity.tenant_id != tenant_id {
            return Err(MultiProviderIdentityError::TenantMismatch(format!(
                "identity {} tenant mismatch",
                id.as_uuid()
            )));
        }
        Ok(identity)
    }

    async fn find_identity_by_external(
        &self,
        tenant_id: TenantId,
        provider: IdentityProvider,
        external_id: &str,
        _actor: ActorContext,
    ) -> Result<Option<ProviderIdentity>, MultiProviderIdentityError> {
        let idx = self.external_index.read().await;
        let key = (tenant_id, provider, external_id.to_string());
        let Some(id) = idx.get(&key).copied() else {
            return Ok(None);
        };
        drop(idx);
        let store = self.identities.read().await;
        Ok(store.get(&id).cloned())
    }

    async fn list_links_by_local_user(
        &self,
        tenant_id: TenantId,
        local_user_id: UserId,
        _actor: ActorContext,
    ) -> Result<Vec<IdentityLink>, MultiProviderIdentityError> {
        let store = self.links.read().await;
        let mut out: Vec<IdentityLink> = store
            .values()
            .filter(|l| l.tenant_id == tenant_id && l.local_user_id == local_user_id)
            .cloned()
            .collect();
        out.sort_by_key(|l| l.created_at);
        Ok(out)
    }

    async fn list_validations_by_link(
        &self,
        tenant_id: TenantId,
        link_id: IdentityLinkId,
        _actor: ActorContext,
    ) -> Result<Vec<IdentityValidation>, MultiProviderIdentityError> {
        let store = self.validations.read().await;
        let mut out: Vec<IdentityValidation> = store
            .values()
            .filter(|v| v.tenant_id == tenant_id && v.link_id == link_id)
            .cloned()
            .collect();
        out.sort_by_key(|v| v.validated_at);
        Ok(out)
    }
}

impl MultiProviderIdentityPort for InMemoryMultiProviderIdentityService {
    fn commands(&self) -> Arc<dyn MultiProviderIdentityCommandPort> {
        Arc::new(self.clone())
    }
    fn queries(&self) -> Arc<dyn MultiProviderIdentityQueryPort> {
        Arc::new(self.clone())
    }
}

// =====================================================================
// Invariant check helpers(对齐 lib.rs § check_invariant_xx)
// =====================================================================

/// INV-MPI-01:tenant_id 必填(对齐 INV-SCM-04)
pub fn check_invariant_01_tenant(
    i: &ProviderIdentity,
) -> Result<(), MultiProviderIdentityError> {
    if i.tenant_id.0.is_nil() {
        return Err(MultiProviderIdentityError::TenantMismatch(
            "tenant_id must not be nil".to_string(),
        ));
    }
    Ok(())
}

/// INV-MPI-02:MVP provider 检查
pub fn check_invariant_02_mvp_provider(
    p: IdentityProvider,
) -> Result<(), MultiProviderIdentityError> {
    InMemoryMultiProviderIdentityService::check_mvp_provider(p)
}

/// INV-MPI-03:identity 字段非空校验
pub fn check_invariant_03_required_fields(
    i: &ProviderIdentity,
) -> Result<(), MultiProviderIdentityError> {
    if i.external_id.trim().is_empty() {
        return Err(MultiProviderIdentityError::ValidationFailed(
            "external_id must not be empty".to_string(),
        ));
    }
    if i.external_login.trim().is_empty() {
        return Err(MultiProviderIdentityError::ValidationFailed(
            "external_login must not be empty".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// 单元测试(模块内 #[cfg(test)])
// =====================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;
    use uuid::Uuid;

    fn make_actor(tenant_id: Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id).with_role("project_admin")
    }

    /// **UT-VMPI-1**:register_provider_identity 正常路径
    #[tokio::test]
    async fn ut_v1_register_provider_identity_happy() {
        let svc = InMemoryMultiProviderIdentityService::new_for_test();
        let tenant = TenantId(Uuid::new_v4());
        let actor = make_actor(tenant.0);

        let cmd = RegisterProviderIdentityCommand {
            tenant_id: tenant,
            local_user_id: None,
            provider: IdentityProvider::Github,
            external_id: "1234567".to_string(),
            external_login: "octocat".to_string(),
            display_name: "Octocat".to_string(),
            email: Some("octo@example.com".to_string()),
            url: "https://github.com/octocat".to_string(),
        };
        let id = svc
            .register_provider_identity(cmd, actor)
            .await
            .unwrap();
        assert_eq!(id.provider, IdentityProvider::Github);
        assert_eq!(id.external_id, "1234567");
        assert_eq!(id.lock_version, 1);
        assert!(id.last_validated_at.is_none());
    }

    /// **UT-VMPI-2**:INV-MPI-03 — 同 `(tenant, provider, external_id)` 重复
    #[tokio::test]
    async fn ut_v2_register_duplicate_rejected() {
        let svc = InMemoryMultiProviderIdentityService::new_for_test();
        let tenant = TenantId(Uuid::new_v4());
        let actor = make_actor(tenant.0);

        let mk = || RegisterProviderIdentityCommand {
            tenant_id: tenant,
            local_user_id: None,
            provider: IdentityProvider::Github,
            external_id: "duplicate-id".to_string(),
            external_login: "u".to_string(),
            display_name: "U".to_string(),
            email: None,
            url: "https://example.com/u".to_string(),
        };
        svc.register_provider_identity(mk(), actor.clone()).await.unwrap();
        let err = svc.register_provider_identity(mk(), actor).await.unwrap_err();
        assert!(matches!(err, MultiProviderIdentityError::Duplicate { .. }));
        assert_eq!(err.code(), "MPI_DUPLICATE");
    }

    /// **UT-VMPI-3**:INV-MPI-02 — Linear provider 在 MVP 拒绝
    #[tokio::test]
    async fn ut_v3_register_linear_rejected() {
        let svc = InMemoryMultiProviderIdentityService::new_for_test();
        let tenant = TenantId(Uuid::new_v4());
        let actor = make_actor(tenant.0);

        let cmd = RegisterProviderIdentityCommand {
            tenant_id: tenant,
            local_user_id: None,
            provider: IdentityProvider::Linear,
            external_id: "lin-1".to_string(),
            external_login: "u".to_string(),
            display_name: "U".to_string(),
            email: None,
            url: "https://linear.app/u".to_string(),
        };
        let err = svc.register_provider_identity(cmd, actor).await.unwrap_err();
        assert!(matches!(
            err,
            MultiProviderIdentityError::UnsupportedProvider(IdentityProvider::Linear)
        ));
    }

    /// **UT-VMPI-4**:IdentityValidation 状态机
    #[test]
    fn ut_v4_validation_state_machine() {
        let mk = |s: ValidationStatus| IdentityValidation {
            id: IdentityValidationId::new(),
            tenant_id: TenantId(Uuid::new_v4()),
            link_id: IdentityLinkId::new(),
            identity_ids: vec![],
            status: s,
            mismatch_fields: vec![],
            validated_at: Utc::now(),
            lock_version: 0,
        };
        // Pending → Validated | Mismatch | Revoked
        assert!(mk(ValidationStatus::Pending).can_transition_to(ValidationStatus::Validated));
        assert!(mk(ValidationStatus::Pending).can_transition_to(ValidationStatus::Mismatch));
        assert!(mk(ValidationStatus::Pending).can_transition_to(ValidationStatus::Revoked));
        // Validated → Mismatch | Revoked
        assert!(mk(ValidationStatus::Validated).can_transition_to(ValidationStatus::Mismatch));
        assert!(mk(ValidationStatus::Validated).can_transition_to(ValidationStatus::Revoked));
        // Mismatch → Validated | Revoked
        assert!(mk(ValidationStatus::Mismatch).can_transition_to(ValidationStatus::Validated));
        assert!(mk(ValidationStatus::Mismatch).can_transition_to(ValidationStatus::Revoked));
        // Revoked 终态
        assert!(!mk(ValidationStatus::Revoked).can_transition_to(ValidationStatus::Validated));
        assert!(!mk(ValidationStatus::Revoked).can_transition_to(ValidationStatus::Mismatch));
        assert!(!mk(ValidationStatus::Revoked).can_transition_to(ValidationStatus::Pending));
    }

    /// **UT-VMPI-5**:link_identities 跨 tenant 拒绝(INV-MPI-01)
    #[tokio::test]
    async fn ut_v5_link_cross_tenant_rejected() {
        let svc = InMemoryMultiProviderIdentityService::new_for_test();
        let tenant_a = TenantId(Uuid::new_v4());
        let tenant_b = TenantId(Uuid::new_v4());
        let actor_a = make_actor(tenant_a.0);

        // 在 tenant_a 注册 identity
        let cmd = RegisterProviderIdentityCommand {
            tenant_id: tenant_a,
            local_user_id: None,
            provider: IdentityProvider::Github,
            external_id: "x".to_string(),
            external_login: "u".to_string(),
            display_name: "U".to_string(),
            email: None,
            url: "https://example.com/u".to_string(),
        };
        let identity = svc.register_provider_identity(cmd, actor_a).await.unwrap();

        // 在 tenant_b 试图 link → 应拒绝
        let actor_b = make_actor(tenant_b.0);
        let link_cmd = LinkIdentitiesCommand {
            tenant_id: tenant_b,
            local_user_id: UserId(Uuid::new_v4()),
            identity_ids: vec![identity.id],
        };
        let err = svc
            .link_identities(link_cmd, actor_b)
            .await
            .unwrap_err();
        assert!(matches!(err, MultiProviderIdentityError::TenantMismatch(_)));
    }
}

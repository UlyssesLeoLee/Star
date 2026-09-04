//! domain-tenant crate
//!
//! 详细 spec: docs/specs/domain-tenant-spec.md
//! 上游基本设计: docs/basic-design.md §2.1(表 18) / §6.1
//! 数据设计: docs/data-design.md §4.1 (`tenant` schema)
//! API 设计: docs/api-design.md §3.2
//!
//! ## 职责
//!
//! 最高安全边界(§16,REQ-SEC-001):Tenant / TenantPolicy / SecurityPolicy / ProviderDataBoundary
//! 4 类聚合根 + tenant_id 颁发与跨租户隔离第一道闸门
//!
//! ## 关键不变量
//!
//! - INV-T-01:tenant_id 必带,跨 tenant 拒绝
//! - INV-T-02:Tenant 3 状态(Active / Suspended / Deleted)
//! - INV-T-03:ProviderDataBoundary credential_ref 仅引用 Broker,不存明文(§5.4 Security)
//! - INV-T-04:slug 全局唯一(URL 友好)
//!
//! Lead 责任: tenant Lead

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

define_uuid_id!(TenantId);
define_uuid_id!(TenantPolicyId);
define_uuid_id!(SecurityPolicyId);
define_uuid_id!(ProviderDataBoundaryId);
define_uuid_id!(UserId);
define_uuid_id!(CredentialRefId);

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
/// 生成租户领域 UUID 强类型 ID 的宏,自动实现 Copy/Eq/Hash/Ord/Serialize/Display 等 trait
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 由 `define_uuid_id!` 宏生成的 UUID 强类型 ID 类型
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成一个新的随机 UUID 强类型 ID
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回内部原始 UUID 值
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

/// Tenant(§4.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// 租户唯一标识
    pub id: TenantId,
    /// 租户 URL 友好唯一 slug(INV-T-04)
    pub slug: String,
    /// 租户显示名称
    pub display_name: String,
    /// 租户当前状态(Active / Suspended / Deleted)
    pub status: TenantStatus,
    /// 租户订阅套餐等级
    pub plan_tier: PlanTier,
    /// 租户创建时间
    pub created_at: DateTime<Utc>,
    /// 试用期结束时间(如适用)
    pub trial_ends_at: Option<DateTime<Utc>>,
    /// 租户被暂停的时间(如适用)
    pub suspended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 租户状态(INV-T-02:Active / Suspended / Deleted 三态)
pub enum TenantStatus {
    /// 正常可用状态
    Active,
    /// 已暂停状态
    Suspended,
    /// 已删除状态(终态)
    Deleted,
}

impl TenantStatus {
    /// 返回状态对应的大写字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Deleted => "DELETED",
        }
    }
    /// 判断是否为终态(Deleted)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Deleted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 租户订阅套餐等级
pub enum PlanTier {
    /// 免费套餐
    Free,
    /// 专业套餐
    Pro,
    /// 企业套餐
    Enterprise,
}

impl PlanTier {
    /// 返回套餐等级对应的大写字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "FREE",
            Self::Pro => "PRO",
            Self::Enterprise => "ENTERPRISE",
        }
    }
}

/// TenantPolicy(§4.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPolicy {
    /// 策略唯一标识
    pub id: TenantPolicyId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 是否允许使用云端 AI
    pub cloud_ai_allowed: bool,
    /// 是否限制云端 AI 使用范围
    pub cloud_ai_restricted: bool,
    /// 是否仅允许使用本地 AI
    pub local_ai_only: bool,
    /// 是否禁止上传代码内容
    pub no_code_upload: bool,
    /// 是否仅允许发送元数据
    pub metadata_only: bool,
    /// 允许使用的特定 Provider ID 白名单
    pub specific_provider_allowed: Vec<Uuid>,
    /// 允许使用的区域列表
    pub allowed_regions: Vec<String>,
    /// 数据驻留区域
    pub data_residency_zone: String,
    /// 策略最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl TenantPolicy {
    /// 为指定租户生成默认 TenantPolicy
    pub fn default_for(tenant_id: TenantId) -> Self {
        Self {
            id: TenantPolicyId::new(),
            tenant_id,
            cloud_ai_allowed: true,
            cloud_ai_restricted: false,
            local_ai_only: false,
            no_code_upload: false,
            metadata_only: false,
            specific_provider_allowed: vec![],
            allowed_regions: vec!["*".to_string()],
            data_residency_zone: "global".to_string(),
            updated_at: Utc::now(),
        }
    }
}

/// SecurityPolicy(§4.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// 策略唯一标识
    pub id: SecurityPolicyId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 是否强制要求多因素认证
    pub require_mfa: bool,
    /// MFA 宽限期时长(秒)
    pub mfa_grace_period_seconds: u32,
    /// 会话最大有效期(秒)
    pub session_max_age_seconds: u32,
    /// 刷新令牌有效期(秒)
    pub refresh_token_ttl_seconds: u32,
    /// 每用户允许的最大设备数
    pub device_max_per_user: u32,
    /// 设备记录有效期(秒)
    pub device_ttl_seconds: u32,
    /// 策略最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl SecurityPolicy {
    /// 为指定租户生成默认 SecurityPolicy
    pub fn default_for(tenant_id: TenantId) -> Self {
        Self {
            id: SecurityPolicyId::new(),
            tenant_id,
            require_mfa: false,
            mfa_grace_period_seconds: 86400,
            session_max_age_seconds: 3600 * 8,
            refresh_token_ttl_seconds: 86400 * 30,
            device_max_per_user: 5,
            device_ttl_seconds: 86400 * 90,
            updated_at: Utc::now(),
        }
    }
}

/// ProviderDataBoundary(§4.1,§5.4 Security)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDataBoundary {
    /// 边界记录唯一标识
    pub id: ProviderDataBoundaryId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// AI Provider ID
    pub provider_id: String,
    /// 使用的模型 ID
    pub model_id: String,
    /// 数据处理所在区域
    pub region: String,
    /// 发送给 Provider 的数据种类
    pub data_sent: Vec<DataKind>,
    /// 数据保留策略
    pub retention_policy: RetentionPolicy,
    /// 凭据引用(仅引用 Broker,不存明文,INV-T-03)
    pub credential_ref: CredentialRefId,
    /// 边界记录创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 发送给 AI Provider 的数据种类
pub enum DataKind {
    /// 提示词
    Prompt,
    /// 源代码
    Code,
    /// 代码差异
    Diff,
    /// 符号信息
    Symbol,
    /// 测试内容
    Test,
    /// 构建日志
    BuildLog,
}

impl DataKind {
    /// 返回数据种类对应的大写字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prompt => "PROMPT",
            Self::Code => "CODE",
            Self::Diff => "DIFF",
            Self::Symbol => "SYMBOL",
            Self::Test => "TEST",
            Self::BuildLog => "BUILD_LOG",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 数据保留策略
pub enum RetentionPolicy {
    /// 不保留(零保留)
    Zero,
    /// 保留指定天数
    NDays(u32),
    /// 保留至任务结束
    UntilTaskEnd,
}

impl RetentionPolicy {
    /// 返回保留策略对应的大写字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zero => "ZERO",
            Self::NDays(_) => "N_DAYS",
            Self::UntilTaskEnd => "UNTIL_TASK_END",
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
/// 租户域错误类型
pub enum TenantError {
    #[error("not found: {0}")]
    /// 资源未找到
    NotFound(String),
    #[error("permission denied")]
    /// 权限不足
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    /// 跨租户访问被拒绝
    CrossTenantDenied(TenantId, TenantId),
    #[error("slug already exists: {0}")]
    /// slug 已存在(违反 INV-T-04)
    SlugExists(String),
    #[error("invalid state: {0}")]
    /// 状态非法,无法执行该操作
    InvalidState(String),
    #[error("conflict: {0}")]
    /// 冲突
    Conflict(String),
    #[error("internal: {0}")]
    /// 内部错误
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建租户命令
pub struct CreateTenantCommand {
    /// 待创建租户的 slug
    pub slug: String,
    /// 待创建租户的显示名称
    pub display_name: String,
    /// 待创建租户的套餐等级
    pub plan_tier: PlanTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 暂停租户命令
pub struct SuspendTenantCommand {
    /// 待暂停的租户 ID
    pub tenant_id: TenantId,
    /// 暂停原因
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 更新租户策略命令
pub struct UpdateTenantPolicyCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 新的租户策略内容
    pub policy: TenantPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 更新安全策略命令
pub struct UpdateSecurityPolicyCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 新的安全策略内容
    pub policy: SecurityPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 注册 Provider 数据边界命令
pub struct RegisterProviderBoundaryCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// AI Provider ID
    pub provider_id: String,
    /// 使用的模型 ID
    pub model_id: String,
    /// 数据处理所在区域
    pub region: String,
    /// 发送给 Provider 的数据种类
    pub data_sent: Vec<DataKind>,
    /// 数据保留策略
    pub retention_policy: RetentionPolicy,
    /// 凭据引用
    pub credential_ref: CredentialRefId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 查询租户请求
pub struct GetTenantQuery {
    /// 待查询的租户 ID
    pub tenant_id: TenantId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
/// 租户命令端口(写操作)
pub trait TenantCommandPort: Send + Sync {
    /// 创建新租户
    async fn create_tenant(
        &self,
        cmd: CreateTenantCommand,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 暂停租户
    async fn suspend_tenant(
        &self,
        cmd: SuspendTenantCommand,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 重新激活已暂停租户
    async fn reactivate_tenant(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 更新租户策略
    async fn update_tenant_policy(
        &self,
        cmd: UpdateTenantPolicyCommand,
        actor: &ActorContext,
    ) -> Result<TenantPolicy, TenantError>;

    /// 更新安全策略
    async fn update_security_policy(
        &self,
        cmd: UpdateSecurityPolicyCommand,
        actor: &ActorContext,
    ) -> Result<SecurityPolicy, TenantError>;

    /// 注册 Provider 数据边界
    async fn register_provider_boundary(
        &self,
        cmd: RegisterProviderBoundaryCommand,
        actor: &ActorContext,
    ) -> Result<ProviderDataBoundary, TenantError>;
}

#[async_trait]
/// 租户查询端口(读操作)
pub trait TenantQueryPort: Send + Sync {
    /// 获取租户信息
    async fn get_tenant(
        &self,
        q: GetTenantQuery,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 获取租户策略
    async fn get_tenant_policy(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<TenantPolicy, TenantError>;

    /// 获取安全策略
    async fn get_security_policy(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<SecurityPolicy, TenantError>;

    /// 列出租户的 Provider 数据边界
    async fn list_provider_boundaries(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Vec<ProviderDataBoundary>, TenantError>;
}

#[async_trait]
/// 租户仓储端口
pub trait TenantRepository: Send + Sync {
    /// 插入租户记录
    async fn insert_tenant(&self, t: Tenant) -> Result<(), TenantError>;
    /// 按 ID 获取租户
    async fn get_tenant(&self, id: TenantId) -> Result<Tenant, TenantError>;
    /// 按 slug 获取租户
    async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>, TenantError>;
    /// 更新租户记录
    async fn update_tenant(&self, t: Tenant) -> Result<(), TenantError>;

    /// 插入或更新租户策略
    async fn upsert_tenant_policy(&self, p: TenantPolicy) -> Result<(), TenantError>;
    /// 按租户 ID 获取租户策略
    async fn get_tenant_policy(&self, tid: TenantId) -> Result<TenantPolicy, TenantError>;

    /// 插入或更新安全策略
    async fn upsert_security_policy(&self, p: SecurityPolicy) -> Result<(), TenantError>;
    /// 按租户 ID 获取安全策略
    async fn get_security_policy(&self, tid: TenantId) -> Result<SecurityPolicy, TenantError>;

    /// 插入 Provider 数据边界记录
    async fn insert_provider_boundary(&self, b: ProviderDataBoundary) -> Result<(), TenantError>;
    /// 列出租户的 Provider 数据边界记录
    async fn list_provider_boundaries(
        &self,
        tid: TenantId,
    ) -> Result<Vec<ProviderDataBoundary>, TenantError>;
}

// =====================================================================
// InMemoryTenantService
// =====================================================================

/// 基于内存的租户服务实现(用于测试/开发)
pub struct InMemoryTenantService {
    repo: Arc<dyn TenantRepository>,
    tenants: Arc<RwLock<HashMap<TenantId, Tenant>>>,
    policies: Arc<RwLock<HashMap<TenantId, TenantPolicy>>>,
    sec_policies: Arc<RwLock<HashMap<TenantId, SecurityPolicy>>>,
    boundaries: Arc<RwLock<HashMap<ProviderDataBoundaryId, ProviderDataBoundary>>>,
}

impl InMemoryTenantService {
    /// 创建一个新的内存租户服务实例
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryTenantRepository::new()),
            tenants: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            sec_policies: Arc::new(RwLock::new(HashMap::new())),
            boundaries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryTenantService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantCommandPort for InMemoryTenantService {
    async fn create_tenant(
        &self,
        cmd: CreateTenantCommand,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError> {
        if !actor.is_platform_admin {
            return Err(TenantError::PermissionDenied);
        }
        // INV-T-04:slug 全局唯一
        if let Some(_) = self.repo.get_tenant_by_slug(&cmd.slug).await? {
            return Err(TenantError::SlugExists(cmd.slug));
        }
        let t = Tenant {
            id: TenantId::new(),
            slug: cmd.slug,
            display_name: cmd.display_name,
            status: TenantStatus::Active,
            plan_tier: cmd.plan_tier,
            created_at: Utc::now(),
            trial_ends_at: None,
            suspended_at: None,
        };
        self.repo.insert_tenant(t.clone()).await?;
        // 默认 policy
        let p = TenantPolicy::default_for(t.id);
        let s = SecurityPolicy::default_for(t.id);
        self.repo.upsert_tenant_policy(p.clone()).await?;
        self.repo.upsert_security_policy(s.clone()).await?;
        self.tenants.write().unwrap().insert(t.id, t.clone());
        self.policies.write().unwrap().insert(t.id, p);
        self.sec_policies.write().unwrap().insert(t.id, s);
        Ok(t)
    }

    async fn suspend_tenant(
        &self,
        cmd: SuspendTenantCommand,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError> {
        if !actor.is_platform_admin {
            return Err(TenantError::PermissionDenied);
        }
        let mut t = self
            .tenants
            .write()
            .unwrap()
            .get_mut(&cmd.tenant_id)
            .cloned()
            .ok_or(TenantError::NotFound(format!(
                "tenant:{}",
                cmd.tenant_id.as_uuid()
            )))?;
        if t.status == TenantStatus::Deleted {
            return Err(TenantError::InvalidState(
                "cannot suspend deleted tenant".to_string(),
            ));
        }
        t.status = TenantStatus::Suspended;
        t.suspended_at = Some(Utc::now());
        self.repo.update_tenant(t.clone()).await?;
        self.tenants.write().unwrap().insert(t.id, t.clone());
        Ok(t)
    }

    async fn reactivate_tenant(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError> {
        if !actor.is_platform_admin {
            return Err(TenantError::PermissionDenied);
        }
        let mut t = self
            .tenants
            .write()
            .unwrap()
            .get_mut(&tenant_id)
            .cloned()
            .ok_or(TenantError::NotFound(format!(
                "tenant:{}",
                tenant_id.as_uuid()
            )))?;
        if t.status != TenantStatus::Suspended {
            return Err(TenantError::InvalidState("not suspended".to_string()));
        }
        t.status = TenantStatus::Active;
        t.suspended_at = None;
        self.repo.update_tenant(t.clone()).await?;
        self.tenants.write().unwrap().insert(t.id, t.clone());
        Ok(t)
    }

    async fn update_tenant_policy(
        &self,
        cmd: UpdateTenantPolicyCommand,
        actor: &ActorContext,
    ) -> Result<TenantPolicy, TenantError> {
        if !actor.is_platform_admin && !actor.has_role("tenant_admin") {
            return Err(TenantError::PermissionDenied);
        }
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut p = cmd.policy;
        p.tenant_id = cmd.tenant_id;
        p.updated_at = Utc::now();
        self.repo.upsert_tenant_policy(p.clone()).await?;
        self.policies
            .write()
            .unwrap()
            .insert(cmd.tenant_id, p.clone());
        Ok(p)
    }

    async fn update_security_policy(
        &self,
        cmd: UpdateSecurityPolicyCommand,
        actor: &ActorContext,
    ) -> Result<SecurityPolicy, TenantError> {
        if !actor.is_platform_admin && !actor.has_role("tenant_admin") {
            return Err(TenantError::PermissionDenied);
        }
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut p = cmd.policy;
        p.tenant_id = cmd.tenant_id;
        p.updated_at = Utc::now();
        self.repo.upsert_security_policy(p.clone()).await?;
        self.sec_policies
            .write()
            .unwrap()
            .insert(cmd.tenant_id, p.clone());
        Ok(p)
    }

    async fn register_provider_boundary(
        &self,
        cmd: RegisterProviderBoundaryCommand,
        actor: &ActorContext,
    ) -> Result<ProviderDataBoundary, TenantError> {
        if !actor.has_role("tenant_admin") && !actor.is_platform_admin {
            return Err(TenantError::PermissionDenied);
        }
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let b = ProviderDataBoundary {
            id: ProviderDataBoundaryId::new(),
            tenant_id: cmd.tenant_id,
            provider_id: cmd.provider_id,
            model_id: cmd.model_id,
            region: cmd.region,
            data_sent: cmd.data_sent,
            retention_policy: cmd.retention_policy,
            credential_ref: cmd.credential_ref,
            created_at: Utc::now(),
        };
        self.repo.insert_provider_boundary(b.clone()).await?;
        self.boundaries.write().unwrap().insert(b.id, b.clone());
        Ok(b)
    }
}

#[async_trait]
impl TenantQueryPort for InMemoryTenantService {
    async fn get_tenant(
        &self,
        q: GetTenantQuery,
        actor: &ActorContext,
    ) -> Result<Tenant, TenantError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        self.tenants
            .read()
            .unwrap()
            .get(&q.tenant_id)
            .cloned()
            .ok_or(TenantError::NotFound(format!(
                "tenant:{}",
                q.tenant_id.as_uuid()
            )))
    }

    async fn get_tenant_policy(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<TenantPolicy, TenantError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        self.policies
            .read()
            .unwrap()
            .get(&tenant_id)
            .cloned()
            .ok_or(TenantError::NotFound(format!(
                "policy:{}",
                tenant_id.as_uuid()
            )))
    }

    async fn get_security_policy(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<SecurityPolicy, TenantError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        self.sec_policies
            .read()
            .unwrap()
            .get(&tenant_id)
            .cloned()
            .ok_or(TenantError::NotFound(format!(
                "sec:{}",
                tenant_id.as_uuid()
            )))
    }

    async fn list_provider_boundaries(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Vec<ProviderDataBoundary>, TenantError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != tenant_id {
            return Err(TenantError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        Ok(self
            .boundaries
            .read()
            .unwrap()
            .values()
            .filter(|b| b.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
}

// =====================================================================
// InMemoryTenantRepository
// =====================================================================

/// 基于内存的租户仓储实现(用于测试/开发)
pub struct InMemoryTenantRepository {
    tenants: RwLock<HashMap<TenantId, Tenant>>,
    by_slug: RwLock<HashMap<String, TenantId>>,
    policies: RwLock<HashMap<TenantId, TenantPolicy>>,
    sec: RwLock<HashMap<TenantId, SecurityPolicy>>,
    boundaries: RwLock<HashMap<ProviderDataBoundaryId, ProviderDataBoundary>>,
}

impl InMemoryTenantRepository {
    /// 创建一个新的内存租户仓储实例
    pub fn new() -> Self {
        Self {
            tenants: RwLock::new(HashMap::new()),
            by_slug: RwLock::new(HashMap::new()),
            policies: RwLock::new(HashMap::new()),
            sec: RwLock::new(HashMap::new()),
            boundaries: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryTenantRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantRepository for InMemoryTenantRepository {
    async fn insert_tenant(&self, t: Tenant) -> Result<(), TenantError> {
        self.tenants.write().unwrap().insert(t.id, t.clone());
        self.by_slug.write().unwrap().insert(t.slug.clone(), t.id);
        Ok(())
    }
    async fn get_tenant(&self, id: TenantId) -> Result<Tenant, TenantError> {
        self.tenants
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(TenantError::NotFound(format!("tenant:{}", id.as_uuid())))
    }
    async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>, TenantError> {
        let id = self.by_slug.read().unwrap().get(slug).cloned();
        match id {
            Some(i) => Ok(self.tenants.read().unwrap().get(&i).cloned()),
            None => Ok(None),
        }
    }
    async fn update_tenant(&self, t: Tenant) -> Result<(), TenantError> {
        self.tenants.write().unwrap().insert(t.id, t);
        Ok(())
    }
    async fn upsert_tenant_policy(&self, p: TenantPolicy) -> Result<(), TenantError> {
        self.policies.write().unwrap().insert(p.tenant_id, p);
        Ok(())
    }
    async fn get_tenant_policy(&self, tid: TenantId) -> Result<TenantPolicy, TenantError> {
        self.policies
            .read()
            .unwrap()
            .get(&tid)
            .cloned()
            .ok_or(TenantError::NotFound(format!("policy:{}", tid.as_uuid())))
    }
    async fn upsert_security_policy(&self, p: SecurityPolicy) -> Result<(), TenantError> {
        self.sec.write().unwrap().insert(p.tenant_id, p);
        Ok(())
    }
    async fn get_security_policy(&self, tid: TenantId) -> Result<SecurityPolicy, TenantError> {
        self.sec
            .read()
            .unwrap()
            .get(&tid)
            .cloned()
            .ok_or(TenantError::NotFound(format!("sec:{}", tid.as_uuid())))
    }
    async fn insert_provider_boundary(&self, b: ProviderDataBoundary) -> Result<(), TenantError> {
        self.boundaries.write().unwrap().insert(b.id, b);
        Ok(())
    }
    async fn list_provider_boundaries(
        &self,
        tid: TenantId,
    ) -> Result<Vec<ProviderDataBoundary>, TenantError> {
        Ok(self
            .boundaries
            .read()
            .unwrap()
            .values()
            .filter(|b| b.tenant_id == tid)
            .cloned()
            .collect())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn platform_admin() -> ActorContext {
        let mut a = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
        a.is_platform_admin = true;
        a
    }

    fn tenant_admin(tid: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid.0).with_role("tenant_admin")
    }

    #[test]
    fn tenant_status_as_str() {
        assert_eq!(TenantStatus::Active.as_str(), "ACTIVE");
        assert_eq!(TenantStatus::Suspended.as_str(), "SUSPENDED");
        assert_eq!(TenantStatus::Deleted.as_str(), "DELETED");
        assert!(TenantStatus::Deleted.is_terminal());
    }

    #[test]
    fn plan_tier_as_str() {
        assert_eq!(PlanTier::Free.as_str(), "FREE");
        assert_eq!(PlanTier::Pro.as_str(), "PRO");
    }

    #[test]
    fn data_kind_as_str() {
        assert_eq!(DataKind::Prompt.as_str(), "PROMPT");
        assert_eq!(DataKind::Code.as_str(), "CODE");
    }

    #[test]
    fn retention_as_str() {
        assert_eq!(RetentionPolicy::Zero.as_str(), "ZERO");
        assert_eq!(RetentionPolicy::NDays(30).as_str(), "N_DAYS");
        assert_eq!(RetentionPolicy::UntilTaskEnd.as_str(), "UNTIL_TASK_END");
    }

    #[test]
    fn tenant_policy_default() {
        let p = TenantPolicy::default_for(TenantId(uuid::Uuid::new_v4()));
        assert!(p.cloud_ai_allowed);
        assert!(!p.local_ai_only);
    }

    #[test]
    fn security_policy_default() {
        let p = SecurityPolicy::default_for(TenantId(uuid::Uuid::new_v4()));
        assert_eq!(p.session_max_age_seconds, 3600 * 8);
    }

    #[tokio::test]
    async fn create_tenant_requires_platform_admin() {
        let svc = InMemoryTenantService::new();
        let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
        let res = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "acme".to_string(),
                    display_name: "Acme".to_string(),
                    plan_tier: PlanTier::Pro,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(TenantError::PermissionDenied)));
    }

    #[tokio::test]
    async fn create_tenant_creates_default_policies() {
        let svc = InMemoryTenantService::new();
        let actor = platform_admin();
        let t = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "acme".to_string(),
                    display_name: "Acme".to_string(),
                    plan_tier: PlanTier::Pro,
                },
                &actor,
            )
            .await
            .unwrap();
        let p = svc.get_tenant_policy(t.id, &actor).await.unwrap();
        let s = svc.get_security_policy(t.id, &actor).await.unwrap();
        assert_eq!(p.tenant_id, t.id);
        assert_eq!(s.tenant_id, t.id);
    }

    #[tokio::test]
    async fn slug_must_be_unique() {
        let svc = InMemoryTenantService::new();
        let actor = platform_admin();
        svc.create_tenant(
            CreateTenantCommand {
                slug: "acme".to_string(),
                display_name: "Acme".to_string(),
                plan_tier: PlanTier::Free,
            },
            &actor,
        )
        .await
        .unwrap();
        let res = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "acme".to_string(),
                    display_name: "Acme2".to_string(),
                    plan_tier: PlanTier::Free,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(TenantError::SlugExists(_))));
    }

    #[tokio::test]
    async fn suspend_and_reactivate() {
        let svc = InMemoryTenantService::new();
        let actor = platform_admin();
        let t = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "x".to_string(),
                    display_name: "X".to_string(),
                    plan_tier: PlanTier::Free,
                },
                &actor,
            )
            .await
            .unwrap();
        let t2 = svc
            .suspend_tenant(
                SuspendTenantCommand {
                    tenant_id: t.id,
                    reason: "trial ended".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(t2.status, TenantStatus::Suspended);
        let t3 = svc.reactivate_tenant(t.id, &actor).await.unwrap();
        assert_eq!(t3.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn cross_tenant_get_denied() {
        let svc = InMemoryTenantService::new();
        let platform = platform_admin();
        let t = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "a".to_string(),
                    display_name: "A".to_string(),
                    plan_tier: PlanTier::Free,
                },
                &platform,
            )
            .await
            .unwrap();
        let other_t = uuid::Uuid::new_v4();
        let user_actor = ActorContext::new(Uuid::new_v4(), other_t);
        let res = svc
            .get_tenant(GetTenantQuery { tenant_id: t.id }, &user_actor)
            .await;
        assert!(matches!(res, Err(TenantError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn update_tenant_policy_self_only() {
        let svc = InMemoryTenantService::new();
        let platform = platform_admin();
        let t = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "a".to_string(),
                    display_name: "A".to_string(),
                    plan_tier: PlanTier::Pro,
                },
                &platform,
            )
            .await
            .unwrap();
        let admin = tenant_admin(t.id);
        let mut p = TenantPolicy::default_for(t.id);
        p.local_ai_only = true;
        let p = svc
            .update_tenant_policy(
                UpdateTenantPolicyCommand {
                    tenant_id: t.id,
                    policy: p,
                },
                &admin,
            )
            .await
            .unwrap();
        assert!(p.local_ai_only);
    }

    #[tokio::test]
    async fn register_provider_boundary() {
        let svc = InMemoryTenantService::new();
        let platform = platform_admin();
        let t = svc
            .create_tenant(
                CreateTenantCommand {
                    slug: "a".to_string(),
                    display_name: "A".to_string(),
                    plan_tier: PlanTier::Pro,
                },
                &platform,
            )
            .await
            .unwrap();
        let admin = tenant_admin(t.id);
        let b = svc
            .register_provider_boundary(
                RegisterProviderBoundaryCommand {
                    tenant_id: t.id,
                    provider_id: "openai".to_string(),
                    model_id: "gpt-4".to_string(),
                    region: "us-east-1".to_string(),
                    data_sent: vec![DataKind::Prompt, DataKind::Code],
                    retention_policy: RetentionPolicy::NDays(30),
                    credential_ref: CredentialRefId::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(b.tenant_id, t.id);
        assert_eq!(b.data_sent.len(), 2);
        let list = svc.list_provider_boundaries(t.id, &admin).await.unwrap();
        assert_eq!(list.len(), 1);
    }
}

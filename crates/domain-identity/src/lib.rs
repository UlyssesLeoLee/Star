//! domain-identity crate
//!
//! 详细 spec: docs/specs/domain-identity-spec.md
//! 上游基本设计: docs/basic-design.md §2.1(表 22) / §23.2
//! 数据设计: docs/data-design.md §4.14 (`identity` schema)
//! API 设计: docs/api-design.md §3.15
//!
//! ## 职责
//!
//! 用户 / 设备身份(§23)。User / Device / Credential / DeviceBinding
//! (tenant + user + project 三重,LRT-001/002)
//!
//! ## 关键不变量
//!
//! - INV-ID-01:User 必带 tenant_id
//! - INV-ID-02:Device 三重绑定 tenant+user+project(LRT-001/002)
//! - INV-ID-03:Credential 仅存 hash / ref,绝不存明文(§5.4 Security)
//! - INV-ID-04:Device 状态机 Active / Revoked / Pending
//!
//! Lead 责任: identity Lead

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

define_uuid_id!(UserId);
define_uuid_id!(DeviceId);
define_uuid_id!(CredentialId);
define_uuid_id!(DeviceBindingId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(CredentialRefId);

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
/// 生成 UUID 强类型 ID 及其 `new`/`as_uuid`/`From<Uuid>`/`Display` 实现的宏
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 领域强类型 ID(由宏统一生成,内部包装一个 Uuid)
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成一个新的随机 ID(由宏统一生成)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回内部的原始 Uuid 值(由宏统一生成)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 平台用户实体,归属单一租户(INV-ID-01)
pub struct User {
    /// 用户主键
    pub id: UserId,
    /// 所属租户 ID(INV-ID-01 必带)
    pub tenant_id: TenantId,
    /// 登录邮箱,租户内唯一
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 用户状态
    pub status: UserStatus,
    /// 关联的凭证引用
    pub credential_ref: CredentialRefId,
    /// 在租户内的角色
    pub tenant_role: TenantRole,
    /// 是否已启用多因素认证
    pub mfa_enabled: bool,
    /// MFA 密钥引用(启用 MFA 时存在)
    pub mfa_secret_ref: Option<CredentialRefId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近一次登录时间
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 用户账号状态:Active / Suspended / Invited
pub enum UserStatus {
    /// 正常可用
    Active,
    /// 已被停用
    Suspended,
    /// 已邀请但未激活
    Invited,
}

impl UserStatus {
    /// 返回状态的大写字符串表示(如 "ACTIVE")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Invited => "INVITED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 租户内角色
pub enum TenantRole {
    /// 租户管理员
    TenantAdmin,
    /// 项目管理员
    ProjectAdmin,
    /// 开发者
    Developer,
    /// 只读查看者
    Viewer,
}

impl TenantRole {
    /// 返回角色的 snake_case 字符串表示(如 "tenant_admin")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TenantAdmin => "tenant_admin",
            Self::ProjectAdmin => "project_admin",
            Self::Developer => "developer",
            Self::Viewer => "viewer",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 设备实体,三重绑定 tenant+user+project(INV-ID-02)
pub struct Device {
    /// 设备主键
    pub id: DeviceId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属用户 ID
    pub user_id: UserId,
    /// 设备类型
    pub kind: DeviceKind,
    /// 设备证书指纹
    pub device_cert_fingerprint: String,
    /// 设备状态
    pub status: DeviceStatus,
    /// 已绑定的项目 ID 列表
    pub project_ids: Vec<ProjectId>,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 最近一次活跃时间
    pub last_seen_at: Option<DateTime<Utc>>,
    /// 撤销时间(未撤销为 None)
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 设备类型
pub enum DeviceKind {
    /// 本地运行时
    LocalRuntime,
    /// 命令行客户端
    Cli,
    /// Web 客户端
    Web,
    /// 移动客户端
    Mobile,
}

impl DeviceKind {
    /// 返回设备类型的 snake_case 字符串表示(如 "local_runtime")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalRuntime => "local_runtime",
            Self::Cli => "cli",
            Self::Web => "web",
            Self::Mobile => "mobile",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 设备状态机(INV-ID-04)
pub enum DeviceStatus {
    /// 待激活
    Pending,
    /// 正常可用
    Active,
    /// 已撤销(终态)
    Revoked,
}

impl DeviceStatus {
    /// 返回状态的大写字符串表示(如 "ACTIVE")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Active => "ACTIVE",
            Self::Revoked => "REVOKED",
        }
    }
    /// 判断是否为终态(已撤销)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Revoked)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 凭证实体,仅存 hash / ref,绝不存明文(INV-ID-03)
pub struct Credential {
    /// 凭证主键
    pub id: CredentialId,
    /// 所属用户 ID
    pub user_id: UserId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 凭证类型
    pub kind: CredentialKind,
    /// INV-ID-03:仅 hash,绝不存明文
    pub secret_hash: String,
    /// MFA 密钥引用
    pub mfa_secret_ref: Option<CredentialRefId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近一次使用时间
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 凭证类型
pub enum CredentialKind {
    /// 密码
    Password,
    /// API Key
    ApiKey,
    /// OAuth 令牌
    OAuthToken,
}

impl CredentialKind {
    /// 返回凭证类型的 snake_case 字符串表示(如 "api_key")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::ApiKey => "api_key",
            Self::OAuthToken => "oauth_token",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 设备绑定关系(tenant+user+project 三重绑定,LRT-001/002)
pub struct DeviceBinding {
    /// 绑定主键
    pub id: DeviceBindingId,
    /// 关联的设备 ID
    pub device_id: DeviceId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属用户 ID
    pub user_id: UserId,
    /// 绑定的项目 ID
    pub project_id: ProjectId,
    /// 绑定类型
    pub binding_kind: BindingKind,
    /// 绑定时间
    pub bound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 设备与项目的绑定类型
pub enum BindingKind {
    /// 所有者
    Owner,
    /// 贡献者
    Contributor,
    /// 只读
    ReadOnly,
}

impl BindingKind {
    /// 返回绑定类型的 snake_case 字符串表示(如 "read_only")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Contributor => "contributor",
            Self::ReadOnly => "read_only",
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
/// identity 领域错误
pub enum IdentityError {
    #[error("not found: {0}")]
    /// 目标资源不存在
    NotFound(String),
    #[error("permission denied")]
    /// 权限不足
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    /// 跨租户访问被拒绝(当前租户 vs 要求租户)
    CrossTenantDenied(TenantId, TenantId),
    #[error("email already exists: {0}")]
    /// 邮箱已存在
    EmailExists(String),
    #[error("device triple binding incomplete (INV-ID-02): need tenant+user+project")]
    /// 设备三重绑定不完整(INV-ID-02)
    IncompleteBinding,
    #[error("device already revoked")]
    /// 设备已被撤销
    DeviceAlreadyRevoked,
    #[error("conflict: {0}")]
    /// 状态冲突
    Conflict(String),
    #[error("internal: {0}")]
    /// 内部错误
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建用户命令
pub struct CreateUserCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 用户邮箱
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 租户内角色
    pub tenant_role: TenantRole,
    /// 凭证引用
    pub credential_ref: CredentialRefId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 注册设备命令
pub struct RegisterDeviceCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 设备所属用户 ID
    pub user_id: UserId,
    /// 设备类型
    pub kind: DeviceKind,
    /// 设备证书指纹
    pub device_cert_fingerprint: String,
    /// 待绑定的项目 ID 列表
    pub project_ids: Vec<ProjectId>,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 绑定设备到项目命令
pub struct BindDeviceCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 待绑定的设备 ID
    pub device_id: DeviceId,
    /// 待绑定的项目 ID
    pub project_id: ProjectId,
    /// 绑定类型
    pub binding_kind: BindingKind,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 撤销设备命令
pub struct RevokeDeviceCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 待撤销的设备 ID
    pub device_id: DeviceId,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 记录登录命令
pub struct RecordLoginCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 登录的用户 ID
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 查询用户请求
pub struct GetUserQuery {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标用户 ID
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 按用户列出设备请求
pub struct ListDevicesByUserQuery {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标用户 ID
    pub user_id: UserId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
/// identity 领域命令端口
pub trait IdentityCommandPort: Send + Sync {
    /// 创建用户
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor: &ActorContext,
    ) -> Result<User, IdentityError>;

    /// 注册设备
    async fn register_device(
        &self,
        cmd: RegisterDeviceCommand,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError>;

    /// 将设备绑定到项目
    async fn bind_device(
        &self,
        cmd: BindDeviceCommand,
        actor: &ActorContext,
    ) -> Result<DeviceBinding, IdentityError>;

    /// 撤销设备
    async fn revoke_device(
        &self,
        cmd: RevokeDeviceCommand,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError>;

    /// 记录用户登录
    async fn record_login(
        &self,
        cmd: RecordLoginCommand,
        actor: &ActorContext,
    ) -> Result<User, IdentityError>;
}

#[async_trait]
/// identity 领域查询端口
pub trait IdentityQueryPort: Send + Sync {
    /// 查询单个用户
    async fn get_user(&self, q: GetUserQuery, actor: &ActorContext) -> Result<User, IdentityError>;

    /// 按用户列出设备
    async fn list_devices(
        &self,
        q: ListDevicesByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Device>, IdentityError>;

    /// 查询单个设备
    async fn get_device(
        &self,
        tenant_id: TenantId,
        device_id: DeviceId,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError>;
}

#[async_trait]
/// identity 领域持久化仓储端口
pub trait IdentityRepository: Send + Sync {
    /// 插入用户
    async fn insert_user(&self, u: User) -> Result<(), IdentityError>;
    /// 按 ID 查询用户
    async fn get_user(&self, id: UserId) -> Result<User, IdentityError>;
    /// 按邮箱查询用户
    async fn get_user_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>, IdentityError>;
    /// 更新用户
    async fn update_user(&self, u: User) -> Result<(), IdentityError>;

    /// 插入设备
    async fn insert_device(&self, d: Device) -> Result<(), IdentityError>;
    /// 按 ID 查询设备
    async fn get_device(&self, id: DeviceId) -> Result<Device, IdentityError>;
    /// 更新设备
    async fn update_device(&self, d: Device) -> Result<(), IdentityError>;
    /// 按租户与用户列出设备
    async fn list_devices_by_user(
        &self,
        tid: TenantId,
        uid: UserId,
    ) -> Result<Vec<Device>, IdentityError>;

    /// 插入设备绑定
    async fn insert_binding(&self, b: DeviceBinding) -> Result<(), IdentityError>;
    /// 按设备列出绑定关系
    async fn list_bindings_by_device(
        &self,
        did: DeviceId,
    ) -> Result<Vec<DeviceBinding>, IdentityError>;
}

// =====================================================================
// InMemoryIdentityService
// =====================================================================

/// 基于内存的 identity 领域服务实现(测试 / 参考用途)
pub struct InMemoryIdentityService {
    repo: Arc<dyn IdentityRepository>,
    users: Arc<RwLock<HashMap<UserId, User>>>,
    devices: Arc<RwLock<HashMap<DeviceId, Device>>>,
    bindings: Arc<RwLock<HashMap<DeviceBindingId, DeviceBinding>>>,
}

impl InMemoryIdentityService {
    /// 创建一个空的内存 identity 服务
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryIdentityRepository::new()),
            users: Arc::new(RwLock::new(HashMap::new())),
            devices: Arc::new(RwLock::new(HashMap::new())),
            bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryIdentityService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdentityCommandPort for InMemoryIdentityService {
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor: &ActorContext,
    ) -> Result<User, IdentityError> {
        if !actor.is_platform_admin && !actor.has_role("tenant_admin") {
            return Err(IdentityError::PermissionDenied);
        }
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if let Some(_) = self
            .repo
            .get_user_by_email(cmd.tenant_id, &cmd.email)
            .await?
        {
            return Err(IdentityError::EmailExists(cmd.email));
        }
        let now = Utc::now();
        let u = User {
            id: UserId::new(),
            tenant_id: cmd.tenant_id,
            email: cmd.email,
            display_name: cmd.display_name,
            status: UserStatus::Invited,
            credential_ref: cmd.credential_ref,
            tenant_role: cmd.tenant_role,
            mfa_enabled: false,
            mfa_secret_ref: None,
            created_at: now,
            last_login_at: None,
        };
        self.repo.insert_user(u.clone()).await?;
        self.users.write().unwrap().insert(u.id, u.clone());
        Ok(u)
    }

    async fn register_device(
        &self,
        cmd: RegisterDeviceCommand,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if UserId::from(actor.user_id) != cmd.user_id
            && !actor.has_role("tenant_admin")
            && !actor.is_platform_admin
        {
            return Err(IdentityError::PermissionDenied);
        }
        // INV-ID-02:三重绑定必带(至少一个 project_id 即可)
        if cmd.project_ids.is_empty() {
            return Err(IdentityError::IncompleteBinding);
        }
        let now = Utc::now();
        let d = Device {
            id: DeviceId::new(),
            tenant_id: cmd.tenant_id,
            user_id: cmd.user_id,
            kind: cmd.kind,
            device_cert_fingerprint: cmd.device_cert_fingerprint,
            status: DeviceStatus::Active,
            project_ids: cmd.project_ids,
            registered_at: now,
            last_seen_at: None,
            revoked_at: None,
        };
        self.repo.insert_device(d.clone()).await?;
        self.devices.write().unwrap().insert(d.id, d.clone());
        Ok(d)
    }

    async fn bind_device(
        &self,
        cmd: BindDeviceCommand,
        actor: &ActorContext,
    ) -> Result<DeviceBinding, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let device = self.repo.get_device(cmd.device_id).await?;
        if device.tenant_id != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                device.tenant_id,
                cmd.tenant_id,
            ));
        }
        if device.status == DeviceStatus::Revoked {
            return Err(IdentityError::DeviceAlreadyRevoked);
        }
        let b = DeviceBinding {
            id: DeviceBindingId::new(),
            device_id: cmd.device_id,
            tenant_id: cmd.tenant_id,
            user_id: device.user_id,
            project_id: cmd.project_id,
            binding_kind: cmd.binding_kind,
            bound_at: Utc::now(),
        };
        self.repo.insert_binding(b.clone()).await?;
        self.bindings.write().unwrap().insert(b.id, b.clone());
        Ok(b)
    }

    async fn revoke_device(
        &self,
        cmd: RevokeDeviceCommand,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("tenant_admin") && !actor.is_platform_admin {
            return Err(IdentityError::PermissionDenied);
        }
        let mut d = self
            .devices
            .write()
            .unwrap()
            .get_mut(&cmd.device_id)
            .cloned()
            .ok_or(IdentityError::NotFound(format!(
                "device:{}",
                cmd.device_id.as_uuid()
            )))?;
        if d.tenant_id != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(d.tenant_id, cmd.tenant_id));
        }
        if d.status == DeviceStatus::Revoked {
            return Err(IdentityError::DeviceAlreadyRevoked);
        }
        d.status = DeviceStatus::Revoked;
        d.revoked_at = Some(Utc::now());
        self.repo.update_device(d.clone()).await?;
        self.devices.write().unwrap().insert(d.id, d.clone());
        Ok(d)
    }

    async fn record_login(
        &self,
        cmd: RecordLoginCommand,
        actor: &ActorContext,
    ) -> Result<User, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut u = self
            .users
            .write()
            .unwrap()
            .get_mut(&cmd.user_id)
            .cloned()
            .ok_or(IdentityError::NotFound(format!(
                "user:{}",
                cmd.user_id.as_uuid()
            )))?;
        if u.tenant_id != cmd.tenant_id {
            return Err(IdentityError::CrossTenantDenied(u.tenant_id, cmd.tenant_id));
        }
        u.last_login_at = Some(Utc::now());
        self.repo.update_user(u.clone()).await?;
        self.users.write().unwrap().insert(u.id, u.clone());
        Ok(u)
    }
}

#[async_trait]
impl IdentityQueryPort for InMemoryIdentityService {
    async fn get_user(&self, q: GetUserQuery, actor: &ActorContext) -> Result<User, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let u =
            self.users
                .read()
                .unwrap()
                .get(&q.user_id)
                .cloned()
                .ok_or(IdentityError::NotFound(format!(
                    "user:{}",
                    q.user_id.as_uuid()
                )))?;
        if u.tenant_id != q.tenant_id {
            return Err(IdentityError::CrossTenantDenied(u.tenant_id, q.tenant_id));
        }
        Ok(u)
    }

    async fn list_devices(
        &self,
        q: ListDevicesByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Device>, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        Ok(self
            .devices
            .read()
            .unwrap()
            .values()
            .filter(|d| d.tenant_id == q.tenant_id && d.user_id == q.user_id)
            .cloned()
            .collect())
    }

    async fn get_device(
        &self,
        tenant_id: TenantId,
        device_id: DeviceId,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError> {
        if !actor.is_platform_admin && TenantId::from(actor.tenant_id) != tenant_id {
            return Err(IdentityError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        let d = self
            .devices
            .read()
            .unwrap()
            .get(&device_id)
            .cloned()
            .ok_or(IdentityError::NotFound(format!(
                "device:{}",
                device_id.as_uuid()
            )))?;
        if d.tenant_id != tenant_id {
            return Err(IdentityError::CrossTenantDenied(d.tenant_id, tenant_id));
        }
        Ok(d)
    }
}

// =====================================================================
// InMemoryIdentityRepository
// =====================================================================

/// 基于内存的身份仓储实现(测试/开发用途)
pub struct InMemoryIdentityRepository {
    users: RwLock<HashMap<UserId, User>>,
    email_index: RwLock<HashMap<(TenantId, String), UserId>>,
    devices: RwLock<HashMap<DeviceId, Device>>,
    bindings: RwLock<HashMap<DeviceBindingId, DeviceBinding>>,
}

impl InMemoryIdentityRepository {
    /// 创建空的内存身份仓储实例
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            email_index: RwLock::new(HashMap::new()),
            devices: RwLock::new(HashMap::new()),
            bindings: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryIdentityRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdentityRepository for InMemoryIdentityRepository {
    async fn insert_user(&self, u: User) -> Result<(), IdentityError> {
        self.email_index
            .write()
            .unwrap()
            .insert((u.tenant_id, u.email.clone()), u.id);
        self.users.write().unwrap().insert(u.id, u);
        Ok(())
    }
    async fn get_user(&self, id: UserId) -> Result<User, IdentityError> {
        self.users
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(IdentityError::NotFound(format!("user:{}", id.as_uuid())))
    }
    async fn get_user_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>, IdentityError> {
        let id = self
            .email_index
            .read()
            .unwrap()
            .get(&(tenant_id, email.to_string()))
            .cloned();
        match id {
            Some(i) => Ok(self.users.read().unwrap().get(&i).cloned()),
            None => Ok(None),
        }
    }
    async fn update_user(&self, u: User) -> Result<(), IdentityError> {
        self.users.write().unwrap().insert(u.id, u);
        Ok(())
    }
    async fn insert_device(&self, d: Device) -> Result<(), IdentityError> {
        self.devices.write().unwrap().insert(d.id, d);
        Ok(())
    }
    async fn get_device(&self, id: DeviceId) -> Result<Device, IdentityError> {
        self.devices
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(IdentityError::NotFound(format!("device:{}", id.as_uuid())))
    }
    async fn update_device(&self, d: Device) -> Result<(), IdentityError> {
        self.devices.write().unwrap().insert(d.id, d);
        Ok(())
    }
    async fn list_devices_by_user(
        &self,
        tid: TenantId,
        uid: UserId,
    ) -> Result<Vec<Device>, IdentityError> {
        Ok(self
            .devices
            .read()
            .unwrap()
            .values()
            .filter(|d| d.tenant_id == tid && d.user_id == uid)
            .cloned()
            .collect())
    }
    async fn insert_binding(&self, b: DeviceBinding) -> Result<(), IdentityError> {
        self.bindings.write().unwrap().insert(b.id, b);
        Ok(())
    }
    async fn list_bindings_by_device(
        &self,
        did: DeviceId,
    ) -> Result<Vec<DeviceBinding>, IdentityError> {
        Ok(self
            .bindings
            .read()
            .unwrap()
            .values()
            .filter(|b| b.device_id == did)
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
    fn admin(tid: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid.0).with_role("tenant_admin")
    }

    fn dev(tid: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid.0)
    }

    #[test]
    fn user_status_as_str() {
        assert_eq!(UserStatus::Active.as_str(), "ACTIVE");
        assert_eq!(UserStatus::Invited.as_str(), "INVITED");
    }

    #[test]
    fn device_kind_as_str() {
        assert_eq!(DeviceKind::LocalRuntime.as_str(), "local_runtime");
        assert_eq!(DeviceKind::Web.as_str(), "web");
    }

    #[test]
    fn device_status_is_terminal() {
        assert!(DeviceStatus::Revoked.is_terminal());
        assert!(!DeviceStatus::Active.is_terminal());
    }

    #[test]
    fn tenant_role_as_str() {
        assert_eq!(TenantRole::TenantAdmin.as_str(), "tenant_admin");
    }

    #[test]
    fn credential_kind_as_str() {
        assert_eq!(CredentialKind::Password.as_str(), "password");
        assert_eq!(CredentialKind::ApiKey.as_str(), "api_key");
    }

    #[tokio::test]
    async fn create_user_requires_tenant_admin() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = dev(TenantId(tid));
        let res = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "u@x.com".to_string(),
                    display_name: "U".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::PermissionDenied)));
    }

    #[tokio::test]
    async fn email_must_be_unique_in_tenant() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = admin(TenantId(tid));
        svc.create_user(
            CreateUserCommand {
                tenant_id: TenantId(tid),
                email: "dup@x.com".to_string(),
                display_name: "A".to_string(),
                tenant_role: TenantRole::Developer,
                credential_ref: CredentialRefId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
        let res = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "dup@x.com".to_string(),
                    display_name: "B".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::EmailExists(_))));
    }

    #[tokio::test]
    async fn register_device_requires_project_ids_invn02() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let user = uuid::Uuid::new_v4();
        // 用 tenant_admin 避免 user_id mismatch
        let actor = admin(TenantId(tid));
        let res = svc
            .register_device(
                RegisterDeviceCommand {
                    tenant_id: TenantId(tid),
                    user_id: UserId(user),
                    kind: DeviceKind::Web,
                    device_cert_fingerprint: "abc".to_string(),
                    project_ids: vec![], // 缺 binding
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::IncompleteBinding)));
    }

    #[tokio::test]
    async fn register_device_with_binding_ok() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let admin = admin(TenantId(tid));
        let user = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "u@x.com".to_string(),
                    display_name: "U".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        let d = svc
            .register_device(
                RegisterDeviceCommand {
                    tenant_id: TenantId(tid),
                    user_id: user.id,
                    kind: DeviceKind::LocalRuntime,
                    device_cert_fingerprint: "abc".to_string(),
                    project_ids: vec![ProjectId::new()],
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(d.status, DeviceStatus::Active);
        assert_eq!(d.project_ids.len(), 1);
    }

    #[tokio::test]
    async fn revoke_device() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let admin = admin(TenantId(tid));
        let user = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "u@x.com".to_string(),
                    display_name: "U".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        let d = svc
            .register_device(
                RegisterDeviceCommand {
                    tenant_id: TenantId(tid),
                    user_id: user.id,
                    kind: DeviceKind::Web,
                    device_cert_fingerprint: "abc".to_string(),
                    project_ids: vec![ProjectId::new()],
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        let d2 = svc
            .revoke_device(
                RevokeDeviceCommand {
                    tenant_id: TenantId(tid),
                    device_id: d.id,
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(d2.status, DeviceStatus::Revoked);
        assert!(d2.revoked_at.is_some());
    }

    #[tokio::test]
    async fn revoke_already_revoked_rejected() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let admin = admin(TenantId(tid));
        let user = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "u@x.com".to_string(),
                    display_name: "U".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        let d = svc
            .register_device(
                RegisterDeviceCommand {
                    tenant_id: TenantId(tid),
                    user_id: user.id,
                    kind: DeviceKind::Web,
                    device_cert_fingerprint: "abc".to_string(),
                    project_ids: vec![ProjectId::new()],
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        svc.revoke_device(
            RevokeDeviceCommand {
                tenant_id: TenantId(tid),
                device_id: d.id,
                actor_user_id: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        let res = svc
            .revoke_device(
                RevokeDeviceCommand {
                    tenant_id: TenantId(tid),
                    device_id: d.id,
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::DeviceAlreadyRevoked)));
    }

    #[tokio::test]
    async fn bind_device_requires_tenant_match() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let admin = admin(TenantId(tid));
        let user = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "u@x.com".to_string(),
                    display_name: "U".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        let d = svc
            .register_device(
                RegisterDeviceCommand {
                    tenant_id: TenantId(tid),
                    user_id: user.id,
                    kind: DeviceKind::Web,
                    device_cert_fingerprint: "abc".to_string(),
                    project_ids: vec![ProjectId::new()],
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        let res = svc
            .bind_device(
                BindDeviceCommand {
                    tenant_id: TenantId(tid),
                    device_id: d.id,
                    project_id: ProjectId::new(),
                    binding_kind: BindingKind::Contributor,
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn cross_tenant_register_denied() {
        let svc = InMemoryIdentityService::new();
        let me = uuid::Uuid::new_v4();
        let other = uuid::Uuid::new_v4();
        let actor = dev(TenantId(me));
        let res = svc
            .register_device(
                RegisterDeviceCommand {
                    tenant_id: TenantId(other),
                    user_id: UserId::new(),
                    kind: DeviceKind::Web,
                    device_cert_fingerprint: "abc".to_string(),
                    project_ids: vec![ProjectId::new()],
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn record_login_updates_last_login() {
        let svc = InMemoryIdentityService::new();
        let tid = uuid::Uuid::new_v4();
        let admin = admin(TenantId(tid));
        let user = svc
            .create_user(
                CreateUserCommand {
                    tenant_id: TenantId(tid),
                    email: "u@x.com".to_string(),
                    display_name: "U".to_string(),
                    tenant_role: TenantRole::Developer,
                    credential_ref: CredentialRefId::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        let u = svc
            .record_login(
                RecordLoginCommand {
                    tenant_id: TenantId(tid),
                    user_id: user.id,
                },
                &admin,
            )
            .await
            .unwrap();
        assert!(u.last_login_at.is_some());
    }
}

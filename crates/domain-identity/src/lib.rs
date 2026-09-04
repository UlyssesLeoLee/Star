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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub email: String,
    pub display_name: String,
    pub status: UserStatus,
    pub credential_ref: CredentialRefId,
    pub tenant_role: TenantRole,
    pub mfa_enabled: bool,
    pub mfa_secret_ref: Option<CredentialRefId>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Suspended,
    Invited,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Invited => "INVITED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantRole {
    TenantAdmin,
    ProjectAdmin,
    Developer,
    Viewer,
}

impl TenantRole {
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
pub struct Device {
    pub id: DeviceId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub kind: DeviceKind,
    pub device_cert_fingerprint: String,
    pub status: DeviceStatus,
    pub project_ids: Vec<ProjectId>,
    pub registered_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceKind {
    LocalRuntime,
    Cli,
    Web,
    Mobile,
}

impl DeviceKind {
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
pub enum DeviceStatus {
    Pending,
    Active,
    Revoked,
}

impl DeviceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Active => "ACTIVE",
            Self::Revoked => "REVOKED",
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Revoked)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: CredentialId,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub kind: CredentialKind,
    /// INV-ID-03:仅 hash,绝不存明文
    pub secret_hash: String,
    pub mfa_secret_ref: Option<CredentialRefId>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialKind {
    Password,
    ApiKey,
    OAuthToken,
}

impl CredentialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::ApiKey => "api_key",
            Self::OAuthToken => "oauth_token",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBinding {
    pub id: DeviceBindingId,
    pub device_id: DeviceId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub project_id: ProjectId,
    pub binding_kind: BindingKind,
    pub bound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingKind {
    Owner,
    Contributor,
    ReadOnly,
}

impl BindingKind {
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
pub enum IdentityError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("email already exists: {0}")]
    EmailExists(String),
    #[error("device triple binding incomplete (INV-ID-02): need tenant+user+project")]
    IncompleteBinding,
    #[error("device already revoked")]
    DeviceAlreadyRevoked,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    pub tenant_id: TenantId,
    pub email: String,
    pub display_name: String,
    pub tenant_role: TenantRole,
    pub credential_ref: CredentialRefId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub kind: DeviceKind,
    pub device_cert_fingerprint: String,
    pub project_ids: Vec<ProjectId>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindDeviceCommand {
    pub tenant_id: TenantId,
    pub device_id: DeviceId,
    pub project_id: ProjectId,
    pub binding_kind: BindingKind,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeDeviceCommand {
    pub tenant_id: TenantId,
    pub device_id: DeviceId,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordLoginCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserQuery {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDevicesByUserQuery {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait IdentityCommandPort: Send + Sync {
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor: &ActorContext,
    ) -> Result<User, IdentityError>;

    async fn register_device(
        &self,
        cmd: RegisterDeviceCommand,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError>;

    async fn bind_device(
        &self,
        cmd: BindDeviceCommand,
        actor: &ActorContext,
    ) -> Result<DeviceBinding, IdentityError>;

    async fn revoke_device(
        &self,
        cmd: RevokeDeviceCommand,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError>;

    async fn record_login(
        &self,
        cmd: RecordLoginCommand,
        actor: &ActorContext,
    ) -> Result<User, IdentityError>;
}

#[async_trait]
pub trait IdentityQueryPort: Send + Sync {
    async fn get_user(&self, q: GetUserQuery, actor: &ActorContext) -> Result<User, IdentityError>;

    async fn list_devices(
        &self,
        q: ListDevicesByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Device>, IdentityError>;

    async fn get_device(
        &self,
        tenant_id: TenantId,
        device_id: DeviceId,
        actor: &ActorContext,
    ) -> Result<Device, IdentityError>;
}

#[async_trait]
pub trait IdentityRepository: Send + Sync {
    async fn insert_user(&self, u: User) -> Result<(), IdentityError>;
    async fn get_user(&self, id: UserId) -> Result<User, IdentityError>;
    async fn get_user_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>, IdentityError>;
    async fn update_user(&self, u: User) -> Result<(), IdentityError>;

    async fn insert_device(&self, d: Device) -> Result<(), IdentityError>;
    async fn get_device(&self, id: DeviceId) -> Result<Device, IdentityError>;
    async fn update_device(&self, d: Device) -> Result<(), IdentityError>;
    async fn list_devices_by_user(
        &self,
        tid: TenantId,
        uid: UserId,
    ) -> Result<Vec<Device>, IdentityError>;

    async fn insert_binding(&self, b: DeviceBinding) -> Result<(), IdentityError>;
    async fn list_bindings_by_device(
        &self,
        did: DeviceId,
    ) -> Result<Vec<DeviceBinding>, IdentityError>;
}

// =====================================================================
// InMemoryIdentityService
// =====================================================================

pub struct InMemoryIdentityService {
    repo: Arc<dyn IdentityRepository>,
    users: Arc<RwLock<HashMap<UserId, User>>>,
    devices: Arc<RwLock<HashMap<DeviceId, Device>>>,
    bindings: Arc<RwLock<HashMap<DeviceBindingId, DeviceBinding>>>,
}

impl InMemoryIdentityService {
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

pub struct InMemoryIdentityRepository {
    users: RwLock<HashMap<UserId, User>>,
    email_index: RwLock<HashMap<(TenantId, String), UserId>>,
    devices: RwLock<HashMap<DeviceId, Device>>,
    bindings: RwLock<HashMap<DeviceBindingId, DeviceBinding>>,
}

impl InMemoryIdentityRepository {
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
                    tenant_id: other,
                    user_id: UserId.new(),
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

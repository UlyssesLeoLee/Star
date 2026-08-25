//! Identity 端口(Port Traits)与命令/查询 DTO
//!
//! **端口清单**:
//! - `IdentityCommandPort`:5 方法(写)
//! - `IdentityQueryPort`:5 方法(读)
//! - `IdentityRepository`:5 方法(纯数据访问)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Credential, Device, DeviceBinding, Role, User};
use crate::error::IdentityError;
use crate::value_object::{
    CredentialType, DeviceId, DeviceType, ProjectId, RoleId, TenantId, UserId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateUserCommand`(颁发 user_id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 邮箱(INV-IDN-01 tenant 内唯一)
    pub email: String,
    /// 显示名
    pub display_name: String,
    /// 头像 URL
    pub avatar_url: Option<String>,
    /// 初始凭证(密码 hash / Passkey 等)
    pub initial_credential: Option<CredentialSpec>,
}

/// 凭证规格(创建 User 时可选同步插入 Credential)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSpec {
    /// 凭证类型
    pub credential_type: CredentialType,
    /// 已 hash 的凭证
    pub hash: String,
    /// OIDC Provider(仅 OIDC 类型)
    pub provider_id: Option<String>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// `UpdateUserCommand`(更新 User 元数据,乐观锁)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserCommand {
    /// User ID
    pub user_id: UserId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 期望版本号
    pub expected_version: u32,
    /// 新显示名
    pub display_name: Option<String>,
    /// 新头像
    pub avatar_url: Option<Option<String>>,
}

/// `RecordLoginCommand`(记录登录,触发 `UserLoggedIn` 事件)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordLoginCommand {
    /// User ID
    pub user_id: UserId,
    /// 设备 ID
    pub device_id: DeviceId,
    /// 设备类型
    pub device_type: DeviceType,
}

/// `BindDeviceCommand`(设备-用户-项目 三重绑定,§23.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindDeviceCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Device ID
    pub device_id: DeviceId,
    /// User ID
    pub user_id: UserId,
    /// Project ID(可空,None 表示 tenant-wide)
    pub project_id: Option<ProjectId>,
    /// 绑定原因
    pub reason: Option<String>,
}

/// `CreateRoleCommand`(创建租户内角色)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 角色名
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 权限码列表
    pub permissions: Vec<String>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListUserQuery`(按租户列出用户)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 按 email 模糊搜索
    pub email_contains: Option<String>,
    /// 按状态过滤
    pub status: Option<crate::value_object::UserStatus>,
    /// 分页:limit
    pub limit: u32,
    /// 分页:offset
    pub offset: u32,
}

impl Default for ListUserQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            email_contains: None,
            status: None,
            limit: 50,
            offset: 0,
        }
    }
}

// =====================================================================
// 端口:IdentityCommandPort(5 方法)
// =====================================================================

/// **Identity 命令端口**
#[async_trait]
pub trait IdentityCommandPort: Send + Sync {
    /// 创建 User(INV-IDN-01/03/04 校验)
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor: ActorContext,
    ) -> Result<User, IdentityError>;

    /// 更新 User 元数据(乐观锁)
    async fn update_user(
        &self,
        cmd: UpdateUserCommand,
        actor: ActorContext,
    ) -> Result<User, IdentityError>;

    /// 记录登录(更新 last_login_at + 触发事件)
    async fn record_login(
        &self,
        cmd: RecordLoginCommand,
        actor: ActorContext,
    ) -> Result<(), IdentityError>;

    /// 设备绑定(§23.2 三重绑定,INV-IDN-02)
    async fn bind_device(
        &self,
        cmd: BindDeviceCommand,
        actor: ActorContext,
    ) -> Result<DeviceBinding, IdentityError>;

    /// 创建 Role
    async fn create_role(
        &self,
        cmd: CreateRoleCommand,
        actor: ActorContext,
    ) -> Result<Role, IdentityError>;
}

// =====================================================================
// 端口:IdentityQueryPort(5 方法)
// =====================================================================

/// **Identity 查询端口**
#[async_trait]
pub trait IdentityQueryPort: Send + Sync {
    /// 按 ID 查询 User
    async fn get_user(
        &self,
        id: UserId,
        viewer: ActorContext,
    ) -> Result<User, IdentityError>;

    /// 按 email 查询 User
    async fn get_user_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
        viewer: ActorContext,
    ) -> Result<User, IdentityError>;

    /// 列出 User(按租户)
    async fn list_users(
        &self,
        q: ListUserQuery,
        viewer: ActorContext,
    ) -> Result<Vec<User>, IdentityError>;

    /// 列出 User 关联的 Device
    async fn list_devices(
        &self,
        user_id: UserId,
        viewer: ActorContext,
    ) -> Result<Vec<Device>, IdentityError>;

    /// 查询 User 关联的 Role
    async fn list_user_roles(
        &self,
        user_id: UserId,
        viewer: ActorContext,
    ) -> Result<Vec<Role>, IdentityError>;

    /// 按 ID 查询 Role
    async fn get_role(
        &self,
        id: RoleId,
        viewer: ActorContext,
    ) -> Result<Role, IdentityError>;

    /// 按 name 查询 Role
    async fn get_role_by_name(
        &self,
        tenant_id: TenantId,
        name: &str,
        viewer: ActorContext,
    ) -> Result<Role, IdentityError>;

    /// 列出 User 关联的 Credential(脱敏,只返回 metadata)
    async fn list_user_credentials(
        &self,
        user_id: UserId,
        viewer: ActorContext,
    ) -> Result<Vec<Credential>, IdentityError>;
}

// =====================================================================
// 仓库端口
// =====================================================================

/// **Identity 仓库端口**
#[async_trait]
pub trait IdentityRepository: Send + Sync {
    /// 插入 User
    async fn insert_user(&self, user: &User) -> Result<(), IdentityError>;
    /// 按 ID 读取 User
    async fn find_user(&self, id: UserId) -> Result<Option<User>, IdentityError>;
    /// 按 tenant 列出全部 email(供唯一性校验)
    async fn list_emails(&self, tenant_id: TenantId) -> Result<Vec<String>, IdentityError>;
    /// 更新 User
    async fn update_user(&self, user: &User) -> Result<(), IdentityError>;
    /// 列出全部 Device
    async fn list_devices(&self) -> Result<Vec<Device>, IdentityError>;
}

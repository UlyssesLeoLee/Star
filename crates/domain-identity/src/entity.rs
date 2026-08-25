//! Identity 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.23 (`user` / `device` / `device_binding` / `credential` / `role`)
//! - `docs/specs/domain-identity-spec.md` §2 (实体清单)
//!
//! 包含 5 个核心实体:
//! - `User` — 平台用户(§4.23.1)
//! - `Device` — 设备(§4.23.2)
//! - `DeviceBinding` — 设备-用户-项目 三重绑定(§4.23.3,§23.2)
//! - `Credential` — 凭证(密码/Passkey/OIDC/API Token)
//! - `Role` — 角色(tenant 内 RBAC)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    CredentialType, DeviceId, DeviceType, ProjectId, RoleId, TenantId, UserId,
};

// =====================================================================
// User 聚合根(§4.23.1)
// =====================================================================

/// **User 聚合根**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 主键
    pub id: UserId,
    /// 租户 ID(必带)
    pub tenant_id: TenantId,
    /// 邮箱(在 tenant 内唯一,INV-IDN-01)
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 状态
    pub status: crate::value_object::UserStatus,
    /// 头像 URL
    pub avatar_url: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最近登录时间
    pub last_login_at: Option<DateTime<Utc>>,
    /// 乐观锁版本
    pub version: u32,
}

impl User {
    /// 字段数(用于 §4.23.1 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 10;

    /// 是否处于活跃状态
    pub fn is_active(&self) -> bool {
        self.status == crate::value_object::UserStatus::Active
    }

    /// 升级乐观锁版本
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Device(§4.23.2)
// =====================================================================

/// **Device**(设备实体)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// 主键
    pub id: DeviceId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 所属用户
    pub user_id: UserId,
    /// 设备指纹(浏览器/设备 ID 哈希)
    pub device_fingerprint: String,
    /// 设备类型
    pub device_type: DeviceType,
    /// 设备名称(用户可读)
    pub device_name: Option<String>,
    /// 首次见到时间
    pub first_seen_at: DateTime<Utc>,
    /// 最近活跃时间
    pub last_seen_at: DateTime<Utc>,
    /// 是否可信设备
    pub trusted: bool,
    /// 乐观锁版本
    pub version: u32,
}

impl Device {
    /// 字段数(用于 §4.23.2 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 10;

    /// 升级乐观锁版本
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.last_seen_at = Utc::now();
    }
}

// =====================================================================
// DeviceBinding(§4.23.3 + §23.2 三重绑定)
// =====================================================================

/// **DeviceBinding**(设备-用户-项目 三重绑定,§23.2)
///
/// INV-IDN-02:同一 (device_id, user_id, project_id) 三元组在平台内全局唯一
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBinding {
    /// 主键
    pub id: crate::value_object::DeviceBindingId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 设备
    pub device_id: DeviceId,
    /// 用户
    pub user_id: UserId,
    /// 项目(可空,空表示 tenant-wide 绑定)
    pub project_id: Option<ProjectId>,
    /// 绑定时间
    pub bound_at: DateTime<Utc>,
    /// 绑定原因
    pub reason: Option<String>,
    /// 乐观锁版本
    pub version: u32,
}

impl DeviceBinding {
    /// 字段数(用于 §4.23.3 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 8;

    /// 是否为 tenant-wide 绑定(project_id 为 None)
    pub fn is_tenant_wide(&self) -> bool {
        self.project_id.is_none()
    }

    /// 升级乐观锁版本
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
    }
}

// =====================================================================
// Credential(§4.23.4)
// =====================================================================

/// **Credential**(凭证:密码 / Passkey / OIDC / API Token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// 主键
    pub id: crate::value_object::CredentialId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 所属用户
    pub user_id: UserId,
    /// 凭证类型
    pub credential_type: CredentialType,
    /// 已 hash 的凭证(永不存明文)
    pub hash: String,
    /// 关联的 OIDC Provider(仅 OIDC 类型)
    pub provider_id: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间(None 表示永不过期)
    pub expires_at: Option<DateTime<Utc>>,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 乐观锁版本
    pub version: u32,
}

impl Credential {
    /// 字段数(用于 §4.23.4 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 10;

    /// 是否已过期
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.map_or(false, |exp| exp < now)
    }
}

// =====================================================================
// Role(§4.23.5)
// =====================================================================

/// **Role**(角色,在 tenant 内定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 主键
    pub id: RoleId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 角色名(`tenant_admin` / `developer` 等)
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 关联的权限码列表
    pub permissions: Vec<String>,
    /// 是否为内置角色(不可删除)
    pub built_in: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本
    pub version: u32,
}

impl Role {
    /// 字段数(用于 §4.23.5 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 10;

    /// 是否具备权限 `perm`
    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| p == perm)
    }
}

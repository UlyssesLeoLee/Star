//! Identity 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.23 (`user` / `device` / `device_binding` / `credential` / `role` schema)
//! - `docs/specs/domain-identity-spec.md` §3 (基本类型)
//!
//! 集中放置强类型 ID 与核心 enum。

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID
// =====================================================================

define_uuid_id!(UserId);
define_uuid_id!(DeviceId);
define_uuid_id!(DeviceBindingId);
define_uuid_id!(CredentialId);
define_uuid_id!(RoleId);
define_uuid_id!(TenantId); // 跨 crate 占位,Phase 3 由 domain-tenant 颁发
define_uuid_id!(ProjectId); // 跨 crate 占位,Phase 3 由 domain-project 颁发

// =====================================================================
// 枚举:UserStatus(data-design §4.23.1 ck_user_status)
// =====================================================================

/// **User 状态**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    /// 活跃
    Active,
    /// 禁用
    Disabled,
    /// 锁定(密码尝试过多等)
    Locked,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "ACTIVE",
            Self::Disabled => "DISABLED",
            Self::Locked => "LOCKED",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:DeviceType(data-design §4.23.2 ck_device_type)
// =====================================================================

/// **设备类型**(`device.device_type` 列)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceType {
    /// Web 浏览器
    Web,
    /// 移动端(iOS / Android)
    Mobile,
    /// 桌面端
    Desktop,
    /// CLI / IDE 插件
    CLI,
}

impl Default for DeviceType {
    fn default() -> Self {
        Self::Web
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Web => "WEB",
            Self::Mobile => "MOBILE",
            Self::Desktop => "DESKTOP",
            Self::CLI => "CLI",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:CredentialType(data-design §4.23.4 ck_credential_type)
// =====================================================================

/// **凭证类型**(`credential.credential_type` 列)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CredentialType {
    /// 密码(Argon2id hash)
    Password,
    /// Passkey(WebAuthn)
    Passkey,
    /// OIDC 第三方登录
    Oidc,
    /// API Token
    ApiToken,
}

impl std::fmt::Display for CredentialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Password => "PASSWORD",
            Self::Passkey => "PASSKEY",
            Self::Oidc => "OIDC",
            Self::ApiToken => "API_TOKEN",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 角色常量
// =====================================================================

/// Identity 相关标准角色常量
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 用户本人
    pub const USER: &str = "user";
    /// 服务账户
    pub const SERVICE_ACCOUNT: &str = "service_account";
}

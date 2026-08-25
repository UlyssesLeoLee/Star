//! Tenant 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.1 (`tenant` / `tenant_policy` / `tenant_quota` schema)
//! - `docs/specs/domain-tenant-spec.md` §2 (实体清单) / §3 (基本类型)
//!
//! 集中放置强类型 ID 与核心 enum,与 `entity` / `port` 解耦。

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

define_uuid_id!(TenantId);
define_uuid_id!(TenantPolicyId);
define_uuid_id!(TenantQuotaId);

// =====================================================================
// 枚举:TenantStatus(data-design §4.1 ck_tenant_status)
// =====================================================================

/// **Tenant 状态**(`tenant.status` 列,CHECK 约束白名单)
///
/// 来源: docs/data-design.md §4.1 (`ck_tenant_status`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantStatus {
    /// 活跃(正常服务)
    Active,
    /// 暂停(临时禁用,可恢复)
    Suspended,
    /// 归档(长期停用,只读)
    Archived,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl std::fmt::Display for TenantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Archived => "ARCHIVED",
        };
        f.write_str(s)
    }
}

impl TenantStatus {
    /// 是否允许迁移到目标状态(INV-TEN-02)。
    ///
    /// 合法迁移:
    /// - Active → Suspended(可暂停)
    /// - Active → Archived(可归档)
    /// - Suspended → Active(可恢复)
    /// - Suspended → Archived(可从暂停直接归档)
    /// - Archived → Active(可重新激活)
    /// - 同状态 → OK(幂等)
    pub fn can_transition_to(self, target: Self) -> bool {
        use TenantStatus::*;
        if self == target {
            return true;
        }
        match (self, target) {
            (Active, Suspended) | (Active, Archived) => true,
            (Suspended, Active) | (Suspended, Archived) => true,
            (Archived, Active) => true,
            _ => false,
        }
    }
}

// =====================================================================
// 枚举:TenantTier(data-design §4.1 ck_tenant_tier)
// =====================================================================

/// **Tenant 等级**(`tenant.tier` 列,服务套餐级别)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantTier {
    /// 免费版
    Free,
    /// 专业版
    Pro,
    /// 企业版
    Enterprise,
}

impl Default for TenantTier {
    fn default() -> Self {
        Self::Free
    }
}

impl std::fmt::Display for TenantTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Free => "FREE",
            Self::Pro => "PRO",
            Self::Enterprise => "ENTERPRISE",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 标准角色(便于测试与调用方使用)
// =====================================================================

/// Tenant 相关标准角色常量
pub mod roles {
    /// 租户管理员(具备 TenantPolicy 写权限)
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 租户只读审计员
    pub const TENANT_AUDITOR: &str = "tenant_auditor";
    /// 平台运营
    pub const PLATFORM_OPERATOR: &str = "platform_operator";
}

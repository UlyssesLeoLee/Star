//! Tenant 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.1 (`tenant` / `tenant_policy` / `tenant_quota` schema)
//! - `docs/specs/domain-tenant-spec.md` §2 (实体清单)
//!
//! 包含 3 个核心实体:
//! - `Tenant` — 聚合根(主表 `tenant`)
//! - `TenantPolicy` — AI 策略(主表 `tenant_policy`,§4.1.2)
//! - `TenantQuota` — 配额(主表 `tenant_quota`,§4.1.3)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{TenantId, TenantPolicyId, TenantQuotaId, TenantStatus, TenantTier};

// =====================================================================
// Tenant 聚合根(§4.1.1)
// =====================================================================

/// **Tenant 聚合根**(继承 `data-design §4.1.1` DDL,共 9 字段)
///
/// 关键约束:
/// - 必带 `tenant_id`(13 类对象必带,§6.1,REQ-SEC-001)
/// - `tenant_key` 在平台内全局唯一(INV-TEN-01)
/// - 状态迁移必须合法(INV-TEN-02,见 [`TenantStatus::can_transition_to`])
/// - tenant_id 由本 crate 颁发(UUIDv4),不可调用方传入(§5.7,security-design §4.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// 主键(UUID,本 crate 颁发)
    pub id: TenantId,
    /// 租户业务键(`acme-corp`,平台全局唯一)
    pub tenant_key: String,
    /// 显示名称
    pub name: String,
    /// 状态
    pub status: TenantStatus,
    /// 服务等级
    pub tier: TenantTier,
    /// 联系邮箱
    pub contact_email: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本号
    pub version: u32,
}

impl Tenant {
    /// 字段数(用于 §4.1.1 DDL 对齐审计)。
    pub const FIELD_COUNT: usize = 9;

    /// 是否处于活跃状态。
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }

    /// 是否处于暂停状态。
    pub fn is_suspended(&self) -> bool {
        self.status == TenantStatus::Suspended
    }

    /// 是否处于归档状态。
    pub fn is_archived(&self) -> bool {
        self.status == TenantStatus::Archived
    }

    /// 升级乐观锁版本号。
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// TenantPolicy(§4.1.2)
// =====================================================================

/// **Tenant AI 策略**(`tenant_policy` 表,定义 AI 上云/本地的允许矩阵)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPolicy {
    /// 主键
    pub id: TenantPolicyId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 是否允许云端 AI(默认 true)
    pub cloud_ai_allowed: bool,
    /// 是否限制云端 AI 范围(白名单 provider)
    pub cloud_ai_restricted: bool,
    /// 仅允许本地 AI(禁止任何云端)
    pub local_ai_only: bool,
    /// 禁止上传代码(仅元数据)
    pub no_code_upload: bool,
    /// 仅元数据上传(代码不外发)
    pub metadata_only: bool,
    /// 白名单 Provider IDs(`cloud_ai_restricted = true` 时生效)
    pub specific_provider_ids: Vec<uuid::Uuid>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本
    pub version: u32,
}

impl TenantPolicy {
    /// 字段数(用于 §4.1.2 DDL 对齐审计)。
    pub const FIELD_COUNT: usize = 10;

    /// 是否拒绝所有云端 AI 调用(`local_ai_only = true`)。
    pub fn blocks_all_cloud_ai(&self) -> bool {
        self.local_ai_only
    }

    /// Provider `p` 是否被允许(`cloud_ai_restricted` 模式)。
    pub fn is_provider_allowed(&self, provider_id: uuid::Uuid) -> bool {
        if self.local_ai_only {
            return false;
        }
        if !self.cloud_ai_restricted {
            return self.cloud_ai_allowed;
        }
        self.specific_provider_ids.contains(&provider_id)
    }

    /// 升级乐观锁版本。
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// TenantQuota(§4.1.3)
// =====================================================================

/// **Tenant 配额**(`tenant_quota` 表,定义用量上限)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    /// 主键
    pub id: TenantQuotaId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 最大用户数
    pub max_users: u32,
    /// 最大 Workspace 数
    pub max_workspaces: u32,
    /// 最大 Project 数
    pub max_projects: u32,
    /// 最大存储字节数
    pub max_storage_bytes: u64,
    /// 已使用用户数
    pub used_users: u32,
    /// 已使用 Workspace 数
    pub used_workspaces: u32,
    /// 已使用 Project 数
    pub used_projects: u32,
    /// 已使用存储字节数
    pub used_storage_bytes: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本
    pub version: u32,
}

impl TenantQuota {
    /// 字段数(用于 §4.1.3 DDL 对齐审计)。
    pub const FIELD_COUNT: usize = 14;

    /// 是否超过用户数上限。
    pub fn is_user_quota_exceeded(&self) -> bool {
        self.used_users >= self.max_users
    }

    /// 是否超过 workspace 数上限。
    pub fn is_workspace_quota_exceeded(&self) -> bool {
        self.used_workspaces >= self.max_workspaces
    }

    /// 是否超过 project 数上限。
    pub fn is_project_quota_exceeded(&self) -> bool {
        self.used_projects >= self.max_projects
    }

    /// 是否超过存储上限。
    pub fn is_storage_quota_exceeded(&self) -> bool {
        self.used_storage_bytes >= self.max_storage_bytes
    }

    /// 升级乐观锁版本。
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

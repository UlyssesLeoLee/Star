//! Tenant 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.2 (CRUD 端点)
//! - `docs/specs/domain-tenant-spec.md` §4 (接口签名)
//!
//! **端口清单**:
//! - `TenantCommandPort`:4 方法(写)
//! - `TenantQueryPort`:4 方法(读)
//! - `TenantRepository`:5 方法(纯数据访问)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Tenant, TenantPolicy, TenantQuota};
use crate::error::TenantError;
use crate::value_object::{TenantId, TenantPolicyId, TenantStatus, TenantTier};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateTenantCommand`(颁发 tenant_id,创建新租户)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantCommand {
    /// 租户业务键(平台全局唯一,INV-TEN-01)
    pub tenant_key: String,
    /// 显示名称
    pub name: String,
    /// 服务等级
    pub tier: TenantTier,
    /// 联系邮箱
    pub contact_email: Option<String>,
    /// 初始 AI 策略(可选)
    pub initial_policy: Option<TenantPolicySpec>,
}

/// AI 策略规格(创建 Tenant 时同步插入 TenantPolicy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPolicySpec {
    /// 是否允许云端 AI
    pub cloud_ai_allowed: bool,
    /// 是否限制云端 AI 范围
    pub cloud_ai_restricted: bool,
    /// 仅本地 AI
    pub local_ai_only: bool,
    /// 禁止上传代码
    pub no_code_upload: bool,
    /// 仅元数据上传
    pub metadata_only: bool,
    /// 白名单 Provider IDs
    pub specific_provider_ids: Vec<uuid::Uuid>,
}

impl Default for TenantPolicySpec {
    fn default() -> Self {
        Self {
            cloud_ai_allowed: true,
            cloud_ai_restricted: false,
            local_ai_only: false,
            no_code_upload: false,
            metadata_only: false,
            specific_provider_ids: Vec::new(),
        }
    }
}

/// `UpdateTenantCommand`(更新 Tenant 元数据,乐观锁)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantCommand {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// 期望版本号
    pub expected_version: u32,
    /// 新名称
    pub name: Option<String>,
    /// 新联系邮箱
    pub contact_email: Option<Option<String>>,
    /// 新等级
    pub tier: Option<TenantTier>,
}

/// `ChangeTenantStatusCommand`(状态机迁移)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTenantStatusCommand {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// 目标状态
    pub target_status: TenantStatus,
    /// 期望版本号
    pub expected_version: u32,
    /// 原因(记入审计)
    pub reason: Option<String>,
}

/// `UpdateTenantPolicyCommand`(更新 AI 策略,乐观锁)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantPolicyCommand {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// TenantPolicy ID
    pub policy_id: TenantPolicyId,
    /// 期望版本号
    pub expected_version: u32,
    /// 新 cloud_ai_allowed
    pub cloud_ai_allowed: Option<bool>,
    /// 新 cloud_ai_restricted
    pub cloud_ai_restricted: Option<bool>,
    /// 新 local_ai_only
    pub local_ai_only: Option<bool>,
    /// 新 no_code_upload
    pub no_code_upload: Option<bool>,
    /// 新 metadata_only
    pub metadata_only: Option<bool>,
    /// 新白名单 Providers(`Some(vec)` 表示覆盖,`None` 表示不修改)
    pub specific_provider_ids: Option<Vec<uuid::Uuid>>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListTenantQuery`(列表查询,仅 platform_operator 可用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTenantQuery {
    /// 按状态过滤
    pub status: Option<TenantStatus>,
    /// 按等级过滤
    pub tier: Option<TenantTier>,
    /// 分页:limit
    pub limit: u32,
    /// 分页:offset
    pub offset: u32,
}

impl Default for ListTenantQuery {
    fn default() -> Self {
        Self {
            status: None,
            tier: None,
            limit: 50,
            offset: 0,
        }
    }
}

// =====================================================================
// 端口:TenantCommandPort(4 方法)
// =====================================================================

/// **Tenant 命令端口**(写操作 4 方法)
#[async_trait]
pub trait TenantCommandPort: Send + Sync {
    /// 创建 Tenant(颁发 tenant_id,INV-TEN-01 校验 tenant_key 唯一)
    async fn create_tenant(
        &self,
        cmd: CreateTenantCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 更新 Tenant 元数据(乐观锁)
    async fn update_tenant(
        &self,
        cmd: UpdateTenantCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 状态机迁移(INV-TEN-02)
    async fn change_status(
        &self,
        cmd: ChangeTenantStatusCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 更新 TenantPolicy
    async fn update_tenant_policy(
        &self,
        cmd: UpdateTenantPolicyCommand,
        actor: ActorContext,
    ) -> Result<TenantPolicy, TenantError>;
}

// =====================================================================
// 端口:TenantQueryPort(4 方法)
// =====================================================================

/// **Tenant 查询端口**(读操作 4 方法)
#[async_trait]
pub trait TenantQueryPort: Send + Sync {
    /// 按 ID 查询
    async fn get_by_id(
        &self,
        id: TenantId,
        viewer: ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 按 tenant_key 查询
    async fn get_by_key(
        &self,
        tenant_key: &str,
        viewer: ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 列表查询(带过滤)
    async fn list_tenants(
        &self,
        q: ListTenantQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Tenant>, TenantError>;

    /// 查询 TenantPolicy
    async fn get_tenant_policy(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<TenantPolicy, TenantError>;

    /// 查询 TenantQuota
    async fn get_tenant_quota(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<TenantQuota, TenantError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **Tenant 仓库端口**(供 SQLx / 内存 / 测试 Adapter 实现)
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// 插入新 Tenant
    async fn insert(&self, tenant: &Tenant) -> Result<(), TenantError>;
    /// 按 ID 读取
    async fn find_by_id(&self, id: TenantId) -> Result<Option<Tenant>, TenantError>;
    /// 按 tenant_key 读取
    async fn find_by_key(&self, key: &str) -> Result<Option<Tenant>, TenantError>;
    /// 更新(乐观锁)
    async fn update(&self, tenant: &Tenant) -> Result<(), TenantError>;
    /// 列出全部(供 service 做 tenant_key 唯一性校验)
    async fn list_all_keys(&self) -> Result<Vec<String>, TenantError>;
}

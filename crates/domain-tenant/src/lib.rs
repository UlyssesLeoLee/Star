//! Tenant 最高安全边界
//!
//! **crate**: `domain-tenant`
//! **上游 spec**: docs/specs/domain-tenant-spec.md §4.10.2 / §6.1 (13 类对象 1-2)
//! **基本设计**: docs/basic-design.md §2.1(表 18) / §4.10.2 / §5.7
//! **数据设计**: docs/data-design.md §4.1 (`tenant` schema) / §7 (RLS)
//! **API 设计**: docs/api-design.md §3.2 (domain-tenant 端点) / §5.5 / §8.3.7
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - 任何聚合根 INSERT/UPDATE 必须携带 tenant_id(§6.1,REQ-SEC-001)
//! - tenant_id 由本 crate 颁发(UUIDv7),不可调用方传入(§5.7,security-design §4.1)
//! - 跨 tenant 访问返回 403 SEC-007 + Audit(security-design §3.5.4)
//! - ProviderDataBoundary.credential_ref 永不明文化(security-design §5.4)

//! ## 上游依赖
//!
//! 本 crate 为依赖图最底层(basic-design §2.3),无 domain-* 上游依赖。

//! ## 关键引用
//!
//! 13 类 tenant_id 对象(§6.1,§4.10.4,已修复 F-06):本 crate 颁发 tenant_id

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================

/// Tenant (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.1 (`tenant` schema) / §7 (RLS)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// TenantPolicy (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.1 (`tenant` schema) / §7 (RLS)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPolicy {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// SecurityPolicy (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.1 (`tenant` schema) / §7 (RLS)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// ProviderDataBoundary (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.1 (`tenant` schema) / §7 (RLS)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDataBoundary {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 端口(Port / 抽象)
// =====================================================================

/// **TenantCommandPort**(命令端口)
///
/// 来源: docs/api-design.md §3.2 (domain-tenant 端点) / §5.5 / §8.3.7
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait TenantCommandPort: Send + Sync {
    async fn create_tenant(
        &self,
        cmd: CreateTenantCommand,
        actor: ActorContext,
    ) -> Result<TenantId, TenantError>;
    async fn update_tenant(
        &self,
        cmd: UpdateTenantCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;
    async fn replace_security_policy(
        &self,
        cmd: ReplaceSecurityPolicyCommand,
        actor: ActorContext,
    ) -> Result<SecurityPolicy, TenantError>;
    async fn upsert_provider_boundary(
        &self,
        cmd: UpsertProviderBoundaryCommand,
        actor: ActorContext,
    ) -> Result<ProviderDataBoundary, TenantError>;
    async fn transition_status(
        &self,
        cmd: TransitionTenantStatusCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;
}


/// **TenantQueryPort**(查询端口)
///
/// 来源: docs/api-design.md §3.2 (domain-tenant 端点) / §5.5 / §8.3.7
#[async_trait]
pub trait TenantQueryPort: Send + Sync {
    async fn get_current(
        &self,
        _dummy: (),
        viewer: ActorContext,
    ) -> Result<Tenant, TenantError>;
    async fn get_by_id(
        &self,
        id: TenantId,
        viewer: ActorContext,
    ) -> Result<Tenant, TenantError>;
    async fn get_security_policy(
        &self,
        id: TenantId,
        viewer: ActorContext,
    ) -> Result<SecurityPolicy, TenantError>;
    async fn list_provider_boundaries(
        &self,
        id: TenantId,
        viewer: ActorContext,
    ) -> Result<Vec<ProviderDataBoundary>, TenantError>;
    async fn get_usage_report(
        &self,
        id: TenantId,
        viewer: ActorContext,
    ) -> Result<TenantUsageReport, TenantError>;
}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================

/// Domain Event: `star.events.tenant.tenant.created.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant01Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Domain Event: `star.events.tenant.tenant.security_policy_replaced.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant02Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Domain Event: `star.events.tenant.tenant.provider_boundary_upserted.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant03Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Domain Event: `star.events.tenant.tenant.status_changed.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant04Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 类型别名与命令/查询/返回类型占位
// =====================================================================
/// **ID 类型别名**(Phase 1 骨架:均为 UUID 别名)
///
/// 真实使用应由 `domain-identity` 颁发强类型 ID(§23.2);
/// 骨架阶段以 `Uuid` 替代以避免跨 crate 编译依赖。

pub type ProviderDataBoundaryId = Uuid;
pub type SecurityPolicyId = Uuid;
pub type TenantId = Uuid;
pub type TenantPolicyId = Uuid;

/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)

/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等
/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceSecurityPolicyCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsageReport {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionTenantStatusCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertProviderBoundaryCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}


// =====================================================================
// Error
// =====================================================================

/// **Tenant 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("not found: {0}")]
    NotFound(Uuid),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 共享类型
// =====================================================================

/// **Actor 上下文**(来自 `domain-identity` / `domain-permission` 的 JWT claim)
///
/// **骨架阶段**: 字段占位;Phase 2 由 `domain-identity` 颁发的 ActorContext 取代
/// 本 crate 内的占位定义(避免循环依赖)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID
    pub user_id: Uuid,
    /// 当前租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<Uuid>,
    /// 当前 Project IDs(用于 Project Policy 校验)
    pub project_ids: Vec<Uuid>,
    /// 当前用户角色(`tenant_admin` / `project_admin` / `developer` / `viewer`)
    pub roles: Vec<String>,
}

// =====================================================================
// 单元测试占位
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// **骨架阶段**: 最小冒烟测试,验证 crate 可编译、ActorContext 字段可达。
    /// Phase 2 由具体 spec 引入完整单元测试(状态机覆盖 / RLS 矩阵等)。
    #[test]
    fn actor_context_skeleton() {
        let actor = ActorContext {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            device_id: None,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
        };
        assert!(!actor.tenant_id.is_nil(), "tenant_id must be non-nil (§6.1,REQ-SEC-001)");
    }
}

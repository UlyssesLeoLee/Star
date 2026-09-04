//! Adapter 实现层 (PostgreSQL / NATS / ObjectStorage / SCM / Agent)
//!
//! **crate**: `infrastructure`
//! **上游 spec**: docs/specs/infrastructure-spec.md §3 ACL / §13.1 数据面
//! **基本设计**: docs/basic-design.md §1.1 / §3 ACL
//! **数据设计**: docs/data-design.md §6 / §7 (RLS) / §8 (索引)
//! **API 设计**: docs/api-design.md —
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - 本 crate 不允许反向依赖 `domain`,只实现 Domain 定义的 Port(§3 ACL)
//! - PostgreSQL = 默认 SoR(§13.1,§30.6)
//! - Database 保持单一 PostgreSQL(非 Database per Domain,§13.5)

//! ## 上游依赖
//!
//! 本 supporting crate 编排多个 domain-*(骨架阶段不实际 import,仅占位模块结构)。

//! ## 关键引用
//!
//! Adapter 仅实现 Domain Port(§3 ACL);非 Database per Domain(§13.5,§30.6)

#![warn(missing_docs)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use uuid::Uuid;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================
// (本 crate 为 supporting 层,无业务实体 — 实体由 domain-* crate 拥有)

// =====================================================================
// 端口(Port / 抽象)
// =====================================================================

/// **AdapterRegistry**(命令端口)
///
/// 来源: docs/api-design.md —
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait AdapterRegistry: Send + Sync {
    async fn register_postgres_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    async fn register_nats_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    async fn register_object_storage_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    async fn register_scm_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    async fn register_agent_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
}

/// **AdapterQuery**(查询端口)
///
/// 来源: docs/api-design.md —
#[async_trait]
pub trait AdapterQuery: Send + Sync {
    async fn list_registered_adapters(
        &self,
        _dummy: (),
        viewer: ActorContext,
    ) -> Result<Vec<AdapterDescriptor>, InfrastructureError>;
}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================
// (本 crate 不直接发布 Domain Event,事件由 domain-* crate 拥有)

// =====================================================================
// 类型别名与命令/查询/返回类型占位
// =====================================================================
/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)

/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等
/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterDescriptor {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

// =====================================================================
// Error
// =====================================================================

/// **Infrastructure 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
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
            is_local_runtime: false,
            is_platform_admin: false,
            is_agent_session: false,
            tenant_policy_id: None,
            project_ids: vec![],
            workspace_ids: vec![],
            roles: vec!["developer".to_string()],
        };
        assert!(
            !actor.tenant_id.is_nil(),
            "tenant_id must be non-nil (§6.1,REQ-SEC-001)"
        );
    }
}

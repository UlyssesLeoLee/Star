//! API Gateway 入口 (REST / WS)
//!
//! **crate**: `api`
//! **上游 spec**: docs/specs/api-spec.md §3 API 端点 / §13.1 gateway role
//! **基本设计**: docs/basic-design.md §1.1 / §13.1
//! **数据设计**: docs/data-design.md —
//! **API 设计**: docs/api-design.md §3 全部 / §5 Event Subject / §8 错误码
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - Gateway 角色与 work-core / identity / worker 同级最小闭环(§13.1)
//! - Realtime 仅在 Long Connection Scaling Boundary 出现后拆出(§13.1,§15)

//! ## 上游依赖
//!
//! 本 supporting crate 编排多个 domain-*(骨架阶段不实际 import,仅占位模块结构)。

//! ## 关键引用
//!
//! Gateway 属最小闭环 4 角色之一(§13.1);Realtime 暂不部署(§15)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub use star_context::ActorContext;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================
// (本 crate 为 supporting 层,无业务实体 — 实体由 domain-* crate 拥有)

// =====================================================================
// 端口(Port / 抽象)
// =====================================================================

/// **ApiGateway**(命令端口)
///
/// 来源: docs/api-design.md §3 全部 / §5 Event Subject / §8 错误码
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait ApiGateway: Send + Sync {
    async fn register_route(&self, cmd: (), actor: ActorContext) -> Result<(), ApiError>;
    async fn register_ws_handler(&self, cmd: (), actor: ActorContext) -> Result<(), ApiError>;
    async fn register_middleware(&self, cmd: (), actor: ActorContext) -> Result<(), ApiError>;
}

/// **ApiQuery**(查询端口)
///
/// 来源: docs/api-design.md §3 全部 / §5 Event Subject / §8 错误码
#[async_trait]
pub trait ApiQuery: Send + Sync {
    async fn list_routes(
        &self,
        _dummy: (),
        viewer: ActorContext,
    ) -> Result<Vec<RouteDescriptor>, ApiError>;
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
pub struct RouteDescriptor {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

// =====================================================================
// Error
// =====================================================================

/// **Api 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
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
            device_id: None, is_local_runtime: false, is_platform_admin: false,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
        };
        assert!(
            !actor.tenant_id.is_nil(),
            "tenant_id must be non-nil (§6.1,REQ-SEC-001)"
        );
    }
}

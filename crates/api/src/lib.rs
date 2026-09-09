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

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use uuid::Uuid;

// =====================================================================
// ARG.4 (P3-C W4): crates/api/src/arg — 13 REST + 1 WebSocket
// per docs/design/DD-AGENT-RELATIONSHIP-001.md v0.1.1 §4.12
// 守门 #14 v2 (5 域 Lead Mavis 临时代签) + #12 v21 ([M] docs 同步)
// =====================================================================
pub mod arg;

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
    /// 注册路由
    async fn register_route(&self, cmd: (), actor: ActorContext) -> Result<(), ApiError>;
    /// 注册 WebSocket 处理器
    async fn register_ws_handler(&self, cmd: (), actor: ActorContext) -> Result<(), ApiError>;
    /// 注册中间件
    async fn register_middleware(&self, cmd: (), actor: ActorContext) -> Result<(), ApiError>;
}

/// **ApiQuery**(查询端口)
///
/// 来源: docs/api-design.md §3 全部 / §5 Event Subject / §8 错误码
#[async_trait]
pub trait ApiQuery: Send + Sync {
    /// 列出路由
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
///
/// **v0 phase 2 标记** (per OPT-NEXT-06-code-stub §3.4 3 supporting crate 占位,
/// per `application/src/lib.rs:127-128` 注释 "Phase 2 由具体 spec 在 `domain-*` 内补全字段;
/// `crates/application` 等 supporting crate 的占位则在 Phase 2 删除, 改为 `use domain_xxx::*;` 引用"):
///
/// - `crates/application` 占位结构: 12 个 (AgentSession + CreateWorkItemFullCommand + Feedback
///   + RegisterRuntimeFullCommand + RegisterWorktreeFullCommand + Runtime
///   + StartAgentSessionFullCommand + SubmitFeedbackFullCommand + WorkItem
///   + WorkItemView + Worktree + WorkItemId type alias)
/// - `crates/api` 占位结构: ~12 个 (RouteDescriptor + ... 同模式)
/// - `crates/infrastructure` 占位结构: ~12 个 (StorageDescriptor + ... 同模式)
///
/// P2 阶段 worker 子代理实装时按 "use domain_xxx::*;" 引用替换 (派前必先
/// `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`, per AGENTS.md §4 #20 守门派生).
/// 当前保留为 P2 阶段前置可编译骨架 (守门 #1 v6 单 crate 100% pass 实证).

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
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`.
///
/// P0-2 (per WBS §14.15) 加 6 个 domain error → ApiError From impls:
/// - `domain_work_item::WorkItemError` (NotFound / PermissionDenied / InvalidState / Internal)
/// - `domain_workspace::WorkspaceError` (NotFound / PermissionDenied / InvalidState / Conflict / Internal)
/// - `domain_worktree::WorktreeError` (NotFound / PermissionDenied / InvalidState / Conflict / RuntimeRequired / Internal)
/// - `domain_search::SearchError` (NotFound / PermissionDenied / InvalidState / Conflict / Internal)
/// - `domain_scm::ScmError` (NotFound / PermissionDenied / InvalidState / Conflict / ProviderError / Internal)
/// - `domain_validation::ValidationError` (NotFound / PermissionDenied / InvalidState / Conflict / Internal)
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// 资源未找到
    #[error("not found: {0}")]
    NotFound(Uuid),
    /// 状态非法
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 权限拒绝
    #[error("permission denied")]
    PermissionDenied,
    /// 资源冲突
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// P0-2 ApiError 映射 (per WBS §14.15, H2 done 后 unblock 启动)
// 6 个 domain error → ApiError From impls
// =====================================================================

impl From<domain_work_item::WorkItemError> for ApiError {
    fn from(e: domain_work_item::WorkItemError) -> Self {
        use domain_work_item::WorkItemError::*;
        match e {
            NotFound(_) => ApiError::NotFound(Uuid::nil()),
            PermissionDenied => ApiError::PermissionDenied,
            CrossTenantDenied(_, _) => ApiError::PermissionDenied,
            InvalidTransition { .. } => ApiError::InvalidState(e.to_string()),
            AiTaskMissingObjective | AiTaskMissingScope | ParentProjectMismatch => {
                ApiError::InvalidState(e.to_string())
            }
            Conflict(_) => ApiError::Conflict(e.to_string()),
            Internal(_) => ApiError::Internal(e.to_string()),
        }
    }
}

impl From<domain_workspace::WorkspaceError> for ApiError {
    fn from(e: domain_workspace::WorkspaceError) -> Self {
        use domain_workspace::WorkspaceError::*;
        match e {
            NotFound(_) => ApiError::NotFound(Uuid::nil()),
            PermissionDenied => ApiError::PermissionDenied,
            InvalidState(_) => ApiError::InvalidState(e.to_string()),
            Conflict(_) => ApiError::Conflict(e.to_string()),
            Internal(_) => ApiError::Internal(e.to_string()),
        }
    }
}

impl From<domain_worktree::WorktreeError> for ApiError {
    fn from(e: domain_worktree::WorktreeError) -> Self {
        use domain_worktree::WorktreeError::*;
        match e {
            NotFound(_) => ApiError::NotFound(Uuid::nil()),
            PermissionDenied => ApiError::PermissionDenied,
            CrossTenantDenied(_, _) => ApiError::PermissionDenied,
            InvalidTransition { .. } => ApiError::InvalidState(e.to_string()),
            RuntimeRequired => ApiError::InvalidState(e.to_string()),
            Conflict(_) => ApiError::Conflict(e.to_string()),
            CompletionGateFailed(_) | IsolationFailed(_) => {
                ApiError::InvalidState(e.to_string())
            }
            Internal(_) => ApiError::Internal(e.to_string()),
        }
    }
}

impl From<domain_search::SearchError> for ApiError {
    fn from(e: domain_search::SearchError) -> Self {
        use domain_search::SearchError::*;
        match e {
            NotFound(_) => ApiError::NotFound(Uuid::nil()),
            PermissionDenied => ApiError::PermissionDenied,
            CrossTenantDenied(_, _) => ApiError::PermissionDenied,
            InvalidState(_) | InvalidQuery(_) => ApiError::InvalidState(e.to_string()),
            Conflict(_) => ApiError::Conflict(e.to_string()),
            Internal(_) => ApiError::Internal(e.to_string()),
        }
    }
}

impl From<domain_scm::ScmError> for ApiError {
    fn from(e: domain_scm::ScmError) -> Self {
        use domain_scm::ScmError::*;
        match e {
            NotFound(_) => ApiError::NotFound(Uuid::nil()),
            PermissionDenied(_) => ApiError::PermissionDenied,
            InvalidState(_) => ApiError::InvalidState(e.to_string()),
            Conflict(_) | IdempotencyConflict => ApiError::Conflict(e.to_string()),
            ProviderError(_) => ApiError::Internal(e.to_string()),
            Internal(_) => ApiError::Internal(e.to_string()),
        }
    }
}

impl From<domain_validation::ValidationError> for ApiError {
    fn from(e: domain_validation::ValidationError) -> Self {
        use domain_validation::ValidationError::*;
        match e {
            NotFound(_) => ApiError::NotFound(Uuid::nil()),
            PermissionDenied => ApiError::PermissionDenied,
            InvalidState(_) => ApiError::InvalidState(e.to_string()),
            Conflict(_) => ApiError::Conflict(e.to_string()),
            InvariantViolated(_) => ApiError::InvalidState(e.to_string()),
            Internal(_) => ApiError::Internal(e.to_string()),
        }
    }
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

    // -----------------------------------------------------------------
    // P0-2 ApiError 映射测试 (per WBS §14.15)
    // 6 个 From impl 各 1 个 positive 验证 (domain error → ApiError variant 映射)
    // -----------------------------------------------------------------

    #[test]
    fn work_item_error_not_found_maps_to_api_not_found() {
        use domain_work_item::WorkItemError;
        let e = WorkItemError::NotFound("work-item-uuid".to_string());
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::NotFound(_)), "expected NotFound");
    }

    #[test]
    fn work_item_error_permission_denied_maps_to_api_permission_denied() {
        use domain_work_item::WorkItemError;
        let e = WorkItemError::PermissionDenied;
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::PermissionDenied), "expected PermissionDenied");
    }

    #[test]
    fn workspace_error_conflict_maps_to_api_conflict() {
        use domain_workspace::WorkspaceError;
        let e = WorkspaceError::Conflict("dup key".to_string());
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::Conflict(_)), "expected Conflict");
    }

    #[test]
    fn worktree_error_runtime_required_maps_to_api_invalid_state() {
        use domain_worktree::WorktreeError;
        let e = WorktreeError::RuntimeRequired;
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::InvalidState(_)), "expected InvalidState");
    }

    #[test]
    fn search_error_invalid_query_maps_to_api_invalid_state() {
        use domain_search::SearchError;
        let e = SearchError::InvalidQuery("bad query".to_string());
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::InvalidState(_)), "expected InvalidState");
    }

    #[test]
    fn scm_error_provider_error_maps_to_api_internal() {
        use domain_scm::ScmError;
        let e = ScmError::ProviderError("upstream timeout".to_string());
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::Internal(_)), "expected Internal (provider error → internal)");
    }

    #[test]
    fn validation_error_invariant_violated_maps_to_api_invalid_state() {
        use domain_validation::ValidationError;
        let e = ValidationError::InvariantViolated("INV-VL-01 broken".to_string());
        let api: ApiError = e.into();
        assert!(matches!(api, ApiError::InvalidState(_)), "expected InvalidState");
    }
}

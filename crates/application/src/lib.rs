//! Application Services 编排层
//!
//! **crate**: `application`
//! **上游 spec**: docs/specs/application-spec.md §2.4 / §14.1 跨域事务
//! **基本设计**: docs/basic-design.md §1.2 / §2.4
//! **数据设计**: docs/data-design.md —
//! **API 设计**: docs/api-design.md —
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - 跨域事务由本 crate 编排,单 PG 事务(§2.4,§14.1)
//! - Outbox 触发事件(非事务组成,异步):AgentSessionCreated / WorktreeStatusObserved / ValidationFailed(§2.4)
//! - 本 crate 不持有 Entity,只编排 Domain Port 调用(§2.4)

//! ## 上游依赖(basic-design §2.3)
//!
//! 本 crate 依赖以下 domain-*(骨架阶段不实际 import,Cargo.toml 仅声明本 crate 自身需要的外部依赖):
//!
//!   - `domain-work-item`
//!   - `domain-worktree`
//!   - `domain-agent`
//!   - `domain-feedback`
//!   - `domain-tenant`
//!   - `domain-audit`
//!   - `domain-permission`
//!   - `domain-scm`
//!   - `domain-development`
//!   - `domain-validation`
//!   - `domain-local-runtime`
//!   - `domain-identity`
//!
//! **禁止反向依赖**(§2.3 禁线)。

//! ## 关键引用
//!
//! Application Service 跨域事务编排(§2.4);不通过 Event Chain 拆分(§14.1,§58)

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

/// **ApplicationService**(命令端口)
///
/// 来源: docs/api-design.md —
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait ApplicationService: Send + Sync {
    /// 创建工作项(跨域事务编排)
    async fn create_work_item_full(
        &self,
        cmd: CreateWorkItemFullCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, ApplicationError>;
    /// 注册 worktree(跨域事务编排)
    async fn register_worktree_full(
        &self,
        cmd: RegisterWorktreeFullCommand,
        actor: ActorContext,
    ) -> Result<Worktree, ApplicationError>;
    /// 启动 Agent 会话(跨域事务编排)
    async fn start_agent_session_full(
        &self,
        cmd: StartAgentSessionFullCommand,
        actor: ActorContext,
    ) -> Result<AgentSession, ApplicationError>;
    /// 提交反馈(跨域事务编排)
    async fn submit_feedback_full(
        &self,
        cmd: SubmitFeedbackFullCommand,
        actor: ActorContext,
    ) -> Result<Feedback, ApplicationError>;
    /// 注册运行时(跨域事务编排)
    async fn register_runtime_full(
        &self,
        cmd: RegisterRuntimeFullCommand,
        actor: ActorContext,
    ) -> Result<Runtime, ApplicationError>;
}

/// **ApplicationQueryService**(查询端口)
///
/// 来源: docs/api-design.md —
#[async_trait]
pub trait ApplicationQueryService: Send + Sync {
    /// 获取工作项视图(查询端口)
    async fn get_work_item_view(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<WorkItemView, ApplicationError>;
}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================
// (本 crate 不直接发布 Domain Event,事件由 domain-* crate 拥有)

// =====================================================================
// 类型别名与命令/查询/返回类型占位
// =====================================================================
/// **ID 类型别名**(Phase 1 骨架:均为 UUID 别名)
///
/// 真实使用应由 `domain-identity` 颁发强类型 ID(§23.2);
/// 骨架阶段以 `Uuid` 替代以避免跨 crate 编译依赖。

pub type WorkItemId = Uuid;

/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)
///
/// **v0 phase 2 标记** (per OPT-NEXT-06-code-stub §3.4 3 supporting crate 占位):
/// `crates/application` / `crates/api` / `crates/infrastructure` 共 3 supporting crate,
/// 12 + 12 + 12 ≈ 36 占位结构, P2 阶段 worker 子代理按 `use domain_xxx::*;` 引用替换.
///
/// 关联 issue: per `OPT-A1-code-todo-scan.output.md` §3.5 line 128 P1 重要
/// "占位结构 (Phase 1 骨架)" 模式, 3 supporting crate 统一 Phase 2 删除.
///
/// P2 阶段 worker 子代理实装时按 feature 名 (e.g. `application::AgentSession`)
/// 替换为真实引用, 派前必先 `automation/dispatcher.py brief(...)` 落
/// `docs/briefs/<task_id>.md` (per AGENTS.md §4 #20 守门派生).

/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等
/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 创建工作项(跨域事务)命令占位结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkItemFullCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 反馈占位实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 注册运行时命令占位结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRuntimeFullCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 注册 worktree 命令占位结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorktreeFullCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 运行时占位实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runtime {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 启动 Agent 会话命令占位结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartAgentSessionFullCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 提交反馈命令占位结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitFeedbackFullCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 工作项占位实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// 工作项视图占位结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemView {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

/// Worktree 占位实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

// =====================================================================
// Error
// =====================================================================

/// **Application 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    /// 未找到
    #[error("not found: {0}")]
    NotFound(Uuid),
    /// 非法状态
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,
    /// 冲突
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// P0-3 ApplicationError 映射 (per WBS §14.15, 跟 P0-2 ApiError 映射同模式)
// 6 个 domain error → ApplicationError From impls
// =====================================================================

impl From<domain_work_item::WorkItemError> for ApplicationError {
    fn from(e: domain_work_item::WorkItemError) -> Self {
        use domain_work_item::WorkItemError::*;
        match e {
            NotFound(_) => ApplicationError::NotFound(Uuid::nil()),
            PermissionDenied => ApplicationError::PermissionDenied,
            CrossTenantDenied(_, _) => ApplicationError::PermissionDenied,
            InvalidTransition { .. } => ApplicationError::InvalidState(e.to_string()),
            AiTaskMissingObjective | AiTaskMissingScope | ParentProjectMismatch => {
                ApplicationError::InvalidState(e.to_string())
            }
            Conflict(_) => ApplicationError::Conflict(e.to_string()),
            Internal(_) => ApplicationError::Internal(e.to_string()),
        }
    }
}

impl From<domain_workspace::WorkspaceError> for ApplicationError {
    fn from(e: domain_workspace::WorkspaceError) -> Self {
        use domain_workspace::WorkspaceError::*;
        match e {
            NotFound(_) => ApplicationError::NotFound(Uuid::nil()),
            PermissionDenied => ApplicationError::PermissionDenied,
            InvalidState(_) => ApplicationError::InvalidState(e.to_string()),
            Conflict(_) => ApplicationError::Conflict(e.to_string()),
            Internal(_) => ApplicationError::Internal(e.to_string()),
        }
    }
}

impl From<domain_worktree::WorktreeError> for ApplicationError {
    fn from(e: domain_worktree::WorktreeError) -> Self {
        use domain_worktree::WorktreeError::*;
        match e {
            NotFound(_) => ApplicationError::NotFound(Uuid::nil()),
            PermissionDenied => ApplicationError::PermissionDenied,
            CrossTenantDenied(_, _) => ApplicationError::PermissionDenied,
            InvalidTransition { .. } => ApplicationError::InvalidState(e.to_string()),
            RuntimeRequired => ApplicationError::InvalidState(e.to_string()),
            Conflict(_) => ApplicationError::Conflict(e.to_string()),
            CompletionGateFailed(_) | IsolationFailed(_) => {
                ApplicationError::InvalidState(e.to_string())
            }
            Internal(_) => ApplicationError::Internal(e.to_string()),
        }
    }
}

impl From<domain_search::SearchError> for ApplicationError {
    fn from(e: domain_search::SearchError) -> Self {
        use domain_search::SearchError::*;
        match e {
            NotFound(_) => ApplicationError::NotFound(Uuid::nil()),
            PermissionDenied => ApplicationError::PermissionDenied,
            CrossTenantDenied(_, _) => ApplicationError::PermissionDenied,
            InvalidState(_) | InvalidQuery(_) => ApplicationError::InvalidState(e.to_string()),
            Conflict(_) => ApplicationError::Conflict(e.to_string()),
            Internal(_) => ApplicationError::Internal(e.to_string()),
        }
    }
}

impl From<domain_scm::ScmError> for ApplicationError {
    fn from(e: domain_scm::ScmError) -> Self {
        use domain_scm::ScmError::*;
        match e {
            NotFound(_) => ApplicationError::NotFound(Uuid::nil()),
            PermissionDenied(_) => ApplicationError::PermissionDenied,
            InvalidState(_) => ApplicationError::InvalidState(e.to_string()),
            Conflict(_) | IdempotencyConflict => ApplicationError::Conflict(e.to_string()),
            ProviderError(_) => ApplicationError::Internal(e.to_string()),
            Internal(_) => ApplicationError::Internal(e.to_string()),
        }
    }
}

impl From<domain_validation::ValidationError> for ApplicationError {
    fn from(e: domain_validation::ValidationError) -> Self {
        use domain_validation::ValidationError::*;
        match e {
            NotFound(_) => ApplicationError::NotFound(Uuid::nil()),
            PermissionDenied => ApplicationError::PermissionDenied,
            InvalidState(_) => ApplicationError::InvalidState(e.to_string()),
            Conflict(_) => ApplicationError::Conflict(e.to_string()),
            InvariantViolated(_) => ApplicationError::InvalidState(e.to_string()),
            Internal(_) => ApplicationError::Internal(e.to_string()),
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
    // P0-3 ApplicationError 映射测试 (per WBS §14.15)
    // 6 个 From impl 各 1 个 positive 验证 (domain error → ApplicationError variant 映射)
    // -----------------------------------------------------------------

    #[test]
    fn work_item_error_not_found_maps_to_application_not_found() {
        use domain_work_item::WorkItemError;
        let e = WorkItemError::NotFound("work-item-uuid".to_string());
        let app_err: ApplicationError = e.into();
        assert!(matches!(app_err, ApplicationError::NotFound(_)), "expected NotFound");
    }

    #[test]
    fn workspace_error_permission_denied_maps_to_application_permission_denied() {
        use domain_workspace::WorkspaceError;
        let e = WorkspaceError::PermissionDenied;
        let app_err: ApplicationError = e.into();
        assert!(matches!(app_err, ApplicationError::PermissionDenied), "expected PermissionDenied");
    }

    #[test]
    fn worktree_error_invalid_transition_maps_to_application_invalid_state() {
        use domain_worktree::WorktreeError;
        let e = WorktreeError::InvalidTransition {
            from: "CREATED".to_string(),
            to: "ABANDONED".to_string(),
        };
        let app_err: ApplicationError = e.into();
        assert!(matches!(app_err, ApplicationError::InvalidState(_)), "expected InvalidState");
    }

    #[test]
    fn search_error_not_found_maps_to_application_not_found() {
        use domain_search::SearchError;
        let e = SearchError::NotFound("index-missing".to_string());
        let app_err: ApplicationError = e.into();
        assert!(matches!(app_err, ApplicationError::NotFound(_)), "expected NotFound");
    }

    #[test]
    fn scm_error_idempotency_conflict_maps_to_application_conflict() {
        use domain_scm::ScmError;
        let e = ScmError::IdempotencyConflict;
        let app_err: ApplicationError = e.into();
        assert!(matches!(app_err, ApplicationError::Conflict(_)), "expected Conflict");
    }

    #[test]
    fn validation_error_internal_maps_to_application_internal() {
        use domain_validation::ValidationError;
        let e = ValidationError::Internal("validator crashed".to_string());
        let app_err: ApplicationError = e.into();
        assert!(matches!(app_err, ApplicationError::Internal(_)), "expected Internal");
    }
}

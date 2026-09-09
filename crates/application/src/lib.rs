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
/// **Application 错误**(6 字段, per AGENTS.md §3 + star-mcp::error.rs 模式, 跟 ApiError 同源)
///
/// v0.29 改造: 从 5-variant enum 升级到 6-field struct (per self-review §3 "6-field ApiError / ApplicationError 改造")
/// - `code` (SCREAMING_SNAKE_CASE)
/// - `message` (human-readable)
/// - `source_module` (`"application"`)
/// - `source_kind` (`internal` / `external` / `policy` / `validation` / `user_input` / `timeout`)
/// - `retriable` (bool)
/// - `hint` (可执行修复提示)
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[error("{code}: {message} (source={source_module}/{source_kind}, retriable={retriable})")]
pub struct ApplicationError {
    /// 错误码,SCREAMING_SNAKE_CASE (e.g. `"NOT_FOUND"` / `"INVALID_STATE"`)
    pub code: String,
    /// 人类可读错误描述
    pub message: String,
    /// 错误来源模块,本 crate 固定 `"application"`
    pub source_module: String,
    /// 错误来源分类 (`internal` / `external` / `policy` / `validation` / `user_input` / `timeout`)
    pub source_kind: String,
    /// 是否可重试 (`true` = 客户端应重试, `false` = 不可重试)
    pub retriable: bool,
    /// 可执行修复提示 (面向调用方)
    pub hint: String,
}

impl ApplicationError {
    /// 通用构造器
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        source_module: impl Into<String>,
        source_kind: impl Into<String>,
        retriable: bool,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source_module: source_module.into(),
            source_kind: source_kind.into(),
            retriable,
            hint: hint.into(),
        }
    }

    /// 快捷: 资源未找到
    pub fn not_found(resource: &str) -> Self {
        Self::new(
            "RESOURCE_NOT_FOUND",
            format!("{resource} not found"),
            "application",
            "validation",
            false,
            "Provide a valid resource id",
        )
    }

    /// 快捷: 状态非法
    pub fn invalid_state(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::new(
            "VALIDATION_FAILED",
            message,
            "application",
            "validation",
            false,
            hint,
        )
    }

    /// 快捷: 权限拒绝
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(
            "POLICY_DENIED",
            message,
            "application",
            "policy",
            false,
            "Check role + tenant + permission scope",
        )
    }

    /// 快捷: 资源冲突
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(
            "CONFLICT",
            message,
            "application",
            "external",
            false,
            "Resolve conflict and retry",
        )
    }

    /// 快捷: 内部错误
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            "INTERNAL",
            message,
            "application",
            "internal",
            true,
            "Retry with backoff; if persistent, contact support",
        )
    }
}

// =====================================================================
// P0-3 ApplicationError 映射 (per WBS §14.15, 跟 P0-2 ApiError 映射同模式)
// 6 个 domain error → ApplicationError From impls
// =====================================================================

impl From<domain_work_item::WorkItemError> for ApplicationError {
    fn from(e: domain_work_item::WorkItemError) -> Self {
        use domain_work_item::WorkItemError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("work-item"),
            PermissionDenied => ApplicationError::permission_denied("work-item: permission denied"),
            CrossTenantDenied(_, _) => {
                ApplicationError::permission_denied("work-item: cross-tenant")
            }
            InvalidTransition { .. } => ApplicationError::invalid_state(
                e.to_string(),
                "Check work-item state machine transitions",
            ),
            AiTaskMissingObjective | AiTaskMissingScope | ParentProjectMismatch => {
                ApplicationError::invalid_state(e.to_string(), "Check work-item invariants")
            }
            Conflict(_) => ApplicationError::conflict(e.to_string()),
            Internal(_) => ApplicationError::internal(e.to_string()),
        }
    }
}

impl From<domain_workspace::WorkspaceError> for ApplicationError {
    fn from(e: domain_workspace::WorkspaceError) -> Self {
        use domain_workspace::WorkspaceError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("workspace"),
            PermissionDenied => ApplicationError::permission_denied("workspace: permission denied"),
            InvalidState(_) => {
                ApplicationError::invalid_state(e.to_string(), "Check workspace state")
            }
            Conflict(_) => ApplicationError::conflict(e.to_string()),
            Internal(_) => ApplicationError::internal(e.to_string()),
        }
    }
}

impl From<domain_worktree::WorktreeError> for ApplicationError {
    fn from(e: domain_worktree::WorktreeError) -> Self {
        use domain_worktree::WorktreeError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("worktree"),
            PermissionDenied => ApplicationError::permission_denied("worktree: permission denied"),
            CrossTenantDenied(_, _) => {
                ApplicationError::permission_denied("worktree: cross-tenant")
            }
            InvalidTransition { .. } => ApplicationError::invalid_state(
                e.to_string(),
                "Check worktree 17-state-machine transitions",
            ),
            RuntimeRequired => {
                ApplicationError::invalid_state(e.to_string(), "Provide runtime_id (INV-WT-03)")
            }
            Conflict(_) => ApplicationError::conflict(e.to_string()),
            CompletionGateFailed(_) | IsolationFailed(_) => {
                ApplicationError::invalid_state(e.to_string(), "Check worktree completion gate")
            }
            Internal(_) => ApplicationError::internal(e.to_string()),
        }
    }
}

impl From<domain_search::SearchError> for ApplicationError {
    fn from(e: domain_search::SearchError) -> Self {
        use domain_search::SearchError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("search-index"),
            PermissionDenied => ApplicationError::permission_denied("search: permission denied"),
            CrossTenantDenied(_, _) => ApplicationError::permission_denied("search: cross-tenant"),
            InvalidState(_) | InvalidQuery(_) => {
                ApplicationError::invalid_state(e.to_string(), "Check query syntax + filters")
            }
            Conflict(_) => ApplicationError::conflict(e.to_string()),
            Internal(_) => ApplicationError::internal(e.to_string()),
        }
    }
}

impl From<domain_scm::ScmError> for ApplicationError {
    fn from(e: domain_scm::ScmError) -> Self {
        use domain_scm::ScmError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("scm-repository"),
            PermissionDenied(_) => ApplicationError::permission_denied("scm: permission denied"),
            InvalidState(_) => {
                ApplicationError::invalid_state(e.to_string(), "Check scm resource state")
            }
            Conflict(_) | IdempotencyConflict => ApplicationError::conflict(e.to_string()),
            ProviderError(_) => ApplicationError::internal(format!("scm provider error: {e}")),
            Internal(_) => ApplicationError::internal(e.to_string()),
        }
    }
}

impl From<domain_validation::ValidationError> for ApplicationError {
    fn from(e: domain_validation::ValidationError) -> Self {
        use domain_validation::ValidationError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("validation"),
            PermissionDenied => {
                ApplicationError::permission_denied("validation: permission denied")
            }
            InvalidState(_) => {
                ApplicationError::invalid_state(e.to_string(), "Check validation state")
            }
            Conflict(_) => ApplicationError::conflict(e.to_string()),
            InvariantViolated(_) => {
                ApplicationError::invalid_state(e.to_string(), "Check validation invariants")
            }
            Internal(_) => ApplicationError::internal(e.to_string()),
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
        assert_eq!(app_err.code, "RESOURCE_NOT_FOUND");
        assert_eq!(app_err.source_module, "application");
        assert_eq!(app_err.source_kind, "validation");
    }

    #[test]
    fn workspace_error_permission_denied_maps_to_application_permission_denied() {
        use domain_workspace::WorkspaceError;
        let e = WorkspaceError::PermissionDenied;
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "POLICY_DENIED");
        assert_eq!(app_err.source_kind, "policy");
    }

    #[test]
    fn worktree_error_invalid_transition_maps_to_application_invalid_state() {
        use domain_worktree::WorktreeError;
        let e = WorktreeError::InvalidTransition {
            from: "CREATED".to_string(),
            to: "ABANDONED".to_string(),
        };
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "VALIDATION_FAILED");
        assert_eq!(app_err.source_kind, "validation");
    }

    #[test]
    fn search_error_not_found_maps_to_application_not_found() {
        use domain_search::SearchError;
        let e = SearchError::NotFound("index-missing".to_string());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "RESOURCE_NOT_FOUND");
        assert_eq!(app_err.source_kind, "validation");
    }

    #[test]
    fn scm_error_idempotency_conflict_maps_to_application_conflict() {
        use domain_scm::ScmError;
        let e = ScmError::IdempotencyConflict;
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "CONFLICT");
        assert_eq!(app_err.source_kind, "external");
    }

    #[test]
    fn validation_error_internal_maps_to_application_internal() {
        use domain_validation::ValidationError;
        let e = ValidationError::Internal("validator crashed".to_string());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "INTERNAL");
        assert_eq!(app_err.source_kind, "internal");
        assert!(app_err.retriable, "internal errors should be retriable");
    }

    /// v0.29 新增: 验证 6-field 结构完整 (code / message / source_module / source_kind / retriable / hint)
    #[test]
    fn application_error_6_field_structure_is_complete() {
        let e = ApplicationError::not_found("test-resource");
        assert!(!e.code.is_empty());
        assert!(!e.message.is_empty());
        assert!(!e.source_module.is_empty());
        assert!(!e.source_kind.is_empty());
        assert!(!e.hint.is_empty());
        let json = serde_json::to_string(&e).expect("serialize");
        assert!(json.contains("\"code\""));
        assert!(json.contains("\"message\""));
        assert!(json.contains("\"source_module\""));
        assert!(json.contains("\"source_kind\""));
        assert!(json.contains("\"retriable\""));
        assert!(json.contains("\"hint\""));
    }
}

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
/// - `source_module` (跟 P0-2 star-api-rest 6-field 对齐, per v0.65 v0.62 反转后修复:
///   caller 传入, e.g. `"domain-feedback"` / `"domain-validation"` / `"infrastructure"`)
/// - `source_kind` (跟 P0-2 + spec 对齐, TitleCase: `Validation` / `Policy` / `External` / `Internal`)
/// - `retriable` (bool)
/// - `hint` (可执行修复提示)
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[error("{code}: {message} (source={source_module}/{source_kind}, retriable={retriable})")]
pub struct ApplicationError {
    /// 错误码,SCREAMING_SNAKE_CASE (e.g. `"NOT_FOUND"` / `"INVALID_STATE"` / `"FB_NOT_FOUND"`)
    pub code: String,
    /// 人类可读错误描述
    pub message: String,
    /// 错误来源模块 (per v0.65 修复: caller 传入, e.g. `"domain-feedback"` / `"infrastructure"`)
    pub source_module: String,
    /// 错误来源分类 (TitleCase: `Validation` / `Policy` / `External` / `Internal`)
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

    /// 快捷: 资源未找到 (v0.65 修复 per self-review Finding 3+4, source_module 参数化 + TitleCase kind)
    pub fn not_found(source_module: impl Into<String>, resource: &str) -> Self {
        Self::new(
            "RESOURCE_NOT_FOUND",
            format!("{resource} not found"),
            source_module,
            "Validation",
            false,
            "Provide a valid resource id",
        )
    }

    /// 快捷: 状态非法 (v0.65 修复: source_module 参数化 + TitleCase)
    pub fn invalid_state(
        source_module: impl Into<String>,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(
            "VALIDATION_FAILED",
            message,
            source_module,
            "Validation",
            false,
            hint,
        )
    }

    /// 快捷: 权限拒绝 (v0.65 修复: source_module 参数化 + TitleCase)
    pub fn permission_denied(source_module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            "POLICY_DENIED",
            message,
            source_module,
            "Policy",
            false,
            "Check role + tenant + permission scope",
        )
    }

    /// 快捷: 资源冲突 (v0.65 修复: source_module 参数化 + TitleCase)
    pub fn conflict(source_module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            "CONFLICT",
            message,
            source_module,
            "External",
            false,
            "Resolve conflict and retry",
        )
    }

    /// 快捷: 内部错误 (v0.65 修复: source_module 参数化 + TitleCase)
    pub fn internal(source_module: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            "INTERNAL",
            message,
            source_module,
            "Internal",
            true,
            "Retry with backoff; if persistent, contact support",
        )
    }

    /// 快捷: 6-field 直接构造 (v0.65 修复, per self-review Finding 3+4: 跟 P0-2 star-api-rest 6-field 对齐)
    /// 适用 domain-specific code (e.g. "FB_NOT_FOUND" / "I_LOOP_GUARD_MISSING") + source_module
    pub fn new_with_kind(
        code: impl Into<String>,
        message: impl Into<String>,
        source_module: impl Into<String>,
        source_kind: impl Into<String>,
        retriable: bool,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(code, message, source_module, source_kind, retriable, hint)
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
            NotFound(_) => ApplicationError::not_found("domain-work-item", "work-item"),
            PermissionDenied => ApplicationError::permission_denied(
                "domain-work-item",
                "work-item: permission denied",
            ),
            CrossTenantDenied(_, _) => {
                ApplicationError::permission_denied("domain-work-item", "work-item: cross-tenant")
            }
            InvalidTransition { .. } => ApplicationError::invalid_state(
                "domain-work-item",
                e.to_string(),
                "Check work-item state machine transitions",
            ),
            AiTaskMissingObjective | AiTaskMissingScope | ParentProjectMismatch => {
                ApplicationError::invalid_state(
                    "domain-work-item",
                    e.to_string(),
                    "Check work-item invariants",
                )
            }
            Conflict(_) => ApplicationError::conflict("domain-work-item", e.to_string()),
            Internal(_) => ApplicationError::internal("domain-work-item", e.to_string()),
        }
    }
}

impl From<domain_workspace::WorkspaceError> for ApplicationError {
    fn from(e: domain_workspace::WorkspaceError) -> Self {
        use domain_workspace::WorkspaceError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("domain-workspace", "workspace"),
            PermissionDenied => ApplicationError::permission_denied(
                "domain-workspace",
                "workspace: permission denied",
            ),
            InvalidState(_) => ApplicationError::invalid_state(
                "domain-workspace",
                e.to_string(),
                "Check workspace state",
            ),
            Conflict(_) => ApplicationError::conflict("domain-workspace", e.to_string()),
            Internal(_) => ApplicationError::internal("domain-workspace", e.to_string()),
        }
    }
}

impl From<domain_worktree::WorktreeError> for ApplicationError {
    fn from(e: domain_worktree::WorktreeError) -> Self {
        use domain_worktree::WorktreeError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("domain-worktree", "worktree"),
            PermissionDenied => ApplicationError::permission_denied(
                "domain-worktree",
                "worktree: permission denied",
            ),
            CrossTenantDenied(_, _) => {
                ApplicationError::permission_denied("domain-worktree", "worktree: cross-tenant")
            }
            InvalidTransition { .. } => ApplicationError::invalid_state(
                "domain-worktree",
                e.to_string(),
                "Check worktree 17-state-machine transitions",
            ),
            RuntimeRequired => ApplicationError::invalid_state(
                "domain-worktree",
                e.to_string(),
                "Provide runtime_id (INV-WT-03)",
            ),
            Conflict(_) => ApplicationError::conflict("domain-worktree", e.to_string()),
            CompletionGateFailed(_) | IsolationFailed(_) => ApplicationError::invalid_state(
                "domain-worktree",
                e.to_string(),
                "Check worktree completion gate",
            ),
            Internal(_) => ApplicationError::internal("domain-worktree", e.to_string()),
        }
    }
}

impl From<domain_search::SearchError> for ApplicationError {
    fn from(e: domain_search::SearchError) -> Self {
        use domain_search::SearchError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("domain-search", "search-index"),
            PermissionDenied => {
                ApplicationError::permission_denied("domain-search", "search: permission denied")
            }
            CrossTenantDenied(_, _) => {
                ApplicationError::permission_denied("domain-search", "search: cross-tenant")
            }
            InvalidState(_) | InvalidQuery(_) => ApplicationError::invalid_state(
                "domain-search",
                e.to_string(),
                "Check query syntax + filters",
            ),
            Conflict(_) => ApplicationError::conflict("domain-search", e.to_string()),
            Internal(_) => ApplicationError::internal("domain-search", e.to_string()),
        }
    }
}

impl From<domain_scm::ScmError> for ApplicationError {
    fn from(e: domain_scm::ScmError) -> Self {
        use domain_scm::ScmError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("domain-scm", "scm-repository"),
            PermissionDenied(_) => {
                ApplicationError::permission_denied("domain-scm", "scm: permission denied")
            }
            InvalidState(_) => ApplicationError::invalid_state(
                "domain-scm",
                e.to_string(),
                "Check scm resource state",
            ),
            Conflict(_) | IdempotencyConflict => {
                ApplicationError::conflict("domain-scm", e.to_string())
            }
            ProviderError(_) => {
                ApplicationError::internal("domain-scm", format!("scm provider error: {e}"))
            }
            Internal(_) => ApplicationError::internal("domain-scm", e.to_string()),
        }
    }
}

impl From<domain_validation::ValidationError> for ApplicationError {
    fn from(e: domain_validation::ValidationError) -> Self {
        use domain_validation::ValidationError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("domain-validation", "validation"),
            PermissionDenied => ApplicationError::permission_denied(
                "domain-validation",
                "validation: permission denied",
            ),
            InvalidState(_) => ApplicationError::invalid_state(
                "domain-validation",
                e.to_string(),
                "Check validation state",
            ),
            Conflict(_) => ApplicationError::conflict("domain-validation", e.to_string()),
            InvariantViolated(_) => ApplicationError::invalid_state(
                "domain-validation",
                e.to_string(),
                "Check validation invariants",
            ),
            Internal(_) => ApplicationError::internal("domain-validation", e.to_string()),
        }
    }
}

// =====================================================================
// P0-4 infrastructure adapter 映射 (per WBS §14.15 P0-4 收官, 跟 P0-3 同模式)
// 1 个 InfrastructureError 5-variant → ApplicationError 6-field struct From impl
// =====================================================================

impl From<infrastructure::InfrastructureError> for ApplicationError {
    fn from(e: infrastructure::InfrastructureError) -> Self {
        use infrastructure::InfrastructureError::*;
        match e {
            NotFound(_) => ApplicationError::not_found("infrastructure", "infrastructure"),
            InvalidState(msg) => ApplicationError::invalid_state(
                "infrastructure",
                msg,
                "Check infrastructure adapter state",
            ),
            PermissionDenied => ApplicationError::permission_denied(
                "infrastructure",
                "infrastructure: permission denied",
            ),
            Conflict(msg) => ApplicationError::conflict(
                "infrastructure",
                format!("infrastructure conflict: {msg}"),
            ),
            Internal(msg) => {
                ApplicationError::internal("infrastructure", format!("infrastructure: {msg}"))
            }
        }
    }
}

// =====================================================================
// 单元测试占位
// =====================================================================

/// `domain_feedback::FeedbackError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_feedback::FeedbackError> for ApplicationError {
    fn from(e: domain_feedback::FeedbackError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_feedback::FeedbackError::NotFound(_) => ("FB_NOT_FOUND", "Validation", false),
            domain_feedback::FeedbackError::InvalidState(_) => {
                ("FB_INVALID_STATE_TRANSITION", "Validation", false)
            }
            domain_feedback::FeedbackError::TargetUnresolvable(_) => {
                ("FB_TARGET_UNRESOLVABLE", "Validation", false)
            }
            domain_feedback::FeedbackError::ReadOnly => ("FB_READ_ONLY", "Validation", false),
            domain_feedback::FeedbackError::NotDeletable => {
                ("FB_NOT_DELETABLE", "Validation", false)
            }
            domain_feedback::FeedbackError::MissingSuccessor => {
                ("FB_MISSING_SUCCESSOR", "Validation", false)
            }
            domain_feedback::FeedbackError::CrossWorktree => {
                ("FB_CROSS_WORKTREE", "Validation", false)
            }
            domain_feedback::FeedbackError::PermissionDenied => {
                ("FB_PERMISSION_DENIED", "Policy", false)
            }
            domain_feedback::FeedbackError::Conflict(_) => ("FB_CONFLICT", "External", false),
            domain_feedback::FeedbackError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("feedback: {e}"),
            source_module: "domain-feedback".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!(
                "Check the feedback id + tenant + role (developer/agent for AI feedback)"
            ),
        }
    }
}

/// `domain_integration::IntegrationError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_integration::IntegrationError> for ApplicationError {
    fn from(e: domain_integration::IntegrationError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_integration::IntegrationError::NotFound(_) => {
                ("RESOURCE_NOT_FOUND", "Validation", false)
            }
            domain_integration::IntegrationError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_integration::IntegrationError::PermissionDenied => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_integration::IntegrationError::Conflict(_) => {
                ("INTEGRATION_CONFLICT", "External", false)
            }
            domain_integration::IntegrationError::InvalidArgument(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_integration::IntegrationError::LoopGuardMissing(_) => {
                ("I_LOOP_GUARD_MISSING", "Validation", false)
            }
            domain_integration::IntegrationError::CredentialMissing(_) => {
                ("I_CREDENTIAL_MISSING", "Validation", false)
            }
            domain_integration::IntegrationError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("integration: {e}"),
            source_module: "domain-integration".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!(
                "Check the integration id + provider + tenant + role (project_admin/developer)"
            ),
        }
    }
}

/// `domain_comment::CommentError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_comment::CommentError> for ApplicationError {
    fn from(e: domain_comment::CommentError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_comment::CommentError::NotFound(_) => ("COMMENT_NOT_FOUND", "Validation", false),
            domain_comment::CommentError::InvalidState(_) => {
                ("VALIDATION_FAILED", "Validation", false)
            }
            domain_comment::CommentError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_comment::CommentError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_comment::CommentError::InvalidObjectKey => {
                ("CMT_INVALID_OBJECT_KEY", "Validation", false)
            }
            domain_comment::CommentError::ReactionExists => {
                ("CMT_REACTION_EXISTS", "External", false)
            }
            domain_comment::CommentError::EditDeleted => ("CMT_EDIT_DELETED", "Validation", false),
            domain_comment::CommentError::Conflict(_) => ("COMMENT_CONFLICT", "External", false),
            domain_comment::CommentError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("comment: {e}"),
            source_module: "domain-comment".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the comment id + thread id + tenant + role (developer/agent)"),
        }
    }
}

/// `domain_batch::BatchError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_batch::BatchError> for ApplicationError {
    fn from(e: domain_batch::BatchError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_batch::BatchError::Unauthenticated => ("BATCH_UNAUTHENTICATED", "Policy", false),
            domain_batch::BatchError::PermissionDenied(_) => {
                ("BATCH_PERMISSION_DENIED", "Policy", false)
            }
            domain_batch::BatchError::NodeTypeNotApproved(_) => {
                ("BATCH_NODE_TYPE_NOT_APPROVED", "Policy", false)
            }
            domain_batch::BatchError::TaskNotFound(_) => {
                ("BATCH_TASK_NOT_FOUND", "Validation", false)
            }
            domain_batch::BatchError::RunNotFound(_) => {
                ("BATCH_RUN_NOT_FOUND", "Validation", false)
            }
            domain_batch::BatchError::NodeNotFound(_) => {
                ("BATCH_NODE_NOT_FOUND", "Validation", false)
            }
            domain_batch::BatchError::NodeTypeNotFound(_) => {
                ("BATCH_NODE_TYPE_NOT_FOUND", "Validation", false)
            }
            domain_batch::BatchError::TaskNameConflict(_) => {
                ("BATCH_TASK_NAME_CONFLICT", "External", false)
            }
            domain_batch::BatchError::RunAlreadyRunning(_) => {
                ("BATCH_RUN_ALREADY_RUNNING", "External", false)
            }
            domain_batch::BatchError::NodeTimeout(_) => ("BATCH_NODE_TIMEOUT", "External", true),
            domain_batch::BatchError::InvalidDagSchema(_) => {
                ("BATCH_INVALID_DAG_SCHEMA", "Validation", false)
            }
            domain_batch::BatchError::DagCycle(_) => ("BATCH_DAG_CYCLE", "Validation", false),
            domain_batch::BatchError::InvalidNodeTypeConfig(_) => {
                ("BATCH_INVALID_NODE_TYPE_CONFIG", "Validation", false)
            }
            domain_batch::BatchError::InvalidCron(_) => ("BATCH_INVALID_CRON", "Validation", false),
            domain_batch::BatchError::ValidationFailed(_) => {
                ("BATCH_VALIDATION_FAILED", "Validation", false)
            }
            domain_batch::BatchError::NodeExecutionFailed(_) => {
                ("BATCH_NODE_EXECUTION_FAILED", "Internal", true)
            }
            domain_batch::BatchError::WorkerLeaseLost(_) => {
                ("BATCH_WORKER_LEASE_LOST", "Internal", true)
            }
            domain_batch::BatchError::Database(_) => ("BATCH_DATABASE_ERROR", "Internal", true),
            domain_batch::BatchError::EngineOverloaded => {
                ("BATCH_ENGINE_OVERLOADED", "Internal", true)
            }
            domain_batch::BatchError::NotImplemented { .. } => {
                ("BATCH_NOT_IMPLEMENTED", "Validation", false)
            }
            domain_batch::BatchError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("batch: {e}"),
            source_module: "domain-batch".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!(
                "Check the task/run/node id + tenant + role (developer); BATCH-REQ-001 §8"
            ),
        }
    }
}

/// `domain_theme::ThemeError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_theme::ThemeError> for ApplicationError {
    fn from(e: domain_theme::ThemeError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_theme::ThemeError::NotFound(_) => ("THEME_NOT_FOUND", "Validation", false),
            domain_theme::ThemeError::DuplicateId { .. } => {
                ("THEME_DUPLICATE_ID", "External", false)
            }
            domain_theme::ThemeError::IncompleteDefinition(_) => {
                ("THEME_INCOMPLETE_DEFINITION", "Validation", false)
            }
            domain_theme::ThemeError::InvalidHex(_) => ("THEME_INVALID_HEX", "Validation", false),
            domain_theme::ThemeError::InvalidSpacing(_) => {
                ("THEME_INVALID_SPACING", "Validation", false)
            }
            domain_theme::ThemeError::PermissionDenied { .. } => {
                ("THEME_PERMISSION_DENIED", "Policy", false)
            }
            domain_theme::ThemeError::Storage(_) => ("INTERNAL", "Internal", true),
            domain_theme::ThemeError::Serialization(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("theme: {e}"),
            source_module: "domain-theme".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the theme id + scope + tenant + role (developer/admin)"),
        }
    }
}

/// `domain_agent::AgentError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_agent::AgentError> for ApplicationError {
    fn from(e: domain_agent::AgentError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_agent::AgentError::NotFound(_) => ("AGENT_NOT_FOUND", "Validation", false),
            domain_agent::AgentError::InvalidTransition { .. } => {
                ("AGENT_INVALID_TRANSITION", "Validation", false)
            }
            domain_agent::AgentError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_agent::AgentError::CrossTenantDenied(_, _) => ("POLICY_DENIED", "Policy", false),
            domain_agent::AgentError::PolicyViolation(_) => {
                ("AGENT_POLICY_VIOLATION", "Policy", false)
            }
            domain_agent::AgentError::AgentAlreadyExists(_) => {
                ("AGENT_ALREADY_EXISTS", "External", false)
            }
            domain_agent::AgentError::WorktreeMismatch => {
                ("AGENT_WORKTREE_MISMATCH", "Validation", false)
            }
            domain_agent::AgentError::Conflict(_) => ("AGENT_CONFLICT", "External", false),
            domain_agent::AgentError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("agent: {e}"),
            source_module: "domain-agent".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the agent id + tenant + role (developer/agent operator)"),
        }
    }
}

/// `domain_cli::hermes::HermesError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_cli::hermes::HermesError> for ApplicationError {
    fn from(e: domain_cli::hermes::HermesError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_cli::hermes::HermesError::Http(_) => ("HERMES_HTTP_ERROR", "External", true),
            domain_cli::hermes::HermesError::Auth(_) => ("HERMES_AUTH_ERROR", "Policy", false),
            domain_cli::hermes::HermesError::ServerError(_, _) => {
                ("HERMES_SERVER_ERROR", "External", true)
            }
            domain_cli::hermes::HermesError::Parse(_) => ("HERMES_PARSE_ERROR", "External", false),
        };
        Self {
            code: code.to_string(),
            message: format!("hermes: {e}"),
            source_module: "domain-cli".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check Hermes API base_url + api_key + network (HTTP/Auth/ServerError/Parse per B.2 4 variants)"),
        }
    }
}

/// `domain_tenant::TenantError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_tenant::TenantError> for ApplicationError {
    fn from(e: domain_tenant::TenantError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_tenant::TenantError::NotFound(_) => ("TENANT_NOT_FOUND", "Validation", false),
            domain_tenant::TenantError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_tenant::TenantError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_tenant::TenantError::SlugExists(_) => ("TENANT_SLUG_EXISTS", "External", false),
            domain_tenant::TenantError::InvalidState(_) => {
                ("TENANT_INVALID_STATE", "Validation", false)
            }
            domain_tenant::TenantError::Conflict(_) => ("TENANT_CONFLICT", "External", false),
            domain_tenant::TenantError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("tenant: {e}"),
            source_module: "domain-tenant".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the tenant id + slug + role (tenant_admin/platform_admin)"),
        }
    }
}

/// `domain_identity::IdentityError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_identity::IdentityError> for ApplicationError {
    fn from(e: domain_identity::IdentityError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_identity::IdentityError::NotFound(_) => {
                ("IDENTITY_NOT_FOUND", "Validation", false)
            }
            domain_identity::IdentityError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_identity::IdentityError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_identity::IdentityError::EmailExists(_) => {
                ("IDENTITY_EMAIL_EXISTS", "External", false)
            }
            domain_identity::IdentityError::IncompleteBinding => {
                ("IDENTITY_INCOMPLETE_BINDING", "Validation", false)
            }
            domain_identity::IdentityError::DeviceAlreadyRevoked => {
                ("IDENTITY_DEVICE_REVOKED", "External", false)
            }
            domain_identity::IdentityError::Conflict(_) => ("IDENTITY_CONFLICT", "External", false),
            domain_identity::IdentityError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("identity: {e}"),
            source_module: "domain-identity".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!("Check the user id + email + tenant + role (developer/platform_admin)"),
        }
    }
}

/// `domain_permission::PermissionError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_permission::PermissionError> for ApplicationError {
    fn from(e: domain_permission::PermissionError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_permission::PermissionError::NotFound(_) => {
                ("PERMISSION_NOT_FOUND", "Validation", false)
            }
            domain_permission::PermissionError::PermissionDenied => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_permission::PermissionError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_permission::PermissionError::InvalidRule(_) => {
                ("PERMISSION_INVALID_RULE", "Validation", false)
            }
            domain_permission::PermissionError::Conflict(_) => {
                ("PERMISSION_CONFLICT", "External", false)
            }
            domain_permission::PermissionError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("permission: {e}"),
            source_module: "domain-permission".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!(
                "Check the permission rule + role + scope (project_admin/tenant_admin/developer)"
            ),
        }
    }
}

/// `domain_project::ProjectError` → `ApplicationError` (per v0.63 P0-3 Stage 1, WBS §14.15)
impl From<domain_project::ProjectError> for ApplicationError {
    fn from(e: domain_project::ProjectError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_project::ProjectError::NotFound(_) => ("PROJECT_NOT_FOUND", "Validation", false),
            domain_project::ProjectError::PermissionDenied => ("POLICY_DENIED", "Policy", false),
            domain_project::ProjectError::CrossTenantDenied(_, _) => {
                ("POLICY_DENIED", "Policy", false)
            }
            domain_project::ProjectError::SlugExists(_) => {
                ("PROJECT_SLUG_EXISTS", "External", false)
            }
            domain_project::ProjectError::InvalidState(_) => {
                ("PROJECT_INVALID_STATE", "Validation", false)
            }
            domain_project::ProjectError::Conflict(_) => ("PROJECT_CONFLICT", "External", false),
            domain_project::ProjectError::Internal(_) => ("INTERNAL", "Internal", true),
        };
        Self {
            code: code.to_string(),
            message: format!("project: {e}"),
            source_module: "domain-project".to_string(),
            source_kind: source_kind.to_string(),
            retriable,
            hint: format!(
                "Check the project id + slug + workspace + tenant + role (project_admin/developer)"
            ),
        }
    }
}

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
        assert_eq!(app_err.source_module, "domain-work-item");
        assert_eq!(app_err.source_kind, "Validation");
    }

    #[test]
    fn workspace_error_permission_denied_maps_to_application_permission_denied() {
        use domain_workspace::WorkspaceError;
        let e = WorkspaceError::PermissionDenied;
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "POLICY_DENIED");
        assert_eq!(app_err.source_kind, "Policy");
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
        assert_eq!(app_err.source_kind, "Validation");
    }

    #[test]
    fn search_error_not_found_maps_to_application_not_found() {
        use domain_search::SearchError;
        let e = SearchError::NotFound("index-missing".to_string());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "RESOURCE_NOT_FOUND");
        assert_eq!(app_err.source_kind, "Validation");
    }

    #[test]
    fn scm_error_idempotency_conflict_maps_to_application_conflict() {
        use domain_scm::ScmError;
        let e = ScmError::IdempotencyConflict;
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "CONFLICT");
        assert_eq!(app_err.source_kind, "External");
    }

    #[test]
    fn validation_error_internal_maps_to_application_internal() {
        use domain_validation::ValidationError;
        let e = ValidationError::Internal("validator crashed".to_string());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "INTERNAL");
        assert_eq!(app_err.source_kind, "Internal");
        assert!(app_err.retriable, "internal errors should be retriable");
    }

    /// v0.29 新增: 验证 6-field 结构完整 (code / message / source_module / source_kind / retriable / hint)
    #[test]
    fn application_error_6_field_structure_is_complete() {
        let e = ApplicationError::not_found("application", "test-resource");
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

    // -----------------------------------------------------------------
    // P0-4 infrastructure adapter 错误映射测试 (per WBS §14.15 P0-4 收官)
    // 1 个 From impl 5-variant → ApplicationError 6-field struct 验证
    // -----------------------------------------------------------------

    #[test]
    fn infrastructure_error_not_found_maps_to_application_not_found() {
        use infrastructure::InfrastructureError;
        let e = InfrastructureError::NotFound(uuid::Uuid::new_v4());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "RESOURCE_NOT_FOUND");
        assert_eq!(app_err.source_module, "infrastructure");
        assert_eq!(app_err.source_kind, "Validation");
        assert!(!app_err.retriable);
    }

    #[test]
    fn infrastructure_error_conflict_maps_to_application_conflict() {
        use infrastructure::InfrastructureError;
        let e = InfrastructureError::Conflict("dup key".to_string());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "CONFLICT");
        assert_eq!(app_err.source_kind, "External");
    }

    #[test]
    fn infrastructure_error_internal_maps_to_application_internal() {
        use infrastructure::InfrastructureError;
        let e = InfrastructureError::Internal("connection lost".to_string());
        let app_err: ApplicationError = e.into();
        assert_eq!(app_err.code, "INTERNAL");
        assert_eq!(app_err.source_kind, "Internal");
        assert!(app_err.retriable, "internal errors should be retriable");
    }
}

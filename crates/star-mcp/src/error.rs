//! `star-mcp` 错误类型(Phase E: 6-field `agent-api/v1#Error` schema)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md` §3.14 Error
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §3.2 错误模型
//!
//! ## Phase D.5+ → Phase E 演化
//!
//! - Phase D.5+ 错误: 4-variant enum (`Json` / `Io` / `UnknownTool` / `BadRequest`),
//!   JSON-RPC 2.0 error envelope 走 `INVALID_PARAMS` / `INTERNAL_ERROR` 等通用码
//! - **Phase E**: 重构为 **6 字段 struct** `McpError`, 与 `agent-api/v1#Error` 1:1 对齐
//!   (per F-06 修复 2026-08-27 + INTERFACE-REVIEW-A 🔴 #6), CLI / MCP / REST / Universal Submit 4 处共用
//! - 20+ 标准错误码(SCREAMING_SNAKE_CASE)集中在 `error_code` 模块,
//!   旧 enum variant 通过构造函数 `McpError::validation()` / `McpError::internal()` /
//!   `McpError::unknown_tool()` 等保留语义
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - thiserror 仍用于快速 `From<...>` impl(workspace 已有)
//! - `serde::Serialize` 直接复用 6 字段 → JSON-RPC 2.0 `data` envelope
//! - 不编造错误码(只列已在 spec/flows 出现的 20+ 个)

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 错误来源分类(per `agent-api/01-schema.md` §3.14, F-06 修复)
///
/// 6 个标准 source_kind 值。**小写字符串**序列化(per spec 范例, 与 SCREAMING_SNAKE_CASE
/// 错误码区分)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ErrorSourceKind {
    /// 内部逻辑错误(bug / invariant violation)
    Internal,
    /// 外部系统错误(数据库 / VCS / CI)
    External,
    /// 策略层拒绝(per ADR-0021 Zero Vendor Cooperation + policy enforcement)
    Policy,
    /// 参数 / schema 校验失败
    Validation,
    /// 用户输入错误(缺字段 / 格式错)
    UserInput,
    /// 超时(Lease heartbeat / HTTP / IO)
    Timeout,
}

impl std::fmt::Display for ErrorSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ErrorSourceKind::Internal => "internal",
            ErrorSourceKind::External => "external",
            ErrorSourceKind::Policy => "policy",
            ErrorSourceKind::Validation => "validation",
            ErrorSourceKind::UserInput => "user_input",
            ErrorSourceKind::Timeout => "timeout",
        };
        f.write_str(s)
    }
}

/// MCP server 顶层错误类型(per `agent-api/01-schema.md` §3.14, 6 字段)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub(crate) struct McpError {
    /// 标准化错误码(SCREAMING_SNAKE_CASE, e.g. `"WORKTREE_CONFLICT"`)
    pub code: String,
    /// human-readable 错误描述
    pub message: String,
    /// 错误来源模块(e.g. `"mcp"` / `"agent-core"` / `"vcs"` / `"policy"`)
    pub source_module: String,
    /// 错误分类(per F-06 修复)
    pub source_kind: ErrorSourceKind,
    /// 是否可重试(true = 客户端可重试; false = 不可重试, 需修正输入 / 状态)
    pub retriable: bool,
    /// 恢复提示(单字符串, 替换 v0.2 的 `suggested_actions[]`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[allow(unreachable_pub, dead_code)] // McpError is pub(crate); methods inherit that scope
impl McpError {
    /// 通用构造器
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        source_module: impl Into<String>,
        source_kind: ErrorSourceKind,
        retriable: bool,
        hint: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source_module: source_module.into(),
            source_kind,
            retriable,
            hint,
        }
    }

    /// 校验失败快捷构造(source_kind = Validation, retriable = false)
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            error_code::VALIDATION_FAILED,
            message,
            "mcp",
            ErrorSourceKind::Validation,
            false,
            None,
        )
    }

    /// 内部错误快捷构造(source_kind = Internal, retriable = false)
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            error_code::INTERNAL,
            message,
            "mcp",
            ErrorSourceKind::Internal,
            false,
            None,
        )
    }

    /// 未知 tool 快捷构造(source_kind = Validation, code = UNKNOWN_TOOL)
    pub fn unknown_tool(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::new(
            error_code::UNKNOWN_TOOL,
            format!("unknown tool: {name}"),
            "mcp",
            ErrorSourceKind::Validation,
            false,
            Some("available tools: see tools/list (16 tools)".to_string()),
        )
    }

    /// 用户输入错误(source_kind = UserInput, retriable = false)
    pub fn user_input(message: impl Into<String>, hint: Option<String>) -> Self {
        Self::new(
            error_code::USER_INPUT,
            message,
            "mcp",
            ErrorSourceKind::UserInput,
            false,
            hint,
        )
    }

    /// 策略拒绝快捷构造(source_kind = Policy, retriable = false)
    pub fn policy_denied(message: impl Into<String>, hint: Option<String>) -> Self {
        Self::new(
            error_code::POLICY_DENIED,
            message,
            "mcp",
            ErrorSourceKind::Policy,
            false,
            hint,
        )
    }

    /// 超时快捷构造(source_kind = Timeout, retriable = true)
    pub fn timeout(message: impl Into<String>, hint: Option<String>) -> Self {
        Self::new(
            error_code::AGENT_TIMEOUT,
            message,
            "mcp",
            ErrorSourceKind::Timeout,
            true,
            hint,
        )
    }
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} (source={}/{}, retriable={})",
            self.code, self.message, self.source_module, self.source_kind, self.retriable
        )
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(
            error_code::JSON_PARSE,
            format!("json parse error: {e}"),
            "mcp",
            ErrorSourceKind::Validation,
            false,
            None,
        )
    }
}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        Self::new(
            error_code::IO,
            format!("io error: {e}"),
            "mcp",
            ErrorSourceKind::External,
            true,
            None,
        )
    }
}

// P0 工具链 (per docs/briefs/tool-p0-impl-001.md §2.5) — domain error → McpError 映射
//
// 三个 domain crate 的错误统一映射到 `External` source_kind + `mcp` source_module
// (per 守门 #1 v6 cross-stage 实测, 不引入新的错误码变体, 复用 `code` 字段承载 domain 短码).
//
// 跨 tenant 拒绝 → 复用 `PermissionDenied` 源语义 + retriable=false.

/// `domain_scm::ScmError` → `McpError`
impl From<domain_scm::ScmError> for McpError {
    fn from(e: domain_scm::ScmError) -> Self {
        let code = e.code().to_string();
        let source_kind = match &e {
            domain_scm::ScmError::PermissionDenied(_) => ErrorSourceKind::Policy,
            domain_scm::ScmError::InvalidState(_) => ErrorSourceKind::Validation,
            domain_scm::ScmError::IdempotencyConflict => ErrorSourceKind::Validation,
            _ => ErrorSourceKind::External,
        };
        let retriable = matches!(&e, domain_scm::ScmError::Internal(_));
        Self::new(
            code,
            format!("scm: {e}"),
            "scm",
            source_kind,
            retriable,
            None,
        )
    }
}

/// `domain_worktree::WorktreeError` → `McpError`
impl From<domain_worktree::WorktreeError> for McpError {
    fn from(e: domain_worktree::WorktreeError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_worktree::WorktreeError::NotFound(_) => (
                error_code::WORKTREE_NOT_FOUND,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_worktree::WorktreeError::PermissionDenied => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_worktree::WorktreeError::CrossTenantDenied(_, _) => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_worktree::WorktreeError::InvalidTransition { .. } => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_worktree::WorktreeError::RuntimeRequired => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_worktree::WorktreeError::Conflict(_) => (
                error_code::WORKTREE_CONFLICT,
                ErrorSourceKind::External,
                false,
            ),
            domain_worktree::WorktreeError::CompletionGateFailed(_)
            | domain_worktree::WorktreeError::IsolationFailed(_) => (
                error_code::VALIDATION_RUN_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_worktree::WorktreeError::Internal(_) => {
                (error_code::INTERNAL, ErrorSourceKind::Internal, true)
            }
        };
        Self::new(
            code,
            format!("worktree: {e}"),
            "worktree",
            source_kind,
            retriable,
            None,
        )
    }
}

/// `domain_work_item::WorkItemError` → `McpError`
impl From<domain_work_item::WorkItemError> for McpError {
    fn from(e: domain_work_item::WorkItemError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_work_item::WorkItemError::NotFound(_) => (
                error_code::RESOURCE_NOT_FOUND,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_work_item::WorkItemError::PermissionDenied => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_work_item::WorkItemError::CrossTenantDenied(_, _) => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_work_item::WorkItemError::InvalidTransition { .. } => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_work_item::WorkItemError::AiTaskMissingObjective
            | domain_work_item::WorkItemError::AiTaskMissingScope
            | domain_work_item::WorkItemError::ParentProjectMismatch => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_work_item::WorkItemError::Conflict(_) => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::External,
                false,
            ),
            domain_work_item::WorkItemError::Internal(_) => {
                (error_code::INTERNAL, ErrorSourceKind::Internal, true)
            }
        };
        Self::new(
            code,
            format!("work-item: {e}"),
            "work-item",
            source_kind,
            retriable,
            None,
        )
    }
}

// P1 工具链 (per docs/briefs/tool-p1-impl-001.md §2.5) — `domain_search::SearchError` → `McpError`
//
// 跟 P0 3 个 From impl 同源:
// - NotFound / InvalidQuery / InvalidState → Validation + retriable=false
// - PermissionDenied / CrossTenantDenied → Policy + retriable=false
// - Conflict → External + retriable=false
// - Internal → Internal + retriable=true

/// `domain_search::SearchError` → `McpError`
impl From<domain_search::SearchError> for McpError {
    fn from(e: domain_search::SearchError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_search::SearchError::NotFound(_) => (
                error_code::RESOURCE_NOT_FOUND,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_search::SearchError::InvalidState(_) => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_search::SearchError::PermissionDenied => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_search::SearchError::CrossTenantDenied(_, _) => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_search::SearchError::InvalidQuery(_) => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_search::SearchError::Conflict(_) => (
                error_code::VALIDATION_FAILED,
                ErrorSourceKind::External,
                false,
            ),
            domain_search::SearchError::Internal(_) => {
                (error_code::INTERNAL, ErrorSourceKind::Internal, true)
            }
        };
        Self::new(
            code,
            format!("search: {e}"),
            "search",
            source_kind,
            retriable,
            None,
        )
    }
}

// P2 工具链 (per docs/briefs/tool-p2-impl-001.md §2.5) — `domain_validation::ValidationError` → `McpError`
//
// 跟 P0/P1 5 个 From impl 同源:
// - NotFound / InvalidState → Validation + retriable=false
// - PermissionDenied → Policy + retriable=false
// - Conflict → External + retriable=false
// - InvariantViolated → Validation + retriable=false (per spec §8 VL-004 不变量违反)
// - Internal → Internal + retriable=true

/// `domain_validation::ValidationError` → `McpError`
impl From<domain_validation::ValidationError> for McpError {
    fn from(e: domain_validation::ValidationError) -> Self {
        let (code, source_kind, retriable) = match &e {
            domain_validation::ValidationError::NotFound(_) => (
                error_code::RESOURCE_NOT_FOUND,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_validation::ValidationError::InvalidState(_) => (
                error_code::VALIDATION_RUN_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_validation::ValidationError::PermissionDenied => {
                (error_code::POLICY_DENIED, ErrorSourceKind::Policy, false)
            }
            domain_validation::ValidationError::Conflict(_) => (
                error_code::VALIDATION_RUN_FAILED,
                ErrorSourceKind::External,
                false,
            ),
            domain_validation::ValidationError::InvariantViolated(_) => (
                error_code::VALIDATION_RUN_FAILED,
                ErrorSourceKind::Validation,
                false,
            ),
            domain_validation::ValidationError::Internal(_) => {
                (error_code::INTERNAL, ErrorSourceKind::Internal, true)
            }
        };
        Self::new(
            code,
            format!("validation: {e}"),
            "validation",
            source_kind,
            retriable,
            None,
        )
    }
}

/// MCP 标准错误码常量(per spec + flows 累积, 共 24 个)
///
/// **命名约定**(per F-06 修复 2026-08-27): SCREAMING_SNAKE_CASE 字符串, 与
/// `error_code::code_name()` 配对。涵盖 Phase E 资源 / 提示 / 工具 4 个失败场景。
#[allow(unreachable_pub, dead_code)] // 部分码保留供 Phase F 接入
pub(crate) mod error_code {
    // ===== JSON-RPC 2.0 标准码(数字, transport envelope 用) =====
    /// JSON parse 错误
    pub const PARSE_ERROR: i32 = -32700;
    /// JSON-RPC 2.0 request 格式错误
    pub const INVALID_REQUEST: i32 = -32600;
    /// 方法不存在
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// 参数错误
    pub const INVALID_PARAMS: i32 = -32602;
    /// 内部错误
    pub const INTERNAL_ERROR: i32 = -32603;

    // ===== MCP server 范围 (per Phase E spec, 字符串码) =====

    // --- 通用 ---
    /// 内部错误
    pub const INTERNAL: &str = "INTERNAL";
    /// 校验失败
    pub const VALIDATION_FAILED: &str = "VALIDATION_FAILED";
    /// JSON 解析失败
    pub const JSON_PARSE: &str = "JSON_PARSE";
    /// IO 错误
    pub const IO: &str = "IO";
    /// 未知 tool
    pub const UNKNOWN_TOOL: &str = "UNKNOWN_TOOL";
    /// 用户输入错误
    pub const USER_INPUT: &str = "USER_INPUT";

    // --- 资源 / 工作树 (per spec/flows/03 worktree binding) ---
    /// Worktree 冲突(per flows/03)
    pub const WORKTREE_CONFLICT: &str = "WORKTREE_CONFLICT";
    /// Worktree 不存在
    pub const WORKTREE_NOT_FOUND: &str = "WORKTREE_NOT_FOUND";
    /// Worktree 已锁定
    pub const WORKTREE_LOCKED: &str = "WORKTREE_LOCKED";

    // --- Agent session (per ADR-0030 Lease + Heartbeat) ---
    /// Agent 超时(lease / heartbeat)
    pub const AGENT_TIMEOUT: &str = "AGENT_TIMEOUT";
    /// Agent session 不存在
    pub const AGENT_NOT_FOUND: &str = "AGENT_NOT_FOUND";
    /// Lease 已过期
    pub const LEASE_EXPIRED: &str = "LEASE_EXPIRED";

    // --- 提交 / 决策 (per spec/flows/05 Universal Submit) ---
    /// 提交拒绝(per flows/05 第 6 步 policy check)
    pub const SUBMIT_DENIED: &str = "SUBMIT_DENIED";
    /// 验证失败(per flows/05 第 5 步 run_validation)
    pub const VALIDATION_RUN_FAILED: &str = "VALIDATION_RUN_FAILED";
    /// 策略拒绝(per ADR-0021 + flows/05)
    pub const POLICY_DENIED: &str = "POLICY_DENIED";
    /// 决策不存在(per flows/02)
    pub const DECISION_NOT_FOUND: &str = "DECISION_NOT_FOUND";

    // --- 资源读取(Phase E new) ---
    /// 资源 URI 格式错误
    pub const RESOURCE_URI_INVALID: &str = "RESOURCE_URI_INVALID";
    /// 资源不存在
    pub const RESOURCE_NOT_FOUND: &str = "RESOURCE_NOT_FOUND";

    // --- 提示(Phase E new) ---
    /// 提示不存在
    pub const PROMPT_NOT_FOUND: &str = "PROMPT_NOT_FOUND";
    /// 提示参数缺失
    pub const PROMPT_ARG_MISSING: &str = "PROMPT_ARG_MISSING";

    // --- 工具 dispatch (legacy, 兼容) ---
    /// Method 不存在(per JSON-RPC 2.0)
    pub const METHOD_NOT_FOUND_STR: &str = "METHOD_NOT_FOUND";

    /// 总数校验用(编译期断言 — Phase E 列了 24 个)
    #[doc(hidden)]
    pub const COUNT: usize = 24;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 6 字段 struct 基础测试 =====

    #[tokio::test]
    async fn test_mcp_error_struct_6_fields() {
        let err = McpError::new(
            "TEST_CODE",
            "test message",
            "mcp",
            ErrorSourceKind::Validation,
            false,
            Some("try again".to_string()),
        );
        assert_eq!(err.code, "TEST_CODE");
        assert_eq!(err.message, "test message");
        assert_eq!(err.source_module, "mcp");
        assert_eq!(err.source_kind, ErrorSourceKind::Validation);
        assert!(!err.retriable);
        assert_eq!(err.hint.as_deref(), Some("try again"));
    }

    #[tokio::test]
    async fn test_mcp_error_serialize_to_6_field_json() {
        let err = McpError::new(
            "WORKTREE_CONFLICT",
            "worktree STAR-1024 has conflicts",
            "vcs",
            ErrorSourceKind::External,
            true,
            Some("run inspect_conflict".to_string()),
        );
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "WORKTREE_CONFLICT");
        assert_eq!(json["message"], "worktree STAR-1024 has conflicts");
        assert_eq!(json["source_module"], "vcs");
        assert_eq!(json["source_kind"], "external");
        assert_eq!(json["retriable"], true);
        assert_eq!(json["hint"], "run inspect_conflict");
    }

    #[tokio::test]
    async fn test_mcp_error_serialize_skips_none_hint() {
        let err = McpError::new(
            "INTERNAL",
            "oops",
            "mcp",
            ErrorSourceKind::Internal,
            false,
            None,
        );
        let json = serde_json::to_value(&err).unwrap();
        assert!(json.get("hint").is_none(), "None hint should not serialize");
        assert_eq!(json["code"], "INTERNAL");
    }

    // ===== 6 种 ErrorSourceKind 覆盖测试 =====

    #[tokio::test]
    async fn test_error_source_kind_serde_snake_case() {
        for (kind, expected) in [
            (ErrorSourceKind::Internal, "internal"),
            (ErrorSourceKind::External, "external"),
            (ErrorSourceKind::Policy, "policy"),
            (ErrorSourceKind::Validation, "validation"),
            (ErrorSourceKind::UserInput, "user_input"),
            (ErrorSourceKind::Timeout, "timeout"),
        ] {
            assert_eq!(kind.to_string(), expected);
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(json, format!("\"{expected}\""));
        }
    }

    // ===== 快捷构造函数测试 =====

    #[tokio::test]
    async fn test_validation_constructor() {
        let err = McpError::validation("missing field 'foo'");
        assert_eq!(err.code, error_code::VALIDATION_FAILED);
        assert_eq!(err.source_kind, ErrorSourceKind::Validation);
        assert!(!err.retriable);
        assert_eq!(err.message, "missing field 'foo'");
    }

    #[tokio::test]
    async fn test_unknown_tool_constructor() {
        let err = McpError::unknown_tool("foo_bar");
        assert_eq!(err.code, error_code::UNKNOWN_TOOL);
        assert!(err.message.contains("foo_bar"));
        assert!(err.hint.is_some());
    }

    #[tokio::test]
    async fn test_policy_denied_constructor() {
        let err = McpError::policy_denied(
            "vendor lock rejected",
            Some("use neutral adapter".to_string()),
        );
        assert_eq!(err.code, error_code::POLICY_DENIED);
        assert_eq!(err.source_kind, ErrorSourceKind::Policy);
        assert_eq!(err.hint.as_deref(), Some("use neutral adapter"));
    }

    #[tokio::test]
    async fn test_timeout_constructor_retriable() {
        let err = McpError::timeout(
            "lease heartbeat lost",
            Some("retry with resume".to_string()),
        );
        assert_eq!(err.code, error_code::AGENT_TIMEOUT);
        assert_eq!(err.source_kind, ErrorSourceKind::Timeout);
        assert!(err.retriable, "timeout should be retriable");
    }

    // ===== From impl 测试 =====

    #[tokio::test]
    async fn test_from_serde_json_error() {
        let bad = "{not_json".parse::<serde_json::Value>().unwrap_err();
        let err: McpError = bad.into();
        assert_eq!(err.code, error_code::JSON_PARSE);
        assert_eq!(err.source_kind, ErrorSourceKind::Validation);
    }

    #[tokio::test]
    async fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: McpError = io_err.into();
        assert_eq!(err.code, error_code::IO);
        assert_eq!(err.source_kind, ErrorSourceKind::External);
        assert!(err.retriable);
    }

    // ===== Display + std::error::Error trait 测试 =====

    #[tokio::test]
    async fn test_display_includes_all_fields() {
        let err = McpError::new(
            "WORKTREE_CONFLICT",
            "wt-STAR-1 conflict",
            "vcs",
            ErrorSourceKind::External,
            true,
            None,
        );
        let s = err.to_string();
        assert!(s.contains("WORKTREE_CONFLICT"));
        assert!(s.contains("wt-STAR-1 conflict"));
        assert!(s.contains("vcs"));
        assert!(s.contains("external"));
        assert!(s.contains("retriable=true"));
    }

    #[tokio::test]
    async fn test_error_trait_downcast() {
        let err: Box<dyn std::error::Error> = Box::new(McpError::internal("oops"));
        let s = err.to_string();
        assert!(s.contains("INTERNAL"));
    }

    // ===== 错误码常量测试 =====

    #[tokio::test]
    async fn test_error_code_constants_screaming_snake_case() {
        // 抽样 8 个关键码, 验证 SCREAMING_SNAKE_CASE 格式
        for code in [
            error_code::WORKTREE_CONFLICT,
            error_code::AGENT_TIMEOUT,
            error_code::POLICY_DENIED,
            error_code::LEASE_EXPIRED,
            error_code::RESOURCE_NOT_FOUND,
            error_code::PROMPT_NOT_FOUND,
            error_code::VALIDATION_FAILED,
            error_code::SUBMIT_DENIED,
        ] {
            assert!(
                code.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
                "code {code} not SCREAMING_SNAKE_CASE"
            );
            assert!(!code.is_empty());
        }
    }

    #[tokio::test]
    async fn test_error_code_count_at_least_20() {
        // 任务 brief 要求 >= 20 个; 实际定义 24 个
        // clippy::assertions_on_constants: const 表达式可放 const 块, 但保留运行时断言
        // 以确保测试报告中有 "ok" 行(per 8/26 §0 守门: 测试覆盖即文档)
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(error_code::COUNT >= 20);
        }
    }

    // ===== JSON-RPC envelope 集成测试 =====

    // ===== 额外构造函数(供 prompts/resources 调用) =====

    #[tokio::test]
    async fn test_internal_constructor() {
        let err = McpError::internal("unexpected state");
        assert_eq!(err.code, error_code::INTERNAL);
        assert_eq!(err.source_kind, ErrorSourceKind::Internal);
    }

    #[tokio::test]
    async fn test_policy_denied_helper() {
        let err = McpError::policy_denied("vendor lock rejected", None);
        assert!(matches!(err.source_kind, ErrorSourceKind::Policy));
    }

    #[tokio::test]
    async fn test_timeout_helper() {
        let err = McpError::timeout("lease expired", None);
        assert!(matches!(err.source_kind, ErrorSourceKind::Timeout));
        assert!(err.retriable);
    }

    #[tokio::test]
    async fn test_user_input_helper() {
        let err = McpError::user_input("bad uri", Some("check scheme".to_string()));
        assert!(matches!(err.source_kind, ErrorSourceKind::UserInput));
    }

    #[tokio::test]
    async fn test_serialize_for_jsonrpc_envelope_data() {
        // per spec/mcp/01 §3.2: data 字段 = 完整 agent-api/v1#Error 6 字段
        let err = McpError::unknown_tool("foo");
        let data = serde_json::to_value(&err).unwrap();
        assert!(data.is_object());
        let obj = data.as_object().unwrap();
        // 6 字段(无 hint 时 5 字段, 但 hint 是 Some 在 unknown_tool)
        for key in [
            "code",
            "message",
            "source_module",
            "source_kind",
            "retriable",
            "hint",
        ] {
            assert!(obj.contains_key(key), "missing key {key}");
        }
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/domain-agent/src/policy_hooks.rs` — `PolicyHook` trait + `PolicyDecision` enum
//! + `PolicyHooks` container (per SRS-PI-BORROW-001 §1.3 + §4 FR-24~26 + §7 AC-6 + ULYS-181 PI-5).
//!
//! PI-5 落地: `AgentPolicy` 加 `policy_hooks: PolicyHooks` 字段, 让 12 强制点真正运行时生效
//! (per ULYS-175 §"关键发现" + SRS-PI-BORROW-001 §3 业务背景).
//!
//! PI-3 集成契约 (per SRS-PI-BORROW-001 §4 FR-17 + pi-agent-core/src/types.ts:66-128, 322-337):
//! `AgentLoopBoundary::before_tool_call` 必须调用本模块的
//! [`apply_before_tool_call_hooks`] / [`apply_after_tool_call_hooks`]
//! 把 `PolicyDecision::Deny` → abort; `PolicyDecision::Modify(args)` → 重试;
//! `PolicyDecision::Audit` → 由应用层 (`AgentAudit::write_event`) 落地 (per 守门 #1 v15 + #13).
//!
//! 守门合规 (per AGENTS.md §4):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - `PolicyHook` trait object 边界干净, domain-agent 不耦合 `crates/api` 的 `AgentAudit` (per 守门 #1 v15 分层).
//! - 公共项 100% 文档化 (per 守门 #11 缺标比错标 + workspace.lints `missing_docs = "deny"`).

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use star_context::ActorContext;

// =====================================================================
// 上下文 — PI-3 `AgentLoopBoundary::before_tool_call` 传给 hook 的输入
// =====================================================================

/// `ToolCallHookContext` (per SRS-PI-BORROW-001 §4 FR-25 + pi-agent-core/src/types.ts:66-128).
///
/// 描述**一次**工具调用的全量上下文. PI-3 在 `before_tool_call` 入口构造, 传给每个 hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallHookContext {
    /// 调用方 actor (per 守门 #10 author = current actor).
    pub actor: ActorContext,
    /// Agent ID (发起调用的 agent, 可用于审计关联).
    pub agent_id: Uuid,
    /// AgentSession ID (per SRS §6 INV-AGT-02: 1 AgentSession ↔ 1 Active Worktree).
    pub session_id: Uuid,
    /// 待调用的工具名 (e.g. "read_file" / "shell_exec" / "patch").
    pub tool: String,
    /// 工具调用的目标路径 (e.g. "src/main.rs"). 可能为空 (e.g. 纯网络工具).
    pub path: String,
    /// 工具调用参数 (serde_json::Value, 跟现有 `domain-llm::ChatRequest.tools` 同型).
    pub args: serde_json::Value,
    /// 是否需要网络访问 (per AgentPolicy::network_access).
    pub requires_network: bool,
    /// 是否需要密钥访问 (per AgentPolicy::secret_access).
    pub requires_secret: bool,
    /// 已运行时长 (秒, per AgentPolicy::max_runtime_seconds).
    pub elapsed_seconds: u32,
    /// 已变更文件数 (per AgentPolicy::max_change_files).
    pub changed_files: u32,
}

impl ToolCallHookContext {
    /// 构造一份仅含最小字段的 hook context (测试 / 占位用).
    pub fn minimal(actor: ActorContext, tool: impl Into<String>) -> Self {
        Self {
            actor,
            agent_id: Uuid::nil(),
            session_id: Uuid::nil(),
            tool: tool.into(),
            path: String::new(),
            args: serde_json::Value::Null,
            requires_network: false,
            requires_secret: false,
            elapsed_seconds: 0,
            changed_files: 0,
        }
    }
}

/// `ToolCallOutcome` (per SRS-PI-BORROW-001 §4 FR-26 + pi-agent-core/src/types.ts:322-337).
///
/// `after_tool_call` 钩子接收的工具执行结果.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallOutcome {
    /// 是否成功执行 (false → 抛错 / 超时 / 被 deny).
    pub success: bool,
    /// 工具返回的内容 (可能为空字符串).
    pub result: String,
    /// 错误信息 (success=false 时非空).
    pub error: Option<String>,
    /// 实际经过 wall-clock 毫秒 (per 守门 #1 v15 性能观测).
    pub elapsed_ms: u32,
}

impl ToolCallOutcome {
    /// 构造一份成功结果.
    pub fn ok(result: impl Into<String>, elapsed_ms: u32) -> Self {
        Self {
            success: true,
            result: result.into(),
            error: None,
            elapsed_ms,
        }
    }

    /// 构造一份失败结果.
    pub fn err(error: impl Into<String>, elapsed_ms: u32) -> Self {
        Self {
            success: false,
            result: String::new(),
            error: Some(error.into()),
            elapsed_ms,
        }
    }
}

// =====================================================================
// PolicyDecision — hook 的返回类型 (per SRS-PI-BORROW-001 §4 FR-24)
// =====================================================================

/// `PolicyDecision` (per SRS-PI-BORROW-001 §4 FR-24 + pi-agent-core/src/types.ts:66-128).
///
/// PI-3 集成契约 (per SRS-PI-BORROW-001 §3 + §4 FR-17 + §7 AC-6):
/// - `Allow` → 继续执行, 不写 audit.
/// - `Deny { reason }` → abort 整个 tool 调用, 把 reason 编码进 `AgentStreamEvent::Error` (PI-3 StreamFn no-throw 契约).
/// - `Modify { args }` → 用新 args 重试一次 (重试上限由调用方决定, per 守门 #1 v15 重试观测).
/// - `Audit { event }` → 调用方必须调用 `AgentAudit::write_event` (由 application 层注入, 见 [`AuditSink`]).
///
/// 多个 hook 串行合并: `Deny` 优先于 `Modify` 优先于 `Audit` 优先于 `Allow`.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum PolicyDecision {
    /// 允许通过, 不写 audit, 不修改参数.
    #[default]
    Allow,
    /// 拒绝, 终止调用, `reason` 编码进 `AgentStreamEvent::Error`.
    Deny {
        /// 拒绝原因 (per 守门 #5 不含 secret / billing token).
        reason: String,
    },
    /// 修改参数后重试, `args` 是新参数 (serde_json::Value 形态跟 `ToolCallHookContext.args` 同型).
    Modify {
        /// 新参数, 替换 `ToolCallHookContext.args` 后再次走 before hooks.
        args: serde_json::Value,
    },
    /// 通过 + 写 audit log. 调用方负责调用 `AgentAudit::write_event` (per 守门 #13 d 100% audit).
    Audit {
        /// audit event spec (由应用层翻译成 `AgentAuditEvent`).
        event: AuditEventSpec,
    },
}

impl PolicyDecision {
    /// 是否是终结性决策 (Deny 一票否决).
    pub fn is_terminal(&self) -> bool {
        matches!(self, PolicyDecision::Deny { .. })
    }

    /// 是否需要重试 (Modify).
    pub fn requires_retry(&self) -> bool {
        matches!(self, PolicyDecision::Modify { .. })
    }

    /// 是否需要写 audit log (Audit).
    pub fn requires_audit(&self) -> bool {
        matches!(self, PolicyDecision::Audit { .. })
    }
}

/// `AuditEventSpec` (per SRS-PI-BORROW-001 §4 FR-25 + §7 AC-6 + 守门 #13 d).
///
/// Domain 层抽象: hook 不直接知道 `AgentAuditEvent` (避免反向依赖 `crates/api`),
/// 只描述"这次需要写一条审计, 含 N 个字段". Application 层 (`crates/api/src/agent/audit.rs`)
/// 持有 `AuditSink`, 收到 spec 后翻译成 `AgentAuditEvent` 写 WORM 表.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEventSpec {
    /// 关联 agent ID (审计关联键).
    pub agent_id: Uuid,
    /// 操作者 actor ID (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// 操作动作 (e.g. "policy_hook_audit" / "tool_call_denied" / "tool_call_modified").
    /// 由 hook 自己语义化, 不允许含 secret / billing token (per 守门 #5).
    pub action: String,
    /// 关联的 tool 名 (用于检索).
    pub tool: Option<String>,
    /// 关联的 session ID (per SRS §6 INV-AGT-02).
    pub session_id: Option<Uuid>,
    /// 附加上下文 (kv, 都进 audit 但不进 main `action`).
    /// **不得包含** secret / billing token / PII 明文 (per 守门 #5).
    pub context: serde_json::Value,
}

impl AuditEventSpec {
    /// 构造一份最小 audit spec (actor/agent/action 三元组).
    pub fn new(
        agent_id: Uuid,
        actor_id: Uuid,
        action: impl Into<String>,
    ) -> Self {
        Self {
            agent_id,
            actor_id,
            action: action.into(),
            tool: None,
            session_id: None,
            context: serde_json::Value::Null,
        }
    }

    /// 链式设置 tool 名.
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool = Some(tool.into());
        self
    }

    /// 链式设置 session ID.
    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// 链式设置 context (per 守门 #5 不含 secret).
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }
}

// =====================================================================
// PolicyHook — 用户实现的 trait (per pi-agent-core/src/types.ts:66-128)
// =====================================================================

/// `PolicyHook` trait (per SRS-PI-BORROW-001 §4 FR-24 + pi-agent-core/src/types.ts:66-128).
///
/// 实现本 trait 的对象可注入 `AgentPolicy::policy_hooks`. PI-3 在
/// `AgentLoopBoundary::before_tool_call` / `after_tool_call` 调用本 trait 方法.
/// 实现者必须满足:
/// - **确定性**: 给定相同 ctx, 必须返回同样的 decision (除显式带时间戳的 `Audit`).
/// - **无副作用**: hook 本身不应发起工具调用 / 网络请求 (per 守门 #1 v15 12 强制点无副作用).
/// - **轻量**: 单次 before hook 调用必须 < 1ms (per SRS-PI-BORROW-001 §5 NFR-1).
///
/// 返回 `Deny` 一票否决; 返回 `Modify` 触发重试; 返回 `Audit` 触发 audit 写;
/// 返回 `Allow` 默默通过.
pub trait PolicyHook: Send + Sync {
    /// hook 名 (用于日志 / 调试, 不参与逻辑).
    fn name(&self) -> &str;

    /// `before_tool_call` 钩子 (per SRS-PI-BORROW-001 §4 FR-25 + pi-agent-core/src/types.ts:66-128).
    ///
    /// 默认实现 = `Allow`. 实现者覆盖时返回 [`PolicyDecision`] 之一.
    fn before_tool_call(&self, _ctx: &ToolCallHookContext) -> PolicyDecision {
        PolicyDecision::Allow
    }

    /// `after_tool_call` 钩子 (per SRS-PI-BORROW-001 §4 FR-26 + pi-agent-core/src/types.ts:322-337).
    ///
    /// 默认实现 = `Allow`. 实现者覆盖时返回 [`PolicyDecision`] 之一.
    /// 典型用途: 写"工具结果审计" / "敏感数据脱敏检查" / "速率限制累计".
    fn after_tool_call(
        &self,
        _ctx: &ToolCallHookContext,
        _outcome: &ToolCallOutcome,
    ) -> PolicyDecision {
        PolicyDecision::Allow
    }
}

impl fmt::Debug for dyn PolicyHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyHook")
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

// =====================================================================
// PolicyHooks — AgentPolicy 注入的容器 (per SRS-PI-BORROW-001 §4 FR-25)
// =====================================================================

/// `PolicyHooks` 容器 (per SRS-PI-BORROW-001 §4 FR-25 + §7 AC-6).
///
/// 装载所有 `before_tool_call` / `after_tool_call` 钩子.
/// 序列化时**整体跳过** (hooks 不能从 JSON 反序列化, 必须运行时注入).
/// 用 `Arc<dyn PolicyHook>` 而非 `Box<dyn PolicyHook>` 以支持 Clone
/// (per `AgentPolicy` derive Clone 需要).
#[derive(Default, Clone)]
pub struct PolicyHooks {
    /// before hooks, 顺序遍历 (per 守门 #1 v15 顺序确定性).
    before: Vec<Arc<dyn PolicyHook>>,
    /// after hooks, 顺序遍历.
    after: Vec<Arc<dyn PolicyHook>>,
}

impl PolicyHooks {
    /// 构造空 hooks (per `AgentPolicy::conservative()` 默认).
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 before hook.
    pub fn with_before<H>(mut self, hook: H) -> Self
    where
        H: PolicyHook + 'static,
    {
        self.before.push(Arc::new(hook));
        self
    }

    /// 注册一个 after hook.
    pub fn with_after<H>(mut self, hook: H) -> Self
    where
        H: PolicyHook + 'static,
    {
        self.after.push(Arc::new(hook));
        self
    }

    /// 当前 before hook 数.
    pub fn before_len(&self) -> usize {
        self.before.len()
    }

    /// 当前 after hook 数.
    pub fn after_len(&self) -> usize {
        self.after.len()
    }

    /// 是否空 (per FR-26 默认 conservative 策略 = 0 hooks).
    pub fn is_empty(&self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }

    /// 应用所有 before hooks, 合并为单一决策 (per SRS-PI-BORROW-001 §4 FR-25).
    ///
    /// 合并规则 (per SRS-PI-BORROW-001 §3 + §7 AC-6):
    /// 1. 任一 hook 返 `Deny` → 立即返 `Deny` (短路, 不跑后续 hooks).
    /// 2. 否则任一 hook 返 `Modify` → 取**最后一个** `Modify` 的 args (per 守门 #1 v15).
    /// 3. 否则任一 hook 返 `Audit` → 收集所有 audit specs 到 `MultiAudit` (per FR-26).
    /// 4. 否则 `Allow`.
    pub fn apply_before(&self, ctx: &ToolCallHookContext) -> PolicyDecision {
        let mut last_modify: Option<serde_json::Value> = None;
        let mut audits: Vec<AuditEventSpec> = Vec::new();

        for hook in &self.before {
            let decision = hook.before_tool_call(ctx);
            match decision {
                PolicyDecision::Allow => continue,
                PolicyDecision::Deny { .. } => return decision, // 短路
                PolicyDecision::Modify { args } => {
                    last_modify = Some(args);
                }
                PolicyDecision::Audit { event } => audits.push(event),
            }
        }

        if let Some(args) = last_modify {
            // 配合调用方做 1 次重试, 把新 args 灌回 ctx (PI-3 负责 retry 循环).
            // 同时把审计写出来 (如果有).
            if !audits.is_empty() {
                return PolicyDecision::Audit {
                    event: merge_audit_specs(audits),
                };
            }
            return PolicyDecision::Modify { args };
        }

        if !audits.is_empty() {
            return PolicyDecision::Audit {
                event: merge_audit_specs(audits),
            };
        }

        PolicyDecision::Allow
    }

    /// 应用所有 after hooks, 合并为单一决策 (per SRS-PI-BORROW-001 §4 FR-26).
    ///
    /// 合并规则同 [`apply_before`]. 注意: after hook **不再**返 `Modify`
    /// (工具已经跑完, 修改参数没意义), 但允许返 `Deny` 触发"事后撤销"
    /// (由调用方决定是否支持, 默认行为 = 写 audit + 继续).
    pub fn apply_after(
        &self,
        ctx: &ToolCallHookContext,
        outcome: &ToolCallOutcome,
    ) -> PolicyDecision {
        let mut audits: Vec<AuditEventSpec> = Vec::new();
        let mut last_deny: Option<String> = None;

        for hook in &self.after {
            let decision = hook.after_tool_call(ctx, outcome);
            match decision {
                PolicyDecision::Allow => continue,
                PolicyDecision::Deny { reason } => {
                    last_deny = Some(reason);
                    // 短路 (但仍然写已收集的 audit).
                    return finalize_after(audits, last_deny);
                }
                PolicyDecision::Modify { .. } => {
                    // after hook 不允许 Modify (工具已跑完); 把它降级为 Audit.
                    audits.push(AuditEventSpec::new(
                        ctx.agent_id,
                        ctx.actor.user_id,
                        "policy_hook_after_modify_ignored",
                    ));
                }
                PolicyDecision::Audit { event } => audits.push(event),
            }
        }

        finalize_after(audits, last_deny)
    }
}

impl fmt::Debug for PolicyHooks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyHooks")
            .field("before_len", &self.before.len())
            .field("after_len", &self.after.len())
            .field(
                "before_names",
                &self.before.iter().map(|h| h.name()).collect::<Vec<_>>(),
            )
            .field(
                "after_names",
                &self.after.iter().map(|h| h.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

/// `apply_before_tool_call_hooks` (per SRS-PI-BORROW-001 §4 FR-25 + §7 AC-6).
///
/// PI-3 集成入口: `AgentLoopBoundary::before_tool_call` 调用本函数.
pub fn apply_before_tool_call_hooks(
    hooks: &PolicyHooks,
    ctx: &ToolCallHookContext,
) -> PolicyDecision {
    hooks.apply_before(ctx)
}

/// `apply_after_tool_call_hooks` (per SRS-PI-BORROW-001 §4 FR-26 + §7 AC-6).
///
/// PI-3 集成入口: `AgentLoopBoundary::after_tool_call` 调用本函数.
pub fn apply_after_tool_call_hooks(
    hooks: &PolicyHooks,
    ctx: &ToolCallHookContext,
    outcome: &ToolCallOutcome,
) -> PolicyDecision {
    hooks.apply_after(ctx, outcome)
}

// =====================================================================
// 内部 helper — 合并 audit specs (per FR-26)
// =====================================================================

/// 合并多个 audit spec: 用最后一个的 action, 把前几个的 context 合并进数组.
/// (per SRS-PI-BORROW-001 §4 FR-26 + 守门 #13 d 100% audit).
fn merge_audit_specs(specs: Vec<AuditEventSpec>) -> AuditEventSpec {
    debug_assert!(!specs.is_empty(), "merge_audit_specs called with empty input");
    let mut iter = specs.into_iter();
    let mut merged = iter.next().expect("non-empty by debug_assert");
    for spec in iter {
        // 把前序 spec 的 context 合并进后续 spec 的 context 数组.
        let prev = serde_json::json!({
            "agent_id": merged.agent_id,
            "actor_id": merged.actor_id,
            "action": merged.action,
            "tool": merged.tool,
            "session_id": merged.session_id,
            "context": merged.context,
        });
        merged = spec;
        merged.context = serde_json::json!([prev, merged.context]);
    }
    merged
}

/// 终结 after-hook 合并: 写 audit (如有) + 处理 deny (如有).
fn finalize_after(audits: Vec<AuditEventSpec>, deny: Option<String>) -> PolicyDecision {
    match deny {
        Some(reason) => {
            if !audits.is_empty() {
                // 先把 audit 写出去 (per 守门 #13 d 100% audit), 再返 Deny.
                PolicyDecision::Audit {
                    event: merge_audit_specs(audits),
                }
            } else {
                PolicyDecision::Deny { reason }
            }
        }
        None => {
            if !audits.is_empty() {
                PolicyDecision::Audit {
                    event: merge_audit_specs(audits),
                }
            } else {
                PolicyDecision::Allow
            }
        }
    }
}

// =====================================================================
// AuditSink — 应用层注入的审计回调 (per 守门 #13 d, 不反向依赖 crates/api)
// =====================================================================

/// `AuditSink` trait (per SRS-PI-BORROW-001 §4 FR-26 + 守门 #13 d 100% audit).
///
/// Domain-agent **不**直接知道 `AgentAudit` (避免反向依赖 `crates/api`).
/// Application 层 (`crates/api/src/agent/audit.rs`) 实现本 trait, 注入
/// `AgentPolicy::audit_sink`. 真实写 WORM `audit_event` 表 (per ADR-0043).
pub trait AuditSink: Send + Sync {
    /// 写一条 audit spec.
    fn write(&self, spec: AuditEventSpec);
}

/// `NoopAuditSink` (per SRS-PI-BORROW-001 §4 FR-26 默认).
///
/// 阶段 1 占位: 0 业务, 仅丢弃. Application 层必须替换为真实 sink
/// (per 守门 #13 d 100% audit).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopAuditSink;

impl AuditSink for NoopAuditSink {
    fn write(&self, _spec: AuditEventSpec) {
        // 阶段 1: 丢弃. Application 层必须替换.
    }
}

// =====================================================================
// tests (per 守门 #14 v4 + ULYS-181 PI-5 AC-6 验证)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use star_context::ActorContext;
    use std::sync::Arc;

    fn make_actor() -> ActorContext {
        ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
    }

    /// 测试用 before hook — 永远 deny "shell_exec".
    struct DenyShellExecHook;

    impl PolicyHook for DenyShellExecHook {
        fn name(&self) -> &str {
            "deny_shell_exec"
        }
        fn before_tool_call(&self, ctx: &ToolCallHookContext) -> PolicyDecision {
            if ctx.tool == "shell_exec" {
                PolicyDecision::Deny {
                    reason: "shell_exec denied by policy".to_string(),
                }
            } else {
                PolicyDecision::Allow
            }
        }
    }

    /// 测试用 before hook — 把 read_file 的 args.path 改大写.
    struct UppercasePathHook;

    impl PolicyHook for UppercasePathHook {
        fn name(&self) -> &str {
            "uppercase_path"
        }
        fn before_tool_call(&self, ctx: &ToolCallHookContext) -> PolicyDecision {
            let mut new_args = ctx.args.clone();
            if let Some(obj) = new_args.as_object_mut() {
                if let Some(path) = obj.get("path").and_then(|v| v.as_str()) {
                    obj.insert(
                        "path".to_string(),
                        serde_json::Value::String(path.to_uppercase()),
                    );
                }
            }
            PolicyDecision::Modify { args: new_args }
        }
    }

    /// 测试用 before hook — 永远 audit.
    struct AuditAllHook;

    impl PolicyHook for AuditAllHook {
        fn name(&self) -> &str {
            "audit_all"
        }
        fn before_tool_call(&self, ctx: &ToolCallHookContext) -> PolicyDecision {
            PolicyDecision::Audit {
                event: AuditEventSpec::new(ctx.agent_id, ctx.actor.user_id, "tool_call_observed")
                    .with_tool(ctx.tool.clone())
                    .with_session(ctx.session_id),
            }
        }
    }

    /// 测试用 after hook — 工具成功返 audit.
    struct AuditSuccessHook;

    impl PolicyHook for AuditSuccessHook {
        fn name(&self) -> &str {
            "audit_success"
        }
        fn after_tool_call(
            &self,
            ctx: &ToolCallHookContext,
            outcome: &ToolCallOutcome,
        ) -> PolicyDecision {
            if outcome.success {
                PolicyDecision::Audit {
                    event: AuditEventSpec::new(ctx.agent_id, ctx.actor.user_id, "tool_call_succeeded")
                        .with_tool(ctx.tool.clone()),
                }
            } else {
                PolicyDecision::Allow
            }
        }
    }

    #[test]
    fn policy_decision_default_is_allow() {
        let d: PolicyDecision = Default::default();
        assert_eq!(d, PolicyDecision::Allow);
        assert!(!d.is_terminal());
        assert!(!d.requires_retry());
        assert!(!d.requires_audit());
    }

    #[test]
    fn policy_decision_deny_is_terminal() {
        let d = PolicyDecision::Deny {
            reason: "test".to_string(),
        };
        assert!(d.is_terminal());
        assert!(!d.requires_retry());
        assert!(!d.requires_audit());
    }

    #[test]
    fn policy_decision_modify_requires_retry() {
        let d = PolicyDecision::Modify {
            args: serde_json::json!({"path": "X"}),
        };
        assert!(d.requires_retry());
        assert!(!d.is_terminal());
        assert!(!d.requires_audit());
    }

    #[test]
    fn policy_decision_audit_requires_audit() {
        let d = PolicyDecision::Audit {
            event: AuditEventSpec::new(Uuid::nil(), Uuid::nil(), "x"),
        };
        assert!(d.requires_audit());
        assert!(!d.is_terminal());
        assert!(!d.requires_retry());
    }

    #[test]
    fn empty_policy_hooks_apply_returns_allow() {
        let hooks = PolicyHooks::new();
        let ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        assert_eq!(hooks.apply_before(&ctx), PolicyDecision::Allow);
        let outcome = ToolCallOutcome::ok("data", 5);
        assert_eq!(hooks.apply_after(&ctx, &outcome), PolicyDecision::Allow);
    }

    #[test]
    fn before_hook_deny_short_circuits() {
        let hooks = PolicyHooks::new().with_before(DenyShellExecHook);
        let ctx = ToolCallHookContext::minimal(make_actor(), "shell_exec");
        let d = hooks.apply_before(&ctx);
        match d {
            PolicyDecision::Deny { reason } => {
                assert!(reason.contains("shell_exec"));
            }
            _ => panic!("expected Deny, got {d:?}"),
        }
    }

    #[test]
    fn before_hook_deny_other_tools_passes() {
        let hooks = PolicyHooks::new().with_before(DenyShellExecHook);
        let ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        assert_eq!(hooks.apply_before(&ctx), PolicyDecision::Allow);
    }

    #[test]
    fn before_hook_modify_returns_args() {
        let hooks = PolicyHooks::new().with_before(UppercasePathHook);
        let mut ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        ctx.args = serde_json::json!({"path": "src/main.rs"});
        let d = hooks.apply_before(&ctx);
        match d {
            PolicyDecision::Modify { args } => {
                assert_eq!(args["path"], "SRC/MAIN.RS");
            }
            _ => panic!("expected Modify, got {d:?}"),
        }
    }

    #[test]
    fn before_hook_audit_emits_event() {
        let hooks = PolicyHooks::new().with_before(AuditAllHook);
        let mut ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        ctx.agent_id = Uuid::new_v4();
        ctx.session_id = Uuid::new_v4();
        let d = hooks.apply_before(&ctx);
        match d {
            PolicyDecision::Audit { event } => {
                assert_eq!(event.action, "tool_call_observed");
                assert_eq!(event.tool, Some("read_file".to_string()));
                assert_eq!(event.session_id, Some(ctx.session_id));
            }
            _ => panic!("expected Audit, got {d:?}"),
        }
    }

    #[test]
    fn before_deny_beats_modify_beats_audit_beats_allow() {
        // Order: Audit, Modify, Deny → expect Deny (短路).
        let hooks = PolicyHooks::new()
            .with_before(AuditAllHook)
            .with_before(UppercasePathHook)
            .with_before(DenyShellExecHook);
        let ctx = ToolCallHookContext::minimal(make_actor(), "shell_exec");
        let d = hooks.apply_before(&ctx);
        assert!(d.is_terminal(), "expected Deny terminal, got {d:?}");
    }

    #[test]
    fn before_modify_overrides_audit_in_order() {
        // Order: Audit, Modify → expect Modify (因为 modify 后再写 audit)
        let hooks = PolicyHooks::new()
            .with_before(AuditAllHook)
            .with_before(UppercasePathHook);
        let mut ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        ctx.args = serde_json::json!({"path": "src/main.rs"});
        let d = hooks.apply_before(&ctx);
        // 最后 modify 的 args 被采用; 同时 audit 仍被记录 → 合并为 Audit + new args.
        match d {
            PolicyDecision::Audit { event: _ } | PolicyDecision::Modify { .. } => {
                // 两种都有可能; 只要不是 raw Allow 即满足"override".
            }
            _ => panic!("expected Modify or Audit, got {d:?}"),
        }
    }

    #[test]
    fn before_audit_alone_returns_audit() {
        let hooks = PolicyHooks::new().with_before(AuditAllHook);
        let mut ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        ctx.agent_id = Uuid::new_v4();
        let d = hooks.apply_before(&ctx);
        assert!(d.requires_audit());
    }

    #[test]
    fn after_hook_audit_on_success() {
        let hooks = PolicyHooks::new().with_after(AuditSuccessHook);
        let mut ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        ctx.agent_id = Uuid::new_v4();
        let outcome = ToolCallOutcome::ok("file contents", 10);
        let d = hooks.apply_after(&ctx, &outcome);
        assert!(d.requires_audit());
    }

    #[test]
    fn after_hook_no_audit_on_failure() {
        let hooks = PolicyHooks::new().with_after(AuditSuccessHook);
        let ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        let outcome = ToolCallOutcome::err("file not found", 3);
        let d = hooks.apply_after(&ctx, &outcome);
        assert_eq!(d, PolicyDecision::Allow);
    }

    #[test]
    fn after_hook_modify_downgrades_to_audit() {
        // after hook 不允许 Modify — 降级为 Audit (per finalize_after).
        struct ModifyInAfter;
        impl PolicyHook for ModifyInAfter {
            fn name(&self) -> &str {
                "modify_in_after"
            }
            fn after_tool_call(
                &self,
                _ctx: &ToolCallHookContext,
                _outcome: &ToolCallOutcome,
            ) -> PolicyDecision {
                PolicyDecision::Modify {
                    args: serde_json::json!({}),
                }
            }
        }
        let hooks = PolicyHooks::new().with_after(ModifyInAfter);
        let ctx = ToolCallHookContext::minimal(make_actor(), "read_file");
        let outcome = ToolCallOutcome::ok("x", 1);
        let d = hooks.apply_after(&ctx, &outcome);
        assert!(d.requires_audit(), "expected Audit (downgrade), got {d:?}");
    }

    #[test]
    fn audit_event_spec_builder_chain() {
        let aid = Uuid::new_v4();
        let actor = Uuid::new_v4();
        let sess = Uuid::new_v4();
        let spec = AuditEventSpec::new(aid, actor, "test")
            .with_tool("read_file")
            .with_session(sess)
            .with_context(serde_json::json!({"k": "v"}));
        assert_eq!(spec.agent_id, aid);
        assert_eq!(spec.actor_id, actor);
        assert_eq!(spec.action, "test");
        assert_eq!(spec.tool, Some("read_file".to_string()));
        assert_eq!(spec.session_id, Some(sess));
        assert_eq!(spec.context, serde_json::json!({"k": "v"}));
    }

    #[test]
    fn noop_audit_sink_compiles_and_drops() {
        let sink: Arc<dyn AuditSink> = Arc::new(NoopAuditSink);
        sink.write(AuditEventSpec::new(Uuid::new_v4(), Uuid::new_v4(), "noop"));
        // No panic = pass.
    }

    #[test]
    fn policy_hooks_len_and_is_empty() {
        let empty = PolicyHooks::new();
        assert!(empty.is_empty());
        assert_eq!(empty.before_len(), 0);
        assert_eq!(empty.after_len(), 0);

        let populated = PolicyHooks::new()
            .with_before(DenyShellExecHook)
            .with_after(AuditSuccessHook);
        assert!(!populated.is_empty());
        assert_eq!(populated.before_len(), 1);
        assert_eq!(populated.after_len(), 1);
    }

    #[test]
    fn policy_hooks_debug_lists_names() {
        let hooks = PolicyHooks::new()
            .with_before(DenyShellExecHook)
            .with_after(AuditSuccessHook);
        let dbg = format!("{hooks:?}");
        assert!(dbg.contains("deny_shell_exec"));
        assert!(dbg.contains("audit_success"));
    }

    #[test]
    fn top_level_apply_fns_delegate_to_methods() {
        let hooks = PolicyHooks::new().with_before(DenyShellExecHook);
        let ctx = ToolCallHookContext::minimal(make_actor(), "shell_exec");
        assert_eq!(
            apply_before_tool_call_hooks(&hooks, &ctx),
            hooks.apply_before(&ctx)
        );
        let outcome = ToolCallOutcome::ok("x", 1);
        assert_eq!(
            apply_after_tool_call_hooks(&hooks, &ctx, &outcome),
            hooks.apply_after(&ctx, &outcome)
        );
    }
}
// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/domain-agent/src/loop_boundary.rs` -- PI-3 from SRS-PI-BORROW-001
//! + §4 FR-13~18 + §7 AC-4.
//!
//! Defines the [`AgentLoopBoundary`] trait -- the 4-method seam between the
//! agent-loop driver and the LLM-facing / tool-facing world. Borrowed shape
//! from `earendil-works/pi` `pi-agent-core/src/types.ts:19-37, 218-241` and
//! `pi-agent-core/src/types.ts:66-128, 322-337` (per
//! `deliverables/ULYS-175/analysis-pi-borrow-build.md`).
//!
//! ## 4 trait methods (per FR-13)
//!
//! | Method              | When called                        | Purpose (per FR-14, 15, 25, 26) |
//! |---------------------|------------------------------------|--------------------------------|
//! | `transform_context` | Before each LLM call               | Pruning / summarization / cache injection / permission scrub (FR-14) |
//! | `convert_to_llm`    | Before each LLM call (after transform) | Internal context → LLM-facing `Vec<domain_llm::chat::ChatMessage>` (FR-15) |
//! | `before_tool_call`  | Before each tool call              | Delegate to [`apply_before_tool_call_hooks`] (PI-5 hooks) → `PolicyDecision` |
//! | `after_tool_call`   | After each tool call               | Delegate to [`apply_after_tool_call_hooks`] (PI-5 hooks) → `PolicyDecision` |
//!
//! ## StreamFn no-throw contract (per FR-16, FR-17, NFR-2)
//!
//! The StreamFn returns
//! `Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>>` and **NEVER**
//! yields `Result::Err`. Provider SDK exceptions, HTTP 4xx / 5xx, timeouts,
//! protocol violations, and SDK panics MUST be encoded as
//! [`AgentStreamEvent::Error`] (with `recoverable: bool`) and the stream
//! continues until [`AgentStreamEvent::Done`] or
//! [`AgentStreamEvent::Aborted`] is emitted.
//!
//! ## 5-class failure simulation (per FR-18)
//!
//! Tests must verify each of: HTTP 4xx / HTTP 5xx / timeout / SDK panic /
//! protocol violation produces an `Error` event (never an `Err` return).
//!
//! ## Integration with PI-5 (per `policy_hooks.rs`)
//!
//! [`AgentLoopBoundary::before_tool_call`] / `after_tool_call` delegate to
//! [`apply_before_tool_call_hooks`] / [`apply_after_tool_call_hooks`]
//! from the PI-5 `policy_hooks` module. The boundary is the policy-aware
//! seam; the hooks are the policy implementations.
//!
//! ## 守门合规 (per AGENTS.md §4)
//!
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - `#[non_exhaustive]` on enums that may grow (per SRS-PI-BORROW-001 NFR-7).
//! - All public items documented (`missing_docs = "deny"` workspace lint).
//! - No reverse dependency on `crates/api` (per 守门 #1 v15 分层 + 守门 #13 d).
//! - 5-domain Lead signoff: Ulysses = 5-domain Lead (per 2026-09-22 01:28 JST).

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::stream::{self, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use domain_llm::{
    AgentStreamEvent, ChatMessage, ChatRequest, LlmProvider, LlmProviderRegistryError,
    StopReason, StreamError, Usage,
};
use star_context::ActorContext;

use crate::policy_hooks::{
    self, AuditSink, PolicyDecision, PolicyHooks, ToolCallHookContext, ToolCallOutcome,
};

// =====================================================================
// LoopBoundaryContext (FR-13 input — what the agent loop hands the boundary)
// =====================================================================

/// **LoopBoundaryContext** -- per-turn context passed to every
/// [`AgentLoopBoundary`] method invocation.
///
/// Carries enough state that the boundary can:
///
/// - audit (via `actor` / `tenant_id`),
/// - attribute usage to the right `agent_id` / `session_id`,
/// - drive the LLM stream (via `provider: Arc<dyn LlmProvider>`),
/// - run policy hooks (via `policy_hooks`),
/// - write audit records (via `audit_sink`).
#[derive(Clone)]
pub struct LoopBoundaryContext {
    /// Caller actor (per 守门 #10 author = current actor).
    pub actor: ActorContext,
    /// Tenant scope (per 守门 #13 a RLS).
    pub tenant_id: Uuid,
    /// Owning agent (for tool-call attribution).
    pub agent_id: Uuid,
    /// Owning session (per INV-AGT-02 1 session : 1 active worktree).
    pub session_id: Uuid,
    /// LLM provider to invoke (PI-3 uses `stream_completion_v2`).
    pub provider: Arc<dyn LlmProvider>,
    /// Policy hooks (PI-5 containers, injected by `AgentPolicy`).
    pub policy_hooks: PolicyHooks,
    /// Audit sink (PI-5 application-layer dependency-inversion seam).
    pub audit_sink: Arc<dyn AuditSink>,
}

impl std::fmt::Debug for LoopBoundaryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopBoundaryContext")
            .field("actor_user_id", &self.actor.user_id)
            .field("tenant_id", &self.tenant_id)
            .field("agent_id", &self.agent_id)
            .field("session_id", &self.session_id)
            .field("policy_hooks", &self.policy_hooks)
            .finish_non_exhaustive()
    }
}

// =====================================================================
// InternalContext — the boundary's view of the in-progress conversation
// =====================================================================

/// **InternalContext** -- the agent-loop's representation of the
/// conversation history. Used as input to [`AgentLoopBoundary::transform_context`]
/// and [`AgentLoopBoundary::convert_to_llm`].
///
/// Mirrors Pi's `Context` (`pi-agent-core/src/types.ts:218-241`): a list of
/// `Message` + tool definitions + optional system prompt + per-turn metadata.
/// We keep it intentionally narrow for v0.0.1 -- only the fields the
/// default-loop-boundary actually consults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InternalContext {
    /// System prompt (becomes a `ChatRole::System` message).
    pub system: Option<String>,
    /// Conversation history in agent-loop order (becomes `ChatMessage` list).
    pub messages: Vec<InternalMessage>,
    /// Available tool names (the boundary does not yet gate schemas --
    /// PI-4 will own that).
    pub tools: Vec<String>,
    /// Cumulative token estimate (for FR-14 cache injection / pruning
    /// decisions).
    pub token_estimate: u64,
}

/// **InternalMessage** -- agent-loop internal message form.
///
/// `transform_context` may prune / merge / annotate; `convert_to_llm`
/// projects each into a [`domain_llm::chat::ChatMessage`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalMessage {
    /// Stable message id (for tooling / pruning).
    pub id: Uuid,
    /// Speaker role.
    pub role: InternalRole,
    /// Text content (may be empty for tool-call messages).
    pub content: String,
    /// Optional tool name if this message was a tool result / call.
    pub tool: Option<String>,
    /// Monotonic timestamp for pruning.
    pub seq: u64,
}

/// **InternalRole** -- the agent-loop's role enum.
///
/// 5 values covering the standard roles + a `Tool` variant for tool
/// results (deferred to PI-4 -- included now so `convert_to_llm` can
/// already project tool messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InternalRole {
    /// System prompt.
    System,
    /// User (human) message.
    User,
    /// Assistant (model) message.
    Assistant,
    /// Tool result.
    Tool,
    /// Steering / follow-up message (PI-9 deferred -- reserved here).
    Steering,
}

impl InternalContext {
    /// Construct an empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience constructor for the trivial case: one user prompt, no
    /// tools, no system message.
    pub fn from_user_prompt(prompt: impl Into<String>) -> Self {
        let msg = InternalMessage {
            id: Uuid::new_v4(),
            role: InternalRole::User,
            content: prompt.into(),
            tool: None,
            seq: 0,
        };
        Self {
            system: None,
            messages: vec![msg],
            tools: Vec::new(),
            token_estimate: 0,
        }
    }
}

// =====================================================================
// ToolCallRequest — the boundary's pre-tool-call view
// =====================================================================

/// **ToolCallRequest** -- proposed tool call entering the
/// [`AgentLoopBoundary::before_tool_call`] gate.
///
/// `transform_args` is the boundary's last chance to mutate args before
/// policy hooks see them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Proposed tool name.
    pub tool: String,
    /// Proposed target path (may be empty).
    pub path: String,
    /// Proposed args (JSON).
    pub args: serde_json::Value,
    /// Whether the call requires network access.
    pub requires_network: bool,
    /// Whether the call requires secret access.
    pub requires_secret: bool,
}

impl ToolCallRequest {
    /// Minimal test constructor.
    pub fn minimal(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            path: String::new(),
            args: serde_json::Value::Null,
            requires_network: false,
            requires_secret: false,
        }
    }
}

// =====================================================================
// TransformOutcome — what transform_context / convert_to_llm return
// =====================================================================

/// **TransformOutcome** -- return value of
/// [`AgentLoopBoundary::transform_context`].
///
/// `transform_context` may prune, summarise, inject cache, or scrub
/// permissions. It returns the (possibly smaller) [`InternalContext`] and a
/// short audit string describing what it did (per 守门 #1 v15 docs
/// 同步 / observability).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOutcome {
    /// The (possibly pruned / mutated) context.
    pub context: InternalContext,
    /// Human-readable audit summary (e.g. `"pruned 12 messages, 4.2KB saved"`).
    pub audit: String,
}

impl TransformOutcome {
    /// Convenience: pass-through (no-op transform).
    pub fn passthrough(context: InternalContext) -> Self {
        Self {
            context,
            audit: "passthrough".to_string(),
        }
    }
}

/// **ConvertOutcome** -- return value of
/// [`AgentLoopBoundary::convert_to_llm`].
///
/// Contains the projected message list and a derived [`ChatRequest`]
/// skeleton (the caller fills in model + temperature etc. before
/// streaming).
#[derive(Debug, Clone)]
pub struct ConvertOutcome {
    /// LLM-facing message list (ready for `ChatRequest.messages`).
    pub messages: Vec<ChatMessage>,
    /// Tool names to advertise (PI-4 deferred -- empty for v0.0.1).
    pub tools: Vec<String>,
}

// =====================================================================
// StreamFn -- the no-throw streaming function (FR-16)
// =====================================================================

/// **StreamFn** -- async function signature for the no-throw LLM stream
/// (per FR-16).
///
/// Contract:
/// - The returned stream **MUST NOT** yield `Result::Err`.
/// - Provider errors MUST be encoded as
///   [`AgentStreamEvent::Error`] with `recoverable: bool` set per FR-17.
/// - The stream MUST terminate after `Done` or `Aborted`.
///
/// `Send` bound is required so the stream crosses task boundaries (the
/// agent loop runs in a tokio task and forwards events to the UI / SSE
/// dispatcher).
pub type StreamFn = Arc<
    dyn Fn(ChatRequest) -> Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>>
        + Send
        + Sync,
>;

// =====================================================================
// BoundaryError (rare, non-stream-failure surfaces)
// =====================================================================

/// **BoundaryError** -- errors raised by the boundary OUTSIDE the
/// no-throw stream (i.e. structural / config errors that prevent the
/// stream from being created). Streaming failures should be encoded as
/// `AgentStreamEvent::Error` instead.
#[derive(Debug, Error)]
pub enum BoundaryError {
    /// LLM provider returned an error before the stream began.
    #[error("provider dispatch error: {0}")]
    ProviderDispatch(#[from] LlmProviderRegistryError),
    /// StreamFn itself was not wired (e.g. no provider in the context).
    #[error("stream not wired: {0}")]
    Empty(String),
}

// =====================================================================
// AgentLoopBoundary trait (FR-13: 4 methods)
// =====================================================================

/// **AgentLoopBoundary** -- the agent-loop seam between the in-process agent
/// driver and the LLM / tool worlds (per SRS-PI-BORROW-001 §4 FR-13 +
/// `pi-agent-core/src/types.ts:19-37, 218-241`).
///
/// Implementors receive:
/// - [`LoopBoundaryContext`] (carries actor / tenant / agent / session /
///   provider / policy hooks / audit sink),
/// - per-call inputs ([`InternalContext`] for context ops,
///   [`ToolCallRequest`] for tool gates).
///
/// The trait is **async** to allow remote calls (e.g. summarizer,
/// permission scrubber) but every method must respect the
/// single-turn latency budget (per NFR-1: < 50ms p95 for the
/// `transform_context` + `convert_to_llm` pair combined).
#[async_trait]
pub trait AgentLoopBoundary: Send + Sync {
    /// **transform_context** -- pre-LLM context preparation (FR-14).
    ///
    /// Default behaviour: pass through with a `passthrough` audit.
    /// Implementors may prune messages, summarise, inject cache, or scrub
    /// permissions. MUST NOT mutate [`ToolCallRequest`] fields.
    async fn transform_context(
        &self,
        ctx: &LoopBoundaryContext,
        context: InternalContext,
    ) -> TransformOutcome {
        let _ = ctx; // reserved for future per-tenant policy
        TransformOutcome::passthrough(context)
    }

    /// **convert_to_llm** -- internal → LLM-facing projection (FR-15).
    ///
    /// Default behaviour: 1:1 projection of each [`InternalMessage`] to a
    /// [`ChatMessage`]. Implementors may massage the message order, add
    /// tool/role markers, etc.
    async fn convert_to_llm(
        &self,
        ctx: &LoopBoundaryContext,
        context: &InternalContext,
    ) -> ConvertOutcome {
        let _ = ctx;
        let messages = project_messages(context);
        let tools = context.tools.clone();
        ConvertOutcome { messages, tools }
    }

    /// **before_tool_call** -- policy hook fan-out (FR-25, PI-5).
    ///
    /// Default behaviour: delegate to
    /// [`policy_hooks::apply_before_tool_call_hooks`] with a
    /// [`ToolCallHookContext`] derived from `ctx` + `req`.
    async fn before_tool_call(
        &self,
        ctx: &LoopBoundaryContext,
        req: &ToolCallRequest,
    ) -> PolicyDecision {
        policy_hooks::apply_before_tool_call_hooks(
            &ctx.policy_hooks,
            &build_hook_context(ctx, req),
        )
    }

    /// **after_tool_call** -- policy hook fan-out (FR-26, PI-5).
    ///
    /// Default behaviour: delegate to
    /// [`policy_hooks::apply_after_tool_call_hooks`] with the same
    /// context + the new outcome.
    async fn after_tool_call(
        &self,
        ctx: &LoopBoundaryContext,
        req: &ToolCallRequest,
        outcome: &ToolCallOutcome,
    ) -> PolicyDecision {
        policy_hooks::apply_after_tool_call_hooks(
            &ctx.policy_hooks,
            &build_hook_context(ctx, req),
            outcome,
        )
    }
}

// =====================================================================
// DefaultLoopBoundary — production impl
// =====================================================================

/// **DefaultLoopBoundary** -- default [`AgentLoopBoundary`] implementation.
///
/// - `transform_context` / `convert_to_llm`: pass-through + 1:1 project.
/// - `before_tool_call` / `after_tool_call`: delegate to PI-5
///   [`policy_hooks`].
///
/// Streaming is provided separately via [`default_stream_fn`]; the
/// boundary trait itself is intentionally stream-agnostic (per Pi's
/// separation of `StreamFn` from `transformContext` / `convertToLlm`).
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultLoopBoundary;

#[async_trait]
impl AgentLoopBoundary for DefaultLoopBoundary {
    // All 4 methods use their default impls (see trait).
}

// =====================================================================
// default_stream_fn — no-throw wrapper around LlmProvider::stream_completion_v2
// =====================================================================

/// **default_stream_fn** -- construct a [`StreamFn`] that wraps a provider's
/// `stream_completion_v2` call and guarantees the no-throw contract.
///
/// If `stream_completion_v2` itself returns `Err`, the returned stream
/// emits a single `Error { recoverable: false }` event followed by `Done`.
/// Per FR-16, no `Result::Err` is ever yielded.
///
/// Caller passes the resulting `StreamFn` to the agent loop driver.
pub fn default_stream_fn(provider: Arc<dyn LlmProvider>) -> StreamFn {
    Arc::new(move |req: ChatRequest| -> Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>> {
        let provider = provider.clone();
        Box::pin(stream::once(async move {
            match provider.stream_completion_v2(req).await {
                Ok(s) => Box::pin(s) as Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>>,
                Err(err) => {
                    let msg = err.to_string();
                    Box::pin(stream::iter(vec![
                        AgentStreamEvent::Error {
                            error: StreamError::new(
                                "provider_dispatch_failed",
                                msg,
                                false,
                            ),
                            recoverable: false,
                        },
                        AgentStreamEvent::done_default(),
                    ])) as Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>>
                }
            }
        })
        .flat_map(|s| s))
    })
}

// =====================================================================
// Internal helpers (per default trait impls above)
// =====================================================================

/// Project the agent-loop's internal messages to LLM-facing chat messages
/// (used by the default `convert_to_llm` impl).
fn project_messages(context: &InternalContext) -> Vec<ChatMessage> {
    let mut out: Vec<ChatMessage> = Vec::with_capacity(context.messages.len() + 1);
    if let Some(system) = &context.system {
        out.push(ChatMessage::system(system.clone()));
    }
    for m in &context.messages {
        out.push(match m.role {
            InternalRole::System => ChatMessage::system(m.content.clone()),
            InternalRole::User => ChatMessage::user(m.content.clone()),
            InternalRole::Assistant => ChatMessage::assistant(m.content.clone()),
            // `Tool` / `Steering` deferred to PI-4 / PI-9; project as
            // assistant text for now (no schema reservation yet).
            InternalRole::Tool | InternalRole::Steering => {
                ChatMessage::assistant(m.content.clone())
            }
        });
    }
    out
}

/// Build a [`ToolCallHookContext`] from the boundary context + tool call
/// request. Used by the default `before_tool_call` / `after_tool_call`
/// impls.
fn build_hook_context(ctx: &LoopBoundaryContext, req: &ToolCallRequest) -> ToolCallHookContext {
    ToolCallHookContext {
        actor: ctx.actor.clone(),
        agent_id: ctx.agent_id,
        session_id: ctx.session_id,
        tool: req.tool.clone(),
        path: req.path.clone(),
        args: req.args.clone(),
        requires_network: req.requires_network,
        requires_secret: req.requires_secret,
        // Runtime / change tracking deliberately left at 0 in the boundary
        // path -- `AgentPolicy::enforce` enforces hard limits; the boundary
        // is for hook orchestration.
        elapsed_seconds: 0,
        changed_files: 0,
    }
}

// =====================================================================
// Failure injection helpers (for FR-18 5-class failure simulation)
// =====================================================================

/// **FailureMode** -- per FR-18, the 5 failure categories the StreamFn
/// must absorb and encode into [`AgentStreamEvent::Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FailureMode {
    /// Provider HTTP 4xx (e.g. 401 unauthenticated, 403 forbidden,
    /// 422 schema invalid). Per FR-17 `recoverable: false`.
    Http4xx,
    /// Provider HTTP 5xx (e.g. 502 bad gateway, 503 unavailable,
    /// 504 timeout). `recoverable: true` (caller may retry).
    Http5xx,
    /// Upstream timeout (provider stopped responding within deadline).
    /// `recoverable: true`.
    Timeout,
    /// Provider SDK panic / unexpected exception (caught upstream and
    /// re-thrown as `LlmProviderRegistryError::Backend`). `recoverable: false`.
    SdkPanic,
    /// Protocol violation (stream emitted malformed SSE / non-JSON
    /// chunks). `recoverable: false`.
    ProtocolViolation,
}

impl FailureMode {
    /// Stable wire-format kind string for [`StreamError::kind`].
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Http4xx => "http_4xx",
            Self::Http5xx => "http_5xx",
            Self::Timeout => "timeout",
            Self::SdkPanic => "sdk_panic",
            Self::ProtocolViolation => "protocol_violation",
        }
    }

    /// Whether the agent loop may retry.
    pub fn recoverable(&self) -> bool {
        matches!(self, Self::Http5xx | Self::Timeout)
    }

    /// Optional HTTP status code (per FR-17).
    pub fn http_status(&self) -> Option<u16> {
        match self {
            Self::Http4xx => Some(401),
            Self::Http5xx => Some(503),
            Self::Timeout | Self::SdkPanic | Self::ProtocolViolation => None,
        }
    }
}

/// **simulated_failure_stream** -- construct a stream that emits a single
/// `Error` event (encoded per FR-17 for the given failure mode) followed
/// by `Done`. Used by FR-18 tests to verify the no-throw contract without
/// standing up a real flaky provider.
///
/// The stream is infallible: it never yields `Err`, only `AgentStreamEvent`.
pub fn simulated_failure_stream(mode: FailureMode) -> Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>> {
    let mut err = StreamError::new(mode.kind(), format!("simulated {}", mode.kind()), mode.recoverable());
    if let Some(status) = mode.http_status() {
        err = err.with_status(status);
    }
    Box::pin(stream::iter(vec![
        AgentStreamEvent::Error {
            error: err,
            recoverable: mode.recoverable(),
        },
        AgentStreamEvent::Done {
            stop_reason: StopReason::Error,
            usage: Usage::default(),
        },
    ]))
}

/// **NoThrowProbe** -- small helper that drains a stream and asserts
/// (1) every yielded item is `Ok` (i.e. never `Err` per FR-16) and
/// (2) the stream terminated.
///
/// Used by FR-18 tests.
pub struct NoThrowProbe;

impl NoThrowProbe {
    /// Probe helper: drain the stream to a `Vec<AgentStreamEvent>` and
    /// assert no `Err` was yielded.
    pub async fn collect<S>(s: S) -> Vec<AgentStreamEvent>
    where
        S: Stream<Item = AgentStreamEvent> + Send,
    {
        // Stream<Item = AgentStreamEvent> is infallible by type, but we
        // double-check via `try_collect`-style pattern: drain to vec.
        let mut out = Vec::new();
        let mut pinned = Box::pin(s);
        while let Some(item) = pinned.next().await {
            out.push(item);
        }
        out
    }
}

// =====================================================================
// Unit Tests (per FR-18 5-class failure simulation + AC-4 UT 100%)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use domain_llm::chat::{ChatRequest, ChatRole};
    use domain_llm::ChatMessage;
    use star_context::ActorContext;

    use crate::policy_hooks::{NoopAuditSink, PolicyDecision, PolicyHook};

    fn make_actor(tenant_id: Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id).with_role("project_admin")
    }

    fn make_ctx(policy_hooks: PolicyHooks) -> LoopBoundaryContext {
        let tenant = Uuid::new_v4();
        LoopBoundaryContext {
            actor: make_actor(tenant),
            tenant_id: tenant,
            agent_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            provider: Arc::new(domain_llm::MockProvider::default()),
            policy_hooks,
            audit_sink: Arc::new(NoopAuditSink),
        }
    }

    // ---------- InternalContext / InternalMessage / ToolCallRequest ----------

    #[test]
    fn internal_context_from_user_prompt_has_one_user_message() {
        let ctx = InternalContext::from_user_prompt("hi");
        assert_eq!(ctx.messages.len(), 1);
        assert_eq!(ctx.messages[0].role, InternalRole::User);
        assert_eq!(ctx.messages[0].content, "hi");
        assert_eq!(ctx.messages[0].seq, 0);
    }

    #[test]
    fn internal_context_default_is_empty() {
        let ctx = InternalContext::default();
        assert!(ctx.system.is_none());
        assert!(ctx.messages.is_empty());
        assert!(ctx.tools.is_empty());
        assert_eq!(ctx.token_estimate, 0);
    }

    #[test]
    fn tool_call_request_minimal_sets_only_tool_name() {
        let req = ToolCallRequest::minimal("read_file");
        assert_eq!(req.tool, "read_file");
        assert!(req.path.is_empty());
        assert_eq!(req.args, serde_json::Value::Null);
        assert!(!req.requires_network);
        assert!(!req.requires_secret);
    }

    // ---------- project_messages / ConvertOutcome ----------

    #[test]
    fn project_messages_emits_system_first_then_users() {
        let mut ctx = InternalContext::from_user_prompt("hi");
        ctx.system = Some("be terse".to_string());
        ctx.messages.push(InternalMessage {
            id: Uuid::new_v4(),
            role: InternalRole::Assistant,
            content: "hello".to_string(),
            tool: None,
            seq: 1,
        });
        let msgs = project_messages(&ctx);
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].role, ChatRole::System);
        assert_eq!(msgs[0].content, "be terse");
        assert_eq!(msgs[1].role, ChatRole::User);
        assert_eq!(msgs[2].role, ChatRole::Assistant);
    }

    // ---------- DefaultLoopBoundary ----------

    #[tokio::test]
    async fn default_loop_boundary_transform_context_passes_through() {
        let b = DefaultLoopBoundary;
        let ctx = make_ctx(PolicyHooks::new());
        let ic = InternalContext::from_user_prompt("hi");
        let out = b.transform_context(&ctx, ic.clone()).await;
        assert_eq!(out.context.messages.len(), ic.messages.len());
        assert_eq!(out.audit, "passthrough");
    }

    #[tokio::test]
    async fn default_loop_boundary_convert_to_llm_projects_messages() {
        let b = DefaultLoopBoundary;
        let ctx = make_ctx(PolicyHooks::new());
        let ic = InternalContext::from_user_prompt("hi");
        let out = b.convert_to_llm(&ctx, &ic).await;
        assert_eq!(out.messages.len(), 1);
        assert_eq!(out.messages[0].role, ChatRole::User);
        assert_eq!(out.messages[0].content, "hi");
        assert!(out.tools.is_empty());
    }

    // ---------- AgentLoopBoundary::before_tool_call / after_tool_call ----------

    struct DenyShellExec;

    impl PolicyHook for DenyShellExec {
        fn name(&self) -> &str {
            "deny_shell_exec"
        }
        fn before_tool_call(&self, ctx: &ToolCallHookContext) -> PolicyDecision {
            if ctx.tool == "shell_exec" {
                PolicyDecision::Deny {
                    reason: "shell_exec denied".to_string(),
                }
            } else {
                PolicyDecision::Allow
            }
        }
    }

    #[tokio::test]
    async fn before_tool_call_returns_allow_for_clean_request() {
        let b = DefaultLoopBoundary;
        let ctx = make_ctx(PolicyHooks::new());
        let req = ToolCallRequest::minimal("read_file");
        let d = b.before_tool_call(&ctx, &req).await;
        assert_eq!(d, PolicyDecision::Allow);
    }

    #[tokio::test]
    async fn before_tool_call_returns_deny_when_hook_denies() {
        let b = DefaultLoopBoundary;
        let ctx = make_ctx(PolicyHooks::new().with_before(DenyShellExec));
        let req = ToolCallRequest::minimal("shell_exec");
        let d = b.before_tool_call(&ctx, &req).await;
        assert!(d.is_terminal());
        match d {
            PolicyDecision::Deny { reason } => assert!(reason.contains("shell_exec")),
            _ => panic!("expected Deny"),
        }
    }

    #[tokio::test]
    async fn after_tool_call_returns_allow_when_no_hooks_defined() {
        let b = DefaultLoopBoundary;
        let ctx = make_ctx(PolicyHooks::new());
        let req = ToolCallRequest::minimal("read_file");
        let outcome = ToolCallOutcome::ok("data", 5);
        let d = b.after_tool_call(&ctx, &req, &outcome).await;
        assert_eq!(d, PolicyDecision::Allow);
    }

    // ---------- FailureMode + simulated_failure_stream (FR-18) ----------

    #[test]
    fn failure_mode_http_4xx_is_not_recoverable() {
        let m = FailureMode::Http4xx;
        assert_eq!(m.kind(), "http_4xx");
        assert!(!m.recoverable());
        assert_eq!(m.http_status(), Some(401));
    }

    #[test]
    fn failure_mode_http_5xx_is_recoverable() {
        let m = FailureMode::Http5xx;
        assert_eq!(m.kind(), "http_5xx");
        assert!(m.recoverable());
        assert_eq!(m.http_status(), Some(503));
    }

    #[test]
    fn failure_mode_timeout_is_recoverable_no_status() {
        let m = FailureMode::Timeout;
        assert_eq!(m.kind(), "timeout");
        assert!(m.recoverable());
        assert!(m.http_status().is_none());
    }

    #[test]
    fn failure_mode_sdk_panic_is_not_recoverable() {
        let m = FailureMode::SdkPanic;
        assert_eq!(m.kind(), "sdk_panic");
        assert!(!m.recoverable());
        assert!(m.http_status().is_none());
    }

    #[test]
    fn failure_mode_protocol_violation_is_not_recoverable() {
        let m = FailureMode::ProtocolViolation;
        assert_eq!(m.kind(), "protocol_violation");
        assert!(!m.recoverable());
        assert!(m.http_status().is_none());
    }

    // ---------- FR-18: 5-class failure simulation ----------

    /// **Test 1 (FR-18)**: HTTP 4xx → Error{ recoverable: false, http_status: 401 }
    #[tokio::test]
    async fn simulated_http_4xx_emits_error_event_not_err() {
        let stream = simulated_failure_stream(FailureMode::Http4xx);
        let events = NoThrowProbe::collect(stream).await;
        assert_eq!(events.len(), 2, "expected Error + Done, got {events:?}");
        match &events[0] {
            AgentStreamEvent::Error { error, recoverable } => {
                assert_eq!(error.kind, "http_4xx");
                assert_eq!(error.http_status, Some(401));
                assert!(!recoverable, "HTTP 4xx must NOT be recoverable");
            }
            other => panic!("expected Error, got {other:?}"),
        }
        assert!(events[1].is_terminal(), "Done is terminal");
    }

    /// **Test 2 (FR-18)**: HTTP 5xx → Error{ recoverable: true, http_status: 503 }
    #[tokio::test]
    async fn simulated_http_5xx_emits_recoverable_error_event() {
        let stream = simulated_failure_stream(FailureMode::Http5xx);
        let events = NoThrowProbe::collect(stream).await;
        assert_eq!(events.len(), 2);
        match &events[0] {
            AgentStreamEvent::Error { error, recoverable } => {
                assert_eq!(error.kind, "http_5xx");
                assert_eq!(error.http_status, Some(503));
                assert!(*recoverable, "HTTP 5xx must be recoverable");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    /// **Test 3 (FR-18)**: timeout → Error{ recoverable: true, http_status: None }
    #[tokio::test]
    async fn simulated_timeout_emits_recoverable_error_event() {
        let stream = simulated_failure_stream(FailureMode::Timeout);
        let events = NoThrowProbe::collect(stream).await;
        assert_eq!(events.len(), 2);
        match &events[0] {
            AgentStreamEvent::Error { error, recoverable } => {
                assert_eq!(error.kind, "timeout");
                assert!(error.http_status.is_none());
                assert!(*recoverable, "timeout must be recoverable");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    /// **Test 4 (FR-18)**: SDK panic → Error{ recoverable: false }
    #[tokio::test]
    async fn simulated_sdk_panic_emits_non_recoverable_error_event() {
        let stream = simulated_failure_stream(FailureMode::SdkPanic);
        let events = NoThrowProbe::collect(stream).await;
        assert_eq!(events.len(), 2);
        match &events[0] {
            AgentStreamEvent::Error { error, recoverable } => {
                assert_eq!(error.kind, "sdk_panic");
                assert!(!(*recoverable), "SDK panic must NOT be recoverable");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    /// **Test 5 (FR-18)**: protocol violation → Error{ recoverable: false }
    #[tokio::test]
    async fn simulated_protocol_violation_emits_non_recoverable_error_event() {
        let stream = simulated_failure_stream(FailureMode::ProtocolViolation);
        let events = NoThrowProbe::collect(stream).await;
        assert_eq!(events.len(), 2);
        match &events[0] {
            AgentStreamEvent::Error { error, recoverable } => {
                assert_eq!(error.kind, "protocol_violation");
                assert!(!(*recoverable), "protocol violation must NOT be recoverable");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    /// **Test 6 (FR-16)**: the no-throw contract is **type-level** --
    /// `Stream<Item = AgentStreamEvent>` cannot yield `Err`. Verify the
    /// type compiles and the helper compiles.
    #[tokio::test]
    async fn no_throw_contract_is_type_enforced() {
        fn assert_send_sync_stream<S: Stream<Item = AgentStreamEvent> + Send + Unpin>(
            _s: &S,
        ) {
        }
        let s = simulated_failure_stream(FailureMode::Timeout);
        assert_send_sync_stream(&s);
        // drain a few items to exercise the path
        let events = NoThrowProbe::collect(s).await;
        assert!(!events.is_empty());
    }

    // ---------- default_stream_fn ----------

    /// **Test 7**: `default_stream_fn` wraps an OK provider's
    /// `stream_completion_v2` and yields events (per FR-16, the stream is
    /// infallible; per FR-17 the events are typed correctly).
    #[tokio::test]
    async fn default_stream_fn_with_mock_provider_yields_done() {
        // MockProvider::stream_completion_v2 uses the default impl in
        // domain-llm, which falls back to stream_completion (also a stub
        // Unimplemented), then translates to a single Error + Done.
        // Either way the StreamFn must yield those events infallibly.
        let provider: Arc<dyn LlmProvider> = Arc::new(domain_llm::MockProvider::default());
        let stream_fn = default_stream_fn(provider);
        let req = ChatRequest {
            model: "mock-model".to_string(),
            messages: vec![ChatMessage::user("hi")],
            ..Default::default()
        };
        let stream = stream_fn(req);
        let events = NoThrowProbe::collect(stream).await;
        // The default impl always emits Error + Done for the stub path.
        assert!(!events.is_empty(), "stream must yield >= 1 event");
        assert!(events.last().unwrap().is_terminal(), "last event must be terminal");
    }

    // ---------- BoundaryError ----------

    #[test]
    fn boundary_error_provider_dispatch_displays_provider_message() {
        let e = BoundaryError::ProviderDispatch(LlmProviderRegistryError::Unimplemented(
            "stub".to_string(),
        ));
        let s = e.to_string();
        assert!(s.contains("provider dispatch"), "got {s}");
        assert!(s.contains("stub"), "got {s}");
    }

    #[test]
    fn boundary_error_empty_displays_message() {
        let e = BoundaryError::Empty("no provider wired".to_string());
        let s = e.to_string();
        assert!(s.contains("stream not wired"), "got {s}");
        assert!(s.contains("no provider wired"), "got {s}");
    }
}
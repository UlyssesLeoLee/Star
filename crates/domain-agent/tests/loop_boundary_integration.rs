//! domain-agent loop_boundary (PI-3) + policy_hooks (PI-5) 集成测试 (WBS 9.6)
//!
//! 覆盖:
//! - InternalContext / InternalMessage / InternalRole / ToolCallRequest / TransformOutcome
//! - DefaultLoopBoundary 的 4 个 trait 方法 (经 MockProvider)
//! - FailureMode kind/recoverable/http_status + simulated_failure_stream + NoThrowProbe
//! - PolicyDecision / AuditEventSpec / PolicyHooks 容器 + apply_* 自由函数
//! - ToolCallHookContext / ToolCallOutcome
//! - BoundaryError Display

use std::sync::Arc;

use domain_agent::{
    apply_after_tool_call_hooks, apply_before_tool_call_hooks, AgentLoopBoundary, AuditEventSpec,
    BoundaryError, DefaultLoopBoundary, FailureMode, InternalContext, InternalMessage,
    InternalRole, NoThrowProbe, NoopAuditSink, PolicyDecision, PolicyHook, PolicyHooks,
    ToolCallHookContext, ToolCallOutcome, ToolCallRequest, TransformOutcome,
};
use domain_llm::{
    AgentStreamEvent, ChatRequest, ChatRole, LlmProviderRegistryError, MockProvider, StopReason,
    StreamError, Usage,
};
use futures_util::StreamExt;
use star_context::ActorContext;
use uuid::Uuid;

fn make_ctx(policy_hooks: PolicyHooks) -> domain_agent::LoopBoundaryContext {
    let tenant = Uuid::new_v4();
    domain_agent::LoopBoundaryContext {
        actor: ActorContext::new(Uuid::new_v4(), tenant).with_role("project_admin"),
        tenant_id: tenant,
        agent_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        provider: Arc::new(MockProvider::default()),
        policy_hooks,
        audit_sink: Arc::new(NoopAuditSink),
    }
}

// =====================================================================
// InternalContext / InternalMessage / ToolCallRequest
// =====================================================================

#[test]
fn internal_context_from_user_prompt_shape() {
    let ctx = InternalContext::from_user_prompt("hello");
    assert_eq!(ctx.messages.len(), 1);
    assert_eq!(ctx.messages[0].role, InternalRole::User);
    assert_eq!(ctx.messages[0].content, "hello");
    assert_eq!(ctx.messages[0].seq, 0);
    assert!(ctx.messages[0].tool.is_none());
    assert_ne!(ctx.messages[0].id, Uuid::nil());
    assert!(ctx.system.is_none());
    assert!(ctx.tools.is_empty());
    assert_eq!(ctx.token_estimate, 0);
}

#[test]
fn internal_context_and_message_serde_roundtrip() {
    let mut ctx = InternalContext::from_user_prompt("hi");
    ctx.system = Some("be terse".into());
    ctx.tools = vec!["read_file".into()];
    ctx.token_estimate = 42;
    ctx.messages.push(InternalMessage {
        id: Uuid::new_v4(),
        role: InternalRole::Assistant,
        content: "ok".into(),
        tool: None,
        seq: 1,
    });
    let json = serde_json::to_string(&ctx).unwrap();
    let parsed: InternalContext = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.system.as_deref(), Some("be terse"));
    assert_eq!(parsed.messages.len(), 2);
    assert_eq!(parsed.messages[1].role, InternalRole::Assistant);
    assert_eq!(parsed.token_estimate, 42);
}

#[test]
fn internal_role_serde_all_five() {
    let roles = [
        InternalRole::System,
        InternalRole::User,
        InternalRole::Assistant,
        InternalRole::Tool,
        InternalRole::Steering,
    ];
    for role in roles {
        let json = serde_json::to_string(&role).unwrap();
        let parsed: InternalRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, role);
    }
}

#[test]
fn tool_call_request_minimal_and_full_serde() {
    let req = ToolCallRequest::minimal("edit");
    assert_eq!(req.tool, "edit");
    assert!(req.path.is_empty());
    assert_eq!(req.args, serde_json::Value::Null);
    assert!(!req.requires_network && !req.requires_secret);

    let req = ToolCallRequest {
        tool: "bash".into(),
        path: "/tmp".into(),
        args: serde_json::json!({"cmd": "ls"}),
        requires_network: true,
        requires_secret: true,
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: ToolCallRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.tool, "bash");
    assert!(parsed.requires_network && parsed.requires_secret);
    assert_eq!(parsed.args["cmd"], "ls");
}

#[test]
fn transform_outcome_passthrough() {
    let ic = InternalContext::from_user_prompt("x");
    let out = TransformOutcome::passthrough(ic.clone());
    assert_eq!(out.context.messages.len(), ic.messages.len());
    assert_eq!(out.audit, "passthrough");
    let json = serde_json::to_string(&out).unwrap();
    let parsed: TransformOutcome = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.audit, "passthrough");
}

// =====================================================================
// DefaultLoopBoundary 4 方法
// =====================================================================

#[tokio::test]
async fn boundary_transform_passthrough() {
    let b = DefaultLoopBoundary;
    let ctx = make_ctx(PolicyHooks::new());
    let ic = InternalContext::from_user_prompt("hi");
    let out = b.transform_context(&ctx, ic.clone()).await;
    assert_eq!(out.context.messages.len(), ic.messages.len());
    assert_eq!(out.audit, "passthrough");
}

#[tokio::test]
async fn boundary_convert_projects_system_and_tool_messages() {
    let b = DefaultLoopBoundary;
    let ctx = make_ctx(PolicyHooks::new());
    let mut ic = InternalContext::from_user_prompt("hi");
    ic.system = Some("sys".into());
    ic.tools = vec!["t1".into()];
    ic.messages.push(InternalMessage {
        id: Uuid::new_v4(),
        role: InternalRole::Tool,
        content: "result".into(),
        tool: Some("t1".into()),
        seq: 1,
    });
    let out = b.convert_to_llm(&ctx, &ic).await;
    // system + user + tool → 3 条 (Tool 投影为 assistant)
    assert_eq!(out.messages.len(), 3);
    assert_eq!(out.messages[0].role, ChatRole::System);
    assert_eq!(out.messages[1].role, ChatRole::User);
    assert_eq!(out.messages[2].role, ChatRole::Assistant);
    assert_eq!(out.tools, vec!["t1".to_string()]);
}

// =====================================================================
// FailureMode / simulated_failure_stream / NoThrowProbe
// =====================================================================

#[test]
fn failure_mode_metadata() {
    let cases = [
        (FailureMode::Http4xx, "http_4xx", false, Some(401u16)),
        (FailureMode::Http5xx, "http_5xx", true, Some(503)),
        (FailureMode::Timeout, "timeout", true, None),
        (FailureMode::SdkPanic, "sdk_panic", false, None),
        (
            FailureMode::ProtocolViolation,
            "protocol_violation",
            false,
            None,
        ),
    ];
    for (mode, kind, recoverable, status) in cases {
        assert_eq!(mode.kind(), kind);
        assert_eq!(mode.recoverable(), recoverable, "{kind} recoverable");
        assert_eq!(mode.http_status(), status);
        // serde
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: FailureMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, mode);
    }
}

#[tokio::test]
async fn simulated_failure_stream_emits_error_then_done() {
    let events = NoThrowProbe::collect(simulated_http4xx_stream()).await;
    assert_eq!(events.len(), 2);
    match &events[0] {
        AgentStreamEvent::Error { error, recoverable } => {
            assert_eq!(error.kind, "http_4xx");
            assert_eq!(*recoverable, false);
            assert_eq!(error.http_status, Some(401));
        }
        other => panic!("expected Error, got {other:?}"),
    }
    match &events[1] {
        AgentStreamEvent::Done { stop_reason, .. } => {
            assert_eq!(*stop_reason, StopReason::Error);
        }
        other => panic!("expected Done, got {other:?}"),
    }
}

fn simulated_http4xx_stream(
) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = AgentStreamEvent> + Send>> {
    domain_agent::simulated_failure_stream(FailureMode::Http4xx)
}

#[tokio::test]
async fn simulated_failure_stream_recoverable_timeout() {
    let mut stream = domain_agent::simulated_failure_stream(FailureMode::Timeout);
    match stream.next().await {
        Some(AgentStreamEvent::Error { recoverable, .. }) => assert!(recoverable),
        other => panic!("expected Error, got {other:?}"),
    }
    match stream.next().await {
        Some(AgentStreamEvent::Done { .. }) => {}
        other => panic!("expected Done, got {other:?}"),
    }
    assert!(stream.next().await.is_none(), "stream terminates");
}

#[test]
fn boundary_error_display() {
    let e = BoundaryError::Empty("no provider".into());
    assert_eq!(e.to_string(), "stream not wired: no provider");
    // ProviderDispatch via From
    let inner = LlmProviderRegistryError::Backend("backend down".into());
    let e = BoundaryError::from(inner);
    assert!(e.to_string().contains("provider dispatch error"));
}

// =====================================================================
// PolicyDecision / AuditEventSpec
// =====================================================================

#[test]
fn policy_decision_flags_and_serde() {
    let allow = PolicyDecision::Allow;
    assert!(!allow.is_terminal() && !allow.requires_retry() && !allow.requires_audit());

    let deny = PolicyDecision::Deny {
        reason: "nope".into(),
    };
    assert!(deny.is_terminal());
    assert!(!deny.requires_retry());

    let modify = PolicyDecision::Modify {
        args: serde_json::json!({"x": 1}),
    };
    assert!(modify.requires_retry());

    let audit = PolicyDecision::Audit {
        event: AuditEventSpec::new(Uuid::new_v4(), Uuid::new_v4(), "tool_call_denied"),
    };
    assert!(audit.requires_audit());

    // serde: tag = "kind", snake_case
    let json = serde_json::to_string(&deny).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["kind"], "deny");
    assert_eq!(v["reason"], "nope");
    let parsed: PolicyDecision = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, PolicyDecision::Deny { .. }));

    let json = serde_json::to_string(&modify).unwrap();
    let parsed: PolicyDecision = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, PolicyDecision::Modify { .. }));
}

#[test]
fn audit_event_spec_builder_chain() {
    let agent = Uuid::new_v4();
    let actor = Uuid::new_v4();
    let session = Uuid::new_v4();
    let spec = AuditEventSpec::new(agent, actor, "tool_call_modified")
        .with_tool("edit")
        .with_session(session)
        .with_context(serde_json::json!({"path": "/a/b"}));
    assert_eq!(spec.agent_id, agent);
    assert_eq!(spec.actor_id, actor);
    assert_eq!(spec.action, "tool_call_modified");
    assert_eq!(spec.tool.as_deref(), Some("edit"));
    assert_eq!(spec.session_id, Some(session));
    assert_eq!(spec.context["path"], "/a/b");

    // serde
    let json = serde_json::to_string(&spec).unwrap();
    let parsed: AuditEventSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, spec);
}

// =====================================================================
// ToolCallHookContext / ToolCallOutcome
// =====================================================================

#[test]
fn tool_call_hook_context_minimal() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let ctx = ToolCallHookContext::minimal(actor, "read_file");
    assert_eq!(ctx.tool, "read_file");
    assert_eq!(ctx.args, serde_json::Value::Null);
    assert!(!ctx.requires_network && !ctx.requires_secret);
    assert_eq!(ctx.elapsed_seconds, 0);
    assert_eq!(ctx.changed_files, 0);
}

#[test]
fn tool_call_outcome_ok_and_err() {
    let ok = ToolCallOutcome::ok("data", 120);
    assert!(ok.success);
    assert_eq!(ok.result, "data");
    assert_eq!(ok.elapsed_ms, 120);

    let err = ToolCallOutcome::err("boom", 5);
    assert!(!err.success);
    assert_eq!(err.error.as_deref(), Some("boom"));
    assert_eq!(err.elapsed_ms, 5);
}

// =====================================================================
// PolicyHooks 容器 + apply_* 自由函数
// =====================================================================

struct DenyShell;

impl PolicyHook for DenyShell {
    fn name(&self) -> &str {
        "deny_shell"
    }
    fn before_tool_call(&self, ctx: &ToolCallHookContext) -> PolicyDecision {
        if ctx.tool == "shell" {
            PolicyDecision::Deny {
                reason: "shell not allowed".into(),
            }
        } else {
            PolicyDecision::Allow
        }
    }
}

struct AuditEveryTool;

impl PolicyHook for AuditEveryTool {
    fn name(&self) -> &str {
        "audit_every_tool"
    }
    fn after_tool_call(
        &self,
        _ctx: &ToolCallHookContext,
        _outcome: &ToolCallOutcome,
    ) -> PolicyDecision {
        PolicyDecision::Audit {
            event: AuditEventSpec::new(Uuid::new_v4(), Uuid::new_v4(), "tool_call_done"),
        }
    }
}

#[test]
fn policy_hooks_container_counts() {
    let hooks = PolicyHooks::new();
    assert!(hooks.is_empty());
    assert_eq!((hooks.before_len(), hooks.after_len()), (0, 0));

    let hooks = PolicyHooks::new()
        .with_before(DenyShell)
        .with_after(AuditEveryTool);
    assert!(!hooks.is_empty());
    assert_eq!((hooks.before_len(), hooks.after_len()), (1, 1));
}

#[test]
fn apply_before_hooks_deny_wins() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let hooks = PolicyHooks::new().with_before(DenyShell);

    let shell_ctx = ToolCallHookContext::minimal(actor.clone(), "shell");
    let d = apply_before_tool_call_hooks(&hooks, &shell_ctx);
    assert!(d.is_terminal());
    match d {
        PolicyDecision::Deny { reason } => assert!(reason.contains("shell")),
        other => panic!("expected Deny, got {other:?}"),
    }

    let safe_ctx = ToolCallHookContext::minimal(actor, "read_file");
    let d = apply_before_tool_call_hooks(&hooks, &safe_ctx);
    assert_eq!(d, PolicyDecision::Allow);
}

#[test]
fn apply_after_hooks_returns_audit() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let hooks = PolicyHooks::new().with_after(AuditEveryTool);
    let ctx = ToolCallHookContext::minimal(actor, "read_file");
    let outcome = ToolCallOutcome::ok("ok", 1);
    let d = apply_after_tool_call_hooks(&hooks, &ctx, &outcome);
    assert!(d.requires_audit());

    // 无 after hooks → Allow
    let empty = PolicyHooks::new();
    let d = apply_after_tool_call_hooks(&empty, &ctx, &outcome);
    assert_eq!(d, PolicyDecision::Allow);
}

#[tokio::test]
async fn boundary_delegates_to_hooks() {
    let b = DefaultLoopBoundary;
    let ctx = make_ctx(PolicyHooks::new().with_before(DenyShell));

    let req = ToolCallRequest::minimal("shell");
    let d = b.before_tool_call(&ctx, &req).await;
    assert!(d.is_terminal());

    let req = ToolCallRequest::minimal("read_file");
    let d = b.before_tool_call(&ctx, &req).await;
    assert_eq!(d, PolicyDecision::Allow);

    let outcome = ToolCallOutcome::ok("fine", 2);
    let d = b.after_tool_call(&ctx, &req, &outcome).await;
    assert_eq!(d, PolicyDecision::Allow, "no after hooks → Allow");
}

// =====================================================================
// MockProvider 经 default_stream_fn 的 no-throw 契约
// =====================================================================

#[tokio::test]
async fn default_stream_fn_wraps_mock_provider() {
    let provider = Arc::new(MockProvider::default());
    let stream_fn = domain_agent::default_stream_fn(provider);
    let req = ChatRequest::default();
    let stream = stream_fn(req);
    let events = NoThrowProbe::collect(stream).await;
    // MockProvider 默认流: Done (或空); 不产生 Err (类型级保证)
    assert!(
        events.is_empty() || matches!(events.last(), Some(AgentStreamEvent::Done { .. })),
        "stream must end with Done or be empty, got {events:?}"
    );
}

#[test]
fn stream_error_status_builder() {
    let err = StreamError::new("x", "msg", true).with_status(500);
    assert_eq!(err.http_status, Some(500));
    assert_eq!(err.kind, "x");
    assert_eq!(err.message, "msg");
    assert!(err.recoverable);
    let default_usage = Usage::default();
    assert_eq!(
        (default_usage.input_tokens, default_usage.output_tokens),
        (0, 0)
    );
    assert_eq!(default_usage.cost_usd, 0.0);
}

//! domain-agent 14 状态机 + ID/实体 serde 集成测试 (WBS 9.6)
//!
//! 覆盖 lib.rs 公共 API:
//! - check_status_transition 全允许迁移表 + 非法迁移
//! - AgentSessionStatus as_str / is_terminal / is_active / triggers_notification
//! - AgentType as_str
//! - TenantId + define_uuid_id! ID 的 serde / Display / From
//! - Agent / AgentSession serde 往返

use domain_agent::{
    check_status_transition, Agent, AgentError, AgentId, AgentSession, AgentSessionId,
    AgentSessionStatus, AgentType, ProjectId, TenantId, UserId, WorkItemId, WorktreeId,
};
use uuid::Uuid;

// =====================================================================
// 14 状态机迁移表
// =====================================================================

const ALLOWED: [(AgentSessionStatus, AgentSessionStatus); 28] = [
    // 启动序列
    (AgentSessionStatus::Created, AgentSessionStatus::Starting),
    (AgentSessionStatus::Starting, AgentSessionStatus::Running),
    (AgentSessionStatus::Starting, AgentSessionStatus::Crashed),
    // Tool 循环
    (AgentSessionStatus::Running, AgentSessionStatus::WaitingTool),
    (
        AgentSessionStatus::WaitingTool,
        AgentSessionStatus::ToolRunning,
    ),
    (
        AgentSessionStatus::ToolRunning,
        AgentSessionStatus::ToolCompleted,
    ),
    (
        AgentSessionStatus::ToolCompleted,
        AgentSessionStatus::Running,
    ),
    (AgentSessionStatus::ToolRunning, AgentSessionStatus::Running),
    (AgentSessionStatus::ToolRunning, AgentSessionStatus::Crashed),
    // Feedback 循环
    (
        AgentSessionStatus::Running,
        AgentSessionStatus::WaitingFeedback,
    ),
    (
        AgentSessionStatus::WaitingFeedback,
        AgentSessionStatus::FeedbackReceived,
    ),
    (
        AgentSessionStatus::FeedbackReceived,
        AgentSessionStatus::Running,
    ),
    // 验证
    (AgentSessionStatus::Running, AgentSessionStatus::Validating),
    (
        AgentSessionStatus::Validating,
        AgentSessionStatus::Completed,
    ),
    (AgentSessionStatus::Validating, AgentSessionStatus::Failed),
    // 异常路径
    (AgentSessionStatus::Running, AgentSessionStatus::Failed),
    (AgentSessionStatus::Running, AgentSessionStatus::Aborted),
    (AgentSessionStatus::Running, AgentSessionStatus::Crashed),
    (AgentSessionStatus::Running, AgentSessionStatus::Timeout),
    (AgentSessionStatus::Starting, AgentSessionStatus::Failed),
    (AgentSessionStatus::Starting, AgentSessionStatus::Aborted),
    (AgentSessionStatus::Starting, AgentSessionStatus::Timeout),
    (AgentSessionStatus::WaitingTool, AgentSessionStatus::Aborted),
    (AgentSessionStatus::WaitingTool, AgentSessionStatus::Timeout),
    (AgentSessionStatus::ToolRunning, AgentSessionStatus::Aborted),
    (
        AgentSessionStatus::WaitingFeedback,
        AgentSessionStatus::Aborted,
    ),
    (
        AgentSessionStatus::WaitingFeedback,
        AgentSessionStatus::Timeout,
    ),
    (AgentSessionStatus::Validating, AgentSessionStatus::Aborted),
];

#[test]
fn all_allowed_transitions_pass() {
    for (from, to) in ALLOWED {
        assert!(
            check_status_transition(from, to).is_ok(),
            "{from:?} -> {to:?} should be allowed"
        );
    }
}

#[test]
fn invalid_transitions_rejected_with_both_states() {
    let invalid = [
        // 终态不可再迁移
        (AgentSessionStatus::Completed, AgentSessionStatus::Running),
        (AgentSessionStatus::Failed, AgentSessionStatus::Running),
        (AgentSessionStatus::Aborted, AgentSessionStatus::Running),
        (AgentSessionStatus::Crashed, AgentSessionStatus::Running),
        (AgentSessionStatus::Timeout, AgentSessionStatus::Running),
        // Created 只能 Starting
        (AgentSessionStatus::Created, AgentSessionStatus::Running),
        (AgentSessionStatus::Created, AgentSessionStatus::Completed),
        // 跳级
        (AgentSessionStatus::Created, AgentSessionStatus::Completed),
        (AgentSessionStatus::Starting, AgentSessionStatus::Completed),
        (AgentSessionStatus::Running, AgentSessionStatus::Completed),
        (
            AgentSessionStatus::WaitingTool,
            AgentSessionStatus::Completed,
        ),
        // 反向
        (AgentSessionStatus::Running, AgentSessionStatus::Starting),
        (AgentSessionStatus::Running, AgentSessionStatus::Created),
        (
            AgentSessionStatus::ToolCompleted,
            AgentSessionStatus::WaitingTool,
        ),
        (
            AgentSessionStatus::FeedbackReceived,
            AgentSessionStatus::WaitingFeedback,
        ),
        // 自环 (非允许表中)
        (AgentSessionStatus::Created, AgentSessionStatus::Created),
        (AgentSessionStatus::Running, AgentSessionStatus::Running),
        (
            AgentSessionStatus::Validating,
            AgentSessionStatus::Validating,
        ),
    ];
    for (from, to) in invalid {
        let err = check_status_transition(from, to).unwrap_err();
        assert!(
            matches!(err, AgentError::InvalidTransition { .. }),
            "{from:?} -> {to:?} must be InvalidTransition"
        );
        // 错误带 from/to 字符串
        let s = err.to_string();
        assert!(s.contains("invalid state transition"), "{s}");
        assert!(s.contains(from.as_str()), "{s} missing from");
        assert!(s.contains(to.as_str()), "{s} missing to");
    }
}

#[test]
fn status_as_str_all_14() {
    let cases = [
        (AgentSessionStatus::Created, "CREATED"),
        (AgentSessionStatus::Starting, "STARTING"),
        (AgentSessionStatus::Running, "RUNNING"),
        (AgentSessionStatus::WaitingTool, "WAITING_TOOL"),
        (AgentSessionStatus::ToolRunning, "TOOL_RUNNING"),
        (AgentSessionStatus::ToolCompleted, "TOOL_COMPLETED"),
        (AgentSessionStatus::WaitingFeedback, "WAITING_FEEDBACK"),
        (AgentSessionStatus::FeedbackReceived, "FEEDBACK_RECEIVED"),
        (AgentSessionStatus::Validating, "VALIDATING"),
        (AgentSessionStatus::Completed, "COMPLETED"),
        (AgentSessionStatus::Failed, "FAILED"),
        (AgentSessionStatus::Aborted, "ABORTED"),
        (AgentSessionStatus::Crashed, "CRASHED"),
        (AgentSessionStatus::Timeout, "TIMEOUT"),
    ];
    assert_eq!(cases.len(), 14);
    for (status, s) in cases {
        assert_eq!(status.as_str(), s);
        // serde 无 rename_all → variant 名即 as_str
        let json = serde_json::to_string(&status).unwrap();
        let parsed: AgentSessionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }
}

#[test]
fn status_terminal_active_notification_flags() {
    for status in [
        AgentSessionStatus::Completed,
        AgentSessionStatus::Failed,
        AgentSessionStatus::Aborted,
        AgentSessionStatus::Crashed,
        AgentSessionStatus::Timeout,
    ] {
        assert!(status.is_terminal(), "{status:?} must be terminal");
        assert!(!status.is_active(), "terminal must not be active");
    }
    for status in [
        AgentSessionStatus::Starting,
        AgentSessionStatus::Running,
        AgentSessionStatus::WaitingTool,
        AgentSessionStatus::ToolRunning,
        AgentSessionStatus::ToolCompleted,
        AgentSessionStatus::WaitingFeedback,
        AgentSessionStatus::FeedbackReceived,
        AgentSessionStatus::Validating,
    ] {
        assert!(!status.is_terminal(), "{status:?} must not be terminal");
        assert!(status.is_active(), "{status:?} must be active");
    }
    assert!(
        !AgentSessionStatus::Created.is_active(),
        "Created is neither"
    );
    assert!(!AgentSessionStatus::Created.is_terminal());

    for status in [
        AgentSessionStatus::Failed,
        AgentSessionStatus::Crashed,
        AgentSessionStatus::Timeout,
    ] {
        assert!(
            status.triggers_notification(),
            "{status:?} must trigger notification"
        );
    }
    assert!(!AgentSessionStatus::Aborted.triggers_notification());
    assert!(!AgentSessionStatus::Completed.triggers_notification());
}

#[test]
fn agent_type_as_str_all_six() {
    let cases = [
        (AgentType::Codex, "codex"),
        (AgentType::ClaudeCode, "claude_code"),
        (AgentType::GeminiCli, "gemini_cli"),
        (AgentType::OpenAiCompatible, "openai_compatible"),
        (AgentType::Local, "local"),
        (AgentType::Future, "future"),
    ];
    for (t, s) in cases {
        assert_eq!(t.as_str(), s);
        let json = serde_json::to_string(&t).unwrap();
        let parsed: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }
}

// =====================================================================
// ID 类型
// =====================================================================

#[test]
fn tenant_id_full_surface() {
    let t = TenantId::new();
    let u = t.as_uuid();
    assert_ne!(u, Uuid::nil());
    assert_eq!(TenantId::from(u), t);
    assert_eq!(t.to_string(), u.to_string());
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(json, format!("\"{u}\""));
    let parsed: TenantId = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, t);
}

#[test]
fn domain_uuid_ids_roundtrip() {
    macro_rules! check {
        ($t:ty) => {{
            let id = <$t>::new();
            assert_eq!(<$t>::from(id.as_uuid()), id);
            assert_eq!(id.to_string(), id.as_uuid().to_string());
            let json = serde_json::to_string(&id).unwrap();
            let parsed: $t = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, id);
        }};
    }
    check!(AgentId);
    check!(AgentSessionId);
    check!(ProjectId);
    check!(UserId);
    check!(WorktreeId);
    check!(WorkItemId);
}

// =====================================================================
// Agent / AgentSession serde
// =====================================================================

#[test]
fn agent_serde_roundtrip() {
    let now = chrono::Utc::now();
    let agent = Agent {
        id: AgentId::new(),
        tenant_id: TenantId::new(),
        agent_type: AgentType::ClaudeCode,
        provider: "anthropic".into(),
        version: "4".into(),
        capabilities: vec!["edit".into(), "bash".into()],
        policy_template_id: None,
        enabled: true,
        created_at: now,
        updated_at: now,
    };
    let json = serde_json::to_string(&agent).unwrap();
    let parsed: Agent = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.id, agent.id);
    assert_eq!(parsed.agent_type, AgentType::ClaudeCode);
    assert_eq!(
        parsed.capabilities,
        vec!["edit".to_string(), "bash".to_string()]
    );
    assert!(parsed.enabled);
}

#[test]
fn agent_session_serde_roundtrip() {
    let now = chrono::Utc::now();
    let session = AgentSession {
        id: AgentSessionId::new(),
        tenant_id: TenantId::new(),
        agent_id: AgentId::new(),
        agent_type: AgentType::Codex,
        provider: "openai".into(),
        version: "1".into(),
        worktree_id: WorktreeId::new(),
        work_item_id: WorkItemId::new(),
        status: AgentSessionStatus::Running,
        intent: "fix the bug".into(),
        context_packet_id: Some(Uuid::new_v4()),
        plan: Some("1. repro 2. fix".into()),
        decisions: vec![Uuid::new_v4()],
        tool_activity_summary: [("edit".to_string(), 3u32)].into_iter().collect(),
        change_set_ids: vec![],
        validation_result_ids: vec![],
        feedback_consumed_ids: vec![],
        result_summary: None,
        trace_reference: Some("trace-1".into()),
        token_usage: [("input".to_string(), 100u64)].into_iter().collect(),
        cost_summary: [("total".to_string(), 0.5f64)].into_iter().collect(),
        started_at: now,
        ended_at: None,
        created_at: now,
        updated_at: now,
        lock_version: 0,
    };
    let json = serde_json::to_string(&session).unwrap();
    let parsed: AgentSession = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.id, session.id);
    assert_eq!(parsed.status, AgentSessionStatus::Running);
    assert_eq!(parsed.intent, "fix the bug");
    assert_eq!(parsed.context_packet_id, session.context_packet_id);
    assert_eq!(parsed.tool_activity_summary.get("edit"), Some(&3));
    assert_eq!(parsed.token_usage.get("input"), Some(&100));
    assert_eq!(parsed.lock_version, 0);
}

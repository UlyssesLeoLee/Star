//! domain-audit 集成测试 (WBS 9.6 覆盖率补写)
//!
//! 覆盖 src/lib.rs 内联 UT 之外的公共 API 表面:
//! - define_uuid_id! 宏生成 ID 的全方法 (new/from_uuid/as_uuid/into_uuid/Deref/Display/From/Default/serde)
//! - AuditAction / Actor / ExportFormat / ExportStatus / AuditEventKind serde + Display
//! - AuditError code() / is_server_error / From<uuid::Error>
//! - AIAuditMetadata 9 问完整性负路径 + 保留期限
//! - 不变量检查函数全部错误分支
//! - compute_immutable_hash 确定性
//! - InMemoryAuditService: record/query/export/event 通道 全分支
//! - RecordOnboardingFailedCommand::into_record_command 转换

use chrono::{Duration as ChronoDuration, Utc};
use domain_audit::{
    check_invariant_02_required_fields, check_invariant_03_immutable_hash,
    check_invariant_04_cross_tenant_flag, check_invariant_04_no_sensitive_plaintext,
    compute_immutable_hash, roles, run_invariants, AIAuditMetadata, AIAuditMetadataId,
    AIAuditMetadataInput, Actor, AgentId, AgentSessionId, AuditAction, AuditError, AuditEvent,
    AuditEventId, AuditEventKind, AuditExportJobId, AuditListQuery, AuditQueryPort, AuditRecorder,
    ChangeSetId, ContextPacketId, ExportAuditCommand, ExportFormat, ExportStatus, FeedbackId,
    InMemoryAuditService, ProviderDataBoundaryId, RecordAIAuditCommand, RecordAuditCommand,
    RecordCrossTenantAttemptCommand, RecordOnboardingFailedCommand, TenantId, UserId,
    ValidationResultId,
};
use star_context::ActorContext;
use uuid::Uuid;

fn make_admin_actor(tenant: TenantId) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant.0)
        .with_role(roles::TENANT_ADMIN)
        .with_role("audit_reader")
        .with_role("audit_exporter")
}

fn make_actor_user() -> Actor {
    Actor::User {
        user_id: UserId::new(),
    }
}

fn make_valid_event(tenant: TenantId) -> AuditEvent {
    AuditEvent {
        id: AuditEventId::new(),
        tenant_id: tenant,
        actor: make_actor_user(),
        action: AuditAction::WorkItemOperation,
        resource_type: "work_item".to_string(),
        resource_id: Uuid::new_v4(),
        context_refs: vec![],
        before_state: None,
        after_state: None,
        cross_tenant: false,
        immutable_hash: "a".repeat(64),
        occurred_at: Utc::now(),
    }
}

fn make_record_cmd(tenant: TenantId) -> RecordAuditCommand {
    RecordAuditCommand {
        tenant_id: tenant,
        actor: make_actor_user(),
        action: AuditAction::WorkItemOperation,
        resource_type: "work_item".to_string(),
        resource_id: Uuid::new_v4(),
        context_refs: vec![],
        before_state: None,
        after_state: None,
        immutable_hash: None,
    }
}

// =====================================================================
// ID newtypes (define_uuid_id! 宏)
// =====================================================================

#[test]
fn audit_event_id_full_method_surface() {
    let id = AuditEventId::new();
    let u = id.as_uuid();
    assert_eq!(id.into_uuid(), u);
    assert_eq!(AuditEventId::from_uuid(u), id);
    assert_eq!(AuditEventId::from(u), id);
    assert_eq!(*id, u, "Deref to Uuid");
    assert_eq!(id.to_string(), u.to_string(), "Display");
    // serde transparent: 序列化即 uuid 字符串
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, format!("\"{u}\""));
    let parsed: AuditEventId = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, id);
    // Default = 新随机 (非 nil)
    assert_ne!(AuditEventId::default().as_uuid(), Uuid::nil());
}

#[test]
fn all_uuid_id_types_roundtrip() {
    macro_rules! check {
        ($t:ty) => {{
            let id = <$t>::new();
            let json = serde_json::to_string(&id).unwrap();
            let parsed: $t = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, id);
            assert_eq!(<$t>::from_uuid(id.as_uuid()), id);
        }};
    }
    check!(AIAuditMetadataId);
    check!(AuditExportJobId);
    check!(TenantId);
    check!(UserId);
    check!(AgentId);
    check!(AgentSessionId);
    check!(ContextPacketId);
    check!(ChangeSetId);
    check!(ValidationResultId);
    check!(FeedbackId);
    check!(ProviderDataBoundaryId);
}

// =====================================================================
// AuditAction / Actor / 枚举 serde + Display
// =====================================================================

#[test]
fn audit_action_display_all_thirteen() {
    let cases = [
        (AuditAction::WorkItemOperation, "WORK_ITEM_OPERATION"),
        (AuditAction::WorktreeOperation, "WORKTREE_OPERATION"),
        (AuditAction::PullRequestOperation, "PULL_REQUEST_OPERATION"),
        (AuditAction::PermissionChange, "PERMISSION_CHANGE"),
        (AuditAction::RoleAssign, "ROLE_ASSIGN"),
        (AuditAction::CrossTenantAttempt, "CROSS_TENANT_ATTEMPT"),
        (AuditAction::AgentExecute, "AGENT_EXECUTE"),
        (AuditAction::ValidationRun, "VALIDATION_RUN"),
        (AuditAction::FeedbackCreated, "FEEDBACK_CREATED"),
        (AuditAction::AiRetentionPurged, "AI_RETENTION_PURGED"),
        (AuditAction::ExportRequested, "EXPORT_REQUESTED"),
        (
            AuditAction::OnboardingTestKeyFailed,
            "ONBOARDING_TEST_KEY_FAILED",
        ),
        (AuditAction::Custom, "CUSTOM"),
    ];
    for (action, expect) in cases {
        assert_eq!(action.to_string(), expect);
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, format!("\"{expect}\""));
        let parsed: AuditAction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }
    assert_eq!(AuditAction::default(), AuditAction::Custom);
}

#[test]
fn actor_tagged_serde_roundtrip() {
    let user = Actor::User {
        user_id: UserId::new(),
    };
    let json = serde_json::to_string(&user).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "user");
    let parsed: Actor = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, Actor::User { .. }));

    let agent = Actor::Agent {
        session_id: AgentSessionId::new(),
        agent_id: AgentId::new(),
    };
    let json = serde_json::to_string(&agent).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "agent");
    let parsed: Actor = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, Actor::Agent { .. }));

    let json = serde_json::to_string(&Actor::System).unwrap();
    assert_eq!(json, "{\"type\":\"system\"}");
    let parsed: Actor = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, Actor::System));
}

#[test]
fn export_format_and_status_serde() {
    assert_eq!(ExportFormat::default(), ExportFormat::Csv);
    assert_eq!(
        serde_json::to_string(&ExportFormat::Csv).unwrap(),
        "\"CSV\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Parquet).unwrap(),
        "\"PARQUET\""
    );
    let parsed: ExportFormat = serde_json::from_str("\"PARQUET\"").unwrap();
    assert_eq!(parsed, ExportFormat::Parquet);

    for status in [
        ExportStatus::Pending,
        ExportStatus::Running,
        ExportStatus::Completed,
        ExportStatus::Failed,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let parsed: ExportStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }
}

#[test]
fn audit_event_kind_subjects() {
    let tenant = TenantId::new();
    let kind = AuditEventKind::Recorded(domain_audit::AuditRecorded {
        meta: domain_audit::EventMeta::new(tenant),
        audit_event_id: AuditEventId::new(),
        action: AuditAction::Custom,
    });
    assert_eq!(kind.subject(), "star.events.audit.event.recorded.v1");

    let kind = AuditEventKind::CrossTenantAttempted(domain_audit::CrossTenantAttempted {
        meta: domain_audit::EventMeta::new(tenant),
        audit_event_id: AuditEventId::new(),
        actor_user_id: Uuid::new_v4(),
        attempted_resource_type: "work_item".into(),
        attempted_resource_id: Uuid::new_v4(),
    });
    assert_eq!(kind.subject(), "star.events.audit.cross_tenant_attempt.v1");

    // serde tag = "type" snake_case
    let json = serde_json::to_string(&kind).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "cross_tenant_attempted");
}

// =====================================================================
// AuditError
// =====================================================================

#[test]
fn audit_error_codes_and_server_error_flag() {
    let cases = [
        (
            AuditError::NotFound(AuditEventId::new()),
            "AUDIT_NOT_FOUND",
            false,
        ),
        (
            AuditError::InvalidState("x".into()),
            "AUDIT_INVALID_STATE",
            false,
        ),
        (
            AuditError::PermissionDenied,
            "AUDIT_PERMISSION_DENIED",
            false,
        ),
        (AuditError::Conflict("x".into()), "AUDIT_CONFLICT", false),
        (AuditError::Internal("x".into()), "AUDIT_INTERNAL", true),
    ];
    for (err, code, is_server) in cases {
        assert_eq!(err.code(), code);
        assert_eq!(err.is_server_error(), is_server);
    }
    // Display 经 thiserror
    assert!(AuditError::NotFound(AuditEventId::new())
        .to_string()
        .contains("audit event not found"));
}

#[test]
fn audit_error_from_uuid_error() {
    let uuid_err = Uuid::parse_str("not-a-uuid").unwrap_err();
    let err: AuditError = uuid_err.into();
    assert_eq!(err.code(), "AUDIT_INTERNAL");
    assert!(err.to_string().contains("uuid error"));
}

// =====================================================================
// AIAuditMetadata 9 问完整性
// =====================================================================

fn complete_meta_input(session: AgentSessionId, agent: AgentId) -> AIAuditMetadataInput {
    let now = Utc::now();
    AIAuditMetadataInput {
        agent_session_id: session,
        agent_id: agent,
        context_packet_id: Some(ContextPacketId::new()),
        change_set_id: Some(ChangeSetId::new()),
        worktree_id: Some(Uuid::new_v4()),
        started_at: now - ChronoDuration::seconds(5),
        ended_at: now,
        validation_result_ids: vec![],
        feedback_consumed_ids: vec![],
        approver_user_id: Some(UserId::new()),
        data_categories_sent: vec![],
        provider_boundary_ref: None,
        risk_signals: vec![],
        full_prompt_ref: None,
        full_response_ref: None,
        prompt_hash: "a".repeat(64),
        response_hash: "b".repeat(64),
        retention: None,
    }
}

fn meta_from_input(tenant: TenantId, input: AIAuditMetadataInput) -> AIAuditMetadata {
    let now = Utc::now();
    AIAuditMetadata {
        id: AIAuditMetadataId::new(),
        tenant_id: tenant,
        agent_session_id: input.agent_session_id,
        context_packet_id: input.context_packet_id,
        change_set_id: input.change_set_id,
        agent_id: input.agent_id,
        worktree_id: input.worktree_id,
        started_at: input.started_at,
        ended_at: input.ended_at,
        validation_result_ids: input.validation_result_ids,
        feedback_consumed_ids: input.feedback_consumed_ids,
        approver_user_id: input.approver_user_id,
        data_categories_sent: input.data_categories_sent,
        provider_boundary_ref: input.provider_boundary_ref,
        risk_signals: input.risk_signals,
        full_prompt_ref: input.full_prompt_ref,
        full_response_ref: input.full_response_ref,
        prompt_hash: input.prompt_hash,
        response_hash: input.response_hash,
        retention_until: now,
        created_at: now,
    }
}

#[test]
fn ai_meta_9_questions_complete_vs_incomplete() {
    let session = AgentSessionId::new();
    let agent = AgentId::new();
    let tenant = TenantId::new();

    // 完整 → true
    let mut input = complete_meta_input(session, agent);
    let meta = meta_from_input(tenant, input.clone());
    assert!(meta.has_complete_9_questions());

    // nil session → false
    input.agent_session_id = AgentSessionId::from_uuid(Uuid::nil());
    assert!(!meta_from_input(tenant, input.clone()).has_complete_9_questions());

    // 恢复 session, nil agent → false
    input.agent_session_id = session;
    input.agent_id = AgentId::from_uuid(Uuid::nil());
    assert!(!meta_from_input(tenant, input.clone()).has_complete_9_questions());

    // 恢复 agent, hash 长度错 → false
    input.agent_id = agent;
    input.prompt_hash = "short".into();
    assert!(!meta_from_input(tenant, input.clone()).has_complete_9_questions());

    // 恢复 hash, started > ended → false
    input.prompt_hash = "a".repeat(64);
    input.started_at = input.ended_at + ChronoDuration::seconds(1);
    assert!(!meta_from_input(tenant, input).has_complete_9_questions());
}

#[test]
fn ai_meta_is_expired_retention() {
    let tenant = TenantId::new();
    let session = AgentSessionId::new();
    let agent = AgentId::new();
    let mut meta = meta_from_input(tenant, complete_meta_input(session, agent));
    let now = Utc::now();
    meta.retention_until = now - ChronoDuration::seconds(1);
    assert!(meta.is_expired(now));
    meta.retention_until = now + ChronoDuration::seconds(1);
    assert!(!meta.is_expired(now));
}

// =====================================================================
// 不变量检查
// =====================================================================

#[test]
fn invariant_02_rejects_nil_tenant() {
    let mut ev = make_valid_event(TenantId::new());
    ev.tenant_id = TenantId::from_uuid(Uuid::nil());
    assert!(check_invariant_02_required_fields(&ev).is_err());
}

#[test]
fn invariant_02_rejects_empty_resource_type_and_nil_resource() {
    let tenant = TenantId::new();
    let mut ev = make_valid_event(tenant);
    ev.resource_type = String::new();
    assert!(check_invariant_02_required_fields(&ev).is_err());

    let mut ev = make_valid_event(tenant);
    ev.resource_id = Uuid::nil();
    assert!(check_invariant_02_required_fields(&ev).is_err());
}

#[test]
fn invariant_02_rejects_nil_actor_ids() {
    let tenant = TenantId::new();
    let mut ev = make_valid_event(tenant);
    ev.actor = Actor::User {
        user_id: UserId::from_uuid(Uuid::nil()),
    };
    assert!(check_invariant_02_required_fields(&ev).is_err());

    let mut ev = make_valid_event(tenant);
    ev.actor = Actor::Agent {
        session_id: AgentSessionId::from_uuid(Uuid::nil()),
        agent_id: AgentId::new(),
    };
    assert!(check_invariant_02_required_fields(&ev).is_err());

    // System actor → 通过
    let mut ev = make_valid_event(tenant);
    ev.actor = Actor::System;
    assert!(check_invariant_02_required_fields(&ev).is_ok());
}

#[test]
fn invariant_03_rejects_empty_short_and_nonhex() {
    let tenant = TenantId::new();
    let mut ev = make_valid_event(tenant);
    ev.immutable_hash = String::new();
    assert!(check_invariant_03_immutable_hash(&ev).is_err());
    ev.immutable_hash = "abc".into();
    assert!(check_invariant_03_immutable_hash(&ev).is_err());
    ev.immutable_hash = "g".repeat(64); // 非 hex
    assert!(check_invariant_03_immutable_hash(&ev).is_err());
    ev.immutable_hash = "f".repeat(64);
    assert!(check_invariant_03_immutable_hash(&ev).is_ok());
}

#[test]
fn invariant_04_cross_tenant_flag_consistency() {
    let tenant = TenantId::new();
    let mut ev = make_valid_event(tenant);
    ev.action = AuditAction::CrossTenantAttempt;
    ev.cross_tenant = false;
    assert!(check_invariant_04_cross_tenant_flag(&ev).is_err());
    ev.cross_tenant = true;
    assert!(check_invariant_04_cross_tenant_flag(&ev).is_ok());
    // 反过来: cross_tenant=true 但 action 不是 CrossTenantAttempt
    ev.action = AuditAction::Custom;
    assert!(check_invariant_04_cross_tenant_flag(&ev).is_err());
}

#[test]
fn invariant_04_no_sensitive_plaintext_over_256_chars() {
    let tenant = TenantId::new();
    let mut ev = make_valid_event(tenant);
    // prompt 长于 256 → 拒绝
    ev.before_state = Some(serde_json::json!({"prompt": "x".repeat(300)}));
    assert!(check_invariant_04_no_sensitive_plaintext(&ev).is_err());
    // 短 prompt → 允许
    ev.before_state = Some(serde_json::json!({"prompt": "short"}));
    assert!(check_invariant_04_no_sensitive_plaintext(&ev).is_ok());
    // after_state 中的长 prompt 同样拒绝
    ev.before_state = None;
    ev.after_state = Some(serde_json::json!({"prompt": "y".repeat(300)}));
    assert!(check_invariant_04_no_sensitive_plaintext(&ev).is_err());
    // 已知实现限制: 当前只检查 "prompt" key, response/code 长文本不拦 (文档化实际行为)
    ev.after_state = Some(serde_json::json!({"response": "y".repeat(300)}));
    assert!(check_invariant_04_no_sensitive_plaintext(&ev).is_ok());
    // 非 object 值 → 忽略
    ev.after_state = Some(serde_json::json!("just a string"));
    assert!(check_invariant_04_no_sensitive_plaintext(&ev).is_ok());
}

#[test]
fn run_invariants_all_pass_for_valid_event() {
    let ev = make_valid_event(TenantId::new());
    assert!(run_invariants(domain_audit::ALL_INVARIANT_CHECKS, &ev).is_ok());
    assert_eq!(domain_audit::ALL_INVARIANT_CHECKS.len(), 4);
}

#[test]
fn compute_immutable_hash_deterministic_64_hex() {
    let tenant = TenantId::new();
    let actor = make_actor_user();
    let action = AuditAction::WorkItemOperation;
    let resource_id = Uuid::new_v4();
    let occurred = Utc::now();
    let h1 = compute_immutable_hash(tenant, &actor, action, "work_item", resource_id, occurred);
    let h2 = compute_immutable_hash(tenant, &actor, action, "work_item", resource_id, occurred);
    assert_eq!(h1.len(), 64);
    assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(h1, h2, "deterministic");

    // 不同 actor → 不同 hash
    let h3 = compute_immutable_hash(
        tenant,
        &Actor::System,
        action,
        "work_item",
        resource_id,
        occurred,
    );
    assert_ne!(h1, h3);
    // 不同 resource → 不同 hash
    let h4 = compute_immutable_hash(
        tenant,
        &actor,
        action,
        "work_item",
        Uuid::new_v4(),
        occurred,
    );
    assert_ne!(h1, h4);
    // 不同 action → 不同 hash
    let h5 = compute_immutable_hash(
        tenant,
        &actor,
        AuditAction::RoleAssign,
        "work_item",
        resource_id,
        occurred,
    );
    assert_ne!(h1, h5);
}

#[test]
fn audit_list_query_defaults() {
    let q = AuditListQuery::default();
    assert_eq!(q.limit, 100);
    assert_eq!(q.offset, 0);
    assert!(q.action.is_none());
    assert!(q.since.is_none());
    assert!(q.until.is_none());
}

// =====================================================================
// InMemoryAuditService: record 路径
// =====================================================================

#[tokio::test]
async fn record_with_precomputed_hash_uses_it() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let mut cmd = make_record_cmd(tenant);
    cmd.immutable_hash = Some("e".repeat(64));
    let ev = svc
        .record(cmd, make_admin_actor(tenant))
        .await
        .expect("record ok");
    assert_eq!(ev.immutable_hash, "e".repeat(64));
    assert_eq!(ev.action, AuditAction::WorkItemOperation);
    assert!(!ev.cross_tenant);
}

#[tokio::test]
async fn record_rejects_invariant_violation() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let mut cmd = make_record_cmd(tenant);
    cmd.resource_type = String::new(); // INV-AU-02 违规
    let err = svc.record(cmd, make_admin_actor(tenant)).await.unwrap_err();
    assert_eq!(err.code(), "AUDIT_INVALID_STATE");
    assert_eq!(svc.event_count().await, 0);
}

#[tokio::test]
async fn record_cross_tenant_publishes_event_and_flags() {
    let (svc, mut rx) = InMemoryAuditService::new();
    let tenant = TenantId::new();
    let cmd = RecordCrossTenantAttemptCommand {
        actor_user_id: Uuid::new_v4(),
        attempted_resource_type: "work_item".into(),
        attempted_resource_id: Uuid::new_v4(),
    };
    let ev = svc
        .record_cross_tenant_attempt(cmd, make_admin_actor(tenant))
        .await
        .unwrap();
    assert!(ev.cross_tenant);
    assert_eq!(ev.action, AuditAction::CrossTenantAttempt);
    assert_eq!(svc.cross_tenant_count().await, 1);

    // 事件通道收到 CrossTenantAttempted
    match rx.recv().await {
        Some(AuditEventKind::CrossTenantAttempted(a)) => {
            assert_eq!(a.audit_event_id, ev.id);
            assert_eq!(a.attempted_resource_type, "work_item");
        }
        other => panic!("expected CrossTenantAttempted, got {other:?}"),
    }
}

#[tokio::test]
async fn record_ai_incomplete_9_questions_rejected() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let mut input = complete_meta_input(AgentSessionId::new(), AgentId::new());
    input.prompt_hash = "bad".into();
    let err = svc
        .record_ai(
            RecordAIAuditCommand {
                tenant_id: tenant,
                metadata: input,
            },
            make_admin_actor(tenant),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code(), "AUDIT_INVALID_STATE");
    assert_eq!(svc.ai_meta_count().await, 0);
}

#[tokio::test]
async fn record_ai_explicit_retention_overrides_default() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let mut input = complete_meta_input(AgentSessionId::new(), AgentId::new());
    input.retention = Some(std::time::Duration::from_secs(86400)); // 1 天
    let meta = svc
        .record_ai(
            RecordAIAuditCommand {
                tenant_id: tenant,
                metadata: input,
            },
            make_admin_actor(tenant),
        )
        .await
        .unwrap();
    let delta = meta.retention_until - meta.created_at;
    assert_eq!(delta.num_days(), 1);
    assert_eq!(svc.ai_meta_count().await, 1);
}

// =====================================================================
// InMemoryAuditService: query 路径
// =====================================================================

#[tokio::test]
async fn list_events_filters_action_and_pagination() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let admin = make_admin_actor(tenant);

    for _ in 0..3 {
        let mut cmd = make_record_cmd(tenant);
        cmd.action = AuditAction::RoleAssign;
        svc.record(cmd, admin.clone()).await.unwrap();
    }
    let mut cmd = make_record_cmd(tenant);
    cmd.action = AuditAction::PermissionChange;
    svc.record(cmd, admin.clone()).await.unwrap();

    // action 过滤
    let q = AuditListQuery {
        tenant_id: tenant,
        action: Some(AuditAction::RoleAssign),
        ..Default::default()
    };
    let out = svc.list_events(q, admin.clone()).await.unwrap();
    assert_eq!(out.len(), 3);

    // offset + limit 分页
    let q = AuditListQuery {
        tenant_id: tenant,
        limit: 2,
        offset: 2,
        ..Default::default()
    };
    let out = svc.list_events(q, admin.clone()).await.unwrap();
    assert_eq!(out.len(), 2);

    // 无 action 过滤 → 4 条
    let q = AuditListQuery {
        tenant_id: tenant,
        ..Default::default()
    };
    let out = svc.list_events(q, admin.clone()).await.unwrap();
    assert_eq!(out.len(), 4);
}

#[tokio::test]
async fn list_events_cross_tenant_denied() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let admin = make_admin_actor(tenant);
    let _ = svc
        .record(make_record_cmd(tenant), admin.clone())
        .await
        .unwrap();

    // viewer 属于另一租户
    let other_tenant = TenantId::new();
    let foreign_admin = make_admin_actor(other_tenant);
    let q = AuditListQuery {
        tenant_id: tenant,
        ..Default::default()
    };
    let err = svc.list_events(q, foreign_admin).await.unwrap_err();
    assert_eq!(err.code(), "AUDIT_PERMISSION_DENIED");
}

#[tokio::test]
async fn get_event_not_found_and_cross_tenant() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let admin = make_admin_actor(tenant);

    // 不存在 → NotFound
    let err = svc
        .get_event(AuditEventId::new(), admin.clone())
        .await
        .unwrap_err();
    assert!(matches!(err, AuditError::NotFound(_)));

    // 记录后按 id 取 → ok
    let ev = svc
        .record(make_record_cmd(tenant), admin.clone())
        .await
        .unwrap();
    let got = svc.get_event(ev.id, admin.clone()).await.unwrap();
    assert_eq!(got.id, ev.id);

    // 跨租户 viewer → PermissionDenied
    let foreign = make_admin_actor(TenantId::new());
    let err = svc.get_event(ev.id, foreign).await.unwrap_err();
    assert_eq!(err.code(), "AUDIT_PERMISSION_DENIED");
}

#[tokio::test]
async fn list_ai_audit_filters_by_session() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let admin = make_admin_actor(tenant);
    let s1 = AgentSessionId::new();
    let s2 = AgentSessionId::new();

    let mut i1 = complete_meta_input(s1, AgentId::new());
    i1.context_packet_id = None; // 可空字段允许 None
    let _ = svc
        .record_ai(
            RecordAIAuditCommand {
                tenant_id: tenant,
                metadata: i1,
            },
            admin.clone(),
        )
        .await
        .unwrap();
    let mut i2 = complete_meta_input(s2, AgentId::new());
    i2.context_packet_id = None;
    let _ = svc
        .record_ai(
            RecordAIAuditCommand {
                tenant_id: tenant,
                metadata: i2,
            },
            admin.clone(),
        )
        .await
        .unwrap();

    let all = svc
        .list_ai_audit(tenant, None, admin.clone())
        .await
        .unwrap();
    assert_eq!(all.len(), 2);

    let only_s1 = svc
        .list_ai_audit(tenant, Some(s1), admin.clone())
        .await
        .unwrap();
    assert_eq!(only_s1.len(), 1);
    assert_eq!(only_s1[0].agent_session_id, s1);
}

// =====================================================================
// InMemoryAuditService: export 路径
// =====================================================================

#[tokio::test]
async fn export_parquet_job_pending_with_requested_by() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let admin = make_admin_actor(tenant);
    let now = Utc::now();
    let job = svc
        .export(
            ExportAuditCommand {
                tenant_id: tenant,
                format: ExportFormat::Parquet,
                range_start: now - ChronoDuration::days(1),
                range_end: now,
                filter_action: Some(AuditAction::AgentExecute),
            },
            admin.clone(),
        )
        .await
        .unwrap();
    assert_eq!(job.status, ExportStatus::Pending);
    assert_eq!(job.format, ExportFormat::Parquet);
    assert_eq!(job.filter_action, Some(AuditAction::AgentExecute));
    assert_eq!(job.requested_by, UserId::from_uuid(admin.user_id));
    assert!(job.download_url.is_none());
    assert!(job.completed_at.is_none());
}

#[tokio::test]
async fn export_denied_for_non_exporter_and_foreign_tenant() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let developer = ActorContext::new(Uuid::new_v4(), tenant.0).with_role("developer");
    let now = Utc::now();
    let cmd = ExportAuditCommand {
        tenant_id: tenant,
        format: ExportFormat::Csv,
        range_start: now - ChronoDuration::days(1),
        range_end: now,
        filter_action: None,
    };
    let err = svc.export(cmd.clone(), developer).await.unwrap_err();
    assert_eq!(err.code(), "AUDIT_PERMISSION_DENIED");

    // 有 exporter 角色但租户不匹配 → 也拒绝
    let foreign = ActorContext::new(Uuid::new_v4(), TenantId::new().0).with_role("audit_exporter");
    let err = svc.export(cmd, foreign).await.unwrap_err();
    assert_eq!(err.code(), "AUDIT_PERMISSION_DENIED");
}

// =====================================================================
// Onboarding 失败命令转换
// =====================================================================

#[test]
fn onboarding_failed_into_record_command_shape() {
    let tenant = TenantId::new();
    let detected = Uuid::new_v4();
    let cmd = RecordOnboardingFailedCommand {
        tenant_id: tenant,
        detected_key_id: detected,
        provider: "openai".into(),
        label: "OpenAI Primary".into(),
        attempts: 5,
        status_code: 401,
        error_message: "invalid api key".into(),
        client_ip: Some("127.0.0.1".into()),
        request_id: Some(Uuid::new_v4()),
    };
    let record = cmd.into_record_command();
    assert_eq!(record.tenant_id, tenant);
    assert!(matches!(record.actor, Actor::System));
    assert_eq!(record.action, AuditAction::OnboardingTestKeyFailed);
    assert_eq!(record.resource_type, "api_key");
    assert_eq!(record.resource_id, detected);
    assert!(record.immutable_hash.is_none());

    let after = record.after_state.expect("after_state present");
    assert_eq!(after["provider"], "openai");
    assert_eq!(after["label"], "OpenAI Primary");
    assert_eq!(after["attempts"], 5);
    assert_eq!(after["status_code"], 401);
    assert_eq!(after["error_message"], "invalid api key");
    assert_eq!(after["detected_key_id"], detected.to_string());
    // client_ip / request_id 不进 after_state (设计如此: 转 record 时未转记)
    assert!(after.get("client_ip").is_none());
    assert!(after.get("request_id").is_none());
}

// =====================================================================
// Clone / Default 语义
// =====================================================================

#[tokio::test]
async fn service_clone_shares_state() {
    let svc = InMemoryAuditService::new_for_test();
    let tenant = TenantId::new();
    let clone = svc.clone();
    let _ = svc
        .record(make_record_cmd(tenant), make_admin_actor(tenant))
        .await
        .unwrap();
    assert_eq!(clone.event_count().await, 1, "clone shares backing store");
}

#[tokio::test]
async fn service_default_works() {
    let svc = InMemoryAuditService::default();
    assert_eq!(svc.event_count().await, 0);
    assert_eq!(svc.ai_meta_count().await, 0);
    assert_eq!(svc.cross_tenant_count().await, 0);
}

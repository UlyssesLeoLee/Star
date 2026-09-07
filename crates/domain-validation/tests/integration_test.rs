//! 集成测试 (IT) — domain-validation 真实数据接入验证
//!
//! **crate**: domain-validation (test target)
//! **per**: 2026-09-07 P3-B sub-batch 3 业务核心 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): submit_result + get_result + list_results + list_policies
//! - V1 (5 状态机 + Override Protected + Coverage): Pending → Running → Passed → AC 关联
//! - V2 (RLS 13 類 守门 #13 c + 跨 tenant): 跨 tenant 拒绝 + Override 人类守门
//! - V3 (事件总线 + CoverageAchieved 100%): Submitted + Passed + CoverageAchieved NATS subject
//!
//! ## W/T/M 分类 (per 守门 #13)
//!
//! - **ValidationResult / Policy / Override**: M 类 Master (SCD Type 2 + RLS)
//! - **ValidationEvidence**: T 类 Transaction (Append-only, 必带 tenant_id 前缀 INV-VL-08)
//! - **AcceptanceCoverage**: derived view (实时计算,非持久化 M/T 分类)

use domain_validation::{
    AcceptanceCriterionId, ActorContext, AddEvidenceCommand, CreateValidationPolicyCommand,
    EvidenceType, InMemoryValidationService, LinkAcceptanceEvidenceCommand, ListValidationQuery,
    MarkValidationStatusCommand, OverrideValidationCommand, ProjectId, SubmitValidationResultCommand,
    TenantId, UserId, ValidationCommandPort, ValidationError, ValidationEvent, ValidationKind,
    ValidationQueryPort, ValidationStatus, WorkItemId,
};
use uuid::Uuid;

/// IT fixture: developer 角色(人类,可 Override INV-VL-06)
fn make_developer(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("developer")
}

/// IT fixture: service_internal 角色(不可 Override)
fn make_service_internal(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("service_internal")
}

fn make_submit_cmd(tenant_id: TenantId, kind: ValidationKind) -> SubmitValidationResultCommand {
    SubmitValidationResultCommand {
        tenant_id,
        project_id: ProjectId::new(),
        work_item_id: Some(WorkItemId::new()),
        worktree_id: None,
        kind,
        log_excerpt_ref: format!("validation.build_log/{tenant_id}/test.log"),
        evidence_ids: vec![],
        triggered_by_id: None,
        policy_id: None,
        policy_required: false,
        is_ai_complete_claim: false,
    }
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: submit_result → get_result → list_results → list_policies 闭环
#[tokio::test]
async fn it_v0_submit_get_list_lifecycle() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_service_internal(tenant_id);

    let r = svc
        .submit_result(make_submit_cmd(TenantId(tenant_id), ValidationKind::Build), actor.clone())
        .await
        .unwrap();
    assert_eq!(r.status, ValidationStatus::Pending);
    assert_eq!(r.kind, ValidationKind::Build);
    assert!(r.log_excerpt_ref.is_some());

    // get_result
    let fetched = svc.get_result(r.id, actor.clone()).await.unwrap();
    assert_eq!(fetched.id, r.id);
    assert_eq!(fetched.kind, ValidationKind::Build);

    // list_results
    let list = svc
        .list_results(
            ListValidationQuery {
                tenant_id: TenantId(tenant_id),
                work_item_id: None,
                worktree_id: None,
                kind: None,
                status: None,
                limit: 50,
                offset: 0,
            },
            actor.clone(),
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, r.id);

    // list_policies (空)
    let policies = svc.list_policies(actor).await.unwrap();
    assert_eq!(policies.len(), 0);
}

/// **IT-V0-2**: submit_result 缺 log_excerpt_ref (INV-VL-04 / VAL-001) 必拒
#[tokio::test]
async fn it_v0_missing_log_ref_rejected() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_service_internal(tenant_id);
    let mut cmd = make_submit_cmd(TenantId(tenant_id), ValidationKind::UnitTest);
    cmd.log_excerpt_ref = "   ".to_string();
    let res = svc.submit_result(cmd, actor).await;
    assert!(matches!(res, Err(ValidationError::InvalidState(_))));
}

// =====================================================================
// V1: 5 状态机 + Override Protected + Coverage 100%
// =====================================================================

/// **IT-V1-1**: 5 状态机全链 submit → Running → Passed (INV-VL-04 evidence 必带)
#[tokio::test]
async fn it_v1_state_machine_running_to_passed() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_service_internal(tenant_id);
    let r = svc
        .submit_result(make_submit_cmd(TenantId(tenant_id), ValidationKind::Lint), actor.clone())
        .await
        .unwrap();
    assert_eq!(r.status, ValidationStatus::Pending);

    // Pending → Running
    let r = svc
        .mark_status(
            MarkValidationStatusCommand {
                tenant_id: TenantId(tenant_id),
                validation_id: r.id,
                new_status: ValidationStatus::Running,
                failure_summary: None,
            },
            actor.clone(),
        )
        .await
        .unwrap();
    assert_eq!(r.status, ValidationStatus::Running);
    assert!(r.started_at.is_some());

    // Running → Passed
    let r = svc
        .mark_status(
            MarkValidationStatusCommand {
                tenant_id: TenantId(tenant_id),
                validation_id: r.id,
                new_status: ValidationStatus::Passed,
                failure_summary: None,
            },
            actor,
        )
        .await
        .unwrap();
    assert_eq!(r.status, ValidationStatus::Passed);
    assert!(r.completed_at.is_some());
}

/// **IT-V1-2**: Override 必人类 (INV-VL-06): service_internal 必拒
#[tokio::test]
async fn it_v1_override_requires_human_actor() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let svc_actor = make_service_internal(tenant_id);
    let r = svc
        .submit_result(
            make_submit_cmd(TenantId(tenant_id), ValidationKind::Build),
            svc_actor.clone(),
        )
        .await
        .unwrap();
    let res = svc
        .override_result(
            OverrideValidationCommand {
                tenant_id: TenantId(tenant_id),
                validation_id: r.id,
                reason: "测试覆盖".to_string(),
                approver_user_id: UserId::new(),
            },
            svc_actor,
        )
        .await;
    assert!(matches!(res, Err(ValidationError::PermissionDenied)));

    // developer 可成功 Override
    let dev = make_developer(tenant_id);
    let ovr = svc
        .override_result(
            OverrideValidationCommand {
                tenant_id: TenantId(tenant_id),
                validation_id: r.id,
                reason: "测试覆盖".to_string(),
                approver_user_id: UserId(dev.user_id),
            },
            dev,
        )
        .await
        .unwrap();
    assert_eq!(ovr.validation_id, r.id);
}

/// **IT-V1-3**: Policy allow_ai_self_claim=true 必拒 (VAL-001 / INV-VL-09)
#[tokio::test]
async fn it_v1_policy_rejects_ai_self_claim_true() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let res = svc
        .create_policy(
            CreateValidationPolicyCommand {
                tenant_id: TenantId(tenant_id),
                project_id: ProjectId::new(),
                name: "bad-policy".to_string(),
                required_kinds: vec![ValidationKind::Build],
                optional_kinds: vec![],
                pass_thresholds: Default::default(),
                allow_ai_self_claim: true, // VAL-001 禁止
                override_allow: false,
            },
            actor,
        )
        .await;
    assert!(matches!(res, Err(ValidationError::InvariantViolated(_))));
}

/// **IT-V1-4**: AC 关联 必 ValidationResult.status = Passed
#[tokio::test]
async fn it_v1_link_ac_requires_passed_status() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_service_internal(tenant_id);
    let r = svc
        .submit_result(make_submit_cmd(TenantId(tenant_id), ValidationKind::UnitTest), actor.clone())
        .await
        .unwrap();
    // Pending 状态直接 link 必拒
    let res = svc
        .link_to_acceptance_criterion(
            LinkAcceptanceEvidenceCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: r.work_item_id.unwrap(),
                acceptance_criterion_id: Uuid::new_v4(),
                validation_id: r.id,
            },
            actor.clone(),
        )
        .await;
    assert!(matches!(res, Err(ValidationError::InvalidState(_))));

    // Running → Passed
    svc.mark_status(
        MarkValidationStatusCommand {
            tenant_id: TenantId(tenant_id),
            validation_id: r.id,
            new_status: ValidationStatus::Running,
            failure_summary: None,
        },
        actor.clone(),
    )
    .await
    .unwrap();
    svc.mark_status(
        MarkValidationStatusCommand {
            tenant_id: TenantId(tenant_id),
            validation_id: r.id,
            new_status: ValidationStatus::Passed,
            failure_summary: None,
        },
        actor.clone(),
    )
    .await
    .unwrap();
    // Passed 后可 link
    let cov = svc
        .link_to_acceptance_criterion(
            LinkAcceptanceEvidenceCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: r.work_item_id.unwrap(),
                acceptance_criterion_id: Uuid::new_v4(),
                validation_id: r.id,
            },
            actor,
        )
        .await
        .unwrap();
    let _ = AcceptanceCriterionId::new(); // suppress unused
    assert!(cov.is_covered());
}

// =====================================================================
// V2: RLS 13 類 跨 tenant 拒绝 (守门 #13 c)
// =====================================================================

/// **IT-V2-1**: 跨 tenant get_result 拒绝 (INV-VL-07)
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let actor_a = make_service_internal(tenant_a);
    let r = svc
        .submit_result(make_submit_cmd(TenantId(tenant_a), ValidationKind::Build), actor_a)
        .await
        .unwrap();

    let tenant_b = Uuid::new_v4();
    let actor_b = make_service_internal(tenant_b);
    let res = svc.get_result(r.id, actor_b).await;
    assert!(matches!(res, Err(ValidationError::PermissionDenied)));
}

/// **IT-V2-2**: 跨 tenant list_results 拒绝
#[tokio::test]
async fn it_v2_cross_tenant_list_rejected() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let actor_a = make_service_internal(tenant_a);
    svc.submit_result(
        make_submit_cmd(TenantId(tenant_a), ValidationKind::Build),
        actor_a,
    )
    .await
    .unwrap();

    let tenant_b = Uuid::new_v4();
    let actor_b = make_service_internal(tenant_b);
    // actor_b 持 tenant_b, 但查询 q.tenant_id 填 tenant_a → 跨 tenant
    let res = svc
        .list_results(
            ListValidationQuery {
                tenant_id: TenantId(tenant_a),
                work_item_id: None,
                worktree_id: None,
                kind: None,
                status: None,
                limit: 50,
                offset: 0,
            },
            actor_b,
        )
        .await;
    assert!(matches!(res, Err(ValidationError::PermissionDenied)));
}

// =====================================================================
// V3: 事件总线 + Evidence 必带 tenant_id 前缀 (INV-VL-08)
// =====================================================================

/// **IT-V3-1**: 完整事件流 Submitted + Passed NATS subject 校验
#[tokio::test]
async fn it_v3_event_bus_submitted_and_passed() {
    let (svc, mut rx) = InMemoryValidationService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_service_internal(tenant_id);
    let r = svc
        .submit_result(make_submit_cmd(TenantId(tenant_id), ValidationKind::Lint), actor.clone())
        .await
        .unwrap();

    // Submitted 事件
    let evt1 = rx.try_recv().expect("应收到 Submitted 事件");
    assert!(matches!(evt1, ValidationEvent::Submitted(_)));
    assert_eq!(evt1.subject(), "star.events.validation.validation_result.submitted.v1");

    // → Running
    svc.mark_status(
        MarkValidationStatusCommand {
            tenant_id: TenantId(tenant_id),
            validation_id: r.id,
            new_status: ValidationStatus::Running,
            failure_summary: None,
        },
        actor.clone(),
    )
    .await
    .unwrap();
    // → Passed
    svc.mark_status(
        MarkValidationStatusCommand {
            tenant_id: TenantId(tenant_id),
            validation_id: r.id,
            new_status: ValidationStatus::Passed,
            failure_summary: None,
        },
        actor,
    )
    .await
    .unwrap();

    // 收到 Passed 事件
    let mut found_passed = false;
    for _ in 0..5 {
        if let Ok(e) = rx.try_recv() {
            if matches!(e, ValidationEvent::Passed(_)) {
                found_passed = true;
                assert_eq!(e.subject(), "star.events.validation.validation_result.passed.v1");
                break;
            }
        }
    }
    assert!(found_passed, "应收到 Passed 事件");
}

/// **IT-V3-2**: add_evidence 缺 tenant_id 前缀 (INV-VL-08) 必拒
#[tokio::test]
async fn it_v3_evidence_storage_ref_requires_tenant_prefix() {
    let svc = InMemoryValidationService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_service_internal(tenant_id);
    let r = svc
        .submit_result(make_submit_cmd(TenantId(tenant_id), ValidationKind::Build), actor.clone())
        .await
        .unwrap();
    let res = svc
        .add_evidence(
            AddEvidenceCommand {
                tenant_id: TenantId(tenant_id),
                validation_id: r.id,
                evidence_type: EvidenceType::BuildLog,
                storage_ref: "wrong-prefix/file.log".to_string(), // 缺 tenant_id
                size_bytes: Some(1024),
                mime_type: Some("text/plain".to_string()),
            },
            actor,
        )
        .await;
    assert!(matches!(res, Err(ValidationError::InvalidState(_))));
}

//! Validation 领域
//!
//! **crate**: `domain-validation`
//! **上游 spec**: docs/specs/domain-validation-spec.md
//! **基本设计**: docs/basic-design.md §4.5
//! **数据设计**: docs/data-design.md §4.24 (`validation` schema)
//! **API 设计**: docs/api-design.md §3.25 (Validation endpoints)
//!
//! ## 职责
//!
//! ValidationResult 聚合根 + 7+3 类 Validation + 5 状态机 + AcceptanceCoverage
//! 派生 + ValidationPolicy 模板 + Evidence Object Storage 引用。
//! 核心职责是 **AI 自我报告不构成完成**(VAL-001,INV-VL-01)+ **四重门强约束**
//! (ValidationPassed && AcceptanceCoverage==100 && FeedbackResolved && GateApproved)。
//!
//! ## 关键不变量
//!
//! - 7 类 Validation:Build / UnitTest / IntegrationTest / Lint / Format / StaticAnalysis / SecurityCheck
//!   (SOW 必交付;另含 3 类附加 AcceptanceCheck / Review / CustomValidation 与 data-design 对齐)
//! - 5 状态机:PENDING / RUNNING / PASSED / FAILED / SKIPPED
//! - Validation Evidence 独立来源,不可 Agent 自报(INV-VL-04,VAL-001)
//! - 必带 tenant_id,跨 tenant 拒绝(INV-VL-07)
//! - Object Storage Key 必带 tenant_id 前缀(INV-VL-08,13 类 #10/#11)
//! - AI 自我声明完成时必须经四重门(INV-VL-01/02,VAL-001)
//! - AcceptanceCoverage 100% 是 READY_FOR_REVIEW 必要条件(INV-VL-05)
//! - Override 必须人类 Protected 鉴权(INV-VL-06)
//! - ValidationPolicy.allow_ai_self_claim 默认 false(INV-VL-09,VAL-001)
//!
//! ## 上游依赖
//!
//! 本 crate 不依赖任何 domain-* crate(spec §1 职责边界),保持独立性。

#![allow(missing_docs)]

// =====================================================================
// 子模块装载
// =====================================================================

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

// =====================================================================
// 便捷 re-export
// =====================================================================

#[allow(unused_imports)]
use context::ActorContext as _ContextActorContext; // 内部使用 (子模块强类型 ID 版)
pub use entity::{
    AcceptanceCoverage, AcceptanceCoverageReport, EvidenceDownloadURL, ValidationEvidence,
    ValidationOverride, ValidationPolicy, ValidationResult,
};
pub use error::ValidationError;
pub use event::{
    AcceptanceCoverageAchieved, AcceptanceCoverageLinked, EventMeta, EvidenceLinked,
    FeedbackRequired, ValidationEvent, ValidationFailed, ValidationOverridden, ValidationPassed,
    ValidationResultSubmitted,
};
pub use invariants::{
    check_ai_self_claim_requires_validation_passed, check_ai_self_claim_status,
    check_create_invariants, check_invariant_03_state_transition,
    check_invariant_04_evidence_required, check_invariant_05_full_coverage_required,
    check_invariant_06_override_human_only, check_invariant_07_tenant_id_present,
    check_invariant_08_evidence_storage_tenant_prefix,
    check_invariant_09_policy_default_ai_self_claim, check_invariant_10_evidence_type_whitelist,
    check_status_transition,
};
pub use port::{
    AddEvidenceCommand, CreateValidationPolicyCommand, LinkAcceptanceEvidenceCommand,
    LinkEvidenceCommand, ListValidationQuery, MarkValidationStatusCommand,
    OverrideValidationCommand, SubmitValidationResultCommand, ValidationCommandPort,
    ValidationQueryPort, ValidationRepository,
};
pub use service::InMemoryValidationService;
pub use star_context::ActorContext; // 收敛到 star_context 权威版本 (per P0-1 联动协作)
pub use uuid::Uuid;
pub use value_object::{
    is_valid_state_transition, roles, AcceptanceCoverageId, AcceptanceCriterionId, AgentSessionId,
    ChangeSetId, CommitId, CoverageStatus, EvidenceType, ProjectId, TenantId, TriggeredBy, UserId,
    ValidationEvidenceId, ValidationId, ValidationKind, ValidationOverrideId, ValidationPolicyId,
    ValidationStatus, WorkItemId, WorktreeId,
};

// =====================================================================
// 单元测试(SOW 要求 5+ 场景;7 类 + Coverage)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ActorContext; // P0-1 兼容: 显式覆盖 super::* 的 star_context 命名
    use crate::value_object::{
        ProjectId, TenantId, TriggeredBy, UserId, ValidationKind, ValidationStatus, WorkItemId,
    };

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id).with_role(roles::DEVELOPER)
    }

    fn make_service_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id).with_role(roles::SERVICE_INTERNAL)
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

    // -------- 1. 字段数审计(19/9/10/9/6) --------

    #[test]
    fn entity_field_count_audit() {
        assert_eq!(ValidationResult::FIELD_COUNT, 19);
        assert_eq!(ValidationEvidence::FIELD_COUNT, 9);
        assert_eq!(AcceptanceCoverage::FIELD_COUNT, 10);
        assert_eq!(ValidationPolicy::FIELD_COUNT, 9);
        assert_eq!(ValidationOverride::FIELD_COUNT, 6);
    }

    // -------- 2. 7 类 ValidationKind 锁定(SOW) --------

    #[test]
    fn seven_validation_kinds_locked() {
        // SOW 必交付 7 类
        assert_eq!(ValidationKind::SOW_REQUIRED.len(), 7);
        for k in ValidationKind::SOW_REQUIRED {
            assert!(k.is_sow_required());
        }
        // 不在 SOW 集合的
        assert!(!ValidationKind::AcceptanceCheck.is_sow_required());
        assert!(!ValidationKind::Review.is_sow_required());
        assert!(!ValidationKind::CustomValidation.is_sow_required());
    }

    // -------- 3. 5 状态机迁移表 --------

    #[test]
    fn five_state_transitions() {
        // 合法
        assert!(is_valid_state_transition(
            ValidationStatus::Pending,
            ValidationStatus::Running
        ));
        assert!(is_valid_state_transition(
            ValidationStatus::Running,
            ValidationStatus::Passed
        ));
        assert!(is_valid_state_transition(
            ValidationStatus::Running,
            ValidationStatus::Failed
        ));
        assert!(is_valid_state_transition(
            ValidationStatus::Pending,
            ValidationStatus::Skipped
        ));
        // 终态不可迁出
        assert!(!is_valid_state_transition(
            ValidationStatus::Passed,
            ValidationStatus::Running
        ));
        assert!(!is_valid_state_transition(
            ValidationStatus::Failed,
            ValidationStatus::Running
        ));
        assert!(!is_valid_state_transition(
            ValidationStatus::Skipped,
            ValidationStatus::Running
        ));
        // 同态禁止
        assert!(!is_valid_state_transition(
            ValidationStatus::Running,
            ValidationStatus::Running
        ));
    }

    // -------- 4. submit_result 7 类各能成功创建 + INV-VL-04 evidence 必带 --------

    #[tokio::test]
    async fn submit_seven_kinds_all_succeed() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        for (i, kind) in ValidationKind::SOW_REQUIRED.iter().enumerate() {
            let cmd = make_submit_cmd(TenantId(tenant_id), *kind);
            let r = svc
                .submit_result(cmd, actor.clone())
                .await
                .expect("submit 成功");
            assert_eq!(r.kind, *kind);
            assert_eq!(r.status, ValidationStatus::Pending);
            assert!(r.log_excerpt_ref.is_some());
            assert!(!r.is_ai_complete_claim);
            // 用于 SOW 测试断言计数
            assert_eq!(svc.result_count().await, i + 1);
        }
    }

    // -------- 5. INV-VL-04:log_excerpt_ref 缺失必拒(VAL-001) --------

    #[tokio::test]
    async fn invariant_04_evidence_required_reject_empty_log_ref() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        let mut cmd = make_submit_cmd(TenantId(tenant_id), ValidationKind::Build);
        cmd.log_excerpt_ref = "   ".to_string();
        let res = svc.submit_result(cmd, actor).await;
        assert!(matches!(res, Err(ValidationError::InvalidState(_))));
    }

    // -------- 6. 状态机:Running -> Passed 触发 PASSED 事件 + evidence 必带 --------

    #[tokio::test]
    async fn state_transition_running_to_passed_emits_event() {
        let (svc, mut rx) = InMemoryValidationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        let r = svc
            .submit_result(
                make_submit_cmd(TenantId(tenant_id), ValidationKind::UnitTest),
                actor.clone(),
            )
            .await
            .unwrap();
        // Running
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
        // Passed
        let passed = svc
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
        assert_eq!(passed.status, ValidationStatus::Passed);
        // 查事件
        let mut found_passed = false;
        for _ in 0..10 {
            if let Ok(e) = rx.try_recv() {
                if matches!(e, ValidationEvent::Passed(_)) {
                    found_passed = true;
                    break;
                }
            }
        }
        assert!(found_passed, "应收到 ValidationPassed 事件");
    }

    // -------- 7. INV-VL-05:AcceptanceCoverage 100% 派生 + 未达 100% 拒绝 --------

    #[tokio::test]
    async fn acceptance_coverage_100_percent_derived() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        // 提交 3 个 PASSED Validation,关联到 3 个 AC
        let work_item = WorkItemId::new();
        for _ in 0..3 {
            let r = svc
                .submit_result(
                    SubmitValidationResultCommand {
                        work_item_id: Some(work_item),
                        ..make_submit_cmd(TenantId(tenant_id), ValidationKind::AcceptanceCheck)
                    },
                    actor.clone(),
                )
                .await
                .unwrap();
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
            svc.link_to_acceptance_criterion(
                LinkAcceptanceEvidenceCommand {
                    tenant_id: TenantId(tenant_id),
                    work_item_id: work_item,
                    acceptance_criterion_id: uuid::Uuid::new_v4(),
                    validation_id: r.id,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        }
        let report = svc
            .get_acceptance_coverage(work_item, actor.clone())
            .await
            .unwrap();
        assert_eq!(report.total_criteria, 3);
        assert_eq!(report.covered, 3);
        assert!(report.is_fully_covered());
        assert!((report.coverage_percent() - 100.0).abs() < 0.01);

        // INV-VL-05:未达 100% 拒绝
        let work_item2 = WorkItemId::new();
        let res = check_invariant_05_full_coverage_required(3, 2);
        assert!(matches!(res, Err(ValidationError::InvalidState(_))));
        let _ = work_item2;
    }

    // -------- 8. INV-VL-06:Service-Internal 不可 Override(必须人类) --------

    #[tokio::test]
    async fn invariant_06_override_human_only_rejects_service() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let svc_actor = make_service_actor(TenantId(tenant_id));
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
                    reason: "test".to_string(),
                    approver_user_id: UserId::new(),
                },
                svc_actor,
            )
            .await;
        assert!(matches!(res, Err(ValidationError::PermissionDenied)));

        // 人类 Developer 可 Override
        let dev_actor = make_test_actor(TenantId(tenant_id));
        let ovr = svc
            .override_result(
                OverrideValidationCommand {
                    tenant_id: TenantId(tenant_id),
                    validation_id: r.id,
                    reason: "测试覆盖".to_string(),
                    approver_user_id: dev_actor.user_id,
                },
                dev_actor,
            )
            .await
            .expect("Override 成功");
        assert_eq!(ovr.validation_id, r.id);
    }

    // -------- 9. INV-VL-08:Evidence storage_ref 缺 tenant_id 前缀被拒 --------

    #[tokio::test]
    async fn invariant_08_evidence_storage_tenant_prefix_rejected() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        let r = svc
            .submit_result(
                make_submit_cmd(TenantId(tenant_id), ValidationKind::Build),
                actor.clone(),
            )
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

    // -------- 10. INV-VL-09:Policy allow_ai_self_claim=true 必拒(VAL-001) --------

    #[tokio::test]
    async fn invariant_09_policy_allow_ai_self_claim_rejected() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(TenantId(tenant_id));
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

    // -------- 11. 跨 tenant 访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let actor_a = make_service_actor(TenantId(tenant_a));
        let r = svc
            .submit_result(make_submit_cmd(TenantId(tenant_a), ValidationKind::Build), actor_a)
            .await
            .unwrap();
        let actor_b = make_service_actor(TenantId(tenant_b));
        let res = svc.get_result(r.id, actor_b).await;
        assert!(matches!(res, Err(ValidationError::PermissionDenied)));
    }

    // -------- 12. ValidationFailed 触发 FeedbackRequired 事件 --------

    #[tokio::test]
    async fn validation_failed_triggers_feedback_required_event() {
        let (svc, mut rx) = InMemoryValidationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        let work_item = WorkItemId::new();
        let r = svc
            .submit_result(
                SubmitValidationResultCommand {
                    work_item_id: Some(work_item),
                    ..make_submit_cmd(TenantId(tenant_id), ValidationKind::UnitTest)
                },
                actor.clone(),
            )
            .await
            .unwrap();
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
                new_status: ValidationStatus::Failed,
                failure_summary: Some("Test XYZ failed".to_string()),
            },
            actor,
        )
        .await
        .unwrap();
        // 事件队列中应能找到 FeedbackRequired
        let mut found_fr = false;
        let mut found_failed = false;
        for _ in 0..20 {
            if let Ok(e) = rx.try_recv() {
                match e {
                    ValidationEvent::FeedbackRequired(_) => found_fr = true,
                    ValidationEvent::Failed(_) => found_failed = true,
                    _ => {}
                }
                if found_fr && found_failed {
                    break;
                }
            }
        }
        assert!(found_fr, "应收到 FeedbackRequired 事件");
        assert!(found_failed, "应收到 ValidationFailed 事件");
    }

    // -------- 13. INV-VL-01 + AI self claim + status=Passed 必带 evidence --------

    #[tokio::test]
    async fn ai_self_claim_requires_evidence_for_passed() {
        let svc = InMemoryValidationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_service_actor(TenantId(tenant_id));
        // is_ai_complete_claim=true 但 log_excerpt_ref 为空 → submit 即拒
        let mut cmd = make_submit_cmd(TenantId(tenant_id), ValidationKind::Build);
        cmd.is_ai_complete_claim = true;
        cmd.log_excerpt_ref = "".to_string();
        let res = svc.submit_result(cmd, actor).await;
        assert!(matches!(res, Err(ValidationError::InvalidState(_))));

        // 正常 submit 后尝试 mark_status=Passed 但 evidence 缺
        let actor2 = make_service_actor(TenantId(tenant_id));
        let r = svc
            .submit_result(
                make_submit_cmd(TenantId(tenant_id), ValidationKind::UnitTest),
                actor2.clone(),
            )
            .await
            .unwrap();
        // 清除 log_excerpt_ref,模拟 evidence 缺失
        {
            let mut results = svc.results.write().unwrap();
            if let Some(r) = results.get_mut(&r.id) {
                r.log_excerpt_ref = None;
                r.evidence_ids.clear();
            }
        }
        svc.mark_status(
            MarkValidationStatusCommand {
                tenant_id: TenantId(tenant_id),
                validation_id: r.id,
                new_status: ValidationStatus::Running,
                failure_summary: None,
            },
            actor2.clone(),
        )
        .await
        .unwrap();
        let res = svc
            .mark_status(
                MarkValidationStatusCommand {
                    tenant_id: TenantId(tenant_id),
                    validation_id: r.id,
                    new_status: ValidationStatus::Passed,
                    failure_summary: None,
                },
                actor2,
            )
            .await;
        assert!(matches!(res, Err(ValidationError::InvalidState(_))));
    }

    // 静默引用
    #[allow(dead_code)]
    fn _unused_tb(_: TriggeredBy) -> TriggeredBy {
        TriggeredBy::User
    }
}

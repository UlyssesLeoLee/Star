//! 集成测试 — domain-defect 缺陷生命周期验证
//!
//! 验证维度:
//! - IT-V0: 基础 create / get / list（跨 tenant、跨 project 拒绝）
//! - IT-V1: 状态机受控流转（禁止任意状态直接跳 Closed；返回具体原因）
//! - IT-V2: 首次提交修复版本 + 独立验证（验证人 != 报告人，版本匹配）
//! - IT-V3: 验证失败 / 关闭后重开，保留原记录与历史
//! - IT-V4: 版本冲突（expected_version 不匹配）
//! - IT-V5: 幂等重试（同一 idempotency_key 返回原记录）

use domain_defect::{
    CreateDefectCommand, Defect, DefectCommandPort, DefectError, DefectId, DefectQueryPort,
    DefectStatus, InMemoryDefectService, IdempotencyKey, SubmitVerificationCommand,
    TransitionDefectCommand, VerificationResult,
};
use domain_work_item::WorkItemId;
use star_context::ActorContext;
use uuid::Uuid;

fn make_dev_actor(tenant_id: Uuid, user_id: Uuid, project_id: Uuid) -> ActorContext {
    ActorContext::new(user_id, tenant_id)
        .with_role("developer")
        .with_project(project_id)
}

fn make_admin_actor(tenant_id: Uuid, user_id: Uuid, project_id: Uuid) -> ActorContext {
    ActorContext::new(user_id, tenant_id)
        .with_role("tenant_admin")
        .with_project(project_id)
}

fn basic_cmd(tenant_id: Uuid, project_id: Uuid, reporter_id: Uuid) -> CreateDefectCommand {
    CreateDefectCommand {
        idempotency_key: None,
        tenant_id,
        project_id,
        lane_id: None,
        worktree_id: None,
        branch_id: None,
        branch_name: None,
        title: "login crash".to_string(),
        phenomenon: "click submit raises 500".to_string(),
        expected_result: Some("success".to_string()),
        actual_result: Some("500 error".to_string()),
        reproduction_steps: Some("step1: open page; step2: click submit".to_string()),
        impact_scope: Some("auth module".to_string()),
        discovery_stage: Some("implementation".to_string()),
        severity: Some("S2".to_string()),
        priority: Some("P2".to_string()),
        reporter_id,
        environment: Some("staging".to_string()),
        labels: vec!["bug".to_string()],
        discovery_version: Some("v1.0.0".to_string()),
    }
}

// =========================================================================
// IT-V0: 基础 CRUD（跨 tenant / 跨 project 拒绝）
// =========================================================================

/// IT-V0-1: 同 tenant/same project create → get → list 闭环
#[tokio::test]
async fn it_v0_create_get_list() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let cmd = basic_cmd(tenant_id, project_id, actor.user_id);

    let defect = svc.create_defect(cmd.clone(), &actor).await.expect("create 必成功");
    assert_eq!(defect.status, DefectStatus::Triage);
    assert_eq!(defect.tenant_id, tenant_id);
    assert_eq!(defect.project_id, project_id);
    assert_eq!(defect.lifecycle_version, 1);

    let got = svc.get_defect(defect.id, &actor).await.expect("get 必成功");
    assert_eq!(got.id, defect.id);

    let list = svc
        .list_defects(project_id, None, &actor)
        .await
        .expect("list 必成功");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, defect.id);
}

/// IT-V0-2: 跨 tenant 创建被拒绝
#[tokio::test]
async fn it_v0_cross_tenant_create_rejected() {
    let svc = InMemoryDefectService::new();
    let actor_tenant = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();
    let actor = make_dev_actor(actor_tenant, Uuid::new_v4(), Uuid::new_v4());
    let mut cmd = basic_cmd(actor_tenant, Uuid::new_v4(), actor.user_id);
    cmd.tenant_id = other_tenant; // 指定其他 tenant

    let err = svc
        .create_defect(cmd.clone(), &actor)
        .await
        .expect_err("跨 tenant 创建必须被拒绝");
    assert!(matches!(err, DefectError::CrossTenantDenied(_, _)));
}

/// IT-V0-3: 跨 project 创建被拒绝（actor 无权访问该 project）
#[tokio::test]
async fn it_v0_cross_project_create_rejected() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let actor_tenant_project = Uuid::new_v4();
    let other_project = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), actor_tenant_project);
    // Actor 只能访问 actor_tenant_project，命令指定 other_project
    let mut cmd = basic_cmd(tenant_id, other_project, actor.user_id);
    cmd.tenant_id = tenant_id;

    let err = svc
        .create_defect(cmd.clone(), &actor)
        .await
        .expect_err("跨 project 创建必须被拒绝");
    assert!(matches!(err, DefectError::CrossProjectDenied(_, _)));
}

// =========================================================================
// IT-V1: 状态机受控流转（禁止任意状态直接跳 Closed）
// =========================================================================

async fn transition(
    svc: &InMemoryDefectService,
    defect_id: DefectId,
    target: DefectStatus,
    actor: &ActorContext,
    fix_version: Option<&str>,
) -> Result<Defect, DefectError> {
    svc.transition_status(
        defect_id,
        TransitionDefectCommand {
            idempotency_key: None,
            target_status: target,
            expected_version: None,
            fix_version: fix_version.map(|s| s.to_string()),
        },
        actor,
    )
    .await
}

/// IT-V1-1: 正常流转 Triage → Confirmed → InProgress → PendingVerify → Closed
#[tokio::test]
async fn it_v1_normal_flow() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let cmd = basic_cmd(tenant_id, project_id, actor.user_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();

    let d = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    assert_eq!(d.status, DefectStatus::Confirmed);

    let d = transition(&svc, defect.id, DefectStatus::InProgress, &actor, Some("v1.0.1-fix")).await.unwrap();
    assert_eq!(d.status, DefectStatus::InProgress);

    let d = transition(&svc, defect.id, DefectStatus::PendingVerify, &actor, Some("v1.0.1-fix")).await.unwrap();
    assert_eq!(d.status, DefectStatus::PendingVerify);

    // 尚未设定 fix_version，直接跳 Closed 应被拒绝（必须经过 PendingVerify 且有 fix_version）
    // 注：这里状态层面 PendingVerify->Closed 是允许的，业务层 fix_version 检查在 submit_verification
    let d = transition(&svc, defect.id, DefectStatus::Closed, &actor, None).await.unwrap();
    assert_eq!(d.status, DefectStatus::Closed);
    assert_eq!(d.lifecycle_version, 5);
}

/// IT-V1-2: 任意状态直接跳 Closed（绕过 PendingVerify）被拒绝
#[tokio::test]
async fn it_v1_no_direct_close() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let cmd = basic_cmd(tenant_id, project_id, actor.user_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();

    // Triage -> Closed 禁止
    let err = transition(&svc, defect.id, DefectStatus::Closed, &actor, None)
        .await
        .expect_err("Triage -> Closed 必须被拒绝");
    assert!(matches!(err, DefectError::InvalidTransition { .. }));

    // Confirmed -> Closed 禁止
    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    let err = transition(&svc, defect.id, DefectStatus::Closed, &actor, None)
        .await
        .expect_err("Confirmed -> Closed 必须被拒绝");
    assert!(matches!(err, DefectError::InvalidTransition { .. }));
}

/// IT-V1-3: Duplicate / NoAction 为终态，不可再流转
#[tokio::test]
async fn it_v1_terminal_states() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let cmd = basic_cmd(tenant_id, project_id, actor.user_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();

    let _ = transition(&svc, defect.id, DefectStatus::Duplicate, &actor, None).await.unwrap();
    let err = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None)
        .await
        .expect_err("Duplicate -> Confirmed 必须被拒绝");
    assert!(matches!(err, DefectError::InvalidTransition { .. }));

    let defect2 = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    let _ = transition(&svc, defect2.id, DefectStatus::NoAction, &actor, None).await.unwrap();
    let err = transition(&svc, defect2.id, DefectStatus::Triage, &actor, None)
        .await
        .expect_err("NoAction -> Triage 必须被拒绝");
    assert!(matches!(err, DefectError::InvalidTransition { .. }));
}

// =========================================================================
// IT-V2: 首次提交修复版本 + 独立验证
// =========================================================================

/// IT-V2-1: 首次提交修复版本（验证通过 → Closed），验证记录保存全部字段
#[tokio::test]
async fn it_v2_first_submit_verification_passed() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let reporter_id = Uuid::new_v4();
    let verifier_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, verifier_id, project_id);
    let cmd = basic_cmd(tenant_id, project_id, reporter_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();

    // 进入 PendingVerify（非验证人操作也可流转状态，但 submit 需验证权限）
    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::InProgress, &actor, Some("v1.0.1-fix")).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::PendingVerify, &actor, Some("v1.0.1-fix")).await.unwrap();

    // 提交验证：验证人 != 报告人（独立性），fix_version 匹配
    let vcmd = SubmitVerificationCommand {
        idempotency_key: None,
        fix_version: "v1.0.1-fix".to_string(),
        verification_method: Some("manual replay".to_string()),
        result: true,
        evidence_id: None,
    };
    let (_d, v) = svc
        .submit_verification(defect.id, vcmd, &actor)
        .await
        .expect("验证提交必成功");
    assert_eq!(v.result, VerificationResult::Passed);
    assert_eq!(v.fix_version, "v1.0.1-fix");
    assert_eq!(v.method.as_deref(), Some("manual replay"));
    assert_eq!(v.verifier_id, verifier_id);
    assert!(v.evidence_id.is_none());
    assert!(v.created_at <= Utc::now());

    // 缺陷应已关闭
    let got = svc.get_defect(defect.id, &actor).await.unwrap();
    assert_eq!(got.status, DefectStatus::Closed);
}

/// IT-V2-2: 验证独立性——验证人为报告人被拒绝（非管理员）
#[tokio::test]
async fn it_v2_verifier_is_reporter_rejected() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let same_user = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, same_user, project_id); // reporter == verifier
    let cmd = basic_cmd(tenant_id, project_id, same_user);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();

    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::InProgress, &actor, Some("v1.0.1-fix")).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::PendingVerify, &actor, Some("v1.0.1-fix")).await.unwrap();
    let vcmd = SubmitVerificationCommand {
        idempotency_key: None,
        fix_version: "v1.0.1-fix".to_string(),
        verification_method: None,
        result: true,
        evidence_id: None,
    };
    let err = svc
        .submit_verification(defect.id, vcmd, &actor)
        .await
        .expect_err("验证人是报告人必须被拒绝");
    assert!(matches!(err, DefectError::PermissionDenied(_)));
}

/// IT-V2-3: 提交的 fix_version 与缺陷当前不匹配被拒绝
#[tokio::test]
async fn it_v2_fix_version_mismatch_rejected() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let reporter_id = Uuid::new_v4();
    let verifier_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, verifier_id, project_id);
    let cmd = basic_cmd(tenant_id, project_id, reporter_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::InProgress, &actor, Some("v1.0.1-fix")).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::PendingVerify, &actor, Some("v1.0.1-fix")).await.unwrap();

    let vcmd = SubmitVerificationCommand {
        idempotency_key: None,
        fix_version: "wrong-version".to_string(),
        verification_method: None,
        result: true,
        evidence_id: None,
    };
    let err = svc
        .submit_verification(defect.id, vcmd, &actor)
        .await
        .expect_err("fix_version 不匹配必须被拒绝");
    assert!(matches!(err, DefectError::VerificationMismatch(_)));
}

// =========================================================================
// IT-V3: 验证失败 / 关闭后重开，保留原记录与历史
// =========================================================================

/// IT-V3-1: 验证失败 → Reopened → InProgress，原记录保留
#[tokio::test]
async fn it_v3_verification_failed_reopen() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let reporter_id = Uuid::new_v4();
    let verifier_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, verifier_id, project_id);
    let cmd = basic_cmd(tenant_id, project_id, reporter_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::InProgress, &actor, Some("v1.0.1-fix")).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::PendingVerify, &actor, Some("v1.0.1-fix")).await.unwrap();

    let vcmd = SubmitVerificationCommand {
        idempotency_key: None,
        fix_version: "v1.0.1-fix".to_string(),
        verification_method: None,
        result: false,
        evidence_id: None,
    };
    let (_d, v) = svc
        .submit_verification(defect.id, vcmd, &actor)
        .await
        .expect("验证失败提交必成功");
    assert_eq!(v.result, VerificationResult::Failed);
    assert_eq!(v.defect_id, defect.id);

    let got = svc.get_defect(defect.id, &actor).await.unwrap();
    assert_eq!(got.status, DefectStatus::Reopened);

    // 重开后可继续处理
    let d = transition(&svc, defect.id, DefectStatus::InProgress, &actor, None).await.unwrap();
    assert_eq!(d.status, DefectStatus::InProgress);

    // 原验证记录仍可查询（保留历史）
    let vers = svc
        .list_verifications(defect.id, &actor)
        .await
        .expect("list_verifications 必成功");
    assert_eq!(vers.len(), 1);
    assert_eq!(vers[0].result, VerificationResult::Failed);
}

/// IT-V3-2: 已关闭后重开（Closed → Reopened → InProgress）
#[tokio::test]
async fn it_v3_closed_reopen() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let reporter_id = Uuid::new_v4();
    let verifier_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, verifier_id, project_id);
    let cmd = basic_cmd(tenant_id, project_id, reporter_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::InProgress, &actor, Some("v1.0.1-fix")).await.unwrap();
    let _ = transition(&svc, defect.id, DefectStatus::PendingVerify, &actor, Some("v1.0.1-fix")).await.unwrap();

    let vcmd = SubmitVerificationCommand {
        idempotency_key: None,
        fix_version: "v1.0.1-fix".to_string(),
        verification_method: None,
        result: true,
        evidence_id: None,
    };
    svc.submit_verification(defect.id, vcmd, &actor).await.unwrap();

    let got = svc.get_defect(defect.id, &actor).await.unwrap();
    assert_eq!(got.status, DefectStatus::Closed);

    // Closed → Reopened（允许）
    let d = transition(&svc, defect.id, DefectStatus::Reopened, &actor, None).await.unwrap();
    assert_eq!(d.status, DefectStatus::Reopened);

    // Reopened → InProgress（允许）
    let d = transition(&svc, defect.id, DefectStatus::InProgress, &actor, None).await.unwrap();
    assert_eq!(d.status, DefectStatus::InProgress);
}

// =========================================================================
// IT-V4: 版本冲突（expected_version 不匹配）
// =========================================================================

/// IT-V4-1: 并发修改导致版本冲突
#[tokio::test]
async fn it_v4_version_conflict() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let cmd = basic_cmd(tenant_id, project_id, actor.user_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();

    // 第一次修改成功，版本 1 -> 2
    let _ = transition(&svc, defect.id, DefectStatus::Confirmed, &actor, None).await.unwrap();

    // 第二次基于旧版本（expected_version=1）再提交应冲突
    let err = svc
        .transition_status(
            defect.id,
            TransitionDefectCommand {
                idempotency_key: None,
                target_status: DefectStatus::InProgress,
                expected_version: Some(1), // 当前已是 2
                fix_version: None,
            },
            &actor,
        )
        .await
        .expect_err("版本冲突必须被拒绝");
    assert!(matches!(err, DefectError::Conflict(_)));
}

// =========================================================================
// IT-V5: 幂等重试（同一 idempotency_key 返回原记录）
// =========================================================================

/// IT-V5-1: 重复创建命令（同 key）返回原缺陷，不新增记录
#[tokio::test]
async fn it_v5_idempotent_create() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let mut cmd = basic_cmd(tenant_id, project_id, actor.user_id);
    let key = IdempotencyKey::new();
    cmd.idempotency_key = Some(key.clone());

    let d1 = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    let d2 = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    assert_eq!(d1.id, d2.id);

    // 只应有一条记录
    let list = svc
        .list_defects(project_id, None, &actor)
        .await
        .expect("list 必成功");
    assert_eq!(list.len(), 1);
}

/// IT-V5-2: 重复状态流转命令（同 key）返回原缺陷，版本不累加
#[tokio::test]
async fn it_v5_idempotent_transition() {
    let svc = InMemoryDefectService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, Uuid::new_v4(), project_id);
    let cmd = basic_cmd(tenant_id, project_id, actor.user_id);
    let defect = svc.create_defect(cmd.clone(), &actor).await.unwrap();
    let key = IdempotencyKey::new();

    let mut cmd1 = TransitionDefectCommand {
        idempotency_key: Some(key.clone()),
        target_status: DefectStatus::Confirmed,
        expected_version: None,
        fix_version: None,
    };
    let d1 = svc.transition_status(defect.id, cmd1.clone(), &actor).await.unwrap();
    let d2 = svc.transition_status(defect.id, cmd1.clone(), &actor).await.unwrap();
    assert_eq!(d1.id, d2.id);
    assert_eq!(d2.lifecycle_version, 2); // 仅增加一次
}

use chrono::Utc;

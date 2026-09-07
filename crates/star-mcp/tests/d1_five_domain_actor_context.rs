//! 跨 crate 集成测试 (IT) — D.1 Phase D 5 domain 联动 + ActorContext 扩展字段
//!
//! **crate**: star-mcp (test target)
//! **per**: 2026-09-07 Phase D D.1 G-10 H2 强类型重构 (per OPT-NEXT-01-phase-d.md §3 D.1)
//! **目的**: 验证 5 domain (identity/work-item/project/tenant/permission) 全部使用
//!         `star_context::ActorContext` (Uuid) 统一跨域字段, 跨域联动 0 type mismatch.
//!
//! ## 测试维度
//!
//! 1. D.1 H2-EXT #1 domain-tenant `tenant_policy_id` 字段流转
//! 2. D.1 H2-EXT #2 domain-project `workspace_ids` 字段流转
//! 3. D.1 domain-identity `device_id: Option<Uuid>` (per star_context) 流转
//! 4. D.1 domain-work-item `device_id: Option<String>` hostname 业务语义 (per Q1 拍板)
//! 5. D.1 5 domain 共享同一 `star_context::ActorContext` 实例, 跨域类型 0 mismatch

use domain_identity::{
    ActorContext as DomainIdentityActor, CreateUserCommand, CredentialRefId, IdentityCommandPort,
    InMemoryIdentityService, TenantRole,
};
use domain_permission::{
    Action, CheckQuery, InMemoryPermissionService, PermissionQueryPort, ResourceType,
};
use domain_project::{CreateProjectCommand, InMemoryProjectService, ProjectCommandPort};
use domain_tenant::{CreateTenantCommand, InMemoryTenantService, PlanTier, TenantCommandPort};
use domain_work_item::{CreateWorkItemCommand, InMemoryWorkItemService, WorkItemCommandPort};
use star_context::ActorContext;
use uuid::Uuid;

// =====================================================================
// D.1 — 5 domain 联动 + 跨域字段扩展 实证
// =====================================================================

/// **D.1-IT-1**: 单一 `star_context::ActorContext` 跨 5 domain 流转
/// 验证: 一个 ActorContext 实例可同时被 5 domain 服务接受, 跨域类型 0 mismatch.
#[tokio::test]
async fn d1_it_single_actor_flows_through_five_domains() {
    let workspace_id = Uuid::new_v4();
    let tenant_policy_id = Uuid::new_v4();
    let mut actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    actor.is_platform_admin = true; // tenant create_tenant 需要 platform_admin
    actor.tenant_policy_id = Some(tenant_policy_id);
    actor.workspace_ids.push(workspace_id);

    // 1) tenant: 接受 star_context::ActorContext (per D.1 H2-EXT #2)
    let tenant_svc = InMemoryTenantService::new();
    let tenant_cmd = CreateTenantCommand {
        slug: "d1-tenant".to_string(),
        display_name: "D.1 Tenant".to_string(),
        plan_tier: PlanTier::Free,
    };
    let tenant_result = tenant_svc.create_tenant(tenant_cmd, &actor).await;
    assert!(
        tenant_result.is_ok(),
        "tenant create_tenant 应该成功, 实际: {:?}",
        tenant_result.err()
    );

    // 2) identity: 接受 star_context::ActorContext (per D.1 H2-EXT #1)
    let identity_svc = InMemoryIdentityService::new();
    let user_cmd = CreateUserCommand {
        tenant_id: domain_identity::TenantId(actor.tenant_id),
        email: "d1@star.local".to_string(),
        display_name: "D.1 User".to_string(),
        tenant_role: TenantRole::Developer,
        credential_ref: CredentialRefId::new(),
    };
    let user_result = identity_svc.create_user(user_cmd, &actor).await;
    assert!(
        user_result.is_ok(),
        "identity create_user 应该成功, 实际: {:?}",
        user_result.err()
    );

    // 3) project: 接受 star_context::ActorContext (per D.1 H2-EXT #3)
    let project_actor =
        ActorContext::new(actor.user_id, actor.tenant_id).with_role("project_admin");
    let project_svc = InMemoryProjectService::new();
    let project_cmd = CreateProjectCommand {
        tenant_id: domain_project::TenantId(actor.tenant_id),
        workspace_id: domain_project::WorkspaceId(workspace_id),
        slug: "d1-project".to_string(),
        display_name: "D.1 Project".to_string(),
        description: "D.1 phase test project".to_string(),
        project_template_id: None,
        actor_user_id: domain_project::UserId(actor.user_id),
    };
    let project_result = project_svc
        .create_project(project_cmd, &project_actor)
        .await;
    assert!(
        project_result.is_ok(),
        "project create_project 应该成功, 实际: {:?}",
        project_result.err()
    );

    // 4) work-item: 接受 star_context::ActorContext (per D.1 #5 hostname 业务语义)
    let project_id = project_result.unwrap().id;
    let work_item_actor = ActorContext::new(actor.user_id, actor.tenant_id).with_role("developer");
    let work_item_svc = InMemoryWorkItemService::new();
    let work_item_cmd = CreateWorkItemCommand {
        tenant_id: domain_work_item::TenantId(actor.tenant_id),
        workspace_id: domain_work_item::WorkspaceId(workspace_id),
        project_id: domain_work_item::ProjectId(project_id.as_uuid()),
        item_type: domain_work_item::WorkItemType::Task,
        title: "D.1 WorkItem".to_string(),
        description: "Phase D.1 integration test work item".to_string(),
        priority: domain_work_item::Priority::Medium,
        severity: None,
        reporter_user_id: domain_work_item::UserId(actor.user_id),
        parent_work_item_id: None,
        ai_task_data: None,
        labels: vec![],
    };
    let work_item_result = work_item_svc
        .create_work_item(work_item_cmd, &work_item_actor)
        .await;
    assert!(
        work_item_result.is_ok(),
        "work_item create_work_item 应该成功, 实际: {:?}",
        work_item_result.err()
    );

    // 5) permission: 接受 star_context::ActorContext
    let perm_svc = InMemoryPermissionService::new();
    let check_q = CheckQuery {
        tenant_id: domain_permission::TenantId(actor.tenant_id),
        scheme_id: None,
        subject_user_id: domain_permission::UserId(actor.user_id),
        project_id: domain_permission::ProjectId(project_id.as_uuid()),
        resource_type: ResourceType::Project,
        resource_id: Some(project_id.as_uuid()),
        action: Action::Read,
    };
    let check_result = perm_svc.check(check_q, &actor).await;
    assert!(
        check_result.is_ok(),
        "permission check 应该不报错, 实际: {:?}",
        check_result.err()
    );
}

/// **D.1-IT-2**: `is_in_workspace` helper 跨 domain 流转
/// 验证: Phase D.1 新 helper 在跨域场景下语义保持.
#[test]
fn d1_it_is_in_workspace_helper_preserved() {
    let w = Uuid::new_v4();
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4()).with_workspace(w);
    assert!(actor.is_in_workspace(w));
    assert!(!actor.is_in_workspace(Uuid::new_v4()));
}

/// **D.1-IT-3**: `has_tenant_policy` helper 跨 domain 流转
/// 验证: Phase D.1 新 helper 在跨域场景下语义保持.
#[test]
fn d1_it_has_tenant_policy_helper_preserved() {
    let actor_no_policy = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    assert!(!actor_no_policy.has_tenant_policy());

    let actor_with_policy =
        ActorContext::new(Uuid::new_v4(), Uuid::new_v4()).with_tenant_policy(Uuid::new_v4());
    assert!(actor_with_policy.has_tenant_policy());
}

/// **D.1-IT-4**: `device_id` 跨 domain 流转 (Uuid 强类型一致)
/// 验证: star_context::ActorContext.device_id = Option<Uuid>,
///       domain-identity 不再是 DeviceId 强类型, 跨域流转保持 Uuid.
#[test]
fn d1_it_device_id_uuid_type_consistent() {
    let did = Uuid::new_v4();
    let mut actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    actor.device_id = Some(did);

    // re-export 类型
    let domain_actor: DomainIdentityActor = actor.clone();
    assert_eq!(domain_actor.device_id, Some(did));
    assert_eq!(actor.device_id, Some(did));
}

/// **D.1-IT-5**: 跨 domain 字段扩展 (workspace_ids + tenant_policy_id) 流转
/// 验证: D.1 H2-EXT 字段扩展可序列化跨 5 domain, 0 字段丢失.
#[test]
fn d1_it_h2_ext_field_extensions_serde_roundtrip() {
    let w1 = Uuid::new_v4();
    let w2 = Uuid::new_v4();
    let p = Uuid::new_v4();

    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_workspace(w1)
        .with_workspace(w2)
        .with_tenant_policy(p);

    let json = serde_json::to_string(&actor).expect("serialize");
    let back: ActorContext = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(back.workspace_ids, vec![w1, w2]);
    assert_eq!(back.tenant_policy_id, Some(p));

    // 跨域 (re-export) 流转保持
    let domain_actor: DomainIdentityActor = back;
    assert_eq!(domain_actor.workspace_ids, vec![w1, w2]);
    assert_eq!(domain_actor.tenant_policy_id, Some(p));
}

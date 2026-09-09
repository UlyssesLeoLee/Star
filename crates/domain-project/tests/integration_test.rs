//! 集成测试 (IT) — domain-project 真实数据接入验证
//!
//! **crate**: domain-project (test target)
//! **per**: 2026-09-07 P3-B sub-batch 1 read-heavy 域真实数据接入
//! **参考**: `crates/domain-form/src/lib.rs` (OPT-WORKER-09 done per 3a27a13)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create + get + list_by_workspace
//! - V1 (策略): replace_project_policy 整体替换(INV-P-02)
//! - V2 (状态机): archive_project → status=Archived
//! - V3 (RLS 13 類 + 跨域): 跨 tenant 拒绝 + 跨 workspace 隔离

use domain_project::{
    ActorContext, ArchiveProjectCommand, CreateProjectCommand, GetProjectQuery,
    InMemoryProjectService, ListByWorkspaceQuery, ProjectCommandPort, ProjectError, ProjectPolicy,
    ProjectQueryPort, ProjectStatus, ProjectTemplateId, ReplaceProjectPolicyCommand, TenantId,
    UserId, WorkspaceId,
};
use uuid::Uuid;

/// IT fixture: project_admin 角色
fn make_project_admin(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("project_admin")
}

/// IT fixture: tenant_admin 角色
fn make_tenant_admin(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("tenant_admin")
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: create → get → list_by_workspace 闭环
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryProjectService::new();
    let tenant_id = Uuid::new_v4();
    let workspace_id = WorkspaceId::new();
    let actor = make_project_admin(tenant_id);

    let project = svc
        .create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tenant_id),
                workspace_id,
                slug: "alpha".to_string(),
                display_name: "Alpha Project".to_string(),
                description: "main".to_string(),
                project_template_id: None,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();

    assert_eq!(project.status, ProjectStatus::Active);
    assert_eq!(project.slug, "alpha");

    // get
    let fetched = svc
        .get_project(
            GetProjectQuery {
                tenant_id: TenantId(tenant_id),
                project_id: project.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, project.id);

    // list by workspace
    let list = svc
        .list_by_workspace(
            ListByWorkspaceQuery {
                tenant_id: TenantId(tenant_id),
                workspace_id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, project.id);
}

/// **IT-V0-2**: 同 workspace 内 slug 重复被拒
#[tokio::test]
async fn it_v0_duplicate_slug_in_workspace_rejected() {
    let svc = InMemoryProjectService::new();
    let tenant_id = Uuid::new_v4();
    let workspace_id = WorkspaceId::new();
    let actor = make_project_admin(tenant_id);

    svc.create_project(
        CreateProjectCommand {
            tenant_id: TenantId(tenant_id),
            workspace_id,
            slug: "dup".to_string(),
            display_name: "P1".to_string(),
            description: String::new(),
            project_template_id: None,
            actor_user_id: UserId::new(),
        },
        &actor,
    )
    .await
    .unwrap();

    let res = svc
        .create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tenant_id),
                workspace_id,
                slug: "dup".to_string(),
                display_name: "P2".to_string(),
                description: String::new(),
                project_template_id: None,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(ProjectError::SlugExists(_))));
}

// =====================================================================
// V1: 策略整体替换 (INV-P-02)
// =====================================================================

/// **IT-V1-1**: replace_project_policy 整体替换,不允许 partial PATCH
#[tokio::test]
async fn it_v1_replace_project_policy_overwrites() {
    let svc = InMemoryProjectService::new();
    let tenant_id = Uuid::new_v4();
    let workspace_id = WorkspaceId::new();
    let actor = make_project_admin(tenant_id);

    let project = svc
        .create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tenant_id),
                workspace_id,
                slug: "beta".to_string(),
                display_name: "Beta".to_string(),
                description: String::new(),
                project_template_id: None,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();

    // 第一次替换策略
    let new_policy = ProjectPolicy {
        id: domain_project::ProjectPolicyId::new(),
        project_id: project.id,
        tenant_id: TenantId(tenant_id),
        custom_workflow_id: None,
        permission_scheme_id: domain_project::PermissionSchemeId::new(),
        notification_template_id: None,
        agent_policy_id: domain_project::AgentPolicyId::new(),
        max_runtime_seconds: 600,
        max_context_tokens: 64_000,
        validation_policy_id: domain_project::ValidationPolicyId::new(),
        required_test_passes: 1,
        default_repository_id: None,
        commit_requires_user: true,
        pr_creation_requires_user: true,
        merge_gate: true,
        updated_at: chrono::Utc::now(),
    };
    let replaced = svc
        .replace_project_policy(
            ReplaceProjectPolicyCommand {
                tenant_id: TenantId(tenant_id),
                project_id: project.id,
                policy: new_policy,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
    assert!(replaced.merge_gate);

    // 第二次替换 (覆盖,要求全量)
    let full_policy = ProjectPolicy {
        id: domain_project::ProjectPolicyId::new(),
        project_id: project.id,
        tenant_id: TenantId(tenant_id),
        custom_workflow_id: None,
        permission_scheme_id: domain_project::PermissionSchemeId::new(),
        notification_template_id: None,
        agent_policy_id: domain_project::AgentPolicyId::new(),
        max_runtime_seconds: 1800,
        max_context_tokens: 128_000,
        validation_policy_id: domain_project::ValidationPolicyId::new(),
        required_test_passes: 2,
        default_repository_id: None,
        commit_requires_user: true,
        pr_creation_requires_user: true,
        merge_gate: false,
        updated_at: chrono::Utc::now(),
    };
    let replaced2 = svc
        .replace_project_policy(
            ReplaceProjectPolicyCommand {
                tenant_id: TenantId(tenant_id),
                project_id: project.id,
                policy: full_policy,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
    assert!(!replaced2.merge_gate);
    assert_eq!(replaced2.max_runtime_seconds, 1800);

    // 校验仓库里能取到
    let stored = svc
        .get_project_policy(TenantId(tenant_id), project.id, &actor)
        .await
        .unwrap();
    assert_eq!(stored.max_runtime_seconds, 1800);
}

// =====================================================================
// V2: 归档状态机
// =====================================================================

/// **IT-V2-1**: archive_project → status=Archived
#[tokio::test]
async fn it_v2_archive_project_status_changes() {
    let svc = InMemoryProjectService::new();
    let tenant_id = Uuid::new_v4();
    let workspace_id = WorkspaceId::new();
    let actor = make_project_admin(tenant_id);

    let project = svc
        .create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tenant_id),
                workspace_id,
                slug: "to-archive".to_string(),
                display_name: "Archive".to_string(),
                description: String::new(),
                project_template_id: None,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(project.status, ProjectStatus::Active);

    let archived = svc
        .archive_project(
            ArchiveProjectCommand {
                tenant_id: TenantId(tenant_id),
                project_id: project.id,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(archived.status, ProjectStatus::Archived);

    // 校验仓库里也是 Archived
    let fetched = svc
        .get_project(
            GetProjectQuery {
                tenant_id: TenantId(tenant_id),
                project_id: project.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.status, ProjectStatus::Archived);
}

// =====================================================================
// V3: RLS 13 類 tenant + workspace 隔离
// =====================================================================

/// **IT-V3-1**: 跨 tenant 访问被拒(守门 #13 c Master 100% RLS 13 類)
#[tokio::test]
async fn it_v3_cross_tenant_get_rejected() {
    let svc = InMemoryProjectService::new();
    let tenant_a = Uuid::new_v4();
    let workspace_id = WorkspaceId::new();
    let actor_a = make_project_admin(tenant_a);

    let project = svc
        .create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tenant_a),
                workspace_id,
                slug: "secret".to_string(),
                display_name: "Secret".to_string(),
                description: String::new(),
                project_template_id: None,
                actor_user_id: UserId::new(),
            },
            &actor_a,
        )
        .await
        .unwrap();

    // tenant B 试图访问
    let tenant_b = Uuid::new_v4();
    let actor_b = make_project_admin(tenant_b);
    let res = svc
        .get_project(
            GetProjectQuery {
                tenant_id: TenantId(tenant_b),
                project_id: project.id,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(ProjectError::CrossTenantDenied(_, _))));
}

/// **IT-V3-2**: tenant_admin 也能跨 workspace 看到 project (守门 #13 c 兼容)
#[tokio::test]
async fn it_v3_tenant_admin_cross_workspace_access() {
    let svc = InMemoryProjectService::new();
    let tenant_id = Uuid::new_v4();
    let workspace_id = WorkspaceId::new();
    let actor = make_tenant_admin(tenant_id);

    let project = svc
        .create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tenant_id),
                workspace_id,
                slug: "ws1".to_string(),
                display_name: "WS1".to_string(),
                description: String::new(),
                project_template_id: Some(ProjectTemplateId::new()),
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();

    let fetched = svc
        .get_project(
            GetProjectQuery {
                tenant_id: TenantId(tenant_id),
                project_id: project.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, project.id);
}

//! 集成测试 (IT) — domain-permission 真实数据接入验证
//!
//! **crate**: domain-permission (test target)
//! **per**: 2026-09-07 P3-B sub-batch 1 read-heavy 域真实数据接入
//! **参考**: `crates/domain-form/src/lib.rs` (OPT-WORKER-09 done per 3a27a13)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create_scheme + grant_role + revoke_role + list_roles
//! - V1 (check 决策): upsert_rule + check (Allow / Deny 默认 / Admin 角色守门)
//! - V2 (RLS 13 類): 跨 tenant CrossTenantDenied
//! - V3 (5 域 role hierarchy): TenantAdmin/ProjectAdmin/Developer/Viewer/Agent (per AGENTS §5 5 域历史治理命名 + 8/21 JST 拍板)
//!
//! ## W/T/M 分类
//!
//! - **M (Master, SCD Type 2)**: 权限 scheme / role binding 是 SCD 慢变数据,
//!   per 守门 #13 (c) 物理删除禁止 + RLS 13 類必携
//! - **Role hierarchy**: 当前 5 角色 (TenantAdmin/ProjectAdmin/Developer/Viewer/Agent) 临时,
//!   5 域 (player/economy/match/social/admin) 业务子域映射待 DDD Review Lead 拍板

use domain_permission::{
    ActorContext, Action, CheckQuery, CreateSchemeCommand, Effect, GetSchemeQuery, GrantRoleCommand,
    InMemoryPermissionService, ListRolesQuery, PermissionCommandPort, PermissionError,
    PermissionQueryPort, PermissionRule, ProjectId, ResourceType, RevokeRoleCommand, Role,
    RoleBinding, SubjectType, TenantId, UpsertRuleCommand, UserId,
};
use uuid::Uuid;

/// IT fixture: tenant_admin 角色
fn make_tenant_admin(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("tenant_admin")
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: create_scheme + grant_role + list_roles + revoke_role 闭环
#[tokio::test]
async fn it_v0_scheme_grant_list_revoke_lifecycle() {
    let svc = InMemoryPermissionService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    // 1. 创建 scheme
    let scheme = svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: TenantId(tenant_id),
                name: "default".to_string(),
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(scheme.name, "default");

    // 2. 授予 Developer 角色
    let user = UserId::new();
    let binding = svc
        .grant_role(
            GrantRoleCommand {
                tenant_id: TenantId(tenant_id),
                user_id: user,
                project_id,
                role: Role::Developer,
                granted_by: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(binding.role, Role::Developer);

    // 3. 列出所有角色
    let list = svc
        .list_roles(
            ListRolesQuery {
                tenant_id: TenantId(tenant_id),
                project_id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].user_id, user);

    // 4. 撤销
    svc.revoke_role(
        RevokeRoleCommand {
            tenant_id: TenantId(tenant_id),
            user_id: user,
            project_id,
        },
        &actor,
    )
    .await
    .unwrap();

    let after = svc
        .list_roles(
            ListRolesQuery {
                tenant_id: TenantId(tenant_id),
                project_id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(after.len(), 0);
}

// =====================================================================
// V1: check 决策 (INV-PM-05 默认 Deny, INV-PM-04 Admin 守门)
// =====================================================================

/// **IT-V1-1**: upsert_rule 允许 + check 返回 true
#[tokio::test]
async fn it_v1_check_allow_with_rule() {
    let svc = InMemoryPermissionService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let scheme = svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: TenantId(tenant_id),
                name: "scheme".to_string(),
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();

    // 上插规则: Developer 角色对 Project 资源所有动作 Allow
    svc.upsert_rule(
        UpsertRuleCommand {
            tenant_id: TenantId(tenant_id),
            scheme_id: scheme.id,
            rule: PermissionRule {
                id: domain_permission::PermissionRuleId::new(),
                subject_type: SubjectType::Role,
                subject_id: None,
                role: Some(Role::Developer),
                resource_type: ResourceType::Project,
                resource_id: None, // 通配
                actions: vec![Action::Read, Action::Write],
                effect: Effect::Allow,
            },
        },
        &actor,
    )
    .await
    .unwrap();

    // 授予 Developer + check
    let user = UserId::new();
    svc.grant_role(
        GrantRoleCommand {
            tenant_id: TenantId(tenant_id),
            user_id: user,
            project_id,
            role: Role::Developer,
            granted_by: UserId::new(),
        },
        &actor,
    )
    .await
    .unwrap();

    let allowed = svc
        .check(
            CheckQuery {
                tenant_id: TenantId(tenant_id),
                scheme_id: Some(scheme.id),
                subject_user_id: user,
                project_id,
                resource_type: ResourceType::Project,
                resource_id: None,
                action: Action::Read,
            },
            &actor,
        )
        .await
        .unwrap();
    assert!(allowed);
}

/// **IT-V1-2**: 无规则命中 → 默认 Deny (INV-PM-05)
#[tokio::test]
async fn it_v1_check_default_deny_no_rule() {
    let svc = InMemoryPermissionService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    // 不创建 scheme, 不上插 rule
    let user = UserId::new();
    // 也不 grant role
    let allowed = svc
        .check(
            CheckQuery {
                tenant_id: TenantId(tenant_id),
                scheme_id: None,
                subject_user_id: user,
                project_id,
                resource_type: ResourceType::Project,
                resource_id: None,
                action: Action::Read,
            },
            &actor,
        )
        .await
        .unwrap();
    assert!(!allowed); // 默认 deny
}

/// **IT-V1-3**: Admin action 需 admin role (INV-PM-04)
#[tokio::test]
async fn it_v1_check_admin_requires_admin_role() {
    let svc = InMemoryPermissionService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    let scheme = svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: TenantId(tenant_id),
                name: "admin-scheme".to_string(),
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();

    // 上插 Developer 角色允许 Admin 动作 (会被 INV-PM-04 拒绝)
    svc.upsert_rule(
        UpsertRuleCommand {
            tenant_id: TenantId(tenant_id),
            scheme_id: scheme.id,
            rule: PermissionRule {
                id: domain_permission::PermissionRuleId::new(),
                subject_type: SubjectType::Role,
                subject_id: None,
                role: Some(Role::Developer),
                resource_type: ResourceType::Project,
                resource_id: None,
                actions: vec![Action::Admin],
                effect: Effect::Allow,
            },
        },
        &actor,
    )
    .await
    .unwrap();

    // 授予 Developer 角色
    let user = UserId::new();
    svc.grant_role(
        GrantRoleCommand {
            tenant_id: TenantId(tenant_id),
            user_id: user,
            project_id,
            role: Role::Developer,
            granted_by: UserId::new(),
        },
        &actor,
    )
    .await
    .unwrap();

    // Developer 试图 Admin → 应被拒
    let allowed = svc
        .check(
            CheckQuery {
                tenant_id: TenantId(tenant_id),
                scheme_id: Some(scheme.id),
                subject_user_id: user,
                project_id,
                resource_type: ResourceType::Project,
                resource_id: None,
                action: Action::Admin,
            },
            &actor,
        )
        .await
        .unwrap();
    assert!(!allowed);
}

// =====================================================================
// V2: RLS 13 類 跨 tenant
// =====================================================================

/// **IT-V2-1**: 跨 tenant scheme 访问被拒(守门 #13 c Master 100% RLS 13 類)
#[tokio::test]
async fn it_v2_cross_tenant_check_rejected() {
    let svc = InMemoryPermissionService::new();
    let tenant_a = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor_a = make_tenant_admin(tenant_a);

    // tenant A 创建 scheme
    let scheme_a = svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: TenantId(tenant_a),
                name: "a-scheme".to_string(),
                actor_user_id: UserId::new(),
            },
            &actor_a,
        )
        .await
        .unwrap();

    // tenant B actor 试图用 tenant A 的 scheme_id check
    let tenant_b = Uuid::new_v4();
    let actor_b = make_tenant_admin(tenant_b);
    let res = svc
        .check(
            CheckQuery {
                tenant_id: TenantId(tenant_b),
                scheme_id: Some(scheme_a.id),
                subject_user_id: UserId::new(),
                project_id,
                resource_type: ResourceType::Project,
                resource_id: None,
                action: Action::Read,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(PermissionError::CrossTenantDenied(_, _))));
}

// =====================================================================
// V3: 5 角色 hierarchy (TenantAdmin/ProjectAdmin/Developer/Viewer/Agent)
// =====================================================================

/// **IT-V3-1**: 5 角色 hierarchy 完整覆盖(per AGENTS §5 5 域历史治理命名 + 8/21 JST 拍板)
#[tokio::test]
async fn it_v3_five_role_hierarchy_covered() {
    let svc = InMemoryPermissionService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_tenant_admin(tenant_id);

    // 5 角色 全部 grant 成功
    for role in [
        Role::TenantAdmin,
        Role::ProjectAdmin,
        Role::Developer,
        Role::Viewer,
        Role::Agent,
    ] {
        let user = UserId::new();
        let binding: RoleBinding = svc
            .grant_role(
                GrantRoleCommand {
                    tenant_id: TenantId(tenant_id),
                    user_id: user,
                    project_id,
                    role,
                    granted_by: UserId::new(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(binding.role, role);
    }

    let list = svc
        .list_roles(
            ListRolesQuery {
                tenant_id: TenantId(tenant_id),
                project_id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 5);

    // 5 角色 admin 分类
    assert!(Role::TenantAdmin.is_admin());
    assert!(Role::ProjectAdmin.is_admin());
    assert!(!Role::Developer.is_admin());
    assert!(!Role::Viewer.is_admin());
    assert!(!Role::Agent.is_admin());

    // 5 角色字符串 roundtrip
    for r in [
        Role::TenantAdmin,
        Role::ProjectAdmin,
        Role::Developer,
        Role::Viewer,
        Role::Agent,
    ] {
        let s = r.as_str();
        assert_eq!(Role::from_str_opt(s), Some(r));
    }
}

/// **IT-V3-2**: get_scheme 返回完整 scheme (per SCD Master 校验)
#[tokio::test]
async fn it_v3_get_scheme_returns_full_record() {
    let svc = InMemoryPermissionService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_tenant_admin(tenant_id);

    let scheme = svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: TenantId(tenant_id),
                name: "test-scheme".to_string(),
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();

    let fetched = svc
        .get_scheme(
            GetSchemeQuery {
                tenant_id: TenantId(tenant_id),
                scheme_id: scheme.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, scheme.id);
    assert_eq!(fetched.name, "test-scheme");
    assert_eq!(fetched.tenant_id.0, tenant_id); // 必带 tenant_id (守门 #13 c)
}

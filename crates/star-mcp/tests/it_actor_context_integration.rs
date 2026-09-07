//! 跨 crate 集成测试 (IT) — ActorContext 跨 crate 流转
//!
//! **crate**: star-mcp (test target)
//! **per**: 2026-08-31 P0-1 联动审计 + PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT v0.3
//! **目的**: 验证 P0-1 收敛后 star_context::ActorContext 真能被 domain-* 跨 crate 接受
//!
//! ## 测试维度
//!
//! 1. 跨 crate 类型流转: star_context::ActorContext (Uuid) → domain 强类型 ID
//! 2. 强类型 ID 转换: UserId(Uuid) / TenantId(Uuid) (per audit P0-1b)
//! 3. domain_identity InMemoryIdentityService 真能接受
//! 4. domain_permission check 接受 star_context::ActorContext
//! 5. star-mcp handler 接受跨 crate actor

use domain_identity::{
    ActorContext, CreateUserCommand, CredentialRefId, IdentityCommandPort, InMemoryIdentityService,
    TenantId, TenantRole, UserId,
};
use star_context::ActorContext as StarActorContext;
use uuid::Uuid;

/// **IT-CROSS-1**: 跨 crate 类型流转 — `UserId(Uuid)` tuple struct 构造
#[test]
fn it_cross_userid_tenantid_tuple_construct() {
    let user_uuid = Uuid::new_v4();
    let tenant_uuid = Uuid::new_v4();
    let user_id = UserId(user_uuid);
    let tenant_id = TenantId(tenant_uuid);
    assert_eq!(user_id.as_uuid(), user_uuid);
    assert_eq!(tenant_id.as_uuid(), tenant_uuid);
}

/// **IT-CROSS-2**: domain-identity::ActorContext (re-export) 是 7 字段版本
#[test]
fn it_cross_actor_context_7_fields() {
    let actor: ActorContext = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    // 7 字段 (per star_context::ActorContext)
    let _: Uuid = actor.user_id;
    let _: Uuid = actor.tenant_id;
    let _: Option<Uuid> = actor.device_id;
    let _: Vec<Uuid> = actor.project_ids;
    let _: Vec<String> = actor.roles;
    let _: bool = actor.is_local_runtime;
    let _: bool = actor.is_platform_admin;
}

/// **IT-CROSS-3**: 跨 crate 全字段构造 + InMemoryIdentityService 真能接受
#[tokio::test]
async fn it_cross_inmemory_identity_create_user() {
    // actor 必须有 tenant_admin role (否则 service 第一行 PermissionDenied)
    let star_actor =
        StarActorContext::new(Uuid::new_v4(), Uuid::new_v4()).with_role("tenant_admin");
    let svc = InMemoryIdentityService::new();

    let cmd = CreateUserCommand {
        tenant_id: TenantId(star_actor.tenant_id),
        email: "test@star.local".to_string(),
        display_name: "IT Test".to_string(),
        tenant_role: TenantRole::Developer,
        credential_ref: CredentialRefId::new(),
    };

    let result = svc.create_user(cmd, &star_actor).await;
    assert!(
        result.is_ok(),
        "create_user 应该成功, 实际: {:?}",
        result.err()
    );
    let user = result.unwrap();
    assert_eq!(user.email, "test@star.local");
    assert_eq!(user.tenant_id.as_uuid(), star_actor.tenant_id);
}

/// **IT-CROSS-4**: 拒绝跨 tenant 操作 (INV-ID-01)
#[tokio::test]
async fn it_cross_inmemory_identity_cross_tenant_denied() {
    // actor 有 tenant_admin 但跨 tenant → 触发 CrossTenantDenied (跳过第一行 PermissionDenied)
    let star_actor_tenant_a =
        StarActorContext::new(Uuid::new_v4(), Uuid::new_v4()).with_role("tenant_admin");
    let tenant_b_uuid = Uuid::new_v4();
    let svc = InMemoryIdentityService::new();

    let cmd = CreateUserCommand {
        tenant_id: TenantId(tenant_b_uuid), // 跨 tenant
        email: "evil@other.local".to_string(),
        display_name: "Evil".to_string(),
        tenant_role: TenantRole::Developer,
        credential_ref: CredentialRefId::new(),
    };

    let result = svc.create_user(cmd, &star_actor_tenant_a).await;
    assert!(result.is_err(), "跨 tenant 应该被拒绝");
    let err = result.err().unwrap();
    assert!(
        matches!(err, domain_identity::IdentityError::CrossTenantDenied(_, _)),
        "期望 CrossTenantDenied, 实际: {err:?}"
    );
}

/// **IT-CROSS-5**: is_platform_admin 跨平台管理 (绕过跨 tenant 限制)
#[tokio::test]
async fn it_cross_inmemory_identity_platform_admin_bypass() {
    let mut star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    star_actor.is_platform_admin = true; // 平台 admin
    let tenant_b_uuid = Uuid::new_v4();
    let svc = InMemoryIdentityService::new();

    let cmd = CreateUserCommand {
        tenant_id: TenantId(tenant_b_uuid),
        email: "platform@star.local".to_string(),
        display_name: "Platform".to_string(),
        tenant_role: TenantRole::TenantAdmin,
        credential_ref: CredentialRefId::new(),
    };

    let result = svc.create_user(cmd, &star_actor).await;
    assert!(
        result.is_ok(),
        "platform admin 应该能跨 tenant 创建, 实际: {:?}",
        result.err()
    );
}

/// **IT-CROSS-6**: 5 个角色 string 流转保持
#[test]
fn it_cross_actor_context_5_roles_preserved() {
    let star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin")
        .with_role("project_admin")
        .with_role("developer")
        .with_role("viewer")
        .with_role("agent");
    let parsed: ActorContext =
        serde_json::from_str(&serde_json::to_string(&star_actor).unwrap()).unwrap();
    assert_eq!(parsed.roles.len(), 6); // default "developer" + 5
    for r in [
        "tenant_admin",
        "project_admin",
        "developer",
        "viewer",
        "agent",
    ] {
        assert!(parsed.roles.contains(&r.to_string()));
    }
}

/// **IT-CROSS-7**: 跨 crate serde_json 兼容
#[test]
fn it_cross_actor_context_json_through_crate_boundary() {
    let star_actor =
        StarActorContext::new(Uuid::new_v4(), Uuid::new_v4()).with_role("tenant_admin");
    let json = serde_json::to_string(&star_actor).unwrap();
    // domain_identity re-export 同一个类型
    let back: ActorContext = serde_json::from_str(&json).unwrap();
    assert_eq!(back.user_id, star_actor.user_id);
    assert_eq!(back.tenant_id, star_actor.tenant_id);
    assert_eq!(back.roles, star_actor.roles);
}

/// **IT-CROSS-8**: panic 守门 (INV-ACT-01)
#[test]
#[should_panic(expected = "user_id 不能为 nil")]
fn it_cross_actor_context_new_panic_nil_user() {
    let _ = StarActorContext::new(Uuid::nil(), Uuid::new_v4());
}

#[test]
#[should_panic(expected = "tenant_id 不能为 nil")]
fn it_cross_actor_context_new_panic_nil_tenant() {
    let _ = StarActorContext::new(Uuid::new_v4(), Uuid::nil());
}

// =====================================================================
// B.2 Phase B T1.7 ActorContext 集成新增 test (per OPT-WORKER-03 B.2, 2026-09-07)
// =====================================================================

/// **IT-CROSS-9**: `as_local_runtime` builder 跨 crate 验证 (per B.1 helper)
/// 验证: star-mcp 通过 `star_context::ActorContext` 调用 `as_local_runtime` builder,
///       跨 crate 类型流转保持, `is_local_runtime` 字段被设置.
#[test]
fn b2_as_local_runtime_via_star_context() {
    let actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("agent")
        .as_local_runtime();
    assert!(actor.is_local_runtime);
    assert!(actor.is_local_runtime());
    assert!(actor.has_role("agent"));
}

/// **IT-CROSS-10**: `as_local_runtime` 后跨 domain 服务仍可接受 (per P0-1 联动)
/// 验证: 构造 local runtime actor 后, `InMemoryIdentityService.create_user` 跨 crate 接受.
///       走 domain-permission Deny 优先 + INV-ACT-03 is_local_runtime + roles 含 "agent" 守门.
#[tokio::test]
async fn b2_as_local_runtime_cross_domain_accepted() {
    // actor 用 tenant_admin role 触发 is_local_runtime + roles 含 "agent" + tenant_admin
    // 跨 domain-identity create_user 跨 crate 流转保持
    let actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin")
        .as_local_runtime();
    let svc = InMemoryIdentityService::new();

    let cmd = CreateUserCommand {
        tenant_id: TenantId(actor.tenant_id),
        email: "local-runtime@star.local".to_string(),
        display_name: "LocalRuntime".to_string(),
        tenant_role: TenantRole::Developer,
        credential_ref: CredentialRefId::new(),
    };

    let result = svc.create_user(cmd, &actor).await;
    assert!(
        result.is_ok(),
        "local runtime actor + tenant_admin role 应该能创建 user, 实际: {:?}",
        result.err()
    );
    let user = result.unwrap();
    assert_eq!(user.email, "local-runtime@star.local");
}

/// **IT-CROSS-11**: `is_platform_admin` 跨 crate 流转保持
/// 验证: 直接构造 `is_platform_admin = true` 字段 (per 守门 #1 v18 H2-EXT 实证),
///       跨 domain-identity 真能 bypass 跨 tenant 限制 (per IT-CROSS-5 已实证,
///       本 test 复测保持 0 regression).
#[tokio::test]
async fn b2_is_platform_admin_field_propagates() {
    let mut actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    actor.is_platform_admin = true;
    assert!(actor.is_platform_admin());
    assert!(actor.is_platform_admin);

    let svc = InMemoryIdentityService::new();
    let cmd = CreateUserCommand {
        tenant_id: TenantId(Uuid::new_v4()),
        email: "platform-admin@star.local".to_string(),
        display_name: "PlatformAdmin".to_string(),
        tenant_role: TenantRole::TenantAdmin,
        credential_ref: CredentialRefId::new(),
    };

    let result = svc.create_user(cmd, &actor).await;
    assert!(
        result.is_ok(),
        "platform admin 跨 tenant 应 bypass, 实际: {:?}",
        result.err()
    );
}

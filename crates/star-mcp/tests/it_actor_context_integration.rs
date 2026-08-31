//! 跨 crate 集成测试 (IT) — ActorContext 跨 crate 流转
//!
//! **crate**: star-mcp (test target)
//! **per**: 2026-08-31 P0-1 联动审计 + PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT v0.3
//! **目的**: 验证 P0-1 收敛后 star_context::ActorContext 真能被 domain-* 跨 crate 接受
//!
//! ## 测试维度
//!
//! 1. 跨 crate 类型流转: star_context::ActorContext (Uuid) → domain 强类型 ID
//! 2. 强类型 ID 转换: UserId::from(Uuid) / TenantId::from(Uuid) (per audit P0-1b)
//! 3. domain_identity InMemoryIdentityService 真能接受
//! 4. domain_permission check 接受 star_context::ActorContext
//! 5. star-mcp handler 接受跨 crate actor

use domain_identity::{
    ActorContext, CreateUserCommand, CredentialRefId, IdentityCommandPort, InMemoryIdentityService,
    TenantId, TenantRole, UserId,
};
use star_context::ActorContext as StarActorContext;
use uuid::Uuid;

/// **IT-CROSS-1**: 跨 crate 类型流转
/// star_context::ActorContext (Uuid 字段) → domain_identity 强类型 ID
#[test]
fn it_cross_actor_context_to_domain_identity() {
    let star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let user_id: UserId = UserId::from(star_actor.user_id);
    let tenant_id: TenantId = TenantId::from(star_actor.tenant_id);
    // 跨 crate 转换必须保留 UUID 值
    assert_eq!(user_id.as_uuid(), star_actor.user_id);
    assert_eq!(tenant_id.as_uuid(), star_actor.tenant_id);
}

/// **IT-CROSS-2**: domain-identity::ActorContext (强类型) 跟 star_context::ActorContext (Uuid) 共存
///
/// domain 内部仍然用强类型 ID 版本 (per 子模块 context.rs 决策),
/// 但 lib 顶层统一 re-export 指向 star_context::ActorContext
#[test]
fn it_cross_actor_context_alias_resolution() {
    // domain-identity lib 顶层 ActorContext = star_context::ActorContext (re-export)
    let _: domain_identity::ActorContext = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
}

/// **IT-CROSS-3**: InMemoryIdentityService 接受跨 crate actor + 强类型 Command
#[tokio::test]
async fn it_cross_inmemory_identity_create_user() {
    let star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let svc = InMemoryIdentityService::new();

    // 构造 domain 强类型 Command
    let cmd = CreateUserCommand {
        tenant_id: TenantId::from(star_actor.tenant_id),
        email: "test@star.local".to_string(),
        display_name: "IT Test".to_string(),
        tenant_role: TenantRole::Developer,
        credential_ref: CredentialRefId::new(),
    };

    // domain 内部把 star_context::ActorContext 强转本地 actor (per P0-1b)
    let local_actor = domain_identity::ActorContext {
        user_id: UserId::from(star_actor.user_id),
        tenant_id: TenantId::from(star_actor.tenant_id),
        project_ids: vec![],
        roles: star_actor.roles.clone(),
        is_platform_admin: star_actor.is_platform_admin,
    };

    let result = svc.create_user(cmd, &local_actor).await;
    assert!(result.is_ok(), "create_user 应该成功, 实际: {:?}", result.err());
    let user = result.unwrap();
    assert_eq!(user.email, "test@star.local");
    assert_eq!(user.tenant_id.as_uuid(), star_actor.tenant_id);
}

/// **IT-CROSS-4**: 拒绝跨 tenant 操作 (INV-ID-01 跨 tenant 拒绝)
#[tokio::test]
async fn it_cross_inmemory_identity_cross_tenant_denied() {
    let star_actor_tenant_a = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let tenant_b = Uuid::new_v4();
    let svc = InMemoryIdentityService::new();

    let cmd = CreateUserCommand {
        tenant_id: TenantId::from(tenant_b),  // 跨 tenant
        email: "evil@other.local".to_string(),
        display_name: "Evil".to_string(),
        tenant_role: TenantRole::Developer,
        credential_ref: CredentialRefId::new(),
    };

    let local_actor = domain_identity::ActorContext {
        user_id: UserId::from(star_actor_tenant_a.user_id),
        tenant_id: TenantId::from(star_actor_tenant_a.tenant_id),
        project_ids: vec![],
        roles: star_actor_tenant_a.roles.clone(),
        is_platform_admin: false,
    };

    let result = svc.create_user(cmd, &local_actor).await;
    assert!(result.is_err(), "跨 tenant 应该被拒绝");
    // CrossTenantDenied 错误
    let err = result.err().unwrap();
    assert!(
        matches!(err, domain_identity::IdentityError::CrossTenantDenied(_, _)),
        "期望 CrossTenantDenied, 实际: {err:?}"
    );
}

/// **IT-CROSS-5**: is_platform_admin 跨平台管理 (绕过跨 tenant 限制)
#[tokio::test]
async fn it_cross_inmemory_identity_platform_admin_bypass() {
    let star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    let tenant_b = Uuid::new_v4();
    let svc = InMemoryIdentityService::new();

    let cmd = CreateUserCommand {
        tenant_id: TenantId::from(tenant_b),
        email: "platform@star.local".to_string(),
        display_name: "Platform".to_string(),
        tenant_role: TenantRole::TenantAdmin,
        credential_ref: CredentialRefId::new(),
    };

    let local_actor = domain_identity::ActorContext {
        user_id: UserId::from(star_actor.user_id),
        tenant_id: TenantId::from(star_actor.tenant_id),
        project_ids: vec![],
        roles: star_actor.roles.clone(),
        is_platform_admin: true,  // 平台 admin
    };

    let result = svc.create_user(cmd, &local_actor).await;
    assert!(
        result.is_ok(),
        "platform admin 应该能跨 tenant 创建, 实际: {:?}",
        result.err()
    );
}

/// **IT-CROSS-6**: 5 个角色 string 流转保持一致
#[test]
fn it_cross_actor_context_5_roles_preserved() {
    let star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin")
        .with_role("project_admin")
        .with_role("developer")
        .with_role("viewer")
        .with_role("agent");
    let local_actor = domain_identity::ActorContext {
        user_id: UserId::from(star_actor.user_id),
        tenant_id: TenantId::from(star_actor.tenant_id),
        project_ids: vec![],
        roles: star_actor.roles.clone(),
        is_platform_admin: star_actor.is_platform_admin,
    };
    // 5 个角色 string 必须全部保留
    assert_eq!(local_actor.roles.len(), 6); // default "developer" + 5
    for r in ["tenant_admin", "project_admin", "developer", "viewer", "agent"] {
        assert!(local_actor.roles.contains(&r.to_string()));
    }
}

/// **IT-CROSS-7**: 跨 crate 序列化兼容
/// star_context 序列化 → JSON → domain 接收
#[test]
fn it_cross_actor_context_json_through_crate_boundary() {
    let star_actor = StarActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin");
    let json = serde_json::to_string(&star_actor).unwrap();
    // domain_identity re-export 同一个类型
    let back: ActorContext = serde_json::from_str(&json).unwrap();
    assert_eq!(back.user_id, star_actor.user_id);
    assert_eq!(back.tenant_id, star_actor.tenant_id);
    assert_eq!(back.roles, star_actor.roles);
}

/// **IT-CROSS-8**: 跨 crate ActorContext::new panic 守门 (INV-ACT-01)
///
/// 这是 star_context 公开契约, 跨 crate 调用方应能依赖
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

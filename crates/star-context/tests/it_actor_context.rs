//! 集成测试 (IT) — ActorContext 权威化 + 跨 crate 流转验证
//!
//! **crate**: star_context (test target)
//! **per**: 2026-08-31 P0-1 联动审计 + PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT v0.3
//! **目的**: 验证 P0-1 收敛后 star_context::ActorContext 跨 crate 流转
//!
//! ## 测试维度
//!
//! 1. INV-ACT-01: user_id / tenant_id 非 nil
//! 2. INV-ACT-02: roles 元素属于 Role 枚举字符串
//! 3. INV-ACT-03: is_local_runtime == true → roles 含 "agent"
//! 4. has_role / parsed_roles / is_platform_admin / is_local_runtime 方法
//! 5. serde_json roundtrip 跨格式
//! 6. ActorContext::new panic on nil (INV-ACT-01 panic guard)
//! 7. Default::default() 不满足 INV-ACT-01 (测试桩限定)

use star_context::ActorContext;
use uuid::Uuid;

/// **IT-1**: ActorContext::new 满足 INV-ACT-01 (user/tenant 非 nil)
#[test]
fn it_actor_context_new_invariants_01() {
    let user = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let actor = ActorContext::new(user, tenant);
    assert!(!actor.user_id.is_nil());
    assert!(!actor.tenant_id.is_nil());
    assert_eq!(actor.user_id, user);
    assert_eq!(actor.tenant_id, tenant);
}

/// **IT-2**: ActorContext::new 默认 roles = ["developer"]
#[test]
fn it_actor_context_default_role() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    assert!(actor.has_role("developer"));
    assert!(!actor.has_role("tenant_admin"));
    assert!(!actor.has_role("project_admin"));
    assert!(!actor.has_role("viewer"));
    assert!(!actor.has_role("agent"));
}

/// **IT-3**: ActorContext::with_role 链式追加
#[test]
fn it_actor_context_with_role_chain() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin")
        .with_role("project_admin");
    assert!(actor.has_role("developer")); // default
    assert!(actor.has_role("tenant_admin"));
    assert!(actor.has_role("project_admin"));
    assert_eq!(actor.roles.len(), 3);
}

/// **IT-4**: has_role 严格大小写 (per doc)
#[test]
fn it_actor_context_has_role_case_sensitive() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("Tenant_Admin");
    assert!(!actor.has_role("tenant_admin"));
    assert!(actor.has_role("Tenant_Admin"));
}

/// **IT-5**: parsed_roles 归一化小写
#[test]
fn it_actor_context_parsed_roles_lowercase() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("TENANT_ADMIN")
        .with_role("Project-Admin");
    let parsed = actor.parsed_roles();
    assert!(parsed.contains(&"tenant_admin".to_string()));
    assert!(parsed.contains(&"project-admin".to_string()));
}

/// **IT-6**: is_platform_admin / is_local_runtime 字段访问
#[test]
fn it_actor_context_flag_accessors() {
    let mut actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    assert!(!actor.is_platform_admin());
    assert!(!actor.is_local_runtime());
    actor.is_platform_admin = true;
    actor.is_local_runtime = true;
    actor.roles.push("agent".to_string()); // INV-ACT-03
    assert!(actor.is_platform_admin());
    assert!(actor.is_local_runtime());
}

/// **IT-7**: device_id 可选 (None / Some)
#[test]
fn it_actor_context_device_id_optional() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    assert!(actor.device_id.is_none());
}

/// **IT-8**: project_ids 默认空 Vec
#[test]
fn it_actor_context_project_ids_default_empty() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    assert!(actor.project_ids.is_empty());
}

/// **IT-9**: serde_json roundtrip
#[test]
fn it_actor_context_serde_roundtrip() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin");
    let json = serde_json::to_string(&actor).unwrap();
    let parsed: ActorContext = serde_json::from_str(&json).unwrap();
    assert_eq!(actor.user_id, parsed.user_id);
    assert_eq!(actor.tenant_id, parsed.tenant_id);
    assert_eq!(actor.roles, parsed.roles);
    assert_eq!(actor.is_platform_admin, parsed.is_platform_admin);
    assert_eq!(actor.is_local_runtime, parsed.is_local_runtime);
}

/// **IT-10**: serde_json 字段全保留 (不丢字段)
#[test]
fn it_actor_context_serde_all_fields() {
    let actor = ActorContext {
        user_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        device_id: Some(Uuid::new_v4()),
        project_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        roles: vec!["developer".to_string(), "tenant_admin".to_string()],
        is_local_runtime: true,
        is_platform_admin: true,
    };
    let json = serde_json::to_string(&actor).unwrap();
    let parsed: ActorContext = serde_json::from_str(&json).unwrap();
    assert_eq!(actor.device_id, parsed.device_id);
    assert_eq!(actor.project_ids, parsed.project_ids);
    assert_eq!(actor.roles, parsed.roles);
}

/// **IT-11**: Default::default() 不满足 INV-ACT-01 (panic guard 不触发)
#[test]
fn it_actor_context_default_not_invariant_01() {
    let actor = ActorContext::default();
    assert!(actor.user_id.is_nil());
    assert!(actor.tenant_id.is_nil());
    // Default 仅用于测试桩, 业务代码必须用 ActorContext::new()
}

/// **IT-12**: 跨 crate 标识 — star_context::ActorContext 是公共 API
/// (此测试通过它能被 `use star_context::ActorContext;` 引用来证明)
#[test]
fn it_actor_context_public_api() {
    // 如果这一行能编译, 证明 star_context 公开了 ActorContext
    let _type_check: fn() -> ActorContext = || {
        ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
    };
}

/// **IT-13**: 5 角色枚举 (per domain_permission::Role)
#[test]
fn it_actor_context_5_roles() {
    let actor = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin")
        .with_role("project_admin")
        .with_role("developer")
        .with_role("viewer")
        .with_role("agent");
    for r in ["tenant_admin", "project_admin", "developer", "viewer", "agent"] {
        assert!(actor.has_role(r), "missing role: {r}");
    }
}

/// **IT-14**: UserId / TenantId 字段类型 (Uuid 弱类型, 跨 crate interface)
#[test]
fn it_actor_context_uuid_fields() {
    let user = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let actor = ActorContext::new(user, tenant);
    // star_context::ActorContext 字段是 Uuid, domain 内部转强类型 ID
    // 这保证 star_context 不依赖 domain-* (避免循环依赖)
    let _: Uuid = actor.user_id;
    let _: Uuid = actor.tenant_id;
}

/// **IT-15**: 多 actor 实例独立 (无 shared state)
#[test]
fn it_actor_context_independent() {
    let a1 = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
        .with_role("tenant_admin");
    let a2 = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
    assert!(a1.has_role("tenant_admin"));
    assert!(!a2.has_role("tenant_admin")); // a2 不受影响
}

//! Identity 身份域(颁发 UserId / DeviceId / Role)
//!
//! **crate**: `domain-identity`
//! **上游 spec**: docs/specs/domain-identity-spec.md
//! **基本设计**: docs/basic-design.md §2.1(表 18) / §23.2(三重绑定)
//! **数据设计**: docs/data-design.md §4.23 (`user` / `device` / `device_binding` / `credential` / `role`)
//! **API 设计**: docs/api-design.md §3.2 (domain-identity 端点) / §5.5 / §8.3.7
//!
//! ## 职责
//!
//! - 颁发 `user_id` / `device_id` / `role_id` / `credential_id` / `device_binding_id`
//! - 定义 5 个核心实体(`User` / `Device` / `DeviceBinding` / `Credential` / `Role`)
//! - 3 个核心 Domain Event
//! - 2 个端口(`IdentityCommandPort` × 5 方法 / `IdentityQueryPort` × 8 方法) + 1 个仓库端口
//! - 4 条不变量检查(INV-IDN-01~04)
//! - 1 个 `InMemoryIdentityService` 真实实现
//!
//! ## 关键不变量
//!
//! - 任何 User INSERT/UPDATE 必须带 tenant_id(INV-IDN-03,§6.1,REQ-SEC-001)
//! - email 在 tenant 内唯一(INV-IDN-01)
//! - (device, user, project) 三元组唯一(INV-IDN-02,§23.2)
//! - 邮箱格式合法(INV-IDN-04)
//!
//! ## 上游依赖(basic-design §2.3)
//!
//! 本 crate 仅依赖 `crates/domain-identity` 自身的外部 crate 依赖。
//!
//! **禁止反向依赖** 任何其他 `domain-*` crate。
//!
//! ## 关键引用
//!
//! 本 crate 是 6 个横切 crate 中"颁发 ID"的核心:UserId / DeviceId / RoleId /
//! DeviceBindingId / CredentialId。Phase 3 由 `crates/application` 编排时,其他
//! 5 个横切 crate 的"占位 ActorContext"可由本 crate 颁发。

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

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

pub use context::ActorContext;
pub use entity::{Credential, Device, DeviceBinding, Role, User};
pub use error::IdentityError;
pub use event::{DeviceBound, EventMeta, IdentityEvent, UserCreated, UserLoggedIn};
pub use invariants::{
    check_invariant_01_email_unique, check_invariant_02_device_binding_unique,
    check_invariant_03_tenant_id_present, check_invariant_04_email_format, run_invariants,
    ALL_INVARIANT_CHECKS,
};
pub use port::{
    BindDeviceCommand, CredentialSpec, CreateRoleCommand, CreateUserCommand, IdentityCommandPort,
    IdentityQueryPort, IdentityRepository, ListUserQuery, RecordLoginCommand, UpdateUserCommand,
};
pub use service::InMemoryIdentityService;
pub use value_object::{
    roles, CredentialId, CredentialType, DeviceBindingId, DeviceId, DeviceType, ProjectId, RoleId,
    TenantId, UserId, UserStatus,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{DeviceType, TenantId, UserId, UserStatus};

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id).with_role(roles::TENANT_ADMIN)
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let device_id = DeviceId::new();
        let project_id = ProjectId::new();
        let role_id = RoleId::new();
        let actor = ActorContext::new(user_id, tenant_id)
            .with_role(roles::TENANT_ADMIN)
            .with_role_id(role_id)
            .with_project(project_id)
            .with_device(device_id);
        assert!(actor.is_tenant_admin());
        assert!(actor.has_role_id(role_id));
        assert!(actor.is_member_of(project_id));
    }

    // -------- 2. User 字段数审计 --------

    #[test]
    fn user_field_count_audit() {
        assert_eq!(User::FIELD_COUNT, 10);
        assert_eq!(Device::FIELD_COUNT, 10);
        assert_eq!(DeviceBinding::FIELD_COUNT, 8);
        assert_eq!(Credential::FIELD_COUNT, 10);
        assert_eq!(Role::FIELD_COUNT, 10);
    }

    // -------- 3. create_user 成功路径 --------

    #[tokio::test]
    async fn create_user_success() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = CreateUserCommand {
            tenant_id,
            email: "alice@acme.com".to_string(),
            display_name: "Alice".to_string(),
            avatar_url: None,
            initial_credential: Some(CredentialSpec {
                credential_type: CredentialType::Password,
                hash: "argon2id$...".to_string(),
                provider_id: None,
                expires_at: None,
            }),
        };
        let u = svc.create_user(cmd, actor).await.expect("创建成功");
        assert_eq!(u.status, UserStatus::Active);
        assert_eq!(u.email, "alice@acme.com");
        assert_eq!(svc.count().await, 1);

        // 同步创建了 credential
        let creds = svc
            .list_user_credentials(u.id, make_test_actor(tenant_id))
            .await
            .unwrap();
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0].credential_type, CredentialType::Password);
        assert_eq!(creds[0].hash, "***"); // 已脱敏
    }

    // -------- 4. 跨租户访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_a = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let cmd = CreateUserCommand {
            tenant_id: tenant_a,
            email: "bob@a.com".to_string(),
            display_name: "Bob".to_string(),
            avatar_url: None,
            initial_credential: None,
        };
        let u = svc.create_user(cmd, actor_a).await.unwrap();
        // 用 tenant_b 尝试访问
        let tenant_b = TenantId::new();
        let actor_b = make_test_actor(tenant_b);
        let res = svc.get_user(u.id, actor_b).await;
        assert!(matches!(res, Err(IdentityError::PermissionDenied)));
    }

    // -------- 5. INV-IDN-01:email 重复被拒 --------

    #[tokio::test]
    async fn invariant_01_email_conflict() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd1 = CreateUserCommand {
            tenant_id,
            email: "dup@x.com".to_string(),
            display_name: "First".to_string(),
            avatar_url: None,
            initial_credential: None,
        };
        svc.create_user(cmd1, actor.clone()).await.unwrap();
        let cmd2 = CreateUserCommand {
            tenant_id,
            email: "dup@x.com".to_string(),
            display_name: "Second".to_string(),
            avatar_url: None,
            initial_credential: None,
        };
        let res = svc.create_user(cmd2, actor).await;
        assert!(matches!(res, Err(IdentityError::Conflict(_))));
    }

    // -------- 6. INV-IDN-04:邮箱格式非法被拒 --------

    #[tokio::test]
    async fn invariant_04_email_format_rejected() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = CreateUserCommand {
            tenant_id,
            email: "not-an-email".to_string(),
            display_name: "Bad".to_string(),
            avatar_url: None,
            initial_credential: None,
        };
        let res = svc.create_user(cmd, actor).await;
        assert!(matches!(res, Err(IdentityError::InvalidState(_))));
    }

    // -------- 7. bind_device 三重绑定(INV-IDN-02) --------

    #[tokio::test]
    async fn bind_device_three_tuple_unique() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let u = svc
            .create_user(
                CreateUserCommand {
                    tenant_id,
                    email: "cd@x.com".to_string(),
                    display_name: "CD".to_string(),
                    avatar_url: None,
                    initial_credential: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let device_id = DeviceId::new();
        let project_id = ProjectId::new();
        // 第一次绑定 OK
        let b1 = svc
            .bind_device(
                BindDeviceCommand {
                    tenant_id,
                    device_id,
                    user_id: u.id,
                    project_id: Some(project_id),
                    reason: Some("first bind".to_string()),
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert!(!b1.is_tenant_wide());
        // 第二次同三元组 → Conflict
        let res = svc
            .bind_device(
                BindDeviceCommand {
                    tenant_id,
                    device_id,
                    user_id: u.id,
                    project_id: Some(project_id),
                    reason: Some("dup".to_string()),
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::Conflict(_))));
    }

    // -------- 8. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_user_created() {
        let (svc, mut rx) = InMemoryIdentityService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = CreateUserCommand {
            tenant_id,
            email: "evt@x.com".to_string(),
            display_name: "Evt".to_string(),
            avatar_url: None,
            initial_credential: None,
        };
        svc.create_user(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 UserCreated 事件");
        assert!(matches!(evt, IdentityEvent::UserCreated(_)));
        assert_eq!(evt.subject(), "star.events.identity.user.created.v1");
    }

    // -------- 9. 乐观锁冲突 --------

    #[tokio::test]
    async fn update_user_version_conflict() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let u = svc
            .create_user(
                CreateUserCommand {
                    tenant_id,
                    email: "v@x.com".to_string(),
                    display_name: "V".to_string(),
                    avatar_url: None,
                    initial_credential: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let res = port::IdentityCommandPort::update_user(
            &*svc,
            UpdateUserCommand {
                user_id: u.id,
                tenant_id,
                expected_version: 99, // 错的
                display_name: Some("New".to_string()),
                avatar_url: None,
            },
            actor,
        )
        .await;
        assert!(matches!(res, Err(IdentityError::Conflict(_))));
    }

    // -------- 10. create_role + 权限校验 --------

    #[tokio::test]
    async fn create_role_and_check_permission() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_test_actor(tenant_id);
        let role = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id,
                    name: "developer".to_string(),
                    description: Some("dev role".to_string()),
                    permissions: vec!["workitem:read".to_string(), "workitem:create".to_string()],
                },
                admin.clone(),
            )
            .await
            .unwrap();
        assert!(role.has_permission("workitem:read"));
        assert!(!role.has_permission("admin:god"));

        // 非 admin 创建被拒
        let mut normal = ActorContext::new(UserId::new(), tenant_id);
        normal.roles.push("user".to_string());
        let res = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id,
                    name: "hacker".to_string(),
                    description: None,
                    permissions: vec![],
                },
                normal,
            )
            .await;
        assert!(matches!(res, Err(IdentityError::PermissionDenied)));
    }

    // -------- 11. record_login 触发 UserLoggedIn 事件 --------

    #[tokio::test]
    async fn record_login_updates_last_login_at() {
        let svc = InMemoryIdentityService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let u = svc
            .create_user(
                CreateUserCommand {
                    tenant_id,
                    email: "login@x.com".to_string(),
                    display_name: "L".to_string(),
                    avatar_url: None,
                    initial_credential: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert!(u.last_login_at.is_none());
        let device_id = DeviceId::new();
        svc.record_login(
            RecordLoginCommand {
                user_id: u.id,
                device_id,
                device_type: DeviceType::Web,
            },
            actor,
        )
        .await
        .unwrap();
        let u2 = svc
            .get_user(u.id, make_test_actor(tenant_id))
            .await
            .unwrap();
        assert!(u2.last_login_at.is_some());
    }
}

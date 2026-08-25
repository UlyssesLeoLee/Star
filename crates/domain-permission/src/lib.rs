//! Permission RBAC 权限层
//!
//! **crate**: `domain-permission`
//! **上游 spec**: docs/specs/domain-permission-spec.md
//! **基本设计**: docs/basic-design.md §4.8
//! **数据设计**: docs/data-design.md §4.8

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

pub use context::ActorContext;
pub use entity::{Permission, PermissionScheme, Role};
pub use error::PermissionError;
pub use event::{EventMeta, PermissionChecked, PermissionEvent, RoleCreated, SchemeCreated};
pub use invariants::{
    check_invariant_01_permission_code_unique, check_invariant_02_scheme_has_owner,
    check_invariant_03_tenant_id_present, check_invariant_04_role_name_format, run_invariants,
    ALL_INVARIANT_CHECKS,
};
pub use port::{
    CheckPermissionQuery, CreatePermissionSchemeCommand, CreateRoleCommand,
    PermissionCommandPort, PermissionQueryPort, UpdateRoleCommand,
};
pub use service::InMemoryPermissionService;
pub use value_object::{
    perm_codes, roles, PermissionId, PermissionSchemeId, PermissionScope, ProjectId, RoleId,
    TenantId,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_admin(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id).with_role(roles::TENANT_ADMIN)
    }

    fn make_normal(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(Role::FIELD_COUNT, 10);
        assert_eq!(Permission::FIELD_COUNT, 6);
        assert_eq!(PermissionScheme::FIELD_COUNT, 9);
    }

    #[tokio::test]
    async fn list_builtin_permissions() {
        let svc = InMemoryPermissionService::new_for_test();
        let perms = svc.list_permissions().await.unwrap();
        assert!(perms.len() >= 6);
        assert!(perms.iter().any(|p| p.code == "workitem:read"));
    }

    #[tokio::test]
    async fn create_role_success() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let role = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id,
                    name: "developer".to_string(),
                    description: Some("dev role".to_string()),
                    permissions: vec![perm_codes::WORKITEM_READ.to_string()],
                },
                admin,
            )
            .await
            .unwrap();
        assert!(role.has_permission(perm_codes::WORKITEM_READ));
        assert!(!role.built_in);
    }

    #[tokio::test]
    async fn create_role_non_admin_denied() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let normal = make_normal(tenant_id);
        let res = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id,
                    name: "x".to_string(),
                    description: None,
                    permissions: vec![],
                },
                normal,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::PermissionDenied)));
    }

    #[tokio::test]
    async fn create_role_duplicate_name() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let cmd = CreateRoleCommand {
            tenant_id,
            name: "dup".to_string(),
            description: None,
            permissions: vec![],
        };
        svc.create_role(cmd.clone(), admin.clone()).await.unwrap();
        let res = svc.create_role(cmd, admin).await;
        assert!(matches!(res, Err(PermissionError::Conflict(_))));
    }

    #[tokio::test]
    async fn invariant_04_empty_name_rejected() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let res = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id,
                    name: "".to_string(),
                    description: None,
                    permissions: vec![],
                },
                admin,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::InvalidState(_))));
    }

    #[tokio::test]
    async fn check_permission_grants() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let role = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id,
                    name: "r".to_string(),
                    description: None,
                    permissions: vec![perm_codes::WORKITEM_READ.to_string()],
                },
                admin.clone(),
            )
            .await
            .unwrap();
        let granted = svc
            .check_permission(
                CheckPermissionQuery {
                    role_id: role.id,
                    permission: perm_codes::WORKITEM_READ.to_string(),
                },
                admin.clone(),
            )
            .await
            .unwrap();
        assert!(granted);
        let denied = svc
            .check_permission(
                CheckPermissionQuery {
                    role_id: role.id,
                    permission: perm_codes::WORKITEM_DELETE.to_string(),
                },
                admin,
            )
            .await
            .unwrap();
        assert!(!denied);
    }

    #[tokio::test]
    async fn scheme_must_have_default_role() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let mut role_perms = HashMap::new();
        role_perms.insert("developer".to_string(), vec!["workitem:read".to_string()]);
        let res = svc
            .create_scheme(
                CreatePermissionSchemeCommand {
                    project_id: ProjectId::new(),
                    tenant_id,
                    name: "Default".to_string(),
                    default_role: "missing-role".to_string(), // 不在 role_permissions 中
                    role_permissions: role_perms,
                },
                admin,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::InvalidState(_))));
    }

    #[tokio::test]
    async fn scheme_grants_via_default_role() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let mut role_perms = HashMap::new();
        role_perms.insert(
            "developer".to_string(),
            vec![perm_codes::WORKITEM_READ.to_string()],
        );
        let scheme = svc
            .create_scheme(
                CreatePermissionSchemeCommand {
                    project_id: ProjectId::new(),
                    tenant_id,
                    name: "Default".to_string(),
                    default_role: "developer".to_string(),
                    role_permissions: role_perms,
                },
                admin.clone(),
            )
            .await
            .unwrap();
        assert!(scheme.grants("developer", perm_codes::WORKITEM_READ));
        assert!(!scheme.grants("viewer", perm_codes::WORKITEM_READ));
    }

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryPermissionService::new_for_test();
        let tenant_a = TenantId::new();
        let admin_a = make_admin(tenant_a);
        let role = svc
            .create_role(
                CreateRoleCommand {
                    tenant_id: tenant_a,
                    name: "r".to_string(),
                    description: None,
                    permissions: vec![],
                },
                admin_a,
            )
            .await
            .unwrap();
        let tenant_b = TenantId::new();
        let admin_b = make_admin(tenant_b);
        let res = svc.get_role(role.id, admin_b).await;
        assert!(matches!(res, Err(PermissionError::PermissionDenied)));
    }

    #[tokio::test]
    async fn event_bus_receives_role_created() {
        let (svc, mut rx) = InMemoryPermissionService::new();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        svc.create_role(
            CreateRoleCommand {
                tenant_id,
                name: "r".to_string(),
                description: None,
                permissions: vec![],
            },
            admin,
        )
        .await
        .unwrap();
        let evt = rx.try_recv().expect("应收到 RoleCreated 事件");
        assert!(matches!(evt, PermissionEvent::RoleCreated(_)));
        assert_eq!(evt.subject(), "star.events.permission.role.created.v1");
    }
}

//! Workspace 协作单位
//!
//! **crate**: `domain-workspace`
//! **上游 spec**: docs/specs/domain-workspace-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.2
//! **数据设计**: docs/data-design.md §4.2 (`workspace` / `workspace_member` schema)
//! **API 设计**: docs/api-design.md §3.2 (domain-workspace 端点)
//!
//! ## 职责
//!
//! - 颁发 `workspace_id` / `workspace_member_id`
//! - 2 个核心实体(`Workspace` / `WorkspaceMember`)
//! - 3 个核心 Domain Event
//! - 2 个端口(4 cmd / 4 query)
//! - 3 条不变量(INV-WS-01~03)
//! - 1 个 `InMemoryWorkspaceService` 真实实现
//!
//! ## 关键不变量
//!
//! - 任何 Workspace INSERT/UPDATE 必须带 tenant_id(INV-WS-02,§6.1,REQ-SEC-001)
//! - `workspace_key` 在 tenant 内唯一(INV-WS-01)

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
pub use entity::{Workspace, WorkspaceMember};
pub use error::WorkspaceError;
pub use event::{EventMeta, MemberAdded, MemberRemoved, WorkspaceCreated, WorkspaceEvent};
pub use invariants::{
    check_invariant_01_workspace_key_unique, check_invariant_02_tenant_id_present,
    check_invariant_03_workspace_key_format, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    AddMemberCommand, CreateWorkspaceCommand, ListWorkspaceQuery, RemoveMemberCommand,
    UpdateWorkspaceCommand, WorkspaceCommandPort, WorkspaceQueryPort,
};
pub use service::InMemoryWorkspaceService;
pub use value_object::{roles, TenantId, UserId, WorkspaceId, WorkspaceMemberId, WorkspaceRole};

#[cfg(test)]
mod tests {
    use super::*;

    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id).with_role(roles::WORKSPACE_ADMIN)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(Workspace::FIELD_COUNT, 8);
        assert_eq!(WorkspaceMember::FIELD_COUNT, 7);
    }

    #[tokio::test]
    async fn create_workspace_success() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "acme".to_string(),
            name: "Acme Workspace".to_string(),
            description: Some("main".to_string()),
            owner_user_id: UserId::new(),
        };
        let ws = svc.create_workspace(cmd, actor).await.unwrap();
        assert_eq!(ws.version, 1);
        assert_eq!(svc.count().await, 1);
        // owner 自动为 Admin
        let members = svc
            .list_members(ws.id, make_actor(tenant_id))
            .await
            .unwrap();
        assert_eq!(members.len(), 1);
        assert!(members[0].is_admin());
    }

    #[tokio::test]
    async fn invariant_01_workspace_key_conflict() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd1 = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "dup".to_string(),
            name: "W1".to_string(),
            description: None,
            owner_user_id: UserId::new(),
        };
        svc.create_workspace(cmd1, actor.clone()).await.unwrap();
        let cmd2 = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "dup".to_string(),
            name: "W2".to_string(),
            description: None,
            owner_user_id: UserId::new(),
        };
        let res = svc.create_workspace(cmd2, actor).await;
        assert!(matches!(res, Err(WorkspaceError::Conflict(_))));
    }

    #[tokio::test]
    async fn invariant_03_empty_key_rejected() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "".to_string(),
            name: "Empty".to_string(),
            description: None,
            owner_user_id: UserId::new(),
        };
        let res = svc.create_workspace(cmd, actor).await;
        assert!(matches!(res, Err(WorkspaceError::InvalidState(_))));
    }

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_a = TenantId::new();
        let actor_a = make_actor(tenant_a);
        let ws = svc
            .create_workspace(
                CreateWorkspaceCommand {
                    tenant_id: tenant_a,
                    workspace_key: "a".to_string(),
                    name: "A".to_string(),
                    description: None,
                    owner_user_id: UserId::new(),
                },
                actor_a,
            )
            .await
            .unwrap();
        let tenant_b = TenantId::new();
        let actor_b = make_actor(tenant_b);
        let res = svc.get_by_id(ws.id, actor_b).await;
        assert!(matches!(res, Err(WorkspaceError::PermissionDenied)));
    }

    #[tokio::test]
    async fn add_and_remove_member() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let ws = svc
            .create_workspace(
                CreateWorkspaceCommand {
                    tenant_id,
                    workspace_key: "ws".to_string(),
                    name: "WS".to_string(),
                    description: None,
                    owner_user_id: UserId::new(),
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let new_user = UserId::new();
        let m = svc
            .add_member(
                AddMemberCommand {
                    workspace_id: ws.id,
                    tenant_id,
                    user_id: new_user,
                    role: WorkspaceRole::Member,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert!(!m.is_admin());
        // 重复加 → Conflict
        let res = svc
            .add_member(
                AddMemberCommand {
                    workspace_id: ws.id,
                    tenant_id,
                    user_id: new_user,
                    role: WorkspaceRole::Member,
                },
                actor.clone(),
            )
            .await;
        assert!(matches!(res, Err(WorkspaceError::Conflict(_))));

        // 移除
        svc.remove_member(
            RemoveMemberCommand {
                workspace_id: ws.id,
                tenant_id,
                user_id: new_user,
            },
            actor,
        )
        .await
        .unwrap();
        let members = svc.list_members(ws.id, make_actor(tenant_id)).await.unwrap();
        // owner 还在
        assert_eq!(members.len(), 1);
    }

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryWorkspaceService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "evt".to_string(),
            name: "E".to_string(),
            description: None,
            owner_user_id: UserId::new(),
        };
        svc.create_workspace(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 Created 事件");
        assert!(matches!(evt, WorkspaceEvent::Created(_)));
        assert_eq!(evt.subject(), "star.events.workspace.workspace.created.v1");
    }

    #[tokio::test]
    async fn update_workspace_version_conflict() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let ws = svc
            .create_workspace(
                CreateWorkspaceCommand {
                    tenant_id,
                    workspace_key: "v".to_string(),
                    name: "V".to_string(),
                    description: None,
                    owner_user_id: UserId::new(),
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .update_workspace(
                UpdateWorkspaceCommand {
                    workspace_id: ws.id,
                    tenant_id,
                    expected_version: 99,
                    name: Some("New".to_string()),
                    description: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(WorkspaceError::Conflict(_))));
    }
}

//! Project 项目模板与配置
//!
//! **crate**: `domain-project`
//! **上游 spec**: docs/specs/domain-project-spec.md
//! **基本设计**: docs/basic-design.md §4.3
//! **数据设计**: docs/data-design.md §4.3
//!
//! ## 职责
//!
//! - 颁发 `project_id` / `project_template_id` / `project_policy_id`
//! - 3 个核心实体
//! - 2 个核心 Domain Event
//! - 2 个端口(4 cmd / 5 query)
//! - 3 条不变量
//! - 1 个 `InMemoryProjectService`

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
pub use entity::{Project, ProjectPolicy, ProjectTemplate};
pub use error::ProjectError;
pub use event::{EventMeta, ProjectCreated, ProjectEvent, ProjectPolicyUpdated};
pub use invariants::{
    check_invariant_01_project_key_unique, check_invariant_02_tenant_id_present,
    check_invariant_03_project_key_format, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    ArchiveProjectCommand, CreateProjectCommand, ListProjectQuery, ProjectCommandPort,
    ProjectQueryPort, UpdateProjectCommand, UpdateProjectPolicyCommand,
};
pub use service::InMemoryProjectService;
pub use value_object::{
    roles, AgentPolicyId, NotificationSchemeId, PermissionSchemeId, ProjectId, ProjectPolicyId,
    ProjectStatus, ProjectTemplateId, ProjectTemplateType, TenantId, WorkflowId, WorkspaceId,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id).with_role(roles::PROJECT_ADMIN)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(Project::FIELD_COUNT, 16);
        assert_eq!(ProjectTemplate::FIELD_COUNT, 9);
        assert_eq!(ProjectPolicy::FIELD_COUNT, 9);
    }

    #[tokio::test]
    async fn create_project_success() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = CreateProjectCommand {
            tenant_id,
            workspace_id: WorkspaceId::new(),
            project_key: "STAR".to_string(),
            name: "Star Project".to_string(),
            description: None,
            template_type: ProjectTemplateType::SoftwareDev,
            lead_user_id: Some(uuid::Uuid::new_v4()),
        };
        let p = svc.create_project(cmd, actor).await.unwrap();
        assert_eq!(p.status, ProjectStatus::Active);
        assert_eq!(p.version, 1);
        assert_eq!(svc.count().await, 1);
    }

    #[tokio::test]
    async fn invariant_01_project_key_conflict() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let ws = WorkspaceId::new();
        let cmd1 = CreateProjectCommand {
            tenant_id,
            workspace_id: ws,
            project_key: "DUP".to_string(),
            name: "P1".to_string(),
            description: None,
            template_type: ProjectTemplateType::SoftwareDev,
            lead_user_id: None,
        };
        svc.create_project(cmd1, actor.clone()).await.unwrap();
        let cmd2 = CreateProjectCommand {
            tenant_id,
            workspace_id: ws,
            project_key: "DUP".to_string(),
            name: "P2".to_string(),
            description: None,
            template_type: ProjectTemplateType::SoftwareDev,
            lead_user_id: None,
        };
        let res = svc.create_project(cmd2, actor).await;
        assert!(matches!(res, Err(ProjectError::Conflict(_))));
    }

    #[tokio::test]
    async fn invariant_03_empty_key_rejected() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = CreateProjectCommand {
            tenant_id,
            workspace_id: WorkspaceId::new(),
            project_key: "".to_string(),
            name: "X".to_string(),
            description: None,
            template_type: ProjectTemplateType::SoftwareDev,
            lead_user_id: None,
        };
        let res = svc.create_project(cmd, actor).await;
        assert!(matches!(res, Err(ProjectError::InvalidState(_))));
    }

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_a = TenantId::new();
        let actor_a = make_actor(tenant_a);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id: tenant_a,
                    workspace_id: WorkspaceId::new(),
                    project_key: "A".to_string(),
                    name: "A".to_string(),
                    description: None,
                    template_type: ProjectTemplateType::SoftwareDev,
                    lead_user_id: None,
                },
                actor_a,
            )
            .await
            .unwrap();
        let tenant_b = TenantId::new();
        let actor_b = make_actor(tenant_b);
        let res = svc.get_by_id(p.id, actor_b).await;
        assert!(matches!(res, Err(ProjectError::PermissionDenied)));
    }

    #[tokio::test]
    async fn archive_project() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id,
                    workspace_id: WorkspaceId::new(),
                    project_key: "ARC".to_string(),
                    name: "Arc".to_string(),
                    description: None,
                    template_type: ProjectTemplateType::Kanban,
                    lead_user_id: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let p2 = svc
            .archive_project(
                ArchiveProjectCommand {
                    project_id: p.id,
                    tenant_id,
                    expected_version: 1,
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(p2.status, ProjectStatus::Archived);
        assert_eq!(p2.version, 2);
    }

    #[tokio::test]
    async fn list_built_in_templates() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let tpls = svc.list_templates(tenant_id, actor).await.unwrap();
        assert_eq!(tpls.len(), 4);
        assert!(tpls.iter().all(|t| t.built_in));
    }

    #[tokio::test]
    async fn update_project_policy_first_time() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id,
                    workspace_id: WorkspaceId::new(),
                    project_key: "POL".to_string(),
                    name: "Pol".to_string(),
                    description: None,
                    template_type: ProjectTemplateType::SoftwareDev,
                    lead_user_id: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let policy = svc
            .update_project_policy(
                UpdateProjectPolicyCommand {
                    project_id: p.id,
                    tenant_id,
                    expected_version: 0,
                    agent_policy: Some(serde_json::json!({"claude": "enabled"})),
                    worktree_policy: None,
                    validation_policy: None,
                    context_policy: None,
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(policy.version, 1);
        assert_eq!(policy.agent_policy["claude"], "enabled");
    }

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryProjectService::new();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let cmd = CreateProjectCommand {
            tenant_id,
            workspace_id: WorkspaceId::new(),
            project_key: "EVT".to_string(),
            name: "E".to_string(),
            description: None,
            template_type: ProjectTemplateType::SoftwareDev,
            lead_user_id: None,
        };
        svc.create_project(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 Created 事件");
        assert!(matches!(evt, ProjectEvent::Created(_)));
        assert_eq!(evt.subject(), "star.events.project.project.created.v1");
    }

    #[tokio::test]
    async fn update_project_version_conflict() {
        let svc = InMemoryProjectService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_actor(tenant_id);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id,
                    workspace_id: WorkspaceId::new(),
                    project_key: "V".to_string(),
                    name: "V".to_string(),
                    description: None,
                    template_type: ProjectTemplateType::SoftwareDev,
                    lead_user_id: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .update_project(
                UpdateProjectCommand {
                    project_id: p.id,
                    tenant_id,
                    expected_version: 99,
                    name: Some("N".to_string()),
                    description: None,
                    lead_user_id: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(ProjectError::Conflict(_))));
    }
}

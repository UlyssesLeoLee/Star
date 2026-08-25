//! InMemoryProjectService

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{Project, ProjectPolicy, ProjectTemplate};
use crate::error::ProjectError;
use crate::event::{EventMeta, ProjectEvent};
use crate::invariants::{
    check_invariant_01_project_key_unique, run_invariants, ALL_INVARIANT_CHECKS,
};
use crate::port::{
    ArchiveProjectCommand, CreateProjectCommand, ListProjectQuery, ProjectCommandPort,
    ProjectQueryPort, UpdateProjectCommand, UpdateProjectPolicyCommand,
};
use crate::value_object::{
    ProjectId, ProjectPolicyId, ProjectStatus, ProjectTemplateId, ProjectTemplateType, TenantId,
};

/// **InMemory Project 命令/查询服务**
pub struct InMemoryProjectService {
    projects: Arc<RwLock<HashMap<ProjectId, Project>>>,
    templates: Arc<RwLock<HashMap<ProjectTemplateId, ProjectTemplate>>>,
    policies: Arc<RwLock<HashMap<ProjectPolicyId, ProjectPolicy>>>,
    event_tx: mpsc::UnboundedSender<ProjectEvent>,
}

impl InMemoryProjectService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<ProjectEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut templates = HashMap::new();
        // 预置 4 个内置模板
        for tt in [
            ProjectTemplateType::SoftwareDev,
            ProjectTemplateType::Kanban,
            ProjectTemplateType::Scrum,
            ProjectTemplateType::Operations,
        ] {
            let id = ProjectTemplateId::new();
            let now = chrono::Utc::now();
            templates.insert(
                id,
                ProjectTemplate {
                    id,
                    tenant_id: TenantId::new(), // 全局共享
                    name: format!("Built-in {tt}"),
                    template_type: tt,
                    default_settings: serde_json::json!({}),
                    built_in: true,
                    created_at: now,
                    updated_at: now,
                    version: 1,
                },
            );
        }
        let svc = Arc::new(Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(templates)),
            policies: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    pub async fn count(&self) -> usize {
        self.projects.read().await.len()
    }
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), ProjectError> {
        if actor.tenant_id != expected {
            return Err(ProjectError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryProjectService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryProjectService {
    fn clone(&self) -> Self {
        Self {
            projects: self.projects.clone(),
            templates: self.templates.clone(),
            policies: self.policies.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl ProjectCommandPort for InMemoryProjectService {
    async fn create_project(
        &self,
        cmd: CreateProjectCommand,
        actor: ActorContext,
    ) -> Result<Project, ProjectError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let id = ProjectId::new();
        let project = Project {
            id,
            tenant_id: cmd.tenant_id,
            workspace_id: cmd.workspace_id,
            project_key: cmd.project_key.clone(),
            name: cmd.name.clone(),
            description: cmd.description,
            template_type: cmd.template_type,
            status: ProjectStatus::default(),
            workflow_id: None,
            permission_scheme_id: None,
            notification_scheme_id: None,
            agent_policy_id: None,
            lead_user_id: cmd.lead_user_id,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &project)?;
        let existing_keys: Vec<String> = self
            .projects
            .read()
            .await
            .values()
            .filter(|p| p.tenant_id == cmd.tenant_id)
            .map(|p| p.project_key.clone())
            .collect();
        check_invariant_01_project_key_unique(&project, &existing_keys)?;
        self.projects.write().await.insert(id, project.clone());

        // 事件
        let event = ProjectEvent::Created(crate::event::ProjectCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            project_id: id,
            workspace_id: cmd.workspace_id.into_uuid(),
            project_key: project.project_key.clone(),
            template_type: project.template_type,
        });
        let _ = self.event_tx.send(event);
        Ok(project)
    }

    async fn update_project(
        &self,
        cmd: UpdateProjectCommand,
        actor: ActorContext,
    ) -> Result<Project, ProjectError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.projects.write().await;
        let p = store
            .get_mut(&cmd.project_id)
            .ok_or(ProjectError::NotFound(cmd.project_id))?;
        if p.tenant_id != cmd.tenant_id {
            return Err(ProjectError::PermissionDenied);
        }
        if p.version != cmd.expected_version {
            return Err(ProjectError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, p.version
            )));
        }
        if let Some(name) = cmd.name {
            p.name = name;
        }
        if let Some(desc) = cmd.description {
            p.description = desc;
        }
        if let Some(lead) = cmd.lead_user_id {
            p.lead_user_id = lead;
        }
        p.bump_version();
        Ok(p.clone())
    }

    async fn archive_project(
        &self,
        cmd: ArchiveProjectCommand,
        actor: ActorContext,
    ) -> Result<Project, ProjectError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.projects.write().await;
        let p = store
            .get_mut(&cmd.project_id)
            .ok_or(ProjectError::NotFound(cmd.project_id))?;
        if p.tenant_id != cmd.tenant_id {
            return Err(ProjectError::PermissionDenied);
        }
        if p.version != cmd.expected_version {
            return Err(ProjectError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, p.version
            )));
        }
        p.status = ProjectStatus::Archived;
        p.bump_version();
        Ok(p.clone())
    }

    async fn update_project_policy(
        &self,
        cmd: UpdateProjectPolicyCommand,
        actor: ActorContext,
    ) -> Result<ProjectPolicy, ProjectError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.policies.write().await;
        // 查找/创建
        let p_id = {
            let existing = store
                .values()
                .find(|p| p.project_id == cmd.project_id)
                .map(|p| p.id);
            existing.unwrap_or_else(ProjectPolicyId::new)
        };
        let mut policy = store.remove(&p_id).unwrap_or_else(|| ProjectPolicy {
            id: p_id,
            project_id: cmd.project_id,
            agent_policy: serde_json::json!({}),
            worktree_policy: serde_json::json!({}),
            validation_policy: serde_json::json!({}),
            context_policy: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 0,
        });
        if policy.version != cmd.expected_version {
            // 第一次创建时 expected_version 应为 0
            if !(policy.version == 0 && cmd.expected_version == 0) {
                return Err(ProjectError::Conflict(format!(
                    "version mismatch: expected {}, actual {}",
                    cmd.expected_version, policy.version
                )));
            }
        }
        let mut changed = Vec::new();
        if let Some(v) = cmd.agent_policy {
            policy.agent_policy = v;
            changed.push("agent_policy".to_string());
        }
        if let Some(v) = cmd.worktree_policy {
            policy.worktree_policy = v;
            changed.push("worktree_policy".to_string());
        }
        if let Some(v) = cmd.validation_policy {
            policy.validation_policy = v;
            changed.push("validation_policy".to_string());
        }
        if let Some(v) = cmd.context_policy {
            policy.context_policy = v;
            changed.push("context_policy".to_string());
        }
        policy.bump_version();
        let result = policy.clone();
        store.insert(p_id, policy);

        let event = ProjectEvent::PolicyUpdated(crate::event::ProjectPolicyUpdated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            project_id: cmd.project_id,
            changed_fields: changed,
        });
        let _ = self.event_tx.send(event);
        Ok(result)
    }
}

#[async_trait]
impl ProjectQueryPort for InMemoryProjectService {
    async fn get_by_id(
        &self,
        id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Project, ProjectError> {
        let p = self
            .projects
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(ProjectError::NotFound(id))?;
        if p.tenant_id != viewer.tenant_id {
            return Err(ProjectError::PermissionDenied);
        }
        Ok(p)
    }
    async fn get_by_key(
        &self,
        tenant_id: TenantId,
        workspace_id: crate::value_object::WorkspaceId,
        project_key: &str,
        viewer: ActorContext,
    ) -> Result<Project, ProjectError> {
        Self::check_tenant(&viewer, tenant_id)?;
        self.projects
            .read()
            .await
            .values()
            .find(|p| {
                p.tenant_id == tenant_id
                    && p.workspace_id == workspace_id
                    && p.project_key == project_key
            })
            .cloned()
            .ok_or(ProjectError::NotFound(ProjectId::default()))
    }
    async fn list_projects(
        &self,
        q: ListProjectQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Project>, ProjectError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let mut all: Vec<Project> = self
            .projects
            .read()
            .await
            .values()
            .filter(|p| p.tenant_id == q.tenant_id)
            .filter(|p| q.workspace_id.map_or(true, |w| p.workspace_id == w))
            .cloned()
            .collect();
        all.sort_by(|a, b| a.project_key.cmp(&b.project_key));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }
    async fn list_templates(
        &self,
        _tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<Vec<ProjectTemplate>, ProjectError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        Ok(self.templates.read().await.values().cloned().collect())
    }
    async fn get_project_policy(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<ProjectPolicy, ProjectError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        self.policies
            .read()
            .await
            .values()
            .find(|p| p.project_id == project_id)
            .cloned()
            .ok_or(ProjectError::Internal(format!(
                "ProjectPolicy for project {project_id} not found"
            )))
    }
}

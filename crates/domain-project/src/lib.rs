//! domain-project crate
//!
//! 详细 spec: docs/specs/domain-project-spec.md
//! 上游基本设计: docs/basic-design.md §2.1(表 20) / §4.10.2 / §5.7
//! 数据设计: docs/data-design.md §4.3 (`project` schema)
//! API 设计: docs/api-design.md §3.4
//!
//! ## 职责
//!
//! Project 模板 / 配置 / Policy(REQ-TWP-003)。WorkItem / Worktree / Agent 的"配置平面"。
//! ProjectPolicy 整体替换(不允许 partial PATCH 绕过校验)。
//!
//! ## 关键不变量
//!
//! - INV-P-01:Project 必属 Workspace(workspace_id + tenant_id 必带)
//! - INV-P-02:ProjectPolicy 1:1 强一致(整体替换)
//! - INV-P-03:merge_gate=true 时,Merge 必须人类触发(§4.2.6)
//!
//! Lead 责任: project Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(TenantId);
define_uuid_id!(WorkspaceId);
define_uuid_id!(ProjectId);
define_uuid_id!(ProjectPolicyId);
define_uuid_id!(ProjectTemplateId);
define_uuid_id!(UserId);
define_uuid_id!(WorkflowId);
define_uuid_id!(PermissionSchemeId);
define_uuid_id!(AgentPolicyId);
define_uuid_id!(ValidationPolicyId);
define_uuid_id!(ContextPolicyId);
define_uuid_id!(NotificationTemplateId);
define_uuid_id!(RepositoryId);

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 实体
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub slug: String,
    pub display_name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub project_template_id: Option<ProjectTemplateId>,
    pub default_workflow_id: Option<WorkflowId>,
    pub default_permission_scheme_id: Option<PermissionSchemeId>,
    pub default_agent_policy_id: Option<AgentPolicyId>,
    pub default_validation_policy_id: Option<ValidationPolicyId>,
    pub default_context_policy_id: Option<ContextPolicyId>,
    pub max_worktrees: Option<u32>,
    pub max_agent_sessions: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Archived => "ARCHIVED",
            Self::Deleted => "DELETED",
        }
    }
}

/// ProjectPolicy(整体替换,不允许 partial)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPolicy {
    pub id: ProjectPolicyId,
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub custom_workflow_id: Option<WorkflowId>,
    pub permission_scheme_id: PermissionSchemeId,
    pub notification_template_id: Option<NotificationTemplateId>,
    pub agent_policy_id: AgentPolicyId,
    pub max_runtime_seconds: u32,
    pub max_context_tokens: u32,
    pub validation_policy_id: ValidationPolicyId,
    pub required_test_passes: u32,
    pub default_repository_id: Option<RepositoryId>,
    pub commit_requires_user: bool,
    pub pr_creation_requires_user: bool,
    pub merge_gate: bool, // INV-P-03:必须人类 merge
    pub updated_at: DateTime<Utc>,
}

impl ProjectPolicy {
    pub fn default_for(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            id: ProjectPolicyId::new(),
            project_id,
            tenant_id,
            custom_workflow_id: None,
            permission_scheme_id: PermissionSchemeId::new(),
            notification_template_id: None,
            agent_policy_id: AgentPolicyId::new(),
            max_runtime_seconds: 600,
            max_context_tokens: 64_000,
            validation_policy_id: ValidationPolicyId::new(),
            required_test_passes: 1,
            default_repository_id: None,
            commit_requires_user: true,
            pr_creation_requires_user: true,
            merge_gate: true, // INV-P-03 默认开启
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: ProjectTemplateId,
    pub name: String,
    pub category: TemplateCategory,
    pub default_workflow_id: Option<WorkflowId>,
    pub default_permission_scheme_id: Option<PermissionSchemeId>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    SoftwareDevelopment,
    DevOps,
    Research,
}

impl TemplateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SoftwareDevelopment => "software_development",
            Self::DevOps => "devops",
            Self::Research => "research",
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("slug already exists in workspace: {0}")]
    SlugExists(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectCommand {
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub slug: String,
    pub display_name: String,
    pub description: String,
    pub project_template_id: Option<ProjectTemplateId>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceProjectPolicyCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub policy: ProjectPolicy,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveProjectCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProjectQuery {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByWorkspaceQuery {
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait ProjectCommandPort: Send + Sync {
    async fn create_project(
        &self,
        cmd: CreateProjectCommand,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError>;

    /// INV-P-02:整体替换(不允许 partial PATCH)
    async fn replace_project_policy(
        &self,
        cmd: ReplaceProjectPolicyCommand,
        actor: &ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;

    async fn archive_project(
        &self,
        cmd: ArchiveProjectCommand,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError>;
}

#[async_trait]
pub trait ProjectQueryPort: Send + Sync {
    async fn get_project(
        &self,
        q: GetProjectQuery,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError>;

    async fn get_project_policy(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        actor: &ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;

    async fn list_by_workspace(
        &self,
        q: ListByWorkspaceQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Project>, ProjectError>;
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn insert_project(&self, p: Project) -> Result<(), ProjectError>;
    async fn get_project(&self, id: ProjectId) -> Result<Project, ProjectError>;
    async fn update_project(&self, p: Project) -> Result<(), ProjectError>;
    async fn list_by_workspace(
        &self,
        tid: TenantId,
        wid: WorkspaceId,
    ) -> Result<Vec<Project>, ProjectError>;

    async fn upsert_project_policy(&self, p: ProjectPolicy) -> Result<(), ProjectError>;
    async fn get_project_policy(
        &self,
        project_id: ProjectId,
    ) -> Result<ProjectPolicy, ProjectError>;

    async fn get_template(&self, id: ProjectTemplateId) -> Result<ProjectTemplate, ProjectError>;
}

// =====================================================================
// InMemoryProjectService
// =====================================================================

pub struct InMemoryProjectService {
    repo: Arc<dyn ProjectRepository>,
    projects: Arc<RwLock<HashMap<ProjectId, Project>>>,
    policies: Arc<RwLock<HashMap<ProjectId, ProjectPolicy>>>,
}

impl InMemoryProjectService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryProjectRepository::new()),
            projects: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProjectService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectCommandPort for InMemoryProjectService {
    async fn create_project(
        &self,
        cmd: CreateProjectCommand,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(ProjectError::PermissionDenied);
        }
        // 同 workspace 内 slug 唯一
        let dup = {
            let all = self
                .repo
                .list_by_workspace(cmd.tenant_id, cmd.workspace_id)
                .await?;
            all.iter().any(|p| p.slug == cmd.slug)
        };
        if dup {
            return Err(ProjectError::SlugExists(cmd.slug));
        }
        let now = Utc::now();
        let p = Project {
            id: ProjectId::new(),
            tenant_id: cmd.tenant_id,
            workspace_id: cmd.workspace_id,
            slug: cmd.slug,
            display_name: cmd.display_name,
            description: cmd.description,
            status: ProjectStatus::Active,
            project_template_id: cmd.project_template_id,
            default_workflow_id: None,
            default_permission_scheme_id: None,
            default_agent_policy_id: None,
            default_validation_policy_id: None,
            default_context_policy_id: None,
            max_worktrees: Some(20),
            max_agent_sessions: Some(10),
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_project(p.clone()).await?;
        let policy = ProjectPolicy::default_for(p.id, p.tenant_id);
        self.repo.upsert_project_policy(policy.clone()).await?;
        self.projects.write().unwrap().insert(p.id, p.clone());
        self.policies.write().unwrap().insert(p.id, policy);
        Ok(p)
    }

    async fn replace_project_policy(
        &self,
        cmd: ReplaceProjectPolicyCommand,
        actor: &ActorContext,
    ) -> Result<ProjectPolicy, ProjectError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("project_admin") {
            return Err(ProjectError::PermissionDenied);
        }
        let project = self
            .projects
            .read()
            .unwrap()
            .get(&cmd.project_id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!(
                "project:{}",
                cmd.project_id.as_uuid()
            )))?;
        if project.tenant_id != cmd.tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                project.tenant_id,
                cmd.tenant_id,
            ));
        }
        // INV-P-02:整体替换 — 强制 project_id + tenant_id
        let mut policy = cmd.policy;
        policy.project_id = cmd.project_id;
        policy.tenant_id = cmd.tenant_id;
        policy.updated_at = Utc::now();
        self.repo.upsert_project_policy(policy.clone()).await?;
        self.policies
            .write()
            .unwrap()
            .insert(cmd.project_id, policy.clone());
        Ok(policy)
    }

    async fn archive_project(
        &self,
        cmd: ArchiveProjectCommand,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(ProjectError::PermissionDenied);
        }
        let mut p = self
            .projects
            .write()
            .unwrap()
            .get_mut(&cmd.project_id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!(
                "project:{}",
                cmd.project_id.as_uuid()
            )))?;
        if p.tenant_id != cmd.tenant_id {
            return Err(ProjectError::CrossTenantDenied(p.tenant_id, cmd.tenant_id));
        }
        if p.status == ProjectStatus::Deleted {
            return Err(ProjectError::InvalidState("already deleted".to_string()));
        }
        p.status = ProjectStatus::Archived;
        p.updated_at = Utc::now();
        self.repo.update_project(p.clone()).await?;
        self.projects.write().unwrap().insert(p.id, p.clone());
        Ok(p)
    }
}

#[async_trait]
impl ProjectQueryPort for InMemoryProjectService {
    async fn get_project(
        &self,
        q: GetProjectQuery,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let p = self
            .projects
            .read()
            .unwrap()
            .get(&q.project_id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!(
                "project:{}",
                q.project_id.as_uuid()
            )))?;
        if p.tenant_id != q.tenant_id {
            return Err(ProjectError::CrossTenantDenied(p.tenant_id, q.tenant_id));
        }
        Ok(p)
    }

    async fn get_project_policy(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        actor: &ActorContext,
    ) -> Result<ProjectPolicy, ProjectError> {
        if TenantId::from(actor.tenant_id) != tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        let p = self
            .policies
            .read()
            .unwrap()
            .get(&project_id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!(
                "policy:{}",
                project_id.as_uuid()
            )))?;
        if p.tenant_id != tenant_id {
            return Err(ProjectError::CrossTenantDenied(p.tenant_id, tenant_id));
        }
        Ok(p)
    }

    async fn list_by_workspace(
        &self,
        q: ListByWorkspaceQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Project>, ProjectError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(ProjectError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let projects = self.projects.read().unwrap();
        Ok(projects
            .values()
            .filter(|p| p.tenant_id == q.tenant_id && p.workspace_id == q.workspace_id)
            .cloned()
            .collect())
    }
}

// =====================================================================
// InMemoryProjectRepository
// =====================================================================

pub struct InMemoryProjectRepository {
    projects: RwLock<HashMap<ProjectId, Project>>,
    policies: RwLock<HashMap<ProjectId, ProjectPolicy>>,
    templates: RwLock<HashMap<ProjectTemplateId, ProjectTemplate>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            projects: RwLock::new(HashMap::new()),
            policies: RwLock::new(HashMap::new()),
            templates: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryProjectRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn insert_project(&self, p: Project) -> Result<(), ProjectError> {
        self.projects.write().unwrap().insert(p.id, p);
        Ok(())
    }
    async fn get_project(&self, id: ProjectId) -> Result<Project, ProjectError> {
        self.projects
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!("project:{}", id.as_uuid())))
    }
    async fn update_project(&self, p: Project) -> Result<(), ProjectError> {
        self.projects.write().unwrap().insert(p.id, p);
        Ok(())
    }
    async fn list_by_workspace(
        &self,
        tid: TenantId,
        wid: WorkspaceId,
    ) -> Result<Vec<Project>, ProjectError> {
        Ok(self
            .projects
            .read()
            .unwrap()
            .values()
            .filter(|p| p.tenant_id == tid && p.workspace_id == wid)
            .cloned()
            .collect())
    }
    async fn upsert_project_policy(&self, p: ProjectPolicy) -> Result<(), ProjectError> {
        self.policies.write().unwrap().insert(p.project_id, p);
        Ok(())
    }
    async fn get_project_policy(
        &self,
        project_id: ProjectId,
    ) -> Result<ProjectPolicy, ProjectError> {
        self.policies
            .read()
            .unwrap()
            .get(&project_id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!(
                "policy:{}",
                project_id.as_uuid()
            )))
    }
    async fn get_template(&self, id: ProjectTemplateId) -> Result<ProjectTemplate, ProjectError> {
        self.templates
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(ProjectError::NotFound(format!("template:{}", id.as_uuid())))
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn admin(tid: uuid::Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tid).with_role("project_admin")
    }

    #[test]
    fn project_status_as_str() {
        assert_eq!(ProjectStatus::Active.as_str(), "ACTIVE");
        assert_eq!(ProjectStatus::Archived.as_str(), "ARCHIVED");
    }

    #[test]
    fn template_category_as_str() {
        assert_eq!(
            TemplateCategory::SoftwareDevelopment.as_str(),
            "software_development"
        );
    }

    #[test]
    fn default_policy_has_merge_gate() {
        // INV-P-03:默认 merge_gate=true
        let p = ProjectPolicy::default_for(ProjectId::new(), TenantId(uuid::Uuid::new_v4()));
        assert!(p.merge_gate);
        assert!(p.commit_requires_user);
    }

    #[tokio::test]
    async fn create_project_requires_project_admin() {
        let svc = InMemoryProjectService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = ActorContext::new(Uuid::new_v4(), tid);
        let res = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id: TenantId(tid),
                    workspace_id: WorkspaceId::new(),
                    slug: "alpha".to_string(),
                    display_name: "Alpha".to_string(),
                    description: "".to_string(),
                    project_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(ProjectError::PermissionDenied)));
    }

    #[tokio::test]
    async fn create_project_unique_slug_per_workspace() {
        let svc = InMemoryProjectService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = admin(tid);
        let wid = WorkspaceId::new();
        svc.create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tid),
                workspace_id: wid,
                slug: "alpha".to_string(),
                display_name: "Alpha".to_string(),
                description: "".to_string(),
                project_template_id: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        let res = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id: TenantId(tid),
                    workspace_id: wid,
                    slug: "alpha".to_string(),
                    display_name: "Alpha2".to_string(),
                    description: "".to_string(),
                    project_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(ProjectError::SlugExists(_))));
    }

    #[tokio::test]
    async fn same_slug_different_workspace_ok() {
        let svc = InMemoryProjectService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = admin(tid);
        let wid1 = WorkspaceId::new();
        let wid2 = WorkspaceId::new();
        svc.create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tid),
                workspace_id: wid1,
                slug: "alpha".to_string(),
                display_name: "A1".to_string(),
                description: "".to_string(),
                project_template_id: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        svc.create_project(
            CreateProjectCommand {
                tenant_id: TenantId(tid),
                workspace_id: wid2,
                slug: "alpha".to_string(),
                display_name: "A2".to_string(),
                description: "".to_string(),
                project_template_id: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn replace_policy_overwrites() {
        let svc = InMemoryProjectService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = admin(tid);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id: TenantId(tid),
                    workspace_id: WorkspaceId::new(),
                    slug: "x".to_string(),
                    display_name: "X".to_string(),
                    description: "".to_string(),
                    project_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let mut new_policy = ProjectPolicy::default_for(p.id, TenantId(tid));
        new_policy.merge_gate = false; // 关闭 merge gate
        new_policy.required_test_passes = 3;
        let updated = svc
            .replace_project_policy(
                ReplaceProjectPolicyCommand {
                    tenant_id: TenantId(tid),
                    project_id: p.id,
                    policy: new_policy,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert!(!updated.merge_gate);
        assert_eq!(updated.required_test_passes, 3);
    }

    #[tokio::test]
    async fn cross_tenant_get_denied() {
        let svc = InMemoryProjectService::new();
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let admin1 = admin(t1);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id: TenantId(t1),
                    workspace_id: WorkspaceId::new(),
                    slug: "x".to_string(),
                    display_name: "X".to_string(),
                    description: "".to_string(),
                    project_template_id: None,
                    actor_user_id: UserId::from(admin1.user_id),
                },
                &admin1,
            )
            .await
            .unwrap();
        let admin2 = admin(t2);
        let res = svc
            .get_project(
                GetProjectQuery {
                    tenant_id: TenantId(t1),
                    project_id: p.id,
                },
                &admin2,
            )
            .await;
        assert!(matches!(res, Err(ProjectError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn archive_project() {
        let svc = InMemoryProjectService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = admin(tid);
        let p = svc
            .create_project(
                CreateProjectCommand {
                    tenant_id: TenantId(tid),
                    workspace_id: WorkspaceId::new(),
                    slug: "x".to_string(),
                    display_name: "X".to_string(),
                    description: "".to_string(),
                    project_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let res = svc
            .archive_project(
                ArchiveProjectCommand {
                    tenant_id: TenantId(tid),
                    project_id: p.id,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(res.status, ProjectStatus::Archived);
    }

    #[tokio::test]
    async fn list_by_workspace() {
        let svc = InMemoryProjectService::new();
        let tid = uuid::Uuid::new_v4();
        let actor = admin(tid);
        let wid = WorkspaceId::new();
        for i in 0..3 {
            svc.create_project(
                CreateProjectCommand {
                    tenant_id: TenantId(tid),
                    workspace_id: wid,
                    slug: format!("p{}", i),
                    display_name: format!("P{}", i),
                    description: "".to_string(),
                    project_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        }
        let list = svc
            .list_by_workspace(
                ListByWorkspaceQuery {
                    tenant_id: TenantId(tid),
                    workspace_id: wid,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 3);
    }
}

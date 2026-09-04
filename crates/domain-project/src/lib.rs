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
/// 生成基于 Uuid 的强类型 ID 宏,统一实现 new/as_uuid/From<Uuid>/Display
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 领域强类型 ID(由 `define_uuid_id!` 宏统一生成)
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成一个新的随机 ID(由宏统一生成)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回底层 Uuid 值(由宏统一生成)
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
/// 项目实体,隶属某个 Workspace,承载 Policy/模板等配置平面
pub struct Project {
    /// 项目 ID
    pub id: ProjectId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属 Workspace ID
    pub workspace_id: WorkspaceId,
    /// 项目 slug(同一 Workspace 内唯一)
    pub slug: String,
    /// 项目展示名称
    pub display_name: String,
    /// 项目描述
    pub description: String,
    /// 项目状态
    pub status: ProjectStatus,
    /// 创建时使用的项目模板 ID(可选)
    pub project_template_id: Option<ProjectTemplateId>,
    /// 默认工作流 ID(可选)
    pub default_workflow_id: Option<WorkflowId>,
    /// 默认权限方案 ID(可选)
    pub default_permission_scheme_id: Option<PermissionSchemeId>,
    /// 默认 Agent 策略 ID(可选)
    pub default_agent_policy_id: Option<AgentPolicyId>,
    /// 默认校验策略 ID(可选)
    pub default_validation_policy_id: Option<ValidationPolicyId>,
    /// 默认上下文策略 ID(可选)
    pub default_context_policy_id: Option<ContextPolicyId>,
    /// 最大 Worktree 数量上限(可选)
    pub max_worktrees: Option<u32>,
    /// 最大 Agent 会话数量上限(可选)
    pub max_agent_sessions: Option<u32>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 项目状态枚举
pub enum ProjectStatus {
    /// 活跃状态
    Active,
    /// 已归档状态
    Archived,
    /// 已删除状态
    Deleted,
}

impl ProjectStatus {
    /// 返回状态对应的字符串表示
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
    /// 策略 ID
    pub id: ProjectPolicyId,
    /// 所属项目 ID
    pub project_id: ProjectId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 自定义工作流 ID(可选,覆盖项目默认值)
    pub custom_workflow_id: Option<WorkflowId>,
    /// 权限方案 ID
    pub permission_scheme_id: PermissionSchemeId,
    /// 通知模板 ID(可选)
    pub notification_template_id: Option<NotificationTemplateId>,
    /// Agent 策略 ID
    pub agent_policy_id: AgentPolicyId,
    /// Agent 最大运行时长(秒)
    pub max_runtime_seconds: u32,
    /// 最大上下文 Token 数
    pub max_context_tokens: u32,
    /// 校验策略 ID
    pub validation_policy_id: ValidationPolicyId,
    /// 要求通过的测试次数
    pub required_test_passes: u32,
    /// 默认代码仓库 ID(可选)
    pub default_repository_id: Option<RepositoryId>,
    /// 提交是否需要人类确认
    pub commit_requires_user: bool,
    /// 创建 PR 是否需要人类确认
    pub pr_creation_requires_user: bool,
    /// 是否启用 Merge 人工门禁(INV-P-03)
    pub merge_gate: bool, // INV-P-03:必须人类 merge
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
}

impl ProjectPolicy {
    /// 生成给定项目的默认策略配置
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
/// 项目模板,用于创建项目时预置默认配置
pub struct ProjectTemplate {
    /// 模板 ID
    pub id: ProjectTemplateId,
    /// 模板名称
    pub name: String,
    /// 模板分类
    pub category: TemplateCategory,
    /// 默认工作流 ID(可选)
    pub default_workflow_id: Option<WorkflowId>,
    /// 默认权限方案 ID(可选)
    pub default_permission_scheme_id: Option<PermissionSchemeId>,
    /// 模板版本号
    pub version: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 项目模板分类
pub enum TemplateCategory {
    /// 软件开发类模板
    SoftwareDevelopment,
    /// DevOps 类模板
    DevOps,
    /// 研究类模板
    Research,
}

impl TemplateCategory {
    /// 返回分类对应的字符串表示
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
/// Project 领域错误类型
pub enum ProjectError {
    #[error("not found: {0}")]
    /// 资源未找到
    NotFound(String),
    #[error("permission denied")]
    /// 权限不足
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    /// 跨租户访问被拒绝(实际租户 vs 要求租户)
    CrossTenantDenied(TenantId, TenantId),
    #[error("slug already exists in workspace: {0}")]
    /// Slug 在该 Workspace 内已存在
    SlugExists(String),
    #[error("invalid state: {0}")]
    /// 状态非法,无法执行该操作
    InvalidState(String),
    #[error("conflict: {0}")]
    /// 资源冲突
    Conflict(String),
    #[error("internal: {0}")]
    /// 内部错误
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建项目命令
pub struct CreateProjectCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Workspace ID
    pub workspace_id: WorkspaceId,
    /// 项目 slug
    pub slug: String,
    /// 项目展示名称
    pub display_name: String,
    /// 项目描述
    pub description: String,
    /// 使用的项目模板 ID(可选)
    pub project_template_id: Option<ProjectTemplateId>,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 整体替换项目策略命令(INV-P-02)
pub struct ReplaceProjectPolicyCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标项目 ID
    pub project_id: ProjectId,
    /// 待写入的完整策略
    pub policy: ProjectPolicy,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 归档项目命令
pub struct ArchiveProjectCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标项目 ID
    pub project_id: ProjectId,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 查询单个项目的请求
pub struct GetProjectQuery {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标项目 ID
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 按 Workspace 列出项目的请求
pub struct ListByWorkspaceQuery {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Workspace ID
    pub workspace_id: WorkspaceId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
/// 项目命令端口(创建 / 替换策略 / 归档)
pub trait ProjectCommandPort: Send + Sync {
    /// 创建新项目
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

    /// 归档项目
    async fn archive_project(
        &self,
        cmd: ArchiveProjectCommand,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError>;
}

#[async_trait]
/// 项目查询端口
pub trait ProjectQueryPort: Send + Sync {
    /// 获取单个项目
    async fn get_project(
        &self,
        q: GetProjectQuery,
        actor: &ActorContext,
    ) -> Result<Project, ProjectError>;

    /// 获取项目策略
    async fn get_project_policy(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        actor: &ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;

    /// 按 Workspace 列出项目
    async fn list_by_workspace(
        &self,
        q: ListByWorkspaceQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Project>, ProjectError>;
}

#[async_trait]
/// 项目仓储端口(持久化抽象)
pub trait ProjectRepository: Send + Sync {
    /// 插入新项目记录
    async fn insert_project(&self, p: Project) -> Result<(), ProjectError>;
    /// 按 ID 获取项目
    async fn get_project(&self, id: ProjectId) -> Result<Project, ProjectError>;
    /// 更新项目记录
    async fn update_project(&self, p: Project) -> Result<(), ProjectError>;
    /// 按 Workspace 列出项目
    async fn list_by_workspace(
        &self,
        tid: TenantId,
        wid: WorkspaceId,
    ) -> Result<Vec<Project>, ProjectError>;

    /// 插入或更新项目策略
    async fn upsert_project_policy(&self, p: ProjectPolicy) -> Result<(), ProjectError>;
    /// 获取项目策略
    async fn get_project_policy(
        &self,
        project_id: ProjectId,
    ) -> Result<ProjectPolicy, ProjectError>;

    /// 按 ID 获取项目模板
    async fn get_template(&self, id: ProjectTemplateId) -> Result<ProjectTemplate, ProjectError>;
}

// =====================================================================
// InMemoryProjectService
// =====================================================================

/// 基于内存的 ProjectService 实现(用于测试/开发)
pub struct InMemoryProjectService {
    repo: Arc<dyn ProjectRepository>,
    projects: Arc<RwLock<HashMap<ProjectId, Project>>>,
    policies: Arc<RwLock<HashMap<ProjectId, ProjectPolicy>>>,
}

impl InMemoryProjectService {
    /// 创建一个新的内存项目服务实例
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

/// 基于内存的 ProjectRepository 实现(用于测试/开发)
pub struct InMemoryProjectRepository {
    projects: RwLock<HashMap<ProjectId, Project>>,
    policies: RwLock<HashMap<ProjectId, ProjectPolicy>>,
    templates: RwLock<HashMap<ProjectTemplateId, ProjectTemplate>>,
}

impl InMemoryProjectRepository {
    /// 创建一个新的内存项目仓储实例
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

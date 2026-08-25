//! SCM 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.19 (Repository / Branch / Commit / PR / Webhook 端点)
//! - `docs/specs/domain-scm-spec.md` §4 (接口签名)
//! - `docs/basic-design.md` §4.7.3 (ScmPort 抽象)
//!
//! **端口清单**:
//! - `ScmCommandPort`:7 方法(写,含 register / link / sync / configure_webhook / rotate_token / disconnect 等)
//! - `ScmQueryPort`:5 方法(读,含 get / list / list_branches / get_pr / list_webhook_events)
//! - `ScmRepository`:基础设施层使用,本文件声明 trait
//! - `ScmPort`:厂商适配器抽象(GitHub / GitLab / Bitbucket),由 infrastructure crate 实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Branch, Pipeline, PullRequest, Repository, WebhookEvent};
use crate::error::ScmError;
use crate::value_object::{
    BranchId, CommitId, ExternalRepositoryId, PipelineStatus, ProjectId, PullRequestId,
    PullRequestState, RepositoryId, RepositoryOwnership, ScmProvider, SyncStatus, TenantId,
    UserId, WebhookEventId, WebhookEventType, WorkItemId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `RegisterRepositoryCommand`(注册 Repository,Connected 模式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRepositoryCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// Provider
    pub provider: ScmProvider,
    /// 外部 ID
    pub external_id: ExternalRepositoryId,
    /// URL
    pub url: String,
    /// 默认分支
    pub default_branch: String,
    /// 所有权(默认 Connected)
    pub ownership: RepositoryOwnership,
    /// Credential ID 引用
    pub credential_id: Option<uuid::Uuid>,
}

/// `LinkToProjectCommand`(将 Repository 关联到 Project)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkToProjectCommand {
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
}

/// `UpdateSyncStateCommand`(更新同步状态)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSyncStateCommand {
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 新同步状态
    pub sync_status: SyncStatus,
    /// 新同步 Token
    pub sync_token: Option<String>,
    /// 同步时间
    pub synced_at: DateTime<Utc>,
}

/// `ConfigureWebhookCommand`(注册 Webhook)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureWebhookCommand {
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Webhook 端点 URL
    pub endpoint_url: String,
    /// 订阅的事件类型
    pub event_types: Vec<WebhookEventType>,
    /// Secret(明文,基础设施层会加密入库)
    pub secret: String,
}

/// `RotateTokenCommand`(轮换 Credential)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateTokenCommand {
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 新 Credential ID
    pub new_credential_id: uuid::Uuid,
}

/// `RecordWebhookEventCommand`(记录入站 Webhook 事件)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordWebhookEventCommand {
    /// Provider
    pub provider: ScmProvider,
    /// 事件类型
    pub event_type: WebhookEventType,
    /// 原始 payload(JSON 字符串)
    pub payload: String,
    /// 签名
    pub signature: Option<String>,
    /// 幂等 Key
    pub idempotency_key: Option<String>,
}

/// `TransitionPullRequestCommand`(PR 状态机迁移)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionPullRequestCommand {
    /// PR ID
    pub pull_request_id: PullRequestId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 目标状态
    pub next_state: PullRequestState,
    /// 触发者(可选)
    pub triggered_by: Option<UserId>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListBranchesQuery`(列出 Repository 下的 Branch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBranchesQuery {
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    /// 仅列出受保护分支
    pub protected_only: bool,
}

/// `ListPullRequestQuery`(列出 PR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPullRequestQuery {
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    /// 按状态过滤(None = 全部)
    pub state_filter: Option<PullRequestState>,
}

/// `ListWebhookEventsQuery`(列出 Webhook 事件)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWebhookEventsQuery {
    pub tenant_id: Option<TenantId>,
    pub provider: Option<ScmProvider>,
    pub unprocessed_only: bool,
    pub limit: u32,
}

// =====================================================================
// 端口:ScmCommandPort(7 方法)
// =====================================================================

/// **SCM 命令端口**(写操作 7 方法)
#[async_trait]
pub trait ScmCommandPort: Send + Sync {
    /// 注册 Repository(INV-SCM-01/02/03/04/05 校验)
    async fn register_repository(
        &self,
        cmd: RegisterRepositoryCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;

    /// 将 Repository 关联到 Project(INV-SCM-04 跨 tenant 拒绝)
    async fn link_to_project(
        &self,
        cmd: LinkToProjectCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;

    /// 更新同步状态(INV-SCM-03 校验)
    async fn update_sync_state(
        &self,
        cmd: UpdateSyncStateCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;

    /// 配置 Webhook
    async fn configure_webhook(
        &self,
        cmd: ConfigureWebhookCommand,
        actor: ActorContext,
    ) -> Result<WebhookEvent, ScmError>;

    /// 轮换 Credential
    async fn rotate_token(
        &self,
        cmd: RotateTokenCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;

    /// 记录入站 Webhook 事件(INV-SCM-08 幂等校验)
    async fn record_webhook_event(
        &self,
        cmd: RecordWebhookEventCommand,
    ) -> Result<WebhookEvent, ScmError>;

    /// PR 状态机迁移(INV-SCM-07)
    async fn transition_pull_request(
        &self,
        cmd: TransitionPullRequestCommand,
        actor: ActorContext,
    ) -> Result<PullRequest, ScmError>;
}

// =====================================================================
// 端口:ScmQueryPort(5 方法)
// =====================================================================

/// **SCM 查询端口**(读操作 5 方法)
#[async_trait]
pub trait ScmQueryPort: Send + Sync {
    /// 按 ID 查询 Repository(带租户隔离校验)
    async fn get_repository(
        &self,
        id: RepositoryId,
        viewer: ActorContext,
    ) -> Result<Repository, ScmError>;

    /// 列出 Project 下的 Repository
    async fn list_repositories_by_project(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Vec<Repository>, ScmError>;

    /// 列出 Repository 下的 Branch
    async fn list_branches(
        &self,
        q: ListBranchesQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Branch>, ScmError>;

    /// 按 ID 查询 PR
    async fn get_pull_request(
        &self,
        id: PullRequestId,
        viewer: ActorContext,
    ) -> Result<PullRequest, ScmError>;

    /// 列出 Webhook 事件
    async fn list_webhook_events(
        &self,
        q: ListWebhookEventsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<WebhookEvent>, ScmError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **SCM 仓库端口**(供 SQLx / 内存 / 测试 Adapter 实现)
#[async_trait]
pub trait ScmRepository: Send + Sync {
    /// Repository CRUD
    async fn insert_repository(&self, repo: &Repository) -> Result<(), ScmError>;
    async fn find_repository_by_id(
        &self,
        id: RepositoryId,
    ) -> Result<Option<Repository>, ScmError>;
    async fn update_repository(&self, repo: &Repository) -> Result<(), ScmError>;
    async fn delete_repository(&self, id: RepositoryId) -> Result<(), ScmError>;
    async fn list_repositories_raw(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<Repository>, ScmError>;
    /// 按 (tenant_id, provider, external_id) UNIQUE 查找
    async fn find_repository_by_external(
        &self,
        tenant_id: TenantId,
        provider: ScmProvider,
        external_id: &ExternalRepositoryId,
    ) -> Result<Option<Repository>, ScmError>;

    /// Branch CRUD
    async fn insert_branch(&self, branch: &Branch) -> Result<(), ScmError>;
    async fn list_branches_raw(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
    ) -> Result<Vec<Branch>, ScmError>;
    async fn update_branch(&self, branch: &Branch) -> Result<(), ScmError>;
    async fn delete_branch(&self, id: BranchId) -> Result<(), ScmError>;

    /// PullRequest CRUD
    async fn insert_pull_request(&self, pr: &PullRequest) -> Result<(), ScmError>;
    async fn find_pull_request_by_id(
        &self,
        id: PullRequestId,
    ) -> Result<Option<PullRequest>, ScmError>;
    async fn update_pull_request(&self, pr: &PullRequest) -> Result<(), ScmError>;
    async fn list_pull_requests_raw(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        state_filter: Option<PullRequestState>,
    ) -> Result<Vec<PullRequest>, ScmError>;

    /// Pipeline CRUD
    async fn insert_pipeline(&self, p: &Pipeline) -> Result<(), ScmError>;
    async fn list_pipelines_raw(
        &self,
        tenant_id: TenantId,
        pull_request_id: PullRequestId,
    ) -> Result<Vec<Pipeline>, ScmError>;
    async fn update_pipeline_status(
        &self,
        id: uuid::Uuid,
        status: PipelineStatus,
    ) -> Result<(), ScmError>;

    /// WebhookEvent CRUD
    async fn insert_webhook_event(&self, evt: &WebhookEvent) -> Result<(), ScmError>;
    async fn find_webhook_event_by_idempotency(
        &self,
        provider: ScmProvider,
        idempotency_key: &str,
    ) -> Result<Option<WebhookEvent>, ScmError>;
    async fn list_webhook_events_raw(
        &self,
        tenant_id: Option<TenantId>,
        provider: Option<ScmProvider>,
        unprocessed_only: bool,
        limit: u32,
    ) -> Result<Vec<WebhookEvent>, ScmError>;
    async fn update_webhook_event(&self, evt: &WebhookEvent) -> Result<(), ScmError>;
}

// =====================================================================
// 端口:ScmPort(厂商适配器抽象,§4.7.3)
// =====================================================================

/// **ScmPort**(厂商适配器抽象,§4.7.3)
///
/// 跨域使用,实现位于 `crates/infrastructure/src/scm/{github,gitlab,bitbucket}.rs`。
/// 本 trait 仅定义厂商 API 抽象;**Domain 层不出现厂商特有对象**(INV-SCM-01)。
#[async_trait]
pub trait ScmPort: Send + Sync {
    /// 仓库元数据(读)
    async fn get_repository(
        &self,
        external_id: ExternalRepositoryId,
    ) -> Result<Repository, ScmError>;
    /// 列出分支
    async fn list_branches(
        &self,
        repository_id: ExternalRepositoryId,
    ) -> Result<Vec<Branch>, ScmError>;
    /// 取得 commit
    async fn get_commit(
        &self,
        repository_id: ExternalRepositoryId,
        sha: &str,
    ) -> Result<crate::entity::Commit, ScmError>;
    /// 取得 PR
    async fn get_pull_request(
        &self,
        repository_id: ExternalRepositoryId,
        external_pr_id: &str,
    ) -> Result<PullRequest, ScmError>;
    /// 列出 PR(带过滤)
    async fn list_pull_requests(
        &self,
        repository_id: ExternalRepositoryId,
        state: Option<PullRequestState>,
    ) -> Result<Vec<PullRequest>, ScmError>;

    /// 写入操作
    async fn create_pull_request(
        &self,
        repository_id: ExternalRepositoryId,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<PullRequest, ScmError>;

    /// Webhook 注册
    async fn register_webhook(
        &self,
        repository_id: ExternalRepositoryId,
        endpoint_url: &str,
        events: &[WebhookEventType],
        secret: &str,
    ) -> Result<String, ScmError>;
}

// 静默抑制未使用导入(供未来扩展使用)
#[allow(dead_code)]
fn _unused_imports() {
    let _ = CommitId::new();
    let _ = WebhookEventId::new();
    let _ = WorkItemId::new();
}

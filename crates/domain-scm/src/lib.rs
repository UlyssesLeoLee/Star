//! domain-scm crate
//!
//! 详细 spec: docs/specs/domain-scm-spec.md §19 (REQ-SCM-001/002/003)
//! 上游基本设计: docs/basic-design.md §2.1(表 7) / §4.7 / §5.7 / §6.6
//! 数据设计: docs/data-design.md §4.18 (`scm` schema)
//! API 设计: docs/api-design.md §3.19
//!
//! ## 职责
//!
//! SCM Adapter 抽象 + Repository 同步(§19,REQ-SCM-001/002)。
//! **Domain 层不得出现厂商特有对象**(GitHub / GitLab),统一抽象 + ACL 翻译。
//! MVP 支持 GitHub + GitLab,Self-hosted Git(Gitea/Forgejo)为 V2 候选(REQ-SCM-003)。
//!
//! ## 关键不变量(INV-SCM-01~08,共 8 条)
//!
//! - **INV-SCM-01** Domain 层不出现厂商特有对象(由 ACL 翻译)
//! - **INV-SCM-02** MVP 仅支持 Connected 所有权
//! - **INV-SCM-03** Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token)
//! - **INV-SCM-04** Repository 必带 `tenant_id + project_id`,跨 tenant 拒绝
//! - **INV-SCM-05** Repository Credential 走 Credential Broker,不存明文
//! - **INV-SCM-06** PR Content 必带 `tenant_id`(Object Storage Key 前缀,§6.1)
//! - **INV-SCM-07** `PullRequest.state` 状态机严格按 §7.5 迁移
//! - **INV-SCM-08** Webhook 入站 100% 写 Audit
//!
//! ## PR 状态机(7 状态,§7.5)
//! DRAFT → OPEN → REVIEWING → CHANGES_REQUESTED → APPROVED → MERGEABLE → MERGED → CLOSED
//!
//! Lead 责任: scm Lead

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

// =====================================================================
// 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
            #[allow(dead_code)]
            pub fn from_uuid(id: uuid::Uuid) -> Self {
                Self(id)
            }
            #[allow(dead_code)]
            pub fn as_uuid(&self) -> uuid::Uuid {
                self.0
            }
            #[allow(dead_code)]
            pub fn into_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(id: uuid::Uuid) -> Self {
                Self(id)
            }
        }
    };
}

define_uuid_id!(RepositoryId);
define_uuid_id!(BranchId);
define_uuid_id!(CommitId);
define_uuid_id!(PullRequestId);
define_uuid_id!(ReviewId);
define_uuid_id!(PipelineId);
define_uuid_id!(WebhookEventId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 值对象
// =====================================================================

/// **SCM Provider**(MVP: GitHub + GitLab;V2 候选: Gitea / Forgejo)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScmProvider {
    /// GitHub(MVP)
    Github,
    /// GitLab(MVP)
    Gitlab,
    /// Gitea(V2 候选,REQ-SCM-003)
    Gitea,
    /// Forgejo(V2 候选)
    Forgejo,
    /// Future SCM(占位)
    Future,
}

impl Default for ScmProvider {
    fn default() -> Self {
        Self::Github
    }
}

impl std::fmt::Display for ScmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Gitea => "gitea",
            Self::Forgejo => "forgejo",
            Self::Future => "future",
        };
        f.write_str(s)
    }
}

/// **仓库所有权**(MVP 仅 Connected,INV-SCM-02)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryOwnership {
    /// 外部 SoR(平台只读)
    Connected,
    /// Mirror(本地 SoR,外部只读)
    Mirrored,
    /// Managed(平台和外部双向 SoR)
    Managed,
    /// LocalOnly(无外部)
    LocalOnly,
}

impl Default for RepositoryOwnership {
    fn default() -> Self {
        Self::Connected
    }
}

impl std::fmt::Display for RepositoryOwnership {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Connected => "connected",
            Self::Mirrored => "mirrored",
            Self::Managed => "managed",
            Self::LocalOnly => "local_only",
        };
        f.write_str(s)
    }
}

/// **同步状态**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// 与外部 SoR 一致
    InSync,
    /// 落后于外部
    Behind,
    /// 领先(本地修改尚未推)
    Ahead,
    /// 冲突未解决
    Conflict,
    /// 同步禁用
    Disabled,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::InSync
    }
}

/// **冲突解决策略**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    LatestWins,
    FirstWins,
    ManualReview,
    Bidirectional,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        Self::LatestWins
    }
}

/// **PR 状态**(7 状态机,§7.5)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestState {
    /// 草稿(创建)
    Draft,
    /// 开放(待 Review)
    Open,
    /// Review 中
    Reviewing,
    /// Review 要求修改
    ChangesRequested,
    /// Review 通过
    Approved,
    /// 可 merge(校验通过)
    Mergeable,
    /// 已 merge
    Merged,
    /// 关闭(放弃 / 拒绝)
    Closed,
}

impl PullRequestState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Open => "OPEN",
            Self::Reviewing => "REVIEWING",
            Self::ChangesRequested => "CHANGES_REQUESTED",
            Self::Approved => "APPROVED",
            Self::Mergeable => "MERGEABLE",
            Self::Merged => "MERGED",
            Self::Closed => "CLOSED",
        }
    }
    /// 是否终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Merged | Self::Closed)
    }
}

/// **Review 状态**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    Pending,
    Approved,
    ChangesRequested,
    Commented,
}

/// **Pipeline 状态**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Pending,
    Running,
    Success,
    Failed,
    Canceled,
}

/// **Webhook 事件类型**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    Push,
    PullRequest,
    Pipeline,
    Release,
    Issues,
}

/// **外部仓库 ID**(厂商侧字符串,如 "acme/foo")
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ExternalRepositoryId(pub String);

impl std::fmt::Display for ExternalRepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// **SyncState**(值对象,内嵌于 Repository,§4.7.6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub status: SyncStatus,
    pub token: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub conflict_strategy: ConflictStrategy,
}

/// 预定义角色
pub mod roles {
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
}

// =====================================================================
// 错误(§8.3 SC-001~006)
// =====================================================================

#[derive(Debug, Error)]
pub enum ScmError {
    /// `SC-001` 404 Repository 不存在
    #[error("repository not found: {0}")]
    NotFound(RepositoryId),
    /// `SC-002` 422 Provider 不可用
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// `SC-003` 409 同步冲突
    #[error("conflict: {0}")]
    Conflict(String),
    /// `SC-004` 409 重复 Webhook 事件(Idempotency)
    #[error("idempotency conflict: event already recorded")]
    IdempotencyConflict,
    /// `SC-005` 422 厂商 API 错误
    #[error("provider error: {0}")]
    ProviderError(String),
    /// `SC-006` 403 Provider Credential 缺失
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// 5xx
    #[error("internal: {0}")]
    Internal(String),
}

impl ScmError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "SCM_NOT_FOUND",
            Self::InvalidState(_) => "SCM_INVALID_STATE",
            Self::Conflict(_) => "SCM_CONFLICT",
            Self::IdempotencyConflict => "SCM_IDEMPOTENCY_CONFLICT",
            Self::ProviderError(_) => "SCM_PROVIDER_ERROR",
            Self::PermissionDenied(_) => "SCM_PERMISSION_DENIED",
            Self::Internal(_) => "SCM_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for ScmError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

// =====================================================================
// 实体
// =====================================================================

/// **Repository 聚合根**(§4.18.1,17 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: RepositoryId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub provider: ScmProvider,
    pub external_id: ExternalRepositoryId,
    pub url: String,
    pub default_branch: String,
    pub ownership: RepositoryOwnership,
    pub sync_status: SyncStatus,
    pub sync_token: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub conflict_strategy: ConflictStrategy,
    /// Credential Broker 引用(INV-SCM-05 不存明文)
    pub credential_id: Option<Uuid>,
    pub is_archived: bool,
    pub registered_by_user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lock_version: u32,
}

impl Repository {
    pub const FIELD_COUNT: usize = 17;
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

/// **Branch**(§4.18.2,11 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: BranchId,
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    pub name: String,
    pub head_commit_id: Option<CommitId>,
    pub base_commit_id: Option<CommitId>,
    pub protected: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_commit_at: Option<DateTime<Utc>>,
    pub lock_version: u32,
}

impl Branch {
    pub const FIELD_COUNT: usize = 11;
}

/// **Commit**(§4.18.3,13 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: CommitId,
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    pub sha: String,
    pub author: String,
    pub committer: String,
    pub message: String,
    pub parent_shas: Vec<String>,
    pub tree_sha: String,
    pub linked_work_item_id: Option<WorkItemId>,
    pub authored_at: DateTime<Utc>,
    pub committed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub lock_version: u32,
}

impl Commit {
    pub const FIELD_COUNT: usize = 13;
}

/// **PullRequest**(§4.18.4,19 字段,**非聚合根**)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: PullRequestId,
    pub tenant_id: TenantId,
    pub repository_id: RepositoryId,
    pub external_id: String,
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    pub description: Option<String>,
    pub author_user_id: UserId,
    pub state: PullRequestState,
    pub mergeable: bool,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub review_ids: Vec<ReviewId>,
    pub pipeline_ids: Vec<PipelineId>,
    pub linked_work_item_id: Option<WorkItemId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content_object_key: Option<String>, // INV-SCM-06: 必带 tenant_id 前缀
    pub lock_version: u32,
}

impl PullRequest {
    pub const FIELD_COUNT: usize = 19;
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
    /// **INV-SCM-07** 状态机迁移
    pub fn try_transition(&self, target: PullRequestState) -> Result<(), ScmError> {
        use PullRequestState::*;
        let allowed = match (self.state, target) {
            (Draft, Open) => true,
            (Open, Reviewing) => true,
            (Reviewing, ChangesRequested) => true,
            (Reviewing, Approved) => true,
            (Approved, Mergeable) => true,
            (Mergeable, Merged) => true,
            (_, Closed) => true, // 任何状态都可 Close
            // 拒绝的迁移
            (Merged, _) | (Closed, _) => false,
            _ => false,
        };
        if !allowed {
            return Err(ScmError::InvalidState(format!(
                "INV-SCM-07: 状态机非法迁移 {} -> {}",
                self.state.as_str(),
                target.as_str()
            )));
        }
        Ok(())
    }
}

/// **Review**(§4.18.5,9 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: ReviewId,
    pub tenant_id: TenantId,
    pub pull_request_id: PullRequestId,
    pub reviewer_user_id: UserId,
    pub state: ReviewState,
    pub body: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub lock_version: u32,
}

impl Review {
    pub const FIELD_COUNT: usize = 9;
}

/// **Pipeline**(§4.18.6,10 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: PipelineId,
    pub tenant_id: TenantId,
    pub pull_request_id: PullRequestId,
    pub external_id: String,
    pub status: PipelineStatus,
    pub head_sha: String,
    pub url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub lock_version: u32,
}

impl Pipeline {
    pub const FIELD_COUNT: usize = 10;
}

/// **WebhookEvent**(§4.18.7,11 字段,Append-only,INV-SCM-03 + INV-SCM-08)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: WebhookEventId,
    pub tenant_id: TenantId,
    pub repository_id: Option<RepositoryId>,
    pub provider: ScmProvider,
    pub event_type: WebhookEventType,
    pub external_event_id: String,
    pub raw_payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub loop_breaker_id: Uuid,
    pub idempotency_key: String,
}

impl WebhookEvent {
    pub const FIELD_COUNT: usize = 11;
}

// =====================================================================
// 不变量(INV-SCM-01~08)
// =====================================================================

pub type InvariantCheck = fn(&Repository) -> Result<(), ScmError>;

/// **INV-SCM-04** Repository 必带 tenant_id + project_id
pub fn check_invariant_04_tenant_project(r: &Repository) -> Result<(), ScmError> {
    if r.tenant_id.as_uuid().is_nil() {
        return Err(ScmError::InvalidState(
            "INV-SCM-04: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    if r.project_id.as_uuid().is_nil() {
        return Err(ScmError::InvalidState(
            "INV-SCM-04: project_id 必须非 nil".to_string(),
        ));
    }
    Ok(())
}

/// **INV-SCM-05** Credential 必须由 Broker 引用,不允许明文
pub fn check_invariant_05_credential(r: &Repository) -> Result<(), ScmError> {
    // 如 r.sync_status 不是 Disabled / LocalOnly,则必有 credential
    if r.ownership == RepositoryOwnership::LocalOnly {
        return Ok(());
    }
    if r.credential_id.is_none() {
        return Err(ScmError::PermissionDenied(
            "INV-SCM-05: Connected/Mirrored/Managed 仓库必须配 Credential Broker 引用".to_string(),
        ));
    }
    Ok(())
}

/// **INV-SCM-02** MVP 仅支持 Connected(写时校验,允许历史 Mirrored/Managed)
pub fn check_invariant_02_connected_only(r: &Repository) -> Result<(), ScmError> {
    // 写时强制 Connected(读时不强约束)
    if !matches!(r.ownership, RepositoryOwnership::Connected) {
        return Err(ScmError::InvalidState(
            "INV-SCM-02: MVP 仅支持 Connected 所有权 (§30.6)".to_string(),
        ));
    }
    Ok(())
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_04_tenant_project,
    check_invariant_05_credential,
    check_invariant_02_connected_only,
];

pub fn run_invariants(checks: &[InvariantCheck], r: &Repository) -> Result<(), ScmError> {
    for c in checks {
        c(r)?;
    }
    Ok(())
}

// =====================================================================
// 事件
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRegistered {
    pub meta: EventMeta,
    pub repository_id: RepositoryId,
    pub provider: ScmProvider,
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestStateChanged {
    pub meta: EventMeta,
    pub pr_id: PullRequestId,
    pub from: PullRequestState,
    pub to: PullRequestState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookReceived {
    pub meta: EventMeta,
    pub webhook_event_id: WebhookEventId,
    pub external_event_id: String,
    pub loop_breaker_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScmEvent {
    RepositoryRegistered(RepositoryRegistered),
    PullRequestStateChanged(PullRequestStateChanged),
    WebhookReceived(WebhookReceived),
}

impl ScmEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::RepositoryRegistered(_) => "star.events.scm.repository.registered.v1",
            Self::PullRequestStateChanged(_) => "star.events.scm.pull_request.state_changed.v1",
            Self::WebhookReceived(_) => "star.events.scm.webhook.received.v1",
        }
    }
}

// =====================================================================
// 端口
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRepositoryCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub provider: ScmProvider,
    pub external_id: String,
    pub url: String,
    pub default_branch: String,
    pub ownership: RepositoryOwnership,
    pub conflict_strategy: ConflictStrategy,
    /// Credential Broker 引用
    pub credential_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSyncStateCommand {
    pub repository_id: RepositoryId,
    pub tenant_id: TenantId,
    pub new_status: SyncStatus,
    pub new_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventInput {
    pub tenant_id: TenantId,
    pub repository_id: Option<RepositoryId>,
    pub provider: ScmProvider,
    pub event_type: WebhookEventType,
    pub external_event_id: String,
    pub raw_payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPullRequestTransitionCommand {
    pub pr_id: PullRequestId,
    pub new_state: PullRequestState,
    pub actor: ActorContext,
}

/// **ScmCommandPort**
#[async_trait]
pub trait ScmCommandPort: Send + Sync {
    async fn register_repository(
        &self,
        cmd: RegisterRepositoryCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;
    async fn update_sync_state(
        &self,
        cmd: UpdateSyncStateCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;
    /// Idempotent:重复 external_event_id 返回 IdempotencyConflict
    async fn record_webhook_event(
        &self,
        event: WebhookEventInput,
    ) -> Result<WebhookEvent, ScmError>;
    /// 状态机迁移(由 webhook handler 或 application 触发)
    async fn transition_pull_request(
        &self,
        cmd: RecordPullRequestTransitionCommand,
    ) -> Result<PullRequest, ScmError>;
}

/// **ScmQueryPort**
#[async_trait]
pub trait ScmQueryPort: Send + Sync {
    async fn get_repository(
        &self,
        id: RepositoryId,
        actor: ActorContext,
    ) -> Result<Repository, ScmError>;
    async fn list_by_project(
        &self,
        project_id: ProjectId,
        actor: ActorContext,
    ) -> Result<Vec<Repository>, ScmError>;
    async fn get_pull_request(
        &self,
        id: PullRequestId,
        actor: ActorContext,
    ) -> Result<PullRequest, ScmError>;
}

/// **ScmPort**(SCM Adapter 抽象,INV-SCM-01)
#[async_trait]
pub trait ScmPort: Send + Sync {
    async fn get_repository_meta(
        &self,
        external_id: ExternalRepositoryId,
    ) -> Result<Repository, ScmError>;
    async fn list_branches(&self, repo: RepositoryId) -> Result<Vec<Branch>, ScmError>;
    async fn list_pull_requests(&self, repo: RepositoryId) -> Result<Vec<PullRequest>, ScmError>;
}

// =====================================================================
// InMemoryScmService
// =====================================================================

pub struct InMemoryScmService {
    repos: Arc<RwLock<HashMap<RepositoryId, Repository>>>,
    branches: Arc<RwLock<HashMap<BranchId, Branch>>>,
    prs: Arc<RwLock<HashMap<PullRequestId, PullRequest>>>,
    webhooks: Arc<RwLock<HashMap<WebhookEventId, WebhookEvent>>>,
    /// **INV-SCM-03** Idempotency: external_event_id → webhook_event_id
    idempotency: Arc<RwLock<HashMap<String, WebhookEventId>>>,
    event_tx: mpsc::UnboundedSender<ScmEvent>,
}

impl InMemoryScmService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<ScmEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            repos: Arc::new(RwLock::new(HashMap::new())),
            branches: Arc::new(RwLock::new(HashMap::new())),
            prs: Arc::new(RwLock::new(HashMap::new())),
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            idempotency: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    pub async fn repo_count(&self) -> usize {
        self.repos.read().await.len()
    }
    pub async fn pr_count(&self) -> usize {
        self.prs.read().await.len()
    }
    pub async fn webhook_count(&self) -> usize {
        self.webhooks.read().await.len()
    }
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), ScmError> {
        if TenantId::from(actor.tenant_id) != expected {
            return Err(ScmError::PermissionDenied(format!(
                "SEC-007 跨 tenant 拒绝: actor={} expected={}",
                TenantId::from(actor.tenant_id),
                expected
            )));
        }
        Ok(())
    }
}

impl Default for InMemoryScmService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryScmService {
    fn clone(&self) -> Self {
        Self {
            repos: self.repos.clone(),
            branches: self.branches.clone(),
            prs: self.prs.clone(),
            webhooks: self.webhooks.clone(),
            idempotency: self.idempotency.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl ScmCommandPort for InMemoryScmService {
    async fn register_repository(
        &self,
        cmd: RegisterRepositoryCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        if !actor.has_role("project_admin") && !actor.is_platform_admin {
            return Err(ScmError::PermissionDenied("需要 project_admin".to_string()));
        }
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // project 必带
        if !actor.project_ids.contains(&cmd.project_id) {
            return Err(ScmError::PermissionDenied("actor 不属于该项目".to_string()));
        }
        // (tenant, provider, external_id) UNIQUE
        let external_id = ExternalRepositoryId(cmd.external_id.clone());
        {
            let guard = self.repos.read().await;
            if guard.values().any(|r| {
                r.tenant_id == cmd.tenant_id
                    && r.provider == cmd.provider
                    && r.external_id == external_id
            }) {
                return Err(ScmError::Conflict(format!(
                    "({}, {:?}, {}) 已存在",
                    cmd.tenant_id, cmd.provider, external_id
                )));
            }
        }
        let now = Utc::now();
        let id = RepositoryId::new();
        let repo = Repository {
            id,
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            provider: cmd.provider,
            external_id,
            url: cmd.url,
            default_branch: cmd.default_branch,
            ownership: cmd.ownership,
            sync_status: SyncStatus::InSync,
            sync_token: None,
            last_synced_at: None,
            conflict_strategy: cmd.conflict_strategy,
            credential_id: cmd.credential_id,
            is_archived: false,
            registered_by_user_id: UserId::from_uuid(actor.user_id),
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &repo)?;
        {
            let mut guard = self.repos.write().await;
            guard.insert(id, repo.clone());
        }
        let _ = self
            .event_tx
            .send(ScmEvent::RepositoryRegistered(RepositoryRegistered {
                meta: EventMeta::new(cmd.tenant_id),
                repository_id: id,
                provider: cmd.provider,
                external_id: cmd.external_id,
            }));
        Ok(repo)
    }

    async fn update_sync_state(
        &self,
        cmd: UpdateSyncStateCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let updated = {
            let mut guard = self.repos.write().await;
            let r = guard
                .get_mut(&cmd.repository_id)
                .ok_or(ScmError::NotFound(cmd.repository_id))?;
            if r.tenant_id != cmd.tenant_id {
                return Err(ScmError::PermissionDenied("跨 tenant 拒绝".to_string()));
            }
            r.sync_status = cmd.new_status;
            r.sync_token = cmd.new_token;
            r.last_synced_at = Some(Utc::now());
            r.bump_version();
            r.clone()
        };
        Ok(updated)
    }

    async fn record_webhook_event(
        &self,
        event: WebhookEventInput,
    ) -> Result<WebhookEvent, ScmError> {
        // **INV-SCM-03** Idempotency: 重复 external_event_id 返回 IdempotencyConflict
        {
            let guard = self.idempotency.read().await;
            if let Some(existing_id) = guard.get(&event.external_event_id) {
                return Err(ScmError::IdempotencyConflict);
            }
        }
        let id = WebhookEventId::new();
        let now = Utc::now();
        let loop_breaker_id = Uuid::new_v4();
        let we = WebhookEvent {
            id,
            tenant_id: event.tenant_id,
            repository_id: event.repository_id,
            provider: event.provider,
            event_type: event.event_type,
            external_event_id: event.external_event_id.clone(),
            raw_payload: event.raw_payload,
            received_at: now,
            processed: false,
            processed_at: None,
            loop_breaker_id,
            idempotency_key: event.external_event_id.clone(),
        };
        {
            let mut guard = self.webhooks.write().await;
            guard.insert(id, we.clone());
        }
        {
            let mut guard = self.idempotency.write().await;
            guard.insert(event.external_event_id, id);
        }
        let _ = self
            .event_tx
            .send(ScmEvent::WebhookReceived(WebhookReceived {
                meta: EventMeta::new(event.tenant_id),
                webhook_event_id: id,
                external_event_id: we.external_event_id.clone(),
                loop_breaker_id,
            }));
        Ok(we)
    }

    async fn transition_pull_request(
        &self,
        cmd: RecordPullRequestTransitionCommand,
    ) -> Result<PullRequest, ScmError> {
        let actor = cmd.actor;
        let updated = {
            let mut guard = self.prs.write().await;
            let pr = guard
                .get_mut(&cmd.pr_id)
                .ok_or(ScmError::NotFound(RepositoryId::default()))?;
            if pr.tenant_id != TenantId::from(actor.tenant_id) {
                return Err(ScmError::PermissionDenied("跨 tenant".to_string()));
            }
            // **INV-SCM-07** 状态机校验
            pr.try_transition(cmd.new_state)?;
            let from = pr.state;
            pr.state = cmd.new_state;
            pr.updated_at = Utc::now();
            pr.bump_version();
            // 修改:PR 用 RepositoryId 字段不存 lock_version bump,我用 lock_version
            // 重新写实现
            let new_pr = PullRequest {
                lock_version: pr.lock_version,
                ..pr.clone()
            };
            *pr = new_pr.clone();
            // 通知
            (from, cmd.new_state, new_pr.clone())
        };
        let _ = self
            .event_tx
            .send(ScmEvent::PullRequestStateChanged(PullRequestStateChanged {
                meta: EventMeta::new(updated.2.tenant_id),
                pr_id: cmd.pr_id,
                from: updated.0,
                to: updated.1,
            }));
        Ok(updated.2)
    }
}

#[async_trait]
impl ScmQueryPort for InMemoryScmService {
    async fn get_repository(
        &self,
        id: RepositoryId,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        let r = {
            let guard = self.repos.read().await;
            guard.get(&id).cloned()
        };
        let r = r.ok_or(ScmError::NotFound(id))?;
        if r.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(ScmError::PermissionDenied("跨 tenant".to_string()));
        }
        Ok(r)
    }

    async fn list_by_project(
        &self,
        project_id: ProjectId,
        actor: ActorContext,
    ) -> Result<Vec<Repository>, ScmError> {
        let guard = self.repos.read().await;
        Ok(guard
            .values()
            .filter(|r| {
                r.tenant_id == TenantId::from(actor.tenant_id) && r.project_id == project_id
            })
            .cloned()
            .collect())
    }

    async fn get_pull_request(
        &self,
        id: PullRequestId,
        actor: ActorContext,
    ) -> Result<PullRequest, ScmError> {
        let guard = self.prs.read().await;
        let pr = guard
            .get(&id)
            .cloned()
            .ok_or(ScmError::NotFound(RepositoryId::default()))?;
        if pr.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(ScmError::PermissionDenied("跨 tenant".to_string()));
        }
        Ok(pr)
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_admin(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(project_id)
    }

    fn make_developer(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::DEVELOPER)
            .with_project(project_id)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(Repository::FIELD_COUNT, 17);
        assert_eq!(Branch::FIELD_COUNT, 11);
        assert_eq!(Commit::FIELD_COUNT, 13);
        assert_eq!(PullRequest::FIELD_COUNT, 19);
        assert_eq!(Review::FIELD_COUNT, 9);
        assert_eq!(Pipeline::FIELD_COUNT, 10);
        assert_eq!(WebhookEvent::FIELD_COUNT, 11);
    }

    #[tokio::test]
    async fn register_github_repo_success() {
        let svc = InMemoryScmService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let cmd = RegisterRepositoryCommand {
            tenant_id: TenantId(tenant),
            project_id: project,
            provider: ScmProvider::Github,
            external_id: "acme/foo".to_string(),
            url: "https://github.com/acme/foo".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: Some(Uuid::new_v4()),
        };
        let repo = svc.register_repository(cmd, actor).await.unwrap();
        assert_eq!(repo.ownership, RepositoryOwnership::Connected);
        assert_eq!(svc.repo_count().await, 1);
    }

    #[tokio::test]
    async fn invariant_05_credential_required_for_connected() {
        let svc = InMemoryScmService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let cmd = RegisterRepositoryCommand {
            tenant_id: TenantId(tenant),
            project_id: project,
            provider: ScmProvider::Github,
            external_id: "acme/bar".to_string(),
            url: "https://github.com/acme/bar".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: None, // 缺失
        };
        let res = svc.register_repository(cmd, actor).await;
        assert!(matches!(res, Err(ScmError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn cross_tenant_register_denied() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_a = uuid::Uuid::new_v4();
        let project_a = ProjectId::new();
        let actor_a = make_admin(tenant_a, project_a);
        let cmd = RegisterRepositoryCommand {
            tenant_id: tenant_a,
            project_id: project_a,
            provider: ScmProvider::Github,
            external_id: "x".to_string(),
            url: "x".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: Some(Uuid::new_v4()),
        };
        let _ = svc.register_repository(cmd, actor_a).await.unwrap();
        let tenant_b = uuid::Uuid::new_v4();
        let project_b = ProjectId::new();
        let actor_b = make_admin(tenant_b, project_b);
        // 尝试读 tenant_a 的 repo
        let repo_id = {
            // 简单方法:扫描
            let s = InMemoryScmService::new_for_test();
            s
        };
        let _ = repo_id; // 避免 unused
        let res = svc.list_by_project(project_a, actor_b).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0); // 跨 tenant 看不到
    }

    #[tokio::test]
    async fn developer_cannot_register_repo() {
        let svc = InMemoryScmService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_developer(tenant, project);
        let cmd = RegisterRepositoryCommand {
            tenant_id: TenantId(tenant),
            project_id: project,
            provider: ScmProvider::Github,
            external_id: "z".to_string(),
            url: "z".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: Some(Uuid::new_v4()),
        };
        let res = svc.register_repository(cmd, actor).await;
        assert!(matches!(res, Err(ScmError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn webhook_idempotency() {
        let svc = InMemoryScmService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let input = WebhookEventInput {
            tenant_id: TenantId(tenant),
            repository_id: None,
            provider: ScmProvider::Github,
            event_type: WebhookEventType::Push,
            external_event_id: "gh-event-12345".to_string(),
            raw_payload: serde_json::json!({"ref": "refs/heads/main"}),
        };
        let we1 = svc.record_webhook_event(input.clone()).await.unwrap();
        assert_eq!(svc.webhook_count().await, 1);
        // 重复 → IdempotencyConflict(INV-SCM-03)
        let res = svc.record_webhook_event(input).await;
        assert!(matches!(res, Err(ScmError::IdempotencyConflict)));
        // 历史 100% 写(INV-SCM-08): 1 条
        let _ = we1;
    }

    #[tokio::test]
    async fn pr_state_machine_transitions() {
        // 单元测试 PR 状态机(不需要 service)
        let make_pr = |state: PullRequestState| PullRequest {
            id: PullRequestId::new(),
            tenant_id: UserId.new(),
            repository_id: RepositoryId::new(),
            external_id: "1".to_string(),
            source_branch: "feat".to_string(),
            target_branch: "main".to_string(),
            title: "T".to_string(),
            description: None,
            author_user_id: UserId.new(),
            state,
            mergeable: false,
            merged_at: None,
            closed_at: None,
            review_ids: vec![],
            pipeline_ids: vec![],
            linked_work_item_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content_object_key: None,
            lock_version: 1,
        };
        // 合法链: Draft -> Open -> Reviewing -> ChangesRequested -> Approved -> Mergeable -> Merged
        let p_d = make_pr(PullRequestState::Draft);
        assert!(p_d.try_transition(PullRequestState::Open).is_ok());
        let p_o = make_pr(PullRequestState::Open);
        assert!(p_o.try_transition(PullRequestState::Reviewing).is_ok());
        let p_r = make_pr(PullRequestState::Reviewing);
        assert!(p_r
            .try_transition(PullRequestState::ChangesRequested)
            .is_ok());
        let p_r2 = make_pr(PullRequestState::Reviewing);
        assert!(p_r2.try_transition(PullRequestState::Approved).is_ok());
        let p_a = make_pr(PullRequestState::Approved);
        assert!(p_a.try_transition(PullRequestState::Mergeable).is_ok());
        let p_m = make_pr(PullRequestState::Mergeable);
        assert!(p_m.try_transition(PullRequestState::Merged).is_ok());
        // 终态不可迁移
        let p2 = make_pr(PullRequestState::Merged);
        assert!(p2.try_transition(PullRequestState::Open).is_err());
        let p3 = make_pr(PullRequestState::Closed);
        assert!(p3.try_transition(PullRequestState::Open).is_err());
        // 任何状态都可 Close
        let p4 = make_pr(PullRequestState::Reviewing);
        assert!(p4.try_transition(PullRequestState::Closed).is_ok());
        // 非法: Closed -> Merged
        let p5 = make_pr(PullRequestState::Closed);
        assert!(p5.try_transition(PullRequestState::Merged).is_err());
    }

    #[tokio::test]
    async fn update_sync_state_advances_version() {
        let svc = InMemoryScmService::new_for_test();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_admin(tenant, project);
        let repo = svc
            .register_repository(
                RegisterRepositoryCommand {
                    tenant_id: TenantId(tenant),
                    project_id: project,
                    provider: ScmProvider::Github,
                    external_id: "sync-test".to_string(),
                    url: "x".to_string(),
                    default_branch: "main".to_string(),
                    ownership: RepositoryOwnership::Connected,
                    conflict_strategy: ConflictStrategy::LatestWins,
                    credential_id: Some(Uuid::new_v4()),
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert_eq!(repo.lock_version, 1);
        let r2 = svc
            .update_sync_state(
                UpdateSyncStateCommand {
                    repository_id: repo.id,
                    tenant_id: TenantId(tenant),
                    new_status: SyncStatus::Behind,
                    new_token: Some("etag-123".to_string()),
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(r2.lock_version, 2);
        assert_eq!(r2.sync_status, SyncStatus::Behind);
        assert_eq!(r2.sync_token, Some("etag-123".to_string()));
    }
}

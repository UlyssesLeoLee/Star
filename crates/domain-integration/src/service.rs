//! InMemoryIntegrationService: Phase 2 提供的内存实现
//!
//! 来源: `docs/specs/domain-integration-spec.md` §5(实施策略)
//!
//! **目标**: 为 `IntegrationCommandPort` + `IntegrationQueryPort` + `IntegrationRepository`
//! 提供 1 个真实可工作的实现,用于本地集成测试与 P0 演示,
//! 不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**: `crates/infrastructure` 提供 SQLx / NATS Adapter 取代本实现。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{Integration, MappingConfig, SyncDirection, SyncState};
use crate::error::IntegrationError;
use crate::event::{EventMeta, IntegrationEvent};
use crate::invariants::{check_invariant_03_tenant_required, check_register_invariants};
use crate::port::{
    ConfigureIntegrationCommand, CreateIntegrationCommand, GetHistoryQuery, HandleWebhookCommand,
    IntegrationCommandPort, IntegrationQueryPort, IntegrationRepository, ListByProjectQuery,
    PauseIntegrationCommand, ResumeIntegrationCommand, TriggerSyncCommand, UpdateIntegrationCommand,
};
use crate::value_object::{
    ConflictStrategy, ExternalEntityId, ExternalSystemName, IntegrationId, IntegrationRelationType,
    IntegrationSource, IntegrationState, ProjectId, SyncOutcome, SyncStateId, TenantId, UserId,
};

// =====================================================================
// InMemoryIntegrationService
// =====================================================================

/// **InMemory Integration 命令/查询服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryIntegrationService {
    /// Integration 存储
    integrations: Arc<RwLock<HashMap<IntegrationId, Integration>>>,
    /// SyncState 存储(按 integration_id 分组,Append-only)
    sync_states: Arc<RwLock<HashMap<IntegrationId, Vec<SyncState>>>>,
    /// Webhook 幂等(避免重复)
    webhook_events: Arc<RwLock<HashMap<String, SyncStateId>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<IntegrationEvent>,
}

impl InMemoryIntegrationService {
    /// 创建新的内存服务(返回服务和事件接收端)
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<IntegrationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            integrations: Arc::new(RwLock::new(HashMap::new())),
            sync_states: Arc::new(RwLock::new(HashMap::new())),
            webhook_events: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Integration 数量
    pub async fn count_integrations(&self) -> usize {
        self.integrations.read().expect("integrations lock").len()
    }

    /// 当前 SyncState 总数
    pub async fn count_sync_states(&self) -> usize {
        self.sync_states
            .read()
            .expect("sync_states lock")
            .values()
            .map(|v| v.len())
            .sum()
    }

    /// 注入 Integration(测试辅助)
    pub async fn seed_integration(&self, integration: Integration) {
        self.integrations
            .write()
            .expect("integrations lock")
            .insert(integration.id, integration);
    }

    /// 校验 actor 与命令的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), IntegrationError> {
        if actor.tenant_id != expected {
            return Err(IntegrationError::PermissionDenied);
        }
        Ok(())
    }

    /// 按 (tenant_id, source, external_system_name, external_id) UNIQUE 查找
    fn find_existing(
        &self,
        tenant_id: TenantId,
        source: IntegrationSource,
        external_system_name: &ExternalSystemName,
        external_id: &ExternalEntityId,
        relation_type: IntegrationRelationType,
    ) -> Option<IntegrationId> {
        let store = self.integrations.read().expect("integrations lock");
        store
            .values()
            .find(|i| {
                i.tenant_id == tenant_id
                    && i.source == source
                    && i.external_system_name == *external_system_name
                    && i.external_id == *external_id
                    && i.relation_type == relation_type
            })
            .map(|i| i.id)
    }

    /// 更新 Integration 字段(inherent 方法,避免与 trait 同名冲突)
    pub async fn update_integration(
        &self,
        cmd: UpdateIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.integrations.write().expect("integrations lock");
        let integration = store
            .get_mut(&cmd.integration_id)
            .ok_or(IntegrationError::NotFound(cmd.integration_id))?;
        check_invariant_03_tenant_required(integration, cmd.tenant_id)?;

        if let Some(cs) = cmd.conflict_strategy {
            integration.conflict_strategy = cs;
        }
        if let Some(token) = cmd.sync_token {
            integration.sync_token = Some(token);
        }
        if let Some(url) = cmd.external_url {
            integration.external_url = url;
        }
        if let Some(cid) = cmd.credential_id {
            integration.credential_id = Some(cid);
        }
        integration.bump_version();

        // 不变量重检(INV-I-02/05/06)
        crate::invariants::check_invariant_02_bidirectional_loop_guard(integration)?;
        crate::invariants::check_invariant_05_required_fields(integration)?;
        crate::invariants::check_invariant_06_link_no_reverse_sync(integration)?;
        crate::invariants::check_invariant_04_no_plaintext_credential(integration)?;

        let updated = integration.clone();
        drop(store);

        Ok(updated)
    }
}

impl Default for InMemoryIntegrationService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryIntegrationService {
    fn clone(&self) -> Self {
        Self {
            integrations: self.integrations.clone(),
            sync_states: self.sync_states.clone(),
            webhook_events: self.webhook_events.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// IntegrationCommandPort 实现(7 方法)
// =====================================================================

#[async_trait]
impl IntegrationCommandPort for InMemoryIntegrationService {
    async fn create_integration(
        &self,
        cmd: CreateIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // UNIQUE 约束:(tenant_id, source, system, external_id, relation_type) — 同一 external_id 可被 4 类不同关系分别引用
        if let Some(existing_id) = self.find_existing(
            cmd.tenant_id,
            cmd.source,
            &cmd.external_system_name,
            &cmd.external_id,
            cmd.relation_type,
        ) {
            return Err(IntegrationError::Conflict(format!(
                "Integration (tenant={}, source={}, system={}, external_id={}) 已存在(integration_id={})",
                cmd.tenant_id,
                cmd.source.as_str(),
                cmd.external_system_name.as_str(),
                cmd.external_id.as_str(),
                existing_id
            )));
        }

        // INV-I-02:Bidirectional 必须有 initial_sync_token(Loop 防护前置)
        if cmd.relation_type.requires_sync_token() && cmd.initial_sync_token.is_none() {
            return Err(IntegrationError::LoopGuardMissing(format!(
                "INV-I-02: 关系类型 {} 需提供 initial_sync_token",
                cmd.relation_type.as_str()
            )));
        }

        let now = chrono::Utc::now();
        let integration = Integration {
            id: IntegrationId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            source: cmd.source,
            relation_type: cmd.relation_type,
            external_system_name: cmd.external_system_name,
            external_id: cmd.external_id,
            external_url: cmd.external_url,
            mapping_config: MappingConfig::empty(),
            conflict_strategy: cmd.conflict_strategy,
            state: IntegrationState::Initializing,
            sync_token: cmd.initial_sync_token,
            last_synced_at: None,
            last_error: None,
            retry_count: 0,
            credential_id: cmd.credential_id,
            enabled: true,
            created_by_user_id: actor.user_id,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };

        // 不变量检查(INV-I-01/02/03/04/05/06)
        check_register_invariants(&integration, cmd.tenant_id)?;

        // 持久化
        self.insert_integration(&integration).await?;

        // 事件总线广播
        let event = IntegrationEvent::IntegrationCreated(crate::event::IntegrationCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            integration_id: integration.id,
            project_id: integration.project_id,
            source: integration.source,
            relation_type: integration.relation_type,
            external_system_name: integration.external_system_name.clone(),
            external_id: integration.external_id.clone(),
        });
        let _ = self.event_tx.send(event);

        Ok(integration)
    }

    async fn configure_integration(
        &self,
        cmd: ConfigureIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.integrations.write().expect("integrations lock");
        let integration = store
            .get_mut(&cmd.integration_id)
            .ok_or(IntegrationError::NotFound(cmd.integration_id))?;
        check_invariant_03_tenant_required(integration, cmd.tenant_id)?;

        // 配置 Idempotency Key(对 Bidirectional 推荐;非 Bidirectional 也允许作为通用保护)
        if let Some(key) = cmd.idempotency_key {
            // 把 key 写入 sync_token(若为 Bidirectional 且 sync_token 尚未设置)
            if integration.relation_type.requires_loop_guard() && integration.sync_token.is_none() {
                integration.sync_token = Some(key);
            }
        }
        if let Some(json) = cmd.mapping_json {
            integration.mapping_config = MappingConfig::from_json(json);
        }
        integration.bump_version();

        // 不变量重检
        crate::invariants::check_invariant_02_bidirectional_loop_guard(integration)?;

        let updated = integration.clone();
        drop(store);

        Ok(updated)
    }

    async fn pause_integration(
        &self,
        cmd: PauseIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.integrations.write().expect("integrations lock");
        let integration = store
            .get_mut(&cmd.integration_id)
            .ok_or(IntegrationError::NotFound(cmd.integration_id))?;
        check_invariant_03_tenant_required(integration, cmd.tenant_id)?;

        let from = integration.state;
        integration.transition_state(IntegrationState::Paused);
        let updated = integration.clone();
        drop(store);

        // 事件总线
        let event = IntegrationEvent::IntegrationStateChanged(
            crate::event::IntegrationStateChanged {
                meta: EventMeta {
                    actor_user_id: Some(actor.user_id.into_uuid()),
                    ..EventMeta::new(cmd.tenant_id)
                },
                integration_id: updated.id,
                from_state: from.as_str().to_string(),
                to_state: updated.state,
            },
        );
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn resume_integration(
        &self,
        cmd: ResumeIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.integrations.write().expect("integrations lock");
        let integration = store
            .get_mut(&cmd.integration_id)
            .ok_or(IntegrationError::NotFound(cmd.integration_id))?;
        check_invariant_03_tenant_required(integration, cmd.tenant_id)?;

        let from = integration.state;
        integration.transition_state(IntegrationState::Active);
        let updated = integration.clone();
        drop(store);

        let event = IntegrationEvent::IntegrationStateChanged(
            crate::event::IntegrationStateChanged {
                meta: EventMeta {
                    actor_user_id: Some(actor.user_id.into_uuid()),
                    ..EventMeta::new(cmd.tenant_id)
                },
                integration_id: updated.id,
                from_state: from.as_str().to_string(),
                to_state: updated.state,
            },
        );
        let _ = self.event_tx.send(event);

        Ok(updated)
    }

    async fn trigger_sync(
        &self,
        cmd: TriggerSyncCommand,
        actor: ActorContext,
    ) -> Result<SyncState, IntegrationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let integration = {
            let store = self.integrations.read().expect("integrations lock");
            store
                .get(&cmd.integration_id)
                .cloned()
                .ok_or(IntegrationError::NotFound(cmd.integration_id))?
        };
        check_invariant_03_tenant_required(&integration, cmd.tenant_id)?;

        // Link 关系不能反向同步(INV-I-06)
        if integration.is_link() {
            return Err(IntegrationError::InvalidState(format!(
                "INV-I-06: Link 关系不触发同步(只读链接,integration_id={})",
                integration.id
            )));
        }

        // Disabled 状态拒绝
        if matches!(integration.state, IntegrationState::Disabled) {
            return Err(IntegrationError::InvalidState(format!(
                "Integration 已被禁用(integration_id={})",
                integration.id
            )));
        }

        // 创建 SyncState(Pending → 由 Worker 后续完成)
        let now = chrono::Utc::now();
        let sync_state = SyncState {
            id: SyncStateId::new(),
            integration_id: integration.id,
            tenant_id: integration.tenant_id,
            sync_token: integration.sync_token.clone().unwrap_or_default(),
            synced_at: now,
            outcome: SyncOutcome::Success, // Worker 会更新
            error: None,
            direction: match integration.relation_type {
                IntegrationRelationType::Mirror => SyncDirection::Inbound,
                IntegrationRelationType::Bidirectional => SyncDirection::Bidirectional,
                IntegrationRelationType::PlatformOwned => SyncDirection::Outbound,
                IntegrationRelationType::Link => SyncDirection::Inbound, // 不会到这里
            },
            processed_count: 0,
            skipped_count: 0,
            conflict_count: 0,
            triggered_by_user_id: Some(actor.user_id),
            created_at: now,
        };

        self.insert_sync_state(&sync_state).await?;

        // 事件总线:SyncTriggered
        let event = IntegrationEvent::SyncTriggered(crate::event::SyncTriggered {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            integration_id: integration.id,
            relation_type: integration.relation_type,
            manual: true,
        });
        let _ = self.event_tx.send(event);

        Ok(sync_state)
    }

    async fn handle_webhook(
        &self,
        cmd: HandleWebhookCommand,
    ) -> Result<SyncState, IntegrationError> {
        // Webhook 幂等(INV-I-02 Loop 防护 + Webhook 自身幂等)
        let webhook_key = format!("{}:{}", cmd.integration_id, cmd.external_event_id);
        {
            let seen = self.webhook_events.read().expect("webhook lock");
            if let Some(prev_id) = seen.get(&webhook_key) {
                return Err(IntegrationError::Conflict(format!(
                    "Webhook 重复事件(integration_id={}, external_event_id={}, previous_sync_state={})",
                    cmd.integration_id, cmd.external_event_id, prev_id
                )));
            }
        }

        let integration = {
            let store = self.integrations.read().expect("integrations lock");
            store
                .get(&cmd.integration_id)
                .cloned()
                .ok_or(IntegrationError::NotFound(cmd.integration_id))?
        };
        check_invariant_03_tenant_required(&integration, cmd.tenant_id)?;

        // Link 关系不会收到 Webhook(INV-I-06)
        if integration.is_link() {
            return Err(IntegrationError::InvalidState(format!(
                "INV-I-06: Link 关系不应接收 Webhook(只读链接,integration_id={})",
                integration.id
            )));
        }

        // Bidirectional 必须有 source_id 标记(INV-I-02 Loop 防护,生产应解析 payload.source_id)
        if integration.needs_loop_guard() {
            // 简化:这里假设 Webhook payload 含 "source_id" 字段;若缺失,标记为 skipped
            let has_source_marker = cmd.payload.contains("\"source_id\"");
            if !has_source_marker {
                let now = chrono::Utc::now();
                let skipped = SyncState {
                    id: SyncStateId::new(),
                    integration_id: integration.id,
                    tenant_id: integration.tenant_id,
                    sync_token: integration.sync_token.clone().unwrap_or_default(),
                    synced_at: now,
                    outcome: SyncOutcome::Skipped,
                    error: Some("INV-I-02: Bidirectional Webhook 缺 source_id 标记(Loop 防护)".to_string()),
                    direction: SyncDirection::Bidirectional,
                    processed_count: 0,
                    skipped_count: 1,
                    conflict_count: 0,
                    triggered_by_user_id: None,
                    created_at: now,
                };
                self.insert_sync_state(&skipped).await?;
                return Ok(skipped);
            }
        }

        // 创建 SyncState 记录 Webhook 处理
        let now = chrono::Utc::now();
        let sync_state = SyncState {
            id: SyncStateId::new(),
            integration_id: integration.id,
            tenant_id: integration.tenant_id,
            sync_token: integration.sync_token.clone().unwrap_or_default(),
            synced_at: now,
            outcome: SyncOutcome::Success,
            error: None,
            direction: match integration.relation_type {
                IntegrationRelationType::Mirror => SyncDirection::Inbound,
                IntegrationRelationType::Bidirectional => SyncDirection::Bidirectional,
                IntegrationRelationType::PlatformOwned => SyncDirection::Inbound,
                IntegrationRelationType::Link => SyncDirection::Inbound,
            },
            processed_count: 0,
            skipped_count: 0,
            conflict_count: 0,
            triggered_by_user_id: None,
            created_at: now,
        };

        // 记录 Webhook 幂等
        {
            let mut seen = self.webhook_events.write().expect("webhook lock");
            seen.insert(webhook_key, sync_state.id);
        }

        self.insert_sync_state(&sync_state).await?;

        // 事件总线:SyncCompleted
        let event = IntegrationEvent::SyncCompleted(crate::event::SyncCompleted {
            meta: EventMeta::new(integration.tenant_id),
            integration_id: integration.id,
            outcome: sync_state.outcome,
            synced_at: sync_state.synced_at,
            conflict_count: 0,
        });
        let _ = self.event_tx.send(event);

        Ok(sync_state)
    }
}

// =====================================================================
// IntegrationQueryPort 实现(4 方法)
// =====================================================================

#[async_trait]
impl IntegrationQueryPort for InMemoryIntegrationService {
    async fn get_integration(
        &self,
        id: IntegrationId,
        viewer: ActorContext,
    ) -> Result<Integration, IntegrationError> {
        let store = self.integrations.read().expect("integrations lock");
        let integration = store
            .get(&id)
            .cloned()
            .ok_or(IntegrationError::NotFound(id))?;
        check_invariant_03_tenant_required(&integration, viewer.tenant_id)?;
        if !viewer.can_access_project(integration.project_id) {
            return Err(IntegrationError::PermissionDenied);
        }
        Ok(integration)
    }

    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Integration>, IntegrationError> {
        if !viewer.can_access_project(q.project_id) {
            return Err(IntegrationError::PermissionDenied);
        }
        let store = self.integrations.read().expect("integrations lock");
        let list: Vec<Integration> = store
            .values()
            .filter(|i| {
                if i.tenant_id != viewer.tenant_id {
                    return false;
                }
                if i.project_id != q.project_id {
                    return false;
                }
                if let Some(src) = q.source_filter {
                    if i.source != src {
                        return false;
                    }
                }
                if let Some(rt) = q.relation_type_filter {
                    if i.relation_type != rt {
                        return false;
                    }
                }
                if let Some(st) = q.state_filter {
                    if i.state != st {
                        return false;
                    }
                }
                if q.active_only && !matches!(i.state, IntegrationState::Active) {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        Ok(list)
    }

    async fn get_sync_state(
        &self,
        id: IntegrationId,
        viewer: ActorContext,
    ) -> Result<SyncState, IntegrationError> {
        // 校验 integration 存在 + 跨 tenant 拒绝
        let integration = {
            let store = self.integrations.read().expect("integrations lock");
            store
                .get(&id)
                .cloned()
                .ok_or(IntegrationError::NotFound(id))?
        };
        check_invariant_03_tenant_required(&integration, viewer.tenant_id)?;
        if !viewer.can_access_project(integration.project_id) {
            return Err(IntegrationError::PermissionDenied);
        }

        // 取最新 SyncState
        let states = self.sync_states.read().expect("sync_states lock");
        let latest = states
            .get(&id)
            .and_then(|v| v.last().cloned())
            .ok_or_else(|| {
                IntegrationError::NotFound(id) // 复用 NotFound 变体:无 sync history
            })?;
        Ok(latest)
    }

    async fn get_history(
        &self,
        q: GetHistoryQuery,
        viewer: ActorContext,
    ) -> Result<Vec<SyncState>, IntegrationError> {
        // 校验 integration 存在 + 跨 tenant 拒绝
        let integration = {
            let store = self.integrations.read().expect("integrations lock");
            store
                .get(&q.integration_id)
                .cloned()
                .ok_or(IntegrationError::NotFound(q.integration_id))?
        };
        check_invariant_03_tenant_required(&integration, q.tenant_id)?;
        if !viewer.can_access_project(integration.project_id) {
            return Err(IntegrationError::PermissionDenied);
        }

        let states = self.sync_states.read().expect("sync_states lock");
        let mut list: Vec<SyncState> = states
            .get(&q.integration_id)
            .map(|v| v.clone())
            .unwrap_or_default();
        if let Some(since) = q.since {
            list.retain(|s| s.created_at >= since);
        }
        // 按时间倒序
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list.truncate(q.limit as usize);
        Ok(list)
    }
}

// =====================================================================
// IntegrationRepository 实现
// =====================================================================

#[async_trait]
impl IntegrationRepository for InMemoryIntegrationService {
    async fn insert_integration(&self, integration: &Integration) -> Result<(), IntegrationError> {
        self.integrations
            .write()
            .expect("integrations lock")
            .insert(integration.id, integration.clone());
        Ok(())
    }

    async fn find_integration_by_id(
        &self,
        id: IntegrationId,
    ) -> Result<Option<Integration>, IntegrationError> {
        Ok(self
            .integrations
            .read()
            .expect("integrations lock")
            .get(&id)
            .cloned())
    }

    async fn update_integration(&self, integration: &Integration) -> Result<(), IntegrationError> {
        self.integrations
            .write()
            .expect("integrations lock")
            .insert(integration.id, integration.clone());
        Ok(())
    }

    async fn delete_integration(&self, id: IntegrationId) -> Result<(), IntegrationError> {
        self.integrations.write().expect("integrations lock").remove(&id);
        self.sync_states.write().expect("sync_states lock").remove(&id);
        Ok(())
    }

    async fn list_integrations_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        source_filter: Option<IntegrationSource>,
        relation_type_filter: Option<IntegrationRelationType>,
        state_filter: Option<IntegrationState>,
        active_only: bool,
    ) -> Result<Vec<Integration>, IntegrationError> {
        let store = self.integrations.read().expect("integrations lock");
        Ok(store
            .values()
            .filter(|i| {
                if i.tenant_id != tenant_id || i.project_id != project_id {
                    return false;
                }
                if let Some(src) = source_filter {
                    if i.source != src {
                        return false;
                    }
                }
                if let Some(rt) = relation_type_filter {
                    if i.relation_type != rt {
                        return false;
                    }
                }
                if let Some(st) = state_filter {
                    if i.state != st {
                        return false;
                    }
                }
                if active_only && !matches!(i.state, IntegrationState::Active) {
                    return false;
                }
                true
            })
            .cloned()
            .collect())
    }

    async fn insert_sync_state(&self, sync_state: &SyncState) -> Result<(), IntegrationError> {
        let mut store = self.sync_states.write().expect("sync_states lock");
        store
            .entry(sync_state.integration_id)
            .or_default()
            .push(sync_state.clone());
        Ok(())
    }

    async fn find_latest_sync_state(
        &self,
        integration_id: IntegrationId,
    ) -> Result<Option<SyncState>, IntegrationError> {
        let store = self.sync_states.read().expect("sync_states lock");
        Ok(store
            .get(&integration_id)
            .and_then(|v| v.last().cloned()))
    }

    async fn list_sync_states(
        &self,
        integration_id: IntegrationId,
        limit: u32,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<SyncState>, IntegrationError> {
        let store = self.sync_states.read().expect("sync_states lock");
        let mut list: Vec<SyncState> = store
            .get(&integration_id)
            .map(|v| v.clone())
            .unwrap_or_default();
        if let Some(since) = since {
            list.retain(|s| s.created_at >= since);
        }
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list.truncate(limit as usize);
        Ok(list)
    }

    async fn find_sync_state_by_webhook_event(
        &self,
        integration_id: IntegrationId,
        external_event_id: &str,
    ) -> Result<Option<SyncState>, IntegrationError> {
        let key = format!("{}:{}", integration_id, external_event_id);
        let seen = self.webhook_events.read().expect("webhook lock");
        if let Some(_sync_state_id) = seen.get(&key) {
            // 仅命中幂等键;具体 sync_state 由 list_sync_states 取得
            let store = self.sync_states.read().expect("sync_states lock");
            let candidates = store.get(&integration_id).cloned().unwrap_or_default();
            return Ok(candidates.last().cloned());
        }
        Ok(None)
    }
}

// 静默抑制未使用导入
#[allow(dead_code)]
fn _unused_imports() {
    let _ = UserId::new();
    let _ = ConflictStrategy::ManualReview;
}

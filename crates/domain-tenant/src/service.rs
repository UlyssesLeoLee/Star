//! InMemoryTenantService:Phase 2 提供的内存实现
//!
//! 来源: spec/domain-tenant-spec.md §5(实施策略)
//!
//! **目标**:为 `TenantCommandPort` + `TenantQueryPort` 提供可工作的内存实现,
//! 用于本地集成测试与 P0 演示,不依赖任何数据库 / NATS 外部基础设施。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{Tenant, TenantPolicy, TenantQuota};
use crate::error::TenantError;
use crate::event::{EventMeta, TenantEvent};
use crate::invariants::{
    check_invariant_01_tenant_key_unique, check_invariant_02_status_transition,
    run_invariants, ALL_INVARIANT_CHECKS,
};
use crate::port::{
    ChangeTenantStatusCommand, CreateTenantCommand, ListTenantQuery, TenantCommandPort,
    TenantQueryPort, TenantRepository, UpdateTenantCommand, UpdateTenantPolicyCommand,
};
use crate::value_object::{
    TenantId, TenantPolicyId, TenantQuotaId, TenantStatus,
};

// =====================================================================
// InMemoryTenantService
// =====================================================================

/// **InMemory Tenant 命令/查询服务**
pub struct InMemoryTenantService {
    tenants: Arc<RwLock<HashMap<TenantId, Tenant>>>,
    policies: Arc<RwLock<HashMap<TenantPolicyId, TenantPolicy>>>,
    quotas: Arc<RwLock<HashMap<TenantQuotaId, TenantQuota>>>,
    event_tx: mpsc::UnboundedSender<TenantEvent>,
}

impl InMemoryTenantService {
    /// 创建新的内存服务(返回服务和事件接收端)。
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<TenantEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            quotas: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)。
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Tenant 数量
    pub async fn count(&self) -> usize {
        self.tenants.read().await.len()
    }

    /// 校验 actor 与命令的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), TenantError> {
        if actor.tenant_id != expected {
            return Err(TenantError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryTenantService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryTenantService {
    fn clone(&self) -> Self {
        Self {
            tenants: self.tenants.clone(),
            policies: self.policies.clone(),
            quotas: self.quotas.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// TenantCommandPort 实现
// =====================================================================

#[async_trait]
impl TenantCommandPort for InMemoryTenantService {
    async fn create_tenant(
        &self,
        cmd: CreateTenantCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError> {
        let now = chrono::Utc::now();
        let id = TenantId::new();
        let tenant = Tenant {
            id,
            tenant_key: cmd.tenant_key.clone(),
            name: cmd.name.clone(),
            status: TenantStatus::default(),
            tier: cmd.tier,
            contact_email: cmd.contact_email,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        // 1. 字段不变量
        run_invariants(ALL_INVARIANT_CHECKS, &tenant)?;

        // 2. 唯一性校验(INV-TEN-01)
        let existing_keys: Vec<String> = self
            .tenants
            .read()
            .await
            .values()
            .map(|t| t.tenant_key.clone())
            .collect();
        check_invariant_01_tenant_key_unique(&tenant, &existing_keys)?;

        // 3. 持久化
        self.tenants.write().await.insert(id, tenant.clone());

        // 4. 创建初始 TenantPolicy(若提供)
        if let Some(spec) = cmd.initial_policy {
            let policy_id = TenantPolicyId::new();
            let policy = TenantPolicy {
                id: policy_id,
                tenant_id: id,
                cloud_ai_allowed: spec.cloud_ai_allowed,
                cloud_ai_restricted: spec.cloud_ai_restricted,
                local_ai_only: spec.local_ai_only,
                no_code_upload: spec.no_code_upload,
                metadata_only: spec.metadata_only,
                specific_provider_ids: spec.specific_provider_ids,
                created_at: now,
                updated_at: now,
                version: 1,
            };
            self.policies.write().await.insert(policy_id, policy);
        }

        // 5. 创建默认 TenantQuota(Free 套餐)
        let quota_id = TenantQuotaId::new();
        let quota = TenantQuota {
            id: quota_id,
            tenant_id: id,
            max_users: 10,
            max_workspaces: 5,
            max_projects: 20,
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            used_users: 0,
            used_workspaces: 0,
            used_projects: 0,
            used_storage_bytes: 0,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.quotas.write().await.insert(quota_id, quota);

        // 6. 发送 Created 事件
        let event = TenantEvent::Created(crate::event::TenantCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(id)
            },
            tenant_id: id,
            tenant_key: tenant.tenant_key.clone(),
            name: tenant.name.clone(),
            tier: tenant.tier,
        });
        let _ = self.event_tx.send(event);

        Ok(tenant)
    }

    async fn update_tenant(
        &self,
        cmd: UpdateTenantCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError> {
        // 1. 校验 actor 角色(需 tenant_admin 或 platform_operator)
        if !actor.is_tenant_admin() && !actor.is_platform_operator() {
            return Err(TenantError::PermissionDenied);
        }
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 取出实体
        let mut store = self.tenants.write().await;
        let t = store
            .get_mut(&cmd.tenant_id)
            .ok_or(TenantError::NotFound(cmd.tenant_id))?;

        // 3. 乐观锁
        if t.version != cmd.expected_version {
            return Err(TenantError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, t.version
            )));
        }

        // 4. 应用变更
        if let Some(name) = cmd.name {
            t.name = name;
        }
        if let Some(email) = cmd.contact_email {
            t.contact_email = email;
        }
        if let Some(tier) = cmd.tier {
            t.tier = tier;
        }
        t.bump_version();

        Ok(t.clone())
    }

    async fn change_status(
        &self,
        cmd: ChangeTenantStatusCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError> {
        if !actor.is_tenant_admin() && !actor.is_platform_operator() {
            return Err(TenantError::PermissionDenied);
        }
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.tenants.write().await;
        let t = store
            .get_mut(&cmd.tenant_id)
            .ok_or(TenantError::NotFound(cmd.tenant_id))?;

        if t.version != cmd.expected_version {
            return Err(TenantError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, t.version
            )));
        }

        // 校验合法迁移(INV-TEN-02)
        check_invariant_02_status_transition(t, cmd.target_status)?;

        let from_status = t.status;
        t.status = cmd.target_status;
        t.bump_version();

        // 发送事件
        let event = TenantEvent::StatusChanged(crate::event::TenantStatusChanged {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            tenant_id: cmd.tenant_id,
            from_status,
            to_status: cmd.target_status,
        });
        let _ = self.event_tx.send(event);

        Ok(t.clone())
    }

    async fn update_tenant_policy(
        &self,
        cmd: UpdateTenantPolicyCommand,
        actor: ActorContext,
    ) -> Result<TenantPolicy, TenantError> {
        if !actor.is_tenant_admin() {
            return Err(TenantError::PermissionDenied);
        }
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.policies.write().await;
        let p = store
            .get_mut(&cmd.policy_id)
            .ok_or(TenantError::Internal(format!(
                "TenantPolicy {} not found",
                cmd.policy_id
            )))?;

        if p.tenant_id != cmd.tenant_id {
            return Err(TenantError::PermissionDenied);
        }
        if p.version != cmd.expected_version {
            return Err(TenantError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, p.version
            )));
        }

        let mut changed = Vec::new();
        if let Some(v) = cmd.cloud_ai_allowed {
            p.cloud_ai_allowed = v;
            changed.push("cloud_ai_allowed".to_string());
        }
        if let Some(v) = cmd.cloud_ai_restricted {
            p.cloud_ai_restricted = v;
            changed.push("cloud_ai_restricted".to_string());
        }
        if let Some(v) = cmd.local_ai_only {
            p.local_ai_only = v;
            changed.push("local_ai_only".to_string());
        }
        if let Some(v) = cmd.no_code_upload {
            p.no_code_upload = v;
            changed.push("no_code_upload".to_string());
        }
        if let Some(v) = cmd.metadata_only {
            p.metadata_only = v;
            changed.push("metadata_only".to_string());
        }
        if let Some(v) = cmd.specific_provider_ids {
            p.specific_provider_ids = v;
            changed.push("specific_provider_ids".to_string());
        }
        p.bump_version();

        // 发送事件
        let event = TenantEvent::PolicyUpdated(crate::event::TenantPolicyUpdated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            tenant_id: cmd.tenant_id,
            policy_id: cmd.policy_id,
            changed_fields: changed,
        });
        let _ = self.event_tx.send(event);

        Ok(p.clone())
    }
}

// =====================================================================
// TenantQueryPort 实现
// =====================================================================

#[async_trait]
impl TenantQueryPort for InMemoryTenantService {
    async fn get_by_id(
        &self,
        id: TenantId,
        viewer: ActorContext,
    ) -> Result<Tenant, TenantError> {
        // 平台运营可跨租户查询;否则只允许同租户
        if !viewer.is_platform_operator() && viewer.tenant_id != id {
            return Err(TenantError::PermissionDenied);
        }
        self.tenants
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(TenantError::NotFound(id))
    }

    async fn get_by_key(
        &self,
        tenant_key: &str,
        viewer: ActorContext,
    ) -> Result<Tenant, TenantError> {
        let t = self
            .tenants
            .read()
            .await
            .values()
            .find(|t| t.tenant_key == tenant_key)
            .cloned()
            .ok_or_else(|| {
                TenantError::Internal(format!("tenant_key '{tenant_key}' not found"))
            })?;
        if !viewer.is_platform_operator() && viewer.tenant_id != t.id {
            return Err(TenantError::PermissionDenied);
        }
        Ok(t)
    }

    async fn list_tenants(
        &self,
        q: ListTenantQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Tenant>, TenantError> {
        if !viewer.is_platform_operator() {
            return Err(TenantError::PermissionDenied);
        }
        let store = self.tenants.read().await;
        let mut all: Vec<Tenant> = store
            .values()
            .filter(|t| q.status.map_or(true, |s| t.status == s))
            .filter(|t| q.tier.map_or(true, |tier| t.tier == tier))
            .cloned()
            .collect();
        all.sort_by(|a, b| a.tenant_key.cmp(&b.tenant_key));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }

    async fn get_tenant_policy(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<TenantPolicy, TenantError> {
        if !viewer.is_platform_operator() && viewer.tenant_id != tenant_id {
            return Err(TenantError::PermissionDenied);
        }
        self.policies
            .read()
            .await
            .values()
            .find(|p| p.tenant_id == tenant_id)
            .cloned()
            .ok_or(TenantError::Internal(format!(
                "TenantPolicy for tenant {tenant_id} not found"
            )))
    }

    async fn get_tenant_quota(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<TenantQuota, TenantError> {
        if !viewer.is_platform_operator() && viewer.tenant_id != tenant_id {
            return Err(TenantError::PermissionDenied);
        }
        self.quotas
            .read()
            .await
            .values()
            .find(|q| q.tenant_id == tenant_id)
            .cloned()
            .ok_or(TenantError::Internal(format!(
                "TenantQuota for tenant {tenant_id} not found"
            )))
    }
}

// =====================================================================
// TenantRepository 实现
// =====================================================================

#[async_trait]
impl TenantRepository for InMemoryTenantService {
    async fn insert(&self, tenant: &Tenant) -> Result<(), TenantError> {
        self.tenants.write().await.insert(tenant.id, tenant.clone());
        Ok(())
    }
    async fn find_by_id(&self, id: TenantId) -> Result<Option<Tenant>, TenantError> {
        Ok(self.tenants.read().await.get(&id).cloned())
    }
    async fn find_by_key(&self, key: &str) -> Result<Option<Tenant>, TenantError> {
        Ok(self
            .tenants
            .read()
            .await
            .values()
            .find(|t| t.tenant_key == key)
            .cloned())
    }
    async fn update(&self, tenant: &Tenant) -> Result<(), TenantError> {
        self.tenants.write().await.insert(tenant.id, tenant.clone());
        Ok(())
    }
    async fn list_all_keys(&self) -> Result<Vec<String>, TenantError> {
        Ok(self
            .tenants
            .read()
            .await
            .values()
            .map(|t| t.tenant_key.clone())
            .collect())
    }
}

//! InMemoryIdentityService:Phase 2 提供的内存实现
//!
//! 为 `IdentityCommandPort` + `IdentityQueryPort` 提供可工作的内存实现。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{Credential, Device, DeviceBinding, Role, User};
use crate::error::IdentityError;
use crate::event::{EventMeta, IdentityEvent};
use crate::invariants::{
    check_invariant_01_email_unique, check_invariant_02_device_binding_unique, run_invariants,
    ALL_INVARIANT_CHECKS,
};
use crate::port::{
    BindDeviceCommand, CreateRoleCommand, CreateUserCommand, IdentityCommandPort,
    IdentityQueryPort, IdentityRepository, ListUserQuery, RecordLoginCommand, UpdateUserCommand,
};
use crate::value_object::{
    CredentialId, DeviceId, DeviceType, ProjectId, RoleId, TenantId, UserId,
};

// =====================================================================
// InMemoryIdentityService
// =====================================================================

/// **InMemory Identity 命令/查询服务**(Phase 2 真实实现)
pub struct InMemoryIdentityService {
    users: Arc<RwLock<HashMap<UserId, User>>>,
    devices: Arc<RwLock<HashMap<DeviceId, Device>>>,
    bindings: Arc<RwLock<HashMap<crate::value_object::DeviceBindingId, DeviceBinding>>>,
    credentials: Arc<RwLock<HashMap<CredentialId, Credential>>>,
    roles: Arc<RwLock<HashMap<RoleId, Role>>>,
    event_tx: mpsc::UnboundedSender<IdentityEvent>,
}

impl InMemoryIdentityService {
    /// 创建新的内存服务
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<IdentityEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            devices: Arc::new(RwLock::new(HashMap::new())),
            bindings: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 User 数量
    pub async fn count(&self) -> usize {
        self.users.read().await.len()
    }

    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), IdentityError> {
        if actor.tenant_id != expected {
            return Err(IdentityError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryIdentityService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryIdentityService {
    fn clone(&self) -> Self {
        Self {
            users: self.users.clone(),
            devices: self.devices.clone(),
            bindings: self.bindings.clone(),
            credentials: self.credentials.clone(),
            roles: self.roles.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// IdentityCommandPort 实现
// =====================================================================

#[async_trait]
impl IdentityCommandPort for InMemoryIdentityService {
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor: ActorContext,
    ) -> Result<User, IdentityError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let now = chrono::Utc::now();
        let id = uuid::Uuid::new_v4();
        let user = User {
            id,
            tenant_id: cmd.tenant_id,
            email: cmd.email.clone(),
            display_name: cmd.display_name.clone(),
            status: crate::value_object::UserStatus::Active,
            avatar_url: cmd.avatar_url,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            version: 1,
        };

        // 2. 基础不变量
        run_invariants(ALL_INVARIANT_CHECKS, &user)?;

        // 3. 邮箱唯一性 (INV-IDN-01)
        let existing_emails: Vec<String> = self
            .users
            .read()
            .await
            .values()
            .filter(|u| u.tenant_id == cmd.tenant_id)
            .map(|u| u.email.clone())
            .collect();
        check_invariant_01_email_unique(&user, &existing_emails)?;

        // 4. 持久化
        self.users.write().await.insert(id, user.clone());

        // 5. 创建初始 Credential
        if let Some(spec) = cmd.initial_credential {
            let cred_id = CredentialId::new();
            let cred = Credential {
                id: cred_id,
                tenant_id: cmd.tenant_id,
                user_id: id,
                credential_type: spec.credential_type,
                hash: spec.hash,
                provider_id: spec.provider_id,
                created_at: now,
                expires_at: spec.expires_at,
                last_used_at: None,
                version: 1,
            };
            self.credentials.write().await.insert(cred_id, cred);
        }

        // 6. 发送事件
        let event = IdentityEvent::UserCreated(crate::event::UserCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            user_id: id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
        });
        let _ = self.event_tx.send(event);

        Ok(user)
    }

    async fn update_user(
        &self,
        cmd: UpdateUserCommand,
        actor: ActorContext,
    ) -> Result<User, IdentityError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let mut store = self.users.write().await;
        let u = store
            .get_mut(&cmd.user_id)
            .ok_or(IdentityError::NotFound(cmd.user_id))?;

        if u.tenant_id != cmd.tenant_id {
            return Err(IdentityError::PermissionDenied);
        }
        if u.version != cmd.expected_version {
            return Err(IdentityError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, u.version
            )));
        }

        if let Some(name) = cmd.display_name {
            u.display_name = name;
        }
        if let Some(avatar) = cmd.avatar_url {
            u.avatar_url = avatar;
        }
        u.bump_version();

        Ok(u.clone())
    }

    async fn record_login(
        &self,
        cmd: RecordLoginCommand,
        actor: ActorContext,
    ) -> Result<(), IdentityError> {
        let mut users = self.users.write().await;
        let u = users
            .get_mut(&cmd.user_id)
            .ok_or(IdentityError::NotFound(cmd.user_id))?;

        if u.tenant_id != actor.tenant_id {
            return Err(IdentityError::PermissionDenied);
        }
        let now = chrono::Utc::now();
        u.last_login_at = Some(now);
        u.bump_version();

        // 发送事件
        let event = IdentityEvent::UserLoggedIn(crate::event::UserLoggedIn {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(u.tenant_id)
            },
            user_id: cmd.user_id,
            device_id: cmd.device_id,
            device_type: cmd.device_type,
            logged_in_at: now,
        });
        let _ = self.event_tx.send(event);

        Ok(())
    }

    async fn bind_device(
        &self,
        cmd: BindDeviceCommand,
        actor: ActorContext,
    ) -> Result<DeviceBinding, IdentityError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 1. 校验 user 存在
        {
            let users = self.users.read().await;
            if !users.contains_key(&cmd.user_id) {
                return Err(IdentityError::NotFound(cmd.user_id));
            }
        }

        // 2. INV-IDN-02 三元组唯一
        let existing: Vec<(uuid::Uuid, UserId, Option<uuid::Uuid>)> = self
            .bindings
            .read()
            .await
            .values()
            .map(|b| (b.device_id.into_uuid(), b.user_id, b.project_id.map(|p| p.into_uuid())))
            .collect();
        check_invariant_02_device_binding_unique(
            cmd.device_id.into_uuid(),
            cmd.user_id,
            cmd.project_id.map(|p| p.into_uuid()),
            &existing,
        )?;

        // 3. 持久化
        let id = crate::value_object::DeviceBindingId::new();
        let binding = DeviceBinding {
            id,
            tenant_id: cmd.tenant_id,
            device_id: cmd.device_id,
            user_id: cmd.user_id,
            project_id: cmd.project_id,
            bound_at: chrono::Utc::now(),
            reason: cmd.reason,
            version: 1,
        };
        self.bindings.write().await.insert(id, binding.clone());

        // 4. 发送事件
        // 查找/创建 Device
        let device = {
            let mut devices = self.devices.write().await;
            devices
                .entry(cmd.device_id)
                .or_insert_with(|| Device {
                    id: cmd.device_id,
                    tenant_id: cmd.tenant_id,
                    user_id: cmd.user_id,
                    device_fingerprint: format!("fp-{}", cmd.device_id),
                    device_type: DeviceType::default(),
                    device_name: None,
                    first_seen_at: chrono::Utc::now(),
                    last_seen_at: chrono::Utc::now(),
                    trusted: false,
                    version: 1,
                })
                .clone()
        };
        let event = IdentityEvent::DeviceBound(crate::event::DeviceBound {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            device_id: cmd.device_id,
            user_id: cmd.user_id,
            device_fingerprint: device.device_fingerprint,
        });
        let _ = self.event_tx.send(event);

        Ok(binding)
    }

    async fn create_role(
        &self,
        cmd: CreateRoleCommand,
        actor: ActorContext,
    ) -> Result<Role, IdentityError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        if !actor.is_tenant_admin() {
            return Err(IdentityError::PermissionDenied);
        }
        if cmd.name.trim().is_empty() {
            return Err(IdentityError::InvalidState(
                "role name 不能为空".to_string(),
            ));
        }

        // 唯一性: (tenant_id, name)
        if self
            .roles
            .read()
            .await
            .values()
            .any(|r| r.tenant_id == cmd.tenant_id && r.name == cmd.name)
        {
            return Err(IdentityError::Conflict(format!(
                "role '{}' 已存在",
                cmd.name
            )));
        }

        let now = chrono::Utc::now();
        let id = RoleId::new();
        let role = Role {
            id,
            tenant_id: cmd.tenant_id,
            name: cmd.name,
            description: cmd.description,
            permissions: cmd.permissions,
            built_in: false,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.roles.write().await.insert(id, role.clone());
        Ok(role)
    }
}

// =====================================================================
// IdentityQueryPort 实现
// =====================================================================

#[async_trait]
impl IdentityQueryPort for InMemoryIdentityService {
    async fn get_user(
        &self,
        id: UserId,
        viewer: ActorContext,
    ) -> Result<User, IdentityError> {
        let u = self
            .users
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(IdentityError::NotFound(id))?;
        if u.tenant_id != viewer.tenant_id {
            return Err(IdentityError::PermissionDenied);
        }
        Ok(u)
    }

    async fn get_user_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
        viewer: ActorContext,
    ) -> Result<User, IdentityError> {
        Self::check_tenant(&viewer, tenant_id)?;
        let u = self
            .users
            .read()
            .await
            .values()
            .find(|u| u.tenant_id == tenant_id && u.email.eq_ignore_ascii_case(email))
            .cloned()
            .ok_or(IdentityError::NotFound(UserId::nil_or_default()))?;
        Ok(u)
    }

    async fn list_users(
        &self,
        q: ListUserQuery,
        viewer: ActorContext,
    ) -> Result<Vec<User>, IdentityError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let mut all: Vec<User> = self
            .users
            .read()
            .await
            .values()
            .filter(|u| u.tenant_id == q.tenant_id)
            .filter(|u| {
                q.email_contains
                    .as_ref()
                    .map_or(true, |s| u.email.to_lowercase().contains(&s.to_lowercase()))
            })
            .filter(|u| q.status.map_or(true, |s| u.status == s))
            .cloned()
            .collect();
        all.sort_by(|a, b| a.email.cmp(&b.email));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }

    async fn list_devices(
        &self,
        user_id: UserId,
        viewer: ActorContext,
    ) -> Result<Vec<Device>, IdentityError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        Ok(self
            .devices
            .read()
            .await
            .values()
            .filter(|d| d.user_id == user_id && d.tenant_id == viewer.tenant_id)
            .cloned()
            .collect())
    }

    async fn list_user_roles(
        &self,
        user_id: UserId,
        viewer: ActorContext,
    ) -> Result<Vec<Role>, IdentityError> {
        // 通过 bindings/user_role 中间表关联;简化:返回 tenant 内全部 role
        let _ = user_id;
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        Ok(self
            .roles
            .read()
            .await
            .values()
            .filter(|r| r.tenant_id == viewer.tenant_id)
            .cloned()
            .collect())
    }

    async fn get_role(
        &self,
        id: RoleId,
        viewer: ActorContext,
    ) -> Result<Role, IdentityError> {
        let r = self
            .roles
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(IdentityError::Internal(format!("role {id} not found")))?;
        if r.tenant_id != viewer.tenant_id {
            return Err(IdentityError::PermissionDenied);
        }
        Ok(r)
    }

    async fn get_role_by_name(
        &self,
        tenant_id: TenantId,
        name: &str,
        viewer: ActorContext,
    ) -> Result<Role, IdentityError> {
        Self::check_tenant(&viewer, tenant_id)?;
        self.roles
            .read()
            .await
            .values()
            .find(|r| r.tenant_id == tenant_id && r.name == name)
            .cloned()
            .ok_or(IdentityError::Internal(format!(
                "role '{name}' in tenant {tenant_id} not found"
            )))
    }

    async fn list_user_credentials(
        &self,
        user_id: UserId,
        viewer: ActorContext,
    ) -> Result<Vec<Credential>, IdentityError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        Ok(self
            .credentials
            .read()
            .await
            .values()
            .filter(|c| c.user_id == user_id && c.tenant_id == viewer.tenant_id)
            .map(|c| {
                // 脱敏:hash 字段置空
                let mut sanitized = c.clone();
                sanitized.hash = "***".to_string();
                sanitized
            })
            .collect())
    }
}

// =====================================================================
// IdentityRepository 实现
// =====================================================================

#[async_trait]
impl IdentityRepository for InMemoryIdentityService {
    async fn insert_user(&self, user: &User) -> Result<(), IdentityError> {
        self.users.write().await.insert(user.id, user.clone());
        Ok(())
    }
    async fn find_user(&self, id: UserId) -> Result<Option<User>, IdentityError> {
        Ok(self.users.read().await.get(&id).cloned())
    }
    async fn list_emails(&self, tenant_id: TenantId) -> Result<Vec<String>, IdentityError> {
        Ok(self
            .users
            .read()
            .await
            .values()
            .filter(|u| u.tenant_id == tenant_id)
            .map(|u| u.email.clone())
            .collect())
    }
    async fn update_user(&self, user: &User) -> Result<(), IdentityError> {
        self.users.write().await.insert(user.id, user.clone());
        Ok(())
    }
    async fn list_devices(&self) -> Result<Vec<Device>, IdentityError> {
        Ok(self.devices.read().await.values().cloned().collect())
    }
}

// 占位:在 IdentityQueryPort::get_user_by_email 错误时返回
impl UserId {
    fn nil_or_default() -> Self {
        Self::default()
    }
}

// 防止 unused import
#[allow(dead_code)]
fn _unused() {
    let _ = ProjectId::default();
}

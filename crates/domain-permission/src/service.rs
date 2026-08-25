//! InMemoryPermissionService

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{Permission, PermissionScheme, Role};
use crate::error::PermissionError;
use crate::event::{EventMeta, PermissionEvent};
use crate::invariants::{
    check_invariant_02_scheme_has_owner, run_invariants, ALL_INVARIANT_CHECKS,
};
use crate::port::{
    CheckPermissionQuery, CreatePermissionSchemeCommand, CreateRoleCommand,
    PermissionCommandPort, PermissionQueryPort, UpdateRoleCommand,
};
use crate::value_object::{PermissionId, PermissionSchemeId, PermissionScope, RoleId, TenantId};

/// **InMemory Permission 命令/查询服务**
pub struct InMemoryPermissionService {
    roles: Arc<RwLock<HashMap<RoleId, Role>>>,
    permissions: Arc<RwLock<HashMap<PermissionId, Permission>>>,
    schemes: Arc<RwLock<HashMap<PermissionSchemeId, PermissionScheme>>>,
    event_tx: mpsc::UnboundedSender<PermissionEvent>,
}

impl InMemoryPermissionService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<PermissionEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut permissions = HashMap::new();
        // 预置 6 个标准权限
        let builtins = [
            ("workitem:read", "Read work items", PermissionScope::Project),
            ("workitem:create", "Create work items", PermissionScope::Project),
            ("workitem:update", "Update work items", PermissionScope::Project),
            ("workitem:delete", "Delete work items", PermissionScope::Project),
            ("worktree:create", "Create worktree", PermissionScope::Project),
            ("project:admin", "Project admin", PermissionScope::Project),
        ];
        for (code, name, scope) in builtins {
            let id = PermissionId::new();
            permissions.insert(
                id,
                Permission {
                    id,
                    code: code.to_string(),
                    name: name.to_string(),
                    description: None,
                    scope,
                    created_at: chrono::Utc::now(),
                },
            );
        }
        let svc = Arc::new(Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(permissions)),
            schemes: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), PermissionError> {
        if actor.tenant_id != expected {
            return Err(PermissionError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryPermissionService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryPermissionService {
    fn clone(&self) -> Self {
        Self {
            roles: self.roles.clone(),
            permissions: self.permissions.clone(),
            schemes: self.schemes.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl PermissionCommandPort for InMemoryPermissionService {
    async fn create_role(
        &self,
        cmd: CreateRoleCommand,
        actor: ActorContext,
    ) -> Result<Role, PermissionError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        if !actor.is_tenant_admin() {
            return Err(PermissionError::PermissionDenied);
        }
        if self
            .roles
            .read()
            .await
            .values()
            .any(|r| r.tenant_id == cmd.tenant_id && r.name == cmd.name)
        {
            return Err(PermissionError::Conflict(format!(
                "role '{}' already exists",
                cmd.name
            )));
        }
        let now = chrono::Utc::now();
        let id = RoleId::new();
        let role = Role {
            id,
            tenant_id: cmd.tenant_id,
            name: cmd.name.clone(),
            description: cmd.description,
            permissions: cmd.permissions,
            built_in: false,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &role)?;
        self.roles.write().await.insert(id, role.clone());

        let event = PermissionEvent::RoleCreated(crate::event::RoleCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            role_id: id,
            name: role.name.clone(),
        });
        let _ = self.event_tx.send(event);
        Ok(role)
    }

    async fn update_role(
        &self,
        cmd: UpdateRoleCommand,
        actor: ActorContext,
    ) -> Result<Role, PermissionError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        if !actor.is_tenant_admin() {
            return Err(PermissionError::PermissionDenied);
        }
        let mut store = self.roles.write().await;
        let r = store
            .get_mut(&cmd.role_id)
            .ok_or(PermissionError::NotFound(cmd.role_id))?;
        if r.tenant_id != cmd.tenant_id {
            return Err(PermissionError::PermissionDenied);
        }
        if r.built_in {
            return Err(PermissionError::InvalidState(
                "内置 role 不可修改".to_string(),
            ));
        }
        if r.version != cmd.expected_version {
            return Err(PermissionError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, r.version
            )));
        }
        if let Some(perms) = cmd.permissions {
            r.permissions = perms;
        }
        if let Some(desc) = cmd.description {
            r.description = desc;
        }
        r.bump_version();
        Ok(r.clone())
    }

    async fn delete_role(
        &self,
        role_id: RoleId,
        actor: ActorContext,
    ) -> Result<(), PermissionError> {
        if !actor.is_tenant_admin() {
            return Err(PermissionError::PermissionDenied);
        }
        let mut store = self.roles.write().await;
        let r = store.get(&role_id).cloned();
        match r {
            Some(r) => {
                if r.tenant_id != actor.tenant_id {
                    return Err(PermissionError::PermissionDenied);
                }
                if r.built_in {
                    return Err(PermissionError::InvalidState(
                        "内置 role 不可删除".to_string(),
                    ));
                }
                store.remove(&role_id);
                Ok(())
            }
            None => Err(PermissionError::NotFound(role_id)),
        }
    }

    async fn create_scheme(
        &self,
        cmd: CreatePermissionSchemeCommand,
        actor: ActorContext,
    ) -> Result<PermissionScheme, PermissionError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let id = PermissionSchemeId::new();
        let scheme = PermissionScheme {
            id,
            project_id: cmd.project_id,
            tenant_id: cmd.tenant_id,
            name: cmd.name.clone(),
            role_permissions: cmd.role_permissions,
            default_role: cmd.default_role,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        check_invariant_02_scheme_has_owner(&scheme)?;
        self.schemes.write().await.insert(id, scheme.clone());

        let event = PermissionEvent::SchemeCreated(crate::event::SchemeCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id),
                ..EventMeta::new(cmd.tenant_id)
            },
            scheme_id: id,
            project_id: cmd.project_id.into_uuid(),
            name: scheme.name.clone(),
        });
        let _ = self.event_tx.send(event);
        Ok(scheme)
    }
}

#[async_trait]
impl PermissionQueryPort for InMemoryPermissionService {
    async fn get_role(
        &self,
        id: RoleId,
        viewer: ActorContext,
    ) -> Result<Role, PermissionError> {
        let r = self
            .roles
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(PermissionError::NotFound(id))?;
        if r.tenant_id != viewer.tenant_id {
            return Err(PermissionError::PermissionDenied);
        }
        Ok(r)
    }
    async fn list_roles(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<Vec<Role>, PermissionError> {
        Self::check_tenant(&viewer, tenant_id)?;
        Ok(self
            .roles
            .read()
            .await
            .values()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
    async fn get_scheme(
        &self,
        id: PermissionSchemeId,
        viewer: ActorContext,
    ) -> Result<PermissionScheme, PermissionError> {
        let s = self
            .schemes
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(PermissionError::Internal(format!("scheme {id} not found")))?;
        if s.tenant_id != viewer.tenant_id {
            return Err(PermissionError::PermissionDenied);
        }
        Ok(s)
    }
    async fn get_scheme_by_project(
        &self,
        project_id: crate::value_object::ProjectId,
        viewer: ActorContext,
    ) -> Result<PermissionScheme, PermissionError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        self.schemes
            .read()
            .await
            .values()
            .find(|s| s.project_id == project_id)
            .cloned()
            .ok_or(PermissionError::Internal(format!(
                "scheme for project {project_id} not found"
            )))
    }
    async fn check_permission(
        &self,
        q: CheckPermissionQuery,
        viewer: ActorContext,
    ) -> Result<bool, PermissionError> {
        let r = self
            .roles
            .read()
            .await
            .get(&q.role_id)
            .cloned()
            .ok_or(PermissionError::NotFound(q.role_id))?;
        if r.tenant_id != viewer.tenant_id {
            return Err(PermissionError::PermissionDenied);
        }
        let granted = r.has_permission(&q.permission);
        let event = PermissionEvent::PermissionChecked(crate::event::PermissionChecked {
            meta: EventMeta {
                actor_user_id: Some(viewer.user_id),
                ..EventMeta::new(r.tenant_id)
            },
            role_id: q.role_id,
            permission: q.permission,
            granted,
        });
        let _ = self.event_tx.send(event);
        Ok(granted)
    }
    async fn list_permissions(&self) -> Result<Vec<Permission>, PermissionError> {
        Ok(self.permissions.read().await.values().cloned().collect())
    }
}

// 防止 unused import
#[allow(dead_code)]
fn _unused(p: Permission) {
    let _ = p.code;
}

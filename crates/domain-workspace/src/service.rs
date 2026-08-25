//! InMemoryWorkspaceService

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{Workspace, WorkspaceMember};
use crate::error::WorkspaceError;
use crate::event::{EventMeta, WorkspaceEvent};
use crate::invariants::{
    check_invariant_01_workspace_key_unique, run_invariants, ALL_INVARIANT_CHECKS,
};
use crate::port::{
    AddMemberCommand, CreateWorkspaceCommand, ListWorkspaceQuery, RemoveMemberCommand,
    UpdateWorkspaceCommand, WorkspaceCommandPort, WorkspaceQueryPort,
};
use crate::value_object::{TenantId, UserId, WorkspaceId, WorkspaceMemberId, WorkspaceRole};

/// **InMemory Workspace 命令/查询服务**
pub struct InMemoryWorkspaceService {
    workspaces: Arc<RwLock<HashMap<WorkspaceId, Workspace>>>,
    members: Arc<RwLock<HashMap<WorkspaceMemberId, WorkspaceMember>>>,
    event_tx: mpsc::UnboundedSender<WorkspaceEvent>,
}

impl InMemoryWorkspaceService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<WorkspaceEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    pub async fn count(&self) -> usize {
        self.workspaces.read().await.len()
    }
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), WorkspaceError> {
        if actor.tenant_id != expected {
            return Err(WorkspaceError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryWorkspaceService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryWorkspaceService {
    fn clone(&self) -> Self {
        Self {
            workspaces: self.workspaces.clone(),
            members: self.members.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl WorkspaceCommandPort for InMemoryWorkspaceService {
    async fn create_workspace(
        &self,
        cmd: CreateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let id = WorkspaceId::new();
        let ws = Workspace {
            id,
            tenant_id: cmd.tenant_id,
            workspace_key: cmd.workspace_key.clone(),
            name: cmd.name.clone(),
            description: cmd.description,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &ws)?;
        let existing_keys: Vec<String> = self
            .workspaces
            .read()
            .await
            .values()
            .filter(|w| w.tenant_id == cmd.tenant_id)
            .map(|w| w.workspace_key.clone())
            .collect();
        check_invariant_01_workspace_key_unique(&ws, &existing_keys)?;
        self.workspaces.write().await.insert(id, ws.clone());

        // 自动加 owner 为 Admin 成员
        let m_id = WorkspaceMemberId::new();
        let member = WorkspaceMember {
            id: m_id,
            workspace_id: id,
            tenant_id: cmd.tenant_id,
            user_id: cmd.owner_user_id,
            role: WorkspaceRole::Admin,
            joined_at: now,
            version: 1,
        };
        self.members.write().await.insert(m_id, member);

        // 事件
        let event = WorkspaceEvent::Created(crate::event::WorkspaceCreated {
            meta: EventMeta {
                actor_user_id: Some(UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            workspace_id: id,
            workspace_key: ws.workspace_key.clone(),
            name: ws.name.clone(),
        });
        let _ = self.event_tx.send(event);
        Ok(ws)
    }

    async fn update_workspace(
        &self,
        cmd: UpdateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.workspaces.write().await;
        let w = store
            .get_mut(&cmd.workspace_id)
            .ok_or(WorkspaceError::NotFound(cmd.workspace_id))?;
        if w.tenant_id != cmd.tenant_id {
            return Err(WorkspaceError::PermissionDenied);
        }
        if w.version != cmd.expected_version {
            return Err(WorkspaceError::Conflict(format!(
                "version mismatch: expected {}, actual {}",
                cmd.expected_version, w.version
            )));
        }
        if let Some(name) = cmd.name {
            w.name = name;
        }
        if let Some(desc) = cmd.description {
            w.description = desc;
        }
        w.bump_version();
        Ok(w.clone())
    }

    async fn add_member(
        &self,
        cmd: AddMemberCommand,
        actor: ActorContext,
    ) -> Result<WorkspaceMember, WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // workspace 存在性
        if !self.workspaces.read().await.contains_key(&cmd.workspace_id) {
            return Err(WorkspaceError::NotFound(cmd.workspace_id));
        }
        // 重复成员检查
        if self
            .members
            .read()
            .await
            .values()
            .any(|m| m.workspace_id == cmd.workspace_id && m.user_id == cmd.user_id)
        {
            return Err(WorkspaceError::Conflict(format!(
                "user {} 已是 workspace {} 成员",
                cmd.user_id, cmd.workspace_id
            )));
        }
        let id = WorkspaceMemberId::new();
        let member = WorkspaceMember {
            id,
            workspace_id: cmd.workspace_id,
            tenant_id: cmd.tenant_id,
            user_id: cmd.user_id,
            role: cmd.role,
            joined_at: chrono::Utc::now(),
            version: 1,
        };
        self.members.write().await.insert(id, member.clone());

        let event = WorkspaceEvent::MemberAdded(crate::event::MemberAdded {
            meta: EventMeta {
                actor_user_id: Some(UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            workspace_id: cmd.workspace_id,
            user_id: cmd.user_id,
            role: cmd.role,
        });
        let _ = self.event_tx.send(event);
        Ok(member)
    }

    async fn remove_member(
        &self,
        cmd: RemoveMemberCommand,
        actor: ActorContext,
    ) -> Result<(), WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.members.write().await;
        let m_id = store
            .iter()
            .find(|(_, m)| {
                m.workspace_id == cmd.workspace_id
                    && m.user_id == cmd.user_id
                    && m.tenant_id == cmd.tenant_id
            })
            .map(|(id, _)| *id);
        if let Some(id) = m_id {
            store.remove(&id);
        } else {
            return Err(WorkspaceError::NotFound(cmd.workspace_id));
        }
        let event = WorkspaceEvent::MemberRemoved(crate::event::MemberRemoved {
            meta: EventMeta {
                actor_user_id: Some(UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            workspace_id: cmd.workspace_id,
            user_id: cmd.user_id,
        });
        let _ = self.event_tx.send(event);
        Ok(())
    }
}

#[async_trait]
impl WorkspaceQueryPort for InMemoryWorkspaceService {
    async fn get_by_id(
        &self,
        id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        let w = self
            .workspaces
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(WorkspaceError::NotFound(id))?;
        if w.tenant_id != viewer.tenant_id {
            return Err(WorkspaceError::PermissionDenied);
        }
        Ok(w)
    }
    async fn get_by_key(
        &self,
        tenant_id: TenantId,
        workspace_key: &str,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        Self::check_tenant(&viewer, tenant_id)?;
        self.workspaces
            .read()
            .await
            .values()
            .find(|w| w.tenant_id == tenant_id && w.workspace_key == workspace_key)
            .cloned()
            .ok_or(WorkspaceError::NotFound(WorkspaceId::default()))
    }
    async fn list_workspaces(
        &self,
        q: ListWorkspaceQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Workspace>, WorkspaceError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let mut all: Vec<Workspace> = self
            .workspaces
            .read()
            .await
            .values()
            .filter(|w| w.tenant_id == q.tenant_id)
            .cloned()
            .collect();
        all.sort_by(|a, b| a.workspace_key.cmp(&b.workspace_key));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }
    async fn list_members(
        &self,
        workspace_id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Vec<WorkspaceMember>, WorkspaceError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        Ok(self
            .members
            .read()
            .await
            .values()
            .filter(|m| m.workspace_id == workspace_id && m.tenant_id == viewer.tenant_id)
            .cloned()
            .collect())
    }
}

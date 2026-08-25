//! InMemoryCollaborationService:Phase 2 提供的内存实现
//!
//! 来源: spec §5(实施策略) + parent task
//!
//! **目标**:为 `CollaborationCommandPort` + `CollaborationQueryPort` +
//! `CollaborationRepository` + `RealtimeEventRouter` 提供 1 个真实可工作的实现,
//! 用于本地集成测试与 P0 演示,不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**:`crates/infrastructure` 提供 SQLx / NATS / WebSocket Adapter 取代本实现。

use async_trait::async_trait;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{
    CollaborationSession, PresenceCursor, PresenceParticipant, RealtimeChannel,
};
use crate::error::CollaborationError;
use crate::event::{CollaborationEvent, EventMeta};
use crate::invariants::{
    check_create_invariants, check_invariant_01_tenant_id_present,
    check_invariant_05_channel_filter_not_empty, check_invariant_07_owner_or_admin,
    check_invariant_08_cursor_selection_valid,
};
use crate::port::{
    CloseSessionCommand, CollaborationCommandPort, CollaborationQueryPort,
    CollaborationRepository, GetCursorQuery, HeartbeatCommand, JoinSessionCommand,
    LeaveSessionCommand, ListActiveSessionsQuery, ListParticipantsQuery, OpenSessionCommand,
    RealtimeEventRouter, UpdateCursorCommand,
};
use crate::value_object::{
    ChannelId, ParticipantId, ParticipantStatus, ProjectId, ResourceType, SessionId, TenantId,
};

/// 单连接允许的最大 Channel 数(spec §8 CB-002)
pub const MAX_CHANNELS_PER_CONNECTION: usize = 100;

/// 默认心跳超时(秒,INV-CB-03)
pub const DEFAULT_HEARTBEAT_TIMEOUT_SECS: i64 = 60;

/// 默认 Session idle 阈值(秒,用于 is_stale)
pub const DEFAULT_SESSION_IDLE_SECS: i64 = 30 * 60; // 30 min

/// Channel 7 天 TTL(api-design §4.2)
pub const CHANNEL_TTL_SECS: i64 = 7 * 24 * 60 * 60;

// =====================================================================
// InMemoryCollaborationService
// =====================================================================

/// **InMemory Collaboration 命令/查询/仓库/路由服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryCollaborationService {
    /// Session 存储
    sessions: Arc<RwLock<HashMap<SessionId, CollaborationSession>>>,
    /// Participant 存储
    participants: Arc<RwLock<HashMap<ParticipantId, PresenceParticipant>>>,
    /// Session → Participant 索引
    session_participants: Arc<RwLock<HashMap<SessionId, HashSet<ParticipantId>>>>,
    /// Cursor 存储(Participant → Cursor,1对0..1)
    cursors: Arc<RwLock<HashMap<ParticipantId, PresenceCursor>>>,
    /// Channel 存储
    channels: Arc<RwLock<HashMap<ChannelId, RealtimeChannel>>>,
    /// Connection(User) → Channels 索引(用于 INV-CB-02 限流)
    connection_channels: Arc<RwLock<HashMap<uuid::Uuid, HashSet<ChannelId>>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<CollaborationEvent>,
    /// 路由投递计数(测试断言)
    route_counter: Arc<RwLock<HashMap<uuid::Uuid, usize>>>,
}

impl InMemoryCollaborationService {
    /// 创建新的内存服务(返回服务和事件接收端)。
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<CollaborationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            participants: Arc::new(RwLock::new(HashMap::new())),
            session_participants: Arc::new(RwLock::new(HashMap::new())),
            cursors: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            connection_channels: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
            route_counter: Arc::new(RwLock::new(HashMap::new())),
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)。
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Session 数量
    pub async fn count_sessions(&self) -> usize {
        self.sessions.read().expect("lock").len()
    }

    /// 当前 Participant 数量
    pub async fn count_participants(&self) -> usize {
        self.participants.read().expect("lock").len()
    }

    /// 当前 Channel 数量
    pub async fn count_channels(&self) -> usize {
        self.channels.read().expect("lock").len()
    }

    /// 检查 actor.tenant_id 是否与 cmd.tenant_id 一致
    fn check_tenant(
        actor: &ActorContext,
        expected: TenantId,
    ) -> Result<(), CollaborationError> {
        check_invariant_01_tenant_id_present(actor.tenant_id, expected)
    }

    /// 取得 Session(带租户校验,owner / tenant_admin 可跨)
    fn fetch_session(
        &self,
        id: SessionId,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollaborationError> {
        let store = self.sessions.read().expect("lock");
        let s = store
            .get(&id)
            .cloned()
            .ok_or(CollaborationError::NotFound(id))?;
        if s.tenant_id != actor.tenant_id {
            return Err(CollaborationError::PermissionDenied);
        }
        Ok(s)
    }
}

impl Default for InMemoryCollaborationService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryCollaborationService {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            participants: self.participants.clone(),
            session_participants: self.session_participants.clone(),
            cursors: self.cursors.clone(),
            channels: self.channels.clone(),
            connection_channels: self.connection_channels.clone(),
            event_tx: self.event_tx.clone(),
            route_counter: self.route_counter.clone(),
        }
    }
}

// =====================================================================
// CollaborationCommandPort 实现(6 方法)
// =====================================================================

#[async_trait]
impl CollaborationCommandPort for InMemoryCollaborationService {
    async fn open_session(
        &self,
        cmd: OpenSessionCommand,
        actor: ActorContext,
    ) -> Result<CollaborationSession, CollaborationError> {
        // INV-CB-01:tenant 校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        let now = Utc::now();
        let session = CollaborationSession {
            id: SessionId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            workspace_id: cmd.workspace_id,
            name: cmd.name.clone(),
            description: cmd.description.clone(),
            owner_user_id: actor.user_id,
            is_open: cmd.is_open,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };

        // 创建时基本不变量校验
        check_create_invariants(&session)?;

        // 持久化
        self.sessions
            .write()
            .expect("lock")
            .insert(session.id, session.clone());

        // 事件
        let event = CollaborationEvent::SessionOpened(crate::event::SessionOpened {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            session_id: session.id,
            project_id: session.project_id,
            name: session.name.clone(),
            is_open: session.is_open,
        });
        let _ = self.event_tx.send(event);

        Ok(session)
    }

    async fn join_session(
        &self,
        cmd: JoinSessionCommand,
        actor: ActorContext,
    ) -> Result<PresenceParticipant, CollaborationError> {
        // INV-CB-01
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // 取 Session(带租户校验)
        let session = self.fetch_session(cmd.session_id, &actor)?;
        // INV-CB-06:actor 必须在 Session.project_id 范围内(简化:Session 自身 project 一致,
        // 实际由上层 application / API Gateway 校验 actor 对 project 的可访问性)

        let now = Utc::now();
        let p = PresenceParticipant {
            id: ParticipantId::new(),
            session_id: cmd.session_id,
            tenant_id: cmd.tenant_id,
            project_id: session.project_id,
            user_id: cmd.user_id,
            status: ParticipantStatus::Active,
            resource_type: cmd.resource_type.clone(),
            resource_id: cmd.resource_id,
            last_active_at: now,
            heartbeat_expires_at: now + chrono::Duration::seconds(DEFAULT_HEARTBEAT_TIMEOUT_SECS),
            joined_at: now,
        };

        self.participants
            .write()
            .expect("lock")
            .insert(p.id, p.clone());
        self.session_participants
            .write()
            .expect("lock")
            .entry(cmd.session_id)
            .or_default()
            .insert(p.id);

        // 事件
        let event = CollaborationEvent::ParticipantJoined(crate::event::ParticipantJoined {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            session_id: p.session_id,
            participant_id: p.id,
            user_id: p.user_id,
            resource_type: p.resource_type.clone(),
        });
        let _ = self.event_tx.send(event);

        Ok(p)
    }

    async fn leave_session(
        &self,
        cmd: LeaveSessionCommand,
        actor: ActorContext,
    ) -> Result<(), CollaborationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let session = self.fetch_session(cmd.session_id, &actor)?;

        // 校验 participant 存在
        let p = {
            let store = self.participants.read().expect("lock");
            store
                .get(&cmd.participant_id)
                .cloned()
                .ok_or(CollaborationError::NotFound(session.id))?
        };
        if p.tenant_id != cmd.tenant_id || p.session_id != cmd.session_id {
            return Err(CollaborationError::PermissionDenied);
        }
        // INV-CB-07:仅本人 / tenant_admin 可 leave(本人 = p.user_id)
        check_invariant_07_owner_or_admin(
            actor.user_id,
            actor.is_tenant_admin(),
            p.user_id,
        )?;

        self.participants
            .write()
            .expect("lock")
            .remove(&cmd.participant_id);
        if let Some(set) = self
            .session_participants
            .write()
            .expect("lock")
            .get_mut(&cmd.session_id)
        {
            set.remove(&cmd.participant_id);
        }
        // 关联 Cursor 一并清理
        self.cursors
            .write()
            .expect("lock")
            .remove(&cmd.participant_id);

        let event = CollaborationEvent::ParticipantLeft(crate::event::ParticipantLeft {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            session_id: cmd.session_id,
            participant_id: cmd.participant_id,
        });
        let _ = self.event_tx.send(event);

        Ok(())
    }

    async fn heartbeat(
        &self,
        cmd: HeartbeatCommand,
        actor: ActorContext,
    ) -> Result<PresenceParticipant, CollaborationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let session = self.fetch_session(cmd.session_id, &actor)?;

        let mut p = {
            let store = self.participants.read().expect("lock");
            store
                .get(&cmd.participant_id)
                .cloned()
                .ok_or(CollaborationError::NotFound(session.id))?
        };
        if p.tenant_id != cmd.tenant_id || p.session_id != cmd.session_id {
            return Err(CollaborationError::PermissionDenied);
        }
        // 仅本参与者可心跳
        if p.user_id != actor.user_id {
            return Err(CollaborationError::PermissionDenied);
        }

        p.bump_version();
        p.status = ParticipantStatus::Active;
        self.participants
            .write()
            .expect("lock")
            .insert(p.id, p.clone());

        let event = CollaborationEvent::HeartbeatReceived(crate::event::HeartbeatReceived {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            session_id: p.session_id,
            participant_id: p.id,
            last_active_at: p.last_active_at,
            status: p.status,
        });
        let _ = self.event_tx.send(event);

        Ok(p)
    }

    async fn update_cursor(
        &self,
        cmd: UpdateCursorCommand,
        actor: ActorContext,
    ) -> Result<PresenceCursor, CollaborationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let session = self.fetch_session(cmd.session_id, &actor)?;

        // 校验 participant 存在且属于 actor
        let p = {
            let store = self.participants.read().expect("lock");
            store
                .get(&cmd.participant_id)
                .cloned()
                .ok_or(CollaborationError::NotFound(session.id))?
        };
        if p.tenant_id != cmd.tenant_id || p.user_id != actor.user_id {
            return Err(CollaborationError::PermissionDenied);
        }

        let now = Utc::now();
        let cursor = PresenceCursor {
            id: uuid::Uuid::new_v4(),
            session_id: cmd.session_id,
            participant_id: cmd.participant_id,
            tenant_id: cmd.tenant_id,
            resource_type: cmd.resource_type.clone(),
            resource_id: cmd.resource_id,
            position_x: cmd.position_x,
            position_y: cmd.position_y,
            selection_start: cmd.selection_start,
            selection_end: cmd.selection_end,
            selection_shape: cmd.selection_shape,
            updated_at: now,
        };

        // INV-CB-08
        check_invariant_08_cursor_selection_valid(&cursor)?;

        self.cursors
            .write()
            .expect("lock")
            .insert(cmd.participant_id, cursor.clone());

        let event = CollaborationEvent::CursorMoved(crate::event::CursorMoved {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            session_id: cursor.session_id,
            participant_id: cursor.participant_id,
            resource_type: cursor.resource_type.clone(),
            resource_id: cursor.resource_id,
            position_x: cursor.position_x,
            position_y: cursor.position_y,
            selection_start: cursor.selection_start,
            selection_end: cursor.selection_end,
            selection_shape: cursor.selection_shape,
        });
        let _ = self.event_tx.send(event);

        Ok(cursor)
    }

    async fn close_session(
        &self,
        cmd: CloseSessionCommand,
        actor: ActorContext,
    ) -> Result<(), CollaborationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let session = self.fetch_session(cmd.session_id, &actor)?;

        // INV-CB-07:owner or tenant_admin
        check_invariant_07_owner_or_admin(
            actor.user_id,
            actor.is_tenant_admin(),
            session.owner_user_id,
        )?;

        // 收集 participant / channel id 以便级联清理
        let participant_ids: Vec<ParticipantId> = self
            .session_participants
            .read()
            .expect("lock")
            .get(&cmd.session_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        let channel_ids: Vec<ChannelId> = self
            .channels
            .read()
            .expect("lock")
            .values()
            .filter(|c| c.session_id == cmd.session_id)
            .map(|c| c.id)
            .collect();

        // 删除 Session
        self.sessions
            .write()
            .expect("lock")
            .remove(&cmd.session_id);
        // 删除关联 Participant / Cursor / Channel
        {
            let mut participants = self.participants.write().expect("lock");
            for pid in &participant_ids {
                participants.remove(pid);
            }
        }
        {
            let mut cursors = self.cursors.write().expect("lock");
            for pid in &participant_ids {
                cursors.remove(pid);
            }
        }
        {
            let mut channels = self.channels.write().expect("lock");
            for cid in &channel_ids {
                channels.remove(cid);
            }
        }
        self.session_participants
            .write()
            .expect("lock")
            .remove(&cmd.session_id);

        let event = CollaborationEvent::SessionClosed(crate::event::SessionClosed {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            session_id: cmd.session_id,
        });
        let _ = self.event_tx.send(event);

        Ok(())
    }
}

// =====================================================================
// CollaborationQueryPort 实现(4 方法)
// =====================================================================

#[async_trait]
impl CollaborationQueryPort for InMemoryCollaborationService {
    async fn get_session(
        &self,
        id: SessionId,
        viewer: ActorContext,
    ) -> Result<CollaborationSession, CollaborationError> {
        self.fetch_session(id, &viewer)
    }

    async fn list_active_sessions(
        &self,
        q: ListActiveSessionsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<CollaborationSession>, CollaborationError> {
        // 租户隔离
        if viewer.tenant_id != q.tenant_id {
            return Err(CollaborationError::PermissionDenied);
        }
        let store = self.sessions.read().expect("lock");
        let now = Utc::now();
        let mut out: Vec<CollaborationSession> = store
            .values()
            .filter(|s| s.tenant_id == q.tenant_id)
            .filter(|s| s.project_id == q.project_id)
            .filter(|s| match q.owner_user_id {
                Some(uid) => s.owner_user_id == uid,
                None => true,
            })
            // 默认排除 stale 超过 idle 阈值的 Session
            .filter(|s| !s.is_stale(now, DEFAULT_SESSION_IDLE_SECS))
            .cloned()
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.updated_at));
        Ok(out)
    }

    async fn list_participants(
        &self,
        q: ListParticipantsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<PresenceParticipant>, CollaborationError> {
        // 租户隔离 + Session 存在
        let _session = self.fetch_session(q.session_id, &viewer)?;
        if viewer.tenant_id != q.tenant_id {
            return Err(CollaborationError::PermissionDenied);
        }
        let store = self.participants.read().expect("lock");
        let now = Utc::now();
        let mut out: Vec<PresenceParticipant> = store
            .values()
            .filter(|p| p.session_id == q.session_id)
            .filter(|p| p.tenant_id == q.tenant_id)
            .filter(|p| match q.status_filter {
                Some(s) => p.status == s,
                None => true,
            })
            // 仅返回未 stale 的 Participant(stale 由 query 调用方通过 list 决定)
            .filter(|p| !p.is_stale(now, DEFAULT_HEARTBEAT_TIMEOUT_SECS))
            .cloned()
            .collect();
        out.sort_by_key(|a| a.joined_at);
        Ok(out)
    }

    async fn get_cursor(
        &self,
        q: GetCursorQuery,
        viewer: ActorContext,
    ) -> Result<Option<PresenceCursor>, CollaborationError> {
        let _session = self.fetch_session(q.session_id, &viewer)?;
        if viewer.tenant_id != q.tenant_id {
            return Err(CollaborationError::PermissionDenied);
        }
        let store = self.cursors.read().expect("lock");
        Ok(store.get(&q.participant_id).cloned())
    }
}

// =====================================================================
// CollaborationRepository 实现
// =====================================================================

#[async_trait]
impl CollaborationRepository for InMemoryCollaborationService {
    async fn insert_session(
        &self,
        s: &CollaborationSession,
    ) -> Result<(), CollaborationError> {
        self.sessions.write().expect("lock").insert(s.id, s.clone());
        Ok(())
    }

    async fn find_session(
        &self,
        id: SessionId,
    ) -> Result<Option<CollaborationSession>, CollaborationError> {
        Ok(self.sessions.read().expect("lock").get(&id).cloned())
    }

    async fn list_sessions_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<CollaborationSession>, CollaborationError> {
        let store = self.sessions.read().expect("lock");
        Ok(store
            .values()
            .filter(|s| s.tenant_id == tenant_id && s.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn delete_session(&self, id: SessionId) -> Result<(), CollaborationError> {
        self.sessions.write().expect("lock").remove(&id);
        Ok(())
    }

    async fn insert_participant(
        &self,
        p: &PresenceParticipant,
    ) -> Result<(), CollaborationError> {
        self.participants
            .write()
            .expect("lock")
            .insert(p.id, p.clone());
        Ok(())
    }

    async fn find_participant(
        &self,
        id: ParticipantId,
    ) -> Result<Option<PresenceParticipant>, CollaborationError> {
        Ok(self.participants.read().expect("lock").get(&id).cloned())
    }

    async fn list_participants_by_session(
        &self,
        tenant_id: TenantId,
        session_id: SessionId,
    ) -> Result<Vec<PresenceParticipant>, CollaborationError> {
        let store = self.participants.read().expect("lock");
        Ok(store
            .values()
            .filter(|p| p.tenant_id == tenant_id && p.session_id == session_id)
            .cloned()
            .collect())
    }

    async fn update_participant(
        &self,
        p: &PresenceParticipant,
    ) -> Result<(), CollaborationError> {
        self.participants
            .write()
            .expect("lock")
            .insert(p.id, p.clone());
        Ok(())
    }

    async fn delete_participant(
        &self,
        id: ParticipantId,
    ) -> Result<(), CollaborationError> {
        self.participants.write().expect("lock").remove(&id);
        Ok(())
    }

    async fn upsert_cursor(&self, c: &PresenceCursor) -> Result<(), CollaborationError> {
        self.cursors
            .write()
            .expect("lock")
            .insert(c.participant_id, c.clone());
        Ok(())
    }

    async fn find_cursor(
        &self,
        _session_id: SessionId,
        participant_id: ParticipantId,
    ) -> Result<Option<PresenceCursor>, CollaborationError> {
        Ok(self.cursors.read().expect("lock").get(&participant_id).cloned())
    }

    async fn delete_cursors_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<(), CollaborationError> {
        // 找出 Session 下所有 Participant 再删 Cursor
        let pids: Vec<ParticipantId> = {
            let participants = self.participants.read().expect("lock");
            participants
                .values()
                .filter(|p| p.session_id == session_id)
                .map(|p| p.id)
                .collect()
        };
        let mut cursors = self.cursors.write().expect("lock");
        for pid in pids {
            cursors.remove(&pid);
        }
        Ok(())
    }

    async fn insert_channel(&self, c: &RealtimeChannel) -> Result<(), CollaborationError> {
        // INV-CB-02:单 Connection Channel 限流
        let conn_channels = self.connection_channels.read().expect("lock");
        let count = conn_channels
            .get(&c.user_id.into_uuid())
            .map(|s| s.len())
            .unwrap_or(0);
        drop(conn_channels);
        if count >= MAX_CHANNELS_PER_CONNECTION {
            return Err(CollaborationError::RateLimited(format!(
                "INV-CB-02: Connection 已达 {count} 个 Channel 上限"
            )));
        }
        // INV-CB-05:filter.resource_types 非空
        check_invariant_05_channel_filter_not_empty(c)?;

        self.channels
            .write()
            .expect("lock")
            .insert(c.id, c.clone());
        self.connection_channels
            .write()
            .expect("lock")
            .entry(c.user_id.into_uuid())
            .or_default()
            .insert(c.id);
        Ok(())
    }

    async fn list_channels_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<RealtimeChannel>, CollaborationError> {
        let store = self.channels.read().expect("lock");
        Ok(store
            .values()
            .filter(|c| c.session_id == session_id)
            .cloned()
            .collect())
    }

    async fn update_channel(&self, c: &RealtimeChannel) -> Result<(), CollaborationError> {
        self.channels
            .write()
            .expect("lock")
            .insert(c.id, c.clone());
        Ok(())
    }

    async fn delete_channel(&self, id: ChannelId) -> Result<(), CollaborationError> {
        self.channels.write().expect("lock").remove(&id);
        // 从 connection_channels 索引移除
        let mut conn_channels = self.connection_channels.write().expect("lock");
        for set in conn_channels.values_mut() {
            set.remove(&id);
        }
        Ok(())
    }
}

// =====================================================================
// RealtimeEventRouter 实现
// =====================================================================

#[async_trait]
impl RealtimeEventRouter for InMemoryCollaborationService {
    async fn route(
        &self,
        _event_type: &str,
        event_id: uuid::Uuid,
        event_tenant_id: TenantId,
        _event_project_id: ProjectId,
        _event_resource_type: ResourceType,
    ) -> Result<usize, CollaborationError> {
        // INV-CB-04:跨 tenant 拒绝
        let channels = self.channels.read().expect("lock");
        let mut delivered = 0usize;
        for c in channels.values() {
            // 跨 tenant 拒绝
            if c.tenant_id != event_tenant_id {
                continue;
            }
            if !c.is_active {
                continue;
            }
            // INV-CB-05:filter 已在 insert 时强制非空
            // 简化:仅当 filter 包含通配时路由;MVP 阶段 channel 暂不主动路由
            // (实际由 WS Gateway 在持有 channel 后,基于事件类型匹配推送)。
            delivered += 1;
        }
        // 累加测试计数器
        self.route_counter
            .write()
            .expect("lock")
            .insert(event_id, delivered);
        Ok(delivered)
    }
}

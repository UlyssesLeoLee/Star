//! Collaboration 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/specs/domain-collaboration-spec.md` §4(接口签名)
//! - `docs/api-design.md` §3.18(Presence) / §4.2(WS 端点)
//!
//! **端口清单**:
//! - `CollaborationCommandPort`:6 方法(写) — open_session / join_session /
//!   leave_session / heartbeat / update_cursor / close_session
//! - `CollaborationQueryPort`:4 方法(读) — get_session / list_active_sessions /
//!   list_participants / get_cursor
//! - `CollaborationRepository`:基础设施层使用,本文件声明 trait
//! - `RealtimeEventRouter`:1 方法(订阅 + 路由),spec §4

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{
    CollaborationSession, PresenceCursor, PresenceParticipant, RealtimeChannel,
};
use crate::error::CollaborationError;
use crate::value_object::{
    ChannelId, ParticipantId, ParticipantStatus, ProjectId, ResourceType, SelectionShape,
    SessionId, TenantId, UserId,
};

// =====================================================================
// 命令 DTO(写操作输入)
// =====================================================================

/// `OpenSessionCommand`(创建协作会话)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSessionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID(Session 必带项目范围)
    pub project_id: ProjectId,
    /// 可选 Workspace 挂载
    pub workspace_id: Option<crate::value_object::WorkspaceId>,
    /// Session 名
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 是否开放加入
    pub is_open: bool,
}

/// `JoinSessionCommand`(参与者加入会话)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinSessionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
    /// 加入用户的 User ID
    pub user_id: UserId,
    /// 初始资源类型
    pub resource_type: Option<String>,
    /// 初始资源 ID
    pub resource_id: Option<uuid::Uuid>,
}

/// `LeaveSessionCommand`(参与者离开会话)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveSessionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
    /// 离开的 Participant ID
    pub participant_id: ParticipantId,
}

/// `HeartbeatCommand`(心跳上报)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
    /// 心跳的 Participant ID
    pub participant_id: ParticipantId,
    /// 客户端当前时间(用于 RTT / 时钟漂移检测)
    pub client_now: Option<DateTime<Utc>>,
}

/// `UpdateCursorCommand`(光标 / 选区更新)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCursorCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
    /// Participant ID
    pub participant_id: ParticipantId,
    /// 资源类型
    pub resource_type: String,
    /// 资源 ID
    pub resource_id: uuid::Uuid,
    /// X / 行
    pub position_x: i32,
    /// Y / 列
    pub position_y: i32,
    /// 选区起点
    pub selection_start: Option<i32>,
    /// 选区终点
    pub selection_end: Option<i32>,
    /// 选区形状
    pub selection_shape: SelectionShape,
}

/// `CloseSessionCommand`(关闭 / 销毁协作会话)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseSessionCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListActiveSessionsQuery`(列出活跃会话)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActiveSessionsQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 仅返回指定 owner 的 Session(None = 全部)
    pub owner_user_id: Option<UserId>,
}

/// `ListParticipantsQuery`(列出 Session 参与者)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParticipantsQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
    /// 可选状态过滤
    pub status_filter: Option<ParticipantStatus>,
}

/// `GetCursorQuery`(查询指定 Participant 的光标)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCursorQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Session ID
    pub session_id: SessionId,
    /// Participant ID
    pub participant_id: ParticipantId,
}

// =====================================================================
// 端口:CollaborationCommandPort(6 方法)
// =====================================================================

/// **Collaboration 命令端口**(写操作 6 方法)
#[async_trait]
pub trait CollaborationCommandPort: Send + Sync {
    /// **打开协作会话**(open_session)
    /// INV-CB-01:必带 tenant_id,跨 tenant 拒绝。
    async fn open_session(
        &self,
        cmd: OpenSessionCommand,
        actor: ActorContext,
    ) -> Result<CollaborationSession, CollaborationError>;

    /// **加入协作会话**(join_session,自动创建 Participant)
    /// INV-CB-03:心跳默认 60s 过期,Participant 创建时 heartbeat_expires_at = now + 60s。
    async fn join_session(
        &self,
        cmd: JoinSessionCommand,
        actor: ActorContext,
    ) -> Result<PresenceParticipant, CollaborationError>;

    /// **离开协作会话**(leave_session,移除 Participant)
    async fn leave_session(
        &self,
        cmd: LeaveSessionCommand,
        actor: ActorContext,
    ) -> Result<(), CollaborationError>;

    /// **心跳上报**(heartbeat)
    /// INV-CB-03:心跳刷新 last_active_at + heartbeat_expires_at。
    async fn heartbeat(
        &self,
        cmd: HeartbeatCommand,
        actor: ActorContext,
    ) -> Result<PresenceParticipant, CollaborationError>;

    /// **更新光标 / 选区**(update_cursor)
    async fn update_cursor(
        &self,
        cmd: UpdateCursorCommand,
        actor: ActorContext,
    ) -> Result<PresenceCursor, CollaborationError>;

    /// **关闭协作会话**(close_session,销毁 Session + 全部 Participant / Cursor)
    async fn close_session(
        &self,
        cmd: CloseSessionCommand,
        actor: ActorContext,
    ) -> Result<(), CollaborationError>;
}

// =====================================================================
// 端口:CollaborationQueryPort(4 方法)
// =====================================================================

/// **Collaboration 查询端口**(读操作 4 方法)
#[async_trait]
pub trait CollaborationQueryPort: Send + Sync {
    /// 按 ID 查询 Session
    async fn get_session(
        &self,
        id: SessionId,
        viewer: ActorContext,
    ) -> Result<CollaborationSession, CollaborationError>;

    /// 列出 Project 下活跃 Session
    async fn list_active_sessions(
        &self,
        q: ListActiveSessionsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<CollaborationSession>, CollaborationError>;

    /// 列出 Session 全部 Participant(可按状态过滤)
    async fn list_participants(
        &self,
        q: ListParticipantsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<PresenceParticipant>, CollaborationError>;

    /// 查询指定 Participant 的当前 Cursor(0..1)
    async fn get_cursor(
        &self,
        q: GetCursorQuery,
        viewer: ActorContext,
    ) -> Result<Option<PresenceCursor>, CollaborationError>;
}

// =====================================================================
// 端口:CollaborationRepository(基础设施层适配)
// =====================================================================

/// **Collaboration 仓库端口**(供 SQLx / 内存 / 测试 Adapter 实现)
#[async_trait]
pub trait CollaborationRepository: Send + Sync {
    // Session
    /// 插入 Session
    async fn insert_session(
        &self,
        s: &CollaborationSession,
    ) -> Result<(), CollaborationError>;
    /// 按 ID 读 Session
    async fn find_session(
        &self,
        id: SessionId,
    ) -> Result<Option<CollaborationSession>, CollaborationError>;
    /// 列出 Project 下 Session
    async fn list_sessions_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<CollaborationSession>, CollaborationError>;
    /// 删除 Session
    async fn delete_session(&self, id: SessionId) -> Result<(), CollaborationError>;

    // Participant
    /// 插入 Participant
    async fn insert_participant(
        &self,
        p: &PresenceParticipant,
    ) -> Result<(), CollaborationError>;
    /// 按 ID 读 Participant
    async fn find_participant(
        &self,
        id: ParticipantId,
    ) -> Result<Option<PresenceParticipant>, CollaborationError>;
    /// 列出 Session 下 Participant
    async fn list_participants_by_session(
        &self,
        tenant_id: TenantId,
        session_id: SessionId,
    ) -> Result<Vec<PresenceParticipant>, CollaborationError>;
    /// 更新 Participant(乐观锁)
    async fn update_participant(
        &self,
        p: &PresenceParticipant,
    ) -> Result<(), CollaborationError>;
    /// 删除 Participant
    async fn delete_participant(
        &self,
        id: ParticipantId,
    ) -> Result<(), CollaborationError>;

    // Cursor
    /// 插入或更新 Cursor(同一 Participant 仅保留最新)
    async fn upsert_cursor(&self, c: &PresenceCursor) -> Result<(), CollaborationError>;
    /// 查询 Participant 当前 Cursor
    async fn find_cursor(
        &self,
        session_id: SessionId,
        participant_id: ParticipantId,
    ) -> Result<Option<PresenceCursor>, CollaborationError>;
    /// 删除 Session 下全部 Cursor
    async fn delete_cursors_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<(), CollaborationError>;

    // Channel
    /// 插入 Channel
    async fn insert_channel(&self, c: &RealtimeChannel) -> Result<(), CollaborationError>;
    /// 列出 Session 下 Channel
    async fn list_channels_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<RealtimeChannel>, CollaborationError>;
    /// 更新 Channel
    async fn update_channel(&self, c: &RealtimeChannel) -> Result<(), CollaborationError>;
    /// 删除 Channel
    async fn delete_channel(&self, id: ChannelId) -> Result<(), CollaborationError>;
}

// =====================================================================
// 端口:RealtimeEventRouter(spec §4 内部事件路由)
// =====================================================================

/// **Realtime 事件路由器**(spec §4 `RealtimeEventRouter`)
///
/// 内部从 NATS / Domain Event 接收事件,按 Subscription.filter 匹配,推送给对应 WS Channel。
/// MVP 阶段由 `InMemoryCollaborationService` 提供简单实现。
#[async_trait]
pub trait RealtimeEventRouter: Send + Sync {
    /// 路由一个 Domain Event 到匹配的全部 Channel,返回投递计数。
    async fn route(
        &self,
        event_type: &str,
        event_id: uuid::Uuid,
        event_tenant_id: TenantId,
        event_project_id: ProjectId,
        event_resource_type: ResourceType,
    ) -> Result<usize, CollaborationError>;
}

//! Collaboration 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.collaboration.*`
//!
//! **本 crate 事件清单**(spec §5 + parent task 协作投影):
//! 1. `SessionOpened` — `star.events.collaboration.session.opened.v1`
//! 2. `SessionClosed` — `star.events.collaboration.session.closed.v1`
//! 3. `ParticipantJoined` — `star.events.collaboration.participant.joined.v1`
//! 4. `ParticipantLeft` — `star.events.collaboration.participant.left.v1`
//! 5. `HeartbeatReceived` — `star.events.collaboration.participant.heartbeat.v1`
//! 6. `CursorMoved` — `star.events.collaboration.cursor.moved.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ParticipantId, ParticipantStatus, ProjectId, ResourceType, SelectionShape, SessionId,
    TenantId, UserId,
};

/// 事件通用元数据(所有 Domain Event 共享的最小字段集,CloudEvents 1.0 envelope 简版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// CloudEvents `id`(UUID v4)
    pub event_id: uuid::Uuid,
    /// CloudEvents `source`(常量 `star.domain.collaboration`)
    pub source: String,
    /// CloudEvents `time`
    pub occurred_at: DateTime<Utc>,
    /// 租户 ID(spec INV-CB-04 必带)
    pub tenant_id: TenantId,
    /// 触发者
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    /// 构造一个 `EventMeta`(便于测试 / 命令 impl 中调用)。
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            source: "star.domain.collaboration".to_string(),
            occurred_at: Utc::now(),
            tenant_id,
            actor_user_id: None,
        }
    }
}

/// `SessionOpened` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOpened {
    /// 事件元数据
    pub meta: EventMeta,
    /// 新建 Session ID
    pub session_id: SessionId,
    /// Project ID
    pub project_id: ProjectId,
    /// Session 名
    pub name: String,
    /// 是否开放加入
    pub is_open: bool,
}

/// `SessionClosed` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClosed {
    /// 事件元数据
    pub meta: EventMeta,
    /// 关闭的 Session ID
    pub session_id: SessionId,
}

/// `ParticipantJoined` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantJoined {
    /// 事件元数据
    pub meta: EventMeta,
    /// Session ID
    pub session_id: SessionId,
    /// Participant ID
    pub participant_id: ParticipantId,
    /// 加入者 User ID
    pub user_id: UserId,
    /// 初始资源类型
    pub resource_type: Option<String>,
}

/// `ParticipantLeft` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantLeft {
    /// 事件元数据
    pub meta: EventMeta,
    /// Session ID
    pub session_id: SessionId,
    /// 离开的 Participant ID
    pub participant_id: ParticipantId,
}

/// `HeartbeatReceived` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatReceived {
    /// 事件元数据
    pub meta: EventMeta,
    /// Session ID
    pub session_id: SessionId,
    /// Participant ID
    pub participant_id: ParticipantId,
    /// 心跳刷新后的 last_active_at
    pub last_active_at: DateTime<Utc>,
    /// 当前状态
    pub status: ParticipantStatus,
}

/// `CursorMoved` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorMoved {
    /// 事件元数据
    pub meta: EventMeta,
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

// =====================================================================
// 枚举:全部 Collaboration 域事件
// =====================================================================

/// 全部 Collaboration 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollaborationEvent {
    /// Session 打开
    SessionOpened(SessionOpened),
    /// Session 关闭
    SessionClosed(SessionClosed),
    /// 参与者加入
    ParticipantJoined(ParticipantJoined),
    /// 参与者离开
    ParticipantLeft(ParticipantLeft),
    /// 心跳收到
    HeartbeatReceived(HeartbeatReceived),
    /// 光标移动
    CursorMoved(CursorMoved),
}

impl CollaborationEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::SessionOpened(_) => "star.events.collaboration.session.opened.v1",
            Self::SessionClosed(_) => "star.events.collaboration.session.closed.v1",
            Self::ParticipantJoined(_) => "star.events.collaboration.participant.joined.v1",
            Self::ParticipantLeft(_) => "star.events.collaboration.participant.left.v1",
            Self::HeartbeatReceived(_) => "star.events.collaboration.participant.heartbeat.v1",
            Self::CursorMoved(_) => "star.events.collaboration.cursor.moved.v1",
        }
    }

    /// 事件的 tenant_id(从 meta 提取)
    pub fn tenant_id(&self) -> TenantId {
        match self {
            Self::SessionOpened(e) => e.meta.tenant_id,
            Self::SessionClosed(e) => e.meta.tenant_id,
            Self::ParticipantJoined(e) => e.meta.tenant_id,
            Self::ParticipantLeft(e) => e.meta.tenant_id,
            Self::HeartbeatReceived(e) => e.meta.tenant_id,
            Self::CursorMoved(e) => e.meta.tenant_id,
        }
    }

    /// 事件关联的 Project ID(用于路由过滤)
    pub fn project_id(&self) -> ProjectId {
        match self {
            Self::SessionOpened(e) => e.project_id,
            Self::SessionClosed(_) => ProjectId::new(), // 关闭事件无 project_id,路由需用 Channel 的 session 上下文
            Self::ParticipantJoined(_) => ProjectId::new(), // 由 session 上下文填充
            Self::ParticipantLeft(_) => ProjectId::new(),
            Self::HeartbeatReceived(_) => ProjectId::new(),
            Self::CursorMoved(_) => ProjectId::new(),
        }
    }

    /// 用于 `RealtimeEventRouter` 路由的 ResourceType 推断
    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::SessionOpened(_) | Self::SessionClosed(_) => ResourceType::Presence,
            Self::ParticipantJoined(_)
            | Self::ParticipantLeft(_)
            | Self::HeartbeatReceived(_)
            | Self::CursorMoved(_) => ResourceType::Presence,
        }
    }
}

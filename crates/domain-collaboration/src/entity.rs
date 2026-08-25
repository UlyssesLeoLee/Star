//! Collaboration 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.17 (`collaboration` schema)
//! - `docs/specs/domain-collaboration-spec.md` §2 (实体清单)
//!
//! 包含 1 个核心聚合根 + 3 个子实体:
//! - `CollaborationSession` — 协作会话聚合根(11 字段)
//! - `PresenceParticipant` — 参与者在线状态(11 字段,内嵌 spec §2 `Presence`)
//! - `PresenceCursor` — 光标 / 选区位置(10 字段,parent task 专属)
//! - `RealtimeChannel` — WS 通道标识(10 字段,对应 spec §2 `RealtimeSubscription`)
//!
//! **合并设计说明**(spec vs parent task):
//! spec 把 `Presence` / `RealtimeSubscription` / `RealtimeEventPayload` 设计为 3 个并列实体;
//! parent task 把协作抽象为 `CollaborationSession` 内含 Participant / Cursor / Channel。
//! 本 crate 采用 **parent task 优先**:`CollaborationSession` 聚合根统一管理
//! 参与者列表 + 光标集合 + 通道集合;spec 的 `Presence` / `RealtimeSubscription`
//! 语义以 `PresenceParticipant.status` / `RealtimeChannel.filter` 形式承载。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ChannelId, ParticipantId, ParticipantStatus, ProjectId, SelectionShape, SessionId, TenantId,
    UserId, WorkspaceId,
};

// =====================================================================
// PresenceCursor 子实体(光标 / 选区)
// =====================================================================

/// **PresenceCursor**(协作光标 + 选区,10 字段)
///
/// 来源: parent task 协作投影 + spec §2 `Presence.resource_id` 范围。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceCursor {
    /// 主键
    pub id: uuid::Uuid,
    /// 所属 Session
    pub session_id: SessionId,
    /// 所属 Participant
    pub participant_id: ParticipantId,
    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,
    /// 资源类型(标识光标位于哪类资源上)
    pub resource_type: String,
    /// 资源 ID
    pub resource_id: uuid::Uuid,
    /// 行 / X 坐标
    pub position_x: i32,
    /// 列 / Y 坐标
    pub position_y: i32,
    /// 选区起点(仅 Range / Block 有意义)
    pub selection_start: Option<i32>,
    /// 选区终点
    pub selection_end: Option<i32>,
    /// 选区形状
    pub selection_shape: SelectionShape,
    /// 更新时间(心跳维度)
    pub updated_at: DateTime<Utc>,
}

impl PresenceCursor {
    /// 字段数(用于 §4.17 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 12;

    /// 升级乐观锁版本号(由 service 显式调用,本字段由 updated_at 承载)
    pub fn bump_version(&mut self) {
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// PresenceParticipant 子实体(spec §2 Presence 内嵌)
// =====================================================================

/// **PresenceParticipant**(参与者在线状态,11 字段)
///
/// 来源: docs/data-design.md §4.17.1 (`collaboration.presence` 表) +
/// spec §2 `Presence` 实体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceParticipant {
    /// 主键
    pub id: ParticipantId,
    /// 所属 Session
    pub session_id: SessionId,
    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,
    /// 关联 Project
    pub project_id: ProjectId,
    /// 用户 ID
    pub user_id: UserId,
    /// 状态
    pub status: ParticipantStatus,
    /// 当前正在查看的资源类型(`worktree` / `agent_session` / `feedback` ...)
    pub resource_type: Option<String>,
    /// 当前正在查看的资源 ID
    pub resource_id: Option<uuid::Uuid>,
    /// 最近一次心跳时间
    pub last_active_at: DateTime<Utc>,
    /// 心跳过期时间(默认 last_active_at + 60s,INV-CB-03)
    pub heartbeat_expires_at: DateTime<Utc>,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
}

impl PresenceParticipant {
    /// 字段数(用于 §4.17.1 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 11;

    /// 升级心跳时间(同时重算过期时间)
    pub fn bump_version(&mut self) {
        self.last_active_at = Utc::now();
        self.heartbeat_expires_at = self.last_active_at + Duration::seconds(60);
    }

    /// **是否 stale**(INV-CB-03:心跳 60s 未到 → Offline)
    ///
    /// `now` 由调用方注入(便于测试固定时间)。
    pub fn is_stale(&self, now: DateTime<Utc>, heartbeat_timeout_secs: i64) -> bool {
        let elapsed = now.signed_duration_since(self.last_active_at);
        elapsed.num_seconds() > heartbeat_timeout_secs
    }
}

// =====================================================================
// RealtimeChannel 子实体(spec §2 RealtimeSubscription 内嵌)
// =====================================================================

/// **RealtimeChannel**(WS 通道 + Subscription 过滤,10 字段)
///
/// 来源: docs/data-design.md §4.17.2 (`collaboration.realtime_subscription`) +
/// spec §2 `RealtimeSubscription` + api-design §4.4 过滤维度。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeChannel {
    /// 主键
    pub id: ChannelId,
    /// 所属 Session
    pub session_id: SessionId,
    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,
    /// 用户 ID(Connection 拥有者)
    pub user_id: UserId,
    /// 过滤订阅的资源类型列表
    pub filter_resource_types: Vec<crate::value_object::ResourceType>,
    /// 过滤订阅的项目(空 = JWT 全部可访问 Project)
    pub filter_project_ids: Vec<ProjectId>,
    /// 续传用 last_event_id
    pub last_event_id: Option<uuid::Uuid>,
    /// 是否活跃
    pub is_active: bool,
    /// 过期时间(7 天无活跃,api-design §4.2)
    pub expires_at: DateTime<Utc>,
    /// 最近一次 ping 时间
    pub last_ping_at: DateTime<Utc>,
}

impl RealtimeChannel {
    /// 字段数(用于 §4.17.2 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 10;

    /// 升级版本号(更新 ping 时间)
    pub fn bump_version(&mut self) {
        self.last_ping_at = Utc::now();
    }

    /// **是否 stale**(Channel 过期检测,api-design §4.2:7 天无活跃)
    pub fn is_stale(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }
}

// =====================================================================
// CollaborationSession 聚合根
// =====================================================================

/// **CollaborationSession**(协作会话聚合根,11 字段)
///
/// 来源: parent task 协作投影 + spec §1 职责说明。
///
/// **聚合关系**:
/// - 1 个 Session → N 个 Participant (1对多)
/// - 1 个 Participant → 0..1 个 Cursor (1对0..1)
/// - 1 个 Session → N 个 RealtimeChannel (1对多)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// 主键
    pub id: SessionId,
    /// 租户 ID(必带,§6.1, INV-CB-01)
    pub tenant_id: TenantId,
    /// Project ID(必带,Participant 共享)
    pub project_id: ProjectId,
    /// Workspace ID(可选挂载)
    pub workspace_id: Option<WorkspaceId>,
    /// Session 名(便于人读 / 调试)
    pub name: String,
    /// Session 描述
    pub description: Option<String>,
    /// 拥有者(创建者)User ID
    pub owner_user_id: UserId,
    /// 是否开放(开放 = 任何同 Project User 可加入;关闭 = 仅 owner 邀请)
    pub is_open: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本号
    pub lock_version: u32,
}

impl CollaborationSession {
    /// 字段数(用于聚合根 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 11;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// **是否 stale**(Session 视角:无活跃 Participant 且超过 idle 阈值)
    ///
    /// 简化语义:`updated_at` 距今超过传入的 idle 阈值秒数则视为 stale。
    pub fn is_stale(&self, now: DateTime<Utc>, idle_threshold_secs: i64) -> bool {
        let elapsed = now.signed_duration_since(self.updated_at);
        elapsed.num_seconds() > idle_threshold_secs
    }
}

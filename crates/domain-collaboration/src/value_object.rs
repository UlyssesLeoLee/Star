//! Collaboration 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.17 (`collaboration` schema)
//! - `docs/specs/domain-collaboration-spec.md` §2 (实体清单) / §3 (基本类型)
//!
//! 集中放置强类型 ID、Participant 状态、Resource 类型、Cursor 选区等。

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

// 协作会话 ID(聚合根主键)
define_uuid_id!(SessionId);

// 参与者 ID(Session 内的子实体)
define_uuid_id!(ParticipantId);

// WS / Realtime 通道 ID(Subscription 对齐 spec §2 `RealtimeSubscription`)
define_uuid_id!(ChannelId);

// 强类型 Tenant ID(避免依赖 domain-tenant)
define_uuid_id!(TenantId);

// 强类型 User ID
define_uuid_id!(UserId);

// Workspace ID(Session 可选挂载的工作区)
define_uuid_id!(WorkspaceId);

// Project ID(Session 必带的项目范围,§3 INV-CB-01 跨 tenant 拒绝)
define_uuid_id!(ProjectId);

// =====================================================================
// 枚举:ParticipantStatus(对应 data-design `ck_presence_status`)
// =====================================================================

/// **参与者在线状态**(`collaboration.presence.status` 列)
///
/// 来源: docs/data-design.md §4.17.1(`ck_presence_status` 约束),
/// docs/specs/domain-collaboration-spec.md §2 (`ONLINE / AWAY / OFFLINE`)。
///
/// 4 状态机: ACTIVE(本规范扩展,等价于 ONLINE) / IDLE(轻量闲置) /
/// AWAY(主动离开 / 5min 心跳过期) / OFFLINE(60s 心跳过期,INV-CB-03)。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParticipantStatus {
    /// 活跃(本规范扩展,等价 spec ONLINE)
    #[default]
    Active,
    /// 轻量闲置(前端仍可见 tab 但无活动)
    Idle,
    /// 主动离开 / 心跳 ≤ 5min 未更新
    Away,
    /// 离线(心跳 60s 未到,INV-CB-03)
    Offline,
}

impl std::fmt::Display for ParticipantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "ACTIVE",
            Self::Idle => "IDLE",
            Self::Away => "AWAY",
            Self::Offline => "OFFLINE",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:ResourceType(Subscription 过滤维度,api-design §4.4)
// =====================================================================

/// **订阅资源类型**(`realtime_subscription.filter.resource_types`)
///
/// 来源: docs/api-design.md §4.4: `worktree / agent_session / validation_result /
/// feedback / runtime / presence`。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// Worktree 状态变更
    Worktree,
    /// AgentSession 状态
    AgentSession,
    /// ValidationResult
    ValidationResult,
    /// Feedback
    Feedback,
    /// Local Runtime 上线下线
    Runtime,
    /// Presence 在线状态
    Presence,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Worktree => "worktree",
            Self::AgentSession => "agent_session",
            Self::ValidationResult => "validation_result",
            Self::Feedback => "feedback",
            Self::Runtime => "runtime",
            Self::Presence => "presence",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:SelectionShape(Cursor 选区形状)
// =====================================================================

/// **Cursor 选区形状**(光标 / 选区表示)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SelectionShape {
    /// 单点光标(无选区)
    #[default]
    Point,
    /// 线性选区(行内,例如文本)
    Range,
    /// 矩形选区(例如表格 / 网格)
    Block,
}

// =====================================================================
// 标准角色 / Permission 字符串
// =====================================================================

/// Collaboration 相关标准角色 / Permission 字符串常量
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 平台运营
    pub const PLATFORM_OPERATOR: &str = "platform_operator";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读观察者
    pub const VIEWER: &str = "viewer";
}

/// 权限字符串(spec §7)
pub mod permissions {
    /// 实时订阅权限
    pub const REALTIME_SUBSCRIBE: &str = "realtime:subscribe";
    /// Presence 读取权限
    pub const PRESENCE_READ: &str = "presence:read";
}

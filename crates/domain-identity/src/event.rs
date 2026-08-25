//! Identity 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.identity.*`
//!
//! **本 crate 事件清单**:
//! 1. `UserCreated` — `star.events.identity.user.created.v1`
//! 2. `UserLoggedIn` — `star.events.identity.user.logged_in.v1`
//! 3. `DeviceBound` — `star.events.identity.device.bound.v1`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{DeviceId, DeviceType, TenantId, UserId};

/// 事件通用元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件唯一 ID
    pub event_id: uuid::Uuid,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者
    pub actor_user_id: Option<UserId>,
}

impl EventMeta {
    /// 构造一个 `EventMeta`
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// `UserCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// 新建 User ID
    pub user_id: UserId,
    /// 邮箱
    pub email: String,
    /// 显示名
    pub display_name: String,
}

/// `UserLoggedIn` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    /// 事件元数据
    pub meta: EventMeta,
    /// User ID
    pub user_id: UserId,
    /// 登录设备 ID
    pub device_id: DeviceId,
    /// 设备类型
    pub device_type: DeviceType,
    /// 登录时间
    pub logged_in_at: DateTime<Utc>,
}

/// `DeviceBound` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBound {
    /// 事件元数据
    pub meta: EventMeta,
    /// Device ID
    pub device_id: DeviceId,
    /// User ID
    pub user_id: UserId,
    /// 设备指纹
    pub device_fingerprint: String,
}

/// 全部 Identity 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IdentityEvent {
    /// User 创建
    UserCreated(UserCreated),
    /// User 登录
    UserLoggedIn(UserLoggedIn),
    /// 设备绑定
    DeviceBound(DeviceBound),
}

impl IdentityEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::UserCreated(_) => "star.events.identity.user.created.v1",
            Self::UserLoggedIn(_) => "star.events.identity.user.logged_in.v1",
            Self::DeviceBound(_) => "star.events.identity.device.bound.v1",
        }
    }
}

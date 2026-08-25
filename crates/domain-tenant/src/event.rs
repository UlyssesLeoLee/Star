//! Tenant 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.tenant.*`
//!
//! **本 crate 事件清单**:
//! 1. `TenantCreated` — `star.events.tenant.tenant.created.v1`
//! 2. `TenantStatusChanged` — `star.events.tenant.tenant.status_changed.v1`
//! 3. `TenantPolicyUpdated` — `star.events.tenant.policy.updated.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{TenantId, TenantPolicyId, TenantStatus, TenantTier};

/// 事件通用元数据(所有 Domain Event 共享的最小字段集)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件唯一 ID(UUID v4)
    pub event_id: uuid::Uuid,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    /// 构造一个 `EventMeta`(便于测试 / 命令 impl 中调用)。
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// `TenantCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// 新建 Tenant ID
    pub tenant_id: TenantId,
    /// 租户业务键
    pub tenant_key: String,
    /// 显示名称
    pub name: String,
    /// 服务等级
    pub tier: TenantTier,
}

/// `TenantStatusChanged` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStatusChanged {
    /// 事件元数据
    pub meta: EventMeta,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// 旧状态
    pub from_status: TenantStatus,
    /// 新状态
    pub to_status: TenantStatus,
}

/// `TenantPolicyUpdated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPolicyUpdated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// TenantPolicy ID
    pub policy_id: TenantPolicyId,
    /// 变更字段列表
    pub changed_fields: Vec<String>,
}

/// 全部 Tenant 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TenantEvent {
    /// Tenant 创建
    Created(TenantCreated),
    /// Tenant 状态变化
    StatusChanged(TenantStatusChanged),
    /// TenantPolicy 更新
    PolicyUpdated(TenantPolicyUpdated),
}

impl TenantEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.tenant.tenant.created.v1",
            Self::StatusChanged(_) => "star.events.tenant.tenant.status_changed.v1",
            Self::PolicyUpdated(_) => "star.events.tenant.policy.updated.v1",
        }
    }
}

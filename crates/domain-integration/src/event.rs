//! Integration 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.integration.*`
//!
//! **本 crate 事件清单**(spec §5):
//! 1. `IntegrationCreated` — `star.events.integration.integration.created.v1`
//! 2. `IntegrationStateChanged` — `star.events.integration.integration.state_changed.v1`
//! 3. `SyncTriggered` — `star.events.integration.sync.triggered.v1`
//! 4. `SyncCompleted` — `star.events.integration.sync.completed.v1`
//! 5. `SyncConflictDetected` — `star.events.integration.sync.conflict_detected.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ConflictStrategy, ExternalEntityId, ExternalSystemName, IntegrationId, IntegrationRelationType,
    IntegrationSource, IntegrationState, ProjectId, SyncOutcome, TenantId,
};

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
            event_id: UserId.new(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

// =====================================================================
// 事件载荷
// =====================================================================

/// `IntegrationCreated` 事件载荷(`create_integration` 成功)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Integration ID
    pub integration_id: IntegrationId,
    /// Project ID
    pub project_id: ProjectId,
    /// 源系统分类
    pub source: IntegrationSource,
    /// 关系类型
    pub relation_type: IntegrationRelationType,
    /// 外部系统名
    pub external_system_name: ExternalSystemName,
    /// 外部实体 ID
    pub external_id: ExternalEntityId,
}

/// `IntegrationStateChanged` 事件载荷(状态机迁移)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStateChanged {
    /// 事件元数据
    pub meta: EventMeta,
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 旧状态(字符串)
    pub from_state: String,
    /// 新状态
    pub to_state: IntegrationState,
}

/// `SyncTriggered` 事件载荷(`trigger_sync` 成功)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTriggered {
    /// 事件元数据
    pub meta: EventMeta,
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 关系类型(用于 worker 决定 sync 方向)
    pub relation_type: IntegrationRelationType,
    /// 是否为手动触发
    pub manual: bool,
}

/// `SyncCompleted` 事件载荷(Worker 同步完成)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCompleted {
    /// 事件元数据
    pub meta: EventMeta,
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 同步结果
    pub outcome: SyncOutcome,
    /// 同步时间
    pub synced_at: DateTime<Utc>,
    /// 冲突记录数
    pub conflict_count: u32,
}

/// `SyncConflictDetected` 事件载荷(ConflictStrategy 触发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflictDetected {
    /// 事件元数据
    pub meta: EventMeta,
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 冲突策略
    pub conflict_strategy: ConflictStrategy,
    /// 冲突摘要
    pub conflict_summary: String,
}

// =====================================================================
// 枚举:全部 Integration 域事件
// =====================================================================

/// 全部 Integration 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IntegrationEvent {
    /// Integration 创建
    IntegrationCreated(IntegrationCreated),
    /// Integration 状态变化
    IntegrationStateChanged(IntegrationStateChanged),
    /// 同步触发
    SyncTriggered(SyncTriggered),
    /// 同步完成
    SyncCompleted(SyncCompleted),
    /// 同步冲突检测
    SyncConflictDetected(SyncConflictDetected),
}

impl IntegrationEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::IntegrationCreated(_) => "star.events.integration.integration.created.v1",
            Self::IntegrationStateChanged(_) => {
                "star.events.integration.integration.state_changed.v1"
            }
            Self::SyncTriggered(_) => "star.events.integration.sync.triggered.v1",
            Self::SyncCompleted(_) => "star.events.integration.sync.completed.v1",
            Self::SyncConflictDetected(_) => "star.events.integration.sync.conflict_detected.v1",
        }
    }

    /// 事件的 tenant_id(便于订阅者按租户过滤)
    pub fn tenant_id(&self) -> TenantId {
        match self {
            Self::IntegrationCreated(e) => e.meta.tenant_id,
            Self::IntegrationStateChanged(e) => e.meta.tenant_id,
            Self::SyncTriggered(e) => e.meta.tenant_id,
            Self::SyncCompleted(e) => e.meta.tenant_id,
            Self::SyncConflictDetected(e) => e.meta.tenant_id,
        }
    }
}

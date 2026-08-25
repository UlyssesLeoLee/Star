//! Development 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.development.*`
//!
//! **本 crate 事件清单**(spec §5):
//! 1. `ExecutionCreated` — `star.events.development.execution.created.v1`
//! 2. `ChangeSetObserved` — `star.events.development.change_set.observed.v1`
//! 3. `RiskSignalDetected` — `star.events.development.risk_signal.detected.v1`
//! 4. `ExecutionClosed` — `star.events.development.execution.closed.v1`
//! 5. `SymbolIndexRefreshed` — `star.events.development.symbol_index.refreshed.v1`
//!
//! 事件传输由 `infrastructure` crate 中的 NATS / JetStream Adapter 负责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AgentSessionId, ChangeSetId, CommitId, ExecutionId, RepositoryId, RiskSeverity,
    RiskSignalKind, TenantId, WorktreeId,
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
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

// =====================================================================
// 事件载荷
// =====================================================================

/// `ExecutionCreated` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCreated {
    /// 事件元数据
    pub meta: EventMeta,
    /// Execution ID
    pub execution_id: ExecutionId,
    /// 关联 WorkItem
    pub work_item_id: uuid::Uuid,
    /// 关联 Repository
    pub repository_id: RepositoryId,
}

/// `ChangeSetObserved` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSetObserved {
    /// 事件元数据
    pub meta: EventMeta,
    /// ChangeSet ID
    pub change_set_id: ChangeSetId,
    /// 关联 Worktree
    pub worktree_id: WorktreeId,
    /// 关联 Agent Session
    pub agent_session_id: Option<AgentSessionId>,
    /// 关联 Commit
    pub commit_id: CommitId,
    /// 高严重度 Risk Signal 数量
    pub risk_signal_count: u32,
}

/// `RiskSignalDetected` 事件载荷(severity >= High 时触发,spec §5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSignalDetected {
    /// 事件元数据
    pub meta: EventMeta,
    /// ChangeSet ID
    pub change_set_id: ChangeSetId,
    /// Risk Signal 类型
    pub kind: RiskSignalKind,
    /// 严重度
    pub severity: RiskSeverity,
    /// 证据
    pub evidence: String,
}

/// `ExecutionClosed` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionClosed {
    /// 事件元数据
    pub meta: EventMeta,
    /// Execution ID
    pub execution_id: ExecutionId,
    /// 结束时间
    pub ended_at: DateTime<Utc>,
    /// ChangeSet 总数
    pub change_set_count: u32,
}

/// `SymbolIndexRefreshed` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIndexRefreshed {
    /// 事件元数据
    pub meta: EventMeta,
    /// Repository ID
    pub repository_id: RepositoryId,
    /// 刷新后版本号
    pub version: u32,
    /// 符号总数
    pub symbol_count: u32,
}

// =====================================================================
// 枚举:全部 Development 域事件
// =====================================================================

/// 全部 Development 域事件的枚举包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DevelopmentEvent {
    /// Execution 创建
    ExecutionCreated(ExecutionCreated),
    /// ChangeSet 观察
    ChangeSetObserved(ChangeSetObserved),
    /// Risk Signal 检测(severity >= High)
    RiskSignalDetected(RiskSignalDetected),
    /// Execution 关闭
    ExecutionClosed(ExecutionClosed),
    /// SymbolIndex 刷新
    SymbolIndexRefreshed(SymbolIndexRefreshed),
}

impl DevelopmentEvent {
    /// 事件的 CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::ExecutionCreated(_) => "star.events.development.execution.created.v1",
            Self::ChangeSetObserved(_) => "star.events.development.change_set.observed.v1",
            Self::RiskSignalDetected(_) => "star.events.development.risk_signal.detected.v1",
            Self::ExecutionClosed(_) => "star.events.development.execution.closed.v1",
            Self::SymbolIndexRefreshed(_) => "star.events.development.symbol_index.refreshed.v1",
        }
    }
}

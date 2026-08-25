//! Board 域事件(Domain Events,CloudEvents 1.0)
//!
//! 主题前缀: `star.events.board.*`
//!
//! **本 crate 事件清单**(spec §5):
//! 1. `BoardReplaced` — `star.events.board.board.replaced.v1`
//! 2. `BoardPatched` — `star.events.board.board.patched.v1`
//! 3. `ColumnReordered` — `star.events.board.column.reordered.v1`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{BoardId, ColumnId, ProjectId, TenantId, UserId};

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
    pub actor_user_id: Option<uuid::Uuid>,
}

impl EventMeta {
    /// 构造 `EventMeta`。
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// `BoardReplaced` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardReplaced {
    /// 事件元数据
    pub meta: EventMeta,
    /// Board ID
    pub board_id: BoardId,
    /// 关联 Project
    pub project_id: ProjectId,
    /// 新版本号
    pub version: u32,
}

/// `BoardPatched` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardPatched {
    /// 事件元数据
    pub meta: EventMeta,
    /// Board ID
    pub board_id: BoardId,
    /// 修改字段列表
    pub patched_fields: Vec<String>,
}

/// `ColumnReordered` 事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnReordered {
    /// 事件元数据
    pub meta: EventMeta,
    /// Board ID
    pub board_id: BoardId,
    /// 新顺序 column_ids
    pub new_order: Vec<ColumnId>,
    /// 触发者
    pub actor_user_id: UserId,
}

/// 全部 Board 域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BoardEvent {
    /// Board 整体替换
    Replaced(BoardReplaced),
    /// Board 部分更新
    Patched(BoardPatched),
    /// Column 顺序调整
    ColumnReordered(ColumnReordered),
}

impl BoardEvent {
    /// CloudEvents subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Replaced(_) => "star.events.board.board.replaced.v1",
            Self::Patched(_) => "star.events.board.board.patched.v1",
            Self::ColumnReordered(_) => "star.events.board.column.reordered.v1",
        }
    }
}

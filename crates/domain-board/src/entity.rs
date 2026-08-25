//! Board 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.6 (`board` schema)
//! - `docs/specs/domain-board-spec.md` §2 (实体清单)
//!
//! 包含 3 个核心实体:
//! - `Board` — 主聚合根
//! - `Column` — 看板列(映射 Workflow State)
//! - `Swimlane` — 泳道(按维度分组)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    BoardId, BoardType, ColumnId, GroupByField, ProjectId, StateId, SwimlaneId, TenantId, UserId,
};

// =====================================================================
// Board 聚合根
// =====================================================================

/// **Board 聚合根**(继承 `data-design §4.6` DDL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// 主键 UUID
    pub id: BoardId,

    /// 租户 ID(必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// Project ID(必带,INV-B-01)
    pub project_id: ProjectId,

    /// Board 类型(Kanban / Scrum)
    pub board_type: BoardType,

    /// Board 名称
    pub name: String,

    /// 描述
    pub description: Option<String>,

    /// 过滤:仅看某分配人的 WorkItem
    pub filter_assignee: Option<UserId>,

    /// 过滤:仅看某标签的 WorkItem
    pub filter_label: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 乐观锁版本
    pub lock_version: u32,
}

impl Board {
    /// 字段数
    pub const FIELD_COUNT: usize = 12;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Column 实体
// =====================================================================

/// **Column**(看板列,引用 Workflow State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// 主键
    pub id: ColumnId,

    /// Board ID
    pub board_id: BoardId,

    /// 租户 ID(必带)
    pub tenant_id: TenantId,

    /// 列名
    pub name: String,

    /// 引用的 Workflow State(INV-B-02 引用完整性)
    pub state_id: StateId,

    /// 显示顺序(INV-B-03 UNIQUE)
    pub display_order: u32,

    /// WIP 限制(可空,INV-B-05 软告警)
    pub wip_limit: Option<u32>,

    /// 列颜色
    pub display_color: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Column {
    /// 字段数
    pub const FIELD_COUNT: usize = 10;
}

// =====================================================================
// Swimlane 实体
// =====================================================================

/// **Swimlane**(泳道,按指定字段分组)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swimlane {
    /// 主键
    pub id: SwimlaneId,

    /// Board ID
    pub board_id: BoardId,

    /// 租户 ID
    pub tenant_id: TenantId,

    /// 名称(如 "By Assignee")
    pub name: String,

    /// group_by 维度(INV-B-04 仅 assignee/label/epic)
    pub group_by_field: GroupByField,

    /// 显示顺序
    pub display_order: u32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Swimlane {
    /// 字段数
    pub const FIELD_COUNT: usize = 8;
}

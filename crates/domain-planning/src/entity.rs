//! Planning 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.7 (`planning` schema)
//! - `docs/specs/domain-planning-spec.md` §2 (实体清单)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    BacklogId, BurndownSnapshotId, MilestoneId, ProjectId, RoadmapId, SprintId, SprintState,
    TenantId, UserId, WorkItemId,
};

// =====================================================================
// Sprint 聚合根
// =====================================================================

/// **Sprint 聚合根**(继承 `data-design §4.7` DDL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    /// 主键 UUID
    pub id: SprintId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// Sprint 名称
    pub name: String,
    /// Sprint 目标
    pub goal: Option<String>,
    /// 起始时间
    pub start_at: DateTime<Utc>,
    /// 结束时间
    pub end_at: DateTime<Utc>,
    /// 状态(Planning/Active/Closed,INV-PL-01 不可逆)
    pub state: SprintState,
    /// 关联 WorkItem IDs
    pub work_item_ids: Vec<WorkItemId>,
    /// 容量(故事点,可空)
    pub capacity_story_points: Option<u32>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 启动时间
    pub started_at: Option<DateTime<Utc>>,
    /// 关闭时间
    pub closed_at: Option<DateTime<Utc>>,
    /// 乐观锁版本
    pub lock_version: u32,
}

impl Sprint {
    /// 字段数
    pub const FIELD_COUNT: usize = 15;
    /// 升级乐观锁版本
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Backlog 聚合根(Project 1:1)
// =====================================================================

/// **Backlog 聚合根**(Project 1:1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlog {
    /// 主键
    pub id: BacklogId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 排序(WorkItem ID 数组,按 display_order 升序)
    pub work_item_order: Vec<WorkItemId>,
    /// 容量(可空)
    pub capacity: Option<u32>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本
    pub lock_version: u32,
}

impl Backlog {
    /// 字段数
    pub const FIELD_COUNT: usize = 8;
    /// 升级乐观锁版本
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// Roadmap 聚合根
// =====================================================================

/// **Roadmap 聚合根**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roadmap {
    /// 主键
    pub id: RoadmapId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// Milestone IDs
    pub milestone_ids: Vec<MilestoneId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本
    pub lock_version: u32,
}

impl Roadmap {
    /// 字段数
    pub const FIELD_COUNT: usize = 9;
}

// =====================================================================
// Milestone 实体
// =====================================================================

/// **Milestone**(Roadmap 下的里程碑)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// 主键
    pub id: MilestoneId,
    /// Roadmap ID
    pub roadmap_id: RoadmapId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 名称
    pub name: String,
    /// 目标日期
    pub target_date: DateTime<Utc>,
    /// 关联 WorkItem
    pub work_item_ids: Vec<WorkItemId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Milestone {
    /// 字段数
    pub const FIELD_COUNT: usize = 8;
}

// =====================================================================
// BurndownSnapshot(Projection)
// =====================================================================

/// **BurndownSnapshot**(Worker 周期刷新的 Projection)
///
/// 来源: docs/specs/domain-planning-spec.md §2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownSnapshot {
    pub id: BurndownSnapshotId,
    pub sprint_id: SprintId,
    pub tenant_id: TenantId,
    /// 快照时间
    pub snapshot_at: DateTime<Utc>,
    /// 剩余故事点
    pub remaining_story_points: u32,
    /// 剩余 WorkItem 数
    pub remaining_work_item_count: u32,
    /// 理想故事点(线性插值)
    pub ideal_story_points: u32,
}

impl BurndownSnapshot {
    /// 字段数
    pub const FIELD_COUNT: usize = 7;
}

// =====================================================================
// BurndownReport(查询返回)
// =====================================================================

/// **BurndownReport**(查询结果聚合)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownReport {
    /// Sprint ID
    pub sprint_id: SprintId,
    /// Sprint 总故事点
    pub total_story_points: u32,
    /// 当前剩余故事点
    pub current_remaining_story_points: u32,
    /// 快照序列(按时间升序)
    pub snapshots: Vec<BurndownSnapshot>,
    /// 报告生成时间
    pub generated_at: DateTime<Utc>,
}

// 静默引用避免未使用警告
#[allow(dead_code)]
fn _unused_id(u: UserId) -> UserId {
    u
}

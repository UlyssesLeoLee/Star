//! Relation 域实体
//!
//! 来源:
//! - `docs/data-design.md` §4.8 (`relation` schema)
//! - `docs/specs/domain-relation-spec.md` §2 (实体清单)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{ProjectId, RelationId, RelationType, TenantId, UserId, WorkItemId};

// =====================================================================
// Relation 聚合根
// =====================================================================

/// **Relation 聚合根**(WorkItem 间关系)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: RelationId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub source_work_item_id: WorkItemId,
    pub target_work_item_id: WorkItemId,
    pub relation_type: RelationType,
    pub created_by_user_id: UserId,
    pub created_at: DateTime<Utc>,
    /// 可选说明
    pub note: Option<String>,
}

impl Relation {
    /// 字段数
    pub const FIELD_COUNT: usize = 9;
}

// =====================================================================
// DependencyProjection(派生)
// =====================================================================

/// **Dependency**(派生 Projection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub work_item_id: WorkItemId,
    /// 直接依赖
    pub direct_dependencies: Vec<WorkItemId>,
    /// 间接依赖(传递闭包)
    pub transitive_dependencies: Vec<WorkItemId>,
    /// 是否构成循环
    pub is_circular: bool,
}

impl Dependency {
    pub const FIELD_COUNT: usize = 4;
}

// =====================================================================
// CircularDependencyReport
// =====================================================================

/// **CircularDependencyReport**(循环依赖报告)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyReport {
    pub work_item_id: WorkItemId,
    /// 发现的循环
    pub cycle: Vec<WorkItemId>,
    /// 是否真的循环
    pub is_circular: bool,
}

impl CircularDependencyReport {
    pub const FIELD_COUNT: usize = 3;
}

// =====================================================================
// GanttReport(派生)
// =====================================================================

/// **GanttReport**(Gantt 派生)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttReport {
    pub work_item_id: WorkItemId,
    pub start_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub dependencies: Vec<WorkItemId>,
    /// 是否在关键路径
    pub is_critical_path: bool,
}

impl GanttReport {
    pub const FIELD_COUNT: usize = 5;
}

// =====================================================================
// DateRange
// =====================================================================

/// **DateRange**(日期范围)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

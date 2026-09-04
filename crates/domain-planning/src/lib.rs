//! domain-planning crate
//!
//! **crate**: `domain-planning`
//! **上游 spec**: docs/specs/domain-planning-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.9.2 / §4.9.4
//! **数据设计**: docs/data-design.md §4.7 (`planning` schema)
//! **API 设计**: docs/api-design.md §3.8 (Sprint / Backlog / Roadmap)
//!
//! ## 职责
//!
//! 敏捷规划核心数据(§9, §10, REQ-PLAN-001~005):
//! - 2 个核心聚合根(`Sprint` / `Milestone`)
//! - 3 个值对象(`SprintBacklogItem` / `Capacity` / `BurndownPoint`)
//! - 1 个状态枚举(`SprintStatus` + `MilestoneStatus`)
//! - 3 个端口(`PlanningCommandPort` × 6 / `PlanningQueryPort` × 3 / `PlanningRepository`)
//! - 5 条不变量(INV-PL-01~05)
//! - 1 个 `InMemoryPlanningService` + 1 个 `InMemoryPlanningRepository`
//!
//! ## 关键不变量
//!
//! - INV-PL-01:Sprint 必带 tenant_id + project_id
//! - INV-PL-02:同 project 内 Sprint 日期不重叠
//! - INV-PL-03:Milestone 必带 tenant_id
//! - INV-PL-04:Cancelled / Completed 不可再 activate
//! - INV-PL-05:start_sprint 需 project_admin
//!
//! Lead 责任: planning Lead

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(SprintId);
define_uuid_id!(MilestoneId);
define_uuid_id!(SprintBacklogItemId);
define_uuid_id!(CapacityId);
define_uuid_id!(BurndownPointId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 角色常量
// =====================================================================

/// 角色字符串常量(与 domain-permission 保持一致)
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读访客
    pub const VIEWER: &str = "viewer";
    /// Agent 自身
    pub const AGENT: &str = "agent";
}

// =====================================================================
// 枚举:SprintStatus(§9.1 状态机)
// =====================================================================

/// Sprint 状态(§9.1)
/// 状态机迁移:
///   Planned   -> Active   (start,需 project_admin / INV-PL-05)
///   Active    -> Completed (complete,需 project_admin)
///   Planned   -> Cancelled (cancel)
///   Active    -> Cancelled (cancel)
/// 终态:Completed / Cancelled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SprintStatus {
    /// 已规划,未启动
    Planned,
    /// 进行中
    Active,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

impl SprintStatus {
    /// 大写字符串序列化
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "PLANNED",
            Self::Active => "ACTIVE",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
        }
    }
    /// 是否终态(INV-PL-04)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }
    /// 严格状态机迁移检查
    pub fn check_transition(from: SprintStatus, to: SprintStatus) -> Result<(), PlanningError> {
        use SprintStatus::*;
        let allowed = matches!(
            (from, to),
            (Planned, Active) | (Planned, Cancelled) | (Active, Completed) | (Active, Cancelled)
        );
        if !allowed {
            return Err(PlanningError::InvalidState(format!(
                "sprint status: {} -> {} not allowed",
                from.as_str(),
                to.as_str()
            )));
        }
        Ok(())
    }
}

// =====================================================================
// 枚举:MilestoneStatus(§10)
// =====================================================================

/// Milestone 状态(§10)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilestoneStatus {
    /// 开放(尚未达成 / 未过期)
    Open,
    /// 已达成
    Achieved,
    /// 错过(超过 due_date 仍未达成)
    Missed,
}

impl MilestoneStatus {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Achieved => "ACHIEVED",
            Self::Missed => "MISSED",
        }
    }
    /// 是否终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Achieved | Self::Missed)
    }
}

// =====================================================================
// 实体:Sprint(聚合根,§9.1)
// =====================================================================

/// **Sprint 聚合根**(§9.1, REQ-PLAN-001)
///
/// 关键不变量:
/// - INV-PL-01:必带 tenant_id + project_id
/// - INV-PL-02:同 project 内 Sprint 日期不重叠
/// - INV-PL-04:Completed / Cancelled 不可再 activate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    /// 主键
    pub id: SprintId,
    /// 租户 ID(INV-PL-01)
    pub tenant_id: TenantId,
    /// 项目 ID(INV-PL-01)
    pub project_id: ProjectId,
    /// Sprint 名称
    pub name: String,
    /// Sprint 目标
    pub goal: String,
    /// 起始时间
    pub start_date: DateTime<Utc>,
    /// 结束时间
    pub end_date: DateTime<Utc>,
    /// 状态
    pub status: SprintStatus,
    /// 关联 WorkItem 列表(backlog 内嵌简化)
    pub work_item_ids: Vec<WorkItemId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Sprint {
    /// 构造一个新 Sprint(INV-PL-01 校验:tenant_id + project_id 必带)
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        name: String,
        goal: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Self, PlanningError> {
        if name.trim().is_empty() {
            return Err(PlanningError::InvalidState(
                "sprint name required".to_string(),
            ));
        }
        if end_date <= start_date {
            return Err(PlanningError::InvalidState(
                "sprint end_date must be after start_date".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: SprintId::new(),
            tenant_id,
            project_id,
            name,
            goal,
            start_date,
            end_date,
            status: SprintStatus::Planned,
            work_item_ids: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    /// 状态机迁移(§9.1, INV-PL-04)
    pub fn transition(&mut self, to: SprintStatus) -> Result<(), PlanningError> {
        SprintStatus::check_transition(self.status, to)?;
        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 追加 WorkItem(去重)
    pub fn add_work_item(&mut self, work_item_id: WorkItemId) -> Result<(), PlanningError> {
        if self.work_item_ids.contains(&work_item_id) {
            return Err(PlanningError::Conflict(format!(
                "work_item {} already in sprint",
                work_item_id
            )));
        }
        self.work_item_ids.push(work_item_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 与另一个 Sprint 是否日期重叠(INV-PL-02)
    pub fn overlaps(&self, other: &Sprint) -> bool {
        if self.project_id != other.project_id {
            return false;
        }
        if self.id == other.id {
            return false;
        }
        // 半开区间重叠:[start, end)
        self.start_date < other.end_date && other.start_date < self.end_date
    }
}

// =====================================================================
// 实体:SprintBacklogItem(值对象,§9.1)
// =====================================================================

/// **SprintBacklogItem 值对象**(§9.1)
/// Sprint 与 WorkItem 的关联记录(带 story_points 与 added_at)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintBacklogItem {
    pub id: SprintBacklogItemId,
    pub sprint_id: SprintId,
    pub work_item_id: WorkItemId,
    pub story_points: Option<u32>,
    pub added_at: DateTime<Utc>,
}

impl SprintBacklogItem {
    /// 构造一个 backlog 项
    pub fn new(sprint_id: SprintId, work_item_id: WorkItemId, story_points: Option<u32>) -> Self {
        Self {
            id: SprintBacklogItemId::new(),
            sprint_id,
            work_item_id,
            story_points,
            added_at: Utc::now(),
        }
    }
}

// =====================================================================
// 实体:Milestone(聚合根,§10)
// =====================================================================

/// **Milestone 聚合根**(§10)
///
/// 关键不变量:
/// - INV-PL-03:Milestone 必带 tenant_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: MilestoneId,
    /// INV-PL-03:必带
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub description: String,
    pub due_date: DateTime<Utc>,
    pub status: MilestoneStatus,
    pub work_item_ids: Vec<WorkItemId>,
    pub created_at: DateTime<Utc>,
}

impl Milestone {
    /// 构造一个新 Milestone
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        name: String,
        description: String,
        due_date: DateTime<Utc>,
    ) -> Result<Self, PlanningError> {
        if name.trim().is_empty() {
            return Err(PlanningError::InvalidState(
                "milestone name required".to_string(),
            ));
        }
        Ok(Self {
            id: MilestoneId::new(),
            tenant_id,
            project_id,
            name,
            description,
            due_date,
            status: MilestoneStatus::Open,
            work_item_ids: vec![],
            created_at: Utc::now(),
        })
    }

    /// 标记达成(Open -> Achieved)
    pub fn achieve(&mut self) -> Result<(), PlanningError> {
        if self.status != MilestoneStatus::Open {
            return Err(PlanningError::InvalidState(format!(
                "milestone already {}",
                self.status.as_str()
            )));
        }
        self.status = MilestoneStatus::Achieved;
        Ok(())
    }

    /// 标记错过(Open -> Missed,通常由后台 cron 推进)
    pub fn mark_missed(&mut self) -> Result<(), PlanningError> {
        if self.status != MilestoneStatus::Open {
            return Err(PlanningError::InvalidState(format!(
                "milestone already {}",
                self.status.as_str()
            )));
        }
        self.status = MilestoneStatus::Missed;
        Ok(())
    }

    /// 追加 WorkItem(去重)
    pub fn add_work_item(&mut self, work_item_id: WorkItemId) -> Result<(), PlanningError> {
        if self.work_item_ids.contains(&work_item_id) {
            return Err(PlanningError::Conflict(format!(
                "work_item {} already in milestone",
                work_item_id
            )));
        }
        self.work_item_ids.push(work_item_id);
        Ok(())
    }
}

// =====================================================================
// 值对象:Capacity(§9.2)
// =====================================================================

/// **Capacity 值对象**(§9.2)
/// 每个 user 在 sprint 内的承诺工时与实际工时
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capacity {
    pub id: CapacityId,
    pub sprint_id: SprintId,
    pub user_id: UserId,
    pub committed_hours: f32,
    pub actual_hours: f32,
}

impl Capacity {
    /// 新建
    pub fn new(sprint_id: SprintId, user_id: UserId, committed_hours: f32) -> Self {
        Self {
            id: CapacityId::new(),
            sprint_id,
            user_id,
            committed_hours,
            actual_hours: 0.0,
        }
    }

    /// 记录实际工时
    pub fn record_actual(&mut self, hours: f32) -> Result<(), PlanningError> {
        if hours < 0.0 {
            return Err(PlanningError::InvalidState(
                "actual_hours cannot be negative".to_string(),
            ));
        }
        if hours > self.committed_hours * 2.0 {
            // 超过承诺 2 倍视为异常(INV-PL-06 衍生)
            return Err(PlanningError::CapacityExceeded(format!(
                "actual_hours {} > 2x committed {}",
                hours, self.committed_hours
            )));
        }
        self.actual_hours = hours;
        Ok(())
    }

    /// 是否超出承诺
    pub fn is_over_capacity(&self) -> bool {
        self.actual_hours > self.committed_hours
    }
}

// =====================================================================
// 值对象:BurndownPoint(§9.3)
// =====================================================================

/// **BurndownPoint 值对象**(§9.3)
/// Sprint 内每日剩余 story points 快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownPoint {
    pub id: BurndownPointId,
    pub sprint_id: SprintId,
    pub date: DateTime<Utc>,
    pub remaining_points: u32,
    pub ideal_remaining: u32,
}

impl BurndownPoint {
    /// 新建
    pub fn new(sprint_id: SprintId, date: DateTime<Utc>, remaining: u32, ideal: u32) -> Self {
        Self {
            id: BurndownPointId::new(),
            sprint_id,
            date,
            remaining_points: remaining,
            ideal_remaining: ideal,
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

/// **PlanningError** — 规划域统一错误
#[derive(Debug, Error)]
pub enum PlanningError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: actor tenant {0} vs resource tenant {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("sprint date overlap with existing sprint {0} in same project")]
    SprintOverlap(SprintId),
    #[error("capacity exceeded: {0}")]
    CapacityExceeded(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl PlanningError {
    /// 错误码
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "PLANNING_NOT_FOUND",
            Self::PermissionDenied => "PLANNING_PERMISSION_DENIED",
            Self::CrossTenantDenied(_, _) => "PLANNING_CROSS_TENANT_DENIED",
            Self::InvalidState(_) => "PLANNING_INVALID_STATE",
            Self::SprintOverlap(_) => "PLANNING_SPRINT_OVERLAP",
            Self::CapacityExceeded(_) => "PLANNING_CAPACITY_EXCEEDED",
            Self::Conflict(_) => "PLANNING_CONFLICT",
            Self::Internal(_) => "PLANNING_INTERNAL",
        }
    }
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

/// 创建 Sprint 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSprintCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub goal: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// 添加 WorkItem 到 Sprint 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddToBacklogCommand {
    pub tenant_id: TenantId,
    pub sprint_id: SprintId,
    pub work_item_id: WorkItemId,
    pub story_points: Option<u32>,
}

/// 创建 Milestone 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMilestoneCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub description: String,
    pub due_date: DateTime<Utc>,
}

/// 查询:获取 Sprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSprintQuery {
    pub tenant_id: TenantId,
    pub sprint_id: SprintId,
}

/// 查询:列出 Active Sprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActiveSprintQuery {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
}

/// 查询:获取 Burndown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBurndownQuery {
    pub tenant_id: TenantId,
    pub sprint_id: SprintId,
}

/// 追加 Burndown 点的命令(用于数据采集)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendBurndownPointCommand {
    pub tenant_id: TenantId,
    pub sprint_id: SprintId,
    pub remaining_points: u32,
    pub ideal_remaining: u32,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **PlanningCommandPort** — 写操作
#[async_trait]
pub trait PlanningCommandPort: Send + Sync {
    /// 创建 Sprint
    async fn create_sprint(
        &self,
        cmd: CreateSprintCommand,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 启动 Sprint(Planned -> Active,INV-PL-05 需 project_admin)
    async fn start_sprint(
        &self,
        sprint_id: SprintId,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 完成 Sprint(Active -> Completed,需 project_admin)
    async fn complete_sprint(
        &self,
        sprint_id: SprintId,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 取消 Sprint(Planned / Active -> Cancelled,需 project_admin)
    async fn cancel_sprint(
        &self,
        sprint_id: SprintId,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 添加 WorkItem 到 Sprint backlog
    async fn add_to_backlog(
        &self,
        cmd: AddToBacklogCommand,
        actor: &ActorContext,
    ) -> Result<SprintBacklogItem, PlanningError>;

    /// 创建 Milestone
    async fn create_milestone(
        &self,
        cmd: CreateMilestoneCommand,
        actor: &ActorContext,
    ) -> Result<Milestone, PlanningError>;

    /// 达成 Milestone
    async fn achieve_milestone(
        &self,
        milestone_id: MilestoneId,
        actor: &ActorContext,
    ) -> Result<Milestone, PlanningError>;

    /// 追加 Burndown 点
    async fn append_burndown_point(
        &self,
        cmd: AppendBurndownPointCommand,
        actor: &ActorContext,
    ) -> Result<BurndownPoint, PlanningError>;
}

/// **PlanningQueryPort** — 读操作
#[async_trait]
pub trait PlanningQueryPort: Send + Sync {
    /// 获取 Sprint
    async fn get_sprint(
        &self,
        q: GetSprintQuery,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 列出 Active Sprint
    async fn list_active_sprints(
        &self,
        q: ListActiveSprintQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Sprint>, PlanningError>;

    /// 获取 Sprint 的 burndown
    async fn get_burndown(
        &self,
        q: GetBurndownQuery,
        actor: &ActorContext,
    ) -> Result<Vec<BurndownPoint>, PlanningError>;
}

/// **PlanningRepository** — 持久化抽象
#[async_trait]
pub trait PlanningRepository: Send + Sync {
    async fn insert_sprint(&self, s: Sprint) -> Result<(), PlanningError>;
    async fn get_sprint(&self, id: SprintId) -> Result<Sprint, PlanningError>;
    async fn update_sprint(&self, s: Sprint) -> Result<(), PlanningError>;
    async fn list_sprints(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
        status: Option<SprintStatus>,
    ) -> Result<Vec<Sprint>, PlanningError>;

    async fn insert_milestone(&self, m: Milestone) -> Result<(), PlanningError>;
    async fn get_milestone(&self, id: MilestoneId) -> Result<Milestone, PlanningError>;
    async fn update_milestone(&self, m: Milestone) -> Result<(), PlanningError>;
    async fn list_milestones(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<Milestone>, PlanningError>;

    async fn insert_backlog_item(&self, item: SprintBacklogItem) -> Result<(), PlanningError>;
    async fn list_backlog_items(
        &self,
        sprint_id: SprintId,
    ) -> Result<Vec<SprintBacklogItem>, PlanningError>;

    async fn insert_burndown_point(&self, point: BurndownPoint) -> Result<(), PlanningError>;
    async fn list_burndown_points(
        &self,
        sprint_id: SprintId,
    ) -> Result<Vec<BurndownPoint>, PlanningError>;
}

// =====================================================================
// InMemoryPlanningService
// =====================================================================

/// 内存版 Planning Service(测试 / 本地运行)
pub struct InMemoryPlanningService {
    repo: Arc<dyn PlanningRepository>,
    /// 缓存:跨方法保持写后读一致性
    sprints: Arc<RwLock<HashMap<SprintId, Sprint>>>,
    milestones: Arc<RwLock<HashMap<MilestoneId, Milestone>>>,
    backlog: Arc<RwLock<HashMap<SprintId, Vec<SprintBacklogItem>>>>,
    burndown: Arc<RwLock<HashMap<SprintId, Vec<BurndownPoint>>>>,
}

impl InMemoryPlanningService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryPlanningRepository::new()),
            sprints: Arc::new(RwLock::new(HashMap::new())),
            milestones: Arc::new(RwLock::new(HashMap::new())),
            backlog: Arc::new(RwLock::new(HashMap::new())),
            burndown: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_repo(repo: Arc<dyn PlanningRepository>) -> Self {
        Self {
            repo,
            sprints: Arc::new(RwLock::new(HashMap::new())),
            milestones: Arc::new(RwLock::new(HashMap::new())),
            backlog: Arc::new(RwLock::new(HashMap::new())),
            burndown: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryPlanningService {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// InMemoryPlanningRepository
// =====================================================================

/// 内存版 Planning Repository
pub struct InMemoryPlanningRepository {
    sprints: RwLock<HashMap<SprintId, Sprint>>,
    milestones: RwLock<HashMap<MilestoneId, Milestone>>,
    backlog: RwLock<HashMap<SprintId, Vec<SprintBacklogItem>>>,
    burndown: RwLock<HashMap<SprintId, Vec<BurndownPoint>>>,
}

impl InMemoryPlanningRepository {
    pub fn new() -> Self {
        Self {
            sprints: RwLock::new(HashMap::new()),
            milestones: RwLock::new(HashMap::new()),
            backlog: RwLock::new(HashMap::new()),
            burndown: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPlanningRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlanningRepository for InMemoryPlanningRepository {
    async fn insert_sprint(&self, s: Sprint) -> Result<(), PlanningError> {
        let mut store = self.sprints.write().expect("lock");
        if store.contains_key(&s.id) {
            return Err(PlanningError::Conflict(format!(
                "sprint {} already exists",
                s.id
            )));
        }
        store.insert(s.id, s);
        Ok(())
    }

    async fn get_sprint(&self, id: SprintId) -> Result<Sprint, PlanningError> {
        let store = self.sprints.read().expect("lock");
        store
            .get(&id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", id)))
    }

    async fn update_sprint(&self, s: Sprint) -> Result<(), PlanningError> {
        let mut store = self.sprints.write().expect("lock");
        store.insert(s.id, s);
        Ok(())
    }

    async fn list_sprints(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
        status: Option<SprintStatus>,
    ) -> Result<Vec<Sprint>, PlanningError> {
        let store = self.sprints.read().expect("lock");
        Ok(store
            .values()
            .filter(|s| s.tenant_id == tenant_id)
            .filter(|s| project_id.map(|p| s.project_id == p).unwrap_or(true))
            .filter(|s| status.map(|st| s.status == st).unwrap_or(true))
            .cloned()
            .collect())
    }

    async fn insert_milestone(&self, m: Milestone) -> Result<(), PlanningError> {
        let mut store = self.milestones.write().expect("lock");
        if store.contains_key(&m.id) {
            return Err(PlanningError::Conflict(format!(
                "milestone {} already exists",
                m.id
            )));
        }
        store.insert(m.id, m);
        Ok(())
    }

    async fn get_milestone(&self, id: MilestoneId) -> Result<Milestone, PlanningError> {
        let store = self.milestones.read().expect("lock");
        store
            .get(&id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("milestone:{}", id)))
    }

    async fn update_milestone(&self, m: Milestone) -> Result<(), PlanningError> {
        let mut store = self.milestones.write().expect("lock");
        store.insert(m.id, m);
        Ok(())
    }

    async fn list_milestones(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<Milestone>, PlanningError> {
        let store = self.milestones.read().expect("lock");
        Ok(store
            .values()
            .filter(|m| m.tenant_id == tenant_id)
            .filter(|m| project_id.map(|p| m.project_id == p).unwrap_or(true))
            .cloned()
            .collect())
    }

    async fn insert_backlog_item(&self, item: SprintBacklogItem) -> Result<(), PlanningError> {
        let mut store = self.backlog.write().expect("lock");
        store.entry(item.sprint_id).or_default().push(item);
        Ok(())
    }

    async fn list_backlog_items(
        &self,
        sprint_id: SprintId,
    ) -> Result<Vec<SprintBacklogItem>, PlanningError> {
        let store = self.backlog.read().expect("lock");
        Ok(store.get(&sprint_id).cloned().unwrap_or_default())
    }

    async fn insert_burndown_point(&self, point: BurndownPoint) -> Result<(), PlanningError> {
        let mut store = self.burndown.write().expect("lock");
        store.entry(point.sprint_id).or_default().push(point);
        Ok(())
    }

    async fn list_burndown_points(
        &self,
        sprint_id: SprintId,
    ) -> Result<Vec<BurndownPoint>, PlanningError> {
        let store = self.burndown.read().expect("lock");
        Ok(store.get(&sprint_id).cloned().unwrap_or_default())
    }
}

// =====================================================================
// 内部工具
// =====================================================================

/// 检查同 project 内是否已有日期重叠的 Sprint(INV-PL-02)
fn check_sprint_overlap(
    sprints: &HashMap<SprintId, Sprint>,
    candidate: &Sprint,
) -> Result<(), PlanningError> {
    for existing in sprints.values() {
        if existing.project_id == candidate.project_id
            && existing.id != candidate.id
            && existing.status != SprintStatus::Cancelled
            && existing.status != SprintStatus::Completed
        {
            if existing.overlaps(candidate) {
                return Err(PlanningError::SprintOverlap(existing.id));
            }
        }
    }
    Ok(())
}

// =====================================================================
// InMemoryPlanningService - PlanningCommandPort 实现
// =====================================================================

#[async_trait]
impl PlanningCommandPort for InMemoryPlanningService {
    async fn create_sprint(
        &self,
        cmd: CreateSprintCommand,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError> {
        // 跨租户检查
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let sprint = Sprint::new(
            cmd.tenant_id,
            cmd.project_id,
            cmd.name,
            cmd.goal,
            cmd.start_date,
            cmd.end_date,
        )?;
        // INV-PL-02:同 project 日期不重叠
        {
            let store = self.sprints.read().expect("lock");
            check_sprint_overlap(&store, &sprint)?;
        }
        self.repo.insert_sprint(sprint.clone()).await?;
        self.sprints
            .write()
            .expect("lock")
            .insert(sprint.id, sprint.clone());
        Ok(sprint)
    }

    async fn start_sprint(
        &self,
        sprint_id: SprintId,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError> {
        // INV-PL-05:start_sprint 需 project_admin
        if !(actor.has_role("project_admin") || actor.is_platform_admin) {
            return Err(PlanningError::PermissionDenied);
        }
        let mut sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", sprint_id)))?;
        if sprint.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        // INV-PL-04:Completed / Cancelled 不可再 activate
        sprint.transition(SprintStatus::Active)?;
        self.repo.update_sprint(sprint.clone()).await?;
        self.sprints
            .write()
            .expect("lock")
            .insert(sprint.id, sprint.clone());
        Ok(sprint)
    }

    async fn complete_sprint(
        &self,
        sprint_id: SprintId,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError> {
        if !(actor.has_role("project_admin") || actor.is_platform_admin) {
            return Err(PlanningError::PermissionDenied);
        }
        let mut sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", sprint_id)))?;
        if sprint.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        sprint.transition(SprintStatus::Completed)?;
        self.repo.update_sprint(sprint.clone()).await?;
        self.sprints
            .write()
            .expect("lock")
            .insert(sprint.id, sprint.clone());
        Ok(sprint)
    }

    async fn cancel_sprint(
        &self,
        sprint_id: SprintId,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError> {
        if !(actor.has_role("project_admin") || actor.is_platform_admin) {
            return Err(PlanningError::PermissionDenied);
        }
        let mut sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", sprint_id)))?;
        if sprint.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        sprint.transition(SprintStatus::Cancelled)?;
        self.repo.update_sprint(sprint.clone()).await?;
        self.sprints
            .write()
            .expect("lock")
            .insert(sprint.id, sprint.clone());
        Ok(sprint)
    }

    async fn add_to_backlog(
        &self,
        cmd: AddToBacklogCommand,
        actor: &ActorContext,
    ) -> Result<SprintBacklogItem, PlanningError> {
        let mut sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&cmd.sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", cmd.sprint_id)))?;
        if sprint.tenant_id != cmd.tenant_id {
            return Err(PlanningError::CrossTenantDenied(
                cmd.tenant_id,
                sprint.tenant_id,
            ));
        }
        if sprint.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        // Sprint 终态后不能再加
        if sprint.status.is_terminal() {
            return Err(PlanningError::InvalidState(format!(
                "cannot add to {} sprint",
                sprint.status.as_str()
            )));
        }
        // 内嵌 work_item_ids + 独立 SprintBacklogItem 两路保持
        sprint.add_work_item(cmd.work_item_id)?;
        self.repo.update_sprint(sprint.clone()).await?;
        self.sprints
            .write()
            .expect("lock")
            .insert(sprint.id, sprint.clone());

        let item = SprintBacklogItem::new(cmd.sprint_id, cmd.work_item_id, cmd.story_points);
        self.repo.insert_backlog_item(item.clone()).await?;
        self.backlog
            .write()
            .expect("lock")
            .entry(cmd.sprint_id)
            .or_default()
            .push(item.clone());
        Ok(item)
    }

    async fn create_milestone(
        &self,
        cmd: CreateMilestoneCommand,
        actor: &ActorContext,
    ) -> Result<Milestone, PlanningError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let milestone = Milestone::new(
            cmd.tenant_id,
            cmd.project_id,
            cmd.name,
            cmd.description,
            cmd.due_date,
        )?;
        self.repo.insert_milestone(milestone.clone()).await?;
        self.milestones
            .write()
            .expect("lock")
            .insert(milestone.id, milestone.clone());
        Ok(milestone)
    }

    async fn achieve_milestone(
        &self,
        milestone_id: MilestoneId,
        actor: &ActorContext,
    ) -> Result<Milestone, PlanningError> {
        let mut milestone = self
            .milestones
            .read()
            .expect("lock")
            .get(&milestone_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("milestone:{}", milestone_id)))?;
        if milestone.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                milestone.tenant_id,
            ));
        }
        milestone.achieve()?;
        self.repo.update_milestone(milestone.clone()).await?;
        self.milestones
            .write()
            .expect("lock")
            .insert(milestone.id, milestone.clone());
        Ok(milestone)
    }

    async fn append_burndown_point(
        &self,
        cmd: AppendBurndownPointCommand,
        actor: &ActorContext,
    ) -> Result<BurndownPoint, PlanningError> {
        // 校验 sprint 存在 + 跨租户
        let sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&cmd.sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", cmd.sprint_id)))?;
        if sprint.tenant_id != TenantId::from(actor.tenant_id) || sprint.tenant_id != cmd.tenant_id
        {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        let point = BurndownPoint::new(
            cmd.sprint_id,
            Utc::now(),
            cmd.remaining_points,
            cmd.ideal_remaining,
        );
        self.repo.insert_burndown_point(point.clone()).await?;
        self.burndown
            .write()
            .expect("lock")
            .entry(cmd.sprint_id)
            .or_default()
            .push(point.clone());
        Ok(point)
    }
}

// =====================================================================
// InMemoryPlanningService - PlanningQueryPort 实现
// =====================================================================

#[async_trait]
impl PlanningQueryPort for InMemoryPlanningService {
    async fn get_sprint(
        &self,
        q: GetSprintQuery,
        actor: &ActorContext,
    ) -> Result<Sprint, PlanningError> {
        let sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&q.sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", q.sprint_id)))?;
        if sprint.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        Ok(sprint)
    }

    async fn list_active_sprints(
        &self,
        q: ListActiveSprintQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Sprint>, PlanningError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let store = self.sprints.read().expect("lock");
        let mut out: Vec<Sprint> = store
            .values()
            .filter(|s| s.tenant_id == q.tenant_id)
            .filter(|s| s.status == SprintStatus::Active)
            .filter(|s| q.project_id.map(|p| s.project_id == p).unwrap_or(true))
            .cloned()
            .collect();
        out.sort_by_key(|s| s.start_date);
        Ok(out)
    }

    async fn get_burndown(
        &self,
        q: GetBurndownQuery,
        actor: &ActorContext,
    ) -> Result<Vec<BurndownPoint>, PlanningError> {
        // 跨租户校验
        let sprint = self
            .sprints
            .read()
            .expect("lock")
            .get(&q.sprint_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(format!("sprint:{}", q.sprint_id)))?;
        if sprint.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(PlanningError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                sprint.tenant_id,
            ));
        }
        let mut pts = self
            .burndown
            .read()
            .expect("lock")
            .get(&q.sprint_id)
            .cloned()
            .unwrap_or_default();
        pts.sort_by_key(|p| p.date);
        Ok(pts)
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    fn make_admin_actor(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(*project_id.as_uuid())
    }

    fn make_dev_actor(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::DEVELOPER)
            .with_project(*project_id.as_uuid())
    }

    fn make_create_sprint_cmd(tenant_id: TenantId, project_id: ProjectId) -> CreateSprintCommand {
        let now = Utc::now();
        CreateSprintCommand {
            tenant_id,
            project_id,
            name: "Sprint 1".to_string(),
            goal: "Ship MVP".to_string(),
            start_date: now,
            end_date: now + Duration::days(14),
        }
    }

    // ----- 1. create_sprint 基本 -----
    #[tokio::test]
    async fn create_sprint_basic() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &actor).await.expect("create");
        assert_eq!(sprint.status, SprintStatus::Planned);
        assert_eq!(sprint.tenant_id, TenantId(tenant_id));
        assert_eq!(sprint.project_id, project_id);
        assert!(sprint.work_item_ids.is_empty());
    }

    // ----- 2. Sprint 状态机(Planned→Active→Completed)-----
    #[tokio::test]
    async fn sprint_status_state_machine() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &actor).await.unwrap();
        let started = svc.start_sprint(sprint.id, &actor).await.unwrap();
        assert_eq!(started.status, SprintStatus::Active);
        let completed = svc.complete_sprint(sprint.id, &actor).await.unwrap();
        assert_eq!(completed.status, SprintStatus::Completed);
        assert!(completed.status.is_terminal());
    }

    // ----- 3. start_sprint 需 project_admin (INV-PL-05) -----
    #[tokio::test]
    async fn start_sprint_requires_admin() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let admin = make_admin_actor(TenantId(tenant_id), project_id);
        let dev = make_dev_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &admin).await.unwrap();
        // dev 不能 start
        let res = svc.start_sprint(sprint.id, &dev).await;
        assert!(matches!(res, Err(PlanningError::PermissionDenied)));
        // admin 可以
        let res2 = svc.start_sprint(sprint.id, &admin).await;
        assert!(res2.is_ok());
    }

    // ----- 4. cancel_sprint(Planned/Active → Cancelled)-----
    #[tokio::test]
    async fn cancel_sprint() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &actor).await.unwrap();
        let cancelled = svc.cancel_sprint(sprint.id, &actor).await.unwrap();
        assert_eq!(cancelled.status, SprintStatus::Cancelled);
        assert!(cancelled.status.is_terminal());
    }

    // ----- 5. Completed 不可再 activate (INV-PL-04) -----
    #[tokio::test]
    async fn completed_sprint_terminal() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &actor).await.unwrap();
        svc.start_sprint(sprint.id, &actor).await.unwrap();
        svc.complete_sprint(sprint.id, &actor).await.unwrap();
        // 尝试 start → InvalidState
        let res = svc.start_sprint(sprint.id, &actor).await;
        assert!(matches!(res, Err(PlanningError::InvalidState(_))));
    }

    // ----- 6. add_to_backlog -----
    #[tokio::test]
    async fn add_to_backlog() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &actor).await.unwrap();
        let wi = WorkItemId::new();
        let item = svc
            .add_to_backlog(
                AddToBacklogCommand {
                    tenant_id: TenantId(tenant_id),
                    sprint_id: sprint.id,
                    work_item_id: wi,
                    story_points: Some(5),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(item.work_item_id, wi);
        assert_eq!(item.story_points, Some(5));
        // 再次添加相同 wi → Conflict
        let res = svc
            .add_to_backlog(
                AddToBacklogCommand {
                    tenant_id: TenantId(tenant_id),
                    sprint_id: sprint.id,
                    work_item_id: wi,
                    story_points: Some(3),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(PlanningError::Conflict(_))));
    }

    // ----- 7. Sprint 日期重叠拒绝 (INV-PL-02) -----
    #[tokio::test]
    async fn sprint_date_overlap_rejected() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let now = Utc::now();
        // 第一个 sprint
        let cmd1 = CreateSprintCommand {
            tenant_id,
            project_id,
            name: "A".to_string(),
            goal: "g".to_string(),
            start_date: now,
            end_date: now + Duration::days(14),
        };
        svc.create_sprint(cmd1, &actor).await.unwrap();
        // 重叠
        let cmd2 = CreateSprintCommand {
            tenant_id,
            project_id,
            name: "B".to_string(),
            goal: "g".to_string(),
            start_date: now + Duration::days(7),
            end_date: now + Duration::days(21),
        };
        let res = svc.create_sprint(cmd2, &actor).await;
        assert!(matches!(res, Err(PlanningError::SprintOverlap(_))));
    }

    // ----- 8. 跨租户拒绝 -----
    #[tokio::test]
    async fn cross_tenant_sprint_denied() {
        let svc = InMemoryPlanningService::new();
        let actor_tenant = uuid::Uuid::new_v4();
        let cmd_tenant = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = ActorContext::new(Uuid::new_v4(), actor_tenant.0)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(*project_id.as_uuid());
        let cmd = CreateSprintCommand {
            tenant_id: cmd_tenant,
            project_id,
            name: "X".to_string(),
            goal: "g".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now() + Duration::days(14),
        };
        let res = svc.create_sprint(cmd, &actor).await;
        assert!(matches!(res, Err(PlanningError::CrossTenantDenied(_, _))));
    }

    // ----- 9. create_milestone -----
    #[tokio::test]
    async fn create_milestone() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let m = svc
            .create_milestone(
                CreateMilestoneCommand {
                    tenant_id: TenantId(tenant_id),
                    project_id,
                    name: "v1.0".to_string(),
                    description: "Release".to_string(),
                    due_date: Utc::now() + Duration::days(30),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(m.status, MilestoneStatus::Open);
        assert_eq!(m.tenant_id, TenantId(tenant_id));
    }

    // ----- 10. achieve_milestone -----
    #[tokio::test]
    async fn achieve_milestone() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let m = svc
            .create_milestone(
                CreateMilestoneCommand {
                    tenant_id: TenantId(tenant_id),
                    project_id,
                    name: "v1.0".to_string(),
                    description: "Release".to_string(),
                    due_date: Utc::now() + Duration::days(30),
                },
                &actor,
            )
            .await
            .unwrap();
        let achieved = svc.achieve_milestone(m.id, &actor).await.unwrap();
        assert_eq!(achieved.status, MilestoneStatus::Achieved);
        assert!(achieved.status.is_terminal());
    }

    // ----- 11. missed_milestone -----
    #[tokio::test]
    async fn missed_milestone() {
        let m = Milestone::new(
            uuid::Uuid::new_v4(),
            ProjectId::new(),
            "v1.0".to_string(),
            "Release".to_string(),
            Utc::now(),
        )
        .unwrap();
        let mut m = m;
        m.mark_missed().unwrap();
        assert_eq!(m.status, MilestoneStatus::Missed);
        assert!(m.status.is_terminal());
    }

    // ----- 12. burndown_point_append -----
    #[tokio::test]
    async fn burndown_point_append() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        let cmd = make_create_sprint_cmd(TenantId(tenant_id), project_id);
        let sprint = svc.create_sprint(cmd, &actor).await.unwrap();
        svc.start_sprint(sprint.id, &actor).await.unwrap();
        // 追加 3 个 burndown 点
        for (rem, ideal) in [(100, 100), (80, 90), (60, 80)] {
            svc.append_burndown_point(
                AppendBurndownPointCommand {
                    tenant_id: TenantId(tenant_id),
                    sprint_id: sprint.id,
                    remaining_points: rem,
                    ideal_remaining: ideal,
                },
                &actor,
            )
            .await
            .unwrap();
        }
        let pts = svc
            .get_burndown(
                GetBurndownQuery {
                    tenant_id: TenantId(tenant_id),
                    sprint_id: sprint.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0].remaining_points, 100);
        assert_eq!(pts[2].remaining_points, 60);
    }

    // ----- 13. 额外:list_active_sprints -----
    #[tokio::test]
    async fn list_active_sprints_filters() {
        let svc = InMemoryPlanningService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_admin_actor(TenantId(tenant_id), project_id);
        // 第一个 sprint:14 天
        let now = Utc::now();
        let cmd1 = CreateSprintCommand {
            tenant_id,
            project_id,
            name: "A".to_string(),
            goal: "g".to_string(),
            start_date: now,
            end_date: now + Duration::days(14),
        };
        let s1 = svc.create_sprint(cmd1, &actor).await.unwrap();
        svc.start_sprint(s1.id, &actor).await.unwrap();
        // 第二个 sprint:不重叠(从 day 15 开始)
        let cmd2 = CreateSprintCommand {
            tenant_id,
            project_id,
            name: "B".to_string(),
            goal: "g".to_string(),
            start_date: now + Duration::days(15),
            end_date: now + Duration::days(28),
        };
        svc.create_sprint(cmd2, &actor).await.unwrap();
        // 只应有 1 个 Active(s1)
        let active = svc
            .list_active_sprints(
                ListActiveSprintQuery {
                    tenant_id: TenantId(tenant_id),
                    project_id: Some(project_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, s1.id);
    }

    // ----- 14. 额外:cross_tenant get_sprint -----
    #[tokio::test]
    async fn get_sprint_cross_tenant_denied() {
        let svc = InMemoryPlanningService::new();
        let tenant_a = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor_a = make_admin_actor(tenant_a, project_id);
        let sprint = svc
            .create_sprint(make_create_sprint_cmd(tenant_a, project_id), &actor_a)
            .await
            .unwrap();
        let tenant_b = uuid::Uuid::new_v4();
        let actor_b = ActorContext::new(Uuid::new_v4(), tenant_b.0).with_role(roles::PROJECT_ADMIN);
        let res = svc
            .get_sprint(
                GetSprintQuery {
                    tenant_id: tenant_b,
                    sprint_id: sprint.id,
                },
                &actor_b,
            )
            .await;
        assert!(matches!(res, Err(PlanningError::CrossTenantDenied(_, _))));
    }
}

pub mod whatif;

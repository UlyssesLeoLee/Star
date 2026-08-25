//! Planning 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.8 (Sprint / Backlog / Roadmap 端点)
//! - `docs/specs/domain-planning-spec.md` §4 (接口签名)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Backlog, BurndownReport, Roadmap, Sprint};
use crate::error::PlanningError;
use crate::value_object::{
    BacklogId, CloseMoveTarget, ProjectId, RoadmapId, SprintId, TenantId, WorkItemId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateSprintCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSprintCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub goal: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub capacity_story_points: Option<u32>,
}

/// `UpdateSprintCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSprintCommand {
    pub sprint_id: SprintId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub name: Option<String>,
    pub goal: Option<Option<String>>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub capacity_story_points: Option<Option<u32>>,
}

/// `CloseSprintCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseSprintCommand {
    pub move_incomplete_to: CloseMoveTarget,
    /// 若选择 NextSprint,需指定目标 Sprint ID
    pub next_sprint_id: Option<SprintId>,
}

/// `BacklogReorderCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogReorderCommand {
    pub backlog_id: BacklogId,
    pub tenant_id: TenantId,
    pub work_item_order: Vec<WorkItemId>,
}

/// `AddWorkItemToSprintCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWorkItemToSprintCommand {
    pub sprint_id: SprintId,
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
}

/// `RemoveWorkItemFromSprintCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveWorkItemFromSprintCommand {
    pub sprint_id: SprintId,
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListSprintQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSprintQuery {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub state: Option<crate::value_object::SprintState>,
    pub limit: u32,
    pub offset: u32,
}

// =====================================================================
// 端口:PlanningCommandPort(7 方法)
// =====================================================================

/// **Planning 命令端口**
#[async_trait]
pub trait PlanningCommandPort: Send + Sync {
    async fn create_sprint(
        &self,
        cmd: CreateSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError>;

    async fn update_sprint(
        &self,
        cmd: UpdateSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 启动 Sprint(Planning → Active,INV-PL-03)
    async fn start_sprint(
        &self,
        sprint_id: SprintId,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError>;

    /// 关闭 Sprint(Active → Closed,INV-PL-01)
    async fn close_sprint(
        &self,
        sprint_id: SprintId,
        cmd: CloseSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError>;

    async fn reorder_backlog(
        &self,
        cmd: BacklogReorderCommand,
        actor: ActorContext,
    ) -> Result<Backlog, PlanningError>;

    async fn add_work_item_to_sprint(
        &self,
        cmd: AddWorkItemToSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError>;

    async fn remove_work_item_from_sprint(
        &self,
        cmd: RemoveWorkItemFromSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError>;
}

// =====================================================================
// 端口:PlanningQueryPort(5 方法)
// =====================================================================

/// **Planning 查询端口**
#[async_trait]
pub trait PlanningQueryPort: Send + Sync {
    async fn list_sprints(
        &self,
        q: ListSprintQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Sprint>, PlanningError>;
    async fn get_sprint(
        &self,
        id: SprintId,
        viewer: ActorContext,
    ) -> Result<Sprint, PlanningError>;
    async fn get_backlog(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Backlog, PlanningError>;
    async fn get_roadmap(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Roadmap, PlanningError>;
    async fn get_burndown(
        &self,
        sprint_id: SprintId,
        viewer: ActorContext,
    ) -> Result<BurndownReport, PlanningError>;
}

// =====================================================================
// 仓库端口
// =====================================================================

/// **Planning 仓库端口**
#[async_trait]
pub trait PlanningRepository: Send + Sync {
    async fn insert_sprint(&self, sprint: &Sprint) -> Result<(), PlanningError>;
    async fn find_sprint(&self, id: SprintId) -> Result<Option<Sprint>, PlanningError>;
    async fn save_sprint(&self, sprint: &Sprint) -> Result<(), PlanningError>;
    async fn list_sprints_raw(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<Sprint>, PlanningError>;

    async fn insert_backlog(&self, backlog: &Backlog) -> Result<(), PlanningError>;
    async fn find_backlog(&self, id: BacklogId) -> Result<Option<Backlog>, PlanningError>;
    async fn save_backlog(&self, backlog: &Backlog) -> Result<(), PlanningError>;
    async fn find_backlog_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Backlog>, PlanningError>;

    async fn insert_roadmap(&self, roadmap: &Roadmap) -> Result<(), PlanningError>;
    async fn find_roadmap_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Roadmap>, PlanningError>;
}

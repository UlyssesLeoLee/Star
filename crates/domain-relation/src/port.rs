//! Relation 端口(Port Traits)与命令/查询 DTO

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{CircularDependencyReport, DateRange, Dependency, GanttReport, Relation};
use crate::error::RelationError;
use crate::value_object::{
    ProjectId, RelationId, RelationType, TenantId, UserId, WorkItemId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateRelationCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub source_work_item_id: WorkItemId,
    pub target_work_item_id: WorkItemId,
    pub relation_type: RelationType,
    pub note: Option<String>,
    /// source / target 同 Project 校验(由 service 在 INV-R-03 校验)
    pub same_project: bool,
}

// =====================================================================
// 端口:RelationCommandPort(2 方法)
// =====================================================================

/// **Relation 命令端口**
#[async_trait]
pub trait RelationCommandPort: Send + Sync {
    async fn create_relation(
        &self,
        cmd: CreateRelationCommand,
        actor: ActorContext,
    ) -> Result<Relation, RelationError>;
    async fn delete_relation(
        &self,
        relation_id: RelationId,
        actor: ActorContext,
    ) -> Result<(), RelationError>;
}

// =====================================================================
// 端口:RelationQueryPort(4 方法)
// =====================================================================

/// **Relation 查询端口**
#[async_trait]
pub trait RelationQueryPort: Send + Sync {
    async fn list_by_work_item(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<Relation>, RelationError>;
    async fn list_dependencies(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Dependency, RelationError>;
    async fn detect_circular(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<CircularDependencyReport, RelationError>;
    async fn get_gantt(
        &self,
        work_item_id: WorkItemId,
        range: DateRange,
        viewer: ActorContext,
    ) -> Result<GanttReport, RelationError>;
}

// =====================================================================
// 仓库端口
// =====================================================================

/// **Relation 仓库端口**
#[async_trait]
pub trait RelationRepository: Send + Sync {
    async fn insert(&self, r: &Relation) -> Result<(), RelationError>;
    async fn find_by_id(&self, id: RelationId) -> Result<Option<Relation>, RelationError>;
    async fn delete(&self, id: RelationId) -> Result<(), RelationError>;
    /// 仓库层:按 WorkItem 查(无 tenant 校验)
    async fn list_by_work_item_raw(
        &self,
        work_item_id: WorkItemId,
    ) -> Result<Vec<Relation>, RelationError>;
    async fn list_all_raw(&self) -> Result<Vec<Relation>, RelationError>;
}

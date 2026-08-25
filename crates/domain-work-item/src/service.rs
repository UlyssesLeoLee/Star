//! InMemoryWorkItemService:Phase 2 提供的内存实现
//!
//! 来源: spec/domain-work-item-spec.md §5(实施策略)
//!
//! **目标**:为 `WorkItemCommandPort` + `WorkItemQueryPort` 提供 1-2 个真实可工作的实现,
//! 用于本地集成测试与 P0 演示,不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**:`crates/infrastructure` 提供 SQLx / NATS Adapter 取代本实现;
//! 本内存实现保留供单元测试 + 本地开发使用。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{AcceptanceCriterion, BusinessGoal, Requirement, WorkItem};
use crate::error::WorkItemError;
use crate::event::{EventMeta, WorkItemEvent};
use crate::invariant::{check_invariant_09_status_transition_default, run_invariants, ALL_INVARIANT_CHECKS, DELETE_INVARIANT_CHECKS};
use crate::port::{
    BulkResult, CreateAcceptanceCriterionCommand, CreateRequirementCommand, CreateWorkItemCommand,
    DeleteWorkItemCommand, LinkRepositoryCommand, ListBusinessGoalQuery, ListWorkItemQuery,
    Transition, TransitionStatusCommand, UpdateWorkItemCommand, WorkItemBulkUpdate,
    WorkItemCommandPort, WorkItemQueryPort, WorkItemRepository,
};
use crate::value_object::{
    ProjectId, TenantId, WorkItemId, WorkItemStatus,
};

// =====================================================================
// InMemoryWorkItemService
// =====================================================================

/// **InMemory WorkItem 命令/查询服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryWorkItemService {
    /// WorkItem 存储
    work_items: Arc<RwLock<HashMap<WorkItemId, WorkItem>>>,
    /// Requirement 存储
    requirements: Arc<RwLock<HashMap<uuid::Uuid, Requirement>>>,
    /// AcceptanceCriterion 存储
    acceptance_criteria: Arc<RwLock<HashMap<uuid::Uuid, AcceptanceCriterion>>>,
    /// BusinessGoal 存储
    business_goals: Arc<RwLock<HashMap<uuid::Uuid, BusinessGoal>>>,
    /// 状态机迁移历史
    transitions: Arc<RwLock<Vec<Transition>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<WorkItemEvent>,
}

impl InMemoryWorkItemService {
    /// 创建新的内存服务(返回服务和事件接收端)。
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<WorkItemEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            work_items: Arc::new(RwLock::new(HashMap::new())),
            requirements: Arc::new(RwLock::new(HashMap::new())),
            acceptance_criteria: Arc::new(RwLock::new(HashMap::new())),
            business_goals: Arc::new(RwLock::new(HashMap::new())),
            transitions: Arc::new(RwLock::new(Vec::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)。
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 WorkItem 数量(供测试断言)。
    pub async fn count(&self) -> usize {
        self.work_items.read().await.len()
    }

    /// 校验 actor 与命令的 tenant_id 一致。
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), WorkItemError> {
        if actor.tenant_id != expected {
            return Err(WorkItemError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryWorkItemService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

// 手工 Clone(因为内部字段是 Arc,Clone 便宜)
impl Clone for InMemoryWorkItemService {
    fn clone(&self) -> Self {
        Self {
            work_items: self.work_items.clone(),
            requirements: self.requirements.clone(),
            acceptance_criteria: self.acceptance_criteria.clone(),
            business_goals: self.business_goals.clone(),
            transitions: self.transitions.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// WorkItemCommandPort 实现(完整 8 方法,2 个真实 + 6 个 todo!)
// =====================================================================

#[async_trait]
impl WorkItemCommandPort for InMemoryWorkItemService {
    async fn create_work_item(
        &self,
        cmd: CreateWorkItemCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        // 1. 租户校验(INV-WI-07)
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 构造 WorkItem 实体
        let now = chrono::Utc::now();
        let id = WorkItemId::new();
        let wi = WorkItem {
            id,
            tenant_id: cmd.tenant_id,
            workspace_id: crate::value_object::WorkspaceId::from_uuid(cmd.workspace_id),
            project_id: cmd.project_id,
            work_item_type: cmd.work_item_type,
            work_item_key: cmd.work_item_key,
            title: cmd.title,
            description: cmd.description,
            status: WorkItemStatus::TODO,
            priority: cmd.priority,
            severity: cmd.severity.unwrap_or_default(),
            story_points: cmd.story_points,
            sprint_id: None,
            parent_work_item_id: cmd.parent_work_item_id,
            requirement_ids: Vec::new(),
            acceptance_criterion_ids: Vec::new(),
            repository_ids: Vec::new(),
            worktree_ids: Vec::new(),
            assignee_user_id: None,
            assignee_agent_id: None,
            reporter_user_id: crate::value_object::UserId::from_uuid(cmd.reporter_user_id),
            labels: Vec::new(),
            components: Vec::new(),
            due_date: cmd.due_date,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            version: 1,
        };

        // 3. 执行所有创建时不变量
        run_invariants(ALL_INVARIANT_CHECKS, &wi)?;

        // 4. 持久化
        self.work_items.write().await.insert(id, wi.clone());

        // 5. 发送 Created 事件
        let event = WorkItemEvent::Created(crate::event::WorkItemCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            work_item_id: id,
            project_id: cmd.project_id.into_uuid(),
            work_item_type: cmd.work_item_type.to_string(),
            work_item_key: wi.work_item_key.clone(),
        });
        // 事件发送失败不阻塞主路径(只记入 Internal 错误)
        let _ = self.event_tx.send(event);

        Ok(wi)
    }

    async fn update_work_item(
        &self,
        cmd: UpdateWorkItemCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 取出实体
        let mut store = self.work_items.write().await;
        let wi = store
            .get_mut(&cmd.work_item_id)
            .ok_or(WorkItemError::NotFound(cmd.work_item_id))?;

        // 3. 跨租户防护
        if wi.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        // 4. 软删除不可更新
        if wi.is_deleted() {
            return Err(WorkItemError::InvalidState(
                "WorkItem 已被软删除,不可更新".to_string(),
            ));
        }
        // 5. 乐观锁
        if wi.version != cmd.expected_version {
            return Err(WorkItemError::Conflict(format!(
                "version mismatch: expected={}, actual={}",
                cmd.expected_version, wi.version
            )));
        }

        // 6. 应用变更
        let mut changed = Vec::new();
        if let Some(t) = cmd.title {
            wi.title = t;
            changed.push("title".to_string());
        }
        if let Some(d) = cmd.description {
            wi.description = d;
            changed.push("description".to_string());
        }
        if let Some(p) = cmd.priority {
            wi.priority = p;
            changed.push("priority".to_string());
        }
        if let Some(s) = cmd.severity {
            wi.severity = s;
            changed.push("severity".to_string());
        }
        if let Some(sp) = cmd.story_points {
            wi.story_points = sp;
            changed.push("story_points".to_string());
        }
        if let Some(dd) = cmd.due_date {
            wi.due_date = dd;
            changed.push("due_date".to_string());
        }
        if let Some(au) = cmd.assignee_user_id {
            wi.assignee_user_id = au.map(crate::value_object::UserId::from_uuid);
            changed.push("assignee_user_id".to_string());
        }
        if let Some(aa) = cmd.assignee_agent_id {
            wi.assignee_agent_id = aa.map(crate::value_object::AgentId::from_uuid);
            changed.push("assignee_agent_id".to_string());
        }

        // 7. 不变量校验 + 乐观锁自增
        run_invariants(ALL_INVARIANT_CHECKS, wi)?;
        wi.bump_version();

        // 8. 发送 Updated 事件
        let event = WorkItemEvent::Updated(crate::event::WorkItemUpdated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            work_item_id: wi.id,
            changed_fields: changed,
        });
        let _ = self.event_tx.send(event);

        Ok(wi.clone())
    }

    async fn delete_work_item(
        &self,
        cmd: DeleteWorkItemCommand,
        actor: ActorContext,
    ) -> Result<(), WorkItemError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 取出
        let mut store = self.work_items.write().await;
        let wi = store
            .get(&cmd.work_item_id)
            .ok_or(WorkItemError::NotFound(cmd.work_item_id))?
            .clone();
        if wi.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        // 3. 乐观锁
        if wi.version != cmd.expected_version {
            return Err(WorkItemError::Conflict(format!(
                "version mismatch: expected={}, actual={}",
                cmd.expected_version, wi.version
            )));
        }
        // 4. INV-WI-06:级联检查 Worktree
        run_invariants(DELETE_INVARIANT_CHECKS, &wi)?;
        // 5. 软删除
        let mut wi = wi;
        wi.deleted_at = Some(chrono::Utc::now());
        wi.bump_version();
        store.insert(wi.id, wi.clone());

        // 6. 事件
        let event = WorkItemEvent::Deleted(crate::event::WorkItemDeleted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            work_item_id: wi.id,
        });
        let _ = self.event_tx.send(event);
        Ok(())
    }

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.work_items.write().await;
        let wi = store
            .get_mut(&cmd.work_item_id)
            .ok_or(WorkItemError::NotFound(cmd.work_item_id))?;
        if wi.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        if wi.is_deleted() {
            return Err(WorkItemError::InvalidState(
                "WorkItem 已被软删除,不可迁移状态".to_string(),
            ));
        }
        if wi.version != cmd.expected_version {
            return Err(WorkItemError::Conflict(format!(
                "version mismatch: expected={}, actual={}",
                cmd.expected_version, wi.version
            )));
        }
        // INV-WI-09:状态机兜底(完整校验由 domain-workflow 决定)
        check_invariant_09_status_transition_default(wi, cmd.target_status)?;
        // 记录迁移
        let transition = Transition {
            id: uuid::Uuid::new_v4(),
            tenant_id: cmd.tenant_id,
            work_item_id: wi.id,
            from_status: wi.status,
            to_status: cmd.target_status,
            actor_user_id: actor.user_id.into_uuid(),
            reason: cmd.reason.clone(),
            occurred_at: chrono::Utc::now(),
        };
        self.transitions
            .write()
            .await
            .push(transition.clone());
        let from = wi.status;
        wi.status = cmd.target_status;
        wi.bump_version();
        let to = wi.status;
        // 事件
        let event = WorkItemEvent::StatusChanged(crate::event::WorkItemStatusChanged {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            work_item_id: wi.id,
            from_status: from,
            to_status: to,
        });
        let _ = self.event_tx.send(event);
        Ok(wi.clone())
    }

    async fn bulk_update(
        &self,
        _cmd: WorkItemBulkUpdate,
        _actor: ActorContext,
    ) -> Result<BulkResult, WorkItemError> {
        // Phase 3 完整实现:逐项 update_work_item + 聚合成功/失败
        Err(WorkItemError::Internal(
            "bulk_update 待 Phase 3 完整实现".to_string(),
        ))
    }

    async fn link_repository(
        &self,
        cmd: LinkRepositoryCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.work_items.write().await;
        let wi = store
            .get_mut(&cmd.work_item_id)
            .ok_or(WorkItemError::NotFound(cmd.work_item_id))?;
        if wi.tenant_id != cmd.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        if !wi.repository_ids.contains(&cmd.repository_id) {
            wi.repository_ids.push(cmd.repository_id);
        }
        run_invariants(ALL_INVARIANT_CHECKS, wi)?;
        wi.bump_version();
        Ok(wi.clone())
    }

    async fn create_requirement(
        &self,
        cmd: CreateRequirementCommand,
        actor: ActorContext,
    ) -> Result<Requirement, WorkItemError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let req = Requirement {
            id: crate::value_object::RequirementId::new(),
            tenant_id: cmd.tenant_id,
            business_goal_id: cmd
                .business_goal_id
                .map(crate::value_object::BusinessGoalId::from_uuid),
            statement: cmd.statement,
            rationale: cmd.rationale,
            linked_work_item_ids: cmd.linked_work_item_ids,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            version: 1,
        };
        self.requirements
            .write()
            .await
            .insert(req.id.into_uuid(), req.clone());
        Ok(req)
    }

    async fn create_acceptance_criterion(
        &self,
        cmd: CreateAcceptanceCriterionCommand,
        actor: ActorContext,
    ) -> Result<AcceptanceCriterion, WorkItemError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let ac = AcceptanceCriterion {
            id: crate::value_object::AcceptanceCriterionId::new(),
            tenant_id: cmd.tenant_id,
            work_item_id: cmd.work_item_id,
            requirement_id: cmd.requirement_id,
            statement: cmd.statement,
            coverage_status: crate::entity::CoverageStatus::UNCOVERED,
            covered_by_validation_ids: Vec::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            version: 1,
        };
        self.acceptance_criteria
            .write()
            .await
            .insert(ac.id.into_uuid(), ac.clone());
        Ok(ac)
    }
}

// =====================================================================
// WorkItemQueryPort 实现(P0:3 个核心方法 + 3 个简化版)
// =====================================================================

#[async_trait]
impl WorkItemQueryPort for InMemoryWorkItemService {
    async fn list_by_project(
        &self,
        q: ListWorkItemQuery,
        viewer: ActorContext,
    ) -> Result<Vec<WorkItem>, WorkItemError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let store = self.work_items.read().await;
        let mut out: Vec<WorkItem> = store
            .values()
            .filter(|wi| wi.tenant_id == q.tenant_id && wi.project_id == q.project_id)
            .filter(|wi| q.status.is_none_or(|s| wi.status == s))
            .filter(|wi| q.work_item_type.is_none_or(|t| wi.work_item_type == t))
            .filter(|wi| {
                q.assignee_user_id
                    .is_none_or(|a| wi.assignee_user_id.map(|u| u.into_uuid()) == Some(a))
            })
            .filter(|wi| q.sprint_id.is_none_or(|s| wi.sprint_id.map(|x| x.into_uuid()) == Some(s)))
            .filter(|wi| {
                q.parent_work_item_id
                    .is_none_or(|p| wi.parent_work_item_id == Some(p))
            })
            .filter(|wi| !wi.is_deleted())
            .cloned()
            .collect();
        // 简单分页
        let start = q.offset as usize;
        let end = std::cmp::min(start + q.limit as usize, out.len());
        if start >= out.len() {
            out.clear();
        } else {
            out = out.split_off(start);
            out.truncate(end - start);
        }
        // 按 updated_at DESC
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(out)
    }

    async fn get_by_id(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        let store = self.work_items.read().await;
        let wi = store.get(&id).ok_or(WorkItemError::NotFound(id))?;
        if wi.tenant_id != viewer.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        if wi.is_deleted() {
            return Err(WorkItemError::NotFound(id));
        }
        Ok(wi.clone())
    }

    async fn list_transitions(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<Transition>, WorkItemError> {
        let store = self.work_items.read().await;
        let wi = store.get(&id).ok_or(WorkItemError::NotFound(id))?;
        if wi.tenant_id != viewer.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        let transitions = self.transitions.read().await;
        Ok(transitions
            .iter()
            .filter(|t| t.work_item_id == id)
            .cloned()
            .collect())
    }

    async fn list_requirements(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<Requirement>, WorkItemError> {
        let store = self.work_items.read().await;
        let wi = store.get(&id).ok_or(WorkItemError::NotFound(id))?;
        if wi.tenant_id != viewer.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        let reqs = self.requirements.read().await;
        Ok(reqs
            .values()
            .filter(|r| r.tenant_id == viewer.tenant_id && r.linked_work_item_ids.contains(&id))
            .cloned()
            .collect())
    }

    async fn list_acceptance_criteria(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<AcceptanceCriterion>, WorkItemError> {
        let store = self.work_items.read().await;
        let wi = store.get(&id).ok_or(WorkItemError::NotFound(id))?;
        if wi.tenant_id != viewer.tenant_id {
            return Err(WorkItemError::PermissionDenied);
        }
        let acs = self.acceptance_criteria.read().await;
        Ok(acs.values().filter(|a| a.work_item_id == id).cloned().collect())
    }

    async fn list_business_goals(
        &self,
        q: ListBusinessGoalQuery,
        viewer: ActorContext,
    ) -> Result<Vec<BusinessGoal>, WorkItemError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let bgs = self.business_goals.read().await;
        let mut out: Vec<BusinessGoal> = bgs
            .values()
            .filter(|bg| bg.tenant_id == q.tenant_id && !bg.deleted_at.is_some())
            .cloned()
            .collect();
        let start = q.offset as usize;
        let end = std::cmp::min(start + q.limit as usize, out.len());
        if start >= out.len() {
            out.clear();
        } else {
            out = out.split_off(start);
            out.truncate(end - start);
        }
        Ok(out)
    }
}

// =====================================================================
// WorkItemRepository 实现(供 infra crate 借鉴 / 测试)
// =====================================================================

#[async_trait]
impl WorkItemRepository for InMemoryWorkItemService {
    async fn insert(&self, work_item: &WorkItem) -> Result<(), WorkItemError> {
        self.work_items
            .write()
            .await
            .insert(work_item.id, work_item.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: WorkItemId) -> Result<Option<WorkItem>, WorkItemError> {
        Ok(self.work_items.read().await.get(&id).cloned())
    }

    async fn update(&self, work_item: &WorkItem) -> Result<(), WorkItemError> {
        self.work_items
            .write()
            .await
            .insert(work_item.id, work_item.clone());
        Ok(())
    }

    async fn soft_delete(&self, id: WorkItemId) -> Result<(), WorkItemError> {
        let mut store = self.work_items.write().await;
        if let Some(wi) = store.get_mut(&id) {
            wi.deleted_at = Some(chrono::Utc::now());
            wi.bump_version();
        }
        Ok(())
    }

    async fn list_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<WorkItem>, WorkItemError> {
        Ok(self
            .work_items
            .read()
            .await
            .values()
            .filter(|wi| wi.tenant_id == tenant_id && wi.project_id == project_id && !wi.is_deleted())
            .cloned()
            .collect())
    }
}

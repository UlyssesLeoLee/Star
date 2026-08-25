//! InMemoryPlanningService:Phase 2 提供的内存实现

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{Backlog, BurndownReport, Roadmap, Sprint};
use crate::error::PlanningError;
use crate::event::{
    EventMeta, PlanningEvent, SprintClosed, SprintCreated, SprintStarted, WorkItemAddedToSprint,
};
use crate::invariants::{
    check_create_invariants, check_invariant_01_sprint_state_legal,
    check_invariant_04_no_duplicate_work_item, check_invariant_06_backlog_no_duplicates,
};
use crate::port::{
    AddWorkItemToSprintCommand, BacklogReorderCommand, CloseSprintCommand, CreateSprintCommand,
    ListSprintQuery, PlanningCommandPort, PlanningQueryPort, PlanningRepository,
    RemoveWorkItemFromSprintCommand, UpdateSprintCommand,
};
use crate::value_object::{
    BacklogId, ProjectId, SprintId, SprintState, TenantId, WorkItemId,
};

// =====================================================================
// InMemoryPlanningService
// =====================================================================

/// **InMemory Planning 命令/查询服务**
pub struct InMemoryPlanningService {
    sprints: Arc<RwLock<HashMap<SprintId, Sprint>>>,
    backlogs: Arc<RwLock<HashMap<BacklogId, Backlog>>>,
    /// backlog 按 project_id 索引
    backlogs_by_project: Arc<RwLock<HashMap<ProjectId, BacklogId>>>,
    roadmaps: Arc<RwLock<HashMap<ProjectId, Roadmap>>>,
    event_tx: mpsc::UnboundedSender<PlanningEvent>,
}

impl InMemoryPlanningService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<PlanningEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            sprints: Arc::new(RwLock::new(HashMap::new())),
            backlogs: Arc::new(RwLock::new(HashMap::new())),
            backlogs_by_project: Arc::new(RwLock::new(HashMap::new())),
            roadmaps: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    pub async fn count_sprints(&self) -> usize {
        self.sprints.read().expect("lock").len()
    }

    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), PlanningError> {
        if actor.tenant_id != expected {
            return Err(PlanningError::PermissionDenied);
        }
        Ok(())
    }

    /// 计算同 Project 当前 Active Sprint 数(INV-PL-03)
    fn count_active_sprints(sprints: &HashMap<SprintId, Sprint>, project_id: ProjectId) -> usize {
        sprints
            .values()
            .filter(|s| s.project_id == project_id && s.state == SprintState::Active)
            .count()
    }
}

impl Default for InMemoryPlanningService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryPlanningService {
    fn clone(&self) -> Self {
        Self {
            sprints: self.sprints.clone(),
            backlogs: self.backlogs.clone(),
            backlogs_by_project: self.backlogs_by_project.clone(),
            roadmaps: self.roadmaps.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// PlanningCommandPort 实现
// =====================================================================

#[async_trait]
impl PlanningCommandPort for InMemoryPlanningService {
    async fn create_sprint(
        &self,
        cmd: CreateSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = chrono::Utc::now();
        let sprint = Sprint {
            id: SprintId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name: cmd.name.clone(),
            goal: cmd.goal,
            start_at: cmd.start_at,
            end_at: cmd.end_at,
            state: SprintState::Planning,
            work_item_ids: Vec::new(),
            capacity_story_points: cmd.capacity_story_points,
            created_at: now,
            updated_at: now,
            started_at: None,
            closed_at: None,
            lock_version: 1,
        };
        // INV-PL-02
        check_create_invariants(&sprint, 0)?;
        // 持久化
        self.sprints.write().expect("lock").insert(sprint.id, sprint.clone());

        // 事件
        let event = PlanningEvent::Created(SprintCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            sprint_id: sprint.id,
            project_id: sprint.project_id,
            start_at: sprint.start_at,
            end_at: sprint.end_at,
        });
        let _ = self.event_tx.send(event);
        Ok(sprint)
    }

    async fn update_sprint(
        &self,
        cmd: UpdateSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut sprints = self.sprints.write().expect("lock");
        let s = sprints
            .get_mut(&cmd.sprint_id)
            .ok_or(PlanningError::NotFound(cmd.sprint_id))?;
        if s.tenant_id != cmd.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        if s.lock_version != cmd.expected_version {
            return Err(PlanningError::Conflict(format!(
                "lock_version mismatch: expected={}, actual={}",
                cmd.expected_version, s.lock_version
            )));
        }
        if s.state == SprintState::Closed {
            return Err(PlanningError::InvalidState(
                "Closed Sprint 不可更新".to_string(),
            ));
        }
        let mut changed = false;
        if let Some(n) = cmd.name {
            s.name = n;
            changed = true;
        }
        if let Some(g) = cmd.goal {
            s.goal = g;
            changed = true;
        }
        if let Some(st) = cmd.start_at {
            s.start_at = st;
            changed = true;
        }
        if let Some(et) = cmd.end_at {
            s.end_at = et;
            changed = true;
        }
        if let Some(c) = cmd.capacity_story_points {
            s.capacity_story_points = c;
            changed = true;
        }
        if changed {
            // 重新校验 INV-PL-02(可能 duration 越界)
            crate::invariants::check_invariant_02_sprint_duration(s)?;
            s.bump_version();
        }
        Ok(s.clone())
    }

    async fn start_sprint(
        &self,
        sprint_id: SprintId,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        let mut sprints = self.sprints.write().expect("lock");
        let s = sprints
            .get(&sprint_id)
            .ok_or(PlanningError::NotFound(sprint_id))?
            .clone();
        if s.tenant_id != actor.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        // INV-PL-01
        check_invariant_01_sprint_state_legal(&s, SprintState::Active)?;
        // INV-PL-03:同 Project active count
        let active_count = Self::count_active_sprints(&sprints, s.project_id);
        crate::invariants::check_invariant_03_single_active_sprint(active_count)?;
        // 应用变更
        let mut s = s;
        s.state = SprintState::Active;
        s.started_at = Some(chrono::Utc::now());
        s.bump_version();
        sprints.insert(s.id, s.clone());

        let event = PlanningEvent::Started(SprintStarted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(s.tenant_id)
            },
            sprint_id: s.id,
            started_at: s.started_at.unwrap(),
            work_item_count: s.work_item_ids.len() as u32,
        });
        let _ = self.event_tx.send(event);
        Ok(s)
    }

    async fn close_sprint(
        &self,
        sprint_id: SprintId,
        cmd: CloseSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        let mut sprints = self.sprints.write().expect("lock");
        let s = sprints
            .get(&sprint_id)
            .ok_or(PlanningError::NotFound(sprint_id))?
            .clone();
        if s.tenant_id != actor.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        // INV-PL-01
        check_invariant_01_sprint_state_legal(&s, SprintState::Closed)?;
        let mut s = s;
        s.state = SprintState::Closed;
        s.closed_at = Some(chrono::Utc::now());
        // 处理未完成 WorkItem(简化:仅记录 move target;实际业务由 application 层处理)
        s.bump_version();
        sprints.insert(s.id, s.clone());

        let move_target = match cmd.move_incomplete_to {
            crate::value_object::CloseMoveTarget::Backlog => "BACKLOG".to_string(),
            crate::value_object::CloseMoveTarget::NextSprint => "NEXT_SPRINT".to_string(),
        };
        let event = PlanningEvent::Closed(SprintClosed {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(s.tenant_id)
            },
            sprint_id: s.id,
            closed_at: s.closed_at.unwrap(),
            moved_incomplete_to: move_target,
        });
        let _ = self.event_tx.send(event);
        Ok(s)
    }

    async fn reorder_backlog(
        &self,
        cmd: BacklogReorderCommand,
        actor: ActorContext,
    ) -> Result<Backlog, PlanningError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // INV-PL-06
        check_invariant_06_backlog_no_duplicates(&cmd.work_item_order)?;
        let mut backlogs = self.backlogs.write().expect("lock");
        let b = backlogs
            .get_mut(&cmd.backlog_id)
            .ok_or(PlanningError::NotFound(SprintId::from_uuid(uuid::Uuid::nil())))?;
        if b.tenant_id != cmd.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        b.work_item_order = cmd.work_item_order.clone();
        b.bump_version();

        let event = PlanningEvent::BacklogReordered(crate::event::BacklogReordered {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            project_id: b.project_id,
            new_order: cmd.work_item_order,
        });
        let _ = self.event_tx.send(event);
        Ok(b.clone())
    }

    async fn add_work_item_to_sprint(
        &self,
        cmd: AddWorkItemToSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut sprints = self.sprints.write().expect("lock");
        let s = sprints
            .get_mut(&cmd.sprint_id)
            .ok_or(PlanningError::NotFound(cmd.sprint_id))?;
        if s.tenant_id != cmd.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        if s.state == SprintState::Closed {
            return Err(PlanningError::InvalidState(
                "Closed Sprint 不可添加 WorkItem".to_string(),
            ));
        }
        // INV-PL-04
        check_invariant_04_no_duplicate_work_item(s, cmd.work_item_id)?;
        s.work_item_ids.push(cmd.work_item_id);
        s.bump_version();

        let event = PlanningEvent::WorkItemAdded(WorkItemAddedToSprint {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            sprint_id: s.id,
            work_item_id: cmd.work_item_id,
        });
        let _ = self.event_tx.send(event);
        Ok(s.clone())
    }

    async fn remove_work_item_from_sprint(
        &self,
        cmd: RemoveWorkItemFromSprintCommand,
        actor: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut sprints = self.sprints.write().expect("lock");
        let s = sprints
            .get_mut(&cmd.sprint_id)
            .ok_or(PlanningError::NotFound(cmd.sprint_id))?;
        if s.tenant_id != cmd.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        if s.state == SprintState::Closed {
            return Err(PlanningError::InvalidState(
                "Closed Sprint 不可移除 WorkItem".to_string(),
            ));
        }
        let pos = s
            .work_item_ids
            .iter()
            .position(|wid| *wid == cmd.work_item_id)
            .ok_or_else(|| {
                PlanningError::NotFound(SprintId::from_uuid(uuid::Uuid::nil()))
            })?;
        s.work_item_ids.remove(pos);
        s.bump_version();
        Ok(s.clone())
    }
}

// =====================================================================
// PlanningQueryPort 实现
// =====================================================================

#[async_trait]
impl PlanningQueryPort for InMemoryPlanningService {
    async fn list_sprints(
        &self,
        q: ListSprintQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Sprint>, PlanningError> {
        let sprints = self.sprints.read().expect("lock");
        let mut out: Vec<Sprint> = sprints
            .values()
            .filter(|s| s.tenant_id == q.tenant_id)
            .filter(|s| q.project_id.is_none_or(|p| s.project_id == p))
            .filter(|s| q.state.is_none_or(|st| s.state == st))
            .cloned()
            .collect();
        out.sort_by(|a, b| a.start_at.cmp(&b.start_at));
        let start = q.offset as usize;
        let end = std::cmp::min(start + q.limit as usize, out.len());
        if start >= out.len() {
            out.clear();
        } else {
            out = out.split_off(start);
            out.truncate(end - start);
        }
        // 检查 viewer tenant
        if viewer.tenant_id != q.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        Ok(out)
    }

    async fn get_sprint(
        &self,
        id: SprintId,
        viewer: ActorContext,
    ) -> Result<Sprint, PlanningError> {
        let sprints = self.sprints.read().expect("lock");
        let s = sprints
            .get(&id)
            .ok_or(PlanningError::NotFound(id))?
            .clone();
        if s.tenant_id != viewer.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        Ok(s)
    }

    async fn get_backlog(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Backlog, PlanningError> {
        let map = self.backlogs_by_project.read().expect("lock");
        let bid = map
            .get(&project_id)
            .copied()
            .ok_or_else(|| PlanningError::NotFound(SprintId::from_uuid(uuid::Uuid::nil())))?;
        drop(map);
        let backlogs = self.backlogs.read().expect("lock");
        let b = backlogs.get(&bid).cloned().ok_or_else(|| {
            PlanningError::NotFound(SprintId::from_uuid(uuid::Uuid::nil()))
        })?;
        if b.tenant_id != viewer.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        Ok(b)
    }

    async fn get_roadmap(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Roadmap, PlanningError> {
        let roadmaps = self.roadmaps.read().expect("lock");
        let r = roadmaps
            .get(&project_id)
            .cloned()
            .ok_or_else(|| PlanningError::NotFound(SprintId::from_uuid(uuid::Uuid::nil())))?;
        if r.tenant_id != viewer.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        Ok(r)
    }

    async fn get_burndown(
        &self,
        sprint_id: SprintId,
        viewer: ActorContext,
    ) -> Result<BurndownReport, PlanningError> {
        let sprints = self.sprints.read().expect("lock");
        let s = sprints
            .get(&sprint_id)
            .ok_or(PlanningError::NotFound(sprint_id))?
            .clone();
        if s.tenant_id != viewer.tenant_id {
            return Err(PlanningError::PermissionDenied);
        }
        // BurndownReport:Phase 2 返回空快照(实际由 worker 异步刷新)
        let total = s.capacity_story_points.unwrap_or(0);
        Ok(BurndownReport {
            sprint_id,
            total_story_points: total,
            current_remaining_story_points: total,
            snapshots: Vec::new(),
            generated_at: chrono::Utc::now(),
        })
    }
}

// =====================================================================
// PlanningRepository 实现
// =====================================================================

#[async_trait]
impl PlanningRepository for InMemoryPlanningService {
    async fn insert_sprint(&self, sprint: &Sprint) -> Result<(), PlanningError> {
        self.sprints.write().expect("lock").insert(sprint.id, sprint.clone());
        Ok(())
    }

    async fn find_sprint(&self, id: SprintId) -> Result<Option<Sprint>, PlanningError> {
        Ok(self.sprints.read().expect("lock").get(&id).cloned())
    }

    async fn save_sprint(&self, sprint: &Sprint) -> Result<(), PlanningError> {
        self.sprints.write().expect("lock").insert(sprint.id, sprint.clone());
        Ok(())
    }

    async fn list_sprints_raw(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<Sprint>, PlanningError> {
        let sprints = self.sprints.read().expect("lock");
        Ok(sprints
            .values()
            .filter(|s| project_id.is_none_or(|p| s.project_id == p))
            .cloned()
            .collect())
    }

    async fn insert_backlog(&self, backlog: &Backlog) -> Result<(), PlanningError> {
        self.backlogs_by_project
            .write()
            .expect("lock")
            .insert(backlog.project_id, backlog.id);
        self.backlogs
            .write()
            .expect("lock")
            .insert(backlog.id, backlog.clone());
        Ok(())
    }

    async fn find_backlog(&self, id: BacklogId) -> Result<Option<Backlog>, PlanningError> {
        Ok(self.backlogs.read().expect("lock").get(&id).cloned())
    }

    async fn save_backlog(&self, backlog: &Backlog) -> Result<(), PlanningError> {
        self.backlogs
            .write()
            .expect("lock")
            .insert(backlog.id, backlog.clone());
        Ok(())
    }

    async fn find_backlog_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Backlog>, PlanningError> {
        let map = self.backlogs_by_project.read().expect("lock");
        let bid = match map.get(&project_id) {
            Some(b) => *b,
            None => return Ok(None),
        };
        drop(map);
        Ok(self.backlogs.read().expect("lock").get(&bid).cloned())
    }

    async fn insert_roadmap(&self, roadmap: &Roadmap) -> Result<(), PlanningError> {
        self.roadmaps
            .write()
            .expect("lock")
            .insert(roadmap.project_id, roadmap.clone());
        Ok(())
    }

    async fn find_roadmap_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Roadmap>, PlanningError> {
        Ok(self.roadmaps.read().expect("lock").get(&project_id).cloned())
    }
}

// 静默引用
#[allow(dead_code)]
fn _unused_set<T>() -> HashSet<T> {
    HashSet::new()
}

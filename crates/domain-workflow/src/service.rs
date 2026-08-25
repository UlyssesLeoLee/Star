//! InMemoryWorkflowService:Phase 2 提供的内存实现
//!
//! 来源: spec/domain-workflow-spec.md §5(实施策略)
//!
//! **目标**:为 `WorkflowCommandPort` + `WorkflowQueryPort` + `WorkflowRepository`
//! 提供 1-2 个真实可工作的实现,用于本地集成测试与 P0 演示,
//! 不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**:`crates/infrastructure` 提供 SQLx / NATS Adapter 取代本实现。

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{State, SystemDefault, Transition, WorkflowDefinition};
use crate::error::WorkflowError;
use crate::event::{EventMeta, WorkflowCreated, WorkflowEvent};
use crate::invariants::{
    check_create_invariants, check_invariant_01_system_default_readonly,
    check_invariant_03_transition_distinct, check_invariant_04_state_name_unique,
    check_invariant_05_no_project_reference, check_invariant_06_inherit_default_states,
};
use crate::port::{
    AddStateCommand, AddTransitionCommand, CreateWorkflowCommand, ListStatesQuery,
    ListTransitionsQuery, UpdateWorkflowCommand, WorkflowCommandPort, WorkflowQueryPort,
    WorkflowRepository,
};
use crate::value_object::{StateId, TenantId, TransitionId, WorkflowId};

// =====================================================================
// InMemoryWorkflowService
// =====================================================================

/// **InMemory Workflow 命令/查询服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryWorkflowService {
    /// WorkflowDefinition 存储
    workflows: Arc<RwLock<HashMap<WorkflowId, WorkflowDefinition>>>,
    /// State 存储
    states: Arc<RwLock<HashMap<WorkflowId, HashMap<StateId, State>>>>,
    /// Transition 存储
    transitions: Arc<RwLock<HashMap<WorkflowId, HashMap<TransitionId, Transition>>>>,
    /// Project → Workflow 引用计数(用于 INV-WF-05)
    project_references: Arc<RwLock<HashMap<uuid::Uuid, HashSet<WorkflowId>>>>,
    /// system_default 状态(用 std::sync::Mutex 因为此字段在 new() 同步阶段被初始化)
    system_default: Arc<std::sync::Mutex<Option<SystemDefault>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<WorkflowEvent>,
}

impl InMemoryWorkflowService {
    /// 创建新的内存服务(返回服务和事件接收端),并同步 seed system_default。
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<WorkflowEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            transitions: Arc::new(RwLock::new(HashMap::new())),
            project_references: Arc::new(RwLock::new(HashMap::new())),
            system_default: Arc::new(std::sync::Mutex::new(None)),
            event_tx: tx,
        });
        // 同步 seed system_default(在 std::sync::RwLock 上 write 不会 block runtime)
        let (sd, sd_workflow, sd_states, sd_transitions) = Self::build_system_default_full();
        {
            let mut workflows = svc.workflows.write().expect("workflows lock");
            workflows.insert(sd.workflow_id, sd_workflow);
        }
        {
            let mut states = svc.states.write().expect("states lock");
            let map: HashMap<StateId, State> = sd_states.into_iter().map(|s| (s.id, s)).collect();
            states.insert(sd.workflow_id, map);
        }
        {
            let mut transitions = svc.transitions.write().expect("transitions lock");
            let map: HashMap<TransitionId, Transition> = sd_transitions
                .into_iter()
                .map(|t| (t.id, t))
                .collect();
            transitions.insert(sd.workflow_id, map);
        }
        {
            let mut guard = svc.system_default.lock().expect("system_default mutex");
            *guard = Some(sd);
        }
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)。
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Workflow 数量
    pub async fn count(&self) -> usize {
        self.workflows.read().expect("workflows lock").len()
    }

    /// 校验 actor 与命令的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), WorkflowError> {
        if actor.tenant_id != expected {
            return Err(WorkflowError::PermissionDenied);
        }
        Ok(())
    }

    /// 注册 Project → Workflow 引用(由 application 层调用,INV-WF-05)
    pub async fn add_project_reference(&self, project_id: uuid::Uuid, workflow_id: WorkflowId) {
        let mut refs = self.project_references.write().expect("refs lock");
        refs.entry(project_id).or_default().insert(workflow_id);
    }

    /// 取消 Project → Workflow 引用
    pub async fn remove_project_reference(&self, project_id: uuid::Uuid, workflow_id: WorkflowId) {
        let mut refs = self.project_references.write().expect("refs lock");
        if let Some(set) = refs.get_mut(&project_id) {
            set.remove(&workflow_id);
        }
    }

    /// 一次性生成 system_default 全套数据(共享同一组 UUID,确保 reference 正确)。
    fn build_system_default_full() -> (
        SystemDefault,
        WorkflowDefinition,
        Vec<State>,
        Vec<Transition>,
    ) {
        let workflow_id = WorkflowId::new();
        let todo_id = StateId::new();
        let in_progress_id = StateId::new();
        let done_id = StateId::new();
        let now = chrono::Utc::now();
        let sys_tenant = TenantId::new();

        let sd = SystemDefault {
            workflow_id,
            initial_state_id: todo_id,
            is_system_default: true,
        };

        let sd_workflow = WorkflowDefinition {
            id: workflow_id,
            tenant_id: sys_tenant,
            project_id: None,
            name: "system_default".to_string(),
            description: Some(
                "Platform default 3-state workflow (TODO → IN_PROGRESS → DONE)".to_string(),
            ),
            version: 1,
            is_system_default: true,
            initial_state_id: todo_id,
            created_at: now,
            updated_at: now,
            lock_version: 1,
            created_by_user_id: crate::value_object::UserId::new(),
        };

        let sd_states = vec![
            State {
                id: todo_id,
                workflow_id,
                tenant_id: sys_tenant,
                name: "TODO".to_string(),
                category: crate::value_object::StateCategory::Initial,
                display_color: Some("#999999".to_string()),
                display_order: 0,
                created_at: now,
                updated_at: now,
            },
            State {
                id: in_progress_id,
                workflow_id,
                tenant_id: sys_tenant,
                name: "IN_PROGRESS".to_string(),
                category: crate::value_object::StateCategory::Intermediate,
                display_color: Some("#0066cc".to_string()),
                display_order: 1,
                created_at: now,
                updated_at: now,
            },
            State {
                id: done_id,
                workflow_id,
                tenant_id: sys_tenant,
                name: "DONE".to_string(),
                category: crate::value_object::StateCategory::Terminal,
                display_color: Some("#00aa66".to_string()),
                display_order: 2,
                created_at: now,
                updated_at: now,
            },
        ];

        let sd_transitions = vec![
            Transition {
                id: TransitionId::new(),
                workflow_id,
                tenant_id: sys_tenant,
                from_state_id: todo_id,
                to_state_id: in_progress_id,
                required_permission: None,
                required_role: None,
                trigger_event: None,
            },
            Transition {
                id: TransitionId::new(),
                workflow_id,
                tenant_id: sys_tenant,
                from_state_id: in_progress_id,
                to_state_id: done_id,
                required_permission: None,
                required_role: None,
                trigger_event: None,
            },
        ];

        (sd, sd_workflow, sd_states, sd_transitions)
    }
}

impl Default for InMemoryWorkflowService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

// 手工 Clone(因为内部字段是 Arc,Clone 便宜)
impl Clone for InMemoryWorkflowService {
    fn clone(&self) -> Self {
        Self {
            workflows: self.workflows.clone(),
            states: self.states.clone(),
            transitions: self.transitions.clone(),
            project_references: self.project_references.clone(),
            system_default: self.system_default.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// 静默 OnceLock(未使用,保留供未来同步扩展)
#[allow(dead_code)]
fn _unused_once_lock() {
    // placeholder
}

// =====================================================================
// WorkflowCommandPort 实现(5 方法)
// =====================================================================

#[async_trait]
impl WorkflowCommandPort for InMemoryWorkflowService {
    async fn create_workflow(
        &self,
        cmd: CreateWorkflowCommand,
        actor: ActorContext,
    ) -> Result<WorkflowDefinition, WorkflowError> {
        // 1. 租户校验(INV-WF-07 由 §6.1 隐式要求)
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. draft_id → state_id 映射(草稿聚合,稍后落地为真实 State)
        let mut draft_to_state: HashMap<uuid::Uuid, StateId> = HashMap::new();
        for d in &cmd.states {
            draft_to_state.insert(d.draft_id, StateId::new());
        }

        // 3. 构造 State 列表
        let now = chrono::Utc::now();
        let states: Vec<State> = cmd
            .states
            .iter()
            .map(|d| State {
                id: draft_to_state[&d.draft_id],
                workflow_id: WorkflowId::from_uuid(uuid::Uuid::nil()), // 稍后覆盖
                tenant_id: cmd.tenant_id,
                name: d.name.clone(),
                category: d.category,
                display_color: d.display_color.clone(),
                display_order: d.display_order,
                created_at: now,
                updated_at: now,
            })
            .collect();

        // 4. 解析 initial_state_id
        let initial_state_id = *draft_to_state
            .get(&cmd.initial_state_draft_id)
            .ok_or_else(|| {
                WorkflowError::InvalidState(format!(
                    "initial_state_draft_id {} 不在 states 列表中",
                    cmd.initial_state_draft_id
                ))
            })?;

        // 5. 构造 WorkflowDefinition
        let workflow_id = WorkflowId::new();
        let wf = WorkflowDefinition {
            id: workflow_id,
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name: cmd.name.clone(),
            description: cmd.description.clone(),
            version: 1,
            is_system_default: false,
            initial_state_id,
            created_at: now,
            updated_at: now,
            lock_version: 1,
            created_by_user_id: actor.user_id,
        };

        // 6. 创建时不变量检查(INV-WF-04,06,02,01)
        check_create_invariants(&wf, &states)?;

        // 7. 构造 Transition 列表(把 draft_id 替换为真实 state_id)
        let transitions: Vec<Transition> = cmd
            .transitions
            .iter()
            .map(|d| {
                let from = *draft_to_state.get(&d.from_draft_id).ok_or_else(|| {
                    WorkflowError::InvalidState(format!(
                        "Transition.from_draft_id {} 未在 states 中定义",
                        d.from_draft_id
                    ))
                })?;
                let to = *draft_to_state.get(&d.to_draft_id).ok_or_else(|| {
                    WorkflowError::InvalidState(format!(
                        "Transition.to_draft_id {} 未在 states 中定义",
                        d.to_draft_id
                    ))
                })?;
                Ok::<_, WorkflowError>(Transition {
                    id: TransitionId::new(),
                    workflow_id,
                    tenant_id: cmd.tenant_id,
                    from_state_id: from,
                    to_state_id: to,
                    required_permission: d.required_permission.clone(),
                    required_role: d.required_role.clone(),
                    trigger_event: d.trigger_event.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 8. 每个 Transition 检查 INV-WF-03
        for t in &transitions {
            check_invariant_03_transition_distinct(t)?;
        }

        // 9. 持久化
        let mut state_map: HashMap<StateId, State> = HashMap::new();
        for mut s in states {
            s.workflow_id = workflow_id;
            state_map.insert(s.id, s);
        }
        let mut trans_map: HashMap<TransitionId, Transition> = HashMap::new();
        for t in transitions {
            trans_map.insert(t.id, t);
        }
        self.workflows.write().expect("lock").insert(workflow_id, wf.clone());
        self.states.write().expect("lock").insert(workflow_id, state_map);
        self.transitions.write().expect("lock").insert(workflow_id, trans_map);

        // 10. 事件
        let event = WorkflowEvent::Created(WorkflowCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            workflow_id,
            project_id: cmd.project_id.map(|p| p.into_uuid()).map(crate::value_object::ProjectId::from_uuid),
            is_system_default: false,
        });
        let _ = self.event_tx.send(event);

        Ok(wf)
    }

    async fn update_workflow(
        &self,
        cmd: UpdateWorkflowCommand,
        actor: ActorContext,
    ) -> Result<WorkflowDefinition, WorkflowError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut store = self.workflows.write().expect("lock");
        let wf = store
            .get_mut(&cmd.workflow_id)
            .ok_or(WorkflowError::NotFound(cmd.workflow_id))?;
        if wf.tenant_id != cmd.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        // INV-WF-01
        check_invariant_01_system_default_readonly(wf)?;
        // 乐观锁
        if wf.lock_version != cmd.expected_version {
            return Err(WorkflowError::Conflict(format!(
                "lock_version mismatch: expected={}, actual={}",
                cmd.expected_version, wf.lock_version
            )));
        }
        // 应用变更
        if let Some(n) = cmd.name {
            wf.name = n;
        }
        if let Some(d) = cmd.description {
            wf.description = Some(d);
        }
        wf.bump_version();

        let event = WorkflowEvent::Updated(crate::event::WorkflowUpdated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            workflow_id: wf.id,
            version: wf.lock_version,
            updated_at: wf.updated_at,
        });
        let _ = self.event_tx.send(event);

        Ok(wf.clone())
    }

    async fn delete_workflow(
        &self,
        workflow_id: WorkflowId,
        actor: ActorContext,
    ) -> Result<(), WorkflowError> {
        let mut store = self.workflows.write().expect("lock");
        let wf = store
            .get(&workflow_id)
            .ok_or(WorkflowError::NotFound(workflow_id))?
            .clone();
        if wf.tenant_id != actor.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        // INV-WF-05(包含 INV-WF-01):引用计数检查
        let refs = self.project_references.read().expect("lock");
        let total_refs: usize = refs
            .values()
            .map(|set| if set.contains(&workflow_id) { 1 } else { 0 })
            .sum();
        check_invariant_05_no_project_reference(&wf, total_refs)?;

        // 删除关联 State / Transition
        store.remove(&workflow_id);
        self.states.write().expect("lock").remove(&workflow_id);
        self.transitions.write().expect("lock").remove(&workflow_id);

        let event = WorkflowEvent::Deleted(crate::event::WorkflowDeleted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(wf.tenant_id)
            },
            workflow_id,
        });
        let _ = self.event_tx.send(event);

        Ok(())
    }

    async fn add_state(
        &self,
        cmd: AddStateCommand,
        actor: ActorContext,
    ) -> Result<State, WorkflowError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let store = self.workflows.read().expect("lock");
        let wf = store
            .get(&cmd.workflow_id)
            .ok_or(WorkflowError::NotFound(cmd.workflow_id))?;
        if wf.tenant_id != cmd.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        // INV-WF-01:system_default 不可改
        check_invariant_01_system_default_readonly(wf)?;
        drop(store);

        let now = chrono::Utc::now();
        let state = State {
            id: StateId::new(),
            workflow_id: cmd.workflow_id,
            tenant_id: cmd.tenant_id,
            name: cmd.name.clone(),
            category: cmd.category,
            display_color: cmd.display_color,
            display_order: cmd.display_order,
            created_at: now,
            updated_at: now,
        };
        // INV-WF-04
        let mut states_map = self.states.write().expect("lock");
        let map = states_map
            .entry(cmd.workflow_id)
            .or_insert_with(HashMap::new);
        let existing: Vec<State> = map.values().cloned().collect();
        let mut all = existing;
        all.push(state.clone());
        check_invariant_04_state_name_unique(&all)?;
        // INV-WF-06(如果新 state 不在 {TODO,IN_PROGRESS,DONE} 则不影响,但 add 后必须
        // 再次通过整体校验,这里仅校验名称 UNIQUE)
        map.insert(state.id, state.clone());

        let event = WorkflowEvent::StateAdded(crate::event::StateAdded {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            state_id: state.id,
            workflow_id: cmd.workflow_id,
            name: state.name.clone(),
            category: state.category.to_string(),
        });
        let _ = self.event_tx.send(event);

        Ok(state)
    }

    async fn add_transition(
        &self,
        cmd: AddTransitionCommand,
        actor: ActorContext,
    ) -> Result<Transition, WorkflowError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let store = self.workflows.read().expect("lock");
        let wf = store
            .get(&cmd.workflow_id)
            .ok_or(WorkflowError::NotFound(cmd.workflow_id))?;
        if wf.tenant_id != cmd.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        check_invariant_01_system_default_readonly(wf)?;
        drop(store);

        // 校验 from / to 存在
        let states_map = self.states.read().expect("lock");
        let states = states_map
            .get(&cmd.workflow_id)
            .ok_or_else(|| WorkflowError::InvalidState("Workflow 无 State".to_string()))?;
        if !states.contains_key(&cmd.from_state_id) {
            return Err(WorkflowError::InvalidState(format!(
                "from_state_id {} 不存在",
                cmd.from_state_id
            )));
        }
        if !states.contains_key(&cmd.to_state_id) {
            return Err(WorkflowError::InvalidState(format!(
                "to_state_id {} 不存在",
                cmd.to_state_id
            )));
        }
        drop(states_map);

        let transition = Transition {
            id: TransitionId::new(),
            workflow_id: cmd.workflow_id,
            tenant_id: cmd.tenant_id,
            from_state_id: cmd.from_state_id,
            to_state_id: cmd.to_state_id,
            required_permission: cmd.required_permission,
            required_role: cmd.required_role,
            trigger_event: cmd.trigger_event,
        };
        // INV-WF-03
        check_invariant_03_transition_distinct(&transition)?;

        // 持久化
        let mut trans_map = self.transitions.write().expect("lock");
        trans_map
            .entry(cmd.workflow_id)
            .or_insert_with(HashMap::new)
            .insert(transition.id, transition.clone());

        let event = WorkflowEvent::TransitionAdded(crate::event::TransitionAdded {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            transition_id: transition.id,
            workflow_id: cmd.workflow_id,
            from_state_id: transition.from_state_id,
            to_state_id: transition.to_state_id,
        });
        let _ = self.event_tx.send(event);

        Ok(transition)
    }
}

// =====================================================================
// WorkflowQueryPort 实现(5 方法)
// =====================================================================

#[async_trait]
impl WorkflowQueryPort for InMemoryWorkflowService {
    async fn get_by_id(
        &self,
        id: WorkflowId,
        viewer: ActorContext,
    ) -> Result<WorkflowDefinition, WorkflowError> {
        let store = self.workflows.read().expect("lock");
        let wf = store
            .get(&id)
            .ok_or(WorkflowError::NotFound(id))?
            .clone();
        // system_default 跨租户可见;其他要求 tenant 一致
        if !wf.is_system_default && wf.tenant_id != viewer.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        Ok(wf)
    }

    async fn list_states(
        &self,
        q: ListStatesQuery,
        viewer: ActorContext,
    ) -> Result<Vec<State>, WorkflowError> {
        let store = self.workflows.read().expect("lock");
        let wf = store
            .get(&q.workflow_id)
            .ok_or(WorkflowError::NotFound(q.workflow_id))?;
        if !wf.is_system_default && wf.tenant_id != q.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        if !wf.is_system_default && wf.tenant_id != viewer.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        drop(store);
        let states_map = self.states.read().expect("lock");
        let map = states_map.get(&q.workflow_id);
        let mut out: Vec<State> = match map {
            Some(m) => m.values().cloned().collect(),
            None => Vec::new(),
        };
        out.sort_by_key(|s| s.display_order);
        Ok(out)
    }

    async fn list_transitions(
        &self,
        q: ListTransitionsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Transition>, WorkflowError> {
        let store = self.workflows.read().expect("lock");
        let wf = store
            .get(&q.workflow_id)
            .ok_or(WorkflowError::NotFound(q.workflow_id))?;
        if !wf.is_system_default && wf.tenant_id != q.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        if !wf.is_system_default && wf.tenant_id != viewer.tenant_id {
            return Err(WorkflowError::PermissionDenied);
        }
        drop(store);
        let trans_map = self.transitions.read().expect("lock");
        Ok(trans_map
            .get(&q.workflow_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn validate_transition(
        &self,
        workflow_id: WorkflowId,
        from: StateId,
        to: StateId,
    ) -> Result<bool, WorkflowError> {
        let trans_map = self.transitions.read().expect("lock");
        let map = trans_map.get(&workflow_id);
        if let Some(m) = map {
            for t in m.values() {
                if t.from_state_id == from && t.to_state_id == to {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    async fn get_system_default(&self) -> Result<WorkflowDefinition, WorkflowError> {
        let sd_id = {
            let guard = self.system_default.lock().expect("system_default mutex");
            guard.as_ref().map(|sd| sd.workflow_id).ok_or_else(|| {
                WorkflowError::Internal("system_default 未初始化".to_string())
            })?
        };
        let store = self.workflows.read().expect("lock");
        store.get(&sd_id).cloned().ok_or_else(|| {
            WorkflowError::Internal("system_default WorkflowDefinition 缺失".to_string())
        })
    }
}

// =====================================================================
// WorkflowRepository 实现(供 application / infrastructure 适配)
// =====================================================================

#[async_trait]
impl WorkflowRepository for InMemoryWorkflowService {
    async fn insert(&self, wf: &WorkflowDefinition) -> Result<(), WorkflowError> {
        self.workflows.write().expect("lock").insert(wf.id, wf.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: WorkflowId) -> Result<Option<WorkflowDefinition>, WorkflowError> {
        Ok(self.workflows.read().expect("lock").get(&id).cloned())
    }

    async fn update(&self, wf: &WorkflowDefinition) -> Result<(), WorkflowError> {
        self.workflows.write().expect("lock").insert(wf.id, wf.clone());
        Ok(())
    }

    async fn delete(&self, id: WorkflowId) -> Result<(), WorkflowError> {
        self.workflows.write().expect("lock").remove(&id);
        self.states.write().expect("lock").remove(&id);
        self.transitions.write().expect("lock").remove(&id);
        Ok(())
    }

    async fn list_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<WorkflowDefinition>, WorkflowError> {
        let store = self.workflows.read().expect("lock");
        Ok(store
            .values()
            .filter(|wf| wf.is_system_default || wf.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn find_system_default(&self) -> Result<Option<WorkflowDefinition>, WorkflowError> {
        let sd_id_opt = {
            let guard = self.system_default.lock().expect("system_default mutex");
            guard.as_ref().map(|sd| sd.workflow_id)
        };
        if let Some(sd_id) = sd_id_opt {
            let store = self.workflows.read().expect("lock");
            return Ok(store.get(&sd_id).cloned());
        }
        Ok(None)
    }

    async fn insert_state(&self, state: &State) -> Result<(), WorkflowError> {
        self.states
            .write()
            .expect("lock")
            .entry(state.workflow_id)
            .or_insert_with(HashMap::new)
            .insert(state.id, state.clone());
        Ok(())
    }

    async fn list_states_raw(&self, workflow_id: WorkflowId) -> Result<Vec<State>, WorkflowError> {
        let map = self.states.read().expect("lock");
        Ok(map
            .get(&workflow_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn insert_transition(&self, t: &Transition) -> Result<(), WorkflowError> {
        self.transitions
            .write()
            .expect("lock")
            .entry(t.workflow_id)
            .or_insert_with(HashMap::new)
            .insert(t.id, t.clone());
        Ok(())
    }

    async fn list_transitions_raw(&self, workflow_id: WorkflowId) -> Result<Vec<Transition>, WorkflowError> {
        let map = self.transitions.read().expect("lock");
        Ok(map
            .get(&workflow_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }
}

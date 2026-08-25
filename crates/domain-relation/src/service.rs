//! InMemoryRelationService:Phase 2 内存实现
//!
//! 关键算法:
//! - 循环依赖检测(DFS 找环)
//! - 依赖闭包(直接 + 传递)

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{CircularDependencyReport, DateRange, Dependency, GanttReport, Relation};
use crate::error::RelationError;
use crate::event::{
    CircularDependencyDetected, EventMeta, RelationCreated, RelationDeleted, RelationEvent,
};
use crate::invariants::{
    check_create_invariants, check_invariant_01_source_not_target, check_invariant_02_unique,
    check_invariant_04_no_cycle,
};
use crate::port::{
    CreateRelationCommand, RelationCommandPort, RelationQueryPort, RelationRepository,
};
use crate::value_object::{ProjectId, RelationId, RelationType, TenantId, WorkItemId};

// =====================================================================
// InMemoryRelationService
// =====================================================================

pub struct InMemoryRelationService {
    relations: Arc<RwLock<HashMap<RelationId, Relation>>>,
    /// project_id 索引(简化:Relation 自带 project_id)
    event_tx: mpsc::UnboundedSender<RelationEvent>,
}

impl InMemoryRelationService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<RelationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            relations: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    pub async fn count(&self) -> usize {
        self.relations.read().expect("lock").len()
    }

    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), RelationError> {
        if actor.tenant_id != expected {
            return Err(RelationError::PermissionDenied);
        }
        Ok(())
    }

    /// 邻接表:WorkItemId → 其 blocks 目标列表(用于 DFS)
    fn build_blocks_adj(rels: &[Relation]) -> HashMap<WorkItemId, Vec<WorkItemId>> {
        let mut adj: HashMap<WorkItemId, Vec<WorkItemId>> = HashMap::new();
        for r in rels {
            // 仅 Blocks/BlockedBy 参与依赖图(其他 relates_to/duplicates/clones 不构成依赖环)
            if matches!(r.relation_type, RelationType::Blocks | RelationType::BlockedBy) {
                let (from, to) = match r.relation_type {
                    RelationType::Blocks => (r.source_work_item_id, r.target_work_item_id),
                    // BlockedBy 反向:A blocked_by B 等价 B blocks A
                    RelationType::BlockedBy => (r.target_work_item_id, r.source_work_item_id),
                    _ => unreachable!(),
                };
                adj.entry(from).or_default().push(to);
            }
        }
        adj
    }

    /// DFS 检测循环(从 start 出发)
    fn detect_cycle_dfs(
        adj: &HashMap<WorkItemId, Vec<WorkItemId>>,
        start: WorkItemId,
    ) -> (bool, Vec<WorkItemId>) {
        let mut visited: HashSet<WorkItemId> = HashSet::new();
        let mut stack: Vec<(WorkItemId, Vec<WorkItemId>)> = vec![(start, vec![start])];
        while let Some((node, path)) = stack.pop() {
            if let Some(neighbors) = adj.get(&node) {
                for &next in neighbors {
                    if next == start {
                        // 找到环
                        let mut cycle = path.clone();
                        cycle.push(next);
                        return (true, cycle);
                    }
                    if !visited.contains(&next) {
                        visited.insert(next);
                        let mut new_path = path.clone();
                        new_path.push(next);
                        stack.push((next, new_path));
                    }
                }
            }
        }
        (false, vec![])
    }
}

impl Default for InMemoryRelationService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryRelationService {
    fn clone(&self) -> Self {
        Self {
            relations: self.relations.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// RelationCommandPort 实现
// =====================================================================

#[async_trait]
impl RelationCommandPort for InMemoryRelationService {
    async fn create_relation(
        &self,
        cmd: CreateRelationCommand,
        actor: ActorContext,
    ) -> Result<Relation, RelationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // INV-R-01
        check_invariant_01_source_not_target(cmd.source_work_item_id, cmd.target_work_item_id)?;
        // INV-R-03:同 Project
        if !cmd.same_project {
            return Err(RelationError::InvalidState(
                "INV-R-03 (R-003): source 与 target 跨 Project".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        let r = Relation {
            id: RelationId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            source_work_item_id: cmd.source_work_item_id,
            target_work_item_id: cmd.target_work_item_id,
            relation_type: cmd.relation_type,
            created_by_user_id: actor.user_id,
            created_at: now,
            note: cmd.note,
        };

        // 读取全部 + INV-R-02
        let rels: Vec<Relation> = self.relations.read().expect("lock").values().cloned().collect();
        check_invariant_02_unique(&rels, r.source_work_item_id, r.target_work_item_id, r.relation_type)?;

        // 模拟"加上后"状态
        let mut test_rels = rels.clone();
        test_rels.push(r.clone());

        // INV-R-04:DFS 找环
        let adj = Self::build_blocks_adj(&test_rels);
        let (has_cycle, cycle) = Self::detect_cycle_dfs(&adj, r.source_work_item_id);
        if has_cycle {
            // 发出循环检测事件
            let evt = RelationEvent::CircularDetected(CircularDependencyDetected {
                meta: EventMeta {
                    actor_user_id: Some(actor.user_id.into_uuid()),
                    ..EventMeta::new(cmd.tenant_id)
                },
                work_item_id: r.source_work_item_id,
                cycle: cycle.clone(),
            });
            let _ = self.event_tx.send(evt);
            return Err(check_invariant_04_no_cycle(has_cycle, &cycle).unwrap_err());
        }

        // INV-R-05/06 占位
        check_create_invariants(&r)?;

        // 持久化
        self.relations
            .write()
            .expect("lock")
            .insert(r.id, r.clone());

        // 事件
        let evt = RelationEvent::Created(RelationCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            relation_id: r.id,
            source_id: r.source_work_item_id,
            target_id: r.target_work_item_id,
            relation_type: r.relation_type,
        });
        let _ = self.event_tx.send(evt);

        Ok(r)
    }

    async fn delete_relation(
        &self,
        relation_id: RelationId,
        actor: ActorContext,
    ) -> Result<(), RelationError> {
        let mut rels = self.relations.write().expect("lock");
        let r = rels
            .get(&relation_id)
            .ok_or(RelationError::NotFound(relation_id))?
            .clone();
        if r.tenant_id != actor.tenant_id {
            return Err(RelationError::PermissionDenied);
        }
        rels.remove(&relation_id);

        let evt = RelationEvent::Deleted(RelationDeleted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(r.tenant_id)
            },
            relation_id: r.id,
            source_id: r.source_work_item_id,
            target_id: r.target_work_item_id,
        });
        let _ = self.event_tx.send(evt);
        Ok(())
    }
}

// =====================================================================
// RelationQueryPort 实现
// =====================================================================

#[async_trait]
impl RelationQueryPort for InMemoryRelationService {
    async fn list_by_work_item(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<Relation>, RelationError> {
        let rels = self.relations.read().expect("lock");
        let out: Vec<Relation> = rels
            .values()
            .filter(|r| {
                r.tenant_id == viewer.tenant_id
                    && (r.source_work_item_id == work_item_id
                        || r.target_work_item_id == work_item_id)
            })
            .cloned()
            .collect();
        Ok(out)
    }

    async fn list_dependencies(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Dependency, RelationError> {
        let rels: Vec<Relation> = self
            .relations
            .read()
            .expect("lock")
            .values()
            .filter(|r| r.tenant_id == viewer.tenant_id)
            .cloned()
            .collect();
        // 直接依赖:A blocks B → A 依赖 B 的下游
        let direct: Vec<WorkItemId> = rels
            .iter()
            .filter(|r| {
                r.source_work_item_id == work_item_id && r.relation_type == RelationType::Blocks
            })
            .map(|r| r.target_work_item_id)
            .collect();
        // 传递闭包
        let adj = Self::build_blocks_adj(&rels);
        let mut transitive: HashSet<WorkItemId> = HashSet::new();
        let mut stack: Vec<WorkItemId> = direct.clone();
        while let Some(node) = stack.pop() {
            if let Some(neighbors) = adj.get(&node) {
                for &n in neighbors {
                    if transitive.insert(n) {
                        stack.push(n);
                    }
                }
            }
        }
        // 循环检测
        let (has_cycle, _) = Self::detect_cycle_dfs(&adj, work_item_id);
        Ok(Dependency {
            work_item_id,
            direct_dependencies: direct,
            transitive_dependencies: transitive.into_iter().collect(),
            is_circular: has_cycle,
        })
    }

    async fn detect_circular(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<CircularDependencyReport, RelationError> {
        let rels: Vec<Relation> = self
            .relations
            .read()
            .expect("lock")
            .values()
            .filter(|r| r.tenant_id == viewer.tenant_id)
            .cloned()
            .collect();
        let adj = Self::build_blocks_adj(&rels);
        let (has_cycle, cycle) = Self::detect_cycle_dfs(&adj, work_item_id);
        Ok(CircularDependencyReport {
            work_item_id,
            cycle,
            is_circular: has_cycle,
        })
    }

    async fn get_gantt(
        &self,
        work_item_id: WorkItemId,
        _range: DateRange,
        viewer: ActorContext,
    ) -> Result<GanttReport, RelationError> {
        let rels: Vec<Relation> = self
            .relations
            .read()
            .expect("lock")
            .values()
            .filter(|r| r.tenant_id == viewer.tenant_id)
            .cloned()
            .collect();
        let deps: Vec<WorkItemId> = rels
            .iter()
            .filter(|r| r.source_work_item_id == work_item_id)
            .map(|r| r.target_work_item_id)
            .collect();
        // 简化:critical_path 判定 = 是否有 blocks 关系
        let is_critical_path = !deps.is_empty();
        Ok(GanttReport {
            work_item_id,
            start_date: None,
            due_date: None,
            dependencies: deps,
            is_critical_path,
        })
    }
}

// =====================================================================
// RelationRepository 实现
// =====================================================================

#[async_trait]
impl RelationRepository for InMemoryRelationService {
    async fn insert(&self, r: &Relation) -> Result<(), RelationError> {
        self.relations
            .write()
            .expect("lock")
            .insert(r.id, r.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: RelationId) -> Result<Option<Relation>, RelationError> {
        Ok(self.relations.read().expect("lock").get(&id).cloned())
    }

    async fn delete(&self, id: RelationId) -> Result<(), RelationError> {
        self.relations.write().expect("lock").remove(&id);
        Ok(())
    }

    async fn list_by_work_item_raw(
        &self,
        work_item_id: WorkItemId,
    ) -> Result<Vec<Relation>, RelationError> {
        Ok(self
            .relations
            .read()
            .expect("lock")
            .values()
            .filter(|r| {
                r.source_work_item_id == work_item_id || r.target_work_item_id == work_item_id
            })
            .cloned()
            .collect())
    }

    async fn list_all_raw(&self) -> Result<Vec<Relation>, RelationError> {
        Ok(self
            .relations
            .read()
            .expect("lock")
            .values()
            .cloned()
            .collect())
    }
}

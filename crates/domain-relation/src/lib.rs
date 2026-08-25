//! WorkItem 关系领域
//!
//! **crate**: `domain-relation`
//! **上游 spec**: docs/specs/domain-relation-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.9.4
//! **数据设计**: docs/data-design.md §4.8 (`relation` schema)
//! **API 设计**: docs/api-design.md §3.9 (Relation / Dependency)
//!
//! ## 职责
//!
//! WorkItem 间关系与依赖图(§10, REQ-COLLAB-002):
//! - 1 个核心实体(`Relation`) + 3 个派生 Projection(`Dependency` / `CircularDependencyReport` / `GanttReport`)
//! - 3 个核心 Domain Event
//! - 2 个端口(`RelationCommandPort` × 2 / `RelationQueryPort` × 4) + 1 个仓库端口
//! - 6 条不变量(INV-R-01~06)
//! - 1 个 `InMemoryRelationService` 真实实现
//!
//! ## 关键不变量
//!
//! - source ≠ target(INV-R-01,§4.8)
//! - 同 (source, target, type) UNIQUE(INV-R-02,§4.8)
//! - 创建不引入循环(INV-R-03/04,§4.9.4,DFS 检测)
//! - 删除不级联(INV-R-05,§5.7)

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

pub use context::ActorContext;
pub use entity::{
    CircularDependencyReport, DateRange, Dependency, GanttReport, Relation,
};
pub use error::RelationError;
pub use event::{
    CircularDependencyDetected, EventMeta, RelationCreated, RelationDeleted, RelationEvent,
};
pub use invariants::{
    check_create_invariants, check_invariant_01_source_not_target, check_invariant_02_unique,
    check_invariant_03_same_project_placeholder, check_invariant_04_no_cycle,
    check_invariant_05_no_cascade_placeholder, check_invariant_06_enum_placeholder,
    run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    CreateRelationCommand, RelationCommandPort, RelationQueryPort, RelationRepository,
};
pub use service::InMemoryRelationService;
pub use value_object::{
    roles, ProjectId, RelationId, RelationType, TenantId, UserId, WorkItemId,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{ProjectId, RelationType, TenantId, UserId, WorkItemId};

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id).with_role(roles::DEVELOPER)
    }

    fn make_create_cmd(
        tenant_id: TenantId,
        source: WorkItemId,
        target: WorkItemId,
        rt: RelationType,
    ) -> CreateRelationCommand {
        CreateRelationCommand {
            tenant_id,
            project_id: ProjectId::new(),
            source_work_item_id: source,
            target_work_item_id: target,
            relation_type: rt,
            note: None,
            same_project: true,
        }
    }

    // -------- 1. ActorContext smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(actor.has_role(roles::DEVELOPER));
    }

    // -------- 2. 字段数审计 --------

    #[test]
    fn field_count_audit() {
        assert_eq!(Relation::FIELD_COUNT, 9);
        assert_eq!(Dependency::FIELD_COUNT, 4);
        assert_eq!(CircularDependencyReport::FIELD_COUNT, 3);
        assert_eq!(GanttReport::FIELD_COUNT, 5);
    }

    // -------- 3. create_relation 成功路径 --------

    #[tokio::test]
    async fn create_relation_success() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(
            tenant_id,
            WorkItemId::new(),
            WorkItemId::new(),
            RelationType::RelatesTo,
        );
        let r = svc.create_relation(cmd, actor).await.expect("创建成功");
        assert_eq!(svc.count().await, 1);
        assert_eq!(r.relation_type, RelationType::RelatesTo);
    }

    // -------- 4. INV-R-01:source == target 自关系禁止 --------

    #[tokio::test]
    async fn invariant_01_source_equals_target() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let wi = WorkItemId::new();
        let cmd = make_create_cmd(tenant_id, wi, wi, RelationType::Blocks);
        let res = svc.create_relation(cmd, actor).await;
        assert!(matches!(res, Err(RelationError::InvalidState(_))));
    }

    // -------- 5. INV-R-02:重复关系被拒 --------

    #[tokio::test]
    async fn invariant_02_duplicate_relation() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let src = WorkItemId::new();
        let tgt = WorkItemId::new();
        svc.create_relation(
            make_create_cmd(tenant_id, src, tgt, RelationType::RelatesTo),
            actor.clone(),
        )
        .await
        .unwrap();
        let res = svc
            .create_relation(
                make_create_cmd(tenant_id, src, tgt, RelationType::RelatesTo),
                actor,
            )
            .await;
        assert!(matches!(res, Err(RelationError::Conflict(_))));
    }

    // -------- 6. INV-R-03:跨 Project 被拒 --------

    #[tokio::test]
    async fn invariant_03_cross_project_rejected() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_create_cmd(
            tenant_id,
            WorkItemId::new(),
            WorkItemId::new(),
            RelationType::RelatesTo,
        );
        cmd.same_project = false;
        let res = svc.create_relation(cmd, actor).await;
        assert!(matches!(res, Err(RelationError::InvalidState(_))));
    }

    // -------- 7. INV-R-04:循环依赖检测(A → B → C → A) --------

    #[tokio::test]
    async fn invariant_04_circular_dependency_detected() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let a = WorkItemId::new();
        let b = WorkItemId::new();
        let c = WorkItemId::new();
        svc.create_relation(
            make_create_cmd(tenant_id, a, b, RelationType::Blocks),
            actor.clone(),
        )
        .await
        .unwrap();
        svc.create_relation(
            make_create_cmd(tenant_id, b, c, RelationType::Blocks),
            actor.clone(),
        )
        .await
        .unwrap();
        // c blocks a → 形成 A→B→C→A 环,应被拒绝
        let res = svc
            .create_relation(
                make_create_cmd(tenant_id, c, a, RelationType::Blocks),
                actor.clone(),
            )
            .await;
        assert!(matches!(res, Err(RelationError::InvalidState(_))));

        // 实际不存储环,所以 detect_circular 返回 is_circular=false(已成功拦截)
        let viewer = make_test_actor(tenant_id);
        let report = svc.detect_circular(a, viewer).await.unwrap();
        assert!(!report.is_circular, "环已被拦截,无环");
    }

    // -------- 8. list_by_work_item 双向查询 --------

    #[tokio::test]
    async fn list_by_work_item_both_sides() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let a = WorkItemId::new();
        let b = WorkItemId::new();
        let c = WorkItemId::new();
        svc.create_relation(
            make_create_cmd(tenant_id, a, b, RelationType::RelatesTo),
            actor.clone(),
        )
        .await
        .unwrap();
        svc.create_relation(
            make_create_cmd(tenant_id, c, a, RelationType::RelatesTo),
            actor.clone(),
        )
        .await
        .unwrap();
        // a 出现在 source 和 target 各一次
        let viewer = make_test_actor(tenant_id);
        let list = svc.list_by_work_item(a, viewer).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    // -------- 9. list_dependencies 派生投影 --------

    #[tokio::test]
    async fn list_dependencies_derives_transitive() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let a = WorkItemId::new();
        let b = WorkItemId::new();
        let c = WorkItemId::new();
        svc.create_relation(
            make_create_cmd(tenant_id, a, b, RelationType::Blocks),
            actor.clone(),
        )
        .await
        .unwrap();
        svc.create_relation(
            make_create_cmd(tenant_id, b, c, RelationType::Blocks),
            actor.clone(),
        )
        .await
        .unwrap();
        let viewer = make_test_actor(tenant_id);
        let dep = svc.list_dependencies(a, viewer).await.unwrap();
        assert_eq!(dep.direct_dependencies, vec![b]);
        // 传递闭包应包含 c
        assert!(dep.transitive_dependencies.contains(&c));
        assert!(!dep.is_circular);
    }

    // -------- 10. delete_relation 成功 --------

    #[tokio::test]
    async fn delete_relation_success() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let r = svc
            .create_relation(
                make_create_cmd(
                    tenant_id,
                    WorkItemId::new(),
                    WorkItemId::new(),
                    RelationType::RelatesTo,
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        svc.delete_relation(r.id, actor).await.unwrap();
        assert_eq!(svc.count().await, 0);
    }

    // -------- 11. 跨租户访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let r = svc
            .create_relation(
                make_create_cmd(
                    tenant_a,
                    WorkItemId::new(),
                    WorkItemId::new(),
                    RelationType::RelatesTo,
                ),
                actor_a,
            )
            .await
            .unwrap();
        let actor_b = make_test_actor(tenant_b);
        let res = svc.delete_relation(r.id, actor_b).await;
        assert!(matches!(res, Err(RelationError::PermissionDenied)));
    }

    // -------- 12. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryRelationService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(
            tenant_id,
            WorkItemId::new(),
            WorkItemId::new(),
            RelationType::RelatesTo,
        );
        svc.create_relation(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 Created 事件");
        assert!(matches!(evt, RelationEvent::Created(_)));
        assert_eq!(evt.subject(), "star.events.relation.relation.created.v1");
    }

    // -------- 13. Gantt 派生 --------

    #[tokio::test]
    async fn get_gantt_critical_path() {
        let svc = InMemoryRelationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let a = WorkItemId::new();
        let b = WorkItemId::new();
        svc.create_relation(
            make_create_cmd(tenant_id, a, b, RelationType::Blocks),
            actor,
        )
        .await
        .unwrap();
        let viewer = make_test_actor(tenant_id);
        let now = chrono::Utc::now();
        let gantt = svc
            .get_gantt(
                a,
                DateRange {
                    start: now,
                    end: now + chrono::Duration::days(7),
                },
                viewer,
            )
            .await
            .unwrap();
        assert!(gantt.is_critical_path);
        assert_eq!(gantt.dependencies, vec![b]);
    }
}

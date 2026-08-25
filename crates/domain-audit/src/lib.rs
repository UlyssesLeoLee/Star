//! Audit 审计日志 + AI Audit Metadata
//!
//! **crate**: `domain-audit`
//! **上游 spec**: docs/specs/domain-audit-spec.md
//! **基本设计**: docs/basic-design.md §4.18
//! **数据设计**: docs/data-design.md §4.18

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
pub use entity::{AIAuditMetadata, AuditEvent};
pub use error::AuditError;
pub use event::{AuditEventAppended, AuditEventKind, EventMeta};
pub use invariants::{
    check_invariant_02_required_fields, check_invariant_03_immutable_hash,
    check_invariant_04_ai_metadata_required, compute_immutable_hash, run_invariants,
    ALL_INVARIANT_CHECKS,
};
pub use port::{
    AuditCommandPort, AuditQueryPort, ListAuditEventQuery, RecordAIAuditMetadataCommand,
    RecordAuditEventCommand,
};
pub use service::InMemoryAuditService;
pub use value_object::{roles, AIAuditMetadataId, AuditAction, AuditEventId, TenantId, UserId};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{AuditAction, TenantId, UserId};

    fn make_admin(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id).with_role(roles::TENANT_ADMIN)
    }

    fn make_auditor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id).with_role(roles::TENANT_AUDITOR)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(AuditEvent::FIELD_COUNT, 9);
        assert_eq!(AIAuditMetadata::FIELD_COUNT, 9);
    }

    #[tokio::test]
    async fn record_event_success() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_admin(tenant_id);
        let cmd = RecordAuditEventCommand {
            tenant_id,
            actor_id: UserId::new(),
            action: AuditAction::UserCreate,
            target_type: "User".to_string(),
            target_id: uuid::Uuid::new_v4(),
            payload_json: serde_json::json!({"email": "x@y.com"}),
        };
        let ev = svc.record_event(cmd, actor).await.unwrap();
        assert_eq!(ev.action, AuditAction::UserCreate);
        assert_eq!(ev.immutable_hash.len(), 64);
        assert_eq!(svc.count().await, 1);
    }

    #[tokio::test]
    async fn invariant_02_missing_target_type() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_admin(tenant_id);
        let cmd = RecordAuditEventCommand {
            tenant_id,
            actor_id: UserId::new(),
            action: AuditAction::UserCreate,
            target_type: "".to_string(),
            target_id: uuid::Uuid::new_v4(),
            payload_json: serde_json::json!({}),
        };
        let res = svc.record_event(cmd, actor).await;
        assert!(matches!(res, Err(AuditError::InvalidState(_))));
    }

    #[tokio::test]
    async fn record_ai_metadata() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let ev = svc
            .record_event(
                RecordAuditEventCommand {
                    tenant_id,
                    actor_id: UserId::new(),
                    action: AuditAction::AgentExecute,
                    target_type: "Agent".to_string(),
                    target_id: uuid::Uuid::new_v4(),
                    payload_json: serde_json::json!({}),
                },
                admin.clone(),
            )
            .await
            .unwrap();
        let m = svc
            .record_ai_metadata(
                RecordAIAuditMetadataCommand {
                    audit_event_id: ev.id,
                    tenant_id,
                    agent_session_id: uuid::Uuid::new_v4(),
                    worktree_id: None,
                    prompt_hash: "a".repeat(64),
                    response_hash: "b".repeat(64),
                    retention_until: None,
                },
                admin,
            )
            .await
            .unwrap();
        assert!(!m.is_expired(chrono::Utc::now()));
    }

    #[tokio::test]
    async fn invariant_04_ai_metadata_missing_hash() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        let ev = svc
            .record_event(
                RecordAuditEventCommand {
                    tenant_id,
                    actor_id: UserId::new(),
                    action: AuditAction::AgentExecute,
                    target_type: "Agent".to_string(),
                    target_id: uuid::Uuid::new_v4(),
                    payload_json: serde_json::json!({}),
                },
                admin.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .record_ai_metadata(
                RecordAIAuditMetadataCommand {
                    audit_event_id: ev.id,
                    tenant_id,
                    agent_session_id: uuid::Uuid::new_v4(),
                    worktree_id: None,
                    prompt_hash: "".to_string(), // 空
                    response_hash: "ok".to_string(),
                    retention_until: None,
                },
                admin,
            )
            .await;
        assert!(matches!(res, Err(AuditError::InvalidState(_))));
    }

    #[tokio::test]
    async fn non_auditor_cannot_list_events() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        svc.record_event(
            RecordAuditEventCommand {
                tenant_id,
                actor_id: UserId::new(),
                action: AuditAction::UserCreate,
                target_type: "User".to_string(),
                target_id: uuid::Uuid::new_v4(),
                payload_json: serde_json::json!({}),
            },
            admin,
        )
        .await
        .unwrap();
        // 普通 user 角色尝试 list → 拒
        let mut normal = ActorContext::new(uuid::Uuid::new_v4(), tenant_id);
        normal.roles.push("user".to_string());
        let res = svc
            .list_events(ListAuditEventQuery::default(), normal)
            .await;
        assert!(matches!(res, Err(AuditError::PermissionDenied)));
    }

    #[tokio::test]
    async fn auditor_can_list_events() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        for _ in 0..3 {
            svc.record_event(
                RecordAuditEventCommand {
                    tenant_id,
                    actor_id: UserId::new(),
                    action: AuditAction::Custom,
                    target_type: "Test".to_string(),
                    target_id: uuid::Uuid::new_v4(),
                    payload_json: serde_json::json!({}),
                },
                admin.clone(),
            )
            .await
            .unwrap();
        }
        let auditor = make_auditor(tenant_id);
        let q = ListAuditEventQuery {
            tenant_id,
            ..Default::default()
        };
        let events = svc.list_events(q.clone(), auditor.clone()).await.unwrap();
        assert_eq!(events.len(), 3);
        let count = svc.count_events(q, auditor).await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn cross_tenant_record_denied() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_a = TenantId::new();
        let admin_a = make_admin(tenant_a);
        let tenant_b = TenantId::new();
        // 用 admin_a 但命令带 tenant_b → 租户不一致 → 拒
        let cmd = RecordAuditEventCommand {
            tenant_id: tenant_b,
            actor_id: UserId::new(),
            action: AuditAction::Custom,
            target_type: "X".to_string(),
            target_id: uuid::Uuid::new_v4(),
            payload_json: serde_json::json!({}),
        };
        let res = svc.record_event(cmd, admin_a).await;
        assert!(matches!(res, Err(AuditError::PermissionDenied)));
    }

    #[tokio::test]
    async fn event_bus_receives_appended() {
        let (svc, mut rx) = InMemoryAuditService::new();
        let tenant_id = TenantId::new();
        let admin = make_admin(tenant_id);
        svc.record_event(
            RecordAuditEventCommand {
                tenant_id,
                actor_id: UserId::new(),
                action: AuditAction::Custom,
                target_type: "X".to_string(),
                target_id: uuid::Uuid::new_v4(),
                payload_json: serde_json::json!({}),
            },
            admin,
        )
        .await
        .unwrap();
        let kind = rx.try_recv().expect("应收到 Appended 事件");
        assert!(matches!(kind, AuditEventKind::Appended(_)));
        assert_eq!(kind.subject(), "star.events.audit.event.appended.v1");
    }
}

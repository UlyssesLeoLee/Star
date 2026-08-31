//! 第三方平台集成领域
//!
//! **crate**: `domain-integration`
//! **上游 spec**: `docs/specs/domain-integration-spec.md` §18 Integration / 双向同步
//! **基本设计**: `docs/basic-design.md` §2.1 / §3.1 接触点表
//! **数据设计**: `docs/data-design.md` §4.12 (`integration` schema)
//! **API 设计**: `docs/api-design.md` §3.13 (Integration / SyncState)
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。本 crate 承载 **第三方平台双向同步抽象**(§18),
//! 区分 4 类关系: **Link** / **Mirror** / **Bidirectional** / **Platform-owned**(§4.7.5)。
//!
//! ## 关键不变量
//!
//! - **INV-I-01**: 4 类关系分类必带(Link / Mirror / Bidirectional / PlatformOwned),禁止混用
//! - **INV-I-02**: Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token,RISK-027)
//! - **INV-I-03**: 必带 tenant_id,跨 tenant 拒绝(§6.1,REQ-SEC-001)
//! - **INV-I-04**: 凭据走 Credential Broker,不存明文(§5.4)
//! - **INV-I-05**: 每条关系定义 Source / Ownership / Version / External ID / Sync Token / Last Synced / Conflict Strategy
//! - **INV-I-06**: 默认 Link(WorkItem ↔ GitHub Issue),不反向同步
//!
//! ## 上游依赖
//!
//! 本 crate 无跨 domain-* crate 依赖(SCM 集成通过 ACL 翻译)。

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// =====================================================================
// 子模块装载
// =====================================================================

pub mod adapter;
pub mod confluence;
pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

// =====================================================================
// 便捷 re-export
// =====================================================================

pub use context::ActorContext as ContextActorContext; // 子模块强类型 ID 版本 (供 domain 内部 use crate::context::ActorContext)
pub use star_context::ActorContext; // 收敛到 star_context 权威版本 (per P0-1 联动协作)
pub use entity::{Integration, MappingConfig, SyncDirection, SyncState};
pub use error::IntegrationError;
pub use event::{
    EventMeta, IntegrationCreated, IntegrationEvent, IntegrationStateChanged, SyncCompleted,
    SyncConflictDetected, SyncTriggered,
};
pub use invariants::{
    check_invariant_01_relation_type_classified, check_invariant_02_bidirectional_loop_guard,
    check_invariant_03_tenant_required, check_invariant_04_no_plaintext_credential,
    check_invariant_05_required_fields, check_invariant_06_link_no_reverse_sync,
    check_register_invariants, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    ConfigureIntegrationCommand, CreateIntegrationCommand, GetHistoryQuery, HandleWebhookCommand,
    IntegrationCommandPort, IntegrationQueryPort, IntegrationRepository, ListByProjectQuery,
    PauseIntegrationCommand, ResumeIntegrationCommand, TriggerSyncCommand,
    UpdateIntegrationCommand,
};
pub use service::InMemoryIntegrationService;
pub use value_object::{
    roles, ConflictStrategy, ExternalEntityId, ExternalSystemName, IntegrationId,
    IntegrationRelationType, IntegrationSource, IntegrationState, ProjectId, SyncOutcome,
    SyncStateId, TenantId, UserId, WebhookDeliveryId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{
        roles, ConflictStrategy, ExternalEntityId, ExternalSystemName, IntegrationRelationType,
        IntegrationSource, IntegrationState, ProjectId, SyncStateId, TenantId, UserId,
    };

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new())
    }

    fn make_create_cmd(
        tenant_id: TenantId,
        relation_type: IntegrationRelationType,
    ) -> CreateIntegrationCommand {
        let project_id = ProjectId::new();
        let initial_sync_token = if relation_type.requires_sync_token() {
            Some(format!("initial-token-{}", uuid::Uuid::new_v4()))
        } else {
            None
        };
        CreateIntegrationCommand {
            tenant_id,
            project_id,
            source: IntegrationSource::Scm,
            relation_type,
            external_system_name: ExternalSystemName::new("github"),
            external_id: ExternalEntityId::new("acme/foo#123"),
            external_url: "https://github.com/acme/foo/issues/123".to_string(),
            conflict_strategy: if relation_type == IntegrationRelationType::Bidirectional {
                ConflictStrategy::ManualReview
            } else {
                ConflictStrategy::LatestWins
            },
            credential_id: None,
            initial_sync_token,
        }
    }

    // -------- 1. 4 类关系创建测试 --------

    #[tokio::test]
    async fn four_relation_types_create_success() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);

        for rt in [
            IntegrationRelationType::Link,
            IntegrationRelationType::Mirror,
            IntegrationRelationType::Bidirectional,
            IntegrationRelationType::PlatformOwned,
        ] {
            let cmd = make_create_cmd(tenant_id, rt);
            let integration = svc
                .create_integration(cmd, actor.clone())
                .await
                .unwrap_or_else(|e| panic!("创建 {:?} 失败: {:?}", rt, e));
            assert_eq!(integration.relation_type, rt);
        }
        assert_eq!(svc.count_integrations().await, 4);
    }

    // -------- 2. 字段数审计 --------

    #[test]
    fn field_count_audit() {
        assert_eq!(Integration::FIELD_COUNT, 19);
        assert_eq!(SyncState::FIELD_COUNT, 12);
    }

    // -------- 3. Link 关系不能 trigger_sync(INV-I-06) --------

    #[tokio::test]
    async fn link_relation_cannot_trigger_sync() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Link);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("Link 创建成功");

        // Link 关系不应携带 sync_token(INV-I-06)
        assert!(integration.sync_token.is_none());

        // trigger_sync 应该被拒
        let res = svc
            .trigger_sync(
                TriggerSyncCommand {
                    integration_id: integration.id,
                    tenant_id,
                    force: false,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(IntegrationError::InvalidState(_))));
    }

    // -------- 4. Bidirectional 缺 sync_token 被拒(INV-I-02,Loop 防护) --------

    #[tokio::test]
    async fn bidirectional_without_sync_token_rejected() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_create_cmd(tenant_id, IntegrationRelationType::Bidirectional);
        cmd.initial_sync_token = None; // 故意缺失 → I-004

        let res = svc.create_integration(cmd, actor).await;
        assert!(matches!(res, Err(IntegrationError::LoopGuardMissing(_))));
    }

    // -------- 5. Bidirectional Webhook 缺 source_id 标记 → Skipped(Loop 防护) --------

    #[tokio::test]
    async fn bidirectional_webhook_without_source_id_skipped() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Bidirectional);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("Bidirectional 创建成功");

        // 发送缺 source_id 标记的 Webhook
        let res = svc
            .handle_webhook(HandleWebhookCommand {
                integration_id: integration.id,
                tenant_id,
                external_event_id: "gh-event-1".to_string(),
                payload: r#"{"action":"opened","issue":{}}"#.to_string(),
                signature: None,
            })
            .await
            .expect("应返回 Skipped SyncState");

        assert_eq!(res.outcome, SyncOutcome::Skipped);
        assert!(res.error.is_some());
        assert_eq!(res.skipped_count, 1);
    }

    // -------- 6. Bidirectional Webhook 含 source_id 标记 → 正常处理 --------

    #[tokio::test]
    async fn bidirectional_webhook_with_source_id_succeeds() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Bidirectional);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("Bidirectional 创建成功");

        // 发送含 source_id 标记的 Webhook
        let res = svc
            .handle_webhook(HandleWebhookCommand {
                integration_id: integration.id,
                tenant_id,
                external_event_id: "gh-event-2".to_string(),
                payload: r#"{"action":"opened","source_id":"platform-uuid-123"}"#.to_string(),
                signature: None,
            })
            .await
            .expect("应成功处理");

        assert_eq!(res.outcome, SyncOutcome::Success);
    }

    // -------- 7. Webhook 幂等(重复事件被拒) --------

    #[tokio::test]
    async fn webhook_idempotency_blocks_duplicates() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Mirror);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("Mirror 创建成功");

        let event_id = "gh-event-3".to_string();
        let payload = r#"{"action":"opened","source_id":"platform-uuid-1"}"#.to_string();

        // 第一次入站
        let res1 = svc
            .handle_webhook(HandleWebhookCommand {
                integration_id: integration.id,
                tenant_id,
                external_event_id: event_id.clone(),
                payload: payload.clone(),
                signature: None,
            })
            .await;
        assert!(res1.is_ok());

        // 第二次入站(同 external_event_id)→ Conflict
        let res2 = svc
            .handle_webhook(HandleWebhookCommand {
                integration_id: integration.id,
                tenant_id,
                external_event_id: event_id,
                payload,
                signature: None,
            })
            .await;
        assert!(matches!(res2, Err(IntegrationError::Conflict(_))));
    }

    // -------- 8. 跨 Tenant 访问被拒(INV-I-03) --------

    #[tokio::test]
    async fn cross_tenant_access_rejected() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let actor_a = make_test_actor(tenant_a);
        let cmd = make_create_cmd(tenant_a, IntegrationRelationType::Mirror);
        let integration = svc
            .create_integration(cmd, actor_a.clone())
            .await
            .expect("创建成功");

        let actor_b = ActorContext::new(uuid::Uuid::new_v4(), tenant_b)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(integration.project_id);
        let res = svc.get_integration(integration.id, actor_b).await;
        assert!(matches!(res, Err(IntegrationError::PermissionDenied)));
    }

    // -------- 9. UNIQUE 约束(同 source+system+external_id 重复被拒) --------

    #[tokio::test]
    async fn unique_constraint_enforced() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd1 = make_create_cmd(tenant_id, IntegrationRelationType::Link);
        svc.create_integration(cmd1.clone(), actor.clone())
            .await
            .expect("首次创建成功");

        let cmd2 = make_create_cmd(tenant_id, IntegrationRelationType::Link);
        let res = svc.create_integration(cmd2, actor).await;
        assert!(matches!(res, Err(IntegrationError::Conflict(_))));
    }

    // -------- 10. URL 含明文凭据被拒(INV-I-04) --------

    #[tokio::test]
    async fn url_with_plaintext_credential_rejected() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_create_cmd(tenant_id, IntegrationRelationType::Mirror);
        cmd.external_url = "https://user:pass@github.com/acme/foo".to_string();
        let res = svc.create_integration(cmd, actor).await;
        assert!(matches!(res, Err(IntegrationError::InvalidState(_))));
    }

    // -------- 11. pause / resume 状态机 + 事件 --------

    #[tokio::test]
    async fn pause_resume_state_machine() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Mirror);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("创建成功");

        let paused = svc
            .pause_integration(
                PauseIntegrationCommand {
                    integration_id: integration.id,
                    tenant_id,
                },
                actor.clone(),
            )
            .await
            .expect("暂停成功");
        assert_eq!(paused.state, IntegrationState::Paused);

        let resumed = svc
            .resume_integration(
                ResumeIntegrationCommand {
                    integration_id: integration.id,
                    tenant_id,
                },
                actor,
            )
            .await
            .expect("恢复成功");
        assert_eq!(resumed.state, IntegrationState::Active);
    }

    // -------- 12. list_by_project 过滤 --------

    #[tokio::test]
    async fn list_by_project_filters() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let actor = make_test_actor(tenant_id).with_project(project_id);
        // 创建 2 个不同 relation_type
        for rt in [
            IntegrationRelationType::Link,
            IntegrationRelationType::Mirror,
        ] {
            let mut cmd = make_create_cmd(tenant_id, rt);
            cmd.project_id = project_id;
            svc.create_integration(cmd, actor.clone())
                .await
                .unwrap_or_else(|e| panic!("创建 {:?} 失败: {:?}", rt, e));
        }

        // 按 relation_type 过滤
        let only_link = svc
            .list_by_project(
                ListByProjectQuery {
                    tenant_id,
                    project_id,
                    source_filter: None,
                    relation_type_filter: Some(IntegrationRelationType::Link),
                    state_filter: None,
                    active_only: false,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert_eq!(only_link.len(), 1);
        assert!(only_link[0].is_link());
    }

    // -------- 13. 事件总线收到 IntegrationCreated + subject 校验 --------

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryIntegrationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Link);
        svc.create_integration(cmd, actor).await.expect("ok");

        let evt = rx.try_recv().expect("应收到事件");
        assert!(matches!(evt, IntegrationEvent::IntegrationCreated(_)));
        assert_eq!(
            evt.subject(),
            "star.events.integration.integration.created.v1"
        );
    }

    // -------- 14. IntegrationRelationType 字面量校验 + requires_loop_guard --------

    #[test]
    fn relation_type_str_and_loop_guard() {
        assert_eq!(IntegrationRelationType::Link.as_str(), "LINK");
        assert_eq!(IntegrationRelationType::Mirror.as_str(), "MIRROR");
        assert_eq!(
            IntegrationRelationType::Bidirectional.as_str(),
            "BIDIRECTIONAL"
        );
        assert_eq!(
            IntegrationRelationType::PlatformOwned.as_str(),
            "PLATFORM_OWNED"
        );

        assert!(!IntegrationRelationType::Link.requires_loop_guard());
        assert!(IntegrationRelationType::Bidirectional.requires_loop_guard());
        assert!(!IntegrationRelationType::Mirror.requires_loop_guard());
        assert!(!IntegrationRelationType::PlatformOwned.requires_loop_guard());

        // Link 不需要 sync_token;其他需要
        assert!(!IntegrationRelationType::Link.requires_sync_token());
        assert!(IntegrationRelationType::Mirror.requires_sync_token());
        assert!(IntegrationRelationType::Bidirectional.requires_sync_token());
        assert!(IntegrationRelationType::PlatformOwned.requires_sync_token());
    }

    // -------- 15. IntegrationSource / IntegrationState 字面量 --------

    #[test]
    fn enums_as_str() {
        assert_eq!(IntegrationSource::Scm.as_str(), "SCM");
        assert_eq!(
            IntegrationSource::ProjectManagement.as_str(),
            "PROJECT_MANAGEMENT"
        );
        assert_eq!(IntegrationSource::Communication.as_str(), "COMMUNICATION");
        assert_eq!(IntegrationSource::Other.as_str(), "OTHER");

        assert_eq!(IntegrationState::Initializing.as_str(), "INITIALIZING");
        assert_eq!(IntegrationState::Active.as_str(), "ACTIVE");
        assert_eq!(IntegrationState::Paused.as_str(), "PAUSED");
        assert_eq!(IntegrationState::Error.as_str(), "ERROR");
        assert_eq!(IntegrationState::Disabled.as_str(), "DISABLED");

        assert_eq!(SyncOutcome::Success.as_str(), "SUCCESS");
        assert_eq!(SyncOutcome::PartialSuccess.as_str(), "PARTIAL_SUCCESS");
        assert_eq!(SyncOutcome::Failed.as_str(), "FAILED");
        assert_eq!(SyncOutcome::Skipped.as_str(), "SKIPPED");
    }

    // -------- 16. trigger_sync 在 Mirror 上发 SyncTriggered 事件 --------

    #[tokio::test]
    async fn trigger_sync_emits_event() {
        let (svc, mut rx) = InMemoryIntegrationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Mirror);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("ok");
        // 消费 created 事件
        let _ = rx.try_recv();

        let res = svc
            .trigger_sync(
                TriggerSyncCommand {
                    integration_id: integration.id,
                    tenant_id,
                    force: false,
                },
                actor,
            )
            .await
            .expect("触发同步成功");
        assert_eq!(res.direction, SyncDirection::Inbound);

        let evt = rx.try_recv().expect("应收到 SyncTriggered");
        assert!(matches!(evt, IntegrationEvent::SyncTriggered(_)));
        assert_eq!(evt.subject(), "star.events.integration.sync.triggered.v1");
    }

    // -------- 17. configure_integration 为 Bidirectional 注入 idempotency_key --------

    #[tokio::test]
    async fn configure_sets_loop_guard_token_for_bidirectional() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_create_cmd(tenant_id, IntegrationRelationType::Bidirectional);
        cmd.initial_sync_token = None; // 故意缺失,后续由 configure 注入
        let res = svc.create_integration(cmd, actor.clone()).await;
        assert!(matches!(res, Err(IntegrationError::LoopGuardMissing(_))));

        // 用 update_integration 注入 sync_token(模拟 configure)
        let cmd2 = make_create_cmd(tenant_id, IntegrationRelationType::Bidirectional);
        let integration = svc
            .create_integration(cmd2, actor.clone())
            .await
            .expect("ok");
        assert!(integration.sync_token.is_some());

        let updated = svc
            .update_integration(
                UpdateIntegrationCommand {
                    integration_id: integration.id,
                    tenant_id,
                    conflict_strategy: Some(ConflictStrategy::Bidirectional {
                        platform_field: "title".to_string(),
                        external_field: "title".to_string(),
                    }),
                    sync_token: Some("stronger-token".to_string()),
                    external_url: None,
                    credential_id: None,
                },
                actor,
            )
            .await
            .expect("更新成功");
        assert_eq!(updated.sync_token.as_deref(), Some("stronger-token"));
        assert!(matches!(
            updated.conflict_strategy,
            ConflictStrategy::Bidirectional { .. }
        ));
    }

    // -------- 18. get_sync_state 与 get_history --------

    #[tokio::test]
    async fn get_sync_state_and_history() {
        let svc = InMemoryIntegrationService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let cmd = make_create_cmd(tenant_id, IntegrationRelationType::Mirror);
        let actor = make_test_actor(tenant_id).with_project(cmd.project_id);
        let integration = svc
            .create_integration(cmd, actor.clone())
            .await
            .expect("ok");

        // trigger_sync 2 次
        for _ in 0..2 {
            svc.trigger_sync(
                TriggerSyncCommand {
                    integration_id: integration.id,
                    tenant_id,
                    force: false,
                },
                actor.clone(),
            )
            .await
            .expect("ok");
        }

        let latest = svc
            .get_sync_state(integration.id, actor.clone())
            .await
            .expect("ok");
        assert_eq!(latest.integration_id, integration.id);

        let history = svc
            .get_history(
                GetHistoryQuery {
                    tenant_id,
                    integration_id: integration.id,
                    limit: 10,
                    since: None,
                },
                actor,
            )
            .await
            .expect("ok");
        assert_eq!(history.len(), 2);
    }

    // 静默抑制未使用导入
    #[allow(dead_code)]
    fn _unused_imports() {
        let _ = SyncStateId::new();
    }
}

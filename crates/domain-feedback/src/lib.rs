//! Feedback 领域
//!
//! **crate**: `domain-feedback`
//! **上游 spec**: docs/specs/domain-feedback-spec.md §25 Feedback Model + 6 状态机
//! **基本设计**: docs/basic-design.md §2.1 / §4.3 / §7.3 (6 状态迁移)
//! **数据设计**: docs/data-design.md §4.22 (`feedback` schema)
//! **API 设计**: docs/api-design.md §3.23 (6 状态迁移端点)
//!
//! ## 职责
//!
//! WorkItem / PR / DiffHunk 上的**结构化 Feedback 一级领域对象**(§25.1, REQ-FBK-001/002):
//! - `Feedback` 聚合根(6 状态机,§7.3)
//! - `FeedbackTarget` 11 种(spec §7 任务范围)
//! - `FeedbackResolution` 解决实体(evidence_refs)
//! - `FeedbackConsumedEvent` 消费投影
//! - `FeedbackInboxItem` Intervention Queue 投影
//! - 2 个端口(`FeedbackCommandPort` × 7 / `FeedbackQueryPort` × 5) + 1 个仓库端口
//! - 8 条不变量(INV-FB-01~08)
//! - 1 个 `InMemoryFeedbackService` 完整实现
//!
//! ## 关键不变量
//!
//! - Feedback 必带 tenant_id(INV-FB-06,§6.1)
//! - Feedback ≠ Comment(INV-FB-08,§25.1,UI 显式区分)
//! - 6 状态机严格迁移(INV-FB-01,§7.3,§10 接口稳定承诺 #7)
//! - Supersede 必带 successor_id(INV-FB-04,FB-006)
//! - 跨 Worktree 禁止(INV-FB-05,FB-007)
//! - AI 提的 Feedback author_agent_id 必带(INV-FB-07)
//! - APPLIED 之后只读(FB-004)
//! - 仅 OPEN 可删(FB-005)
//! - Target 必可解析(INV-FB-02,FB-003)

#![allow(missing_docs)]

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

pub use star_context::ActorContext; // 收敛到 star_context 权威版本 (per P0-1 联动协作)
                                    // 注: 子模块 context::ActorContext 仍然在 context namespace, 域内用 use crate::context::ActorContext 引用
pub use entity::{
    ConsumedByKind, EvidenceKind, Feedback, FeedbackConsumedEvent, FeedbackInboxItem,
    FeedbackResolution, ResolutionEvidence, ResolutionEvidenceRef,
};
pub use error::FeedbackError;
pub use event::{
    EventMeta, FeedbackAcknowledged, FeedbackApplied, FeedbackCreated, FeedbackEvent,
    FeedbackRejected, FeedbackSuperseded, FeedbackVerified,
};
pub use invariants::{
    check_create_invariants, check_invariant_01_six_state_machine_placeholder,
    check_invariant_02_target_resolvable_placeholder, check_invariant_03_status_audit_placeholder,
    check_invariant_04_supersede_has_successor, check_invariant_05_cross_worktree_placeholder,
    check_invariant_06_tenant_id_present, check_invariant_07_agent_required,
    check_invariant_08_not_comment_placeholder, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    CreateFeedbackCommand, FeedbackCommandPort, FeedbackInboxQuery, FeedbackQueryPort,
    FeedbackRepository, ListFeedbackQuery, SubmitResolutionCommand,
    TransitionFeedbackStatusCommand, UpdateFeedbackCommand,
};
pub use service::InMemoryFeedbackService;
pub use value_object::{
    roles, AcceptanceCriterionId, AgentId, AgentSessionId, BuildId, CommitId, DecisionId,
    FeedbackId, FeedbackResolutionId, FeedbackStatus, FeedbackTarget, FeedbackType, LineRange,
    ProjectId, RepositoryId, RequirementId, Severity, SymbolId, TenantId, TestId, UserId,
    WorkItemId, WorktreeId,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ConsumedByKind;
    use crate::value_object::{FeedbackTarget, Severity};
    use uuid::Uuid;

    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid()).with_role(roles::DEVELOPER)
    }

    fn make_create_cmd(tenant_id: TenantId, target: FeedbackTarget) -> CreateFeedbackCommand {
        CreateFeedbackCommand {
            tenant_id,
            project_id: ProjectId::new(),
            work_item_id: WorkItemId::new(),
            target,
            r#type: FeedbackType::ReviewFinding,
            severity: Severity::P1,
            intent: "重构为 AuthProvider".to_string(),
            expected_behavior: "调用方不变,内部抽象".to_string(),
            preserve: vec!["public API".to_string()],
            prohibit: vec!["breaking change".to_string()],
            author_agent_id: None,
            acceptance_criteria_id: None,
            predecessor_id: None,
        }
    }

    // -------- 1. 6 状态机 can_transition_to 全覆盖 --------

    #[test]
    fn six_state_machine_all_valid_transitions() {
        use FeedbackStatus::*;
        // 合法
        assert!(Open.can_transition_to(Acknowledged));
        assert!(Acknowledged.can_transition_to(Applied));
        assert!(Applied.can_transition_to(Verified));
        // 任意中间态 -> Rejected
        assert!(Open.can_transition_to(Rejected));
        assert!(Acknowledged.can_transition_to(Rejected));
        assert!(Applied.can_transition_to(Rejected));
        // 任意非终态 -> Superseded
        assert!(Open.can_transition_to(Superseded));
        assert!(Acknowledged.can_transition_to(Superseded));
        assert!(Applied.can_transition_to(Superseded));
        assert!(Verified.can_transition_to(Superseded));
        // 非法: 终态 -> 任何
        assert!(!Rejected.can_transition_to(Open));
        assert!(!Rejected.can_transition_to(Acknowledged));
        assert!(!Rejected.can_transition_to(Applied));
        assert!(!Rejected.can_transition_to(Verified));
        assert!(!Superseded.can_transition_to(Open));
        assert!(!Superseded.can_transition_to(Verified));
        // 非法: 跳级
        assert!(!Open.can_transition_to(Applied));
        assert!(!Open.can_transition_to(Verified));
        assert!(!Acknowledged.can_transition_to(Verified));
        // 非法: 自反
        assert!(!Open.can_transition_to(Open));
        assert!(!Verified.can_transition_to(Verified));
        // 非法: Rejected/Superseded 不可达 Verified
        assert!(!Verified.can_transition_to(Rejected));
    }

    // -------- 2. 11 Target 类型构造 + 类型 tag 校验 --------

    #[test]
    fn eleven_target_types_construction_and_kinds() {
        // 11 种 target 都能构造
        let targets = vec![
            FeedbackTarget::WorkItem {
                work_item_id: WorkItemId::new(),
            },
            FeedbackTarget::Requirement {
                requirement_id: RequirementId::new(),
            },
            FeedbackTarget::AcceptanceCriterion {
                ac_id: AcceptanceCriterionId::new(),
            },
            FeedbackTarget::Worktree {
                worktree_id: WorktreeId::new(),
            },
            FeedbackTarget::AgentSession {
                session_id: AgentSessionId::new(),
            },
            FeedbackTarget::File {
                repository_id: RepositoryId::new(),
                path: "src/auth.rs".to_string(),
                line_range: Some(LineRange { start: 10, end: 30 }),
            },
            FeedbackTarget::Symbol {
                symbol_id: SymbolId::new(),
                ref_name: "AuthProvider::verify".to_string(),
            },
            FeedbackTarget::DiffHunk {
                commit_id: CommitId::new(),
                hunk_index: 2,
            },
            FeedbackTarget::Test {
                test_id: TestId::new(),
            },
            FeedbackTarget::Build {
                build_id: BuildId::new(),
            },
            FeedbackTarget::Decision {
                decision_id: DecisionId::new(),
            },
        ];
        assert_eq!(targets.len(), FeedbackTarget::COUNT);
        assert_eq!(FeedbackTarget::COUNT, 11);

        // kind 标签互不重复
        let kinds: Vec<&str> = targets.iter().map(|t| t.kind()).collect();
        let unique: std::collections::HashSet<&str> = kinds.iter().copied().collect();
        assert_eq!(unique.len(), 11);
    }

    // -------- 3. create_feedback 成功 + 事件发布 + tenant 必带 --------

    #[tokio::test]
    async fn create_feedback_success_and_event() {
        let (svc, mut rx) = InMemoryFeedbackService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let target = FeedbackTarget::WorkItem {
            work_item_id: WorkItemId::new(),
        };
        let cmd = make_create_cmd(tenant_id, target.clone());
        let f = svc.create_feedback(cmd, actor).await.expect("create OK");
        assert_eq!(f.status, FeedbackStatus::Open);
        assert_eq!(f.tenant_id, tenant_id);
        assert!(matches!(f.target, FeedbackTarget::WorkItem { .. }));
        assert_eq!(svc.count_feedbacks().await, 1);

        // 收到 created 事件
        let mut found = false;
        for _ in 0..5 {
            if let Ok(e) = rx.try_recv() {
                if matches!(e, FeedbackEvent::Created(_)) {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "应收到 Created 事件");
    }

    // -------- 4. 6 状态机完整迁移链:OPEN → ACK → APPLIED → VERIFIED --------

    #[tokio::test]
    async fn full_six_state_chain_open_to_verified() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        let fid = f.id;

        // OPEN -> ACKNOWLEDGED
        let f = svc
            .transition_status(
                TransitionFeedbackStatusCommand {
                    feedback_id: fid,
                    tenant_id,
                    from: FeedbackStatus::Open,
                    to: FeedbackStatus::Acknowledged,
                    reason: "ack".into(),
                    successor_id: None,
                    actor_worktree_id: None,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert_eq!(f.status, FeedbackStatus::Acknowledged);

        // ACKNOWLEDGED -> APPLIED
        let f = svc
            .mark_applied(fid, Uuid::new_v4(), actor.clone())
            .await
            .unwrap();
        assert_eq!(f.status, FeedbackStatus::Applied);

        // APPLIED -> VERIFIED
        let f = svc
            .mark_verified(fid, Uuid::new_v4(), vec![], actor)
            .await
            .unwrap();
        assert_eq!(f.status, FeedbackStatus::Verified);
        assert!(f.resolved_at.is_some());
    }

    // -------- 5. 非法状态迁移被拒(FB-002) --------

    #[tokio::test]
    async fn invalid_state_transition_rejected() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        // OPEN -> VERIFIED 跳级,应被拒
        let res = svc
            .transition_status(
                TransitionFeedbackStatusCommand {
                    feedback_id: f.id,
                    tenant_id,
                    from: FeedbackStatus::Open,
                    to: FeedbackStatus::Verified,
                    reason: "skip".into(),
                    successor_id: None,
                    actor_worktree_id: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(FeedbackError::InvalidState(_))));
    }

    // -------- 6. 任意 -> REJECTED 终态 --------

    #[tokio::test]
    async fn reject_from_open_terminal() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        let f = svc
            .transition_status(
                TransitionFeedbackStatusCommand {
                    feedback_id: f.id,
                    tenant_id,
                    from: FeedbackStatus::Open,
                    to: FeedbackStatus::Rejected,
                    reason: "不相关".into(),
                    successor_id: None,
                    actor_worktree_id: None,
                },
                actor,
            )
            .await
            .unwrap();
        assert!(f.is_terminal());
        assert!(f.resolved_at.is_some());
    }

    // -------- 7. Supersede 必带 successor(FB-006) --------

    #[tokio::test]
    async fn supersede_without_successor_rejected() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .transition_status(
                TransitionFeedbackStatusCommand {
                    feedback_id: f.id,
                    tenant_id,
                    from: FeedbackStatus::Open,
                    to: FeedbackStatus::Superseded,
                    reason: "supersede".into(),
                    successor_id: None, // 缺 successor
                    actor_worktree_id: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(FeedbackError::MissingSuccessor)));
    }

    #[tokio::test]
    async fn supersede_with_successor_ok() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f1 = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        let f2 = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        let f1_res = svc
            .transition_status(
                TransitionFeedbackStatusCommand {
                    feedback_id: f1.id,
                    tenant_id,
                    from: FeedbackStatus::Open,
                    to: FeedbackStatus::Superseded,
                    reason: "supersede by f2".into(),
                    successor_id: Some(f2.id),
                    actor_worktree_id: None,
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(f1_res.status, FeedbackStatus::Superseded);
        assert_eq!(f1_res.successor_id, Some(f2.id));
    }

    // -------- 8. APPLIED 之后 update 被拒(FB-004) --------

    #[tokio::test]
    async fn update_after_applied_rejected() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        // OPEN -> ACK
        svc.transition_status(
            TransitionFeedbackStatusCommand {
                feedback_id: f.id,
                tenant_id,
                from: FeedbackStatus::Open,
                to: FeedbackStatus::Acknowledged,
                reason: "ack".into(),
                successor_id: None,
                actor_worktree_id: None,
            },
            actor.clone(),
        )
        .await
        .unwrap();
        // ACK -> APPLIED
        svc.mark_applied(f.id, Uuid::new_v4(), actor.clone())
            .await
            .unwrap();
        // update
        let res = svc
            .update_feedback(
                UpdateFeedbackCommand {
                    feedback_id: f.id,
                    tenant_id,
                    expected_version: 3,
                    new_intent: Some("改不动".into()),
                    new_expected_behavior: None,
                    new_preserve: None,
                    new_prohibit: None,
                    new_severity: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(FeedbackError::ReadOnly)));
    }

    // -------- 9. 仅 OPEN 可删(FB-005) --------

    #[tokio::test]
    async fn delete_only_open_allowed() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        // ACK 后不可删
        svc.transition_status(
            TransitionFeedbackStatusCommand {
                feedback_id: f.id,
                tenant_id,
                from: FeedbackStatus::Open,
                to: FeedbackStatus::Acknowledged,
                reason: "ack".into(),
                successor_id: None,
                actor_worktree_id: None,
            },
            actor.clone(),
        )
        .await
        .unwrap();
        let res = svc.delete_feedback(f.id, actor).await;
        assert!(matches!(res, Err(FeedbackError::NotDeletable)));
    }

    // -------- 10. Inbox P0-P3 优先级排序 --------

    #[tokio::test]
    async fn inbox_severity_priority_ordering() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let project_id = ProjectId::new();

        // P3 第一个创建,P0 最后创建
        let mut cmd = make_create_cmd(
            tenant_id,
            FeedbackTarget::WorkItem {
                work_item_id: WorkItemId::new(),
            },
        );
        cmd.project_id = project_id;
        cmd.severity = Severity::P3;
        svc.create_feedback(cmd, actor.clone()).await.unwrap();
        let mut cmd = make_create_cmd(
            tenant_id,
            FeedbackTarget::WorkItem {
                work_item_id: WorkItemId::new(),
            },
        );
        cmd.project_id = project_id;
        cmd.severity = Severity::P1;
        svc.create_feedback(cmd, actor.clone()).await.unwrap();
        let mut cmd = make_create_cmd(
            tenant_id,
            FeedbackTarget::WorkItem {
                work_item_id: WorkItemId::new(),
            },
        );
        cmd.project_id = project_id;
        cmd.severity = Severity::P0;
        svc.create_feedback(cmd, actor.clone()).await.unwrap();
        let mut cmd = make_create_cmd(
            tenant_id,
            FeedbackTarget::WorkItem {
                work_item_id: WorkItemId::new(),
            },
        );
        cmd.project_id = project_id;
        cmd.severity = Severity::P2;
        svc.create_feedback(cmd, actor).await.unwrap();

        let inbox = svc
            .inbox(
                FeedbackInboxQuery {
                    tenant_id,
                    project_id,
                    min_severity: None,
                    limit: 10,
                    offset: 0,
                },
                ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid()).with_role(roles::DEVELOPER),
            )
            .await
            .unwrap();
        assert_eq!(inbox.len(), 4);
        // P0 排第一
        assert_eq!(inbox[0].severity, Severity::P0);
        assert_eq!(inbox[1].severity, Severity::P1);
        assert_eq!(inbox[2].severity, Severity::P2);
        assert_eq!(inbox[3].severity, Severity::P3);
    }

    // -------- 11. 跨 tenant 访问被拒(INV-FB-06) --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let actor_a = make_actor(tenant_a);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_a,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor_a,
            )
            .await
            .unwrap();
        let actor_b = make_actor(tenant_b);
        let res = svc.get_by_id(f.id, actor_b).await;
        assert!(matches!(res, Err(FeedbackError::PermissionDenied)));
    }

    // -------- 12. 跨 Worktree target 被拒(INV-FB-05,FB-007) --------

    #[tokio::test]
    async fn cross_worktree_target_rejected() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let worktree_a = WorktreeId::new();
        let worktree_b = WorktreeId::new();
        let target = FeedbackTarget::Worktree {
            worktree_id: worktree_a,
        };
        let f = svc
            .create_feedback(make_create_cmd(tenant_id, target), actor.clone())
            .await
            .unwrap();
        // ACK 时 actor 在 worktree_b,应被拒
        let res = svc
            .transition_status(
                TransitionFeedbackStatusCommand {
                    feedback_id: f.id,
                    tenant_id,
                    from: FeedbackStatus::Open,
                    to: FeedbackStatus::Acknowledged,
                    reason: "ack from other worktree".into(),
                    successor_id: None,
                    actor_worktree_id: Some(worktree_b.into_uuid()),
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(FeedbackError::CrossWorktree)));
    }

    // -------- 13. AI 提的 author_agent_id 必带(INV-FB-07) --------

    #[tokio::test]
    async fn ai_authored_feedback_records_agent_id() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let mut actor = make_actor(tenant_id);
        actor.is_agent_session = true;
        // 不显式传 author_agent_id,service 应自动兜底
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor,
            )
            .await
            .unwrap();
        assert!(
            f.author_agent_id.is_some(),
            "AI 提的 Feedback author_agent_id 必带"
        );
    }

    // -------- 14. FeedbackConsumedEvent 投影(Agent / Context / ChangeSet) --------

    #[tokio::test]
    async fn consumed_event_projection_three_kinds() {
        let svc = InMemoryFeedbackService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let f = svc
            .create_feedback(
                make_create_cmd(
                    tenant_id,
                    FeedbackTarget::WorkItem {
                        work_item_id: WorkItemId::new(),
                    },
                ),
                actor.clone(),
            )
            .await
            .unwrap();
        for kind in [
            ConsumedByKind::AgentSession,
            ConsumedByKind::ContextPacket,
            ConsumedByKind::ChangeSet,
        ] {
            svc.record_consumed(f.id, kind, Uuid::new_v4(), actor.clone())
                .await
                .unwrap();
        }
        let events = svc.list_consumed_events(f.id, actor).await.unwrap();
        assert_eq!(events.len(), 3);
    }

    // -------- 15. 字段数审计 + 角色 + 7 类型枚举完整性 --------

    #[test]
    fn field_count_audit_and_role_constants() {
        assert_eq!(Feedback::FIELD_COUNT, 20);
        assert_eq!(FeedbackResolution::FIELD_COUNT, 9);
        assert_eq!(roles::DEVELOPER, "developer");
        assert_eq!(roles::TENANT_ADMIN, "tenant_admin");

        // 7 FeedbackType + 4 Severity + 6 FeedbackStatus 完整性
        let types = [
            FeedbackType::Question,
            FeedbackType::Architecture,
            FeedbackType::ReviewFinding,
            FeedbackType::Security,
            FeedbackType::Conflict,
            FeedbackType::TestFailure,
            FeedbackType::Other,
        ];
        assert_eq!(types.len(), 7);
        let sevs = [Severity::P0, Severity::P1, Severity::P2, Severity::P3];
        assert_eq!(sevs.len(), 4);
        let stats = [
            FeedbackStatus::Open,
            FeedbackStatus::Acknowledged,
            FeedbackStatus::Applied,
            FeedbackStatus::Verified,
            FeedbackStatus::Rejected,
            FeedbackStatus::Superseded,
        ];
        assert_eq!(stats.len(), 6);
    }
}

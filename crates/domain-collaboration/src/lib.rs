//! Collaboration 领域(Realtime Presence + 协作投影)
//!
//! **crate**: `domain-collaboration`
//! **上游 spec**: docs/specs/domain-collaboration-spec.md §15 Realtime / Presence
//! **基本设计**: docs/basic-design.md §2.1(表 24) / §1.1 部署图 / §15 REQ-RT-001~003
//! **数据设计**: docs/data-design.md §4.17 (`collaboration` schema)
//! **API 设计**: docs/api-design.md §3.18 (Presence) / §4 (Realtime WS 通道)
//!
//! ## 职责
//!
//! 协作(实时状态、Presence)(§15, REQ-RT-001~003):
//! - 1 个核心聚合根 `CollaborationSession`(11 字段)
//! - 3 个子实体 `PresenceParticipant` / `PresenceCursor` / `RealtimeChannel`
//! - 6 个核心 Domain Event(CloudEvents 1.0 envelope)
//! - 2 个端口 `CollaborationCommandPort`(6 方法) / `CollaborationQueryPort`(4 方法) +
//!   1 个仓库端口 `CollaborationRepository` + 1 个路由端口 `RealtimeEventRouter`
//! - 8 条不变量检查(INV-CB-01~08)
//! - 1 个 `InMemoryCollaborationService` 真实实现
//!
//! ## 关键不变量
//!
//! - 必带 tenant_id,跨 tenant 拒绝(INV-CB-01,§6.1, REQ-SEC-001)
//! - 单 Connection ≤ 100 Subscription(INV-CB-02,api-design §4.2)
//! - Presence 60s 心跳过期(INV-CB-03,spec §2,basic-design §23.4)
//! - Realtime Event 必带 tenant_id,跨 tenant 推送拒绝(INV-CB-04)
//! - Subscription filter.resource_types 非空(INV-CB-05)
//! - Project 范围匹配(INV-CB-06)
//! - Session owner 才能 close(INV-CB-07)
//! - Cursor 选区范围合法(INV-CB-08)
//!
//! ## 上游依赖
//!
//! 本 crate 仅依赖自身外部依赖,无跨 domain-* crate 依赖(强类型 ID newtype 隔离)。
//!
//! ## 关键引用
//!
//! 高频 Token Stream 可旁路(§15, REQ-RT-003);MVP 不拆 realtime-service(§13.1, §15)

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

// =====================================================================
// 子模块装载
// =====================================================================

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

pub use context::ActorContext;
pub use entity::{
    CollaborationSession, PresenceCursor, PresenceParticipant, RealtimeChannel,
};
pub use error::CollaborationError;
pub use event::{
    CollaborationEvent, CursorMoved, EventMeta, HeartbeatReceived, ParticipantJoined,
    ParticipantLeft, SessionClosed, SessionOpened,
};
pub use invariants::{
    check_create_invariants, check_invariant_01_tenant_id_present,
    check_invariant_02_channel_quota, check_invariant_03_heartbeat_not_expired,
    check_invariant_04_event_tenant_match, check_invariant_05_channel_filter_not_empty,
    check_invariant_06_project_scope_match, check_invariant_07_owner_or_admin,
    check_invariant_08_cursor_selection_valid, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    CloseSessionCommand, CollaborationCommandPort, CollaborationQueryPort,
    CollaborationRepository, GetCursorQuery, HeartbeatCommand, JoinSessionCommand,
    LeaveSessionCommand, ListActiveSessionsQuery, ListParticipantsQuery, OpenSessionCommand,
    RealtimeEventRouter, UpdateCursorCommand,
};
pub use service::{
    InMemoryCollaborationService, CHANNEL_TTL_SECS, DEFAULT_HEARTBEAT_TIMEOUT_SECS,
    DEFAULT_SESSION_IDLE_SECS, MAX_CHANNELS_PER_CONNECTION,
};
pub use value_object::{
    permissions, roles, ChannelId, ParticipantId, ParticipantStatus, ProjectId, ResourceType,
    SelectionShape, SessionId, TenantId, UserId, WorkspaceId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{SelectionShape, SessionId, TenantId, UserId};
    use std::sync::Arc;

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::DEVELOPER)
            .with_project(ProjectId::new())
    }

    fn make_open_cmd(tenant_id: TenantId, project_id: ProjectId) -> OpenSessionCommand {
        OpenSessionCommand {
            tenant_id,
            project_id,
            workspace_id: None,
            name: "test-session".to_string(),
            description: Some("unit test session".to_string()),
            is_open: true,
        }
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.has_role(roles::DEVELOPER));
    }

    // -------- 2. FIELD_COUNT 字段数审计 --------

    #[test]
    fn entity_field_count_audit() {
        assert_eq!(CollaborationSession::FIELD_COUNT, 11);
        assert_eq!(PresenceParticipant::FIELD_COUNT, 11);
        assert_eq!(PresenceCursor::FIELD_COUNT, 12);
        assert_eq!(RealtimeChannel::FIELD_COUNT, 10);
    }

    // -------- 3. open_session 成功路径 --------

    #[tokio::test]
    async fn open_session_success() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_open_cmd(tenant_id, actor.project_ids[0]);
        let s = svc.open_session(cmd, actor).await.expect("open_session");
        assert_eq!(s.name, "test-session");
        assert_eq!(s.lock_version, 1);
        assert!(s.is_open);
        assert_eq!(svc.count_sessions().await, 1);
    }

    // -------- 4. INV-CB-01:跨租户拒绝 --------

    #[tokio::test]
    async fn invariant_01_cross_tenant_rejected() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let actor_b = make_test_actor(tenant_b);
        let cmd = make_open_cmd(tenant_a, actor_a.project_ids[0]);
        let s = svc.open_session(cmd, actor_a).await.unwrap();
        // 跨租户读取
        let res = svc.get_session(s.id, actor_b).await;
        assert!(matches!(res, Err(CollaborationError::PermissionDenied)));
    }

    // -------- 5. join_session + heartbeat + stale 判定 --------

    #[tokio::test]
    async fn join_heartbeat_and_stale_detection() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let open_cmd = make_open_cmd(tenant_id, actor.project_ids[0]);
        let session = svc.open_session(open_cmd, actor.clone()).await.unwrap();

        // join
        let join = JoinSessionCommand {
            tenant_id,
            session_id: session.id,
            user_id: actor.user_id,
            resource_type: Some("worktree".to_string()),
            resource_id: Some(uuid::Uuid::new_v4()),
        };
        let p = svc.join_session(join, actor.clone()).await.expect("join");
        assert_eq!(p.status, ParticipantStatus::Active);
        assert_eq!(svc.count_participants().await, 1);

        // heartbeat
        let hb = HeartbeatCommand {
            tenant_id,
            session_id: session.id,
            participant_id: p.id,
            client_now: None,
        };
        let p2 = svc.heartbeat(hb, actor.clone()).await.expect("heartbeat");
        assert_eq!(p2.status, ParticipantStatus::Active);
        // heartbeat 之后 last_active_at 应该更新
        assert!(p2.last_active_at >= p.last_active_at);

        // 构造一个明显 stale 的 Participant(从未来 100s 的视角,心跳早已过期)
        let now_plus_100s = chrono::Utc::now() + chrono::Duration::seconds(100);
        assert!(p2.is_stale(now_plus_100s, DEFAULT_HEARTBEAT_TIMEOUT_SECS));
    }

    // -------- 6. update_cursor + INV-CB-08 选区合法/非法 --------

    #[tokio::test]
    async fn update_cursor_and_invariant_08() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let open_cmd = make_open_cmd(tenant_id, actor.project_ids[0]);
        let session = svc.open_session(open_cmd, actor.clone()).await.unwrap();
        let join = JoinSessionCommand {
            tenant_id,
            session_id: session.id,
            user_id: actor.user_id,
            resource_type: None,
            resource_id: None,
        };
        let p = svc.join_session(join, actor.clone()).await.unwrap();

        // 合法 Point cursor
        let c1 = UpdateCursorCommand {
            tenant_id,
            session_id: session.id,
            participant_id: p.id,
            resource_type: "worktree".to_string(),
            resource_id: uuid::Uuid::new_v4(),
            position_x: 10,
            position_y: 20,
            selection_start: None,
            selection_end: None,
            selection_shape: SelectionShape::Point,
        };
        let cur = svc.update_cursor(c1, actor.clone()).await.expect("point cursor");
        assert_eq!(cur.position_x, 10);
        assert_eq!(cur.position_y, 20);

        // 合法 Range cursor
        let c2 = UpdateCursorCommand {
            tenant_id,
            session_id: session.id,
            participant_id: p.id,
            resource_type: "worktree".to_string(),
            resource_id: uuid::Uuid::new_v4(),
            position_x: 0,
            position_y: 0,
            selection_start: Some(5),
            selection_end: Some(15),
            selection_shape: SelectionShape::Range,
        };
        svc.update_cursor(c2, actor.clone()).await.expect("range cursor");

        // 非法:start > end
        let c_bad = UpdateCursorCommand {
            tenant_id,
            session_id: session.id,
            participant_id: p.id,
            resource_type: "worktree".to_string(),
            resource_id: uuid::Uuid::new_v4(),
            position_x: 0,
            position_y: 0,
            selection_start: Some(20),
            selection_end: Some(10),
            selection_shape: SelectionShape::Range,
        };
        let res = svc.update_cursor(c_bad, actor.clone()).await;
        assert!(matches!(res, Err(CollaborationError::InvalidState(_))));

        // get_cursor 验证最新值
        let q = GetCursorQuery {
            tenant_id,
            session_id: session.id,
            participant_id: p.id,
        };
        let got = svc.get_cursor(q, actor).await.expect("get_cursor");
        assert!(got.is_some());
    }

    // -------- 7. list_participants 含 stale 过滤 --------

    #[tokio::test]
    async fn list_participants_excludes_stale() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let open_cmd = make_open_cmd(tenant_id, actor.project_ids[0]);
        let session = svc.open_session(open_cmd, actor.clone()).await.unwrap();
        let join = JoinSessionCommand {
            tenant_id,
            session_id: session.id,
            user_id: actor.user_id,
            resource_type: None,
            resource_id: None,
        };
        let p = svc.join_session(join, actor.clone()).await.unwrap();

        let q = ListParticipantsQuery {
            tenant_id,
            session_id: session.id,
            status_filter: None,
        };
        let active = svc.list_participants(q, actor).await.expect("list");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, p.id);
    }

    // -------- 8. INV-CB-07:非 owner 不可 close --------

    #[tokio::test]
    async fn invariant_07_non_owner_cannot_close() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_id = TenantId::new();
        let owner = make_test_actor(tenant_id);
        let open_cmd = make_open_cmd(tenant_id, owner.project_ids[0]);
        let session = svc.open_session(open_cmd, owner.clone()).await.unwrap();

        // 另一个非 owner actor
        let other = ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::DEVELOPER)
            .with_project(owner.project_ids[0]);
        let close = CloseSessionCommand {
            tenant_id,
            session_id: session.id,
        };
        let res = svc.close_session(close, other).await;
        assert!(matches!(res, Err(CollaborationError::PermissionDenied)));
        // owner 可 close
        let close2 = CloseSessionCommand {
            tenant_id,
            session_id: session.id,
        };
        let res2 = svc.close_session(close2, owner).await;
        assert!(res2.is_ok());
        assert_eq!(svc.count_sessions().await, 0);
    }

    // -------- 9. close_session 级联清理 Participant / Cursor --------

    #[tokio::test]
    async fn close_session_cascades_participants_and_cursors() {
        let svc = InMemoryCollaborationService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let open_cmd = make_open_cmd(tenant_id, actor.project_ids[0]);
        let session = svc.open_session(open_cmd, actor.clone()).await.unwrap();
        let join = JoinSessionCommand {
            tenant_id,
            session_id: session.id,
            user_id: actor.user_id,
            resource_type: None,
            resource_id: None,
        };
        let p = svc.join_session(join, actor.clone()).await.unwrap();
        let cur = UpdateCursorCommand {
            tenant_id,
            session_id: session.id,
            participant_id: p.id,
            resource_type: "wt".to_string(),
            resource_id: uuid::Uuid::new_v4(),
            position_x: 1,
            position_y: 2,
            selection_start: None,
            selection_end: None,
            selection_shape: SelectionShape::Point,
        };
        svc.update_cursor(cur, actor.clone()).await.unwrap();
        assert_eq!(svc.count_participants().await, 1);

        let close = CloseSessionCommand {
            tenant_id,
            session_id: session.id,
        };
        svc.close_session(close, actor).await.unwrap();
        assert_eq!(svc.count_sessions().await, 0);
        assert_eq!(svc.count_participants().await, 0);
    }

    // -------- 10. INV-CB-02:Channel 超过 100/Connection 拒绝 --------

    #[tokio::test]
    async fn invariant_02_channel_quota_rejected() {
        let svc = InMemoryCollaborationService::new_for_test();
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        // 直接通过 repository 插入 100 个 Channel
        for _ in 0..MAX_CHANNELS_PER_CONNECTION {
            let c = RealtimeChannel {
                id: ChannelId::new(),
                session_id: SessionId::new(),
                tenant_id,
                user_id,
                filter_resource_types: vec![ResourceType::Presence],
                filter_project_ids: vec![],
                last_event_id: None,
                is_active: true,
                expires_at: chrono::Utc::now() + chrono::Duration::seconds(CHANNEL_TTL_SECS),
                last_ping_at: chrono::Utc::now(),
            };
            svc.insert_channel(&c).await.expect("insert_channel");
        }
        // 第 101 个被拒
        let c101 = RealtimeChannel {
            id: ChannelId::new(),
            session_id: SessionId::new(),
            tenant_id,
            user_id,
            filter_resource_types: vec![ResourceType::Presence],
            filter_project_ids: vec![],
            last_event_id: None,
            is_active: true,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(CHANNEL_TTL_SECS),
            last_ping_at: chrono::Utc::now(),
        };
        let res = svc.insert_channel(&c101).await;
        assert!(matches!(res, Err(CollaborationError::RateLimited(_))));
    }

    // -------- 11. INV-CB-05:Channel filter.resource_types 为空被拒 --------

    #[tokio::test]
    async fn invariant_05_empty_filter_rejected() {
        let svc = InMemoryCollaborationService::new_for_test();
        let c = RealtimeChannel {
            id: ChannelId::new(),
            session_id: SessionId::new(),
            tenant_id: TenantId::new(),
            user_id: UserId::new(),
            filter_resource_types: vec![],
            filter_project_ids: vec![],
            last_event_id: None,
            is_active: true,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(CHANNEL_TTL_SECS),
            last_ping_at: chrono::Utc::now(),
        };
        let res = svc.insert_channel(&c).await;
        assert!(matches!(res, Err(CollaborationError::InvalidState(_))));
    }

    // -------- 12. INV-CB-04:Realtime 路由跨 tenant 拒绝 --------

    #[tokio::test]
    async fn invariant_04_event_tenant_mismatch_blocked() {
        let svc = Arc::new(InMemoryCollaborationService::new().0);
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let user_id = UserId::new();
        // tenant_a 下插入一个 Channel
        let c = RealtimeChannel {
            id: ChannelId::new(),
            session_id: SessionId::new(),
            tenant_id: tenant_a,
            user_id,
            filter_resource_types: vec![ResourceType::Presence],
            filter_project_ids: vec![],
            last_event_id: None,
            is_active: true,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(CHANNEL_TTL_SECS),
            last_ping_at: chrono::Utc::now(),
        };
        svc.insert_channel(&c).await.unwrap();

        // tenant_b 的事件不投递
        let ev_id = uuid::Uuid::new_v4();
        let delivered = svc
            .route(
                "star.events.test",
                ev_id,
                tenant_b,
                ProjectId::new(),
                ResourceType::Presence,
            )
            .await
            .unwrap();
        assert_eq!(delivered, 0);

        // tenant_a 的事件正常投递
        let ev_id2 = uuid::Uuid::new_v4();
        let delivered2 = svc
            .route(
                "star.events.test",
                ev_id2,
                tenant_a,
                ProjectId::new(),
                ResourceType::Presence,
            )
            .await
            .unwrap();
        assert_eq!(delivered2, 1);
    }

    // -------- 13. event bus 收到 SessionOpened / SessionClosed --------

    #[tokio::test]
    async fn event_bus_receives_session_lifecycle() {
        let (svc, mut rx) = InMemoryCollaborationService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let open_cmd = make_open_cmd(tenant_id, actor.project_ids[0]);
        let session = svc.open_session(open_cmd, actor.clone()).await.unwrap();
        let close = CloseSessionCommand {
            tenant_id,
            session_id: session.id,
        };
        svc.close_session(close, actor).await.unwrap();

        // 检查收到 SessionOpened 和 SessionClosed
        let mut got_opened = false;
        let mut got_closed = false;
        for _ in 0..20 {
            if let Ok(evt) = rx.try_recv() {
                match evt {
                    CollaborationEvent::SessionOpened(_) => got_opened = true,
                    CollaborationEvent::SessionClosed(_) => got_closed = true,
                    _ => {}
                }
                if got_opened && got_closed {
                    break;
                }
            }
        }
        assert!(got_opened, "应收到 SessionOpened 事件");
        assert!(got_closed, "应收到 SessionClosed 事件");
    }
}

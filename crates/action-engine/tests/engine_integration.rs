//! action-engine 集成测试 (WBS 9.6 覆盖率补写)
//!
//! 覆盖 src/ 内联 UT 之外的公共 API 表面:
//! - ActionType 21 个变体的分类矩阵 / serde / Default
//! - dispatch 全路径: 各 action 的 new_state 映射、RBAC、confirm、idempotency replay
//! - ActionRegistry 部分注册 / 空注册 → not_available 分支
//! - IdempotencyStore 显式 now 注入的 TTL 边界
//! - AuditWriter 查询接口 + 错误 Display
//! - RetryPolicy serde + retry_with_backoff 各分支
//! - ActionError 6-field / Display / serde

use action_engine::action::{
    default_metadata, ActionClass, ActionRequest, ActionResult, ActionType,
};
use action_engine::audit_writer::{AuditRecord, AuditWriter};
use action_engine::engine::{ActionEngine, ActionRegistry};
use action_engine::error::ActionError;
use action_engine::idempotency::{IdempotencyRecord, IdempotencyStore, IDEMPOTENCY_TTL};
use action_engine::rbac::{check_permission, PermissionDecision, Role, User};
use action_engine::retry::{decide, retry_with_backoff, RetryDecision, RetryError, RetryPolicy};
use chrono::{Duration as ChronoDuration, Utc};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

// =====================================================================
// ActionType / metadata 矩阵
// =====================================================================

#[test]
fn action_type_all_21_variants_with_expected_classification() {
    let mut safe = 0;
    let mut warning = 0;
    let mut destructive = 0;
    for at in ActionType::ALL.iter().copied() {
        let m = default_metadata(at);
        match m.classification {
            ActionClass::Safe => safe += 1,
            ActionClass::Warning => warning += 1,
            ActionClass::Destructive => destructive += 1,
        }
    }
    assert_eq!((safe, warning, destructive), (9, 5, 7));
    assert_eq!(ActionType::COUNT, 21);
}

#[test]
fn action_type_serde_roundtrip_all_21() {
    for at in ActionType::ALL.iter().copied() {
        let json = serde_json::to_string(&at).unwrap();
        let parsed: ActionType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, at);
    }
    // snake_case 形态抽查
    assert_eq!(
        serde_json::to_string(&ActionType::OpenInIDE).unwrap(),
        "\"open_in_i_d_e\""
    );
    assert_eq!(
        serde_json::to_string(&ActionType::RemoveDependency).unwrap(),
        "\"remove_dependency\""
    );
}

#[test]
fn action_type_default_is_open() {
    assert_eq!(ActionType::default(), ActionType::Open);
}

#[test]
fn destructive_actions_not_reversible_safe_are() {
    for at in ActionType::ALL.iter().copied() {
        let m = default_metadata(at);
        if m.classification == ActionClass::Destructive {
            assert!(!m.reversible, "{at:?} must be irreversible");
            assert!(m.requires_confirm, "{at:?} must require confirm");
        }
        if m.classification == ActionClass::Safe {
            assert!(!m.requires_confirm, "{at:?} must not require confirm");
        }
    }
    // 具体反例 (Archive 可逆, Delete 不可逆)
    assert!(default_metadata(ActionType::Archive).reversible);
    assert!(!default_metadata(ActionType::Delete).reversible);
}

#[test]
fn action_request_default_and_serde() {
    let r = ActionRequest::default();
    assert_eq!(r.action_type, ActionType::Open);
    assert!(r.worktree_id.is_nil());
    assert!(!r.confirm);
    assert_eq!(r.params, serde_json::Value::Null);

    let r = ActionRequest {
        action_type: ActionType::Merge,
        worktree_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        confirm: true,
        params: serde_json::json!({"strategy": "squash"}),
    };
    let json = serde_json::to_string(&r).unwrap();
    let parsed: ActionRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.action_type, ActionType::Merge);
    assert!(parsed.confirm);
    assert_eq!(parsed.params["strategy"], "squash");
}

#[test]
fn action_result_serde_roundtrip() {
    let r = ActionResult {
        success: false,
        worktree_id: Uuid::new_v4(),
        new_state: None,
        duration_ms: 9,
        warnings: vec![action_engine::ActionWarning {
            code: "WARN.X".into(),
            message: "w".into(),
        }],
        errors: vec!["boom".into()],
    };
    let json = serde_json::to_string(&r).unwrap();
    let parsed: ActionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.success, false);
    assert_eq!(parsed.errors, vec!["boom".to_string()]);
    assert_eq!(parsed.warnings[0].code, "WARN.X");
}

// =====================================================================
// ActionError
// =====================================================================

#[test]
fn action_error_display_and_source_chain() {
    let e = ActionError::timeout("Merge", 3000)
        .with_source("inner")
        .at("a.rs", 1);
    let s = e.to_string();
    assert!(
        s.contains("[ACTION_TIMEOUT] action Merge timed out after 3000 ms"),
        "{s}"
    );
    assert!(s.contains("cause: inner"), "{s}");
    assert!(s.contains("a.rs:1"), "{s}");
}

#[test]
fn action_error_serde_roundtrip() {
    let e = ActionError::permission_denied("u1").with_context(serde_json::json!({"k": 1}));
    let json = serde_json::to_string(&e).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["code"], "PERMISSION_DENIED");
    assert_eq!(v["context"]["k"], 1);
    let parsed: ActionError = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.code, "PERMISSION_DENIED");
    assert_eq!(parsed.trace_id, e.trace_id);
}

// =====================================================================
// RBAC
// =====================================================================

#[test]
fn role_serde_and_default() {
    for role in Role::ALL {
        let json = serde_json::to_string(&role).unwrap();
        let parsed: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, role);
    }
    assert_eq!(Role::default(), Role::SRE);
}

#[test]
fn user_has_role_and_domain_id() {
    let mut u = User::new(Uuid::new_v4(), Role::Lead);
    assert!(u.has_role(Role::Lead));
    assert!(!u.has_role(Role::Owner));
    assert!(u.domain_id.is_none());
    u.domain_id = Some("canvas".into());
    let json = serde_json::to_string(&u).unwrap();
    let parsed: User = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.domain_id.as_deref(), Some("canvas"));
}

#[test]
fn sre_matrix_cases() {
    let sre = User::new(Uuid::new_v4(), Role::SRE);
    assert_eq!(
        check_permission(&sre, ActionType::ForceRebase),
        PermissionDecision::Allow
    );
    assert_eq!(
        check_permission(&sre, ActionType::ExplainRisk),
        PermissionDecision::Allow
    );
    // SRE 不能 CreatePR / Archive / Cleanup
    assert_eq!(
        check_permission(&sre, ActionType::CreatePR),
        PermissionDecision::Deny
    );
    assert_eq!(
        check_permission(&sre, ActionType::Cleanup),
        PermissionDecision::Deny
    );
}

#[test]
fn pm_and_viewer_read_only_matrix() {
    for role in [Role::PM, Role::Viewer] {
        let u = User::new(Uuid::new_v4(), role);
        for allowed in [
            ActionType::Focus,
            ActionType::ExplainRisk,
            ActionType::Compare,
            ActionType::Open,
            ActionType::OpenInIDE,
            ActionType::ListEvents,
        ] {
            assert_eq!(
                check_permission(&u, allowed),
                PermissionDecision::Allow,
                "{role:?} should allow {allowed:?}"
            );
        }
        assert_eq!(
            check_permission(&u, ActionType::AiChat),
            PermissionDecision::Deny,
            "{role:?} should deny AiChat (AI action not in read-only matrix)"
        );
    }
}

// =====================================================================
// ActionRegistry
// =====================================================================

#[test]
fn registry_register_single_and_metadata() {
    let mut reg = ActionRegistry::new();
    assert_eq!(reg.registered_count(), 0);
    reg.register(ActionType::Merge);
    assert_eq!(reg.registered_count(), 1);
    let m = reg.metadata(ActionType::Merge).expect("registered");
    assert_eq!(m.classification, ActionClass::Destructive);
    assert!(reg.metadata(ActionType::Delete).is_none());
    // Debug 输出不含 action 内部表
    assert!(format!("{reg:?}").contains("registered"));
    assert_eq!(reg.registered_types(), vec![ActionType::Merge]);
}

#[test]
fn registry_register_all_sorted_types() {
    let mut reg = ActionRegistry::new();
    reg.register_all();
    assert_eq!(reg.registered_count(), 21);
    let types = reg.registered_types();
    assert_eq!(types.len(), 21);
    // 按 enum discriminant 升序
    assert!(types.windows(2).all(|w| (w[0] as u8) < (w[1] as u8)));
    // 默认构造 = 空
    assert_eq!(ActionRegistry::default().registered_count(), 0);
}

// =====================================================================
// ActionEngine::dispatch 全分支
// =====================================================================

fn make_req(action_type: ActionType, confirm: bool) -> ActionRequest {
    ActionRequest {
        action_type,
        worktree_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        confirm,
        params: serde_json::Value::Null,
    }
}

#[tokio::test]
async fn dispatch_new_state_mapping_per_action() {
    let cases: Vec<(ActionType, Option<&str>)> = vec![
        (ActionType::Merge, Some("merged")),
        (ActionType::Delete, Some("deleted")),
        (ActionType::ForceDelete, Some("deleted")),
        (ActionType::Cleanup, Some("deleted")),
        (ActionType::Archive, Some("archived")),
        (ActionType::MarkSuperseded, Some("superseded")),
        (ActionType::SyncMain, Some("synced")),
        (ActionType::Rebase, Some("synced")),
        (ActionType::ForceRebase, Some("synced")),
        (ActionType::Compare, None),
        (ActionType::Focus, None),
        (ActionType::CreatePR, None),
        (ActionType::AiChat, None),
    ];
    for (action, expected) in cases {
        let engine = ActionEngine::new();
        let req = make_req(action, true);
        let result = engine
            .dispatch(req, Role::Owner, Uuid::new_v4())
            .await
            .unwrap_or_else(|e| panic!("{action:?} should dispatch: {e}"));
        assert!(result.success);
        assert_eq!(
            result.new_state.as_deref(),
            expected,
            "{action:?} new_state mismatch"
        );
    }
}

#[tokio::test]
async fn dispatch_warning_action_requires_confirm() {
    let engine = ActionEngine::new();
    let req = make_req(ActionType::Rebase, false);
    let err = engine
        .dispatch(req, Role::SRE, Uuid::new_v4())
        .await
        .unwrap_err();
    assert_eq!(err.code, "ACTION_CONFIRM_REQUIRED");
}

#[tokio::test]
async fn dispatch_viewer_denied_for_ai_chat() {
    let engine = ActionEngine::new();
    let req = make_req(ActionType::AiChat, false);
    let err = engine
        .dispatch(req, Role::Viewer, Uuid::new_v4())
        .await
        .unwrap_err();
    assert_eq!(err.code, "ACTION_DENIED_BY_RBAC");
}

#[tokio::test]
async fn dispatch_empty_registry_returns_not_available() {
    let engine = ActionEngine::with_registry(ActionRegistry::new());
    let req = make_req(ActionType::Compare, false);
    let err = engine
        .dispatch(req, Role::Owner, Uuid::new_v4())
        .await
        .unwrap_err();
    assert_eq!(err.code, "ACTION_NOT_AVAILABLE");
}

#[tokio::test]
async fn dispatch_audit_and_idempotency_recorded() {
    let engine = ActionEngine::new();
    let wt = Uuid::new_v4();
    let req = ActionRequest {
        action_type: ActionType::Compare,
        worktree_id: wt,
        user_id: Uuid::new_v4(),
        confirm: false,
        params: serde_json::json!({"a": 1}),
    };
    let key = Uuid::new_v4();
    engine.dispatch(req, Role::PM, key).await.unwrap();

    let audit = engine
        .audit()
        .by_idempotency_key(key)
        .expect("audit present");
    assert_eq!(audit.worktree_id, wt);
    assert_eq!(audit.action_type, "Compare");
    assert_eq!(audit.result, "success");
    assert_eq!(audit.payload, serde_json::json!({"a": 1}));
    assert!(audit.is_current);
    assert!(audit.valid_to.is_none());

    let record = engine.idempotency().check(key).expect("idempotency record");
    let replayed: ActionResult = serde_json::from_value(record.result).unwrap();
    assert!(replayed.success);
    assert_eq!(replayed.worktree_id, wt);
}

#[tokio::test]
async fn dispatch_list_types_has_all_21() {
    let engine = ActionEngine::new();
    assert_eq!(engine.list_types().len(), 21);
    assert_eq!(engine.registry().registered_count(), 21);
    assert!(engine.idempotency().is_empty());
    assert!(engine.audit().is_empty());
    assert!(format!("{engine:?}").contains("registered"));
}

// =====================================================================
// IdempotencyStore TTL 边界 (显式 now 注入)
// =====================================================================

fn record_at(key: Uuid, created: chrono::DateTime<Utc>) -> IdempotencyRecord {
    IdempotencyRecord {
        key,
        worktree_id: Uuid::new_v4(),
        result: serde_json::json!({"ok": true}),
        created_at: created,
    }
}

#[test]
fn idempotency_ttl_boundary_exactly_24h() {
    let store = IdempotencyStore::new();
    let base = Utc::now();

    // 记录创建于恰好 24h 前 (秒级截断, 与 is_expired 内部实现对齐)
    let created = base - ChronoDuration::seconds(IDEMPOTENCY_TTL.as_secs() as i64);
    let created_ts = created.timestamp().max(0) as u64;
    let key_edge = Uuid::new_v4();
    store.record(record_at(key_edge, created));

    // now = 恰好 24h 边界: live == TTL → 未过期 (实现用 > 而非 >=)
    let now_exact =
        SystemTime::UNIX_EPOCH + Duration::from_secs(created_ts + IDEMPOTENCY_TTL.as_secs());
    assert!(
        store.check_at(key_edge, now_exact).is_some(),
        "exactly 24h is still live"
    );

    // now = 24h + 1s → 过期
    let now_over =
        SystemTime::UNIX_EPOCH + Duration::from_secs(created_ts + IDEMPOTENCY_TTL.as_secs() + 1);
    assert!(
        store.check_at(key_edge, now_over).is_none(),
        "24h+1s must expire"
    );

    // 未记录 key → None
    assert!(store.check_at(Uuid::new_v4(), now_exact).is_none());
}

#[test]
fn idempotency_evict_expired_at_keeps_live() {
    let store = IdempotencyStore::new();
    let base = Utc::now();
    let live_key = Uuid::new_v4();
    let dead_key = Uuid::new_v4();
    store.record(record_at(live_key, base));
    store.record(record_at(
        dead_key,
        base - ChronoDuration::seconds(25 * 3600),
    ));
    assert_eq!(store.len(), 2);

    store.evict_expired_at(SystemTime::now());
    assert_eq!(store.len(), 1);
    assert!(store.check(live_key).is_some());
    assert!(store.check(dead_key).is_none());
}

#[test]
fn idempotency_record_overwrite_same_key() {
    let store = IdempotencyStore::new();
    let key = Uuid::new_v4();
    store.record(record_at(key, Utc::now()));
    store.record(IdempotencyRecord {
        key,
        worktree_id: Uuid::new_v4(),
        result: serde_json::json!({"second": true}),
        created_at: Utc::now(),
    });
    assert_eq!(store.len(), 1);
    let hit = store.check(key).unwrap();
    assert_eq!(hit.result, serde_json::json!({"second": true}));
}

// =====================================================================
// AuditWriter
// =====================================================================

#[test]
fn audit_writer_queries_and_ordering() {
    let w = AuditWriter::new();
    assert!(w.is_empty());
    let wt_a = Uuid::new_v4();
    let wt_b = Uuid::new_v4();
    let key1 = Uuid::new_v4();
    let key2 = Uuid::new_v4();

    let id1 = w
        .append(
            "Delete",
            wt_a,
            Uuid::new_v4(),
            "user",
            "success",
            key1,
            serde_json::json!({}),
        )
        .unwrap();
    let id2 = w
        .append(
            "Merge",
            wt_b,
            Uuid::new_v4(),
            "agent",
            "failed",
            key2,
            serde_json::json!({"s": 1}),
        )
        .unwrap();
    assert_eq!((id1, id2), (1, 2));

    let all: Vec<AuditRecord> = w.all();
    assert_eq!(all.len(), 2);
    assert!(all[0].id < all[1].id, "append order by id asc");
    assert!(all[0].valid_from <= Utc::now());

    assert!(w.by_idempotency_key(key1).is_some());
    assert!(w.by_idempotency_key(Uuid::new_v4()).is_none());
    assert_eq!(w.by_worktree(wt_a).len(), 1);
    assert_eq!(w.by_worktree(wt_b).len(), 1);
    assert_eq!(w.by_worktree(Uuid::new_v4()).len(), 0);
    assert_eq!(w.len(), 2);
}

#[test]
fn audit_writer_error_display() {
    let e = action_engine::AuditWriterError {
        code: "AUDIT.LOCK".into(),
        message: "poisoned".into(),
        trace_id: "t1".into(),
    };
    let s = e.to_string();
    assert!(s.contains("[AUDIT.LOCK] poisoned"), "{s}");
    assert!(s.contains("trace=t1"), "{s}");
    // StdError 实现
    let _: &dyn std::error::Error = &e;
}

// =====================================================================
// Retry
// =====================================================================

#[test]
fn retry_policy_and_decision_serde_roundtrip() {
    let policies = [
        RetryPolicy::Network { attempt: 1 },
        RetryPolicy::GraphConnection { attempt: 0 },
        RetryPolicy::LlmTimeout { attempt: 1 },
    ];
    for p in policies {
        let json = serde_json::to_string(&p).unwrap();
        let parsed: RetryPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }
    let d = RetryDecision::Retry { delay_ms: 500 };
    let json = serde_json::to_string(&d).unwrap();
    let parsed: RetryDecision = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, d);
}

#[test]
fn decide_exhausted_at_max_attempts() {
    assert_eq!(
        decide(&RetryPolicy::Network { attempt: 3 }),
        RetryDecision::Exhausted
    );
    assert_eq!(
        decide(&RetryPolicy::LlmTimeout { attempt: 2 }),
        RetryDecision::Exhausted
    );
    assert_eq!(
        decide(&RetryPolicy::Network { attempt: 0 }),
        RetryDecision::Retry { delay_ms: 1000 }
    );
}

#[test]
fn retry_policy_next_advances_attempt() {
    let p = RetryPolicy::GraphConnection { attempt: 0 };
    let p2 = p.next();
    assert_eq!(p2, RetryPolicy::GraphConnection { attempt: 1 });
}

#[test]
fn retry_with_backoff_success_first_try() {
    let mut calls = 0;
    let out = retry_with_backoff(RetryPolicy::Network { attempt: 0 }, || {
        calls += 1;
        Ok::<u32, RetryError>(7)
    });
    assert_eq!(out.unwrap(), 7);
    assert_eq!(calls, 1);
}

#[test]
fn retry_with_backoff_fatal_no_retry() {
    let mut calls = 0;
    let out = retry_with_backoff(RetryPolicy::Network { attempt: 0 }, || {
        calls += 1;
        Err::<u32, RetryError>(RetryError::Fatal("permission denied".into()))
    });
    assert_eq!(out.unwrap_err(), "permission denied");
    assert_eq!(calls, 1);
}

#[test]
fn retry_with_backoff_llm_timeout_immediate_retry_no_sleep() {
    // LlmTimeout attempt=0 → delay 0ms (立即重试, 不 sleep)
    let started = std::time::Instant::now();
    let mut calls = 0;
    let out = retry_with_backoff(RetryPolicy::LlmTimeout { attempt: 0 }, || {
        calls += 1;
        if calls < 2 {
            Err(RetryError::Retryable("transient".into()))
        } else {
            Ok::<u32, RetryError>(99)
        }
    });
    assert_eq!(out.unwrap(), 99);
    assert_eq!(calls, 2);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "immediate retry must not sleep"
    );
}

#[test]
fn retry_error_display() {
    assert_eq!(RetryError::Retryable("x".into()).to_string(), "x");
    assert_eq!(RetryError::Fatal("y".into()).to_string(), "y");
}

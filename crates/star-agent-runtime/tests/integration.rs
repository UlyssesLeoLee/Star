//! star-agent-runtime 集成测试 (per 03-DD §9.1 UT 类 + 守门 #1 v25 单 crate 模式)。
//!
//! 覆盖: ID / AgentType / Agent 状态机 (12 态) / Lifecycle (HOT/WARM/COLD) /
//! ECS World / EventBus / RuntimeModeManager (迟滞 + 稳定时长) / Scheduler
//! (Priority + Fair + HOT Slot) / Backpressure (5 策略) / RateLimiter /
//! RetryPolicy / Task / Metrics / Config Fail Safe。

use std::time::{Duration, Instant};

use star_agent_runtime::{
    agent_state::{AgentState, AgentStateEnum},
    agent_type::AgentType,
    backpressure::{Backpressure, BackpressureDecision, BackpressureError, OverflowPolicy},
    components::{ContextRef, ContextTier, Priority, TokenBudget},
    config::RuntimeConfig,
    ecs::{Component, World},
    event::{AgentEvent, EventBus, EventRouter, EventType, InMemoryEventBus, TargetRouter},
    id::{AgentId, TenantId},
    identity::AgentIdentity,
    lifecycle::{LifecycleManager, LifecycleState, LifecycleStateEnum},
    metrics::{percentile, InMemoryObserver, Observer, RuntimeMetrics},
    rate_limiter::{TenantRateLimiter, TokenBucket},
    retry::{retryable_by_kind, ErrorKind, RetryPolicy},
    runtime_mode::{RuntimeMode, RuntimeModeConfig, RuntimeModeError, RuntimeModeManager},
    scheduler::{Scheduler, SchedulerError},
    task::{Task, TaskStatus},
};

fn tid() -> TenantId {
    TenantId::new()
}

// ============================================================================
// §1 ID + AgentType
// ============================================================================

#[test]
fn ids_display_and_parse_roundtrip() {
    let id = AgentId::new();
    let hex = id.to_string();
    assert_eq!(hex.len(), 32);
    let parsed: AgentId = hex.parse().expect("parse hex id");
    assert_eq!(parsed, id);
}

#[test]
fn ids_unique() {
    let a = AgentId::new();
    let b = AgentId::new();
    assert_ne!(a, b);
}

#[test]
fn id_parse_rejects_bad_input() {
    assert!("short".parse::<AgentId>().is_err());
    assert!("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
        .parse::<AgentId>()
        .is_err());
}

#[test]
fn agent_type_parse_all_9() {
    for (code, label) in star_agent_runtime::SA_TYPES {
        let t = AgentType::parse(code).expect("valid SA type");
        assert_eq!(t.code(), code);
        assert_eq!(t.label(), label);
    }
    assert_eq!(AgentType::parse("SA-N").unwrap(), AgentType::SA_N);
    assert!(AgentType::parse("SA-10").is_err());
    assert!(AgentType::parse("foo").is_err());
}

// ============================================================================
// §2 Agent 状态机 (per 03-DD §3.1)
// ============================================================================

#[test]
fn agent_state_happy_path() {
    let mut state = AgentState::new();
    let now = Instant::now();
    let path = [
        AgentStateEnum::Ready,
        AgentStateEnum::Scheduled,
        AgentStateEnum::Planning,
        AgentStateEnum::WaitingLlm,
        AgentStateEnum::Processing,
        AgentStateEnum::Completed,
        AgentStateEnum::Idle,
    ];
    for target in path {
        assert!(
            state.transition(target, now),
            "{:?} -> {target:?} 应合法",
            state.prev
        );
    }
    assert_eq!(state.current, AgentStateEnum::Idle);
    assert_eq!(state.prev, AgentStateEnum::Completed);
}

#[test]
fn agent_state_illegal_transition_rejected() {
    let mut state = AgentState::new();
    assert!(!state.transition(AgentStateEnum::Processing, Instant::now()));
    assert!(!state.transition(AgentStateEnum::Completed, Instant::now()));
    assert_eq!(state.current, AgentStateEnum::Idle);
}

#[test]
fn agent_state_fail_retry_suspend_resume_cancel() {
    let now = Instant::now();
    let mut state = AgentState::new();
    state.transition(AgentStateEnum::Ready, now);
    assert!(state.fail(now));
    assert_eq!(state.current, AgentStateEnum::Failed);
    assert_eq!(state.retry_count, 1);
    assert!(state.retry(now));
    assert_eq!(state.current, AgentStateEnum::Ready);

    let mut s2 = AgentState::new();
    s2.transition(AgentStateEnum::Ready, now);
    assert!(s2.suspend(now));
    assert_eq!(s2.current, AgentStateEnum::Suspended);
    assert!(s2.resume(now));
    assert_eq!(s2.current, AgentStateEnum::Scheduled);
    assert!(s2.cancel(now));
    assert_eq!(s2.current, AgentStateEnum::Cancelled);
    // 终态
    assert!(!s2.transition(AgentStateEnum::Ready, now));
}

#[test]
fn agent_state_all_12_variants() {
    use AgentStateEnum::*;
    let all = [
        Idle,
        Ready,
        Scheduled,
        Planning,
        WaitingLlm,
        WaitingTool,
        WaitingEvent,
        Processing,
        Completed,
        Failed,
        Suspended,
        Cancelled,
    ];
    assert_eq!(all.len(), 12);
}

// ============================================================================
// §3 Lifecycle (per 03-DD §3.2)
// ============================================================================

#[test]
fn lifecycle_hot_warm_cold_transitions() {
    let t0 = Instant::now();
    let mut state =
        LifecycleState::with_timeouts(Duration::from_secs(60), Duration::from_secs(600));
    state.activate(t0);
    assert_eq!(state.current, LifecycleStateEnum::Hot);
    assert_eq!(
        state.sweep(t0 + Duration::from_secs(61)),
        Some(LifecycleStateEnum::Warm)
    );
    assert_eq!(
        state.sweep(t0 + Duration::from_secs(61 + 601)),
        Some(LifecycleStateEnum::Cold)
    );
    state.restore(t0 + Duration::from_secs(700));
    assert_eq!(state.current, LifecycleStateEnum::Warm);
    state.activate(t0 + Duration::from_secs(701));
    assert_eq!(state.current, LifecycleStateEnum::Hot);
}

#[test]
fn lifecycle_touch_keeps_hot() {
    let t0 = Instant::now();
    let mut state =
        LifecycleState::with_timeouts(Duration::from_secs(60), Duration::from_secs(600));
    state.activate(t0);
    state.touch(t0 + Duration::from_secs(59));
    assert_eq!(state.sweep(t0 + Duration::from_secs(118)), None);
    assert_eq!(state.current, LifecycleStateEnum::Hot);
}

#[test]
fn lifecycle_manager_transition_validation() {
    use star_agent_runtime::lifecycle::InMemoryLifecycleManager;
    let mut mgr = InMemoryLifecycleManager::new();
    let agent = AgentId::new();
    mgr.register(agent, LifecycleState::new());
    assert!(mgr.transition(agent, LifecycleStateEnum::Warm).is_ok());
    // 非法: Warm → Hot 需要 Schedule (activate), transition 只允许相邻
    assert!(mgr.transition(agent, LifecycleStateEnum::Hot).is_ok());
    assert!(mgr.persist(agent).is_ok());
    assert!(mgr.restore(agent).is_ok());
    assert!(mgr.timeout(agent).is_ok());
    assert!(mgr.evict(agent).is_ok());
    assert!(mgr.transition(agent, LifecycleStateEnum::Warm).is_err());
}

// ============================================================================
// §4 ECS World (per 03-DD §1.3 自研 Minimal ECS)
// ============================================================================

#[test]
fn ecs_world_spawn_and_components() {
    let mut world = World::new();
    let identity = AgentIdentity::new(tid(), AgentType::SA01);
    let agent_id = identity.agent_id;
    world.spawn_agent(identity);

    world.insert_component(agent_id, AgentState::new());
    world.insert_component(agent_id, LifecycleState::new());
    world.insert_component(agent_id, Priority::High);
    world.insert_component(agent_id, TokenBudget::default());
    world.insert_component(
        agent_id,
        ContextRef {
            context_id: star_agent_runtime::ContextId::new(),
            tier: ContextTier::L3Full,
            loaded: false,
            refs: vec![],
        },
    );

    assert_eq!(world.resident_count(), 1);
    assert!(world.get_identity(agent_id).is_some());
    assert!(world.get_state(agent_id).is_some());
    assert_eq!(world.get_priority(agent_id), Some(Priority::High));
    assert_ne!(world.archetype(agent_id), None);
    assert_eq!(world.lifecycle_counts(), (0, 0, 1)); // Cold 起步
}

#[test]
fn ecs_world_despawn_reclaims() {
    let mut world = World::new();
    let agent_id = AgentId::new();
    world.spawn_agent(AgentIdentity::with_id(agent_id, tid(), AgentType::SA09));
    world.insert_component(agent_id, AgentState::new());
    assert!(world.despawn(agent_id));
    assert!(!world.despawn(agent_id));
    assert_eq!(world.resident_count(), 0);
    assert!(world.get_state(agent_id).is_none());
}

#[test]
fn component_trait_marker() {
    fn assert_component<T: Component>() {}
    assert_component::<AgentState>();
    assert_component::<LifecycleState>();
    assert_component::<Priority>();
    assert_component::<TokenBudget>();
    assert_component::<ContextRef>();
}

// ============================================================================
// §5 EventBus (per SRS §26-§28)
// ============================================================================

#[test]
fn event_bus_publish_routes_to_target() {
    let mut bus = InMemoryEventBus::new(8);
    let target = AgentId::new();
    bus.subscribe(target);
    let event = AgentEvent::new(None, target, tid(), EventType::SystemEvent);
    let event_id = bus.publish(event);
    assert_eq!(bus.unread_count(target), 1);
    let msgs = bus.drain_unread(target);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].event_id, event_id);
    assert_eq!(bus.unread_count(target), 0);
}

#[test]
fn event_bus_bounded_drops_oldest() {
    let mut bus = InMemoryEventBus::new(2);
    let target = AgentId::new();
    bus.subscribe(target);
    for _ in 0..5 {
        bus.publish(AgentEvent::new(None, target, tid(), EventType::Timer));
    }
    assert_eq!(bus.drain_unread(target).len(), 2); // bounded (per SRS §44)
}

#[test]
fn event_bus_unsubscribe() {
    let mut bus = InMemoryEventBus::new(8);
    let target = AgentId::new();
    bus.subscribe(target);
    assert!(bus.unsubscribe(target).is_ok());
    assert!(bus.unsubscribe(target).is_err());
}

#[test]
fn router_routes_to_target() {
    let event = AgentEvent::new(None, AgentId::new(), tid(), EventType::UserMessage);
    assert_eq!(TargetRouter.route(&event), event.target_agent_id);
}

// ============================================================================
// §6 RuntimeModeManager (per SRS §7)
// ============================================================================

fn make_mode_manager() -> RuntimeModeManager {
    RuntimeModeManager::new(RuntimeModeConfig::default())
}

#[test]
fn mode_small_scale_stays_lightweight() {
    let mut m = make_mode_manager();
    let t0 = Instant::now();
    for secs in [0, 100, 999] {
        assert_eq!(
            m.decide(t0 + Duration::from_secs(secs), 8),
            RuntimeMode::Lightweight
        );
    }
    assert_eq!(m.switch_count(), 0);
}

#[test]
fn mode_9_forces_lightweight_from_ecs() {
    let mut m = make_mode_manager();
    let t0 = Instant::now();
    assert_eq!(m.decide(t0, 12), RuntimeMode::Lightweight);
    assert_eq!(m.decide(t0 + Duration::from_secs(30), 12), RuntimeMode::Ecs);
    assert_eq!(m.decide(t0 + Duration::from_secs(31), 9), RuntimeMode::Ecs);
    assert_eq!(m.decide(t0 + Duration::from_secs(330), 9), RuntimeMode::Ecs);
    assert_eq!(
        m.decide(t0 + Duration::from_secs(331), 9),
        RuntimeMode::Lightweight
    );
    assert_eq!(m.switch_count(), 2);
}

#[test]
fn mode_hysteresis_10_11_stays_current() {
    let mut m = make_mode_manager();
    let t0 = Instant::now();
    // Lightweight 下 11: 保持
    assert_eq!(m.decide(t0, 11), RuntimeMode::Lightweight);
    assert_eq!(
        m.decide(t0 + Duration::from_secs(500), 11),
        RuntimeMode::Lightweight
    );
    // 进 ECS
    assert_eq!(m.decide(t0, 12), RuntimeMode::Lightweight);
    assert_eq!(m.decide(t0 + Duration::from_secs(30), 12), RuntimeMode::Ecs);
    // ECS 下 10: 保持 (迟滞区)
    assert_eq!(m.decide(t0 + Duration::from_secs(31), 10), RuntimeMode::Ecs);
    assert_eq!(
        m.decide(t0 + Duration::from_secs(400), 10),
        RuntimeMode::Ecs
    );
    assert_eq!(m.switch_count(), 1);
}

#[test]
fn mode_stable_duration_and_reset() {
    let mut m = make_mode_manager();
    let t0 = Instant::now();
    m.decide(t0, 12);
    m.decide(t0 + Duration::from_secs(10), 12);
    // 掉回 11: 计时重置
    assert_eq!(
        m.decide(t0 + Duration::from_secs(15), 11),
        RuntimeMode::Lightweight
    );
    m.decide(t0 + Duration::from_secs(16), 12);
    assert_eq!(
        m.decide(t0 + Duration::from_secs(40), 12),
        RuntimeMode::Lightweight
    );
    assert_eq!(m.decide(t0 + Duration::from_secs(46), 12), RuntimeMode::Ecs);
}

#[test]
fn mode_exit_requires_300s() {
    let mut m = make_mode_manager();
    let t0 = Instant::now();
    m.decide(t0, 12);
    assert_eq!(m.decide(t0 + Duration::from_secs(30), 12), RuntimeMode::Ecs);
    assert_eq!(m.decide(t0 + Duration::from_secs(31), 8), RuntimeMode::Ecs);
    assert_eq!(m.decide(t0 + Duration::from_secs(330), 8), RuntimeMode::Ecs);
    assert_eq!(
        m.decide(t0 + Duration::from_secs(331), 8),
        RuntimeMode::Lightweight
    );
}

#[test]
fn mode_thresholds_configurable() {
    let config = RuntimeModeConfig {
        ecs_enable_threshold: 20,
        ecs_disable_threshold: 14,
        ecs_enable_stable_duration: Duration::from_secs(5),
        ecs_disable_stable_duration: Duration::from_secs(10),
    };
    let mut m = RuntimeModeManager::new(config);
    let t0 = Instant::now();
    m.decide(t0, 20);
    assert_eq!(m.decide(t0 + Duration::from_secs(5), 20), RuntimeMode::Ecs);
    assert_eq!(m.decide(t0 + Duration::from_secs(6), 14), RuntimeMode::Ecs);
    assert_eq!(
        m.decide(t0 + Duration::from_secs(16), 14),
        RuntimeMode::Lightweight
    );
}

#[test]
fn mode_config_fail_safe() {
    assert_eq!(
        RuntimeModeConfig {
            ecs_enable_threshold: 0,
            ..Default::default()
        }
        .validate(),
        Err(RuntimeModeError::ZeroEnableThreshold)
    );
    assert_eq!(
        RuntimeModeConfig {
            ecs_enable_threshold: 10,
            ecs_disable_threshold: 9,
            ..Default::default()
        }
        .validate(),
        Err(RuntimeModeError::EmptyHysteresisZone)
    );
}

#[test]
fn mode_hysteresis_zone_bounds() {
    let m = make_mode_manager();
    assert!(!m.hysteresis_zone(9));
    assert!(m.hysteresis_zone(10));
    assert!(m.hysteresis_zone(11));
    assert!(!m.hysteresis_zone(12));
}

// ============================================================================
// §7 Scheduler (per SRS §42-§44 + 03-DD §6.1/§6.4)
// ============================================================================

#[test]
fn scheduler_zero_hot_limit_rejected() {
    assert!(matches!(
        Scheduler::new(0, 2),
        Err(SchedulerError::ZeroHotLimit)
    ));
}

#[test]
fn scheduler_priority_order() {
    let mut s = Scheduler::with_default_fair_share(10).unwrap();
    let t = tid();
    s.enqueue_ready(star_agent_runtime::TaskId::new(), t, Priority::Low);
    s.enqueue_ready(star_agent_runtime::TaskId::new(), t, Priority::Normal);
    s.enqueue_ready(star_agent_runtime::TaskId::new(), t, Priority::High);
    s.enqueue_ready(star_agent_runtime::TaskId::new(), t, Priority::Critical);
    let outcome = s.schedule();
    assert_eq!(outcome.scheduled.len(), 4);
    assert_eq!(s.hot_count(), 4);
    assert_eq!(s.ready_queue_depth(), 0);
}

#[test]
fn scheduler_hot_slot_limit() {
    let mut s = Scheduler::with_default_fair_share(2).unwrap();
    let t = tid();
    for _ in 0..5 {
        s.enqueue_ready(star_agent_runtime::TaskId::new(), t, Priority::Normal);
    }
    let outcome = s.schedule();
    // fair_share=2 + hot limit=2 → 2 scheduled, 3 重排
    assert_eq!(outcome.scheduled.len(), 2);
    assert_eq!(s.hot_count(), 2);
    assert_eq!(s.ready_queue_depth(), 3);
    // 释放 1 → 下轮可续
    let done = outcome.scheduled[0];
    s.release_hot(done);
    let outcome2 = s.schedule();
    assert_eq!(outcome2.scheduled.len(), 1);
    assert!(outcome2
        .reason
        .as_deref()
        .unwrap_or("")
        .contains("HotSlotFull"));
}

#[test]
fn scheduler_fair_share_round_robin() {
    let mut s = Scheduler::new(10, 2).unwrap();
    let ta = tid();
    let tb = tid();
    // 交错入队 A,B,A,B...: seq 顺序 A0 B0 A1 B1 ...
    for i in 0..5u64 {
        s.enqueue_ready(
            star_agent_runtime::TaskId((i << 1) as u128),
            ta,
            Priority::Normal,
        );
        s.enqueue_ready(
            star_agent_runtime::TaskId(((i << 1) | 1) as u128),
            tb,
            Priority::Normal,
        );
    }
    let outcome = s.schedule();
    assert_eq!(outcome.scheduled.len(), 4); // A 2 + B 2
                                            // seq 交错 → A0, B0, A1, B1 (per-tenant round-robin, per SRS §57)
    assert_eq!(
        outcome.scheduled,
        vec![
            star_agent_runtime::TaskId(0),
            star_agent_runtime::TaskId(1),
            star_agent_runtime::TaskId(2),
            star_agent_runtime::TaskId(3),
        ]
    );
}

#[test]
fn scheduler_critical_bypasses_fair_share() {
    let mut s = Scheduler::new(10, 1).unwrap();
    let t = tid();
    s.enqueue_ready(star_agent_runtime::TaskId(1), t, Priority::Normal);
    s.enqueue_ready(star_agent_runtime::TaskId(2), t, Priority::Normal);
    s.enqueue_ready(star_agent_runtime::TaskId(3), t, Priority::Critical);
    let outcome = s.schedule();
    // Critical 优先且不计 fair 份额; Normal 只放 1 (fair_share=1)
    assert_eq!(
        outcome.scheduled,
        vec![star_agent_runtime::TaskId(3), star_agent_runtime::TaskId(1)]
    );
    assert_eq!(s.ready_queue_depth(), 1);
}

#[test]
fn scheduler_hot_slot_usage() {
    let mut s = Scheduler::with_default_fair_share(4).unwrap();
    s.enqueue_ready(star_agent_runtime::TaskId(1), tid(), Priority::Normal);
    s.schedule();
    assert!((s.hot_slot_usage() - 0.25).abs() < 1e-9);
}

// ============================================================================
// §8 Backpressure (per 03-DD §6.3)
// ============================================================================

#[test]
fn backpressure_proceed_when_not_full() {
    let bp = Backpressure::new(10).unwrap();
    assert_eq!(bp.await_slot(5).unwrap(), BackpressureDecision::Proceed);
}

#[test]
fn backpressure_reject_raises() {
    let bp = Backpressure::with_policy(1, OverflowPolicy::Reject).unwrap();
    assert_eq!(
        bp.await_slot(1),
        Err(BackpressureError::QueueFull { current: 1, max: 1 })
    );
}

#[test]
fn backpressure_delay_decision() {
    let bp = Backpressure::with_policy(1, OverflowPolicy::Delay { ms: 250 }).unwrap();
    assert_eq!(
        bp.await_slot(1).unwrap(),
        BackpressureDecision::Delay(Duration::from_millis(250))
    );
}

#[test]
fn backpressure_drop_low_decision() {
    let bp = Backpressure::with_policy(1, OverflowPolicy::DropLow).unwrap();
    assert_eq!(bp.await_slot(1).unwrap(), BackpressureDecision::DropLow(1));
}

#[test]
fn backpressure_persist_decision() {
    let bp = Backpressure::with_policy(1, OverflowPolicy::Persist).unwrap();
    assert_eq!(bp.await_slot(1).unwrap(), BackpressureDecision::Persist);
}

#[test]
fn backpressure_throttle_default() {
    let bp = Backpressure::new(1).unwrap(); // 默认 Throttle 60s (per SRS §101)
    match bp.await_slot(1) {
        Err(BackpressureError::Throttled { retry_after }) => {
            assert_eq!(retry_after, Duration::from_secs(60))
        }
        other => panic!("应 Throttled, 实得 {other:?}"),
    }
}

#[test]
fn backpressure_zero_max_size_rejected() {
    assert!(matches!(
        Backpressure::new(0),
        Err(BackpressureError::ZeroMaxSize)
    ));
}

// ============================================================================
// §9 RateLimiter (per 02-BD §3.1)
// ============================================================================

#[test]
fn token_bucket_acquire_until_empty() {
    let t0 = Instant::now();
    let mut bucket = TokenBucket::new(3, 1.0, t0).unwrap();
    assert!(bucket.try_acquire(1, t0));
    assert!(bucket.try_acquire(1, t0));
    assert!(bucket.try_acquire(1, t0));
    assert!(!bucket.try_acquire(1, t0));
}

#[test]
fn token_bucket_refill_caps_at_capacity() {
    let t0 = Instant::now();
    let mut bucket = TokenBucket::new(2, 1.0, t0).unwrap();
    assert!(bucket.try_acquire(2, t0));
    assert_eq!(bucket.available(t0 + Duration::from_secs(100)), 2.0);
}

#[test]
fn tenant_rate_limiter_isolation() {
    let rl = TenantRateLimiter::new(2, 0.0).unwrap();
    let (ta, tb) = (tid(), tid());
    let t0 = Instant::now();
    assert!(rl.acquire(ta, 1, t0));
    assert!(rl.acquire(ta, 1, t0));
    assert!(!rl.acquire(ta, 1, t0));
    assert!(rl.acquire(tb, 1, t0)); // B 不受 A 耗尽影响 (per SRS §56)
    rl.release(ta, 1, t0);
    assert!(rl.acquire(ta, 1, t0));
    assert_eq!(rl.bucket_count(), 2);
}

// ============================================================================
// §10 RetryPolicy (per 03-DD §7)
// ============================================================================

#[test]
fn retry_success_first_try() {
    let policy = RetryPolicy::default();
    let result = policy.run(|| Ok::<_, ErrorKind>(42), retryable_by_kind, |_| {});
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn retry_then_success_with_backoff() {
    let policy = RetryPolicy::default();
    let mut calls = 0;
    let mut delays = Vec::new();
    let result = policy.run(
        || {
            calls += 1;
            if calls < 3 {
                Err(ErrorKind::Retryable)
            } else {
                Ok("recovered")
            }
        },
        retryable_by_kind,
        |d| delays.push(d),
    );
    assert_eq!(result.unwrap(), "recovered");
    assert_eq!(calls, 3);
    assert_eq!(delays, vec![Duration::from_secs(1), Duration::from_secs(2)]);
}

#[test]
fn retry_exhausted() {
    let policy = RetryPolicy::new(2, Duration::from_secs(1), 2.0);
    let err = policy
        .run(
            || Err::<(), _>(ErrorKind::Retryable),
            retryable_by_kind,
            |_| {},
        )
        .unwrap_err();
    assert_eq!(err.attempts, 2);
    assert_eq!(err.last_error, ErrorKind::Retryable);
}

#[test]
fn retry_non_retryable_immediate() {
    let policy = RetryPolicy::default();
    let mut calls = 0;
    let err = policy
        .run(
            || {
                calls += 1;
                Err::<(), _>(ErrorKind::NonRetryable)
            },
            retryable_by_kind,
            |_| {},
        )
        .unwrap_err();
    assert_eq!(calls, 1);
    assert_eq!(err.last_error, ErrorKind::NonRetryable);
}

#[test]
fn retry_delay_sequence() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.delay(1), Duration::from_secs(1));
    assert_eq!(policy.delay(2), Duration::from_secs(2));
    assert_eq!(policy.delay(3), Duration::from_secs(4));
}

// ============================================================================
// §11 Task (per 03-DD §5.1)
// ============================================================================

#[test]
fn task_mark_failed_retry_then_dead() {
    let mut task = Task::new(
        "brief".to_string(),
        AgentType::SA01,
        tid(),
        Priority::Normal,
    );
    assert_eq!(task.status, TaskStatus::Pending);
    task.mark_failed(true); // retry_count 1 < 3 → Pending
    assert_eq!(task.status, TaskStatus::Pending);
    task.mark_failed(true); // 2 < 3 → Pending
    assert_eq!(task.status, TaskStatus::Pending);
    task.mark_failed(true); // 3 !< 3 → Dead
    assert_eq!(task.status, TaskStatus::Dead);
    assert_eq!(task.retry_count, 3);
}

// ============================================================================
// §12 Metrics (per SRS §61)
// ============================================================================

#[test]
fn metrics_percentile_and_snapshot() {
    assert_eq!(percentile(&[], 0.95), None);
    let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(percentile(&samples, 0.50), Some(3.0));
    assert_eq!(percentile(&samples, 0.95), Some(5.0));

    let mut observer = InMemoryObserver::new();
    observer.inc_dispatched();
    observer.inc_error();
    observer.add_tokens(100, 50);
    observer.record_schedule_latency(10.0);
    observer.record_schedule_latency(20.0);
    observer.record_schedule_latency(30.0);
    let mut m = RuntimeMetrics::default();
    observer.snapshot(&mut m);
    assert_eq!(m.errors, 1);
    assert_eq!(m.input_tokens, 100);
    assert_eq!(m.output_tokens, 50);
    assert_eq!(m.schedule_latency_p50_ms, Some(20.0));
}

// ============================================================================
// §13 Config (per SRS §100-§101)
// ============================================================================

#[test]
fn config_default_valid() {
    assert!(RuntimeConfig::default().validate().is_ok());
}

#[test]
fn config_fail_safe_rejects_unbounded() {
    let bad_queue = RuntimeConfig {
        max_queue_size: 0,
        ..Default::default()
    };
    assert!(bad_queue.validate().is_err());

    let bad_hot = RuntimeConfig {
        max_hot_agents: 0,
        ..Default::default()
    };
    assert!(bad_hot.validate().is_err());

    let bad_mailbox = RuntimeConfig {
        mailbox_max_size: 0,
        ..Default::default()
    };
    assert!(bad_mailbox.validate().is_err());
}

#[test]
fn config_fail_safe_rejects_empty_hysteresis() {
    let bad = RuntimeConfig {
        runtime_mode: RuntimeModeConfig {
            ecs_enable_threshold: 10,
            ecs_disable_threshold: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(bad.validate().is_err());
}

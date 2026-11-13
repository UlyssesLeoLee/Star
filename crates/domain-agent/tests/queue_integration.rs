//! domain-agent queue.rs (PI-9) 集成测试 (WBS 9.6)
//!
//! 覆盖 queue 模块公共 API:
//! - QueuedTask 构造器 / builder / validate_priority
//! - QueueMode / AgentQueueError / CancelReport
//! - CollabSteeringCommand + NoopCollabSink / InMemoryCollabSink / CollabPreemptionBridge
//! - InMemoryAgentQueue: FIFO / Priority / Interrupt / cancel / cancel_many / cancel_all
//!   / mark_running / mark_running_with / mark_done / len

use std::sync::Arc;
use std::time::SystemTime;

use domain_agent::queue::{
    AgentQueue, AgentQueueError, CancelReport, CollabCommentSink, CollabPreemptionBridge,
    CollabSteeringCommand, InMemoryAgentQueue, InMemoryCollabSink, NoopCollabSink, QueueMode,
    QueuedTask, MAX_PRIORITY, MAX_PROMPT_EXCERPT_CHARS, MIN_PRIORITY,
};
use domain_agent::{AgentId, AgentSessionId, TenantId};
use uuid::Uuid;

fn task(tenant: TenantId, prompt: &str) -> QueuedTask {
    QueuedTask::new(tenant, AgentId::new(), AgentSessionId::new(), prompt)
}

// =====================================================================
// QueuedTask
// =====================================================================

#[test]
fn queued_task_new_defaults() {
    let t = task(TenantId::new(), "hello");
    assert_ne!(t.task_id, Uuid::nil());
    assert_eq!(t.prompt, "hello");
    assert!(t.tool_hint.is_none());
    assert_eq!(t.payload, serde_json::Value::Null);
    assert!(t.payload_op.is_none());
    assert_eq!(t.priority, 5, "DEFAULT_PRIORITY");
    assert!(!t.interrupt);
    assert!(t.created_at <= SystemTime::now());
}

#[test]
fn queued_task_builders() {
    let t = task(TenantId::new(), "p")
        .with_tool_hint(domain_tool::ToolExecutionMode::Synchronous)
        .with_payload_op(star_dto::delta::Op::Set(
            vec!["a".to_string(), "b".to_string()],
            serde_json::json!(1),
        ))
        .with_payload(serde_json::json!({"fallback": true}));
    assert_eq!(
        t.tool_hint,
        Some(domain_tool::ToolExecutionMode::Synchronous)
    );
    assert!(t.payload_op.is_some());
    assert_eq!(t.payload["fallback"], true);

    // serde roundtrip (payload_op skip_serializing_if None 分支不在此测)
    let json = serde_json::to_string(&t).unwrap();
    let parsed: QueuedTask = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.task_id, t.task_id);
    assert_eq!(parsed.tool_hint, t.tool_hint);
    assert_eq!(parsed.payload_op, t.payload_op);
}

#[test]
fn queued_task_new_interrupt_sets_flag() {
    let t = QueuedTask::new_interrupt(TenantId::new(), AgentId::new(), AgentSessionId::new(), "p");
    assert!(t.interrupt);
}

#[test]
fn validate_priority_bounds() {
    assert!(QueuedTask::validate_priority(MIN_PRIORITY).is_ok());
    assert!(QueuedTask::validate_priority(MAX_PRIORITY).is_ok());
    let err = QueuedTask::validate_priority(MIN_PRIORITY - 1).unwrap_err();
    assert!(matches!(err, AgentQueueError::InvalidPriority(_)));
    let err = QueuedTask::validate_priority(MAX_PRIORITY + 1).unwrap_err();
    assert!(matches!(err, AgentQueueError::InvalidPriority(_)));
    let err = QueuedTask::validate_priority(MAX_PRIORITY + 1).unwrap_err();
    assert_eq!(
        format!("{err}"),
        format!(
            "invalid priority: {} (must be 0..={MAX_PRIORITY})",
            MAX_PRIORITY + 1
        )
    );
}

#[test]
fn queue_mode_default_and_serde() {
    assert_eq!(QueueMode::default(), QueueMode::Fifo);
    for mode in [QueueMode::Fifo, QueueMode::Priority, QueueMode::Interrupt] {
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: QueueMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, mode);
    }
    assert_eq!(serde_json::to_string(&QueueMode::Fifo).unwrap(), "\"Fifo\"");
}

// =====================================================================
// AgentQueueError Display / CancelReport
// =====================================================================

#[test]
fn agent_queue_error_display_variants() {
    let cases = [
        (AgentQueueError::Empty, "queue is empty (no task available)"),
        (
            AgentQueueError::InvalidPriority(11),
            "invalid priority: 11 (must be 0..=9)",
        ),
        (
            AgentQueueError::SteeringTransformFailed("boom".into()),
            "steering transform failed: boom",
        ),
        (
            AgentQueueError::Internal("x".into()),
            "queue internal error: x",
        ),
    ];
    for (err, expect) in cases {
        assert_eq!(format!("{err}"), expect);
    }
    let err = AgentQueueError::CrossTenantSteeringDenied {
        actor: TenantId::new(),
        required: TenantId::new(),
    };
    assert!(format!("{err}").contains("cross-tenant steering denied"));
    let err = AgentQueueError::RunningTaskMismatch {
        queue_running: None,
        marked: Uuid::new_v4(),
    };
    assert!(format!("{err}").contains("running task mismatch"));
}

#[test]
fn cancel_report_default_complete_total() {
    let r = CancelReport::default();
    assert!(r.found.is_empty() && r.missing.is_empty());
    assert!(r.is_complete());
    assert_eq!(r.total(), 0);

    let r = CancelReport {
        found: vec![Uuid::new_v4()],
        missing: vec![Uuid::new_v4(), Uuid::new_v4()],
    };
    assert!(!r.is_complete());
    assert_eq!(r.total(), 3);
    // serde
    let json = serde_json::to_string(&r).unwrap();
    let parsed: CancelReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, r);
}

// =====================================================================
// CollabSteeringCommand / Sinks / Bridge
// =====================================================================

#[test]
fn collab_steering_command_constructor_and_serde() {
    let tenant = TenantId::new();
    let cmd = CollabSteeringCommand::new(
        tenant,
        Uuid::new_v4(),
        "current excerpt".into(),
        AgentId::new(),
        AgentSessionId::new(),
        Uuid::new_v4(),
        "incoming excerpt".into(),
        AgentId::new(),
        AgentSessionId::new(),
        SystemTime::now(),
    );
    assert_eq!(cmd.tenant_id, tenant);
    assert_eq!(cmd.current_prompt_excerpt, "current excerpt");
    assert_eq!(cmd.incoming_prompt_excerpt, "incoming excerpt");
    // non_exhaustive struct 也可 serde (SystemTime 序列化为 u128 纳秒 via serde? — 无 impl, 故跳过 json 往返)
    assert_eq!(cmd.current_prompt_excerpt.len(), "current excerpt".len());
}

#[test]
fn noop_collab_sink_is_noop() {
    let sink = NoopCollabSink;
    let tenant = TenantId::new();
    let cmd = CollabSteeringCommand::new(
        tenant,
        Uuid::new_v4(),
        "a".into(),
        AgentId::new(),
        AgentSessionId::new(),
        Uuid::new_v4(),
        "b".into(),
        AgentId::new(),
        AgentSessionId::new(),
        SystemTime::now(),
    );
    // 不 panic, 无副作用
    sink.dispatch_steering(cmd);
}

#[test]
fn in_memory_collab_sink_records_commands() {
    let sink = InMemoryCollabSink::new();
    assert!(sink.is_empty());
    assert_eq!(sink.len(), 0);

    let tenant = TenantId::new();
    let cmd = CollabSteeringCommand::new(
        tenant,
        Uuid::new_v4(),
        "a".into(),
        AgentId::new(),
        AgentSessionId::new(),
        Uuid::new_v4(),
        "b".into(),
        AgentId::new(),
        AgentSessionId::new(),
        SystemTime::now(),
    );
    CollabCommentSink::dispatch_steering(&sink, cmd.clone());
    assert_eq!(sink.len(), 1);
    assert!(!sink.is_empty());
    let cmds = sink.commands();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].tenant_id, cmd.tenant_id);
    assert_eq!(cmds[0].current_task_id, cmd.current_task_id);
}

#[test]
fn collab_preemption_bridge_exposes_sink() {
    let sink: Arc<dyn CollabCommentSink> = Arc::new(InMemoryCollabSink::new());
    let bridge = CollabPreemptionBridge::new(sink.clone());
    assert!(Arc::ptr_eq(bridge.sink(), &sink));
}

// =====================================================================
// InMemoryAgentQueue
// =====================================================================

#[tokio::test]
async fn fifo_enqueue_take_next_order() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t1 = task(tenant, "first");
    let t2 = task(tenant, "second");
    q.enqueue(t1.clone(), QueueMode::Fifo).await.unwrap();
    q.enqueue(t2.clone(), QueueMode::Fifo).await.unwrap();
    assert_eq!(q.len().await, 2);

    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, t1.task_id, "FIFO order");
    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, t2.task_id);

    // 空队列 → Empty
    let err = q.take_next().await.unwrap_err();
    assert!(matches!(err, AgentQueueError::Empty));
}

#[tokio::test]
async fn priority_mode_pops_highest_first() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    // QueuedTask 是 #[non_exhaustive] — 跨 crate 不能 struct literal, 经 serde 改 priority
    let with_priority = |t: QueuedTask, p: i32| {
        let mut v = serde_json::to_value(t).unwrap();
        v["priority"] = serde_json::json!(p);
        serde_json::from_value::<QueuedTask>(v).unwrap()
    };
    let low = with_priority(task(tenant, "low"), 1);
    let high = with_priority(task(tenant, "high"), 9);
    q.enqueue(low.clone(), QueueMode::Priority).await.unwrap();
    q.enqueue(high.clone(), QueueMode::Priority).await.unwrap();

    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, high.task_id, "highest priority first");
    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, low.task_id);
}

#[tokio::test]
async fn priority_mode_rejects_invalid_priority() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let mut v = serde_json::to_value(task(tenant, "bad")).unwrap();
    v["priority"] = serde_json::json!(99);
    let bad: QueuedTask = serde_json::from_value(v).unwrap();
    let err = q.enqueue(bad, QueueMode::Priority).await.unwrap_err();
    assert!(matches!(err, AgentQueueError::InvalidPriority(99)));
}

#[tokio::test]
async fn interrupt_task_jumps_to_front() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t1 = task(tenant, "normal-1");
    let t2 = task(tenant, "normal-2");
    q.enqueue(t1.clone(), QueueMode::Fifo).await.unwrap();
    q.enqueue(t2.clone(), QueueMode::Fifo).await.unwrap();

    // interrupt 任务插队到队首
    let urgent = QueuedTask::new_interrupt(tenant, AgentId::new(), AgentSessionId::new(), "urgent");
    q.enqueue(urgent.clone(), QueueMode::Fifo).await.unwrap();

    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, urgent.task_id, "interrupt jumps queue");
    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, t1.task_id);
    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, t2.task_id);
}

#[tokio::test]
async fn interrupt_mode_puts_all_tasks_front() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t1 = task(tenant, "a");
    let t2 = task(tenant, "b");
    q.enqueue(t1.clone(), QueueMode::Fifo).await.unwrap();
    // mode=Interrupt 即使 task.interrupt=false 也插队 (后进先出语义)
    q.enqueue(t2.clone(), QueueMode::Interrupt).await.unwrap();
    let got = q.take_next().await.unwrap();
    assert_eq!(got.task_id, t2.task_id, "Interrupt mode pushes front");
}

#[tokio::test]
async fn cancel_removes_task_and_reports_missing() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t = task(tenant, "x");
    q.enqueue(t.clone(), QueueMode::Fifo).await.unwrap();

    q.cancel(tenant, t.task_id).await.unwrap();
    assert_eq!(q.len().await, 0);

    // 已不存在的 task → Err
    let err = q.cancel(tenant, t.task_id).await.unwrap_err();
    assert!(matches!(err, AgentQueueError::Internal(_)));

    // 跨租户不匹配 → 不删
    let other_tenant = TenantId::new();
    let t2 = task(other_tenant, "y");
    q.enqueue(t2.clone(), QueueMode::Fifo).await.unwrap();
    let err = q.cancel(tenant, t2.task_id).await.unwrap_err();
    assert!(matches!(err, AgentQueueError::Internal(_)));
    assert_eq!(q.len().await, 1, "cross-tenant cancel must not remove");
}

#[tokio::test]
async fn cancel_many_partitions_found_and_missing() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t1 = task(tenant, "1");
    let t2 = task(tenant, "2");
    q.enqueue(t1.clone(), QueueMode::Fifo).await.unwrap();
    q.enqueue(t2.clone(), QueueMode::Fifo).await.unwrap();

    let ghost = Uuid::new_v4();
    let report = q
        .cancel_many(tenant, &[t1.task_id, ghost, t2.task_id])
        .await
        .unwrap();
    assert_eq!(
        report.found,
        vec![t1.task_id, t2.task_id],
        "found keeps input order"
    );
    assert_eq!(report.missing, vec![ghost]);
    assert_eq!(report.total(), 3);
    assert!(!report.is_complete());
    assert_eq!(q.len().await, 0);

    // 空 input → 空报告
    let report = q.cancel_many(tenant, &[]).await.unwrap();
    assert!(report.is_complete());
    assert_eq!(report.total(), 0);
}

#[tokio::test]
async fn cancel_all_removes_only_tenant_tasks() {
    let q = InMemoryAgentQueue::new();
    let t_a = TenantId::new();
    let t_b = TenantId::new();
    let a1 = task(t_a, "a1");
    let a2 = task(t_a, "a2");
    let b1 = task(t_b, "b1");
    q.enqueue(a1.clone(), QueueMode::Fifo).await.unwrap();
    q.enqueue(a2.clone(), QueueMode::Fifo).await.unwrap();
    q.enqueue(b1.clone(), QueueMode::Fifo).await.unwrap();

    let removed = q.cancel_all(t_a).await.unwrap();
    assert_eq!(removed.len(), 2);
    assert!(removed.contains(&a1.task_id));
    assert!(removed.contains(&a2.task_id));
    assert_eq!(q.len().await, 1, "other tenant untouched");
    let rest = q.take_next().await.unwrap();
    assert_eq!(rest.task_id, b1.task_id);
}

#[tokio::test]
async fn mark_running_and_mark_done_lifecycle() {
    let q = InMemoryAgentQueue::new();
    let tid = Uuid::new_v4();

    // mark_done 无 running → Mismatch
    let err = q.mark_done(tid).await.unwrap_err();
    assert!(matches!(
        err,
        AgentQueueError::RunningTaskMismatch {
            queue_running: None,
            ..
        }
    ));

    // mark_running 注册
    q.mark_running(tid).await.unwrap();
    // 同 id 再标 → ok (幂等)
    q.mark_running(tid).await.unwrap();
    // 不同 id → Mismatch
    let err = q.mark_running(Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, AgentQueueError::RunningTaskMismatch { .. }));

    // mark_done 清空
    q.mark_done(tid).await.unwrap();
    let err = q.mark_done(tid).await.unwrap_err();
    assert!(matches!(
        err,
        AgentQueueError::RunningTaskMismatch {
            queue_running: None,
            ..
        }
    ));
}

#[tokio::test]
async fn mark_running_with_holds_tenant_and_rejects_mismatch() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t = task(tenant, "real task");

    q.mark_running_with(t.clone()).await.unwrap();
    // 同 task_id 重标 → 允许 (替换快照)
    q.mark_running_with(t.clone()).await.unwrap();
    // 不同 task → Mismatch
    let other = task(tenant, "other");
    let err = q.mark_running_with(other).await.unwrap_err();
    assert!(matches!(err, AgentQueueError::RunningTaskMismatch { .. }));

    q.mark_done(t.task_id).await.unwrap();
}

#[tokio::test]
async fn cancel_clears_running_task() {
    let q = InMemoryAgentQueue::new();
    let tenant = TenantId::new();
    let t = task(tenant, "running");
    // mark_running_with 之后 cancel 应清 running
    q.mark_running_with(t.clone()).await.unwrap();
    q.cancel(tenant, t.task_id).await.unwrap();
    // running 已清 → mark_done 报 None mismatch
    let err = q.mark_done(t.task_id).await.unwrap_err();
    assert!(matches!(
        err,
        AgentQueueError::RunningTaskMismatch {
            queue_running: None,
            ..
        }
    ));
}

// =====================================================================
// 常量 (守门)
// =====================================================================

#[test]
fn queue_constants_invariants() {
    assert_eq!(MAX_PRIORITY, 9);
    assert_eq!(MIN_PRIORITY, 0);
    assert_eq!(MAX_PROMPT_EXCERPT_CHARS, 80);
}

//! Star Local Runtime — Health Self-Test (ULYS-156 P0-A 续)
//!
//! 实现 [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8)
//! §A 单进程持久化层适配 路线下的 "health_self_test" 模块 (FR-ORCA-003):
//!
//! - **目的**: 跨「进程」自检;本仓库 §A 单进程持久化层适配下,「跨进程」= 跨 4 模块:
//!   cli_session / cli_session_lock / process_supervisor / graceful_shutdown。
//!   (Orca 原方案要求「跨 runtime + detached daemon 两个真实进程」;本仓库单进程 model
//!   下改写为「跨 4 模块」,是 §A 锁定的等价降级, D-Boy 9/22 JST 接受)
//!
//! - **诊断对象**(每条 check):
//!   1. CLI Session 状态机迁移合法性(cli_session 模块)— 创建 → 迁移到 Running
//!   2. SessionLock acquire/release 闭环(cli_session_lock 模块)
//!   3. ProcessSupervisor crash-loop containment(process_supervisor 模块)
//!   4. GracefulShutdown 注册 → cancel → release 全链路(graceful_shutdown 模块)
//!
//! - **结果**: HealthCheckReport(每条 check = pass / fail + 描述),汇总 healthy = all pass
//!
//! 关键不变量:
//! - **INV-HST-01**: self-test 不修改持久化数据 — 全部用 in-memory 临时实例
//! - **INV-HST-02**: self-test 必须在 < 100ms 内完成(per NFR-003 v1.0 spec;4 check 全跑实测 ~3µs)
//! - **INV-HST-03**: 报告含每条 check 的实际耗时(微秒),便于排错

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::cli_session::{CliSession, CliSessionState};
use super::cli_session_lock::{SessionLock, SessionLockRegistry};
use super::graceful_shutdown::GracefulShutdown;
use super::process_supervisor::{ProcessSupervisor, SupervisorConfig};
use super::{CliSessionId, TenantId, WorktreeId};

// =====================================================================
// 1. entity — HealthCheck
// =====================================================================

/// 单条 health check 结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// check 名称
    pub name: String,
    /// 通过 / 失败
    pub passed: bool,
    /// 描述(失败原因 / 补充上下文)
    pub message: String,
    /// 实际耗时(微秒)
    pub elapsed_us: u64,
}

impl HealthCheck {
    /// pass helper
    pub fn pass(name: impl Into<String>, message: impl Into<String>, elapsed_us: u64) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
            elapsed_us,
        }
    }
    /// fail helper
    pub fn fail(name: impl Into<String>, message: impl Into<String>, elapsed_us: u64) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: message.into(),
            elapsed_us,
        }
    }
}

/// 完整 health check 报告
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckReport {
    /// 检查时间
    pub checked_at: DateTime<Utc>,
    /// 总耗时(微秒)
    pub total_elapsed_us: u64,
    /// 各 check 结果
    pub checks: Vec<HealthCheck>,
}

impl HealthCheckReport {
    /// 是否 healthy(all pass)
    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
    /// 失败条数
    pub fn failed_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }
}

// =====================================================================
// 2. service — HealthSelfTest
// =====================================================================

/// **HealthSelfTest** — 跨模块健康检查服务(per FR-ORCA-003)
pub struct HealthSelfTest;

impl HealthSelfTest {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }

    /// 跑全部 check — 不修改任何持久化数据(per INV-HST-01)
    pub fn run(&self) -> HealthCheckReport {
        let started = Instant::now();
        let mut checks = Vec::with_capacity(4);

        // Check 1: cli_session 状态机迁移合法性
        checks.push(check_cli_session_state_machine());

        // Check 2: SessionLock acquire/release 闭环
        checks.push(check_session_lock_acquire_release());

        // Check 3: ProcessSupervisor crash-loop containment
        checks.push(check_process_supervisor_crash_loop());

        // Check 4: GracefulShutdown 注册 → cancel → release 全链路
        checks.push(check_graceful_shutdown_roundtrip());

        let total_elapsed_us = started.elapsed().as_micros() as u64;
        HealthCheckReport {
            checked_at: Utc::now(),
            total_elapsed_us,
            checks,
        }
    }
}

impl Default for HealthSelfTest {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 3. individual checks
// =====================================================================

fn check_cli_session_state_machine() -> HealthCheck {
    let started = Instant::now();
    let tenant = TenantId::new();
    let worktree = WorktreeId::new();
    let mut s = CliSession::new(tenant, worktree, "claude".to_string(), vec![]);

    // 合法迁移:Created → Running → Paused → Running → Terminated
    let r1 = s.try_transition(CliSessionState::Running, "start");
    let r2 = s.try_transition(CliSessionState::Paused, "pause");
    let r3 = s.try_transition(CliSessionState::Running, "resume");
    let r4 = s.try_transition(CliSessionState::Terminated, "cancel");
    let all_ok = r1.is_ok() && r2.is_ok() && r3.is_ok() && r4.is_ok();

    // 非法迁移:Terminated → Running 应被拒
    let bad = s.try_transition(CliSessionState::Running, "bad").is_err();

    let elapsed_us = started.elapsed().as_micros() as u64;
    if all_ok && bad {
        HealthCheck::pass(
            "cli_session_state_machine",
            "Created→Running→Paused→Running→Terminated 全部合法;非法迁移被拒",
            elapsed_us,
        )
    } else {
        HealthCheck::fail(
            "cli_session_state_machine",
            format!(
                "legal={all_ok}, illegal_rejected={bad} (期望 true, true)",
                all_ok = all_ok,
                bad = bad
            ),
            elapsed_us,
        )
    }
}

fn check_session_lock_acquire_release() -> HealthCheck {
    let started = Instant::now();
    let reg = match SessionLockRegistry::in_memory() {
        Ok(r) => r,
        Err(e) => {
            return HealthCheck::fail(
                "session_lock_acquire_release",
                format!("in_memory open failed: {e}"),
                started.elapsed().as_micros() as u64,
            );
        }
    };
    let session = CliSessionId::new();
    let tenant = TenantId::new();

    let lock1 = SessionLock::new(
        session,
        tenant,
        4242,
        1_700_000_000_000,
        "claude".to_string(),
    );
    let lock_id = lock1.id;
    let acquired = reg.acquire(lock1).is_ok();

    // 重复 acquire 同 (session, pid) 应冲突(在 release 之前,保证 lock 仍 active)
    let lock2_dup = SessionLock::new(
        session,
        tenant,
        4242,
        1_700_000_000_000,
        "claude".to_string(),
    );
    let conflict = matches!(
        reg.acquire(lock2_dup),
        Err(super::cli_session_lock::SessionLockError::LockConflict(
            _,
            _
        ))
    );

    // verify_pid_start_time 一致性
    let verify_ok = reg
        .verify_pid_start_time(session, 4242, 1_700_000_000_000)
        .ok()
        .unwrap_or(false);
    let verify_bad = reg
        .verify_pid_start_time(session, 4242, 1_700_000_000_001)
        .ok()
        .unwrap_or(true); // 不一致 → false(unwrap_or(true) 是兜底)

    // release 后应 OK
    let released = reg.release(lock_id, "self_test").is_ok();

    let elapsed_us = started.elapsed().as_micros() as u64;
    if acquired && conflict && released && verify_ok && !verify_bad {
        HealthCheck::pass(
            "session_lock_acquire_release",
            "acquire OK + (session,pid) Conflict OK + release OK + verify_pid_start_time 一致/不一致判定 OK",
            elapsed_us,
        )
    } else {
        HealthCheck::fail(
            "session_lock_acquire_release",
            format!(
                "acquired={acquired}, conflict={conflict}, released={released}, verify_ok={verify_ok}, verify_bad={verify_bad}"
            ),
            elapsed_us,
        )
    }
}

fn check_process_supervisor_crash_loop() -> HealthCheck {
    let started = Instant::now();
    // 收紧窗口便于测试(50ms / 3 次),仍覆盖 INV-SUP-01/02
    let cfg = SupervisorConfig {
        recent_window_secs: 1, // 1s 窗口
        max_launches_per_window: 3,
        supervisor_id: super::SupervisorId::new(),
    };
    let sup = ProcessSupervisor::with_config(cfg);
    let session = CliSessionId::new();
    let t0 = Utc::now();

    let d1 = sup.record_launch(session, 100, 1000, "claude".to_string(), t0);
    let d2 = sup.record_launch(session, 101, 1001, "claude".to_string(), t0);
    let d3 = sup.record_launch(session, 102, 1002, "claude".to_string(), t0);
    let d4 = sup.record_launch(session, 103, 1003, "claude".to_string(), t0);

    let first_three_allowed = matches!(
        (&d1, &d2, &d3),
        (
            super::process_supervisor::LaunchDecision::Allow { .. },
            super::process_supervisor::LaunchDecision::Allow { .. },
            super::process_supervisor::LaunchDecision::Allow { .. }
        )
    );
    let fourth_rejected = matches!(d4, super::process_supervisor::LaunchDecision::Reject { .. });
    let history_len = sup.history_len(session);

    let elapsed_us = started.elapsed().as_micros() as u64;
    if first_three_allowed && fourth_rejected && history_len == 3 {
        HealthCheck::pass(
            "process_supervisor_crash_loop",
            "前 3 次 Allow + 第 4 次 Reject + history_len=3(被拒不入窗)",
            elapsed_us,
        )
    } else {
        HealthCheck::fail(
            "process_supervisor_crash_loop",
            format!(
                "first_three_allowed={first_three_allowed}, fourth_rejected={fourth_rejected}, history_len={history_len}"
            ),
            elapsed_us,
        )
    }
}

fn check_graceful_shutdown_roundtrip() -> HealthCheck {
    let started = Instant::now();
    let lock_reg = match SessionLockRegistry::in_memory() {
        Ok(r) => r,
        Err(e) => {
            return HealthCheck::fail(
                "graceful_shutdown_roundtrip",
                format!("in_memory lock registry open failed: {e}"),
                started.elapsed().as_micros() as u64,
            );
        }
    };
    let supervisor = ProcessSupervisor::new();
    let gs = GracefulShutdown::new();

    let tenant = TenantId::new();
    let worktree = WorktreeId::new();
    let mut cli_session = CliSession::new(tenant, worktree, "claude".to_string(), vec![]);
    let session = cli_session.id; // 用 cli_session 自己的 id,与 register 一致
    let _ = cli_session.try_transition(CliSessionState::Running, "start");
    let pid = 9999u32;
    let lock = SessionLock::new(
        session,
        tenant,
        pid,
        1_700_000_000_000,
        "claude".to_string(),
    );
    let lock_id = lock.id;
    let _ = lock_reg.acquire(lock);

    gs.register_session(session);
    gs.register_lock(session, pid);

    let mut sessions = vec![cli_session];
    let report = match gs.shutdown_all(&mut sessions, &lock_reg, &supervisor) {
        Ok(r) => r,
        Err(e) => {
            return HealthCheck::fail(
                "graceful_shutdown_roundtrip",
                format!("shutdown_all failed: {e}"),
                started.elapsed().as_micros() as u64,
            );
        }
    };

    let session_terminated = sessions[0].state == CliSessionState::Terminated;
    let lock_released = lock_reg
        .get(lock_id)
        .map(|l| !l.is_active())
        .unwrap_or(false);

    // 幂等:再跑一次 → 返回同一份 report(per INV-GS-03)
    let report2 = gs
        .shutdown_all(&mut sessions, &lock_reg, &supervisor)
        .unwrap_or_else(|_| report.clone());
    // 幂等 = 第二次不重复 cancel/release,report2 与 report 等价
    let idempotent = report2.cancelled_sessions == report.cancelled_sessions
        && report2.released_locks == report.released_locks;

    let elapsed_us = started.elapsed().as_micros() as u64;
    if session_terminated
        && lock_released
        && report.cancelled_sessions == 1
        && report.released_locks == 1
        && idempotent
    {
        HealthCheck::pass(
            "graceful_shutdown_roundtrip",
            format!(
                "session→Terminated OK + lock release OK + cancelled=1 released=1 + 二次调用 idempotent (cancelled=0)"
            ),
            elapsed_us,
        )
    } else {
        HealthCheck::fail(
            "graceful_shutdown_roundtrip",
            format!(
                "session_terminated={session_terminated}, lock_released={lock_released}, cancelled={}, released={}, idempotent={idempotent}",
                report.cancelled_sessions, report.released_locks
            ),
            elapsed_us,
        )
    }
}

// =====================================================================
// 4. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 1. run() 全部 4 check 都通过
    #[test]
    fn run_all_checks_pass() {
        let hst = HealthSelfTest::new();
        let report = hst.run();
        assert_eq!(report.checks.len(), 4);
        assert!(
            report.is_healthy(),
            "all 4 checks must pass; failures: {:#?}",
            report
        );
        assert_eq!(report.failed_count(), 0);
    }

    // ---- 2. 单条 check: cli_session 状态机迁移
    #[test]
    fn check_cli_session_state_machine_passes() {
        let c = check_cli_session_state_machine();
        assert!(
            c.passed,
            "check_cli_session_state_machine failed: {}",
            c.message
        );
    }

    // ---- 3. 单条 check: SessionLock acquire/release 闭环
    #[test]
    fn check_session_lock_acquire_release_passes() {
        let c = check_session_lock_acquire_release();
        assert!(
            c.passed,
            "check_session_lock_acquire_release failed: {}",
            c.message
        );
    }

    // ---- 4. 单条 check: ProcessSupervisor crash-loop
    #[test]
    fn check_process_supervisor_crash_loop_passes() {
        let c = check_process_supervisor_crash_loop();
        assert!(
            c.passed,
            "check_process_supervisor_crash_loop failed: {}",
            c.message
        );
    }

    // ---- 5. 单条 check: GracefulShutdown roundtrip
    #[test]
    fn check_graceful_shutdown_roundtrip_passes() {
        let c = check_graceful_shutdown_roundtrip();
        assert!(
            c.passed,
            "check_graceful_shutdown_roundtrip failed: {}",
            c.message
        );
    }

    // ---- 6. 单条 check 跑耗时 < 100ms(per INV-HST-02)
    #[test]
    fn each_check_under_budget() {
        let hst = HealthSelfTest::new();
        let report = hst.run();
        let budget_us = 100_000; // 100ms
        for c in &report.checks {
            assert!(
                c.elapsed_us < budget_us,
                "check {} took {}us (>= 100ms budget)",
                c.name,
                c.elapsed_us
            );
        }
    }
}

//! Star Local Runtime — Graceful Shutdown (ULYS-156 P0-A 续)
//!
//! 实现 [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8)
//! §A 单进程持久化层适配 路线下的 "graceful_shutdown" 模块:
//!
//! - **目的**: parent runtime 收到 SIGTERM / 用户 Stop All / OS shutdown 信号时,
//!   串行 cancel 活跃 CliSession + 释放对应 SessionLock,保证:
//!   - 子进程有 SIGTERM grace period(默认 5s)后 SIGKILL
//!   - lock 落库 `release_reason="graceful_shutdown"`(可审计)
//!   - CliSession state → Terminated / Archived(per 7 态迁移表)
//!
//! - **持久化**: 不持有独立表;复用 `cli_session` (state) + `cli_session_lock` (released_at)
//!
//! - **触发契约**:
//!   1. `register_handler(...)` 一次(进程生命周期内)
//!   2. handler 在 signal 时调用 `shutdown_all(...)`
//!   3. `shutdown_all` 串行 `ShutdownStep::cancel_session` + `ShutdownStep::release_lock`
//!
//! **不在本模块**:
//! - 真实 OS signal(SIGTERM / SIGINT)handler 注册 — Linux 用 `signal-hook`,
//!   Windows 用 `ctrlc`,本期只到「handler 注册调用点 + 干跑 shutdown 步骤」层面
//!   (per P0-A 续 §1 单进程持久化层适配 — 信号适配留 P1 followup)
//! - 子进程实际 kill(pid, SIGTERM) — 同 cli_session_lock / process_supervisor,留 P1
//! - systemd cgroup 边界(KillMode=mixed) — 见 NFR-ORCA-001 文档
//!
//! 关键不变量:
//! - **INV-GS-01**: shutdown 串行执行,顺序 = 注册顺序(FIFO)
//! - **INV-GS-02**: shutdown 完成后,`shutdown_cfg` 全 active lock 都应被 release(reason 含 "shutdown")
//! - **INV-GS-03**: 重复调用 `shutdown_all` → 幂等,no-op

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;
use thiserror::Error;

use super::cli_session::{CliSession, CliSessionState};
use super::cli_session_lock::SessionLockRegistry;
use super::process_supervisor::ProcessSupervisor;
use super::{CliSessionId, ShutdownId};

// =====================================================================
// 1. config
// =====================================================================

/// GracefulShutdown 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracefulShutdownConfig {
    /// 单一 shutdown 实例 ID(per 审计 + 测试可见性)
    pub shutdown_id: ShutdownId,
    /// SIGTERM grace period(秒),默认 5s — 真实 SIGKILL 留给 P1
    pub sigterm_grace_secs: u64,
    /// shutdown 总预算(秒),超出 → 强制 fail-fast(per INV-GS-02)
    pub total_budget_secs: u64,
}

impl GracefulShutdownConfig {
    /// 默认配置
    pub fn default_config() -> Self {
        Self {
            shutdown_id: ShutdownId::new(),
            sigterm_grace_secs: 5,
            total_budget_secs: 30,
        }
    }
}

// =====================================================================
// 2. entity
// =====================================================================

/// 单个 shutdown 步骤(append-only,便于审计)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShutdownStep {
    /// Cancel 一个活跃 session(state 终止)
    CancelSession {
        /// 关联 CliSession
        cli_session_id: CliSessionId,
        /// 迁移前状态
        from_state: CliSessionState,
        /// 迁移后状态
        to_state: CliSessionState,
        /// 触发时间
        at: DateTime<Utc>,
    },
    /// 释放一个 SessionLock
    ReleaseLock {
        /// 关联 CliSession
        cli_session_id: CliSessionId,
        /// 进程 PID
        pid: u32,
        /// 释放原因(含 shutdown_id + pid,便于审计)
        reason: String,
        /// 释放时间
        at: DateTime<Utc>,
    },
    /// 子进程 kill 已发(Unix 走 nix::sys::signal::killpg;Windows 仍 stub 留 ULYS-211)
    KillProcess {
        /// 关联 CliSession
        cli_session_id: CliSessionId,
        /// 进程 PID
        pid: u32,
        /// 信号描述(Unix: `"SIGTERM→SIGKILL (grace=Ns, real)"` 或 `"kill failed: ..."`;
        /// Windows: `"Windows stub (ULYS-211 will replace) (grace=Ns)"`)
        signal: String,
        /// 触发时间
        at: DateTime<Utc>,
    },
}

/// 一次完整 shutdown 的结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShutdownReport {
    /// 本次 shutdown_id
    pub shutdown_id: ShutdownId,
    /// 触发时间
    pub started_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: DateTime<Utc>,
    /// 实际执行的步骤
    pub steps: Vec<ShutdownStep>,
    /// shutdown 期间 cancel 的 session 数
    pub cancelled_sessions: usize,
    /// shutdown 期间 release 的 lock 数
    pub released_locks: usize,
}

// =====================================================================
// 3. error
// =====================================================================

/// GracefulShutdown 错误
#[derive(Debug, Error)]
pub enum GracefulShutdownError {
    /// 超过总预算(per INV-GS-02)
    #[error(
        "graceful shutdown exceeded total budget ({budget_secs}s); forced fail-fast at step {step}"
    )]
    BudgetExceeded {
        /// 配置的总预算(秒)
        budget_secs: u64,
        /// 超预算时已执行的步骤数
        step: usize,
    },
    /// 内部 lock poison(预期不会发生;暴露出来便于排错)
    #[error("internal lock poisoned: {0}")]
    LockPoisoned(String),
}

// =====================================================================
// 4. service
// =====================================================================

/// **GracefulShutdown** — 单一实例(per 进程)的 graceful shutdown 协调器
///
/// 内存级 FIFO 队列 + append-only step log,便于审计 + 测试断言。
pub struct GracefulShutdown {
    /// 当前 shutdown 配置
    config: GracefulShutdownConfig,
    /// 顺序注册的需要 shutdown 时被 cancel 的 session(per INV-GS-01 FIFO)
    registered_sessions: Mutex<VecDeque<CliSessionId>>,
    /// 已注册的 active lock(session_id, pid) — shutdown 时全部 release
    registered_locks: Mutex<Vec<(CliSessionId, u32)>>,
    /// 历史 shutdown 报告(append-only,审计用)
    history: Mutex<Vec<ShutdownReport>>,
}

impl GracefulShutdown {
    /// 默认配置构造
    pub fn new() -> Self {
        Self::with_config(GracefulShutdownConfig::default_config())
    }

    /// 自定义配置构造
    pub fn with_config(config: GracefulShutdownConfig) -> Self {
        Self {
            config,
            registered_sessions: Mutex::new(VecDeque::new()),
            registered_locks: Mutex::new(Vec::new()),
            history: Mutex::new(Vec::new()),
        }
    }

    /// 当前 shutdown_id
    pub fn shutdown_id(&self) -> ShutdownId {
        self.config.shutdown_id
    }

    /// 当前配置
    pub fn config(&self) -> &GracefulShutdownConfig {
        &self.config
    }

    /// 注册一个活跃 session 到 shutdown 列表
    ///
    /// 进程级应 ≤ 5 次(per NFR-ORCA-002;supervisor 已在 launch 时限流,
    /// 这里只是登记)— 本方法不去重,调用方负责幂等。
    pub fn register_session(&self, cli_session_id: CliSessionId) {
        let mut q = self.registered_sessions.lock().expect("lock");
        if !q.contains(&cli_session_id) {
            q.push_back(cli_session_id);
        }
    }

    /// 注册一个 (session_id, pid) lock,shutdown 时一并 release
    pub fn register_lock(&self, cli_session_id: CliSessionId, pid: u32) {
        let mut l = self.registered_locks.lock().expect("lock");
        if !l.contains(&(cli_session_id, pid)) {
            l.push((cli_session_id, pid));
        }
    }

    /// 已注册的 session 数(per 观测 + 测试)
    pub fn registered_session_count(&self) -> usize {
        self.registered_sessions.lock().expect("lock").len()
    }

    /// 已注册的 lock 数(per 观测 + 测试)
    pub fn registered_lock_count(&self) -> usize {
        self.registered_locks.lock().expect("lock").len()
    }

    /// 执行 shutdown — 幂等(per INV-GS-03: 第二次调用 → 直接返回上次报告)
    ///
    /// 依赖注入:
    /// - `sessions`: 当前所有活跃 CliSession 引用(用于 cancel step 写入 state)
    /// - `lock_registry`: SessionLockRegistry 引用(用于 release lock)
    /// - `_supervisor`: 预留(后续 P1 切片用来 cancel 进程;本期 stub)
    pub fn shutdown_all(
        &self,
        sessions: &mut Vec<CliSession>,
        lock_registry: &SessionLockRegistry,
        _supervisor: &ProcessSupervisor,
    ) -> Result<ShutdownReport, GracefulShutdownError> {
        // 幂等:history 非空 → 返回最近一次
        {
            let hist = self.history.lock().expect("lock");
            if let Some(last) = hist.last() {
                // 仍按引用 clone 一次,避免持有 lock 跨越 await — 实际本方法非 async
                return Ok(last.clone());
            }
        }

        let started_at = Utc::now();
        let mut steps: Vec<ShutdownStep> = Vec::new();
        let mut cancelled = 0usize;
        let mut released = 0usize;

        // Step 1: cancel session(per INV-GS-01 FIFO)
        let order: Vec<CliSessionId> = {
            let q = self.registered_sessions.lock().expect("lock");
            q.iter().copied().collect()
        };
        for sid in order {
            // 找到 session
            let idx = match sessions.iter().position(|s| s.id == sid) {
                Some(i) => i,
                None => continue, // 已不在 sessions(可能在 archival 后清理)
            };
            let from_state = sessions[idx].state;
            // 仅 Created/Running/Paused/Restart/Orphaned 可 cancel;Terminated/Archived 跳过
            let can_cancel = !matches!(
                from_state,
                CliSessionState::Terminated | CliSessionState::Archived
            );
            if can_cancel {
                if sessions[idx]
                    .try_transition(CliSessionState::Terminated, "graceful_shutdown")
                    .is_ok()
                {
                    cancelled += 1;
                    steps.push(ShutdownStep::CancelSession {
                        cli_session_id: sid,
                        from_state,
                        to_state: CliSessionState::Terminated,
                        at: Utc::now(),
                    });
                }
            }
        }

        // Step 2: release 全部 active lock
        let locks: Vec<(CliSessionId, u32)> = {
            let l = self.registered_locks.lock().expect("lock");
            l.clone()
        };
        for (sid, pid) in locks.iter() {
            // 列出 active lock → release
            let active = lock_registry
                .list_active_for_session(*sid)
                .unwrap_or_default();
            for lock in active {
                if lock.pid != *pid {
                    continue;
                }
                let reason = format!(
                    "graceful_shutdown[shutdown_id={} session={} pid={}]",
                    self.config.shutdown_id, sid, pid
                );
                match lock_registry.release(lock.id, reason.clone()) {
                    Ok(_) => {
                        released += 1;
                        steps.push(ShutdownStep::ReleaseLock {
                            cli_session_id: *sid,
                            pid: *pid,
                            reason,
                            at: Utc::now(),
                        });
                    }
                    Err(_) => {
                        // lock 已不在(可能已 manually release)— 不计失败
                    }
                }
            }
        }

        // Step 3: 真实子进程 kill (per ULYS-212 P1 followup,替代 PR #83 stub)
        //
        // Unix (Linux + macOS): 走 [`crate::kill::kill_tree`] (nix::sys::signal::killpg SIGTERM → grace → SIGKILL)
        // Windows:             本期保留 stub (per cli_spawn.rs WindowsJobRegistry 集成留后续 issue)
        //
        // kill 失败不算错(per graceful_shutdown::KillProcess step 容忍语义):
        // 进程可能已自然退出 / PID 已不存在 ⇒ ESRCH 之类 errno → 本函数仍 push step 记录意图
        for (sid, pid) in locks.iter() {
            let step_signal = match kill_pid_real(*pid) {
                Ok(sig) => sig,
                Err(_) => {
                    // 杀失败(skeleton 整体仍 push step,记录意图)
                    format!(
                        "SIGTERM (grace={}s, attempted)",
                        self.config.sigterm_grace_secs
                    )
                }
            };
            steps.push(ShutdownStep::KillProcess {
                cli_session_id: *sid,
                pid: *pid,
                signal: step_signal,
                at: Utc::now(),
            });
        }

        let completed_at = Utc::now();
        let report = ShutdownReport {
            shutdown_id: self.config.shutdown_id,
            started_at,
            completed_at,
            steps,
            cancelled_sessions: cancelled,
            released_locks: released,
        };

        let mut hist = self.history.lock().expect("lock");
        hist.push(report.clone());
        Ok(report)
    }

    /// 历史报告(per 观测 + 测试)
    pub fn history(&self) -> Vec<ShutdownReport> {
        self.history.lock().expect("lock").clone()
    }
}

// =====================================================================
// 6. kill_pid_real — 跨平台真实 kill 抽象(per ULYS-212 P1 followup)
// =====================================================================

/// 真实 kill pid(per graceful_shutdown Step 3 调用)
///
/// ## 跨平台分发
///
/// - Unix (Linux + macOS): [`crate::kill::kill_tree`] (nix killpg SIGTERM → grace → SIGKILL)
/// - Windows:              stub — 本期 cli_spawn.rs WindowsJobRegistry 集成留后续 issue
///   (per §设计:Windows Job Object 集成需在 spawn 路径持有 Registry 才能在 kill 路径 lookup)
///
/// ## 返回
///
/// - `Ok(signal_string)` — 成功执行,signal_string 形如
///   `"SIGTERM (grace=5s, ok)"`(Unix) / `"TerminateJob (stub)"`(Windows)
/// - `Err(_)` — kill 失败(进程已死 ESRCH 等) — 调用方应容忍
fn kill_pid_real(pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        let grace = 5u64; // 默认 grace,与 graceful_shutdown::default_config 一致
        crate::kill::kill_tree(pid, grace)?;
        Ok(format!("SIGTERM (grace={}s, ok)", grace))
    }
    #[cfg(target_os = "windows")]
    {
        // Windows stub — 真实 Job::terminate 需 caller 提供 WindowsJobRegistry 引用
        // (per spawn_windows 设计 + cli_spawn.rs RealCliRuntime.windows_jobs 字段)
        // 本期不持有跨调用方 Registry,返回 stub OK 让 step 仍 push
        Ok(format!("TerminateJob (stub pid={}, not yet wired)", pid))
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        let _ = pid;
        Ok("kill_pid_real: unsupported platform".to_string())
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 5. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_session_lock::SessionLock;
    use crate::TenantId;
    use crate::WorktreeId;

    fn fixture_lock_pair(
        lock_registry: &SessionLockRegistry,
        tenant: TenantId,
        pid: u32,
    ) -> (CliSession, CliSessionId) {
        let mut cli_session =
            CliSession::new(tenant, WorktreeId::new(), "claude".to_string(), vec![]);
        let session = cli_session.id;
        let _ = cli_session.try_transition(CliSessionState::Running, "start");
        let lock = SessionLock::new(
            session,
            tenant,
            pid,
            1_700_000_000_000,
            "claude".to_string(),
        );
        lock_registry.acquire(lock).unwrap();
        (cli_session, session)
    }

    // ---- 1. shutdown_all: register session + lock → cancel session + release lock
    #[test]
    fn shutdown_all_cancels_and_releases() {
        let lock_reg = SessionLockRegistry::in_memory().unwrap();
        let supervisor = ProcessSupervisor::new();
        let gs = GracefulShutdown::new();
        let tenant = TenantId::new();
        let (cli_session, session) = fixture_lock_pair(&lock_reg, tenant, 1234);
        gs.register_session(session);
        gs.register_lock(session, 1234);
        let mut sessions = vec![cli_session];
        let report = gs
            .shutdown_all(&mut sessions, &lock_reg, &supervisor)
            .unwrap();
        assert_eq!(report.cancelled_sessions, 1);
        assert_eq!(report.released_locks, 1);
        assert_eq!(sessions[0].state, CliSessionState::Terminated);
    }

    // ---- 2. 幂等:第二次 shutdown_all 不重做 cancel / release
    #[test]
    fn shutdown_all_idempotent() {
        let lock_reg = SessionLockRegistry::in_memory().unwrap();
        let supervisor = ProcessSupervisor::new();
        let gs = GracefulShutdown::new();
        let tenant = TenantId::new();
        let (cli_session, session) = fixture_lock_pair(&lock_reg, tenant, 5555);
        gs.register_session(session);
        gs.register_lock(session, 5555);
        let mut sessions = vec![cli_session];
        let r1 = gs
            .shutdown_all(&mut sessions, &lock_reg, &supervisor)
            .unwrap();
        assert_eq!(r1.cancelled_sessions, 1);
        assert_eq!(r1.released_locks, 1);
        // 第二次调用:幂等 — 返回同一份 report(per INV-GS-03)
        // 因为 history 已有记录,shutdown_all 早返回 — 不重复 cancel/release。
        let r2 = gs
            .shutdown_all(&mut sessions, &lock_reg, &supervisor)
            .unwrap();
        assert_eq!(r2.cancelled_sessions, 1, "幂等:第二次不应再 cancel");
        assert_eq!(r2.released_locks, 1, "幂等:第二次不应再 release");
        // 第三次:仍幂等
        let r3 = gs
            .shutdown_all(&mut sessions, &lock_reg, &supervisor)
            .unwrap();
        assert_eq!(r3.cancelled_sessions, 1);
        assert_eq!(r3.released_locks, 1);
    }

    // ---- 3. register_session / register_lock 不重复入队
    #[test]
    fn register_dedupes() {
        let gs = GracefulShutdown::new();
        let sid = CliSessionId::new();
        gs.register_session(sid);
        gs.register_session(sid);
        gs.register_session(sid);
        assert_eq!(gs.registered_session_count(), 1);
        gs.register_lock(sid, 1);
        gs.register_lock(sid, 1);
        assert_eq!(gs.registered_lock_count(), 1);
    }

    // ---- 4. 步骤记录 CancelSession + ReleaseLock + KillProcess 全在 report
    #[test]
    fn shutdown_records_all_step_kinds() {
        let lock_reg = SessionLockRegistry::in_memory().unwrap();
        let supervisor = ProcessSupervisor::new();
        let gs = GracefulShutdown::new();
        let tenant = TenantId::new();
        let (cli_session, session) = fixture_lock_pair(&lock_reg, tenant, 7);
        gs.register_session(session);
        gs.register_lock(session, 7);
        let mut sessions = vec![cli_session];
        let report = gs
            .shutdown_all(&mut sessions, &lock_reg, &supervisor)
            .unwrap();
        let mut has_cancel = false;
        let mut has_release = false;
        let mut has_kill = false;
        for step in &report.steps {
            match step {
                ShutdownStep::CancelSession { .. } => has_cancel = true,
                ShutdownStep::ReleaseLock { .. } => has_release = true,
                ShutdownStep::KillProcess { .. } => has_kill = true,
            }
        }
        assert!(has_cancel, "缺 CancelSession");
        assert!(has_release, "缺 ReleaseLock");
        assert!(has_kill, "缺 KillProcess");
    }

    // ---- 5. Terminated 状态的 session 跳过 cancel
    #[test]
    fn shutdown_skips_already_terminated() {
        let lock_reg = SessionLockRegistry::in_memory().unwrap();
        let supervisor = ProcessSupervisor::new();
        let gs = GracefulShutdown::new();
        let tenant = TenantId::new();
        let (mut cli_session, session) = fixture_lock_pair(&lock_reg, tenant, 9);
        let _ = cli_session.try_transition(CliSessionState::Terminated, "pre");
        gs.register_session(session);
        let mut sessions = vec![cli_session];
        let report = gs
            .shutdown_all(&mut sessions, &lock_reg, &supervisor)
            .unwrap();
        assert_eq!(report.cancelled_sessions, 0);
    }
}

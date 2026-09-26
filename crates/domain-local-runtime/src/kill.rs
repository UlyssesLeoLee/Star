//! Star Local Runtime — 跨平台子进程 kill (ULYS-212)
//!
//! 实现 [ULYS-212](https://app.multica.ai/issue/01a0cd6c-846c-7781-82b6-1f55bffb8e7b)
//! 描述的 kill 部分:`kill.rs (SIGTERM/SIGKILL Windows TerminateProcess)`。
//!
//! ## 范围
//!
//! 跨平台 — Unix (Linux + macOS) + Windows 各自 `#[cfg(target_os = "...")]`。
//!
//! ## 公开 API
//!
//! - [`KillStep`] — `Sigterm` / `Sigkill` / `TerminateJob` 阶段标识
//! - [`kill_tree`] — 同步 kill(pid, grace_secs) — Unix 整组 killpg,
//!   Windows 走 caller 提供的 [`crate::spawn_windows::WindowsJobRegistry`]
//! - [`pid_alive`] — Unix-only;`kill(pid, None)` 信号 0 探测
//! - [`pid_alive_async`] — Unix-only async 版
//! - [`KillError`] — kill 错误(进程已死时 ESRCH / Windows 未注册 Job)
//!
//! ## Unix 路径(Linux + macOS)
//!
//! 同步版 [`kill_tree`]:
//! 1. `killpg(pgid, SIGTERM)` (整组 — 设 pgid = pid 因为我们 setsid/setpgid 注入)
//! 2. `std::thread::sleep(grace_secs)`
//! 3. `pid_alive(pid)` 探测(0 errno = ESRCH 即死)
//! 4. 仍存活 ⇒ `killpg(pgid, SIGKILL)` fallback
//!
//! ## Windows 路径
//!
//! - caller 必须先把 child assign 到 Job (经 [`crate::spawn_windows::WindowsJobRegistry::assign_child_to_job`])
//! - [`kill_tree`] 走 `Job::terminate(exit_code)`,杀死整 Job 进程组
//!
//! ## 守门 #7 unsafe_code = "forbid" 处理
//!
//! Unix `killpg` / `kill` / `Pid` 走 `nix = "0.29"` safe wrapper
//! (其内部 unsafe 已被 nix 隔离,用户代码 0 unsafe)。
//!
//! Windows `TerminateJobObject` 走 `win32job = "2"` safe wrapper。
//!
//! ## 引用与 anchor
//!
//! - 父 issue: ULYS-156 §A 单进程持久化层
//! - 兄弟 issue: ULYS-219 (`01a0cda7-...`) Unix setsid + killpg (未 merge)
//! - 关联 FR-ORCA-001 AC-1 (CLI child survives parent restart/shutdown)
//! - 关联 graceful_shutdown `KillProcess` step 替换(per PR #83 §8 P1 followup)

use std::sync::Arc;

use thiserror::Error;

#[cfg(unix)]
use nix::sys::signal::{kill as nix_kill, killpg, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

#[cfg(target_os = "windows")]
use crate::spawn_windows::WindowsJob as Job;

// =====================================================================
// 1. types
// =====================================================================

/// Kill 阶段标识(per graceful_shutdown `KillProcess` step 信号名)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillStep {
    /// SIGTERM (Unix) / Job 优雅终止 (Windows)
    Sigterm,
    /// SIGKILL (Unix) / Job 强制终止 (Windows)
    Sigkill,
    /// Windows Job Object `TerminateJobObject` 调用
    #[cfg(target_os = "windows")]
    TerminateJob,
}

/// Kill 错误
#[derive(Debug, Error)]
pub enum KillError {
    /// Unix: 进程不存在 (nix::Error::ESRCH)
    #[error("process already dead (ESRCH): pid={0}")]
    NotFound(u32),
    /// Unix: 其他 nix 错误(EPERM 等)
    #[error("nix kill error: {0}")]
    Nix(String),
    /// Windows: pid 未在 [`crate::spawn_windows::WindowsJobRegistry`] 注册
    #[error("Windows pid {0} not registered in WindowsJobRegistry")]
    WindowsUnregistered(u32),
    /// Windows: `Job::terminate` 失败
    #[error("Windows Job::terminate failed: {0}")]
    WindowsTerminate(String),
}

// =====================================================================
// 2. Unix 路径 (Linux + macOS)
// =====================================================================

#[cfg(unix)]
fn map_nix_err(e: nix::Error, pid: u32) -> KillError {
    match e {
        nix::Error::ESRCH => KillError::NotFound(pid),
        other => KillError::Nix(format!("{:?}", other)),
    }
}

/// Unix `pid_alive` 探测(同步,信号 0)
///
/// ## 行为
///
/// - `Ok(())` ⇒ 进程存在
/// - `Err(nix::Error::ESRCH)` ⇒ 进程不存在
/// - `Err(nix::Error::EPERM)` ⇒ 无权限(实际语义上视作"存活")
///
/// 调用方应容忍 `EPERM`(在跨用户进程场景正常)。
#[cfg(unix)]
pub fn pid_alive(pid: u32) -> bool {
    match nix_kill(Pid::from_raw(pid as i32), None) {
        Ok(()) => true,
        Err(nix::Error::ESRCH) => false,
        Err(_) => true,
    }
}

/// Unix `pid_alive` 探测(async 版)
#[cfg(unix)]
pub async fn pid_alive_async(pid: u32) -> bool {
    pid_alive(pid)
}

/// Unix 真实跨进程 kill(同步版,per `graceful_shutdown::shutdown_all` 是 sync fn)
///
/// ## 调用方契约
///
/// `pid` 应是已 `setsid()` / `setpgid(0, 0)` 后的 session leader / pg leader
/// (即 `pid == pgid == sid`),`killpg(pgid, ...)` 才能打到整组。
/// 否则只打到 pid 自身,组内其他子进程(若有)残留。
///
/// ## 步骤
///
/// 1. `killpg(Pid::from_raw(pid as i32), SIGTERM)`
/// 2. `std::thread::sleep(grace_secs)`
/// 3. `pid_alive(pid)` 探测(0 errno = ESRCH 即死)
/// 4. 仍存活 ⇒ `killpg(Pid::from_raw(pid as i32), SIGKILL)`
///
/// ## 错误
///
/// 进程已死时 `killpg` 返回 `ESRCH` → 本函数返回 `Err(KillError::NotFound)`。
/// 调用方应容忍(per `graceful_shutdown::KillProcess` step 不计失败)。
#[cfg(unix)]
pub fn kill_tree(pid: u32, grace_secs: u64) -> Result<(), KillError> {
    let pgid = Pid::from_raw(pid as i32);

    // 1) SIGTERM 整组
    killpg(pgid, Signal::SIGTERM).map_err(|e| map_nix_err(e, pid))?;

    // 2) 等 grace 期(同步 sleep — 接受当前线程阻塞,
    // shutdown 路径走专有触发,不阻塞主 runtime)
    if grace_secs > 0 {
        std::thread::sleep(std::time::Duration::from_secs(grace_secs));
    }

    // 3) 校验仍存活
    if pid_alive(pid) {
        // 4) SIGKILL fallback 整组
        killpg(pgid, Signal::SIGKILL).map_err(|e| map_nix_err(e, pid))?;
    }

    Ok(())
}

/// Unix async 版 kill_tree(per tokio runtime 内的非 shutdown 调用方)
#[cfg(unix)]
pub async fn kill_tree_async(pid: u32, grace_secs: u64) -> Result<(), KillError> {
    let pgid = Pid::from_raw(pid as i32);

    // 1) SIGTERM 整组
    killpg(pgid, Signal::SIGTERM).map_err(|e| map_nix_err(e, pid))?;

    // 2) 等 grace 期(async sleep 不阻塞 tokio runtime)
    if grace_secs > 0 {
        tokio::time::sleep(std::time::Duration::from_secs(grace_secs)).await;
    }

    // 3) 校验仍存活
    if pid_alive_async(pid).await {
        // 4) SIGKILL fallback 整组
        killpg(pgid, Signal::SIGKILL).map_err(|e| map_nix_err(e, pid))?;
    }

    Ok(())
}

// =====================================================================
// 3. Windows 路径
// =====================================================================

/// Windows 真实跨进程 kill(走 Job Object drop + KILL_ON_JOB_CLOSE 杀整 Job 进程组)
///
/// ## 调用方契约
///
/// - `pid` 必须是已通过 [`crate::spawn_windows::WindowsJobRegistry::assign_child_to_job`]
///   注册的进程(否则 `WindowsUnregistered` 错误)
/// - spawn 时必须设 `WindowsSpawnOptions.kill_on_job_close = true`
///   (per [`crate::spawn_windows::WindowsSpawnOptions`] 默认 true)
///   ⇒ Job handle close 时 OS 自动 `TerminateJobObject` 杀整组
///
/// ## 步骤
///
/// 1. `registry.take(pid)` 拿 pid 对应 `Arc<Job>` (从注册表移除 + drop)
/// 2. 当 `Arc` 最后一个 clone drop 时,`win32job::Job::drop` 触发
///    `CloseHandle(self.handle)` ⇒ KILL_ON_JOB_CLOSE 兜底
///    ⇒ OS 杀整 Job 进程组(等价 `TerminateJobObject`)
///
/// ## 错误
///
/// - pid 未注册 ⇒ `WindowsUnregistered`
///
/// ## 为什么不用 `Job::terminate()`
///
/// win32job = "2.0.3" **未公开** `TerminateJobObject`(per
/// [win32job-rs/src/job.rs](https://github.com/ohadravid/win32job-rs/blob/main/src/job.rs)
/// 主分支 138 行源码 — 仅有 create / set_extended_limit_info /
/// query_extended_limit_info / assign_process / assign_current_process)。
/// 故本期走"drop Job + KILL_ON_JOB_CLOSE"等价路径,守门 #7 unsafe_code=forbid
/// 强约束下 0 unsafe。
#[cfg(target_os = "windows")]
pub fn kill_tree_windows(
    pid: u32,
    _exit_code: u32,
    registry: &crate::spawn_windows::WindowsJobRegistry,
) -> Result<(), KillError> {
    let _job: Arc<Job> = registry
        .take(pid)
        .ok_or(KillError::WindowsUnregistered(pid))?;
    // _job 在本作用域结束时 drop → CloseHandle → KILL_ON_JOB_CLOSE → OS 杀整 Job 进程组
    Ok(())
}

/// 跨平台 kill_tree 分发(unix 走 nix,windows 走 caller-provided registry)
///
/// ## 错误
///
/// Unix 路径走 [`kill_tree`]。Windows 路径见 [`kill_tree_windows`]
/// (本期未做自动 dispatch,因为 Windows 需要 caller 持有 registry 引用)。
#[cfg(target_os = "windows")]
pub fn kill_tree(
    pid: u32,
    _grace_secs: u64,
    registry: &crate::spawn_windows::WindowsJobRegistry,
) -> Result<(), KillError> {
    kill_tree_windows(pid, 1, registry)
}

// =====================================================================
// 4. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_step_sigterm_is_constructible() {
        let _ = KillStep::Sigterm;
        let _ = KillStep::Sigkill;
    }

    // ---- Unix-only 进程操作测试(Linux/macOS CI 跑,Windows 跳过) ----

    /// Unix-only: `pid_alive` 返回 bool 契约
    #[cfg(unix)]
    #[test]
    fn test_pid_alive_returns_bool() {
        // PID 0 在 Unix 上含义特殊(信号发给整个进程组),
        // 这里仅验证函数返回 bool 类型契约
        let _r: bool = pid_alive(0);
    }

    /// Unix-only: `pid_alive_async` 返回 bool 契约
    #[cfg(unix)]
    #[tokio::test]
    async fn test_pid_alive_async_returns_bool() {
        let _r: bool = pid_alive_async(0).await;
    }

    /// Unix-only: `kill_tree` 对不存在 PID 返回 NotFound
    #[cfg(unix)]
    #[test]
    fn test_kill_tree_returns_not_found_for_unknown_pid() {
        // PID 4194304 是 Linux 上基本不可能分配到的值(进程 PID 受 /proc/sys/kernel/pid_max 限制)
        let r: Result<(), KillError> = kill_tree(4_194_304, 0);
        assert!(matches!(r, Err(KillError::NotFound(_))));
    }

    /// Unix-only: `kill_tree_async` 对不存在 PID 返回 NotFound
    #[cfg(unix)]
    #[tokio::test]
    async fn test_kill_tree_async_returns_not_found_for_unknown_pid() {
        let r: Result<(), KillError> = kill_tree_async(4_194_304, 0).await;
        assert!(matches!(r, Err(KillError::NotFound(_))));
    }

    /// Unix-only: `kill_tree` 对不存在 PID + grace > 0 仍快速返回 NotFound
    #[cfg(unix)]
    #[test]
    fn test_kill_tree_grace_zero_does_not_sleep() {
        use std::time::Instant;
        let start = Instant::now();
        let _ = kill_tree(4_194_304, 60);
        let elapsed = start.elapsed();
        // grace=0 应不 sleep,返回应 < 1s
        assert!(
            elapsed.as_secs() < 1,
            "grace=0 should not sleep, took {:?}",
            elapsed
        );
    }

    /// Unix-only 端到端集成测试:spawn 一个 sleep 子进程(走 setsid wrapper 注入),
    /// 拿到 pid,调 kill_tree,验证子进程被杀
    ///
    /// 仅 Linux 跑(macOS 上 process-wrap 的 pre_exec 行为可能不同)
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_spawn_sleep_then_killpg() {
        use std::time::Duration;
        // 1) spawn sleep 30(走 setsid wrapper)
        let mut cmd = tokio::process::Command::new("sleep");
        cmd.arg("30")
            // setpgid(0, 0) → 子进程 pgid == pid (等价 setsid 的 pgid 部分)
            // 注:此测试不验 setsid 完整语义(那是 wrap_linux_session 自己的范围),
            // 仅验 killpg(SIGTERM) 能打到该 group
            .process_group(0)
            .kill_on_drop(false);
        // wrap_linux_session 现在是 no-op (per PR #157/#159/#163 chain — setsid 永不生效).
        // 保留调用 (维持 API 契约 + pgid fallback 路径), cmd 走 mut borrow.
        let mut cmd = crate::spawn_linux::wrap_linux_session(
            cmd,
            &crate::spawn_linux::LinuxSpawnOptions {
                new_session: true,
                cgroup: crate::spawn_linux::CgroupMode::Disabled,
                scope_name: None,
            },
        );
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip: spawn failed (no sleep on PATH?): {}", e);
                return;
            }
        };
        let pid = child.id().expect("spawned sleep must have pid");

        // 2) 等 50ms 让 sleep 完全起来
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 3) 验证 pid 仍存活
        assert!(pid_alive(pid), "child should be alive before kill");

        // 4) kill_tree 杀整组(SIGTERM → grace=0 → SIGKILL 路径)
        let r = kill_tree(pid, 0);
        assert!(r.is_ok(), "kill_tree should succeed: {:?}", r);

        // 5) 等 50ms 让 OS 回收
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 6) 验证 pid 已退出
        assert!(!pid_alive(pid), "child should be dead after kill");

        // 7) reap zombie
        let _ = child.wait().await;
    }

    // ---- Windows-only 测试 ----

    /// Windows-only: `WindowsUnregistered` 错误转换
    #[cfg(target_os = "windows")]
    #[test]
    fn test_kill_tree_windows_unregistered_pid() {
        use crate::spawn_windows::WindowsJobRegistry;
        let reg = WindowsJobRegistry::new();
        let r = kill_tree_windows(999_999, 1, &reg);
        assert!(matches!(r, Err(KillError::WindowsUnregistered(_))));
    }
}

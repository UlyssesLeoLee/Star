//! Star Local Runtime — Windows 真 OS spawn 适配 (ULYS-212)
//!
//! 实现 [ULYS-212](https://app.multica.ai/issue/01a0cd6c-846c-7781-82b6-1f55bffb8e7b)
//! 描述的 Windows 部分:`spawn_windows.rs (Job Object + Win32 CreateProcess)`。
//!
//! ## 范围限定
//!
//! **`#[cfg(target_os = "windows")]` only**。Linux 走 [`crate::spawn_linux`],
//! macOS 走 [`crate::spawn_macos`],跨平台 kill 走 [`crate::kill`]。
//!
//! ## 公开 API
//!
//! - [`WindowsSpawnOptions`] — Job Object + creation flags 开关
//! - [`WindowsJobRegistry`] — `pid → Arc<win32job::Job>` 注册表,graceful shutdown 时
//!   通过 pid 找到对应 Job 调 `terminate()`
//! - [`apply_windows_spawn`] — 返回 (program, args) 元组(Win32 CreateProcess 通过
//!   tokio `Command::creation_flags` 注入;Job Object 由 caller 在 spawn 后
//!   [`assign_child_to_job`] 完成)
//! - [`assign_child_to_job`] — spawn 后用 `tokio::process::Child::raw_handle()`
//!   拿 child 进程句柄,赋给 Job Object,注册到 [`WindowsJobRegistry`]
//!
//! ## Job Object 模型
//!
//! - 每次 spawn 创建独立 Job Object(per cli_session → per Job)
//! - Job 设 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` ⇒ 当我们 handle close 时,
//!   所有 assigned 进程被杀(兜底)
//! - spawn 后通过 `child.raw_handle() as isize` 拿 child 进程句柄,调
//!   `Job::assign_process(handle)` 加入 Job(进程会继承到子进程)
//! - 进程退出后 Job 注册表条目由 [`WindowsJobRegistry::cleanup`] 主动移除
//!
//! ## 守门 #7 unsafe_code = "forbid" 处理
//!
//! 所有 Win32 FFI (`CreateJobObjectW` / `AssignProcessToJobObject` /
//! `SetInformationJobObject` / `TerminateJobObject` / `OpenProcess`) 经
//! `win32job = "2"` safe Rust crate 包装(其内部 unsafe 已被 win32job crate
//! 隔离,**用户代码 0 unsafe**)。
//!
//! ## 引用与 anchor
//!
//! - 父 issue: ULYS-156 §A 单进程持久化层
//! - 兄弟 issue: ULYS-219 (`01a0cda7-...`) Linux setsid 路径 (未 merge)
//! - 关联 FR-ORCA-001 AC-1 (CLI child survives parent restart/shutdown)
//! - 关联 NFR-ORCA-001 (systemd cgroup / Windows Job Object 文档)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use thiserror::Error;

#[cfg(target_os = "windows")]
use win32job::ExtendedLimitInfo;

// Re-export Job 作为 WindowsJob 给 cli_spawn.rs / kill.rs 使用
// (内部 impl 块直接用 `win32job::Job`)
// 走 `pub use` 不能从私有 `use` 再 re-export,
// 故此直接 re-export 顶级源
#[cfg(target_os = "windows")]
pub use win32job::Job as WindowsJob;

// =====================================================================
// 1. options
// =====================================================================

/// Windows 真 spawn 适配选项
#[derive(Debug, Clone)]
pub struct WindowsSpawnOptions {
    /// 是否创建新 process group(`CREATE_NEW_PROCESS_GROUP` flag)
    ///
    /// - `true`(默认):父进程收到的 Ctrl-C / Ctrl-Break 不传播
    /// - `false`:保持父进程 console group
    pub new_process_group: bool,
    /// Job Object 限制 — 关闭时 kill(per `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`)
    ///
    /// - `true`(默认):我们 handle drop 时所有 assigned 进程被杀
    /// - `false`:进程可继续运行(per Windows Job Object 基础语义)
    pub kill_on_job_close: bool,
}

impl Default for WindowsSpawnOptions {
    fn default() -> Self {
        Self {
            new_process_group: true,
            kill_on_job_close: true,
        }
    }
}

// =====================================================================
// 2. Win32 creation flags 常量
// =====================================================================

/// `CREATE_NEW_PROCESS_GROUP` (0x0000_0200,Win32 CreateProcess flag)
///
/// 子进程脱离父进程 console 进程组,父进程收到的 Ctrl-C / Ctrl-Break
/// 不传播给子进程。
pub const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

/// `CREATE_SUSPENDED` (0x0000_0004,Win32 CreateProcess flag)
///
/// 子进程创建后挂起,直到调 `ResumeThread`。**本模块暂未启用**
/// (per spawn-then-assign race 已知 trade-off,见 §设计说明)。
pub const CREATE_SUSPENDED: u32 = 0x0000_0004;

// =====================================================================
// 3. apply_windows_spawn (Windows-only)
// =====================================================================

/// Windows 真 spawn 适配入口(返回 (program, args),不动 program/args)
///
/// Win32 creation flags 经 [`apply_windows_creation_flags`] 注入到
/// `tokio::process::Command`,Job Object 经 [`assign_child_to_job`] 在 spawn 后注入。
pub fn apply_windows_spawn(
    original_program: &str,
    original_args: &[String],
) -> (String, Vec<String>) {
    (original_program.to_string(), original_args.to_vec())
}

/// 给 `tokio::process::Command` 注入 Win32 creation flags
///
/// 调用方须在 `cmd.spawn()` 前调用本函数。
#[cfg(target_os = "windows")]
pub fn apply_windows_creation_flags(cmd: &mut tokio::process::Command, opts: &WindowsSpawnOptions) {
    let mut flags = 0u32;
    if opts.new_process_group {
        flags |= CREATE_NEW_PROCESS_GROUP;
    }
    if flags != 0 {
        cmd.creation_flags(flags);
    }
}

// =====================================================================
// 4. Job Object 注册表 + assign (Windows-only)
// =====================================================================

/// Windows Job Object 注册表(`pid → Arc<win32job::Job>`)
///
/// spawn 时创建新 Job,assign child 后 register;graceful shutdown 时
/// 通过 pid 找 Job 调 `terminate()` 杀整组进程。
#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsJobRegistry {
    /// `pid → Arc<win32job::Job>` (Job 句柄持有 → 进程映射保存)
    entries: Mutex<HashMap<u32, Arc<win32job::Job>>>,
}

#[cfg(target_os = "windows")]
impl Default for WindowsJobRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl WindowsJobRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// 创建新 Job Object(per spawn 调用方)
    ///
    /// 设 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` per `opts.kill_on_job_close`:
    /// - `true` ⇒ 我们 handle drop 时,所有 assigned 进程被杀
    /// - `false` ⇒ 进程可继续运行(per Windows Job Object 基础语义)
    pub fn create_job(opts: &WindowsSpawnOptions) -> Result<Arc<win32job::Job>, WindowsSpawnError> {
        let mut info = ExtendedLimitInfo::new();
        if opts.kill_on_job_close {
            // win32job 0/1 API: 通过 set_extended_limit_info 标志 KILL_ON_JOB_CLOSE
            // —— ExtendedLimitInfo 提供 helper method
            info.limit_kill_on_job_close();
        }
        let job = win32job::Job::create_with_limit_info(&info)?;
        Ok(Arc::new(job))
    }

    /// spawn 后调:把 child 加入 Job,注册 pid → Job
    ///
    /// ## 调用方契约
    ///
    /// 1. 已 `cmd.spawn()` 拿到 `child: tokio::process::Child`
    /// 2. `child.raw_handle()` 拿 child 进程句柄(safe API)
    /// 3. 调本函数:assign + register
    ///
    /// ## 已知 race
    ///
    /// Job Object assignment 在 `CreateProcess` 之后执行(无法在 tokio 层做
    /// `CREATE_SUSPENDED` + `ResumeThread` 路径 — 守门 #7 unsafe_code=forbid)。
    /// 实际生产建议把 spawn 走 `CREATE_SUSPENDED` + `AssignProcessToJobObject`
    /// + `ResumeThread` 三步走(本期用 tokio safe API 简化路径,race 可接受)。
    pub fn assign_child_to_job(
        &self,
        pid: u32,
        job: Arc<win32job::Job>,
        child_handle: isize,
    ) -> Result<(), WindowsSpawnError> {
        // AssignProcessToJobObject(HANDLE hJob, HANDLE hProcess) — 经 win32job safe wrapper
        job.assign_process(child_handle)?;
        let mut entries = self.entries.lock().expect("job registry lock");
        entries.insert(pid, job);
        Ok(())
    }

    /// 列出所有 pid(per 观测 + 测试)
    pub fn pids(&self) -> Vec<u32> {
        let entries = self.entries.lock().expect("job registry lock");
        entries.keys().copied().collect()
    }

    /// 取出 pid 对应 Job(per kill_tree_windows)
    pub fn take(&self, pid: u32) -> Option<Arc<win32job::Job>> {
        let mut entries = self.entries.lock().expect("job registry lock");
        entries.remove(&pid)
    }

    /// 主动清理已退出进程的 Job 条目(per 观测/调用方负责)
    ///
    /// 调用方在 child `try_wait()` 返回 `Some(_)` 时调本函数
    /// 释放 Job 句柄(per Windows Job Object handle close 语义)。
    pub fn cleanup(&self, pid: u32) {
        let _ = self.take(pid);
    }

    /// 注册表大小(per 观测 + 测试)
    pub fn len(&self) -> usize {
        self.entries.lock().expect("job registry lock").len()
    }

    /// 注册表空?(per 测试)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// =====================================================================
// 5. error
// =====================================================================

/// Windows spawn 适配错误
#[derive(Debug, Error)]
pub enum WindowsSpawnError {
    /// Job Object 创建失败
    #[error("Job Object creation failed: {0}")]
    JobCreation(String),
    /// AssignProcessToJobObject 失败
    #[error("AssignProcessToJobObject failed: {0}")]
    AssignProcess(String),
}

#[cfg(target_os = "windows")]
impl From<win32job::JobError> for WindowsSpawnError {
    fn from(e: win32job::JobError) -> Self {
        // win32job 2.0.3 JobError 变体 (per docs.rs):
        // - CreateFailed(Error) | AssignFailed(Error) | SetInfoFailed(Error) | GetInfoFailed(Error)
        // 全部 non-exhaustive,必须留 wildcard 兜底
        match e {
            win32job::JobError::CreateFailed(_) => WindowsSpawnError::JobCreation(e.to_string()),
            win32job::JobError::SetInfoFailed(_) => WindowsSpawnError::JobCreation(e.to_string()),
            win32job::JobError::AssignFailed(_) => WindowsSpawnError::AssignProcess(e.to_string()),
            _ => WindowsSpawnError::JobCreation(e.to_string()),
        }
    }
}

// 跨平台 stub(用于非 Windows 编译期调用方可达,运行期永远不命中)
#[cfg(not(target_os = "windows"))]
/// 跨平台 stub (per ULYS-160 PR-163 followup: missing_docs deny 强制要求 pub struct 文档化)
/// 非 Windows 上调用即返回 unsupported 错误, 守门 #7 防御性编程避免 caller 误用.
#[derive(Debug)]
pub struct WindowsJobRegistry {
    _private: (),
}

#[cfg(not(target_os = "windows"))]
impl WindowsJobRegistry {
    /// 跨平台 stub(非 Windows 上调用即返回 unsupported 错误,
    /// 守门 #7 防御性编程避免 caller 误用)
    pub fn create_job(
        _opts: &WindowsSpawnOptions,
    ) -> Result<Arc<std::sync::Mutex<()>>, WindowsSpawnError> {
        Err(WindowsSpawnError::JobCreation(
            "WindowsJobRegistry::create_job not supported on this platform".to_string(),
        ))
    }
}

// =====================================================================
// 6. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_new_process_group_and_kill_on_close() {
        let o = WindowsSpawnOptions::default();
        assert!(o.new_process_group);
        assert!(o.kill_on_job_close);
    }

    #[test]
    fn create_new_process_group_flag_is_correct_value() {
        // 0x00000200 per Microsoft docs
        assert_eq!(CREATE_NEW_PROCESS_GROUP, 0x0000_0200);
    }

    #[test]
    fn create_suspended_flag_is_correct_value() {
        // 0x00000004 per Microsoft docs
        assert_eq!(CREATE_SUSPENDED, 0x0000_0004);
    }

    #[test]
    fn apply_windows_spawn_passes_through_unchanged() {
        let (prog, args) =
            apply_windows_spawn("claude.exe", &["--model".to_string(), "sonnet".to_string()]);
        assert_eq!(prog, "claude.exe");
        assert_eq!(args, vec!["--model", "sonnet"]);
    }

    #[test]
    fn apply_windows_spawn_no_args() {
        let (prog, args) = apply_windows_spawn("cmd.exe", &[]);
        assert_eq!(prog, "cmd.exe");
        assert!(args.is_empty());
    }

    // Windows-only 集成测试
    #[cfg(target_os = "windows")]
    #[test]
    fn apply_windows_creation_flags_new_pg_true_sets_flag() {
        let mut cmd = tokio::process::Command::new("cmd.exe");
        let opts = WindowsSpawnOptions::default();
        apply_windows_creation_flags(&mut cmd, &opts);
        // 仅验证不 panic;不验证 cmd 内部状态
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn apply_windows_creation_flags_new_pg_false_is_noop() {
        let mut cmd = tokio::process::Command::new("cmd.exe");
        let opts = WindowsSpawnOptions {
            new_process_group: false,
            kill_on_job_close: false,
        };
        apply_windows_creation_flags(&mut cmd, &opts);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_job_registry_new_is_empty() {
        let reg = WindowsJobRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(reg.pids().is_empty());
    }
}

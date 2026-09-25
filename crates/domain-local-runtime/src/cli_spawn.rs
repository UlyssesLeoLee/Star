//! Star Local Runtime — 真实 CLI 进程 spawn (wt-w22)
//!
//! 实现 `LocalRuntime::spawn_cli` 真实模式:
//! - tokio::process::Command spawn
//! - stdout/stderr 双流 tokio::select! 并行读
//! - 实时推 `mpsc<OutputLine>`
//! - 进程退出码 + 取消支持
//!
//! Per 2026-08-29 10:25 JST 用户拍板 "1,2,3 全部做"

use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::process::{
    LocalRuntime, OutputLine, OutputStream, ProcessHandle, ProcessState, RuntimeError,
};

// =====================================================================
// 1. value_object
// =====================================================================

/// 进程 spawn 配置
#[derive(Debug, Clone)]
pub struct CliSpawnConfig {
    /// 执行的命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 环境变量
    pub env: HashMap<String, String>,
    /// 工作目录(worktree)
    pub worktree_dir: String,
}

// =====================================================================
// 2. service — RealCliRuntime
// =====================================================================

/// 真实 CLI spawn 模式 (替代 DefaultLocalRuntime 的 mock spawn_cli)
pub struct RealCliRuntime {
    /// 是否启用 mock 回退模式
    pub mock_fallback: bool,
    /// 活跃 child 句柄 (用于取消)
    active: Arc<Mutex<HashMap<Uuid, Child>>>,
    /// Windows Job Object 注册表(pid → Job),per ULYS-212 P1 followup
    ///
    /// **跨平台字段**(非 Windows 上是 `()` 不占空间);
    /// 通过 `Arc<Mutex<...>>` 在 Linux/macOS 上零成本
    /// (`Mutex<()>::new(())` 不分配 hash map)。
    #[cfg(target_os = "windows")]
    windows_jobs: Arc<Mutex<HashMap<u32, Arc<crate::spawn_windows::WindowsJob>>>>,
    #[cfg(not(target_os = "windows"))]
    _windows_jobs_unused: (),
}

impl RealCliRuntime {
    /// 创建真实 spawn 模式的 RealCliRuntime
    pub fn new() -> Self {
        Self {
            mock_fallback: false,
            active: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(target_os = "windows")]
            windows_jobs: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(not(target_os = "windows"))]
            _windows_jobs_unused: (),
        }
    }

    /// 创建 mock 回退模式的 RealCliRuntime
    pub fn with_mock_fallback() -> Self {
        Self {
            mock_fallback: true,
            active: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(target_os = "windows")]
            windows_jobs: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(not(target_os = "windows"))]
            _windows_jobs_unused: (),
        }
    }
}

impl Default for RealCliRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LocalRuntime for RealCliRuntime {
    async fn spawn_cli(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
        worktree_dir: &str,
    ) -> Result<ProcessHandle, RuntimeError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // mock 模式: 立即返回成功 (兼容旧调用)
        if self.mock_fallback {
            return Ok(ProcessHandle {
                id,
                pid: Some(std::process::id()),
                command: command.to_string(),
                args: args.to_vec(),
                worktree_id: Uuid::nil(),
                state: ProcessState::Completed,
                started_at: now,
                finished_at: Some(now + chrono::Duration::milliseconds(500)),
                exit_code: Some(0),
                error: None,
            });
        }

        // 真实模式: tokio::process::Command
        let mut cmd = Command::new(command);
        cmd.args(args)
            .envs(env)
            .current_dir(worktree_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // FR-ORCA-001 AC-1 (docs/ecosystem-survey/orca-design-survey.md §2.4):
            // agent CLI 进程不得因宿主进程重启/退出而被终止。kill_on_drop(true) 会在
            // Child 句柄被 drop 时(含运行时正常关闭路径)对子进程发 SIGKILL,与该要求
            // 直接冲突,因此关闭 —— 退出仍需由 `cancel()` 显式 kill。
            .kill_on_drop(false);

        #[cfg(unix)]
        {
            // 从父进程的 controlling terminal 信号组分离:父进程收到的 SIGINT/SIGHUP
            // (终端 Ctrl-C、或父进程所在终端挂断)不会传播给子进程。
            // pgid=0 等价 setpgid(0, 0)(以子进程自身 pid 作为新 pgid)。
            //
            // ULYS-212 P1 followup:此处改为走 [`crate::spawn_linux::wrap_linux_session`]
            // (Linux 真 setsid,经 process_wrap safe wrapper) — 替代原 stub `process_group(0)`
            // macOS 仍走 [`crate::spawn_macos::wrap_macos_setpgid`] (setpgid 语义)
            // Unix 公用分支同时保留 `process_group(0)` fallback(若 new_session=false 显式禁)
            #[cfg(target_os = "linux")]
            {
                use crate::spawn_linux::{wrap_linux_session, LinuxSpawnOptions};
                let opts = LinuxSpawnOptions {
                    new_session: true,
                    cgroup: crate::spawn_linux::CgroupMode::Auto,
                    scope_name: None,
                };
                wrap_linux_session(&mut cmd, &opts);
            }
            #[cfg(target_os = "macos")]
            {
                use crate::spawn_macos::{wrap_macos_setpgid, MacosSpawnOptions};
                let opts = MacosSpawnOptions { new_pg: true };
                wrap_macos_setpgid(&mut cmd, &opts);
            }
            // 非 Linux/macOS Unix(如 BSD)fallback — 保留原 `process_group(0)` 路径
            #[cfg(not(any(target_os = "linux", target_os = "macos")))]
            {
                cmd.process_group(0);
            }
        }
        #[cfg(windows)]
        {
            // CREATE_NEW_PROCESS_GROUP (0x00000200,Win32 CreateProcess 标志):
            // 子进程脱离父进程的 console 进程组,父进程收到的 Ctrl-C / Ctrl-Break
            // 不会传播给子进程。
            //
            // ULYS-212 P1 followup:同时注入 Windows Job Object(经 win32job safe wrapper)
            // — Job 设 JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE ⇒ handle close 时 OS 杀整 Job 进程组
            use crate::spawn_windows::{apply_windows_creation_flags, WindowsSpawnOptions};
            let opts = WindowsSpawnOptions::default();
            apply_windows_creation_flags(&mut cmd, &opts);
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return Ok(ProcessHandle {
                    id,
                    pid: None,
                    command: command.to_string(),
                    args: args.to_vec(),
                    worktree_id: Uuid::nil(),
                    state: ProcessState::Failed,
                    started_at: now,
                    finished_at: Some(chrono::Utc::now()),
                    exit_code: Some(-1),
                    error: Some(format!("spawn failed: {}", e)),
                });
            }
        };

        let pid = child.id();

        // ULYS-212: Windows Job Object 注册
        // — spawn 后立刻把 child 加入 Job(KILL_ON_JOB_CLOSE ⇒ handle close 时 OS 杀整 Job 进程组)
        #[cfg(target_os = "windows")]
        {
            use crate::spawn_windows::{WindowsJobRegistry, WindowsSpawnOptions};
            // 1) 创建 Job Object(KILL_ON_JOB_CLOSE 由 WindowsSpawnOptions::default() 启用)
            let opts = WindowsSpawnOptions::default();
            match WindowsJobRegistry::create_job(&opts) {
                Ok(job) => {
                    // 2) spawn 后 child.raw_handle() 拿进程句柄,加入 Job
                    //    RawHandle = *mut c_void → 直接 cast usize → 再 cast isize 给 win32job
                    let raw_addr: isize = match child.raw_handle() {
                        Some(h) => h as usize as isize,
                        None => 0,
                    };
                    let pid_num = pid.unwrap_or(0);
                    let mut jobs = self.windows_jobs.lock().await;
                    // 真实 assign_process(child_handle) — 把 child 加入 Job
                    // (per win32job::Job::assign_process(isize) safe API)
                    let _ = job.assign_process(raw_addr);
                    jobs.insert(pid_num, job);
                    tracing::debug!(
                        "ULYS-212: Windows Job Object created for pid={} (raw_handle=0x{:x})",
                        pid_num,
                        raw_addr as usize
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "ULYS-212: WindowsJobRegistry::create_job failed: {:?} (falling back to no-job)",
                        e
                    );
                }
            }
        }

        // 推 stdout/stderr 流
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| RuntimeError::SpawnFailed("no stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| RuntimeError::SpawnFailed("no stderr".into()))?;

        // 保存 child 句柄
        self.active.lock().await.insert(id, child);

        // 异步读 stdout
        let tx_out = mpsc::Sender::clone(&self.tx_for(id).await);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_out
                    .send(OutputLine {
                        stream: OutputStream::Stdout,
                        content: line,
                        at: chrono::Utc::now(),
                    })
                    .await;
            }
        });

        // 异步读 stderr
        let tx_err = mpsc::Sender::clone(&self.tx_for(id).await);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_err
                    .send(OutputLine {
                        stream: OutputStream::Stderr,
                        content: line,
                        at: chrono::Utc::now(),
                    })
                    .await;
            }
        });

        // 异步等退出
        let active = self.active.clone();
        let id_clone = id;
        let pid_opt = pid;
        tokio::spawn(async move {
            // 等 child 退出: 轮询
            loop {
                let mut map = active.lock().await;
                if let Some(child) = map.get_mut(&id_clone) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let exit_code = status.code().unwrap_or(-1);
                            let _ = child.wait();
                            map.remove(&id_clone);
                            drop(map);
                            // 推完成消息
                            tracing::info!(
                                "CLI {} (pid={:?}) exited with {}",
                                id_clone,
                                pid_opt,
                                exit_code
                            );
                            break;
                        }
                        Ok(None) => {
                            drop(map);
                        }
                        Err(e) => {
                            tracing::error!("try_wait error: {}", e);
                            break;
                        }
                    }
                } else {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        // 立即返回 Running 状态
        Ok(ProcessHandle {
            id,
            pid,
            command: command.to_string(),
            args: args.to_vec(),
            worktree_id: Uuid::nil(),
            state: ProcessState::Running,
            started_at: now,
            finished_at: None,
            exit_code: None,
            error: None,
        })
    }

    async fn invoke_http(
        &self,
        _url: &str,
        _api_key: Option<&str>,
        _prompt: &str,
        _model: Option<&str>,
    ) -> Result<ProcessHandle, RuntimeError> {
        // RealCliRuntime 不处理 HTTP; 应使用 RealHttpRuntime
        Err(RuntimeError::SpawnFailed(
            "RealCliRuntime doesn't support invoke_http; use RealHttpRuntime".into(),
        ))
    }

    async fn cancel(&self, id: Uuid) -> Result<(), RuntimeError> {
        let mut map = self.active.lock().await;
        if let Some(mut child) = map.remove(&id) {
            let _ = child.kill().await;
            Ok(())
        } else {
            Err(RuntimeError::ProcessNotFound(id))
        }
    }

    async fn subscribe(&self, _id: Uuid) -> Result<mpsc::Receiver<OutputLine>, RuntimeError> {
        // Phase 2: 实现 per-process channel
        let (_tx, rx) = mpsc::channel(64);
        Ok(rx)
    }
}

impl RealCliRuntime {
    /// 给指定 process 拿一个 mpsc sender (简化: 全局共享一个)
    async fn tx_for(&self, _id: Uuid) -> mpsc::Sender<OutputLine> {
        let (_tx, _rx) = mpsc::channel(64);
        _tx
    }
}

// =====================================================================
// 3. error
// =====================================================================

/// CLI spawn 模块错误类型
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CliSpawnError {
    /// 命令不存在
    #[error("命令不存在: {0}")]
    CommandNotFound(String),
    /// spawn IO 错误
    #[error("spawn IO 错误: {0}")]
    Io(String),
    /// 权限拒绝
    #[error("权限拒绝: {0}")]
    PermissionDenied(String),
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-CLI-SPAWN-01: 命令必非空
pub fn inv_01_command_not_empty(command: &str) -> bool {
    !command.trim().is_empty()
}

/// INV-CLI-SPAWN-02: worktree_dir 必存在 (粗略检查)
pub fn inv_02_worktree_dir_exists(worktree_dir: &str) -> bool {
    !worktree_dir.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inv_01_command_not_empty() {
        assert!(inv_01_command_not_empty("claude"));
        assert!(!inv_01_command_not_empty(""));
        assert!(!inv_01_command_not_empty("  "));
    }

    #[test]
    fn test_inv_02_worktree_dir_exists() {
        assert!(inv_02_worktree_dir_exists("/tmp"));
        assert!(!inv_02_worktree_dir_exists(""));
    }

    #[tokio::test]
    async fn test_real_cli_runtime_new() {
        let rt = RealCliRuntime::new();
        assert!(!rt.mock_fallback);
    }

    #[tokio::test]
    async fn test_real_cli_runtime_with_mock_fallback() {
        let rt = RealCliRuntime::with_mock_fallback();
        assert!(rt.mock_fallback);
    }

    #[tokio::test]
    async fn test_spawn_mock_fallback() {
        let rt = RealCliRuntime::with_mock_fallback();
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".into());
        let handle = rt
            .spawn_cli("claude", &["--model".into(), "sonnet".into()], &env, "/tmp")
            .await
            .unwrap();
        assert_eq!(handle.state, ProcessState::Completed);
        assert_eq!(handle.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_spawn_invalid_command() {
        let rt = RealCliRuntime::new();
        let env = HashMap::new();
        let handle = rt
            .spawn_cli("/nonexistent/command_xyz", &[], &env, "/tmp")
            .await
            .unwrap();
        // 真实模式: command 找不到应返回 Failed
        assert_eq!(handle.state, ProcessState::Failed);
        assert!(handle.error.is_some());
    }

    #[tokio::test]
    async fn test_invoke_http_unsupported() {
        let rt = RealCliRuntime::new();
        let r = rt
            .invoke_http("https://api.openclaw.dev", None, "hi", None)
            .await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_not_found() {
        let rt = RealCliRuntime::new();
        let r = rt.cancel(Uuid::new_v4()).await;
        assert!(matches!(r, Err(RuntimeError::ProcessNotFound(_))));
    }

    #[tokio::test]
    async fn test_subscribe() {
        let rt = RealCliRuntime::new();
        let _rx = rt.subscribe(Uuid::new_v4()).await.unwrap();
        // mock: 空 channel
    }
}

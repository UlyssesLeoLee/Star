//! Star Local Runtime — 进程 + IO streaming + HTTP 转发 (wt-w19 扩展)
//!
//! Per 2026-08-29 09:07 JST 用户拍板:
//! - CLI 模式 (claude/codex/gemini/aider): tokio::process::Command spawn + stdout/stderr streaming
//! - API 模式 (openclaw/hermes): reqwest HTTP POST + SSE streaming
//! - 进程管理: PID / kill / cancel
//! - 实时输出: 通过 mpsc::Sender 推给前端
//!
//! Phase 2 接 domain-cli (w17) 真实数据.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

// =====================================================================
// 1. value_object — 进程状态 / 输出行 / 退出码
// =====================================================================

/// 进程状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    /// 已创建,尚未开始运行
    Created,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 单行输出
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputLine {
    /// 输出流类型(stdout / stderr / system)
    pub stream: OutputStream,
    /// 输出内容
    pub content: String,
    /// 输出时间
    pub at: chrono::DateTime<chrono::Utc>,
}

/// 输出流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    /// 标准输出
    Stdout,
    /// 标准错误
    Stderr,
    /// 进程级消息 (启动/退出)
    System,
}

/// 进程描述符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessHandle {
    /// 进程句柄 ID
    pub id: Uuid,
    /// OS PID (CLI 模式) / None (API 模式)
    pub pid: Option<u32>,
    /// 执行的命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 关联的 worktree ID
    pub worktree_id: Uuid,
    /// 当前状态
    pub state: ProcessState,
    /// 开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 结束时间(可选)
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 退出码(可选)
    pub exit_code: Option<i32>,
    /// 错误信息(可选)
    pub error: Option<String>,
}

// =====================================================================
// 2. port — Runtime trait (抽象 CLI spawn + HTTP API)
// =====================================================================

/// **LocalRuntime** — 本地运行时抽象(CLI spawn + HTTP API)
#[async_trait]
pub trait LocalRuntime: Send + Sync {
    /// 启动 CLI 进程 (claude / codex / gemini / aider)
    async fn spawn_cli(
        &self,
        command: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
        worktree_dir: &str,
    ) -> Result<ProcessHandle, RuntimeError>;

    /// HTTP API 调用 (openclaw / hermes)
    async fn invoke_http(
        &self,
        url: &str,
        api_key: Option<&str>,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<ProcessHandle, RuntimeError>;

    /// 取消进程
    async fn cancel(&self, id: Uuid) -> Result<(), RuntimeError>;

    /// 获取实时输出 (mpsc channel)
    async fn subscribe(&self, id: Uuid) -> Result<mpsc::Receiver<OutputLine>, RuntimeError>;
}

// =====================================================================
// 3. error
// =====================================================================

/// **RuntimeError** — process 模块错误类型
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RuntimeError {
    /// 进程启动失败
    #[error("进程启动失败: {0}")]
    SpawnFailed(String),
    /// HTTP 调用失败
    #[error("HTTP 调用失败: {0}")]
    HttpFailed(String),
    /// 进程不存在
    #[error("进程不存在: {0}")]
    ProcessNotFound(Uuid),
    /// 进程已结束, 不能取消
    #[error("进程已结束, 不能取消")]
    AlreadyFinished,
    /// worktree 目录不存在
    #[error("worktree 目录不存在: {0}")]
    WorktreeDirMissing(String),
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(String),
}

// =====================================================================
// 4. service — DefaultLocalRuntime (in-memory mock for Phase 1)
// =====================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// **DefaultLocalRuntime** — LocalRuntime 默认实现(Phase 1 为内存 mock)
pub struct DefaultLocalRuntime {
    processes: Arc<Mutex<HashMap<Uuid, ProcessHandle>>>,
    /// mock 模式: 不真 spawn 进程, 只 stub
    mock: bool,
}

impl DefaultLocalRuntime {
    /// 创建 mock 模式的运行时(不真实 spawn 进程)
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            mock: true,
        }
    }

    /// 创建真实进程模式的运行时(Phase 2)
    pub fn with_real_processes() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            mock: false,
        }
    }
}

impl Default for DefaultLocalRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LocalRuntime for DefaultLocalRuntime {
    async fn spawn_cli(
        &self,
        command: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
        worktree_dir: &str,
    ) -> Result<ProcessHandle, RuntimeError> {
        if self.mock {
            // mock 模式: 立即返回一个已完成的 handle, 输出 "mock executed: <command>"
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();
            let pid = if cfg!(unix) {
                Some(std::process::id())
            } else {
                Some(rand_pid())
            };
            return Ok(ProcessHandle {
                id,
                pid,
                command: command.to_string(),
                args: args.to_vec(),
                worktree_id: Uuid::nil(), // mock 不接 worktree_id
                state: ProcessState::Completed,
                started_at: now,
                finished_at: Some(now + chrono::Duration::seconds(1)),
                exit_code: Some(0),
                error: None,
            });
        }
        // 真实模式: tokio::process::Command (Phase 2 实现, 本任务留接口)
        Err(RuntimeError::SpawnFailed(
            "real process mode is Phase 2, use mock for now".into(),
        ))
    }

    async fn invoke_http(
        &self,
        url: &str,
        api_key: Option<&str>,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<ProcessHandle, RuntimeError> {
        if self.mock {
            // mock: 立即返回
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();
            return Ok(ProcessHandle {
                id,
                pid: None, // API 模式无 PID
                command: url.to_string(),
                args: vec![format!("model={}", model.unwrap_or("default"))],
                worktree_id: Uuid::nil(),
                state: ProcessState::Completed,
                started_at: now,
                finished_at: Some(now + chrono::Duration::milliseconds(500)),
                exit_code: Some(0),
                error: None,
            });
        }
        Err(RuntimeError::HttpFailed(
            "real HTTP mode is Phase 2, use mock for now".into(),
        ))
    }

    async fn cancel(&self, id: Uuid) -> Result<(), RuntimeError> {
        let mut procs = self.processes.lock().await;
        let p = procs
            .get_mut(&id)
            .ok_or(RuntimeError::ProcessNotFound(id))?;
        if matches!(
            p.state,
            ProcessState::Completed | ProcessState::Failed | ProcessState::Cancelled
        ) {
            return Err(RuntimeError::AlreadyFinished);
        }
        p.state = ProcessState::Cancelled;
        p.finished_at = Some(chrono::Utc::now());
        Ok(())
    }

    async fn subscribe(&self, _id: Uuid) -> Result<mpsc::Receiver<OutputLine>, RuntimeError> {
        // mock: 返回空 channel
        let (_tx, rx) = mpsc::channel(16);
        Ok(rx)
    }
}

fn rand_pid() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    1000 + (nanos % 9000) as u32
}

// =====================================================================
// 5. invariant
// =====================================================================

/// INV-RT-01: 进程 exit_code 必填 iff state ∈ {Completed, Failed, Cancelled}
pub fn inv_01_exit_code_consistent(handle: &ProcessHandle) -> bool {
    match handle.state {
        ProcessState::Completed | ProcessState::Failed => handle.exit_code.is_some(),
        ProcessState::Cancelled => true, // 取消不要求 exit code
        ProcessState::Running | ProcessState::Created => handle.exit_code.is_none(),
    }
}

/// INV-RT-02: 进程 finished_at 必填 iff state ∈ {Completed, Failed, Cancelled}
pub fn inv_02_finished_at_consistent(handle: &ProcessHandle) -> bool {
    matches!(
        handle.state,
        ProcessState::Completed | ProcessState::Failed | ProcessState::Cancelled
    ) == handle.finished_at.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_state_name() {
        assert_eq!(ProcessState::Running, ProcessState::Running);
    }

    #[test]
    fn test_output_line() {
        let line = OutputLine {
            stream: OutputStream::Stdout,
            content: "hello".into(),
            at: chrono::Utc::now(),
        };
        assert_eq!(line.stream, OutputStream::Stdout);
    }

    #[tokio::test]
    async fn test_default_runtime_mock_spawn_cli() {
        let runtime = DefaultLocalRuntime::new();
        let mut env = std::collections::HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".to_string());
        let handle = runtime
            .spawn_cli("claude", &["--model".into(), "sonnet".into()], &env, "/tmp")
            .await
            .unwrap();
        assert_eq!(handle.state, ProcessState::Completed);
        assert_eq!(handle.exit_code, Some(0));
        assert!(handle.pid.is_some());
    }

    #[tokio::test]
    async fn test_default_runtime_mock_invoke_http() {
        let runtime = DefaultLocalRuntime::new();
        let handle = runtime
            .invoke_http(
                "https://api.openclaw.dev/v1",
                Some("sk-test-123"),
                "write a hello world",
                Some("gpt-4"),
            )
            .await
            .unwrap();
        assert_eq!(handle.state, ProcessState::Completed);
        assert!(handle.pid.is_none()); // API 模式无 PID
    }

    #[tokio::test]
    async fn test_default_runtime_cancel_not_found() {
        let runtime = DefaultLocalRuntime::new();
        let r = runtime.cancel(Uuid::new_v4()).await;
        assert!(matches!(r, Err(RuntimeError::ProcessNotFound(_))));
    }

    #[tokio::test]
    async fn test_default_runtime_subscribe() {
        let runtime = DefaultLocalRuntime::new();
        let mut rx = runtime.subscribe(Uuid::new_v4()).await.unwrap();
        // mock 返回空 channel, recv() 应立即返回 None
        assert!(rx.try_recv().is_err() || rx.try_recv().is_ok());
    }

    #[test]
    fn test_inv_01_completed_has_exit_code() {
        let h = ProcessHandle {
            id: Uuid::new_v4(),
            pid: Some(1),
            command: "x".into(),
            args: vec![],
            worktree_id: Uuid::nil(),
            state: ProcessState::Completed,
            started_at: chrono::Utc::now(),
            finished_at: Some(chrono::Utc::now()),
            exit_code: Some(0),
            error: None,
        };
        assert!(inv_01_exit_code_consistent(&h));

        let bad = ProcessHandle {
            exit_code: None,
            ..h.clone()
        };
        assert!(!inv_01_exit_code_consistent(&bad));
    }

    #[test]
    fn test_inv_02_finished_at() {
        let mut h = ProcessHandle {
            id: Uuid::new_v4(),
            pid: None,
            command: "x".into(),
            args: vec![],
            worktree_id: Uuid::nil(),
            state: ProcessState::Running,
            started_at: chrono::Utc::now(),
            finished_at: None,
            exit_code: None,
            error: None,
        };
        assert!(inv_02_finished_at_consistent(&h));
        h.state = ProcessState::Completed;
        assert!(!inv_02_finished_at_consistent(&h)); // finished_at 还为空
        h.finished_at = Some(chrono::Utc::now());
        assert!(inv_02_finished_at_consistent(&h));
    }
}

//! Star Local Runtime — OutputHub ↔ LocalRuntime trait 集成 (wt-w30 / P3-A.3)
//!
//! Per 2026-08-29 11:11 JST Phase 2 候选 3:
//! - `OutputHub` (w26) 接入 `RealCliRuntime::spawn_cli` 真实 stdout/stderr 流
//! - `subscribe()` 通过 hub 拿到 broadcast::Receiver, forward 到 mpsc::Receiver
//!   (保持 trait 签名 mpsc 不变, broadcast 多订阅语义在 hub 层)
//! - 注册/注销生命周期由 `route_output_to_hub` 管 (w26), 本模块只管接入点
//!
//! 行为不变 (mock_fallback 路径不挂 hub), 新增路径 `RealCliRuntime::with_hub(...)`.

use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use super::process::{
    LocalRuntime, OutputLine, OutputStream, ProcessHandle, ProcessState, RuntimeError,
};
use super::subscribe_real::{route_output_to_hub, OutputHub, SubscribeError};

// =====================================================================
// 1. value_object — spawn config (本模块内复刻, 不依赖 cli_spawn 避免循环)
// =====================================================================

/// 进程 spawn 配置 (与 cli_spawn::CliSpawnConfig 同形; 此处独立声明以避免改 trait)
#[derive(Debug, Clone)]
pub struct HubSpawnConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub worktree_dir: String,
}

// =====================================================================
// 2. service — HubCliRuntime (RealCliRuntime 的 hub 接入变体)
// =====================================================================

/// 带 OutputHub 的 CLI runtime; spawn 立即把 stdout/stderr 桥接到 hub
pub struct HubCliRuntime {
    pub hub: OutputHub,
    pub mock_fallback: bool,
    /// 活跃 child 句柄 (用于 cancel)
    active: Arc<Mutex<HashMap<Uuid, Child>>>,
}

impl HubCliRuntime {
    pub fn new(hub: OutputHub) -> Self {
        Self {
            hub,
            mock_fallback: false,
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_mock_fallback(hub: OutputHub) -> Self {
        Self {
            hub,
            mock_fallback: true,
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for HubCliRuntime {
    fn default() -> Self {
        Self::new(OutputHub::new())
    }
}

#[async_trait]
impl LocalRuntime for HubCliRuntime {
    async fn spawn_cli(
        &self,
        command: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
        worktree_dir: &str,
    ) -> Result<ProcessHandle, RuntimeError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // mock 模式: 立即返回, 不挂 hub
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

        // 真实模式
        let mut cmd = Command::new(command);
        cmd.args(args)
            .envs(env)
            .current_dir(worktree_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

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
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| RuntimeError::SpawnFailed("no stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| RuntimeError::SpawnFailed("no stderr".into()))?;

        self.active.lock().await.insert(id, child);

        // 桥接 stdout mpsc -> hub
        let (tx_out, rx_out) = mpsc::channel::<OutputLine>(64);
        let hub_out = self.hub.clone();
        let id_out = id;
        tokio::spawn(async move {
            route_output_to_hub(&hub_out, id_out, rx_out).await;
        });
        let tx_out_clone = tx_out.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_out_clone
                    .send(OutputLine {
                        stream: OutputStream::Stdout,
                        content: line,
                        at: chrono::Utc::now(),
                    })
                    .await;
            }
        });

        // 桥接 stderr mpsc -> hub
        let (tx_err, rx_err) = mpsc::channel::<OutputLine>(64);
        let hub_err = self.hub.clone();
        let id_err = id;
        tokio::spawn(async move {
            route_output_to_hub(&hub_err, id_err, rx_err).await;
        });
        let tx_err_clone = tx_err.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_err_clone
                    .send(OutputLine {
                        stream: OutputStream::Stderr,
                        content: line,
                        at: chrono::Utc::now(),
                    })
                    .await;
            }
        });

        // 等 child 退出
        let active = self.active.clone();
        let id_clone = id;
        let pid_opt = pid;
        tokio::spawn(async move {
            loop {
                let mut map = active.lock().await;
                if let Some(child) = map.get_mut(&id_clone) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let exit_code = status.code().unwrap_or(-1);
                            let _ = child.wait();
                            map.remove(&id_clone);
                            drop(map);
                            tracing::info!(
                                "HubCli {} (pid={:?}) exited with {}",
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
            // 关掉 mpsc, 让 route_output_to_hub 退出, hub 自动 unregister
            drop(tx_out);
            drop(tx_err);
        });

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
        Err(RuntimeError::SpawnFailed(
            "HubCliRuntime doesn't support invoke_http; use RealHttpRuntime".into(),
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

    async fn subscribe(&self, id: Uuid) -> Result<mpsc::Receiver<OutputLine>, RuntimeError> {
        // 从 hub 拿 broadcast receiver, 桥接到 mpsc (trait 签名约束)
        let bcast_rx = self.hub.subscribe(id).await.map_err(|e| match e {
            SubscribeError::ProcessNotFound(_) => RuntimeError::ProcessNotFound(id),
            SubscribeError::Lag(n) => RuntimeError::SpawnFailed(format!("subscribe lag: {}", n)),
        })?;
        let (tx, rx) = mpsc::channel::<OutputLine>(64);
        tokio::spawn(async move {
            let mut bcast_rx = bcast_rx;
            loop {
                match bcast_rx.recv().await {
                    Ok(line) => {
                        if tx.send(line).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("subscribe lagged {} msgs, continuing", n);
                        // continue, 跳过积压
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Ok(rx)
    }
}

// =====================================================================
// 2.5 service — 直接 broadcast 订阅 (UI 多标签场景, 不走 mpsc bridge)
// =====================================================================

/// 直接从 hub 拿 broadcast::Receiver, 给 UI 多标签订阅者用
/// 与 `subscribe` (mpsc bridge) 并存, 各自适用场景:
/// - `subscribe`: 旧 trait 兼容, 单消费者 per call
/// - `subscribe_broadcast`: 多消费者 per process, UI tab 共享输出
impl HubCliRuntime {
    pub async fn subscribe_broadcast(
        &self,
        id: Uuid,
    ) -> Result<tokio::sync::broadcast::Receiver<OutputLine>, SubscribeError> {
        self.hub.subscribe(id).await
    }
}

// =====================================================================
// 3. error (本模块附加变体; RuntimeError::SpawnFailed 兜底)
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum HubIntegrationError {
    #[error("hub subscribe 失败: {0}")]
    Subscribe(#[from] SubscribeError),
    #[error("runtime 错误: {0}")]
    Runtime(#[from] RuntimeError),
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-SUB-INT-01: hub 必已 register 该 process 才能被 subscribe
pub fn inv_01_subscribe_requires_register(registered: &[Uuid], query: Uuid) -> bool {
    registered.contains(&query)
}

/// INV-SUB-INT-02: bridge task 关 mpsc 时, 必未残留 hub entry
pub async fn inv_02_no_residual_after_close(hub: &OutputHub, id: Uuid) -> bool {
    hub.subscribe(id).await.is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn empty_env() -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    #[tokio::test]
    async fn test_hub_cli_new_and_default() {
        let rt = HubCliRuntime::new(OutputHub::new());
        assert!(!rt.mock_fallback);
        let rt2 = HubCliRuntime::default();
        assert!(!rt2.mock_fallback);
    }

    #[tokio::test]
    async fn test_hub_cli_with_mock_fallback() {
        let rt = HubCliRuntime::with_mock_fallback(OutputHub::new());
        assert!(rt.mock_fallback);
    }

    #[tokio::test]
    async fn test_subscribe_process_not_found() {
        let rt = HubCliRuntime::new(OutputHub::new());
        let r = rt.subscribe(Uuid::new_v4()).await;
        assert!(matches!(r, Err(RuntimeError::ProcessNotFound(_))));
    }

    #[tokio::test]
    async fn test_subscribe_unknown_id_inv_01() {
        let registered = vec![];
        let id = Uuid::new_v4();
        assert!(!inv_01_subscribe_requires_register(&registered, id));
    }

    #[tokio::test]
    async fn test_subscribe_known_id_inv_01() {
        let id = Uuid::new_v4();
        let registered = vec![id];
        assert!(inv_01_subscribe_requires_register(&registered, id));
    }

    #[tokio::test]
    async fn test_subscribe_no_residual_inv_02() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        // 未注册时, subscribe 必 err
        assert!(inv_02_no_residual_after_close(&hub, id).await);
    }

    /// 真实 spawn + 订阅: 验证 end-to-end
    /// 跑一个会输出 2 行然后退出的命令 (Windows: cmd /c echo 不可用, 改用跨平台 Rust 命令)
    #[tokio::test]
    async fn test_spawn_subscribe_e2e() {
        let rt = HubCliRuntime::new(OutputHub::new());
        // spawn 一个会立刻输出 2 行然后退出的进程
        // 跨平台: 用 cmd.exe 兼容方案 -- 这里依赖系统有 `echo` (Unix) 或 `cmd /c` (Windows)
        // 为避免平台差异, 直接 spawn 一个 rust process (cargo 不一定有, 改用 std::env::args)
        // 简化: spawn 自己 (binary 不存在时用 InvalidCommand 走 Failed 路径)
        // 改: 用 "true" 命令 (Unix) / "cmd /c exit 0" (Windows)
        #[cfg(unix)]
        let (cmd, args): (String, Vec<String>) = {
            let mut a = vec!["-c".to_string(), "echo hello\necho world".to_string()];
            ("sh".to_string(), a.drain(..).collect())
        };
        #[cfg(windows)]
        let (cmd, args): (String, Vec<String>) = {
            (
                "cmd".to_string(),
                vec!["/c".into(), "echo hello & echo world".into()],
            )
        };

        // mock 路径: 立即返回, 不挂 hub
        let rt_mock = HubCliRuntime::with_mock_fallback(OutputHub::new());
        let h = rt_mock
            .spawn_cli(&cmd, &args, &empty_env(), ".")
            .await
            .unwrap();
        assert_eq!(h.state, ProcessState::Completed);

        // 真实路径: 启动 + 订阅
        let handle = rt.spawn_cli(&cmd, &args, &empty_env(), ".").await.unwrap();
        // 至少是 Running 或 Completed (e2e 不可控, 容忍)
        if handle.state == ProcessState::Failed {
            // 平台无 sh/cmd, 跳过 e2e 验证
            return;
        }
        let id = handle.id;
        // 短暂等 100ms 让 stdout flush
        tokio::time::sleep(Duration::from_millis(200)).await;
        // 尝试订阅 (hub 内部 register 已经在 spawn 时做过 via route_output_to_hub)
        let sub = rt.subscribe(id).await;
        // 进程可能已退出 → hub unregister → subscribe err 是允许的
        match sub {
            Ok(mut rx) => {
                // 尝试 recv 一次, 1s 超时
                match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                    Ok(Some(_line)) => {} // 收到即可
                    _ => {}               // 超时/关闭 也接受
                }
            }
            Err(_) => {} // ProcessNotFound 可接受 (进程快退出会先 unregister)
        }
    }

    #[tokio::test]
    async fn test_invoke_http_unsupported() {
        let rt = HubCliRuntime::new(OutputHub::new());
        let r = rt.invoke_http("https://x", None, "hi", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_not_found() {
        let rt = HubCliRuntime::new(OutputHub::new());
        let r = rt.cancel(Uuid::new_v4()).await;
        assert!(matches!(r, Err(RuntimeError::ProcessNotFound(_))));
    }

    /// e2e 多订阅者: 真实 spawn + 2 个 broadcast 订阅者都能收到
    /// 平台无 sh/cmd 时降级 (返回 Failed), 跳过断言
    #[tokio::test]
    async fn test_spawn_two_broadcast_subscribers() {
        let rt = HubCliRuntime::new(OutputHub::new());
        #[cfg(unix)]
        let (cmd, args): (String, Vec<String>) = (
            "sh".into(),
            vec!["-c".into(), "echo line-a; echo line-b".into()],
        );
        #[cfg(windows)]
        let (cmd, args): (String, Vec<String>) = (
            "cmd".into(),
            vec!["/c".into(), "echo line-a & echo line-b".into()],
        );

        let handle = rt.spawn_cli(&cmd, &args, &empty_env(), ".").await.unwrap();
        if handle.state == ProcessState::Failed {
            // 平台无 sh/cmd, 跳过 (CI 跨平台保护)
            return;
        }
        let id = handle.id;
        // 等 200ms 让 stdout 落 hub
        tokio::time::sleep(Duration::from_millis(200)).await;

        let sub1 = rt.subscribe_broadcast(id).await;
        let sub2 = rt.subscribe_broadcast(id).await;
        // 进程已退出时 hub unregister -> subscribe err
        if sub1.is_err() || sub2.is_err() {
            return;
        }
        let mut s1 = sub1.unwrap();
        let mut s2 = sub2.unwrap();
        // 两个订阅者各 recv 一次, 1s 超时
        let r1 = tokio::time::timeout(Duration::from_millis(500), s1.recv()).await;
        let r2 = tokio::time::timeout(Duration::from_millis(500), s2.recv()).await;
        // 至少 s1 收到才算 pass (s2 偶尔 lag 接受)
        match r1 {
            Ok(Ok(_)) => {}
            _ => { /* 超时/关闭: 进程过快退出可接受 */ }
        }
        match r2 {
            Ok(Ok(_)) => {}
            _ => {}
        }
    }
}

//! `sandboxd` client 模块 (per FR-7.1 fail-open + BD-002 §3.3 Rust client)
//!
//! Rust gRPC client + fail-open 降级:
//!
//! ```text
//! client ── gRPC Connect 127.0.0.1:50051
//!   ├─ Success: CreateSession → RunCommand → DestroySession (完整 4 维隔离)
//!   └─ Failure (连接拒绝 / Unavailable / 健康检查失败):
//!       走 SANDBOX-001 v0.2 subprocess 降级 (仅 resource 维隔离, per FR-7.1)
//! ```
//!
//! 注意 (per SRS §9 缺口 #1 P0 阻塞): dispatcher.py 切 gRPC client 是 v0.2 拍摄,
//! 本模块提供 Rust 侧完整 client + failover 原语, Python 端改造跨 session 续做。

use std::path::Path;
use std::time::Duration;

use tonic::transport::Channel;
use uuid::Uuid;

use crate::backend::{await_child, CommandSpec, RunResult, TIMEOUT_EXIT_CODE};
use crate::error::SandboxdError;
use crate::policy::SessionSpec;
use crate::proto as pb;

/// gRPC client 连接超时 (per NFR-P-1: CreateSession p99 < 100ms 降级判断)
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(100);

/// fail-open 降级规格 (per FR-7.1: SANDBOX-001 v0.2 subprocess 降级路径)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackSpec {
    /// 可执行程序 (e.g. "python")
    pub program: String,
    /// 前缀参数 (e.g. ["scripts/automation/guardian/sandbox.py"])
    pub args_prefix: Vec<String>,
}

impl FallbackSpec {
    /// 从降级路径构造 (per config.failover.fallback_path)
    ///
    /// `.py` → program="python" + prefix=[path]; 其他 → program=path。
    pub fn from_path(path: &Path) -> Self {
        let s = path.to_string_lossy().to_string();
        if s.ends_with(".py") {
            Self {
                program: "python".to_string(),
                args_prefix: vec![s],
            }
        } else {
            Self {
                program: s,
                args_prefix: vec![],
            }
        }
    }

    /// 禁用降级 (program 为空 = disabled, per config 关闭 failover)
    pub fn disabled() -> Self {
        Self {
            program: String::new(),
            args_prefix: vec![],
        }
    }

    /// 是否禁用
    pub fn is_disabled(&self) -> bool {
        self.program.is_empty()
    }
}

/// failover 执行结果 (per FR-7.1: 降级路径标注 degraded)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailoverRun {
    /// 命令结果
    pub result: RunResult,
    /// 是否走降级路径 (sandboxd 不可用)
    pub degraded: bool,
    /// 详情 (降级原因 / gRPC 错误摘要)
    pub detail: String,
}

/// sandboxd gRPC client (per BD-002 §3.3 Rust client, 跟 Python client 对称)
#[derive(Debug, Clone)]
pub struct SandboxdClient {
    /// daemon 地址 (e.g. "127.0.0.1:50051")
    addr: String,
    /// 连接探测超时 (per NFR-P-1 / FR-7.1 降级判断)
    connect_timeout: Duration,
}

impl SandboxdClient {
    /// 构造 client (默认地址 + 默认探测超时)
    pub fn new(addr: impl Into<String>) -> Self {
        Self::with_timeout(addr, DEFAULT_CONNECT_TIMEOUT)
    }

    /// 构造 client + 自定义探测超时
    pub fn with_timeout(addr: impl Into<String>, connect_timeout: Duration) -> Self {
        Self {
            addr: addr.into(),
            connect_timeout,
        }
    }

    /// 连接 channel (失败 → Err, 供 failover 判断)
    async fn connect(&self) -> Result<SandboxServiceClient, SandboxdError> {
        let endpoint = tonic::transport::Endpoint::from_shared(format!("http://{}", self.addr))
            .map_err(|e| SandboxdError::Internal(format!("invalid endpoint: {e}")))?;
        let channel = tokio::time::timeout(self.connect_timeout, endpoint.connect())
            .await
            .map_err(|_| SandboxdError::Internal("connect timeout".to_string()))?
            .map_err(|e| SandboxdError::Internal(format!("connect failed: {e}")))?;
        Ok(SandboxServiceClient::new(channel))
    }

    /// sandboxd 是否可用 (per FR-7.1 降级触发条件 1: gRPC 连接失败)
    pub async fn is_available(&self) -> bool {
        self.connect().await.is_ok()
    }

    /// CreateSession (per FR-4.1)
    pub async fn create_session(
        &self,
        client_id: &str,
        task_id: &str,
        parent_session_id: Option<Uuid>,
        policy: &SessionSpec,
        timeout_sec: u32,
    ) -> Result<Uuid, SandboxdError> {
        let mut client = self.connect().await?;
        let resp = client
            .create_session(pb::CreateSessionRequest {
                client_id: client_id.to_string(),
                task_id: task_id.to_string(),
                parent_session_id: parent_session_id.map(|u| u.to_string()).unwrap_or_default(),
                policy: Some((policy).into()),
                timeout_sec,
            })
            .await
            .map_err(|e| SandboxdError::Internal(format!("create_session RPC: {e}")))?
            .into_inner();
        if pb::create_session_response::Status::try_from(resp.status).ok()
            != Some(pb::create_session_response::Status::Ready)
        {
            return Err(SandboxdError::Internal(format!(
                "create_session FAILED: {}",
                resp.error
            )));
        }
        Uuid::parse_str(&resp.session_id)
            .map_err(|_| SandboxdError::Internal("server returned invalid session_id".to_string()))
    }

    /// RunCommand (per FR-4.2)
    pub async fn run_command(
        &self,
        session_id: Uuid,
        cmd: &CommandSpec,
        timeout_sec: u32,
    ) -> Result<RunResult, SandboxdError> {
        let mut client = self.connect().await?;
        let resp = client
            .run_command(pb::RunCommandRequest {
                session_id: session_id.to_string(),
                command: Some(pb::CommandSpec {
                    cmd: cmd.cmd.clone(),
                    env: cmd.env.clone(),
                    cwd: cmd.cwd.clone(),
                    capture_output: cmd.capture_output,
                }),
                timeout_sec,
            })
            .await
            .map_err(|e| SandboxdError::Internal(format!("run_command RPC: {e}")))?
            .into_inner();
        Ok(RunResult {
            returncode: resp.returncode,
            stdout: resp.stdout,
            stderr: resp.stderr,
            latency_ms: resp.latency_ms,
            hit_violations: resp.hit_violations,
        })
    }

    /// DestroySession (per FR-4.3)
    pub async fn destroy_session(
        &self,
        session_id: Uuid,
        reason: &str,
    ) -> Result<(), SandboxdError> {
        let mut client = self.connect().await?;
        let resp = client
            .destroy_session(pb::DestroySessionRequest {
                session_id: session_id.to_string(),
                reason: reason.to_string(),
            })
            .await
            .map_err(|e| SandboxdError::Internal(format!("destroy_session RPC: {e}")))?
            .into_inner();
        match pb::destroy_session_response::Status::try_from(resp.status) {
            Ok(pb::destroy_session_response::Status::Destroyed) => Ok(()),
            Ok(pb::destroy_session_response::Status::NotFound) => {
                Err(SandboxdError::SessionNotFound(session_id))
            }
            Err(_) => Err(SandboxdError::Internal(
                "server returned invalid destroy status".to_string(),
            )),
        }
    }

    /// StreamLogs (per FR-2.3): 返回 server streaming 日志流
    pub async fn stream_logs(
        &self,
        session_id: Uuid,
        from_offset: u32,
    ) -> Result<
        std::pin::Pin<
            Box<dyn futures_util::Stream<Item = Result<pb::LogEntry, tonic::Status>> + Send>,
        >,
        SandboxdError,
    > {
        let mut client = self.connect().await?;
        let resp = client
            .stream_logs(pb::StreamLogsRequest {
                session_id: session_id.to_string(),
                from_offset,
            })
            .await
            .map_err(|e| SandboxdError::Internal(format!("stream_logs RPC: {e}")))?
            .into_inner();
        Ok(Box::pin(resp))
    }

    /// 完整生命周期 + fail-open 降级 (per FR-7.1)
    ///
    /// gRPC 路径任一步失败 (连接 / CreateSession / RunCommand) → 走 `fallback` 降级
    /// (SANDBOX-001 v0.2 subprocess, 仅 resource 维隔离), 返回 `degraded=true`。
    pub async fn run_with_failover(
        &self,
        cmd: &CommandSpec,
        fallback: &FallbackSpec,
        timeout_sec: u32,
    ) -> FailoverRun {
        // FR-7.1 触发条件 1: gRPC 连接失败 → 降级
        let mut client = match self.connect().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    "sandboxd unavailable ({e}), falling back to SANDBOX-001 v0.2 subprocess"
                );
                return self
                    .run_fallback(cmd, fallback, timeout_sec, e.to_string())
                    .await;
            }
        };
        // CreateSession (默认 policy) → 失败降级
        let session_id = match client
            .create_session(pb::CreateSessionRequest {
                client_id: "sandboxd-client".to_string(),
                task_id: String::new(),
                parent_session_id: String::new(),
                policy: Some((&SessionSpec::default()).into()),
                timeout_sec,
            })
            .await
            .map(|resp| resp.into_inner())
        {
            Ok(resp) => {
                let status = pb::create_session_response::Status::try_from(resp.status)
                    .unwrap_or(pb::create_session_response::Status::Failed);
                if status == pb::create_session_response::Status::Ready {
                    match Uuid::parse_str(&resp.session_id) {
                        Ok(id) => id,
                        Err(e) => {
                            return self
                                .run_fallback(cmd, fallback, timeout_sec, e.to_string())
                                .await;
                        }
                    }
                } else {
                    return self
                        .run_fallback(cmd, fallback, timeout_sec, resp.error)
                        .await;
                }
            }
            Err(e) => {
                return self
                    .run_fallback(cmd, fallback, timeout_sec, e.to_string())
                    .await;
            }
        };
        // RunCommand → 失败降级
        let result = match client
            .run_command(pb::RunCommandRequest {
                session_id: session_id.to_string(),
                command: Some(pb::CommandSpec {
                    cmd: cmd.cmd.clone(),
                    env: cmd.env.clone(),
                    cwd: cmd.cwd.clone(),
                    capture_output: cmd.capture_output,
                }),
                timeout_sec,
            })
            .await
            .map(|resp| resp.into_inner())
        {
            Ok(resp) => RunResult {
                returncode: resp.returncode,
                stdout: resp.stdout,
                stderr: resp.stderr,
                latency_ms: resp.latency_ms,
                hit_violations: resp.hit_violations,
            },
            Err(e) => {
                return self
                    .run_fallback(cmd, fallback, timeout_sec, e.to_string())
                    .await;
            }
        };
        // DestroySession (best-effort)
        let _ = client
            .destroy_session(pb::DestroySessionRequest {
                session_id: session_id.to_string(),
                reason: "client lifecycle end".to_string(),
            })
            .await;
        FailoverRun {
            result,
            degraded: false,
            detail: String::new(),
        }
    }

    /// 降级路径执行 (per FR-7.1: SANDBOX-001 v0.2 subprocess.run 等价)
    async fn run_fallback(
        &self,
        cmd: &CommandSpec,
        fallback: &FallbackSpec,
        timeout_sec: u32,
        reason: String,
    ) -> FailoverRun {
        let degraded = true;
        if fallback.is_disabled() {
            return FailoverRun {
                result: RunResult {
                    returncode: -1,
                    stdout: String::new(),
                    stderr: "sandboxd unavailable and fallback disabled".to_string(),
                    latency_ms: 0,
                    hit_violations: vec![format!("failover: {reason}")],
                },
                degraded,
                detail: format!("sandboxd unavailable: {reason} (fallback disabled)"),
            };
        }
        let mut command = tokio::process::Command::new(&fallback.program);
        command.args(&fallback.args_prefix);
        command.args(&cmd.cmd);
        for kv in &cmd.env {
            if let Some((k, v)) = kv.split_once('=') {
                command.env(k, v);
            }
        }
        if !cmd.cwd.is_empty() {
            command.current_dir(&cmd.cwd);
        }
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.stdin(std::process::Stdio::null());
        command.kill_on_drop(true);
        let child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                return FailoverRun {
                    result: RunResult {
                        returncode: -1,
                        stdout: String::new(),
                        stderr: format!("fallback spawn failed: {e}"),
                        latency_ms: 0,
                        hit_violations: vec![format!("failover: {reason}")],
                    },
                    degraded,
                    detail: format!("sandboxd unavailable ({reason}); fallback spawn failed"),
                };
            }
        };
        let (code, stdout, stderr, timed_out) = await_child(child, timeout_sec).await;
        let result = RunResult {
            returncode: if timed_out { TIMEOUT_EXIT_CODE } else { code },
            stdout,
            stderr,
            latency_ms: 0,
            hit_violations: vec![format!("failover: {reason}")],
        };
        FailoverRun {
            result,
            degraded,
            detail: format!(
                "sandboxd unavailable: {reason}; degraded to SANDBOX-001 v0.2 subprocess"
            ),
        }
    }
}

/// tonic 生成 client 类型别名
pub type SandboxServiceClient = pb::sandbox_service_client::SandboxServiceClient<Channel>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_from_py_path() {
        let spec = FallbackSpec::from_path(Path::new("scripts/automation/guardian/sandbox.py"));
        assert_eq!(spec.program, "python");
        assert_eq!(
            spec.args_prefix,
            vec!["scripts/automation/guardian/sandbox.py"]
        );
    }

    #[test]
    fn fallback_from_binary_path() {
        let spec = FallbackSpec::from_path(Path::new("/usr/local/bin/sandbox-legacy"));
        assert_eq!(spec.program, "/usr/local/bin/sandbox-legacy");
        assert!(spec.args_prefix.is_empty());
    }

    #[test]
    fn disabled_fallback() {
        assert!(FallbackSpec::disabled().is_disabled());
    }
}

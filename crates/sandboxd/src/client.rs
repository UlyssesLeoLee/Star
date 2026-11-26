/*
@cypher schema=1 source_sha256=14da1f3712058e25aa42e6518c38f8ffccaa3b80d251f0dca7b22803dbc8a319
MERGE (self:File {path:"crates/sandboxd/src/client.rs"})
@endcypher
*/
//! `sandboxd` client 模块 (per FR-7 fail-closed + BD-002 §3.3 Rust client)
//!
//! Rust gRPC client + **fail-closed 安全阻断**:
//!
//! ```text
//! client ── gRPC Connect 127.0.0.1:50051
//!   ├─ Success: CreateSession → RunCommand → DestroySession (完整 4 维隔离)
//!   └─ Failure (连接拒绝 / Unavailable / 健康检查失败 / 策略不满足 / 审计不可用):
//!       **BLOCK** (不降级, per DD-SANDBOX-002 v0.1.2 FR-7 fail-closed)
//! ```
//!
//! ⚠️ 旧版 fail-open 降级路径 (SANDBOX-001 v0.2 subprocess) 已废弃，
//! 仅保留 FallbackSpec 配置结构供 migration 兼容，不再作为产品路径。

use std::path::Path;
use std::time::Duration;

use tonic::transport::Channel;
use uuid::Uuid;

use crate::backend::{CommandSpec, RunResult};
use crate::error::SandboxdError;
use crate::policy::SessionSpec;
use crate::proto as pb;

/// gRPC client 连接超时 (per NFR-P-1: CreateSession p99 < 100ms 降级判断)
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(100);

/// 兼容配置结构 (per FR-7: 保留供 migration 兼容，不再作为产品路径)
/// 
/// ⚠️ 旧版 fail-open 降级规格 (SANDBOX-001 v0.2 subprocess) 已废弃。
/// 生产环境 **fail-closed**: sandboxd 不可用 → BLOCK，不降级。
/// 此结构仅保留字段以便配置文件反序列化不破坏。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackSpec {
    /// 可执行程序 (e.g. "python") — 仅供 migration 兼容
    pub program: String,
    /// 前缀参数 (e.g. ["scripts/automation/guardian/sandbox.py"]) — 仅供 migration 兼容
    pub args_prefix: Vec<String>,
}

impl FallbackSpec {
    /// 从降级路径构造 (per config.failover.fallback_path) — 仅供 migration 兼容
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

    /// 禁用降级 (program 为空 = disabled, per config 关闭 failover) — 仅供 migration 兼容
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

/// failover 执行结果 (per FR-7: 生产执行阻断，不降级到未验证 subprocess)
/// 
/// 生产执行阻断语义 (per DD-SANDBOX-002 v0.1.2 FR-7 fail-closed):
/// - sandboxd gRPC 连接/健康检查失败 → BLOCK (不降级)
/// - CreateSession 失败（策略不满足、资源不足、audit 不可用） → BLOCK
/// - RunCommand 失败 → BLOCK
/// 
/// ⚠️ 旧版 fail-open 降级路径 (SANDBOX-001 v0.2 subprocess) 已废弃，
/// 仅保留配置结构供 migration 兼容，不再作为产品路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailoverRun {
    /// 命令结果 (仅在未 BLOCK 时有效)
    pub result: Option<RunResult>,
    /// 是否因降级路径产生 (保留字段，永远为 false)
    pub degraded: bool,
    /// 降级/失败原因 (BLOCK 时包含诊断信息)
    pub reason: String,
}

impl FailoverRun {
    /// 成功执行
    pub fn success(result: RunResult) -> Self {
        Self {
            result: Some(result),
            degraded: false,
            reason: String::new(),
        }
    }

    /// 安全阻断 (fail-closed)
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self {
            result: None,
            degraded: false,
            reason: reason.into(),
        }
    }
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

    /// 完整生命周期 + fail-closed 安全阻断 (per FR-7)
        ///
        /// 生产执行阻断语义 (per DD-SANDBOX-002 v0.1.2 FR-7 fail-closed):
        /// - sandboxd gRPC 连接/健康检查失败 → BLOCK (不降级)
        /// - CreateSession 失败（策略不满足、资源不足、audit 不可用） → BLOCK
        /// - RunCommand 失败 → BLOCK
        /// 
        /// ⚠️ 旧版 fail-open 降级路径 (SANDBOX-001 v0.2 subprocess) 已废弃，
        /// 仅保留 FallbackSpec 配置结构供 migration 兼容，不再作为产品路径。
        pub async fn run_with_failover(
            &self,
            cmd: &CommandSpec,
            _fallback: &FallbackSpec,
            timeout_sec: u32,
        ) -> FailoverRun {
            // FR-7 触发条件 1: gRPC 连接失败 → BLOCK
            let mut client = match self.connect().await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!(
                        "sandboxd unavailable ({e}), BLOCK (fail-closed: no fallback)"
                    );
                    return FailoverRun::blocked(format!("sandboxd connect failed: {e}"));
                }
            };
            // CreateSession (默认 policy) → 失败 BLOCK
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
                                return FailoverRun::blocked(format!("session ID parse failed: {e}"));
                            }
                        }
                    } else {
                        return FailoverRun::blocked(format!("CreateSession failed: {}", resp.error));
                    }
                }
                Err(e) => {
                    return FailoverRun::blocked(format!("CreateSession RPC failed: {e}"));
                }
            };
            // RunCommand → 失败 BLOCK
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
                    return FailoverRun::blocked(format!("RunCommand RPC failed: {e}"));
                }
            };
            // DestroySession (best-effort)
            let _ = client
                .destroy_session(pb::DestroySessionRequest {
                    session_id: session_id.to_string(),
                    reason: "client lifecycle end".to_string(),
                })
                .await;
            FailoverRun::success(result)
        }

        /// 兼容旧版降级路径 (per FR-7: 仅供 migration 兼容，产品路径不再使用)
            /// 
            /// ⚠️ 生产环境 **fail-closed**: 此方法不应被调用。
            /// 保留仅为配置结构反序列化兼容。
            async fn run_fallback(
                &self,
                _cmd: &CommandSpec,
                _fallback: &FallbackSpec,
                _timeout_sec: u32,
                reason: String,
            ) -> FailoverRun {
                tracing::error!(
                    "run_fallback called in production (should not happen), reason: {}",
                    reason
                );
                FailoverRun::blocked(format!("fallback disabled (fail-closed): {reason}"))
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

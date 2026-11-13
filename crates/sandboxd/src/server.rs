/*
@cypher schema=1 source_sha256=b17ba1c3b91665b34ed74bb5c01c4aa893a90b5a5b925e854fa0840d91b468d9
MERGE (self:File {path:"crates/sandboxd/src/server.rs"})
@endcypher
*/
//! `sandboxd` gRPC server 模块 (per FR-2, DD-SANDBOX-002 §2.2)
//!
//! - `SandboxServiceImpl`: tonic 0.12 生成的 `SandboxService` 4 RPC 实现
//! - `serve`: gRPC (grpc_port) + axum HTTP (/metrics + /healthz, http_port) 同进程
//! - 优雅停机: ctrl_c / SIGTERM → orchestrator.shutdown (per FR-1.3)

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::backend::CommandSpec;
use crate::config::SandboxdConfig;
use crate::error::SandboxdError;
use crate::health;
use crate::metrics::SandboxdMetrics;
use crate::orchestrator::{LogStream, Orchestrator};
use crate::policy::SessionSpec;
use crate::proto as pb;

/// tonic gRPC service 实现 (per DD §2.2)
#[derive(Clone)]
pub struct SandboxServiceImpl {
    /// session 池 orchestrator
    pub orchestrator: Arc<Orchestrator>,
    /// Prometheus 指标
    pub metrics: Arc<SandboxdMetrics>,
}

/// 解析 UUID 字符串 (非法 → invalid_argument, per DD §2.2)
fn parse_session_id(s: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument("invalid session_id"))
}

#[tonic::async_trait]
impl pb::sandbox_service_server::SandboxService for SandboxServiceImpl {
    /// CreateSession (per FR-4.1): 策略校验 + 池化 + audit fail-closed
    async fn create_session(
        &self,
        request: Request<pb::CreateSessionRequest>,
    ) -> Result<Response<pb::CreateSessionResponse>, Status> {
        let _timer = self
            .metrics
            .rpc_duration_seconds
            .with_label_values(&["create_session"])
            .start_timer();
        let req = request.into_inner();
        let policy_proto = req
            .policy
            .ok_or_else(|| Status::invalid_argument("policy missing"))?;
        let policy = SessionSpec::from(&policy_proto);
        let parent = if req.parent_session_id.is_empty() {
            None
        } else {
            Some(parse_session_id(&req.parent_session_id)?)
        };
        match self
            .orchestrator
            .create_session(req.client_id, req.task_id, parent, policy, req.timeout_sec)
            .await
        {
            Ok(session_id) => Ok(Response::new(pb::CreateSessionResponse {
                session_id: session_id.to_string(),
                status: pb::create_session_response::Status::Ready as i32,
                error: String::new(),
            })),
            // per FR-4.1: 失败 → FAILED + error (不抛 gRPC error, 客户端可读 error 字段)
            Err(e) => Ok(Response::new(pb::CreateSessionResponse {
                session_id: String::new(),
                status: pb::create_session_response::Status::Failed as i32,
                error: e.to_string(),
            })),
        }
    }

    /// RunCommand (per FR-4.2): session 内跑命令 + 命中违规不阻断 (fail-open)
    async fn run_command(
        &self,
        request: Request<pb::RunCommandRequest>,
    ) -> Result<Response<pb::RunCommandResponse>, Status> {
        let _timer = self
            .metrics
            .rpc_duration_seconds
            .with_label_values(&["run_command"])
            .start_timer();
        let req = request.into_inner();
        let session_id = parse_session_id(&req.session_id)?;
        let command = req
            .command
            .ok_or_else(|| Status::invalid_argument("command missing"))?;
        let cmd = CommandSpec::from(&command);
        if cmd.cmd.is_empty() {
            return Err(Status::invalid_argument("command argv must not be empty"));
        }
        match self
            .orchestrator
            .run_command(session_id, &cmd, req.timeout_sec)
            .await
        {
            Ok(r) => Ok(Response::new(pb::RunCommandResponse {
                returncode: r.returncode,
                stdout: r.stdout,
                stderr: r.stderr,
                latency_ms: r.latency_ms,
                hit_violations: r.hit_violations,
            })),
            Err(SandboxdError::SessionNotFound(_)) => Err(Status::not_found("session not found")),
            Err(e) => Err(e.into()),
        }
    }

    /// DestroySession (per FR-4.3): 级联 kill + 回收, NotFound → NOT_FOUND
    async fn destroy_session(
        &self,
        request: Request<pb::DestroySessionRequest>,
    ) -> Result<Response<pb::DestroySessionResponse>, Status> {
        let _timer = self
            .metrics
            .rpc_duration_seconds
            .with_label_values(&["destroy_session"])
            .start_timer();
        let req = request.into_inner();
        let session_id = parse_session_id(&req.session_id)?;
        match self
            .orchestrator
            .destroy_session(session_id, &req.reason)
            .await
        {
            Ok(()) => Ok(Response::new(pb::DestroySessionResponse {
                status: pb::destroy_session_response::Status::Destroyed as i32,
            })),
            Err(SandboxdError::SessionNotFound(_)) => {
                Ok(Response::new(pb::DestroySessionResponse {
                    status: pb::destroy_session_response::Status::NotFound as i32,
                }))
            }
            Err(e) => Err(e.into()),
        }
    }

    type StreamLogsStream = LogStream;

    /// StreamLogs (per FR-2.3): 环形缓冲重放 + 实时推送
    async fn stream_logs(
        &self,
        request: Request<pb::StreamLogsRequest>,
    ) -> Result<Response<Self::StreamLogsStream>, Status> {
        let _timer = self
            .metrics
            .rpc_duration_seconds
            .with_label_values(&["stream_logs"])
            .start_timer();
        let req = request.into_inner();
        let session_id = parse_session_id(&req.session_id)?;
        match self
            .orchestrator
            .stream_logs(session_id, req.from_offset as u64)
        {
            Ok(stream) => Ok(Response::new(stream)),
            Err(SandboxdError::SessionNotFound(_)) => Err(Status::not_found("session not found")),
            Err(e) => Err(e.into()),
        }
    }
}

/// 组装 gRPC server (供 serve / 集成测试用, 端口 0 = 随机)
pub fn grpc_server(svc: SandboxServiceImpl) -> tonic::transport::server::Router {
    tonic::transport::Server::builder()
        .add_service(pb::sandbox_service_server::SandboxServiceServer::new(svc))
}

/// 启动 daemon: gRPC + HTTP (/metrics + /healthz) 同进程, ctrl_c 优雅停机 (per FR-1.1 / FR-1.3)
pub async fn serve(
    config: SandboxdConfig,
    orchestrator: Arc<Orchestrator>,
    metrics: Arc<SandboxdMetrics>,
) -> anyhow::Result<()> {
    let grpc_addr = config.grpc_addr();
    let http_addr = config.http_addr();

    let state = health::AppState {
        started_at: orchestrator.started_at,
        orchestrator: Arc::clone(&orchestrator),
        metrics: Arc::clone(&metrics),
        version: crate::VERSION,
    };

    // HTTP (/metrics + /healthz) — 独立 listener (tonic 0.12 无 accept_http1, 偏差见 config.rs)
    let http_listener = tokio::net::TcpListener::bind(&http_addr).await?;
    let http_handle = tokio::spawn(async move {
        axum::serve(http_listener, health::router(state))
            .await
            .map_err(|e| tracing::error!("http server error: {e}"))
    });

    // gRPC
    let svc = SandboxServiceImpl {
        orchestrator: Arc::clone(&orchestrator),
        metrics: Arc::clone(&metrics),
    };
    let grpc_listener = tokio::net::TcpListener::bind(&grpc_addr).await?;
    tracing::info!("sandboxd listening: gRPC {grpc_addr}, HTTP {http_addr}");

    let grpc_handle = tokio::spawn(async move {
        grpc_server(svc)
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
                shutdown_signal(),
            )
            .await
            .map_err(|e| tracing::error!("grpc server error: {e}"))
    });

    let _ = grpc_handle.await;
    // FR-1.3: 优雅停机 (等在飞命令 → grace → 强制销毁残留)
    let grace = std::time::Duration::from_secs(config.server.grace_shutdown_sec);
    orchestrator.shutdown(grace).await;
    http_handle.abort();
    Ok(())
}

/// ctrl_c / SIGTERM shutdown future (per FR-1.3 接收 SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}

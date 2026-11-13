/*
@cypher schema=1 source_sha256=e037c5234ce7ff07e10cb8acbe086ecded86953fd07507228f670bffc9f402c5
MERGE (self:File {path:"crates/sandboxd/tests/integration/test_grpc_create_run_destroy.rs"})
@endcypher
*/
//! tests/integration/test_grpc_create_run_destroy.rs — gRPC 4 RPC 端到端 (per DD §5.2: 8 TC)
//!
//! 进程内启动 tonic server (127.0.0.1:0) + 真实 client 全流程:
//! CreateSession / RunCommand / DestroySession / StreamLogs + 真实子进程执行 (真实 backend)。

use std::sync::Arc;

use futures_util::StreamExt;
use sandboxd::audit::{AuditSink, CompositeAuditSink, JsonLinesAuditSink};
use sandboxd::backend::CommandSpec;
use sandboxd::client::SandboxdClient;
use sandboxd::metrics::SandboxdMetrics;
use sandboxd::orchestrator::Orchestrator;
use sandboxd::policy::SessionSpec;
use sandboxd::server::{grpc_server, SandboxServiceImpl};

/// 平台 echo 命令
fn echo_cmd() -> CommandSpec {
    #[cfg(windows)]
    {
        CommandSpec::new(
            vec![
                "cmd".to_string(),
                "/C".to_string(),
                "echo hello-sandbox".to_string(),
            ],
            vec![],
            String::new(),
            true,
        )
    }
    #[cfg(not(windows))]
    {
        CommandSpec::new(
            vec![
                "sh".to_string(),
                "-c".to_string(),
                "echo hello-sandbox".to_string(),
            ],
            vec![],
            String::new(),
            true,
        )
    }
}

/// 启动进程内 gRPC server, 返回 (addr, orchestrator, shutdown tx)
async fn spawn_server() -> (String, Arc<Orchestrator>, tokio::sync::oneshot::Sender<()>) {
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let audit: Arc<dyn AuditSink> = Arc::new(CompositeAuditSink::new(vec![Box::new(
        JsonLinesAuditSink::new(audit_path.to_str().unwrap()).unwrap(),
    )]));
    // 保持 tempdir 存活到测试进程结束 (audit 文件供 e2e 检查)
    let _keep = dir.keep();
    let metrics = Arc::new(SandboxdMetrics::new().unwrap());
    let orchestrator = Arc::new(Orchestrator::with_defaults(
        10,
        1800,
        audit,
        Arc::clone(&metrics),
    ));
    let svc = SandboxServiceImpl {
        orchestrator: Arc::clone(&orchestrator),
        metrics,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        grpc_server(svc)
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                async move {
                    let _ = rx.await;
                },
            )
            .await
            .unwrap();
    });
    (addr, orchestrator, tx)
}

#[tokio::test]
async fn create_run_destroy_roundtrip() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);

    // is_available (per FR-7.1 降级判断)
    assert!(client.is_available().await);

    // CreateSession
    let session_id = client
        .create_session("dispatcher.py", "task-42", None, &SessionSpec::default(), 0)
        .await
        .unwrap();
    assert!(!session_id.is_nil());

    // RunCommand (真实子进程)
    let result = client
        .run_command(session_id, &echo_cmd(), 30)
        .await
        .unwrap();
    assert_eq!(result.returncode, 0, "stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("hello-sandbox"),
        "stdout: {}",
        result.stdout
    );
    assert!(result.latency_ms > 0);

    // DestroySession
    client
        .destroy_session(session_id, "test done")
        .await
        .unwrap();

    // 二次 destroy → NotFound
    let err = client
        .destroy_session(session_id, "again")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        sandboxd::error::SandboxdError::SessionNotFound(_)
    ));
}

#[tokio::test]
async fn run_command_unknown_session_not_found() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    let err = client
        .run_command(uuid::Uuid::new_v4(), &echo_cmd(), 30)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

#[tokio::test]
async fn create_session_invalid_policy_failed() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    let mut policy = SessionSpec::default();
    policy.resource.cpu_percent = 999;
    let err = client
        .create_session("c", "t", None, &policy, 0)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("FAILED"), "{err}");
}

#[tokio::test]
async fn stream_logs_replays_entries() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    let session_id = client
        .create_session(
            "dispatcher.py",
            "task-stream",
            None,
            &SessionSpec::default(),
            0,
        )
        .await
        .unwrap();
    // 跑 2 条命令 → 4 条 log (ProcessStart/Exit × 2)
    client
        .run_command(session_id, &echo_cmd(), 30)
        .await
        .unwrap();
    client
        .run_command(session_id, &echo_cmd(), 30)
        .await
        .unwrap();

    // StreamLogs from_offset=0 → 重放 + 取 4 条
    let stream = client.stream_logs(session_id, 0).await.unwrap();
    let entries: Vec<_> = stream.take(4).map(|r| r.unwrap()).collect().await;
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].offset, 0);
    assert_eq!(entries[1].offset, 1);
    let types: Vec<i32> = entries.iter().map(|e| e.event_type).collect();
    assert!(types.contains(&(sandboxd::proto::log_entry::EventType::ProcessStart as i32)));
    assert!(types.contains(&(sandboxd::proto::log_entry::EventType::ProcessExit as i32)));

    client.destroy_session(session_id, "done").await.unwrap();
}

#[tokio::test]
async fn stream_logs_unknown_session_errors() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    match client.stream_logs(uuid::Uuid::new_v4(), 0).await {
        Ok(_) => panic!("expected error for unknown session"),
        Err(e) => assert!(e.to_string().contains("not found"), "{e}"),
    }
}

#[tokio::test]
async fn run_command_empty_argv_rejected() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    let session_id = client
        .create_session("c", "t", None, &SessionSpec::default(), 0)
        .await
        .unwrap();
    let err = client
        .run_command(
            session_id,
            &CommandSpec::new(vec![], vec![], String::new(), true),
            30,
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("argv"), "{err}");
}

#[tokio::test]
async fn env_values_never_leak_to_audit() {
    // 守门 #5: env 内容不打印 / 不进 audit; KEY 名可进 (仅 KEY=VALUE 引用)
    let (addr, orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    let session_id = client
        .create_session("c", "t", None, &SessionSpec::default(), 0)
        .await
        .unwrap();
    let mut cmd = echo_cmd();
    cmd.env = vec!["SUPER_SECRET=value-should-not-leak".to_string()];
    client.run_command(session_id, &cmd, 30).await.unwrap();
    // 检查 session log ring: details 里只有 env KEYS, 无 value
    let logs = orch.stream_logs(session_id, 0).unwrap();
    let entries: Vec<_> = tokio_stream::StreamExt::take(logs, 2)
        .map(|r| r.unwrap())
        .collect()
        .await;
    let start = &entries[0];
    assert!(start.details.contains("env_keys"));
    assert!(
        !start.details.contains("value-should-not-leak"),
        "env value leaked: {}",
        start.details
    );
    // KEY 名允许 (per 守门 #5 仅 KEY=VALUE 引用)
    assert!(start.details.contains("SUPER_SECRET"));
}

#[tokio::test]
async fn destroy_returns_status_for_missing() {
    let (addr, _orch, _tx) = spawn_server().await;
    let client = SandboxdClient::new(addr);
    // client 层 NotFound → Err(SessionNotFound); gRPC 层 status=NOT_FOUND 已在 client 内映射
    let err = client
        .destroy_session(uuid::Uuid::new_v4(), "x")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        sandboxd::error::SandboxdError::SessionNotFound(_)
    ));
}

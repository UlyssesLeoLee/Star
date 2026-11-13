//! tests/integration/test_failover_v02_subprocess.rs — SANDBOX-001 v0.2 降级路径 (per DD §5.2: 4 TC)
//!
//! FR-7.1: sandboxd 不可用 → 走 SANDBOX-001 v0.2 subprocess 降级 (仅 resource 维隔离)。

use sandboxd::backend::CommandSpec;
use sandboxd::client::{FallbackSpec, SandboxdClient};

/// 拿一个必然无监听的端口
fn dead_addr() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    drop(listener); // 端口释放, 无监听 → 连接拒绝
    addr
}

/// 平台 fallback: 直接跑 echo (fake 降级脚本, 替代 sandbox.py)
fn fallback_spec() -> FallbackSpec {
    #[cfg(windows)]
    {
        FallbackSpec {
            program: "cmd".to_string(),
            args_prefix: vec!["/C".to_string()],
        }
    }
    #[cfg(not(windows))]
    {
        FallbackSpec {
            program: "sh".to_string(),
            args_prefix: vec!["-c".to_string()],
        }
    }
}

#[tokio::test]
async fn unavailable_daemon_detected() {
    let client = SandboxdClient::new(dead_addr());
    // FR-7.1 触发条件 1: gRPC 连接失败 → is_available = false
    assert!(!client.is_available().await);
}

#[tokio::test]
async fn failover_runs_subprocess() {
    let client = SandboxdClient::new(dead_addr());
    let cmd = CommandSpec::new(
        vec!["echo".to_string(), "fallback-ok".to_string()],
        vec![],
        String::new(),
        true,
    );
    let run = client.run_with_failover(&cmd, &fallback_spec(), 30).await;
    assert!(run.degraded, "expected degraded path");
    assert!(
        run.result.stdout.contains("fallback-ok"),
        "stdout: {} stderr: {}",
        run.result.stdout,
        run.result.stderr
    );
    assert_eq!(run.result.returncode, 0);
    // 降级违规标注 (per FR-7.1 告警联动)
    assert!(run
        .result
        .hit_violations
        .iter()
        .any(|v| v.starts_with("failover:")));
    assert!(run.detail.contains("degraded"));
}

#[tokio::test]
async fn fallback_disabled_returns_error_result() {
    let client = SandboxdClient::new(dead_addr());
    let cmd = CommandSpec::new(
        vec!["echo".to_string(), "x".to_string()],
        vec![],
        String::new(),
        true,
    );
    let run = client
        .run_with_failover(&cmd, &FallbackSpec::disabled(), 30)
        .await;
    assert!(run.degraded);
    assert_eq!(run.result.returncode, -1);
    assert!(run.result.stderr.contains("disabled"));
}

#[tokio::test]
async fn fallback_respects_command_timeout() {
    // 降级路径同样受命令级超时约束 (per FR-4.2 派生)
    let client = SandboxdClient::new(dead_addr());
    #[cfg(windows)]
    let cmd = CommandSpec::new(
        vec![
            "ping".to_string(),
            "-n".to_string(),
            "30".to_string(),
            "127.0.0.1".to_string(),
        ],
        vec![],
        String::new(),
        true,
    );
    #[cfg(not(windows))]
    let cmd = CommandSpec::new(
        vec!["sleep".to_string(), "30".to_string()],
        vec![],
        String::new(),
        true,
    );
    let run = client.run_with_failover(&cmd, &fallback_spec(), 1).await;
    assert!(run.degraded);
    assert_eq!(run.result.returncode, sandboxd::backend::TIMEOUT_EXIT_CODE);
}

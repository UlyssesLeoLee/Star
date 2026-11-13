//! tests/integration/test_backend_smoke.rs — 真实平台 backend 冒烟 (per FR-5)
//!
//! Windows: Job Objects (win32job) 真实子进程 + 超时 + 级联 kill + capability 拦截。
//! Linux/macOS: 对应 backend 的基本 run 路径 (rootless 环境自动降级 PolicyAuditOnly)。

use std::sync::Arc;
use std::time::Duration;

use sandboxd::backend::{
    Backend, BackendFactory, BackendHandle, CommandSpec, DefaultBackendFactory, TIMEOUT_EXIT_CODE,
};
use sandboxd::error::SandboxdError;
use sandboxd::policy::SessionSpec;

/// 平台 echo 命令
fn echo_cmd() -> CommandSpec {
    #[cfg(windows)]
    {
        CommandSpec::new(
            vec![
                "cmd".to_string(),
                "/C".to_string(),
                "echo backend-smoke".to_string(),
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
                "echo backend-smoke".to_string(),
            ],
            vec![],
            String::new(),
            true,
        )
    }
}

/// 平台长命令 (超时测试)
fn long_cmd() -> CommandSpec {
    #[cfg(windows)]
    {
        CommandSpec::new(
            vec![
                "ping".to_string(),
                "-n".to_string(),
                "60".to_string(),
                "127.0.0.1".to_string(),
            ],
            vec![],
            String::new(),
            true,
        )
    }
    #[cfg(not(windows))]
    {
        CommandSpec::new(
            vec!["sleep".to_string(), "60".to_string()],
            vec![],
            String::new(),
            true,
        )
    }
}

/// 平台提权工具名 (capability 拦截测试)
fn escalation_cmd() -> CommandSpec {
    #[cfg(windows)]
    {
        CommandSpec::new(vec!["runas".to_string()], vec![], String::new(), true)
    }
    #[cfg(not(windows))]
    {
        CommandSpec::new(vec!["sudo".to_string()], vec![], String::new(), true)
    }
}

async fn new_handle() -> Arc<dyn BackendHandle> {
    let factory = DefaultBackendFactory;
    let backend = factory.create().unwrap();
    assert!(backend.is_supported());
    let handle = backend
        .apply(&SessionSpec::default(), uuid::Uuid::new_v4())
        .await
        .unwrap();
    Arc::from(handle)
}

#[tokio::test]
async fn run_real_command() {
    let handle = new_handle().await;
    let result = handle.run(&echo_cmd(), 30).await.unwrap();
    assert_eq!(result.returncode, 0, "stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("backend-smoke"),
        "stdout: {}",
        result.stdout
    );
    let health = handle.health().await.unwrap();
    assert!(health.healthy);
    handle.destroy().await.unwrap();
    let health_after = handle.health().await.unwrap();
    assert!(!health_after.healthy);
}

#[tokio::test]
async fn command_timeout_kills_child() {
    let handle = new_handle().await;
    let result = handle.run(&long_cmd(), 1).await.unwrap();
    assert_eq!(result.returncode, TIMEOUT_EXIT_CODE);
    assert!(result
        .hit_violations
        .iter()
        .any(|v| v.starts_with("resource:")));
    handle.destroy().await.unwrap();
}

#[tokio::test]
async fn capability_escalation_blocked() {
    // capability 维: 提权工具 BLOCK (per FR-3.4 drop 语义)
    let handle = new_handle().await;
    let err = handle.run(&escalation_cmd(), 30).await.unwrap_err();
    assert!(
        matches!(err, SandboxdError::CapabilityViolation { .. }),
        "{err}"
    );
    handle.destroy().await.unwrap();
}

#[tokio::test]
async fn fs_cwd_outside_allowlist_audited_not_blocked() {
    // fs 维 fail-open: cwd 命中非 allowlist → hit_violations 返回, 命令照跑 (per FR-4.2)
    let handle = new_handle().await;
    #[cfg(windows)]
    let cwd = std::env::temp_dir().to_string_lossy().to_string(); // 存在且不在默认 allowlist
    #[cfg(not(windows))]
    let cwd = "/etc".to_string(); // 存在且不在默认 allowlist
    let mut cmd = echo_cmd();
    cmd.cwd = cwd.clone();
    let result = handle.run(&cmd, 30).await.unwrap();
    assert_eq!(result.returncode, 0, "stderr: {}", result.stderr);
    assert!(
        result.hit_violations.iter().any(|v| v.starts_with("fs:")),
        "expected fs cwd violation, got {:?}",
        result.hit_violations
    );
    handle.destroy().await.unwrap();
}

#[tokio::test]
async fn destroy_cascade_kills_inflight() {
    // FR-4.3: destroy → 级联 kill session 内所有进程 (Windows: Job handle drop + KILL_ON_JOB_CLOSE)
    // 注: 实测 TerminateJobObject 级联 kill 的子进程退出码为 0 (跟 child.kill()=1 不同),
    //   故断言"完成时间"而非退出码 — 60s ping 若未被杀, task 不可能在 5s 内完成。
    let handle = new_handle().await;
    let h2 = Arc::clone(&handle);
    let task = tokio::spawn(async move { h2.run(&long_cmd(), 0).await });
    // 等子进程真实启动
    tokio::time::sleep(Duration::from_millis(300)).await;
    handle.destroy().await.unwrap();
    // 级联 kill 后 in-flight run 必须在 5s 内返回 (否则说明子进程没被杀)
    let finished = tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .expect("cascade kill failed: run still blocked");
    let result = finished.unwrap().unwrap();
    assert!(
        result.latency_ms < 5000,
        "child ran {}ms — cascade kill did not fire",
        result.latency_ms
    );
}

#[tokio::test]
async fn enforcement_reports_platform() {
    let factory = DefaultBackendFactory;
    let backend = factory.create().unwrap();
    let reports = backend.enforcement();
    assert_eq!(reports.len(), 4, "4 维强制力报告 (per 守门 #11)");
    assert!(reports
        .iter()
        .any(|r| matches!(r.dim, sandboxd::backend::EnforcementDim::Resource)));
    assert!(reports
        .iter()
        .any(|r| matches!(r.dim, sandboxd::backend::EnforcementDim::Network)));
    assert!(reports
        .iter()
        .any(|r| matches!(r.dim, sandboxd::backend::EnforcementDim::Fs)));
    assert!(reports
        .iter()
        .any(|r| matches!(r.dim, sandboxd::backend::EnforcementDim::Capability)));
}

/// 供 cfg 对齐检查: BackendHandle 为 dyn-safe (trait object 已用)
#[allow(dead_code)]
async fn _trait_objects_work(_b: Box<dyn Backend>) {}

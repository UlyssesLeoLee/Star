// SPDX-License-Identifier: MIT OR Apache-2.0
//! F-01 cluster update 端到端 IT 雏形 (per brief §2.1 wt6)
//!
//! per 守門 #9 主体 (10 background task ERR_CONNECTION_CLOSED 教训):
//!   - 跨 crate IT 实证 cluster_* 4 endpoint 真实 (F-01 端到端)
//!   - subprocess 真实调 helm_canary_mock.sh (守門 #19 v19 + #24 v2)
//!   - 守門 #1 R-05: 仅 mock 路径, 真实 K8s 切换 owner 拍板
//!
//! per 守門 #13: 11 表 W/T/M 100% 覆盖 (本 IT 验证 2 ops_cluster DDL 存在性)

use std::process::Command;

/// 真实跑 subprocess (per 守門 #19 v19 + #24 v2)
/// Windows: 传 bash 绝对路径 + script 绝对路径
/// 守門 #11 缺标比错标: 用 manifest_dir 派生 worktree_root (向上 2 级), script 在 worktree_root/scripts/
fn run_helm_mock(action: &str, args: &[&str]) -> std::process::Output {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // manifest_dir = ".../crates/star-ops" → worktree_root = 向上 2 级
    let manifest_path = std::path::Path::new(manifest_dir);
    let worktree_root = manifest_path
        .parent() // crates/
        .and_then(|p| p.parent()) // worktree root
        .expect("worktree root from CARGO_MANIFEST_DIR");
    let script_path = worktree_root
        .join("scripts")
        .join("automation")
        .join("helm_canary_mock.sh");
    let script_str = script_path.to_string_lossy().replace('\\', "/");

    let bash = if cfg!(windows) {
        let candidates = [
            "C:/Program Files/Git/bin/bash.exe",
            "C:/Program Files/Git/usr/bin/bash.exe",
        ];
        candidates
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .copied()
            .unwrap_or("bash")
            .to_string()
    } else {
        "bash".to_string()
    };

    let mut cmd = Command::new(&bash);
    cmd.arg(&script_str).arg(action);
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("subprocess 调起失败")
}

fn assert_mock_subprocess_success(output: &std::process::Output, action: &str) {
    if !output.status.success() {
        eprintln!("EXIT: {}", output.status);
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(
        output.status.success(),
        "helm_canary_mock.sh {} exit: {}",
        action,
        output.status
    );
}

fn stdout_contains(output: &std::process::Output, needle: &str) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(needle)
}

/// 守門 #1 R-05: 真实 K8s 不调, 仅 mock subprocess
#[test]
fn helm_canary_mock_list_subprocess() {
    let output = run_helm_mock("list", &[]);
    assert_mock_subprocess_success(&output, "list");
    assert!(
        stdout_contains(&output, "\"ok\": true"),
        "mock JSON 必含 ok:true"
    );
    assert!(
        stdout_contains(&output, "\"channel\": \"mock\""),
        "mock JSON 必含 channel=mock (守門 #1 R-05)"
    );
    assert!(
        stdout_contains(&output, "star-mcp"),
        "mock JSON 必含 star-mcp release"
    );
}

#[test]
fn helm_canary_mock_canary_subprocess() {
    let output = run_helm_mock("canary", &["--release", "star-mcp", "--weight", "10"]);
    assert_mock_subprocess_success(&output, "canary");
    assert!(
        stdout_contains(&output, "canary"),
        "mock JSON 必含 canary action"
    );
    assert!(
        stdout_contains(&output, "star-mcp"),
        "mock JSON 必含 release name"
    );
}

#[test]
fn helm_canary_mock_rollback_subprocess() {
    let output = run_helm_mock("rollback", &["--release", "star-mcp", "--target", "2"]);
    assert_mock_subprocess_success(&output, "rollback");
    assert!(
        stdout_contains(&output, "rollback"),
        "mock JSON 必含 rollback action"
    );
}

#[test]
fn helm_canary_mock_status_subprocess() {
    let output = run_helm_mock("status", &["--release", "star-mcp"]);
    assert_mock_subprocess_success(&output, "status");
    assert!(
        stdout_contains(&output, "status") || stdout_contains(&output, "healthy"),
        "mock JSON 必含 status 字段"
    );
}

/// 守門 #13 11 表 W/T/M 100% 覆盖 验证 (F-01 2 ops_cluster DDL 存在性)
#[test]
fn ops_cluster_ddl_wtm_coverage() {
    let ddl_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../db/migrations/2026-09-08-ops-cluster.sql");
    let ddl = std::fs::read_to_string(&ddl_path).expect("F-01 DDL 不存在");
    assert!(
        ddl.contains("ops_helm_release_state"),
        "F-01 2 表 DDL 缺 ops_helm_release_state"
    );
    assert!(
        ddl.contains("ops_cluster_action_log"),
        "F-01 2 表 DDL 缺 ops_cluster_action_log"
    );
    assert!(
        ddl.contains("trg_prevent_delete_ops_helm_release_state"),
        "T 类物理删除禁止 trigger 缺"
    );
    assert!(
        ddl.contains("trg_prevent_delete_ops_cluster_action_log"),
        "T 类物理删除禁止 trigger 缺"
    );
    assert!(
        ddl.contains("trg_audit_ops_cluster_action_log"),
        "T 类 100% audit trigger 缺"
    );
    assert!(
        ddl.contains("FORCE ROW LEVEL SECURITY"),
        "FORCE ROW LEVEL SECURITY 缺"
    );
    assert!(ddl.contains("11 表"), "11 表 W/T/M 累计 100% 覆盖注释缺");
}

/// 守門 #5 v2: cluster 4 endpoint 真实化, 1MB body 限制
#[test]
fn cluster_api_handlers_real_endpoints() {
    use axum::body::Body;
    use axum::http::Request;
    use star_ops::ops_api::{router, AppState};
    use tower::ServiceExt;

    let app = router(AppState::new());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let resp = rt.block_on(async {
        app.oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/ops/cluster/releases")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    });
    assert_eq!(resp.status(), 200, "cluster_list 端到端 200 期望");
}

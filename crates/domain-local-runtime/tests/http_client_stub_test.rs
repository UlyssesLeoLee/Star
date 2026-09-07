// SPDX-License-Identifier: MIT OR Apache-2.0
//! RealHttpRuntime::spawn_cli stub 测试 — per OPT-WORKER-01 §3.5
//!
//! 验证 CLI spawn 错误包含结构化字段 (feature / suggestion / p4_phase),
//! 防止 stub 静默退化为通用消息。

use domain_local_runtime::http_client::RealHttpRuntime;
use domain_local_runtime::process::{LocalRuntime, RuntimeError};
use std::collections::HashMap;

#[tokio::test]
async fn real_http_runtime_spawn_cli_returns_structured_not_implemented() {
    let rt = RealHttpRuntime::new();
    let env = HashMap::new();
    let result = rt.spawn_cli("git", &[], &env, ".").await;
    match result {
        Err(RuntimeError::SpawnFailed(msg)) => {
            assert!(
                msg.contains("feature=cli_spawn_in_real_runtime"),
                "msg should contain feature=cli_spawn_in_real_runtime: {msg}"
            );
            assert!(
                msg.contains("suggestion=use DefaultLocalRuntime::with_real_processes()"),
                "msg should contain suggestion: {msg}"
            );
            assert!(
                msg.contains("p4_phase=H.2"),
                "msg should reference p4_phase=H.2: {msg}"
            );
        }
        other => panic!("expected SpawnFailed with structured msg, got {other:?}"),
    }
}

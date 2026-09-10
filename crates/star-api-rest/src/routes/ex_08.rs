// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-08 集成测试 + 性能压测 (Py + Rust + TS) — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.8):
//! - 18 UT (per 8 子项单元测试, 跨 5 module / 4 lib / 1 proc-macro / 1 middleware / 5 metric)
//! - 8 IT (per 8 想定シナリオ S-01..S-08 集成测试)
//! - 12 E2E (4 跨层 场景, 跟 5-LEVEL-FULL §4 E2E 模式一致, Playwright Chromium/Firefox/WebKit)
//! - S-08 1000 并发压测 (per 守门 #19 [P] `e2e_runner.py`)
//! - 守门 #1 4 步实证 (check/fmt/clippy/test --workspace, 单 crate per v25)
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `POST /api/v1/exclusion/test-runs` — 测试运行启动端点
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #19 v19: agent 交互 Python 化.
//! 守门 #1 v25: cargo test 跳过 workspace, 单 crate 测.

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::response::ResponseMeta;

#[derive(Debug, Deserialize)]
pub struct TestRunRequest {
    /// 测试シナリオ ID (per 8 想定 S-01..S-08 + 4 跨层 + 1 压测)
    pub scenario_id: String,
    /// 测试类型: ut / it / e2e / pt
    pub test_kind: String,
    /// 并发数 (e.g. S-08 1000 并发压测)
    pub concurrency: Option<u32>,
    /// 测试域: player / economy / match / social / admin
    pub domain: Option<String>,
}

/// `POST /api/v1/exclusion/test-runs`
///
/// v0.53 stub: 返回 501 not_implemented + 测试矩阵元数据, 真实 e2e_runner.py 跨 session 续.
///
/// 跨 session 续:
/// - 调 `scripts/automation/exclusion/e2e_runner.py` (per 守门 #19 [P] e2e_runner.py)
/// - 18 UT + 8 IT + 12 E2E + 1 1000 并发压测 = 38 测试
/// - 守门 #1 v25 单 crate 模式 (per cargo test 跳过 workspace, 4 步实证)
pub async fn start_test_run(Json(req): Json<TestRunRequest>) -> (StatusCode, Json<Value>) {
    let _ = req; // stub 阶段不实际使用
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-08 集成测试 + 性能压测跨 session 续; 5 域 Lead 真人到位后接入 scripts/automation/exclusion/e2e_runner.py (per 守门 #19 [P] e2e_runner.py + 守门 #1 v25 单 crate 模式)",
            "test_matrix": {
                "ut": 18,
                "it": 8,
                "e2e": 12,
                "pt_concurrency": 1000
            },
            "scenarios": [
                {"id": "S-01", "layer": "UI", "kind": "idempotency-key-conflict"},
                {"id": "S-02", "layer": "L0", "kind": "dispatch-lock-acquire"},
                {"id": "S-03", "layer": "L1", "kind": "subagent-lock-lease"},
                {"id": "S-04", "layer": "L2", "kind": "domain-mutex-cas"},
                {"id": "S-05", "layer": "Cross-UI-L0", "kind": "lock-conflict-toast"},
                {"id": "S-06", "layer": "Cross-L0-L1", "kind": "heartbeat-renewal"},
                {"id": "S-07", "layer": "Cross-L1-L2", "kind": "lease-expiry-cleanup"},
                {"id": "S-08", "layer": "Cross-UI-L2", "kind": "1000-concurrent-pressure-test"}
            ],
            "domains": ["player", "economy", "match", "social", "admin"],
            "守门_#1_v25": "cargo test 跳过 workspace, 单 crate 测 (check/fmt/clippy/test)",
            "meta": ResponseMeta::stub(),
        })),
    )
}

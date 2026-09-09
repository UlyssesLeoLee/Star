// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-07 4 层统一可观测性 (Python) — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.7):
//! - `scripts/automation/exclusion/lock_metric_exporter.py` — 5 Prometheus 指标
//! - `scripts/automation/exclusion/lock_leak_alerter.py` — 24h / 1h 阈值告警
//! - `scripts/automation/exclusion/archive_cron.py` — 24h 归档
//! - `console_server.py` 扩展 `/api/exclusion/*` 8 端点
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `GET /api/v1/exclusion/metrics` — 4 层统一可观测性指标
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #19 v19: agent 交互 Python 化.
//! 守门 #22 v22: 调试控制台 console_server.py 不污染 main 编译.
//! 守门 #23 v23: 调试页 AI 修改 mock 不开外部 API.

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

use crate::response::ResponseMeta;

/// `GET /api/v1/exclusion/metrics`
///
/// v0.53 stub: 返回 501 not_implemented + 4 层可观测性元数据, 真实 metrics_exporter.py 跨 session 续.
///
/// 跨 session 续:
/// - 调 `scripts/automation/exclusion/lock_metric_exporter.py` (per 守门 #19 [P] metrics_exporter.py)
/// - 5 Prometheus 指标 (per EX-07 §3.7)
/// - Grafana dashboard + 告警阈值 (per §14.9 G-EI-06 SRE Lead 评审)
pub async fn get_metrics() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-07 4 层统一可观测性跨 session 续; 5 域 Lead 真人到位后接入 scripts/automation/exclusion/lock_metric_exporter.py + alerter + archive (per 守门 #19 [P] metrics_exporter.py + 守门 #22 控制台不污染 main)",
            "layers": ["UI", "L0-TopAgent", "L1-SubAgent", "L2-Domain-22-crate"],
            "components": [
                {"name": "lock_metric_exporter.py", "kind": "python", "subprocess": true, "metrics_count": 5},
                {"name": "lock_leak_alerter.py", "kind": "python", "thresholds": ["24h", "1h"]},
                {"name": "archive_cron.py", "kind": "python", "interval": "24h"},
                {"name": "console_server.py", "kind": "python", "endpoints": 8, "per守门_#22_不污染_main": true}
            ],
            "prometheus_metrics": [
                "star_exclusion_locks_acquired_total",
                "star_exclusion_locks_released_total",
                "star_exclusion_locks_held_seconds",
                "star_exclusion_idempotency_keys_total",
                "star_exclusion_lock_leak_count"
            ],
            "sre_review_pending": "G-EI-05 / G-EI-06 SRE Lead 拍板 阈值 + Grafana dashboard",
            "meta": ResponseMeta::stub(),
        })),
    )
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-05 UI IdempotencyManager (TypeScript) — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.5):
//! - `frontend/src/lib/exclusion/idempotency.ts` — UI 幂等键管理
//! - `frontend/src/lib/exclusion/lock_status.ts` — 锁状态显示
//! - `frontend/src/lib/exclusion/trace_propagator.ts` — TraceId 透传
//! - `frontend/src/lib/exclusion/business_hash.ts` — 业务 hash 计算
//! - 3 component: LockStatusBadge / LockStatusPanel / LockConflictToast
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `GET /api/v1/exclusion/ui-state` — UI 幂等状态查询
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #19 v19: agent 交互 Python 化 (走 `subprocess.run`, 不开 RPC).

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

use crate::response::ResponseMeta;

/// `GET /api/v1/exclusion/ui-state`
///
/// v0.53 stub: 返回 501 not_implemented + UI 元数据, 真实 frontend gm-console 集成跨 session 续.
///
/// 跨 session 续:
/// - 集成 gm-console AppShell (per EX-05 §3.5 拍板)
/// - 4 lib + 3 component (per EX-05 §3.5 模块拆分)
/// - TraceId 透传 (per EX-02 M-12 TraceIdPropagator 联动)
pub async fn get_ui_state() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-05 UI IdempotencyManager 跨 session 续; 5 域 Lead 真人到位后集成 frontend/src/lib/exclusion/ + 3 component (per 守门 #19 [M] frontend_module_gen.py)",
            "libs": [
                {"path": "frontend/src/lib/exclusion/idempotency.ts", "kind": "ui-store"},
                {"path": "frontend/src/lib/exclusion/lock_status.ts", "kind": "ui-store"},
                {"path": "frontend/src/lib/exclusion/trace_propagator.ts", "kind": "ui-store"},
                {"path": "frontend/src/lib/exclusion/business_hash.ts", "kind": "util"}
            ],
            "components": [
                {"name": "LockStatusBadge", "kind": "react-component"},
                {"name": "LockStatusPanel", "kind": "react-component"},
                {"name": "LockConflictToast", "kind": "react-component"}
            ],
            "integration_target": "gm-console AppShell",
            "meta": ResponseMeta::stub(),
        })),
    )
}

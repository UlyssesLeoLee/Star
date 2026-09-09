// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-03 L0 DispatchLockManager (Python) — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.3):
//! - `scripts/automation/exclusion/dispatch_lock.py` — L0 TopAgent 调度锁
//! - `scripts/automation/exclusion/idempotency_key_store.py` — 幂等键存储
//! - 双键 dedup: client_uuid + business_hash (per D-02 拍板)
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `POST /api/v1/exclusion/dispatch-locks` — L0 调度锁申请
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #19 v19: agent 交互 Python 化 (走 `subprocess.run`, 不开 RPC).

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::response::ResponseMeta;

#[derive(Debug, Deserialize)]
pub struct DispatchLockRequest {
    /// L0 调度任务 ID (per L0 TopAgent 派发链)
    pub dispatch_id: String,
    /// 双键 dedup 客户端 UUID (per D-02 拍板)
    pub client_uuid: String,
    /// 双键 dedup 业务 hash (per D-02 拍板, 缺则自动 fallback)
    pub business_hash: Option<String>,
    /// 业务子域: player / economy / match / social / admin (per 守门 #3 v2 5 域)
    pub domain: String,
}

/// `POST /api/v1/exclusion/dispatch-locks`
///
/// v0.53 stub: 返回 501 not_implemented + L0 调度元数据, 真实 L0 dispatch_lock.py subprocess 跨 session 续.
///
/// 跨 session 续:
/// - 调 `scripts/automation/exclusion/dispatch_lock.py` (per 守门 #19 [P] Python 化)
/// - 双键 dedup (client_uuid + business_hash)
/// - 5 域 RBAC 检查 (per §14.12 5 域 helper, 5 域 Lead 决策临时代签 per 守门 #3 v2)
pub async fn acquire_dispatch_lock(
    Json(req): Json<DispatchLockRequest>,
) -> (StatusCode, Json<Value>) {
    let _ = req; // stub 阶段不实际使用
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-03 L0 DispatchLockManager 跨 session 续; 5 域 Lead 真人到位后接入 scripts/automation/exclusion/dispatch_lock.py (per 守门 #19 [P] Python 化)",
            "components": [
                {"name": "dispatch_lock.py", "kind": "python", "subprocess": true},
                {"name": "idempotency_key_store.py", "kind": "python", "subprocess": true}
            ],
            "dedup_strategy": {
                "primary_key": "client_uuid",
                "secondary_key": "business_hash",
                "fallback": "auto-derive business_hash if missing"
            },
            "domains": ["player", "economy", "match", "social", "admin"],
            "meta": ResponseMeta::stub(),
        })),
    )
}

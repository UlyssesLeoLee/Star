// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-04 L1 SubAgentLock (Python) — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.4):
//! - `scripts/automation/exclusion/subagent_lock.py` — L1 SubAgent 锁
//! - `scripts/automation/exclusion/lock_watcher.py` — lease + heartbeat 30s
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `POST /api/v1/exclusion/subagent-locks` — L1 SubAgent 锁申请
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #19 v19: agent 交互 Python 化 (走 `subprocess.run`, 不开 RPC).
//! 守门 #13 a: L0 协调 L1↔L1 (L2 SubAgent 不能直接调 L3 Domain 互抢, 必须经 L1 TopAgent 派发)

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::response::ResponseMeta;

#[derive(Debug, Deserialize)]
pub struct SubAgentLockRequest {
    /// L1 SubAgent ID (per 9 SA 编号 SA-01..SA-09)
    pub subagent_id: String,
    /// 申请锁的业务键 (per L1 调度目标)
    pub lock_key: String,
    /// 业务子域: player / economy / match / social / admin
    pub domain: String,
    /// 锁租约秒 (默认 60s, 跟 heartbeat 30s 对齐)
    pub lease_seconds: Option<u32>,
}

/// `POST /api/v1/exclusion/subagent-locks`
///
/// v0.53 stub: 返回 501 not_implemented + L1 元数据, 真实 subagent_lock.py subprocess 跨 session 续.
///
/// 跨 session 续:
/// - 调 `scripts/automation/exclusion/subagent_lock.py` (per 守门 #19 [P] Python 化)
/// - lease + heartbeat 30s (per EX-04 lock_watcher.py)
/// - L0 → L1 派发链 (per 守门 #13 a, L0 协调 L1↔L1)
pub async fn acquire_subagent_lock(
    Json(req): Json<SubAgentLockRequest>,
) -> (StatusCode, Json<Value>) {
    let _ = req; // stub 阶段不实际使用
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-04 L1 SubAgentLock 跨 session 续; 5 域 Lead 真人到位后接入 scripts/automation/exclusion/subagent_lock.py + lock_watcher.py (per 守门 #19 [P] Python 化 + 守门 #13 a L0 协调 L1↔L1)",
            "components": [
                {"name": "subagent_lock.py", "kind": "python", "subprocess": true},
                {"name": "lock_watcher.py", "kind": "python", "heartbeat_seconds": 30, "lease_seconds": 60}
            ],
            "l0_coordination": "L0 协调 L1↔L1; L2 SubAgent 必须经 L1 TopAgent 派发 (per 守门 #13 a)",
            "domains": ["player", "economy", "match", "social", "admin"],
            "subagent_ids": ["SA-01", "SA-02", "SA-03", "SA-04", "SA-05", "SA-06", "SA-07", "SA-08", "SA-09"],
            "subagent_count": 9,
            "meta": ResponseMeta::stub(),
        })),
    )
}

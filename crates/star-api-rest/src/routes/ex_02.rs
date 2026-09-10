// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-02 star-mutex 共享 crate — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §3.2):
//! - 5 module: M-08 DomainMutex / M-09 StarMutexAdapter / M-10 LockAuditLogger /
//!   M-11 ExclusionPolicyLoader / M-12 TraceIdPropagator
//! - 2 module (扩展): M-13 VersionCAS / M-14 PgAdvisoryLock
//! - 1 proc-macro: `domain_mutex` (22 domain crate 接入)
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `POST /api/v1/exclusion/mutex` — 申请 mutex 锁
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::response::ResponseMeta;

#[derive(Debug, Deserialize)]
pub struct MutexAcquireRequest {
    /// 锁名称 (业务键, e.g. "work-item:STAR-1024")
    pub lock_name: String,
    /// 持有者 ID (per §14.9 EX-02 DomainMutex owner)
    pub owner_id: String,
    /// TTL 秒 (默认 30s, 跟 EX-04 heartbeat 对齐)
    pub ttl_seconds: Option<u32>,
}

/// `POST /api/v1/exclusion/mutex`
///
/// v0.53 stub: 返回 501 not_implemented + 7 module metadata 列表, 真实 PG advisory lock 跨 session 续.
///
/// 跨 session 续:
/// - 接入 star-mutex crate (per 守门 #19 [M] `rust_module_gen.py`)
/// - PG advisory lock (per D-01 拍板, 跟 ADR-0047 PG checkpointer Tier 3 共享)
/// - Version CAS (per EX-02 M-13 module)
/// - TraceId 透传 (per EX-02 M-12 module)
pub async fn acquire_mutex(Json(req): Json<MutexAcquireRequest>) -> (StatusCode, Json<Value>) {
    let _ = req; // stub 阶段不实际使用
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-02 star-mutex 共享 crate (7 module + 1 proc-macro) 跨 session 续; 5 域 Lead 真人到位后接入 crates/star-mutex/ + 22 domain DomainMutex proc-macro",
            "modules": [
                {"id": "M-08", "name": "DomainMutex", "kind": "trait"},
                {"id": "M-09", "name": "StarMutexAdapter", "kind": "struct"},
                {"id": "M-10", "name": "LockAuditLogger", "kind": "struct"},
                {"id": "M-11", "name": "ExclusionPolicyLoader", "kind": "struct"},
                {"id": "M-12", "name": "TraceIdPropagator", "kind": "struct"},
                {"id": "M-13", "name": "VersionCAS", "kind": "struct"},
                {"id": "M-14", "name": "PgAdvisoryLock", "kind": "struct"}
            ],
            "proc_macro": "domain_mutex",
            "meta": ResponseMeta::stub(),
        })),
    )
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! EX-01 5 张新表 DDL + RLS 13 类 — stub handler (per brief v0.53 §14.9)
//!
//! 范围 (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2 + §5):
//! - 5 张新表: idempotency_keys (T) / lease_log (T) / advisory_lock_audit (T) /
//!   idempotency_keys_archive (T) / exclusion_policy_master (M, SCD Type 2)
//! - RLS 13 类 (per 守门 #13 c Master 100% RLS)
//! - audit trigger (per 守门 #13 d Transaction 100% audit)
//!
//! 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3).
//! 现状: 501 not_implemented (跟 OAuth2 introspect handler 同模式).
//!
//! 路由: `GET /api/v1/exclusion/idempotency-keys` — 列表查询 5 张表
//!
//! 守门 #5 v2: token / secret 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

use crate::response::ResponseMeta;

/// `GET /api/v1/exclusion/idempotency-keys`
///
/// v0.53 stub: 返回 501 not_implemented + 5 张表 metadata 列表, 真实 DB 查询跨 session 续.
///
/// 跨 session 续 (per 守门 #14 v3 Mavis 永久代签 + 5 域 Lead 真人到位后):
/// - 接入 5 张新表 Repository (per `crates/star-pg-adapter` PgAdapter)
/// - 5 域 RBAC 5 × 9 SA × claims.roles 检查 (per §14.12 5 域 RBAC helper)
/// - 跨租户 RLS 13 类强制 (per 守门 #13 c Master 100% RLS)
pub async fn list_idempotency_keys() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "description": "EX-01 5 张新表 (idempotency_keys/lease_log/advisory_lock_audit/idempotency_keys_archive/exclusion_policy_master) DDL 跨 session 续; 5 域 Lead 真人到位后接入 PgAdapter Repository",
            "tables": [
                {"name": "idempotency_keys", "category": "T", "rls": true, "audit_trigger": true},
                {"name": "lease_log", "category": "T", "rls": true, "audit_trigger": true},
                {"name": "advisory_lock_audit", "category": "T", "rls": true, "audit_trigger": true},
                {"name": "idempotency_keys_archive", "category": "T", "rls": true, "audit_trigger": true},
                {"name": "exclusion_policy_master", "category": "M", "scd": "Type2", "rls": true, "audit_trigger": false}
            ],
            "meta": ResponseMeta::stub(),
        })),
    )
}

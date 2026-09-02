// SPDX-License-Identifier: MIT OR Apache-2.0
//! AuditLayer stub (per spec §1.4 + §5.1)
//!
//! MVP 阶段: no-op
//! P2 阶段 worker 子代理实装:
//! - 每个 request 落 T 类 audit (per spec §5.1, 物理删除禁止 + append-only + RLS 13 类必携)
//! - ActorType: API Key 调 → `ActorType.Automation` (per spec §1.4)
//! - 审计表: `api_key_audit_log` (T 类, per spec §5.1 4 表)
//! - 字段: `api_key_id, request_id, method, path, status_code, actor_type, actor_id, tenant_id, ts`

use axum::{extract::Request, middleware::Next, response::Response};

/// 审计中间件 stub — 当前 no-op
pub async fn audit_layer_stub(req: Request, next: Next) -> Response {
    // TODO(per spec §1.4 + §5.1 + AGENTS.md §4 #20): 派 worker 子代理实装
    next.run(req).await
}

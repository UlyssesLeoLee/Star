// SPDX-License-Identifier: MIT OR Apache-2.0
//! RateLimitLayer stub (per spec §1.4)
//!
//! MVP 阶段: no-op
//! P2 阶段 worker 子代理实装:
//! - 60 req/min per key 默认 (per spec §1.4)
//! - 1000 req/hour per key 默认
//! - burst 2x token bucket
//! - 超 → 429 Too Many Requests + `Retry-After` header
//! - MVP in-memory (单实例), Phase 2+ Redis 集群 (per spec §6 G-06)

use axum::{extract::Request, middleware::Next, response::Response};

/// 限流中间件 stub — 当前 no-op
pub async fn rate_limit_layer_stub(req: Request, next: Next) -> Response {
    // TODO(per spec §1.4 + AGENTS.md §4 #20): 派 worker 子代理实装
    next.run(req).await
}

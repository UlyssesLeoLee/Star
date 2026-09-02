// SPDX-License-Identifier: MIT OR Apache-2.0
//! AuthLayer stub (per spec §1.3)
//!
//! MVP 阶段: no-op
//! P2 阶段 worker 子代理实装:
//! - 抽 `Authorization: Bearer <api_key>` header
//! - 查 `api_keys` 表 (M 类, SCD Type 2, per spec §5.1)
//! - 验 key 状态 (`active`) + 验 bcrypt/Argon2id hash (per spec §6 G-09)
//! - 检查 scope (`developer.read` / `developer.write` / `webhook.manage` 等)
//! - IP 白名单 (per spec §6 G-03, P2 阶段)
//! - 通过 → 注入 `ActorContext` 到 request extension (per P0-1 联动审计, AGENTS.md §4.1 v16)

use axum::{extract::Request, middleware::Next, response::Response};

/// 鉴权中间件 stub — 当前 no-op
pub async fn auth_layer_stub(req: Request, next: Next) -> Response {
    // TODO(per spec §1.3 + AGENTS.md §4 #20): 派 worker 子代理实装
    // 派前必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`
    next.run(req).await
}

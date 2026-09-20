// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/routes/` — per-route HTTP modules grouped by feature (ULYS-98-W1
//! 新增, per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 1.3 backend Chat RPC stub").
//!
//! 与既有 `bff/src/collaboration/` + `bff/src/worktree_canvas/` 模式不同 —
//! 那两个模块是自包含 feature surface (state + permission + audit + dto +
//! controller + middleware), 而 `routes/` 是 BFF **轻量级 RPC stub** 集合:
//! - 一个 feature 一个文件 (e.g. `chat.rs`).
//! - 不强加 RBAC / Idempotency / Audit 全套 (W1 是 stub, W2 再按需叠加).
//! - 仍 0 `unsafe`, 仍受 `unsafe_code = "forbid"` workspace lint 保护.
//!
//! 当前包含:
//! - [`chat`] — `POST /v1/chat/send` + `GET /v1/chat/messages` (W1 stub:
//!   in-memory `Arc<Mutex<HashMap>>` session store, assistant reply hardcoded
//!   "AI 暂未接入" per brief §"Sub-task 1.3"). W2 接真 LLM via
//!   `domain_llm::LlmProvider::chat_completion`.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 每个 route 都返 `Result<Json<T>, (StatusCode, Json<ApiError>)>`, 显式不
//!   用 `anyhow` / `unwrap` / `panic` (per 守门 #11 缺标比错标).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2).

pub mod chat;
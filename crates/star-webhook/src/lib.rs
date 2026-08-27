// SPDX-License-Identifier: MIT OR Apache-2.0
//! crates/star-webhook — Webhook receiver (per spec/services/03-webhook-adapter-spec.md §2-§5)
//!
//! 提供 GitHub/GitLab/Bitbucket 三家 SCM 的 webhook 签名验证 + 幂等投递 + 重试 + 死信。
//! Phase F 阶段：in-memory delivery store (TODO: Phase F+ 接 redis/kafka)。

pub mod delivery;
pub mod signature_verify;

use serde::{Deserialize, Serialize};

/// 投递状态 (per spec/services/03 §3 — 4 态机)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WebhookDeliveryState {
    /// 已接收,等待投递
    Pending,
    /// 投递成功
    Delivered,
    /// 投递失败,可重试
    Failed,
    /// 超过最大重试次数,进入死信队列
    DeadLetter,
}

/// Webhook 事件 (envelope)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// 幂等键 (由 provider + delivery_id 复合)
    pub delivery_id: String,
    /// SCM provider ("github" | "gitlab" | "bitbucket")
    pub provider: String,
    /// 事件类型 (e.g. "push", "pull_request", "merge_request")
    pub event_type: String,
    /// 签名头原始值 (不暴露 secret 派生内容)
    pub signature: String,
    /// 业务 payload
    pub payload: serde_json::Value,
    /// 当前投递状态
    pub state: WebhookDeliveryState,
}

pub use delivery::{DeliveryStore, RetryPolicy};
pub use signature_verify::SignatureVerifier;

// SPDX-License-Identifier: MIT OR Apache-2.0
//! DeliveryStore — webhook 幂等投递 + 重试 + 死信 (per spec/services/03 §3-§5)
//!
//! Phase F 阶段：in-memory HashMap 存储 (TODO: Phase F+ 接 redis SETNX + kafka)。
//! API 形状按 spec §3-§5 设计,Phase F+ 切换 backend 时不破坏调用方。

use super::{WebhookDeliveryState, WebhookEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 重试策略 (per spec/services/03 §5)
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试退避基准 (秒,实际 = backoff_sec * 2^attempt)
    pub backoff_sec: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_sec: 60,
        }
    }
}

impl RetryPolicy {
    /// 给定当前 attempt (0-based),返回是否还应重试
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }

    /// 给定 attempt (0-based),返回本次重试前应等待的秒数
    pub fn backoff_for(&self, attempt: u32) -> u64 {
        self.backoff_sec.saturating_mul(1u64 << attempt.min(20))
    }
}

/// 投递存储 (幂等 + 状态机)
pub struct DeliveryStore {
    inner: Arc<RwLock<HashMap<String, WebhookEvent>>>,
    policy: RetryPolicy,
}

impl DeliveryStore {
    /// 创建新 store (使用默认重试策略)
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            policy: RetryPolicy::default(),
        }
    }

    /// 创建带自定义重试策略的 store
    pub fn with_policy(policy: RetryPolicy) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            policy,
        }
    }

    /// 记录事件 (幂等:已存在 delivery_id 则返回 false)
    pub async fn record(&self, event: WebhookEvent) -> Result<bool, String> {
        let mut g = self.inner.write().await;
        if g.contains_key(&event.delivery_id) {
            return Ok(false);
        }
        g.insert(event.delivery_id.clone(), event);
        Ok(true)
    }

    /// 标记投递成功
    pub async fn mark_delivered(&self, delivery_id: &str) -> Result<(), String> {
        let mut g = self.inner.write().await;
        let e = g
            .get_mut(delivery_id)
            .ok_or_else(|| "not found".to_string())?;
        e.state = WebhookDeliveryState::Delivered;
        Ok(())
    }

    /// 标记投递失败 (Failed 态,仍可重试)
    pub async fn mark_failed(&self, delivery_id: &str) -> Result<(), String> {
        let mut g = self.inner.write().await;
        let e = g
            .get_mut(delivery_id)
            .ok_or_else(|| "not found".to_string())?;
        e.state = WebhookDeliveryState::Failed;
        Ok(())
    }

    /// 标记死信 (DeadLetter,不再重试)
    pub async fn mark_dead_letter(&self, delivery_id: &str) -> Result<(), String> {
        let mut g = self.inner.write().await;
        let e = g
            .get_mut(delivery_id)
            .ok_or_else(|| "not found".to_string())?;
        e.state = WebhookDeliveryState::DeadLetter;
        Ok(())
    }

    /// 查询事件
    pub async fn get(&self, delivery_id: &str) -> Option<WebhookEvent> {
        self.inner.read().await.get(delivery_id).cloned()
    }

    /// 当前事件总数 (测试用)
    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }

    /// 是否为空 (clippy `len_without_is_empty`)
    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }

    /// 引用重试策略
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.policy
    }
}

impl Default for DeliveryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_event(id: &str) -> WebhookEvent {
        WebhookEvent {
            delivery_id: id.into(),
            provider: "github".into(),
            event_type: "push".into(),
            signature: "x".into(),
            payload: serde_json::json!({}),
            state: WebhookDeliveryState::Pending,
        }
    }

    #[tokio::test]
    async fn idempotent_record() {
        let s = DeliveryStore::new();
        assert!(s.record(mk_event("d1")).await.unwrap());
        assert!(!s.record(mk_event("d1")).await.unwrap());
        assert_eq!(s.len().await, 1);
    }

    #[tokio::test]
    async fn mark_delivered_transitions() {
        let s = DeliveryStore::new();
        s.record(mk_event("d1")).await.unwrap();
        s.mark_delivered("d1").await.unwrap();
        let e = s.get("d1").await.unwrap();
        assert_eq!(e.state, WebhookDeliveryState::Delivered);
    }

    #[tokio::test]
    async fn mark_dead_letter_transitions() {
        let s = DeliveryStore::new();
        s.record(mk_event("d1")).await.unwrap();
        s.mark_dead_letter("d1").await.unwrap();
        let e = s.get("d1").await.unwrap();
        assert_eq!(e.state, WebhookDeliveryState::DeadLetter);
    }

    #[tokio::test]
    async fn mark_missing_returns_err() {
        let s = DeliveryStore::new();
        assert!(s.mark_delivered("nope").await.is_err());
        assert!(s.mark_dead_letter("nope").await.is_err());
    }

    #[tokio::test]
    async fn retry_policy_default_values() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_retries, 3);
        assert_eq!(p.backoff_sec, 60);
        assert!(p.should_retry(0));
        assert!(p.should_retry(2));
        assert!(!p.should_retry(3));
        assert!(!p.should_retry(99));
    }

    #[tokio::test]
    async fn retry_policy_backoff_exponential() {
        let p = RetryPolicy::default();
        assert_eq!(p.backoff_for(0), 60);
        assert_eq!(p.backoff_for(1), 120);
        assert_eq!(p.backoff_for(2), 240);
    }

    #[tokio::test]
    async fn with_custom_policy() {
        let p = RetryPolicy {
            max_retries: 5,
            backoff_sec: 10,
        };
        let s = DeliveryStore::with_policy(p);
        assert_eq!(s.retry_policy().max_retries, 5);
    }

    #[tokio::test]
    async fn mark_failed_keeps_retryable() {
        let s = DeliveryStore::new();
        s.record(mk_event("d1")).await.unwrap();
        s.mark_failed("d1").await.unwrap();
        let e = s.get("d1").await.unwrap();
        assert_eq!(e.state, WebhookDeliveryState::Failed);
    }
}

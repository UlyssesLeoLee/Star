//! crates/star-saga/src/idempotency_store.rs
//!
//! IdempotencyStore — dedup compensation/call key (per E.6 gap #1, INV-CS-02)
//!
//! ## 关键不变量
//!
//! - INV-IDS-01: `check_and_record` 首次出现返回 true 并记录, 已存在返回 false (dedup 命中)
//! - INV-IDS-02: 跨进程持久化 (Redis/Postgres schema 选型) 需真人拍板, 本 impl 仅进程内存级
//!   (重启丢失, 对齐 `saga_orchestrator.rs` INV-SG-ORCH-04 现状) — 待 match 域 Lead 真人补持久化后端

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// 若 key 首次出现, 记录并返回 true; 若已存在 (dedup 命中), 返回 false
    async fn check_and_record(&self, key: &str) -> bool;
}

/// 进程内存级 dedup store (per INV-IDS-02, 待真人补持久化后端)
#[derive(Default, Clone)]
pub struct InMemoryIdempotencyStore {
    seen: Arc<RwLock<HashSet<String>>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn check_and_record(&self, key: &str) -> bool {
        let mut g = self.seen.write().await;
        g.insert(key.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn first_seen_true_then_dedup_false() {
        let store = InMemoryIdempotencyStore::new();
        assert!(store.check_and_record("k1").await);
        assert!(!store.check_and_record("k1").await);
        assert!(store.check_and_record("k2").await);
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/idempotency.rs` — Idempotency key cache (24h TTL)
//! (per DD-WORKTREE-CANVAS-001 §42 + IMPL-PLAN §4.13.5 + spec §4.4).
//!
//! Idempotency 派生需求 (per spec §4.4 Action 矩阵):
//! - **Warning / Destructive / Lock / Unlock Action 必填 idempotency_key**
//! - 重放同一 key 在 24h 内返回缓存结果 (per FR-ACTION-006 + IMPL-PLAN §5
//!   `worktree_idempotency` 表 retention 24h TTL).
//!
//! 阶段 1 占位: in-memory `HashMap` (per-process), 阶段 2 业务实装落地
//! PostgreSQL `worktree_idempotency` 表 (per IMPL-PLAN §5 W/T/M 100% 覆盖: W 类表).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - Idempotency key 全局唯一 UUID, 0 业务字段 (per spec §4.4).

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::dto::WorktreeApiError;

/// Idempotency record: 缓存 Action 的 response + actor + 创建时间.
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    /// 原始 actor_id (per 守门 #10).
    pub actor_id: Uuid,
    /// Action 名 (e.g. "Merge").
    pub action: String,
    /// 缓存的 response JSON (per FR-ACTION-006 重放 = 同一 response).
    pub response: serde_json::Value,
    /// 创建时间 (Instant, 用于 TTL 过期检查).
    pub created_at: Instant,
}

/// Idempotency cache (per-process in-memory, 阶段 1 占位).
///
/// 线程安全 (RwLock). 24h TTL (per FR-ACTION-006 / IMPL-PLAN §5 worktree_idempotency 表).
#[derive(Debug)]
pub struct IdempotencyCache {
    /// key → record (per FR-ACTION-006 唯一索引).
    map: RwLock<HashMap<Uuid, IdempotencyRecord>>,
    /// TTL (default 24h, per FR-ACTION-006).
    ttl: Duration,
}

impl IdempotencyCache {
    /// 24h TTL 默认 cache.
    pub const DEFAULT_TTL_HOURS: u64 = 24;

    /// 构造一个新的 cache (24h TTL).
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(Self::DEFAULT_TTL_HOURS * 3600),
        }
    }

    /// 构造一个自定义 TTL 的 cache (测试用).
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// 查询一个 key: 命中且未过期 → Some(record), 否则 None.
    pub fn lookup(&self, key: Uuid) -> Option<IdempotencyRecord> {
        let map = self.map.read().ok()?;
        let record = map.get(&key)?;
        if record.created_at.elapsed() > self.ttl {
            // 过期, 调用方应重新执行 Action.
            return None;
        }
        Some(record.clone())
    }

    /// 写入一个新 record (覆盖旧值).
    pub fn store(
        &self,
        key: Uuid,
        actor_id: Uuid,
        action: String,
        response: serde_json::Value,
    ) -> Result<(), WorktreeApiError> {
        if key.is_nil() {
            return Err(WorktreeApiError::bad_request(
                "idempotency_key is nil",
                "idempotency::store",
            ));
        }
        if action.is_empty() {
            return Err(WorktreeApiError::bad_request(
                "action is empty",
                "idempotency::store",
            ));
        }
        let record = IdempotencyRecord {
            actor_id,
            action,
            response,
            created_at: Instant::now(),
        };
        let mut map = self.map.write().map_err(|_| {
            WorktreeApiError::new(
                "INTERNAL",
                "idempotency cache write lock poisoned",
                "bff.worktree_canvas",
                "idempotency::store",
            )
        })?;
        map.insert(key, record);
        Ok(())
    }

    /// 清理过期 record (per IMPL-PLAN §5 worktree_idempotency 表 24h TTL 派生).
    pub fn cleanup_expired(&self) -> usize {
        let mut map = match self.map.write() {
            Ok(m) => m,
            Err(_) => return 0,
        };
        let before = map.len();
        map.retain(|_, r| r.created_at.elapsed() <= self.ttl);
        before - map.len()
    }

    /// 当前 record 数 (per 阶段 1 测试 / 调试用).
    pub fn len(&self) -> usize {
        self.map.read().map(|m| m.len()).unwrap_or(0)
    }

    /// 是否为空.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for IdempotencyCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    #[test]
    fn new_cache_is_empty() {
        let c = IdempotencyCache::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn store_then_lookup_returns_record() {
        let c = IdempotencyCache::new();
        let key = Uuid::new_v4();
        c.store(key, actor(), "Merge".into(), serde_json::json!({"ok": true}))
            .unwrap();
        let r = c.lookup(key).unwrap();
        assert_eq!(r.actor_id, actor());
        assert_eq!(r.action, "Merge");
    }

    #[test]
    fn lookup_misses_after_ttl_expires() {
        let c = IdempotencyCache::with_ttl(Duration::from_millis(50));
        let key = Uuid::new_v4();
        c.store(key, actor(), "Lock".into(), serde_json::json!({"locked": true}))
            .unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert!(c.lookup(key).is_none());
    }

    #[test]
    fn store_rejects_nil_key() {
        let c = IdempotencyCache::new();
        assert!(c
            .store(Uuid::nil(), actor(), "Merge".into(), serde_json::json!({}))
            .is_err());
    }

    #[test]
    fn store_rejects_empty_action() {
        let c = IdempotencyCache::new();
        assert!(c
            .store(Uuid::new_v4(), actor(), String::new(), serde_json::json!({}))
            .is_err());
    }

    #[test]
    fn cleanup_expired_removes_old_entries() {
        let c = IdempotencyCache::with_ttl(Duration::from_millis(50));
        let old_key = Uuid::new_v4();
        c.store(old_key, actor(), "Lock".into(), serde_json::json!({}))
            .unwrap();
        std::thread::sleep(Duration::from_millis(100));
        let new_key = Uuid::new_v4();
        c.store(new_key, actor(), "Unlock".into(), serde_json::json!({}))
            .unwrap();
        let removed = c.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(c.len(), 1);
        assert!(c.lookup(new_key).is_some());
    }
}

//! `idempotency.rs` — Idempotency Store (per INV-WC-11 + DD §42 + FR-ACTION-006)
//!
//! 24h TTL (per ULYS-57.3 T8 acceptance criterion):
//! - Client 传入 `idempotency_key`
//! - Server 在 24h 内见到相同 key 直接返回之前的 result (replay)
//! - 24h 之外当作新请求
//!
//! 内存版 (本期 T8 scope); PG/Redis 落点保留接口 `IdempotencyStore::record` / `check`。

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 24 小时 TTL (per INV-WC-11 守门)
pub const IDEMPOTENCY_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Idempotency 记录 (per DD §42 IdempotencyRecord schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    /// Idempotency Key (per FR-ACTION-006, Uuid v4)
    pub key: Uuid,
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// Action 结果 JSON (per DD §42 — re-run returns same ActionResult)
    pub result: serde_json::Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl IdempotencyRecord {
    /// 检查记录是否已过期 (24h)
    pub fn is_expired(&self, now: SystemTime) -> bool {
        let created_ts = self.created_at.timestamp().max(0) as u64;
        let record_epoch = SystemTime::UNIX_EPOCH + Duration::from_secs(created_ts);
        let live = now.duration_since(record_epoch).unwrap_or_default();
        live > IDEMPOTENCY_TTL
    }
}

/// Idempotency Store (内存版, 线程安全)
///
/// 真实部署用 PG `action_idempotency` 表 (per DD §42 + ULYS-57.1 migration 0005).
/// 本期 T8 用内存 RwLock 实现, 接口签名与 PG 版兼容, 后续 P2 替换 backend 即可。
#[derive(Debug, Default)]
pub struct IdempotencyStore {
    /// key -> record
    inner: RwLock<HashMap<Uuid, IdempotencyRecord>>,
}

impl IdempotencyStore {
    /// 构造一个新 Store
    pub fn new() -> Self {
        Self::default()
    }

    /// 查询: 24h 内若有相同 key, 返回 Some(record) 给 client replay
    ///
    /// TTL 检查: `created_at + 24h > now` 才算 live。
    /// `now` 默认 `SystemTime::now()`, 测试可注入。
    pub fn check(&self, key: Uuid) -> Option<IdempotencyRecord> {
        self.check_at(key, SystemTime::now())
    }

    /// 同 `check` 但接受显式 `now` (便于测试)
    pub fn check_at(&self, key: Uuid, now: SystemTime) -> Option<IdempotencyRecord> {
        let guard = self.inner.read().ok()?;
        let record = guard.get(&key)?;
        if record.is_expired(now) {
            // 已过期, 视为 None
            None
        } else {
            Some(record.clone())
        }
    }

    /// 记录: 把 `record` 写入 store (key UNIQUE — 同 key 第二次 record 视为新请求并替换)
    pub fn record(&self, record: IdempotencyRecord) {
        if let Ok(mut g) = self.inner.write() {
            g.insert(record.key, record);
        }
    }

    /// 清空过期 (housekeeping, 默认 24h 之后的 record 移出)
    pub fn evict_expired(&self) {
        self.evict_expired_at(SystemTime::now())
    }

    /// 同 `evict_expired` 但接受显式 `now`
    pub fn evict_expired_at(&self, now: SystemTime) {
        if let Ok(mut g) = self.inner.write() {
            g.retain(|_, v| !v.is_expired(now));
        }
    }

    /// 当前记录数 (测试用)
    pub fn len(&self) -> usize {
        self.inner.read().map(|g| g.len()).unwrap_or(0)
    }

    /// 是否为空 (测试用)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionResult;
    use chrono::Duration as ChronoDuration;

    fn make_record(key: Uuid, age_secs: i64) -> IdempotencyRecord {
        let created_at = Utc::now() - ChronoDuration::seconds(age_secs);
        IdempotencyRecord {
            key,
            worktree_id: Uuid::new_v4(),
            result: serde_json::json!({"success": true}),
            created_at,
        }
    }

    #[test]
    fn idempotency_24h_ttl_replay() {
        // 守门 UT-3: 24h TTL replay — 24h 内返回旧结果, 之外返回 None
        let store = IdempotencyStore::new();
        let key = Uuid::new_v4();
        let record = make_record(key, 0);
        store.record(record.clone());

        // 1. 立即查询 — 应返回 Some (24h 内)
        let hit = store.check(key);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().key, key);

        // 2. 注入 23h59m 之前的 record — 仍应 hit (在 24h 内)
        let key2 = Uuid::new_v4();
        let record_23h = make_record(key2, 23 * 3600 + 59 * 60);
        store.record(record_23h);
        assert!(store.check(key2).is_some());

        // 3. 注入 25h 之前的 record — 应 NOT hit (过 24h)
        let key3 = Uuid::new_v4();
        let record_25h = make_record(key3, 25 * 3600);
        store.record(record_25h);
        assert!(store.check(key3).is_none());

        // 4. record 后返回的 result JSON 内容一致
        let hit = store.check(key).unwrap();
        assert_eq!(hit.result, serde_json::json!({"success": true}));

        // 5. evict_expired 移除 25h 那条
        let before = store.len();
        store.evict_expired();
        let after = store.len();
        assert!(after < before, "evict should remove expired records");
        // 25h 那条已 evict, 24h 内 2 条保留
        assert_eq!(after, 2);
    }

    #[test]
    fn replay_returns_same_result() {
        let store = IdempotencyStore::new();
        let key = Uuid::new_v4();
        let result = ActionResult {
            success: true,
            worktree_id: Uuid::new_v4(),
            new_state: Some("merged".to_string()),
            duration_ms: 1234,
            warnings: vec![],
            errors: vec![],
        };
        let value = serde_json::to_value(&result).unwrap();
        store.record(IdempotencyRecord {
            key,
            worktree_id: result.worktree_id,
            result: value.clone(),
            created_at: Utc::now(),
        });
        let hit = store.check(key).unwrap();
        let replayed: ActionResult = serde_json::from_value(hit.result).unwrap();
        assert_eq!(replayed.success, result.success);
        assert_eq!(replayed.new_state, result.new_state);
        assert_eq!(replayed.duration_ms, result.duration_ms);
    }
}

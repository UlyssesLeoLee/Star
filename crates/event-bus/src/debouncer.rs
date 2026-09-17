//! `debouncer.rs` — Event 防抖 (per spec §4.5 + DD §11.3 + NFR-REL-002)
//!
//! 防抖策略 (per BD §13.3):
//! | Event                | 防抖窗口 | 合并策略                       |
//! |----------------------|----------|-------------------------------|
//! | WorktreeChanged      | 100ms    | Last-write-wins (同 Worktree)  |
//! | HealthChanged        | 5s       | 30s 内只推送最终值             |
//! | RiskDetected         | 1s       | 同 (RiskType, Worktree) 合并   |
//! | TestCompleted        | 0 (即时) | 不防抖                        |
//!
//! 阶段 1 scope (per WORKTREE-CANVAS-IMPL-PLAN-001 §4.9 T9.4):
//! - 通用 Debouncer: 100ms 合并 + last-write-wins per key
//! - 不同 Event Kind 走不同配置 (per `DebouncerConfig::for_kind`)

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::events::EventKind;

/// 防抖配置 (per spec §4.5 + BD §13.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebouncerConfig {
    /// 防抖窗口 (per kind)
    pub window: Duration,
    /// 最大批量 (per kind)
    pub max_batch: usize,
}

impl DebouncerConfig {
    /// 给定 Event Kind 返回默认配置 (per BD §13.3)
    pub fn for_kind(kind: EventKind) -> Self {
        match kind {
            // WorktreeChanged: 100ms
            EventKind::WorktreeChanged | EventKind::WorktreeCreated => Self {
                window: Duration::from_millis(100),
                max_batch: 64,
            },
            // HealthChanged: 5s
            EventKind::HealthChanged => Self {
                window: Duration::from_secs(5),
                max_batch: 32,
            },
            // RiskDetected: 1s
            EventKind::RiskDetected => Self {
                window: Duration::from_secs(1),
                max_batch: 32,
            },
            // TestCompleted: 0 (不防抖, 即时)
            EventKind::TestCompleted => Self {
                window: Duration::from_millis(0),
                max_batch: 1,
            },
            // 其他: 默认 100ms
            _ => Self {
                window: Duration::from_millis(100),
                max_batch: 32,
            },
        }
    }
}

/// Debouncer — 单 key last-write-wins 合并
///
/// key 通常是 `worktree_id` (per spec §4.5 — 同 Worktree 合并).
/// `pending[k]`: (latest_value, earliest_arrival)
/// - 在 window 内若再来同 key, 替换 value
/// - 超过 window, flush 出 key + value
#[derive(Debug)]
pub struct Debouncer<V: Clone> {
    config: DebouncerConfig,
    pending: Mutex<HashMap<String, (V, Instant)>>,
}

impl<V: Clone> Debouncer<V> {
    /// 构造 (用默认 100ms window)
    pub fn new(config: DebouncerConfig) -> Self {
        Self {
            config,
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// Push 一条 (key, value) — last-write-wins within window
    pub fn push(&self, key: impl Into<String>, value: V) {
        let key = key.into();
        let now = Instant::now();
        let mut g = self.pending.lock().expect("Debouncer lock");
        g.insert(key, (value, now));
    }

    /// Drain 过期 key + value (返回 vec, 测试用)
    pub fn drain_expired(&self) -> Vec<(String, V)> {
        self.drain_expired_at(Instant::now())
    }

    /// 同 `drain_expired` 但接受显式 now (测试用)
    pub fn drain_expired_at(&self, now: Instant) -> Vec<(String, V)> {
        let mut g = self.pending.lock().expect("Debouncer lock");
        let mut out = Vec::new();
        let window = self.config.window;
        g.retain(|k, (v, arrival)| {
            if now.duration_since(*arrival) >= window {
                out.push((k.clone(), v.clone()));
                false
            } else {
                true
            }
        });
        out
    }

    /// Pending key 数 (测试用)
    pub fn pending_count(&self) -> usize {
        self.pending.lock().map(|g| g.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn event_debouncer_100ms_burst_merged() {
        // 守门 UT-3 (T9): 100ms 防抖 + last-write-wins 合并
        // 默认 100ms window
        let config = DebouncerConfig::for_kind(EventKind::WorktreeChanged);
        assert_eq!(config.window, Duration::from_millis(100));

        let deb = Debouncer::<Uuid>::new(config);

        // 100ms 内同 key 推 5 个 — last-write-wins, 只保留最新
        let key = "wt-1".to_string();
        let mut latest = Uuid::nil();
        for _ in 0..5 {
            latest = Uuid::new_v4();
            deb.push(key.clone(), latest);
        }
        // 1 个 pending
        assert_eq!(deb.pending_count(), 1);

        // 立刻 drain — 还没到 window, 应该 0
        let out = deb.drain_expired();
        assert_eq!(out.len(), 0);
        assert_eq!(deb.pending_count(), 1);

        // 等 150ms 后再 drain — 应返回 1 个, value 是 latest
        std::thread::sleep(Duration::from_millis(150));
        let out = deb.drain_expired();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, key);
        assert_eq!(out[0].1, latest);
        assert_eq!(deb.pending_count(), 0);
    }

    #[test]
    fn test_completed_not_debounced() {
        // 守门: TestCompleted 走 0 防抖 (即时)
        let config = DebouncerConfig::for_kind(EventKind::TestCompleted);
        assert_eq!(config.window, Duration::from_millis(0));

        let deb = Debouncer::<String>::new(config);
        deb.push("wt-1", "passed".to_string());
        // 立即 drain — 应返回
        let out = deb.drain_expired();
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn different_keys_batched_separately() {
        let config = DebouncerConfig::for_kind(EventKind::WorktreeChanged);
        let deb = Debouncer::<u32>::new(config);

        // 5 个不同 key — 5 个 pending
        for i in 0..5 {
            deb.push(format!("wt-{i}"), i as u32);
        }
        assert_eq!(deb.pending_count(), 5);

        std::thread::sleep(Duration::from_millis(150));
        let out = deb.drain_expired();
        assert_eq!(out.len(), 5);
    }
}

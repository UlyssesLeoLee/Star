// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Provider 抽象 + cache 层实装 (per spec/acceptance/08 R-007 v0.2 fix 2026-08-27,
//                                       brief OPT-WORKER-02-vcs-cache 2026-09-07)
//
// 目的: 缓解 GitHub/GitLab/Gitea API 限速风险, 为 Version Control Provider
//       (per [spec/vcs/01-version-control-provider.md] +
//        [arch/05 §5 REST API 14 endpoints]) 提供:
//       1. 通用 cache trait (`VcsCache` async: get/put/invalidate/clear)
//       2. TTL 失效策略 (per metadata `ttlMs` + `cacheScope`, per
//          [spec/mcp/01-mcp-spec.md §1.1 ④可缓存 list 结果])
//       3. Provider 集成点 (`VcsProvider` 最小 trait + `ProviderCache` read-through wrapper)
//
// 状态: Phase D 完整实装 (per OPT-CODE-68 + STAR-P4-OPT-WBS §3.2 #3 + STAR-P4-UNIMPL-WBS §5 D.1).
// 触发: Phase D 实施.
// 代签: per 2026-08-27 07:16 JST 代签规则反转, author = Mavis 接手 agent
//       (per DEC-008), committer = Ulysses Leo Lee.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::RwLock;

/// Cache 错误 (per spec/cache + R-007 4 变体, thiserror 派生).
#[derive(Debug, Error)]
pub enum CacheError {
    /// 序列化/反序列化失败 (serde_json / bincode 等).
    #[error("serialization error: {0}")]
    Serialization(String),

    /// 后端错误 (in-memory / Redis / sled 等).
    #[error("backend error: {0}")]
    Backend(String),

    /// 未实装 (e.g. Redis backend 当前为 stub, Phase G+ 启用).
    #[error("not implemented: feature={feature}, suggestion={suggestion}")]
    NotImplemented {
        /// 未实装特性名.
        feature: String,
        /// 后续实装建议路径.
        suggestion: String,
    },

    /// 无效 key (空字符串 / 超长 / 非法字符).
    #[error("invalid key: {0}")]
    InvalidKey(String),
}

impl CacheError {
    /// 错误码 (统一 `CACHE_ERROR` per spec/cache/01 §6 错误模型).
    pub fn code(&self) -> &'static str {
        "CACHE_ERROR"
    }

    /// 是否可重试 (network/connection 错误可重试, 其他不可).
    pub fn retriable(&self) -> bool {
        matches!(self, Self::Backend(_))
    }
}

/// VCS Cache 抽象 (per R-007 v0.2).
///
/// 异步 trait 涵盖 4 个核心操作: get / put / invalidate / clear.
/// 后续 Phase 2+ 可派生出 Redis / sled 等 backend (per spec/cache/01 §5).
#[async_trait]
pub trait VcsCache: Send + Sync {
    /// 取值 (TTL 过期返回 `Ok(None)`, 不区分 miss vs expired).
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// 写值 (`ttl_secs=None` 表示无过期; `Some(0)` 同样视为无过期).
    async fn put(&self, key: &str, value: Vec<u8>, ttl_secs: Option<u64>)
        -> Result<(), CacheError>;

    /// 显式失效 (key 不存在也返回 `Ok`).
    async fn invalidate(&self, key: &str) -> Result<(), CacheError>;

    /// 清空所有 (workspace 切换 / session 终止时调用).
    async fn clear(&self) -> Result<(), CacheError>;
}

/// Cache 指标 (hit / miss / put / evict 计数).
///
/// 用于可观测性: 命中率 (hit_ratio) 反映 cache 效果,
/// 驱逐数 (evictions) 反映 TTL 压力.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheMetrics {
    /// 命中次数.
    pub hits: u64,
    /// 未命中次数 (含 TTL 过期).
    pub misses: u64,
    /// 写入次数.
    pub puts: u64,
    /// 驱逐次数 (TTL 过期 + 显式 invalidate + clear).
    pub evictions: u64,
}

impl CacheMetrics {
    /// 命中率 (0.0 - 1.0; 无请求时返回 0.0).
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// 总请求数 (hits + misses).
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

struct CacheEntry {
    value: Vec<u8>,
    /// `None` 表示无过期.
    expires_at: Option<Instant>,
}

/// 进程内 VCS Cache (per R-007 MVP, 单进程场景).
///
/// 满足 MVP 阶段单进程场景; 后续 Phase 2+ 可换 Redis / sled (per spec/cache/01 §5).
/// 内部用 `tokio::sync::RwLock<HashMap>` 保护并发访问,
/// 不带 `Arc` 包装 — 调用方按需通过 `Arc<InMemoryVcsCache>` 共享.
pub struct InMemoryVcsCache {
    store: RwLock<HashMap<String, CacheEntry>>,
    metrics: RwLock<CacheMetrics>,
}

impl Default for InMemoryVcsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryVcsCache {
    /// 创建新实例.
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            metrics: RwLock::new(CacheMetrics::default()),
        }
    }

    /// 获取当前指标快照.
    pub async fn metrics(&self) -> CacheMetrics {
        *self.metrics.read().await
    }

    /// 主动驱逐已过期条目 (返回驱逐数量).
    ///
    /// 适用于周期性后台清理 (避免惰性驱逐导致内存膨胀).
    pub async fn evict_expired(&self) -> usize {
        let now = Instant::now();
        let mut store = self.store.write().await;
        let before = store.len();
        store.retain(|_, entry| match entry.expires_at {
            Some(t) => t > now,
            None => true,
        });
        let evicted = before - store.len();
        drop(store);
        if evicted > 0 {
            let mut m = self.metrics.write().await;
            m.evictions += evicted as u64;
        }
        evicted
    }

    /// 当前条目数 (含已过期但未驱逐的).
    pub async fn len(&self) -> usize {
        self.store.read().await.len()
    }

    /// 是否为空.
    pub async fn is_empty(&self) -> bool {
        self.store.read().await.is_empty()
    }

    /// 验证 key 合法性 (空字符串 / 超过 1024 字节 → `InvalidKey`).
    fn validate_key(key: &str) -> Result<(), CacheError> {
        if key.is_empty() {
            return Err(CacheError::InvalidKey("empty key".to_string()));
        }
        if key.len() > 1024 {
            return Err(CacheError::InvalidKey(format!(
                "key too long: {} bytes (max 1024)",
                key.len()
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl VcsCache for InMemoryVcsCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        Self::validate_key(key)?;
        let now = Instant::now();
        // write lock — 允许惰性驱逐过期条目
        let mut store = self.store.write().await;
        if let Some(entry) = store.get(key) {
            if let Some(t) = entry.expires_at {
                if t <= now {
                    // 过期 — 主动驱逐
                    store.remove(key);
                    drop(store);
                    let mut m = self.metrics.write().await;
                    m.misses += 1;
                    m.evictions += 1;
                    return Ok(None);
                }
            }
            let value = entry.value.clone();
            drop(store);
            let mut m = self.metrics.write().await;
            m.hits += 1;
            return Ok(Some(value));
        }
        drop(store);
        let mut m = self.metrics.write().await;
        m.misses += 1;
        Ok(None)
    }

    async fn put(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_secs: Option<u64>,
    ) -> Result<(), CacheError> {
        Self::validate_key(key)?;
        // `None` 或 `Some(0)` → 无过期 (per spec/cache/01 §5.3 语义)
        let expires_at = ttl_secs.and_then(|s| {
            if s == 0 {
                None
            } else {
                Some(Instant::now() + Duration::from_secs(s))
            }
        });
        let entry = CacheEntry { value, expires_at };
        let mut store = self.store.write().await;
        store.insert(key.to_string(), entry);
        drop(store);
        let mut m = self.metrics.write().await;
        m.puts += 1;
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        Self::validate_key(key)?;
        let mut store = self.store.write().await;
        let removed = store.remove(key).is_some();
        drop(store);
        if removed {
            let mut m = self.metrics.write().await;
            m.evictions += 1;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut store = self.store.write().await;
        let count = store.len();
        store.clear();
        drop(store);
        if count > 0 {
            let mut m = self.metrics.write().await;
            m.evictions += count as u64;
        }
        Ok(())
    }
}

/// VCS Provider 最小 trait (per R-007 v0.2 + Phase F star-sa 引用点).
///
/// `ProviderCache<P>` 通过本 trait 注入底层 provider (GitHub / GitLab / Gitea / 其它).
/// 当前 MVP 仅 `fetch(key) -> Vec<u8>`, 后续 Phase F 接入 4 Git Provider 后扩展 (e.g.
/// `get_issue_cached` / `get_worktree_cached` / `search_code_cached` per
/// `spec/vcs/01-version-control-provider.md`).
#[async_trait]
pub trait VcsProvider: Send + Sync {
    /// 拉取资源 (e.g. issue / MR / repo file).
    ///
    /// 返回 `Err(CacheError::Backend(_))` 表示 provider 故障, 不缓存错误.
    async fn fetch(&self, key: &str) -> Result<Vec<u8>, CacheError>;
}

/// Provider cache wrapper — read-through pattern.
///
/// 优先查 `inner` cache; miss 则调 `provider.fetch`, 回填 cache (默认 30s TTL per
/// `spec/mcp/01 §1.1 ④`). 替代手写 `get_issue` / `get_worktree` / `search_code` 三件套
/// (per Phase F 集成 todo).
pub struct ProviderCache<P: VcsProvider> {
    inner: InMemoryVcsCache,
    provider: P,
    /// 默认 TTL (秒).
    default_ttl_secs: u64,
}

impl<P: VcsProvider> ProviderCache<P> {
    /// 创建新实例 — 默认 TTL = 30s (per `spec/mcp/01 §1.1 ④`).
    pub fn new(provider: P) -> Self {
        Self {
            inner: InMemoryVcsCache::new(),
            provider,
            default_ttl_secs: 30,
        }
    }

    /// 自定义 TTL (秒).
    pub fn with_ttl(provider: P, default_ttl_secs: u64) -> Self {
        Self {
            inner: InMemoryVcsCache::new(),
            provider,
            default_ttl_secs,
        }
    }

    /// 内部 cache 访问.
    pub fn inner(&self) -> &InMemoryVcsCache {
        &self.inner
    }

    /// provider 引用.
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// 默认 TTL (秒).
    pub fn default_ttl_secs(&self) -> u64 {
        self.default_ttl_secs
    }

    /// 读穿透: cache miss 时调 `provider.fetch` 并回填.
    ///
    /// 1. 先查 inner cache (TTL 过期返回 `Ok(None)`)
    /// 2. miss → 调 `provider.fetch(key)`
    /// 3. put 回 inner cache (默认 TTL)
    /// 4. 返回 `Ok(Some(value))` (成功) 或 `Err(CacheError::Backend(_))` (provider 故障)
    pub async fn cached_get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        if let Some(cached) = self.inner.get(key).await? {
            return Ok(Some(cached));
        }
        let value = self.provider.fetch(key).await?;
        self.inner
            .put(key, value.clone(), Some(self.default_ttl_secs))
            .await?;
        Ok(Some(value))
    }

    /// 显式失效指定 key (proxy 到 inner cache).
    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        self.inner.invalidate(key).await
    }

    /// 清空 cache (workspace 切换时调用).
    pub async fn clear(&self) -> Result<(), CacheError> {
        self.inner.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    /// Mock VcsProvider — 计数 fetch 调用次数 + 返回固定值.
    struct MockProvider {
        call_count: Arc<AtomicU64>,
        /// fetch 返回值 (None 表示模拟 provider 错误).
        response: Option<Vec<u8>>,
    }

    #[async_trait]
    impl VcsProvider for MockProvider {
        async fn fetch(&self, _key: &str) -> Result<Vec<u8>, CacheError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            match &self.response {
                Some(v) => Ok(v.clone()),
                None => Err(CacheError::Backend("mock provider error".to_string())),
            }
        }
    }

    #[tokio::test]
    async fn test_in_memory_basic_get_put() {
        let cache = InMemoryVcsCache::new();
        // 写入 (无 TTL)
        cache.put("key1", b"value1".to_vec(), None).await.unwrap();
        // 读取 — 命中
        let got = cache.get("key1").await.unwrap();
        assert_eq!(got, Some(b"value1".to_vec()));
        // 指标校验
        let m = cache.metrics().await;
        assert_eq!(m.puts, 1);
        assert_eq!(m.hits, 1);
        assert_eq!(m.misses, 0);
        assert_eq!(m.evictions, 0);
        // 不存在的 key — miss
        let miss = cache.get("nonexistent").await.unwrap();
        assert_eq!(miss, None);
        let m = cache.metrics().await;
        assert_eq!(m.misses, 1);
    }

    #[tokio::test]
    async fn test_in_memory_ttl_expiry() {
        let cache = InMemoryVcsCache::new();
        // 写一个 1s TTL
        cache.put("k1", b"v1".to_vec(), Some(1)).await.unwrap();
        // 立即读 — 命中
        let got = cache.get("k1").await.unwrap();
        assert_eq!(got, Some(b"v1".to_vec()));
        // 等待 1.1s 过期
        tokio::time::sleep(Duration::from_millis(1100)).await;
        // 再次读 — miss (且 evict)
        let got = cache.get("k1").await.unwrap();
        assert_eq!(got, None);
        let m = cache.metrics().await;
        assert_eq!(m.misses, 1);
        assert_eq!(m.evictions, 1);
        // store 应已主动驱逐
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_in_memory_eviction() {
        let cache = InMemoryVcsCache::new();
        // 写 3 个 key
        cache.put("a", b"1".to_vec(), None).await.unwrap();
        cache.put("b", b"2".to_vec(), None).await.unwrap();
        cache.put("c", b"3".to_vec(), None).await.unwrap();
        assert_eq!(cache.len().await, 3);
        // 显式 invalidate
        cache.invalidate("b").await.unwrap();
        assert_eq!(cache.len().await, 2);
        assert_eq!(cache.get("b").await.unwrap(), None);
        // 指标
        let m = cache.metrics().await;
        assert_eq!(m.evictions, 1);
        // 清空
        cache.clear().await.unwrap();
        assert_eq!(cache.len().await, 0);
        let m = cache.metrics().await;
        // clear 算 2 个 eviction (a + c)
        assert_eq!(m.evictions, 3);
        // invalidate 不存在的 key 不报错
        cache.invalidate("nope").await.unwrap();
    }

    #[tokio::test]
    async fn test_provider_cache_hit_miss() {
        let call_count = Arc::new(AtomicU64::new(0));
        let provider = MockProvider {
            call_count: Arc::clone(&call_count),
            response: Some(b"hello".to_vec()),
        };
        let cache = ProviderCache::new(provider);
        // 第一次 — miss, provider 被调用 1 次
        let got = cache.cached_get("repo:1:readme").await.unwrap();
        assert_eq!(got, Some(b"hello".to_vec()));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        // 第二次 — hit, provider 不再被调用
        let got = cache.cached_get("repo:1:readme").await.unwrap();
        assert_eq!(got, Some(b"hello".to_vec()));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        // 验证 cache 内容已回填
        let inner = cache.inner();
        let cached = inner.get("repo:1:readme").await.unwrap();
        assert_eq!(cached, Some(b"hello".to_vec()));
        // invalidate 后再次 miss
        cache.invalidate("repo:1:readme").await.unwrap();
        let got = cache.cached_get("repo:1:readme").await.unwrap();
        assert_eq!(got, Some(b"hello".to_vec()));
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    /// 额外覆盖: empty key → InvalidKey.
    #[tokio::test]
    async fn test_invalid_key() {
        let cache = InMemoryVcsCache::new();
        let err = cache.get("").await.unwrap_err();
        assert!(matches!(err, CacheError::InvalidKey(_)));
        let err = cache.put("", b"x".to_vec(), None).await.unwrap_err();
        assert!(matches!(err, CacheError::InvalidKey(_)));
    }

    /// 额外覆盖: provider 错误不缓存.
    #[tokio::test]
    async fn test_provider_error_not_cached() {
        let call_count = Arc::new(AtomicU64::new(0));
        let provider = MockProvider {
            call_count: Arc::clone(&call_count),
            response: None, // 模拟错误
        };
        let cache = ProviderCache::new(provider);
        // 错误透传
        let err = cache.cached_get("k").await.unwrap_err();
        assert!(matches!(err, CacheError::Backend(_)));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        // cache 不应回填 (因为没 put)
        let inner = cache.inner();
        let cached = inner.get("k").await.unwrap();
        assert_eq!(cached, None);
    }
}

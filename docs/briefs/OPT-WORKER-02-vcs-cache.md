# Brief: OPT-WORKER-02 — star-vcs/src/cache.rs 完整空壳实装 (per OPT-CODE-68, R-007 cache)

**Agent**: worker
**Phase**: OPT-P4
**Created**: 2026-09-07 12:04 JST
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses

---

## 1. 任务目标

实装 `crates/star-vcs/src/cache.rs` 完整空壳 (77 行, 当前只有 `pub struct VcsCache;` + `pub struct CacheError;` + 1 个 placeholder test + 4 个 TODO 块)。

**当前状态 (per OPT-A1 §3.2 实证)**:
- `pub struct VcsCache;` 空 struct
- `pub struct CacheError;` 空 struct (应 thiserror enum)
- 4 TODO 块: cache-trait / impl-inmem / provider-integration / tests
- 1 个 `#[test] fn placeholder() { /* 仅占位 */ }` 空测试
- `#![allow(dead_code)]` + `#![allow(unused_imports)]` 抑制 warning

**目标 (per守门 #4 v18 + P4-UNIMPL-WBS §5 D.1 实证)**:
- 实现 VcsCache trait (cache-trait 块)
- 实现 in-memory backend (impl-inmem 块)
- 集成 VCS provider (provider-integration 块, Git/VCS 客户端)
- 加 unit test + integration test (tests 块)

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-vcs-cache D:/Star/.worktrees/wt-opt-vcs-cache main
cd D:/Star/.worktrees/wt-opt-vcs-cache
```

- **Worktree 路径**: `D:/Star/.worktrees/wt-opt-vcs-cache`
- **Branch**: `feat/opt-vcs-cache` (基于 main HEAD `bfb0bca`)

---

## 3. 实装要求

### 3.1 VcsCache trait (cache-trait 块)

```rust
use async_trait::async_trait;  // 或自实现 (per workspace 已有依赖)
use uuid::Uuid;

#[async_trait]
pub trait VcsCache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;
    async fn put(&self, key: &str, value: Vec<u8>, ttl_secs: Option<u64>) -> Result<(), CacheError>;
    async fn invalidate(&self, key: &str) -> Result<(), CacheError>;
    async fn clear(&self) -> Result<(), CacheError>;
}
```

### 3.2 CacheError 完整变体 (替换空 struct)

```rust
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("backend error: {0}")]
    Backend(String),
    #[error("not implemented: feature={feature}, suggestion={suggestion}")]
    NotImplemented { feature: String, suggestion: String },
    #[error("invalid key: {0}")]
    InvalidKey(String),
}
```

### 3.3 InMemoryBackend impl (impl-inmem 块)

```rust
pub struct InMemoryVcsCache {
    store: tokio::sync::RwLock<std::collections::HashMap<String, (Vec<u8>, Option<std::time::Instant>)>>,
    metrics: CacheMetrics,
}

impl InMemoryVcsCache {
    pub fn new() -> Self { ... }
    fn evict_expired(&self) { ... }
}

#[async_trait]
impl VcsCache for InMemoryVcsCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> { ... }
    async fn put(&self, key: &str, value: Vec<u8>, ttl_secs: Option<u64>) -> Result<(), CacheError> { ... }
    async fn invalidate(&self, key: &str) -> Result<(), CacheError> { ... }
    async fn clear(&self) -> Result<(), CacheError> { ... }
}
```

加 `CacheMetrics` struct (hit/miss/put/evict 计数)。

### 3.4 Provider integration (provider-integration 块)

```rust
// 在 crates/star-vcs/src/cache.rs 加 provider cache wrapper
pub struct ProviderCache<P: VcsProvider> {
    inner: InMemoryVcsCache,
    provider: P,
}

impl<P: VcsProvider> ProviderCache<P> {
    pub fn new(provider: P) -> Self { ... }
    pub async fn cached_get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        // 1. 先查 cache
        // 2. miss 则调 provider.fetch
        // 3. put 回 cache
    }
}
```

### 3.5 Tests (tests 块)

替换 placeholder test, 加 4 类 test:
1. `test_in_memory_basic_get_put` — 测基本 get/put
2. `test_in_memory_ttl_expiry` — 测 TTL 过期
3. `test_in_memory_eviction` — 测驱逐
4. `test_provider_cache_hit_miss` — 测 provider cache 包装

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_in_memory_basic_get_put() {
        let cache = InMemoryVcsCache::new();
        cache.put("key1", b"value1".to_vec(), None).await.unwrap();
        let got = cache.get("key1").await.unwrap();
        assert_eq!(got, Some(b"value1".to_vec()));
    }
    
    // ... 其他 3 个 test
}
```

---

## 4. 守门硬约束

1. `cargo check --workspace --all-targets -j 4` 必须 0 err
2. `cargo test -p star-vcs --lib -j 4` 必须全 pass (估 ~10-20 tests)
3. `cargo fmt + clippy` 必须干净
4. 禁回溯叙事 + 禁 RGS 引用 + PowerShell only
5. 移除 `#![allow(dead_code)]` + `#![allow(unused_imports)]` (现在代码实装了, 不再需要)

---

## 5. Commit 形式

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(star-vcs): 实装 R-007 cache layer (per OPT-CODE-68)

- VcsCache trait (async get/put/invalidate/clear)
- CacheError 完整 thiserror enum (4 变体)
- InMemoryVcsCache impl + CacheMetrics
- ProviderCache<P> wrapper 集成 VcsProvider
- 替换 placeholder test, 加 4 类 test (basic/ttl/eviction/provider)

Refs: docs/briefs/OPT-WORKER-02-vcs-cache.md
Refs: docs/reports/STAR-P4-OPT-WBS-001.md §3.2 #3
Refs: STAR-P4-UNIMPL-WBS-001.md §5 D.1 (R-007 cache)
Per守门 #10 author=Ulysses
"
```

---

## 6. 交付物

返回时报告:
1. worktree 路径: `D:/Star/.worktrees/wt-opt-vcs-cache`
2. branch 名称: `feat/opt-vcs-cache`
3. commit hash 7 字符
4. 修改文件列表 (file:line)
5. `cargo check --workspace --all-targets -j 4` 输出 (必须 0 err)
6. `cargo test -p star-vcs --lib -j 4` 输出 (必须全 pass)

---

## 7. 失败处理

- cargo check 不通过: 修复 + 重试, max 2 次
- 仍失败: 报告具体错误, 不 commit

---

## 8. 时间预算

≤ 500K token (因 cache 实装涉及 async + tokio + thiserror + 多 test, 估耗)

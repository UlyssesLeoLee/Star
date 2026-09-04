//! 应用层: Cache trait (per docs/basic-design §7.1, 5min TTL Redis)
//!
//! 阶段 1 走 InMemory 实现, Redis 留 V2 (per brief §1 已知缺口 #1)
//!
//! 不用 generic (dyn trait 不支持), 用 serde_json::Value 中转, 调用方自行 serialize/deserialize

use async_trait::async_trait;

/// 缓存能力抽象(阶段 1 走 InMemory, Redis 留 V2)
#[async_trait]
pub trait Cache: Send + Sync {
    /// 取缓存, 命中返 Some(Value), miss 返 None
    async fn get_json(&self, key: &str) -> Result<Option<serde_json::Value>, String>;

    /// 写缓存, ttl_seconds = 0 表示永不过期
    async fn set_json(
        &self,
        key: &str,
        value: &serde_json::Value,
        ttl_seconds: u64,
    ) -> Result<(), String>;

    /// 失效指定 key
    async fn invalidate(&self, key: &str) -> Result<(), String>;
}

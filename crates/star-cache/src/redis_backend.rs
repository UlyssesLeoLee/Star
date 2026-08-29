// Redis backend (per spec/cache/01 §5 Redis backend)
// Phase G+ stub — 仅占位 URL 解析,所有方法返回 CacheError::Other。
// 缺标比错标安全: Phase G 阶段不实装,避免编造半成品行为。
use super::*;

/// Redis 后端 (Phase G+ stub)
pub struct RedisBackend {
    /// Redis 连接 URL — 来自 REDIS_URL 环境变量
    /// 8/27 11:06 JST secret 安全: 错误消息不打印 URL
    #[allow(dead_code)]
    pub url: String,
}

impl RedisBackend {
    /// 从环境变量 `REDIS_URL` 构造实例
    /// 错误消息仅说明"unset",不打印任何 URL 内容。
    pub fn from_env() -> Result<Self, CacheError> {
        let url = std::env::var("REDIS_URL")
            .map_err(|_| CacheError::Connection("REDIS_URL unset".into()))?;
        Ok(Self { url })
    }
}

#[async_trait]
impl CacheBackend for RedisBackend {
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        Err(CacheError::Other("Phase G+ 待实装 — 当前 stub".into()))
    }

    async fn set(&self, _key: &str, _value: &[u8], _ttl_sec: u32) -> Result<(), CacheError> {
        Err(CacheError::Other("Phase G+ 待实装 — 当前 stub".into()))
    }

    async fn del(&self, _key: &str) -> Result<(), CacheError> {
        Err(CacheError::Other("Phase G+ 待实装 — 当前 stub".into()))
    }

    async fn exists(&self, _key: &str) -> Result<bool, CacheError> {
        Err(CacheError::Other("Phase G+ 待实装 — 当前 stub".into()))
    }

    async fn incr(&self, _key: &str, _delta: i64) -> Result<i64, CacheError> {
        Err(CacheError::Other("Phase G+ 待实装 — 当前 stub".into()))
    }

    async fn expire(&self, _key: &str, _ttl_sec: u32) -> Result<(), CacheError> {
        Err(CacheError::Other("Phase G+ 待实装 — 当前 stub".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_ok() {
        std::env::set_var("REDIS_URL", "redis://x");
        assert!(RedisBackend::from_env().is_ok());
    }

    #[test]
    fn from_env_unset() {
        std::env::remove_var("REDIS_URL");
        let r = RedisBackend::from_env();
        match r {
            Err(CacheError::Connection(msg)) => assert!(msg.contains("unset")),
            Err(_) => panic!("expected Connection error"),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }
}

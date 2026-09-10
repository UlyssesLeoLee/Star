// Valkey backend (per spec/cache/01 §5 Valkey backend)
// Phase G+ stub — 仅占位 URL 解析,所有方法返回 CacheError::Other。
// 缺标比错标安全: Phase G 阶段不实装,避免编造半成品行为。
//
// 选型 (per docs/operation-design.md §4.4 + docs/data-design.md §13.1):
//   Valkey (Linux Foundation 维护的 Redis 开源 fork, BSD-3-Clause) 替代 Redis。
//   原因: Redis 7.4+ 改 RSALv2/SSPL (双重源码可用许可), 不再算 OSI 开源;
//        Valkey 保持 BSD-3-Clause 全开源, 协议 100% 兼容 Redis。
//   env var: VALKEY_URL (per docs/operation-design.md line 309)
use super::*;

/// Valkey 后端 (Phase G+ stub)
pub struct ValkeyBackend {
    /// Valkey 连接 URL — 来自 VALKEY_URL 环境变量
    /// 8/27 11:06 JST secret 安全: 错误消息不打印 URL
    #[allow(dead_code)]
    pub url: String,
}

impl ValkeyBackend {
    /// 从环境变量 `VALKEY_URL` 构造实例
    /// 错误消息仅说明"unset",不打印任何 URL 内容。
    pub fn from_env() -> Result<Self, CacheError> {
        let url = std::env::var("VALKEY_URL")
            .map_err(|_| CacheError::Connection("VALKEY_URL unset".into()))?;
        Ok(Self { url })
    }
}

#[async_trait]
impl CacheBackend for ValkeyBackend {
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
        std::env::set_var("VALKEY_URL", "valkey://x");
        assert!(ValkeyBackend::from_env().is_ok());
    }

    #[test]
    fn from_env_unset() {
        std::env::remove_var("VALKEY_URL");
        let r = ValkeyBackend::from_env();
        match r {
            Err(CacheError::Connection(msg)) => assert!(msg.contains("unset")),
            Err(_) => panic!("expected Connection error"),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }
}

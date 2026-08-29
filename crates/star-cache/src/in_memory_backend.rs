// In-memory LRU backend (per spec/cache/01 §5)
// Tokio RwLock + HashMap + TTL 过期。
// 注: 当前实现是基础 Map + TTL,LRU 淘汰 (max_size) 留待 Phase G+ 实装。
use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

struct Entry {
    value: Vec<u8>,
    /// Unix 时间戳 (秒) — 0 表示不过期
    expires_at: u64,
}

/// 进程内缓存后端 (per spec/cache/01 §5)
pub struct InMemoryBackend {
    inner: Arc<RwLock<HashMap<String, Entry>>>,
    /// 默认 TTL (秒) — set 时 ttl_sec=0 时启用
    #[allow(dead_code)]
    default_ttl: u32,
}

impl InMemoryBackend {
    /// 创建新实例
    pub fn new(default_ttl: u32) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

#[async_trait]
impl CacheBackend for InMemoryBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let g = self.inner.read().await;
        if let Some(e) = g.get(key) {
            if e.expires_at == 0 || e.expires_at > Self::now() {
                return Ok(Some(e.value.clone()));
            }
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &[u8], ttl_sec: u32) -> Result<(), CacheError> {
        let mut g = self.inner.write().await;
        let expires_at = if ttl_sec == 0 {
            0
        } else {
            Self::now() + ttl_sec as u64
        };
        g.insert(
            key.to_string(),
            Entry {
                value: value.to_vec(),
                expires_at,
            },
        );
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<(), CacheError> {
        let mut g = self.inner.write().await;
        g.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        Ok(self.get(key).await?.is_some())
    }

    async fn incr(&self, _key: &str, _delta: i64) -> Result<i64, CacheError> {
        // Phase G+ 实装 — 当前为 stub
        Err(CacheError::Other(
            "incr not implemented for in-memory".into(),
        ))
    }

    async fn expire(&self, _key: &str, _ttl_sec: u32) -> Result<(), CacheError> {
        // Phase G+ 实装 — 当前为 stub
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn set_get() {
        let c = InMemoryBackend::new(60);
        c.set("k", b"v", 60).await.unwrap();
        assert_eq!(c.get("k").await.unwrap(), Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn del_removes_key() {
        let c = InMemoryBackend::new(60);
        c.set("k", b"v", 60).await.unwrap();
        c.del("k").await.unwrap();
        assert_eq!(c.get("k").await.unwrap(), None);
    }

    #[tokio::test]
    async fn exists_after_set() {
        let c = InMemoryBackend::new(60);
        assert!(!c.exists("k").await.unwrap());
        c.set("k", b"v", 60).await.unwrap();
        assert!(c.exists("k").await.unwrap());
    }

    #[tokio::test]
    async fn ttl_zero_means_no_expire() {
        let c = InMemoryBackend::new(0);
        c.set("k", b"v", 0).await.unwrap();
        assert_eq!(c.get("k").await.unwrap(), Some(b"v".to_vec()));
    }
}

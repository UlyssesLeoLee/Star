//! 基础设施: InMemoryCache (per star-cache crate, 阶段 1 简化实装)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::application::cache::Cache;

pub struct InMemoryCache {
    store: RwLock<HashMap<String, (serde_json::Value, Option<Instant>)>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self { store: RwLock::new(HashMap::new()) }
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get_json(&self, key: &str) -> Result<Option<serde_json::Value>, String> {
        let store = self.store.read().map_err(|e| e.to_string())?;
        if let Some((val, expires_at)) = store.get(key) {
            if let Some(exp) = expires_at {
                if Instant::now() > *exp {
                    drop(store);
                    let mut wstore = self.store.write().map_err(|e| e.to_string())?;
                    wstore.remove(key);
                    return Ok(None);
                }
            }
            Ok(Some(val.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set_json(
        &self,
        key: &str,
        value: &serde_json::Value,
        ttl_seconds: u64,
    ) -> Result<(), String> {
        let expires_at = if ttl_seconds > 0 {
            Some(Instant::now() + Duration::from_secs(ttl_seconds))
        } else {
            None
        };
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        store.insert(key.to_string(), (value.clone(), expires_at));
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), String> {
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        store.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestVal { x: i32, y: String }

    #[tokio::test]
    async fn test_set_get() {
        let c = InMemoryCache::new();
        let val = serde_json::to_value(TestVal { x: 1, y: "hi".into() }).unwrap();
        c.set_json("k1", &val, 60).await.unwrap();
        let v = c.get_json("k1").await.unwrap();
        assert_eq!(v, Some(val));
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let c = InMemoryCache::new();
        let val = serde_json::to_value(TestVal { x: 2, y: "hi".into() }).unwrap();
        c.set_json("k2", &val, 0).await.unwrap();  // 永不过期
        let v = c.get_json("k2").await.unwrap();
        assert!(v.is_some());
    }

    #[tokio::test]
    async fn test_invalidate() {
        let c = InMemoryCache::new();
        let val = serde_json::to_value(TestVal { x: 3, y: "hi".into() }).unwrap();
        c.set_json("k3", &val, 60).await.unwrap();
        c.invalidate("k3").await.unwrap();
        let v = c.get_json("k3").await.unwrap();
        assert!(v.is_none());
    }
}

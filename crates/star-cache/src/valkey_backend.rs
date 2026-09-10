// Valkey backend (per spec/cache/01 §2.3 + 选型决定 2026-09-10 23:41 JST)
// Phase G+ 完整实装 — `redis-rs` 0.27+ (tokio-comp + connection-manager features)
// 协议 100% 兼容 Valkey (Redis RESP2/RESP3)
//
// 状态: Phase G+ 完整实装 (per spec/cache/01 §2.3 选型决定 + OPT-NEXT-04-phase-g §3.2)
// 触发: Phase G+ 实施.
// 代签: per 2026-08-27 19:39 JST 用户授权 + 守门 #10
//       author = Ulysses <ulysses@mavis.local>, committer = Ulysses Leo Lee.

use std::sync::Arc;

use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tokio::sync::Mutex;

use super::{CacheBackend, CacheError};

/// Valkey 后端 (Phase G+ 完整实装)
///
/// 内部用 `Arc<Mutex<Option<ConnectionManager>>>` 管理连接:
/// - `Arc` 让后端可以 clone + 跨 await 共享
/// - `Mutex<Option<>>` 让 `connect()` 可以延迟到第一次使用前
/// - `ConnectionManager` 自带 reconnect, 适合长跑 cache 服务
///
/// 用法 (典型):
/// ```ignore
/// let backend = ValkeyBackend::from_env_connected().await?;
/// backend.set("k", b"v", 60).await?;
/// let v = backend.get("k").await?;
/// ```
pub struct ValkeyBackend {
    conn: Arc<Mutex<Option<ConnectionManager>>>,
    /// VALKEY_URL (per docs/operation-design.md line 309)
    /// 8/27 11:06 JST secret 安全: 错误消息不打印 URL
    #[allow(dead_code)]
    url: String,
}

impl ValkeyBackend {
    /// 从环境变量 `VALKEY_URL` 读取 URL, 构造 (未连接) 实例 (sync)
    ///
    /// **不** 验证 URL 合法性, **不** 建立连接 — 这两步在 `connect()` 异步做.
    /// 这样 env-var 读取逻辑可以 sync 测, 不依赖运行时.
    pub fn from_env() -> Result<Self, CacheError> {
        let url = std::env::var("VALKEY_URL")
            .map_err(|_| CacheError::Connection("VALKEY_URL unset".into()))?;
        Ok(Self::from_url(url))
    }

    /// 给定 URL, 构造 (未连接) 实例 (sync)
    pub fn from_url(url: String) -> Self {
        Self {
            conn: Arc::new(Mutex::new(None)),
            url,
        }
    }

    /// 异步建立到 Valkey 的连接 (per `redis::aio::ConnectionManager::new`)
    ///
    /// 内部用 `ConnectionManager` 而不是一次性 `MultiplexedConnection`,
    /// 因为 ConnectionManager 自带 reconnect, 适合 cache 这种长跑服务.
    /// 错误消息不打印 URL 内容 (per 8/27 11:06 JST secret 安全).
    pub async fn connect(&self) -> Result<(), CacheError> {
        let client = redis::Client::open(self.url.as_str())
            .map_err(|e| CacheError::Connection(format!("invalid URL: {e}")))?;
        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::Connection(format!("connect failed: {e}")))?;
        let mut guard = self.conn.lock().await;
        *guard = Some(manager);
        Ok(())
    }

    /// 便捷方法: 从 env 读 URL + async 连接 (一次性完成)
    pub async fn from_env_connected() -> Result<Self, CacheError> {
        let backend = Self::from_env()?;
        backend.connect().await?;
        Ok(backend)
    }

    /// 健康检查: PING 命令
    ///
    /// 未连接时返 `CacheError::Connection`, 已连接但 PING 失败返 `CacheError::Network`.
    pub async fn ping(&self) -> Result<(), CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        let pong: String = redis::cmd("PING")
            .query_async(conn)
            .await
            .map_err(|e| CacheError::Network(e.to_string()))?;
        if pong != "PONG" {
            return Err(CacheError::Other(format!("unexpected PING reply: {pong}")));
        }
        Ok(())
    }
}

#[async_trait]
impl CacheBackend for ValkeyBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        let val: Option<Vec<u8>> = conn
            .get(key)
            .await
            .map_err(|e| CacheError::Network(e.to_string()))?;
        Ok(val)
    }

    async fn set(&self, key: &str, value: &[u8], ttl_sec: u32) -> Result<(), CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        // ttl_sec=0 含义: 不设过期 (per trait doc)
        // Redis 语义: SETEX 必须 ttl>0, 不传 ttl 用 SET K V
        if ttl_sec == 0 {
            let _: () = conn
                .set(key, value)
                .await
                .map_err(|e| CacheError::Network(e.to_string()))?;
        } else {
            let _: () = conn
                .set_ex(key, value, ttl_sec as u64)
                .await
                .map_err(|e| CacheError::Network(e.to_string()))?;
        }
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<(), CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        let _: i64 = conn
            .del(key)
            .await
            .map_err(|e| CacheError::Network(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| CacheError::Network(e.to_string()))?;
        Ok(exists)
    }

    async fn incr(&self, key: &str, delta: i64) -> Result<i64, CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        let val: i64 = conn
            .incr(key, delta)
            .await
            .map_err(|e| CacheError::Network(e.to_string()))?;
        Ok(val)
    }

    async fn expire(&self, key: &str, ttl_sec: u32) -> Result<(), CacheError> {
        let mut guard = self.conn.lock().await;
        let conn = guard
            .as_mut()
            .ok_or_else(|| CacheError::Connection("not connected; call connect() first".into()))?;
        // Redis EXPIRE 返回 bool (1 = 成功设了 TTL, 0 = key 不存在)
        // CacheBackend trait 要求 (), 丢弃 bool
        let _: bool = conn
            .expire(key, ttl_sec as i64)
            .await
            .map_err(|e| CacheError::Network(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_ok() {
        std::env::set_var("VALKEY_URL", "valkey://x");
        let backend = ValkeyBackend::from_env();
        assert!(backend.is_ok());
        // url 字段保留 (但 # err 消息不打印)
        assert_eq!(backend.unwrap().url, "valkey://x");
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

    #[test]
    fn from_url_constructs_without_connecting() {
        let backend = ValkeyBackend::from_url("valkey://example:6379".to_string());
        assert_eq!(backend.url, "valkey://example:6379");
        // conn 字段是 None (未连接)
        // 注意: 不能直接访问 conn (private 字段), 但可以通过 ping() 间接验证
    }

    #[tokio::test]
    async fn ping_fails_when_not_connected() {
        let backend = ValkeyBackend::from_url("valkey://example:6379".to_string());
        let r = backend.ping().await;
        match r {
            Err(CacheError::Connection(msg)) => assert!(msg.contains("not connected")),
            Err(_) => panic!("expected Connection error"),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }

    #[tokio::test]
    async fn get_fails_when_not_connected() {
        let backend = ValkeyBackend::from_url("valkey://example:6379".to_string());
        let r = backend.get("k").await;
        assert!(matches!(r, Err(CacheError::Connection(_))));
    }

    #[tokio::test]
    async fn connect_fails_with_unreachable_host() {
        // 127.0.0.1:1 几乎确定不可达, 但避免依赖网络状态, 只验证 "返回 Err"
        let backend = ValkeyBackend::from_url("valkey://127.0.0.1:1".to_string());
        let r = backend.connect().await;
        assert!(matches!(r, Err(CacheError::Connection(_))));
    }

    #[tokio::test]
    async fn connect_fails_with_invalid_url_scheme() {
        let backend = ValkeyBackend::from_url("not-a-redis-url://x".to_string());
        let r = backend.connect().await;
        // 错误: invalid URL (redis-rs 拒收未知 scheme)
        assert!(matches!(r, Err(CacheError::Connection(_))));
    }

    // === 集成测试 (需要真实 Valkey 实例) ===
    // 启用条件: k3s-local cluster 部署 Valkey 落地 (per WBS §14.15 跨 session 续)
    // 当前 dev env 无 Docker daemon, testcontainers-rs 不可用 (per 守门 #24 v2)
    // 落地后用 `cargo test -p star-cache -- --ignored` 跑
    //
    // #[tokio::test]
    // #[ignore]
    // async fn integration_set_get_del() {
    //     let backend = ValkeyBackend::from_env_connected().await.expect("connect failed");
    //     backend.set("itest:k1", b"v1", 60).await.unwrap();
    //     assert_eq!(backend.get("itest:k1").await.unwrap(), Some(b"v1".to_vec()));
    //     backend.del("itest:k1").await.unwrap();
    //     assert_eq!(backend.get("itest:k1").await.unwrap(), None);
    // }
    //
    // #[tokio::test]
    // #[ignore]
    // async fn integration_incr_expire() {
    //     let backend = ValkeyBackend::from_env_connected().await.expect("connect failed");
    //     backend.del("itest:counter").await.unwrap();
    //     assert_eq!(backend.incr("itest:counter", 1).await.unwrap(), 1);
    //     assert_eq!(backend.incr("itest:counter", 5).await.unwrap(), 6);
    //     backend.expire("itest:counter", 1).await.unwrap();
    //     assert!(backend.exists("itest:counter").await.unwrap());
    //     tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    //     assert!(!backend.exists("itest:counter").await.unwrap());
    // }
    //
    // #[tokio::test]
    // #[ignore]
    // async fn integration_ping() {
    //     let backend = ValkeyBackend::from_env_connected().await.expect("connect failed");
    //     backend.ping().await.expect("PING should succeed");
    // }
}

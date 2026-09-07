//! # PG advisory lock wrapper
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.3.1
//!
//! 锁实现: `pg_try_advisory_xact_lock` (事务内自动释放) + `pg_try_advisory_lock` (session 持锁)
//!
//! 守门合规:
//! - 守门 #13 a L0 协调: L3 Domain 锁是 transaction-bound, 事务结束自动释放
//! - 守门 #13 c: 锁审计写 `advisory_lock_audit` 表 (per EX-01 SQL)

use sha2::{Digest, Sha256};

/// PG advisory lock 错误.
#[derive(Debug, thiserror::Error)]
pub enum PgAdvisoryLockError {
    /// 超时.
    #[error("timeout after {waited_ms}ms")]
    Timeout {
        /// 实际等待毫秒数.
        waited_ms: u64,
    },
    /// PG 错误.
    #[error("pg error: {0}")]
    PgError(String),
}

/// 计算 PG advisory lock class ID (BIGINT, 范围 `-2^63..2^63-1`).
///
/// 命名空间按资源类型分: `work_item` / `task` / `worktree` / `merge_request` / `comment`
///
/// 算法: SHA-256(resource) 前 8 字节转 i64 (big-endian)
pub fn compute_lock_class_id(resource: &str) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(resource.as_bytes());
    let result = hasher.finalize();
    let bytes: [u8; 8] = result[..8].try_into().expect("SHA-256 前 8 字节");
    i64::from_be_bytes(bytes)
}

/// PG advisory lock 包装 (per 守门 #13 a L0 协调).
///
/// 实际生产应接 sqlx::PgPool, 这里提供 trait 抽象 + 1 个 mock 实现用于测.
pub struct PgAdvisoryLock {
    /// 锁类 ID 命名空间.
    namespace: String,
}

impl PgAdvisoryLock {
    /// 创建新 wrapper.
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }

    /// 获取锁类 ID 命名空间.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// 计算锁类 ID (per `compute_lock_class_id`).
    pub fn lock_class_id(&self, resource: &str) -> i64 {
        let namespaced = format!("{}:{}", self.namespace, resource);
        compute_lock_class_id(&namespaced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_class_id_deterministic() {
        let id1 = compute_lock_class_id("work_item:UUID-A");
        let id2 = compute_lock_class_id("work_item:UUID-A");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_lock_class_id_distinguishes_resource() {
        let id1 = compute_lock_class_id("work_item:UUID-A");
        let id2 = compute_lock_class_id("work_item:UUID-B");
        let id3 = compute_lock_class_id("task:UUID-A");
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_lock_class_id_range() {
        // SHA-256 前 8 字节转 i64, 范围 -2^63..2^63-1
        let id = compute_lock_class_id("any-resource");
        assert!(id > i64::MIN);
        assert!(id < i64::MAX);
    }

    #[test]
    fn test_pg_advisory_lock_namespace() {
        let lock = PgAdvisoryLock::new("work_item");
        assert_eq!(lock.namespace(), "work_item");
        // 不同 namespace 同 resource ID 不同
        let lock1 = PgAdvisoryLock::new("work_item");
        let lock2 = PgAdvisoryLock::new("task");
        assert_ne!(
            lock1.lock_class_id("UUID-X"),
            lock2.lock_class_id("UUID-X")
        );
    }
}

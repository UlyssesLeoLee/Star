//! # star-mutex - Star 排他与幂等架构 view (Star-EI) 共享 mutex crate
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/`
//! per `ADR-0048` 4 拍板项 (PG advisory / 双键 / 4 层 / exclusion-idempotency)
//! per `AGENTS.md §4` 守门 #13 a L0 协调 + #13 c/d RLS 13 类 + audit 必携
//!
//! 守门合规:
//! - 守门 #7 0 unsafe (per `Cargo.toml` `unsafe_code = "forbid"`)
//! - 守门 #13 a L0 协调: L2 SubAgent 不直接调 L3 Domain 互抢, 必须经 L0 派发
//! - 守门 #13 c: 5 张新表 100% RLS 13 类必携 (per EX-01 SQL migration)
//! - 守门 #13 d: 4 张 T 表 100% audit trigger 必携 (per ADR-0043 WORM)
//! - 守门 #1 v25: cargo test 单 crate, 跳过 workspace

#![doc = "Domain mutex trait + PG advisory lock + Version CAS 乐观锁"]

use async_trait::async_trait;
use uuid::Uuid;

pub mod advisory_lock;
pub mod audit;
pub mod policy_loader;
pub mod trace;
pub mod version_cas;

pub use advisory_lock::{compute_lock_class_id, PgAdvisoryLock, PgAdvisoryLockError};
pub use audit::LockAuditLogger;
pub use policy_loader::ExclusionPolicyLoader;
pub use trace::TraceIdPropagator;
pub use version_cas::{VersionCAS, VersionMismatchError};

/// Domain 业务行锁 trait (per ADR-0048 D-03 L3 Domain 责任).
///
/// 实现:
/// - `PgAdvisoryLockMutex` (生产, PG advisory_xact_lock)
/// - 测试用 mock 实现 (per H2 v18 实证 5/5 subagent RPC 不可靠)
#[async_trait]
pub trait DomainMutex: Send + Sync {
    /// 尝试获取业务行锁, transaction-bound.
    ///
    /// Args:
    /// - `resource`: e.g. "work_item:UUID"
    /// - `timeout_ms`: 5s 默认 (per F-04)
    ///
    /// Raises:
    /// - `DomainMutexError::Timeout` 超时
    /// - `DomainMutexError::PgError` PG 错误
    async fn try_lock_with_timeout(
        &self,
        resource: &str,
        timeout_ms: u64,
    ) -> Result<MutexGuard, DomainMutexError>;

    /// 基于 version CAS 的乐观锁写入.
    ///
    /// 1. 获取业务行锁
    /// 2. 读当前 version
    /// 3. 校验 `expected_version == current`
    /// 4. 执行业务 + 写新 version (`expected_version + 1`)
    async fn version_cas<F, Fut, T>(
        &self,
        resource: &str,
        expected_version: u64,
        f: F,
    ) -> Result<T, DomainMutexError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<(T, u64), DomainMutexError>> + Send;

    /// 检查资源是否被其他事务持有.
    async fn is_locked(&self, resource: &str) -> Result<bool, DomainMutexError>;
}

/// Mutex guard, Drop 时自动释放 (per `pg_advisory_xact_lock` 事务结束自动释放).
pub struct MutexGuard {
    resource: String,
    lock_class_id: i64,
    acquired_at: chrono::DateTime<chrono::Utc>,
}

impl MutexGuard {
    /// 获取资源标识.
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// 获取锁类 ID (per `compute_lock_class_id`).
    pub fn lock_class_id(&self) -> i64 {
        self.lock_class_id
    }

    /// 获取获取时刻.
    pub fn acquired_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.acquired_at
    }
}

/// Domain mutex 错误 (per 守门 #11 缺标比错标安全).
#[derive(Debug, thiserror::Error)]
pub enum DomainMutexError {
    /// 锁获取超时.
    #[error("timeout after {timeout_ms}ms waiting for {resource}")]
    Timeout {
        /// 资源标识 (e.g. "work_item:UUID").
        resource: String,
        /// 超时毫秒数.
        timeout_ms: u64,
    },

    /// version CAS 不匹配, 数据已被修改.
    #[error("version mismatch: expected {expected}, actual {actual} for {resource}")]
    VersionMismatch {
        /// 期望 version.
        expected: u64,
        /// 实际 current version.
        actual: u64,
        /// 资源标识.
        resource: String,
    },

    /// PG 错误.
    #[error("pg advisory lock failed: {0}")]
    PgError(String),

    /// audit 写入失败.
    #[error("audit write failed: {0}")]
    AuditError(String),
}

/// 资源标识解析 (per 守门 #13 a L0 协调派生规).
///
/// 命名空间按资源类型分:
/// - `work_item:` / `task:` / `worktree:` / `merge_request:` / `comment:`
pub fn parse_resource(resource: &str) -> (&str, &str) {
    let parts: Vec<&str> = resource.splitn(2, ':').collect();
    if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        ("unknown", resource)
    }
}

/// 标准 actor context (per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.3.2).
#[derive(Debug, Clone)]
pub struct ActorContext {
    /// Actor UUID (user 或 agent).
    pub actor_id: Uuid,
    /// Tenant UUID (RLS 必携).
    pub tenant_id: Uuid,
    /// Workspace UUID (RLS 必携).
    pub workspace_id: Uuid,
    /// 跨层 trace UUID.
    pub trace_id: Uuid,
}

impl ActorContext {
    /// 创建新 actor context.
    pub fn new(actor_id: Uuid, tenant_id: Uuid, workspace_id: Uuid, trace_id: Uuid) -> Self {
        Self {
            actor_id,
            tenant_id,
            workspace_id,
            trace_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resource() {
        let (ns, id) = parse_resource("work_item:UUID-123");
        assert_eq!(ns, "work_item");
        assert_eq!(id, "UUID-123");

        let (ns, id) = parse_resource("task:UUID-456");
        assert_eq!(ns, "task");
        assert_eq!(id, "UUID-456");

        // 无冒号 fallback
        let (ns, id) = parse_resource("plain");
        assert_eq!(ns, "unknown");
        assert_eq!(id, "plain");
    }

    #[test]
    fn test_compute_lock_class_id_stable() {
        // 同一 resource 同一锁类 ID
        let id1 = compute_lock_class_id("work_item:UUID-A");
        let id2 = compute_lock_class_id("work_item:UUID-A");
        let id3 = compute_lock_class_id("work_item:UUID-B");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_actor_context_creation() {
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let workspace = Uuid::new_v4();
        let trace = Uuid::new_v4();
        let ctx = ActorContext::new(actor, tenant, workspace, trace);
        assert_eq!(ctx.actor_id, actor);
        assert_eq!(ctx.tenant_id, tenant);
        assert_eq!(ctx.workspace_id, workspace);
        assert_eq!(ctx.trace_id, trace);
    }
}

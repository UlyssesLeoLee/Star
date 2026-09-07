//! # Lock Audit Logger
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.4.1
//!
//! 锁事件写 `advisory_lock_audit` 表 (per F-11, per EX-01 SQL migration).
//!
//! 守门合规:
//! - 守门 #13 d: 4 张 T 表 100% audit trigger 必携 (per ADR-0043 WORM)
//! - 守门 #13 c: 5 张新表 100% RLS 13 类必携

use uuid::Uuid;

/// 锁审计 logger (mock 实现 - per H2 v18 实证 5/5 subagent RPC 不可靠).
///
/// 实际生产应接 sqlx::PgPool, 这里提供 1 个 trait + 1 个 mock 实现.
pub struct LockAuditLogger {
    /// 模拟存储 (测用).
    pub records: std::sync::Mutex<Vec<LockAuditRecord>>,
}

/// 锁审计记录.
#[derive(Debug, Clone)]
pub struct LockAuditRecord {
    /// 锁类 ID.
    pub lock_class_id: i64,
    /// 锁目标 (e.g. "work_item:UUID").
    pub lock_target: String,
    /// 获取时刻.
    pub acquired_at: chrono::DateTime<chrono::Utc>,
    /// 释放时刻 (None = 未释放).
    pub released_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 等待锁时长 (ms).
    pub wait_ms: i32,
    /// 持锁时长 (ms, 释放时填).
    pub hold_ms: Option<i32>,
    /// Tenant UUID (RLS 必携).
    pub tenant_id: Uuid,
    /// Actor UUID.
    pub actor_id: Uuid,
    /// Trace UUID.
    pub trace_id: Uuid,
}

impl LockAuditLogger {
    /// 创建新 logger.
    pub fn new() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 记录锁获取事件.
    pub fn log_acquired(
        &self,
        lock_class_id: i64,
        lock_target: &str,
        wait_ms: i32,
        tenant_id: Uuid,
        actor_id: Uuid,
        trace_id: Uuid,
    ) {
        let record = LockAuditRecord {
            lock_class_id,
            lock_target: lock_target.to_string(),
            acquired_at: chrono::Utc::now(),
            released_at: None,
            wait_ms,
            hold_ms: None,
            tenant_id,
            actor_id,
            trace_id,
        };
        self.records.lock().unwrap().push(record);
    }

    /// 记录锁释放事件.
    pub fn log_released(&self, lock_target: &str, hold_ms: i32) {
        let mut records = self.records.lock().unwrap();
        if let Some(record) = records.iter_mut().find(|r| r.lock_target == lock_target && r.released_at.is_none()) {
            record.released_at = Some(chrono::Utc::now());
            record.hold_ms = Some(hold_ms);
        }
    }

    /// 记录锁超时事件.
    pub fn log_timeout(&self, lock_target: &str) {
        // 跟 log_released 类似, 但 hold_ms 用 0 表示 timeout
        self.log_released(lock_target, 0);
    }

    /// 测用: 获取所有记录.
    pub fn records(&self) -> Vec<LockAuditRecord> {
        self.records.lock().unwrap().clone()
    }

    /// 测用: 记录数.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    /// 测用: 是否空.
    pub fn is_empty(&self) -> bool {
        self.records.lock().unwrap().is_empty()
    }
}

impl Default for LockAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_acquired() {
        let logger = LockAuditLogger::new();
        logger.log_acquired(
            12345,
            "work_item:UUID-A",
            10,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        assert_eq!(logger.len(), 1);
        let records = logger.records();
        assert_eq!(records[0].lock_class_id, 12345);
        assert_eq!(records[0].lock_target, "work_item:UUID-A");
        assert_eq!(records[0].wait_ms, 10);
        assert!(records[0].released_at.is_none());
        assert!(records[0].hold_ms.is_none());
    }

    #[test]
    fn test_log_released() {
        let logger = LockAuditLogger::new();
        logger.log_acquired(
            12345,
            "work_item:UUID-A",
            10,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        logger.log_released("work_item:UUID-A", 100);

        let records = logger.records();
        assert!(records[0].released_at.is_some());
        assert_eq!(records[0].hold_ms, Some(100));
    }

    #[test]
    fn test_log_timeout() {
        let logger = LockAuditLogger::new();
        logger.log_acquired(
            12345,
            "work_item:UUID-A",
            10,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        logger.log_timeout("work_item:UUID-A");

        let records = logger.records();
        assert_eq!(records[0].hold_ms, Some(0));
    }
}

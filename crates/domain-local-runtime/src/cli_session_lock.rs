//! Star Local Runtime — Session PID Lock 记录 (ULYS-156 P0-A 续)
//!
//! 实现 [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8)
//! §A 单进程持久化层适配 路线下的 "PID + 启动时间 lock" 模块 (FR-ORCA-001 AC-4 +
//! FR-ORCA-004):
//!
//! - **目的**: 单进程模型下,parent runtime 升级/重启时通过 PID + start_time epoch
//!   联合验证 child 进程存活 — 防止 PID 回收(waitpid 之后同一 PID 立即被新进程继承)
//!   导致「子进程残留→以为 attach 成功」事故。
//! - **持久化**: SQLite WAL(复用 `cli_session_registry` 同款 `Mutex<Connection>` +
//!   inline `init_schema` 模式),表名 `cli_session_lock`。
//! - **校验**: 跨进程重启后,读 `pid + start_time_epoch_ms + command`,
//!   用 `kill(pid, 0)` (Linux/macOS) / `OpenProcess` (Windows, 本地 FS 端) 二次校验。
//!
//! **不在本模块**:
//! - 真实 OS `kill(pid, 0)` 实现 — Linux 上 `nix::sys::signal::kill` 等 crate 不在
//!   workspace dependency tree, 本期只到「lock 记录落库 + 校验 epoch 一致」层面,
//!   进程级 pid_liveness 校验留 P1 (`process_supervisor::live_pids` 后续切片)。
//! - star-eventbus 通道 — bus 是后续切片, 本模块只保本地持久化原子。

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path as FsPath;
use std::sync::Mutex;
use thiserror::Error;

use super::cli_session::CliSession;
use super::{CliSessionId, SessionLockId, TenantId};

// =====================================================================
// 1. entity
// =====================================================================

/// **SessionLock** — CliSession 关联的 PID + start_time lock 记录
///
/// 关键不变量:
/// - **INV-LOCK-01**: `(cli_session_id, pid)` 唯一 — 同一 session 不能为不同 PID 同时持锁
/// - **INV-LOCK-02**: `start_time_epoch_ms` 与 `pid` 必须同时被设置 + 校验:
///   PID 回收后,新进程 `start_time` 与旧 lock 不匹配 → 锁失效 (FR-ORCA-004 核心)
/// - **INV-LOCK-03**: `released_at` 一旦设置 → 该 lock 失效,只能用作审计
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionLock {
    /// 唯一 ID
    pub id: SessionLockId,
    /// 关联 CliSession
    pub cli_session_id: CliSessionId,
    /// 跨租户隔离(per INV-RT-01)
    pub tenant_id: TenantId,
    /// OS PID (CLI 模式必有)
    pub pid: u32,
    /// 进程启动时间 epoch millis(用于防 PID 回收骗锁)
    pub start_time_epoch_ms: i64,
    /// 进程命令行(辅助人工排错)
    pub command: String,
    /// 锁获得时间(我们的 runtime 视角)
    pub acquired_at: DateTime<Utc>,
    /// 释放时间(若设置 = 失效)
    pub released_at: Option<DateTime<Utc>>,
    /// 释放原因(供审计)
    pub release_reason: Option<String>,
}

impl SessionLock {
    /// 为一个 CliSession + (pid, start_time_epoch_ms) 创建新 lock
    pub fn new(
        cli_session_id: CliSessionId,
        tenant_id: TenantId,
        pid: u32,
        start_time_epoch_ms: i64,
        command: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: SessionLockId::new(),
            cli_session_id,
            tenant_id,
            pid,
            start_time_epoch_ms,
            command,
            acquired_at: now,
            released_at: None,
            release_reason: None,
        }
    }

    /// 是否仍生效(released_at 为 None)
    pub fn is_active(&self) -> bool {
        self.released_at.is_none()
    }

    /// 标记释放(per INV-LOCK-03)
    pub fn release(&mut self, reason: impl Into<String>) {
        let now = Utc::now();
        self.released_at = Some(now);
        self.release_reason = Some(reason.into());
    }
}

// =====================================================================
// 2. repository (SQLite WAL, Mutex<Connection>)
// =====================================================================

/// SessionLock 持久化错误
#[derive(Debug, Error)]
pub enum SessionLockError {
    /// SQLite 底层错误
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// SessionLock 未找到
    #[error("session lock not found: {0}")]
    NotFound(String),
    /// 同一 (session, pid) 已有 active lock — 拒绝重复 acquire (INV-LOCK-01)
    #[error("lock conflict: cli_session {0} already has active lock for pid {1}")]
    LockConflict(String, u32),
    /// UUID 字段解析失败
    #[error("invalid uuid at {field}: {msg}")]
    InvalidUuid {
        /// 字段名(id / tenant_id / worktree_id)
        field: &'static str,
        /// UUID parse 原始错误消息
        msg: String,
    },
}

/// SessionLock SQLite WAL 仓储
///
/// **INV-LOCK-REG-01**: `Mutex<Connection>` 串行化所有 SQL(守门 #DB-13 W/T/M 派生);
/// `journal_mode=WAL` + `synchronous=NORMAL` 与 `cli_session_registry` 同款。
pub struct SessionLockRegistry {
    conn: Mutex<Connection>,
}

impl SessionLockRegistry {
    /// 打开文件模式仓储(<data-root>/cli_session_lock.db)
    pub fn open(path: impl AsRef<FsPath>) -> Result<Self, SessionLockError> {
        let conn = Connection::open(path)?;
        let reg = Self {
            conn: Mutex::new(conn),
        };
        reg.init_schema()?;
        Ok(reg)
    }

    /// 内存模式(:memory:`/`:memory:),用于测试
    pub fn in_memory() -> Result<Self, SessionLockError> {
        let conn = Connection::open_in_memory()?;
        let reg = Self {
            conn: Mutex::new(conn),
        };
        reg.init_schema()?;
        Ok(reg)
    }

    /// inline DDL (复用 cli_session_registry / star-taskqueue / star-credential 同款)
    fn init_schema(&self) -> Result<(), SessionLockError> {
        let conn = self.conn.lock().expect("lock");
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            CREATE TABLE IF NOT EXISTS cli_session_lock (
                id                    TEXT PRIMARY KEY,
                cli_session_id        TEXT NOT NULL,
                tenant_id             TEXT NOT NULL,
                pid                   INTEGER NOT NULL,
                start_time_epoch_ms   INTEGER NOT NULL,
                command               TEXT NOT NULL,
                acquired_at_ms        INTEGER NOT NULL,
                released_at_ms        INTEGER,
                release_reason        TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_lock_session ON cli_session_lock(cli_session_id);
            CREATE INDEX IF NOT EXISTS idx_lock_pid ON cli_session_lock(pid);
            "#,
        )?;
        Ok(())
    }

    /// 获取一个新 lock(INV-LOCK-01:同 session+pid 重复 → Conflict)
    pub fn acquire(&self, lock: SessionLock) -> Result<SessionLock, SessionLockError> {
        let conn = self.conn.lock().expect("lock");
        // 先查重
        let existing: Option<(String,)> = conn
            .query_row(
                "SELECT id FROM cli_session_lock
                 WHERE cli_session_id = ?1 AND pid = ?2 AND released_at_ms IS NULL
                 LIMIT 1",
                params![lock.cli_session_id.to_string(), lock.pid],
                |row| Ok((row.get::<_, String>(0)?,)),
            )
            .optional()?;
        if existing.is_some() {
            return Err(SessionLockError::LockConflict(
                lock.cli_session_id.to_string(),
                lock.pid,
            ));
        }
        conn.execute(
            "INSERT INTO cli_session_lock
             (id, cli_session_id, tenant_id, pid, start_time_epoch_ms, command,
              acquired_at_ms, released_at_ms, release_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL)",
            params![
                lock.id.to_string(),
                lock.cli_session_id.to_string(),
                lock.tenant_id.to_string(),
                lock.pid as i64,
                lock.start_time_epoch_ms,
                lock.command,
                lock.acquired_at.timestamp_millis(),
            ],
        )?;
        Ok(lock)
    }

    /// 释放 lock(reason 由调用方注入)
    pub fn release(
        &self,
        lock_id: SessionLockId,
        reason: impl Into<String>,
    ) -> Result<SessionLock, SessionLockError> {
        let conn = self.conn.lock().expect("lock");
        let reason = reason.into();
        let now_ms = Utc::now().timestamp_millis();
        let updated = conn.execute(
            "UPDATE cli_session_lock
             SET released_at_ms = ?2, release_reason = ?3
             WHERE id = ?1 AND released_at_ms IS NULL",
            params![lock_id.to_string(), now_ms, &reason],
        )?;
        if updated == 0 {
            return Err(SessionLockError::NotFound(lock_id.to_string()));
        }
        // 重新读出
        drop(conn);
        self.get(lock_id)
    }

    /// 按 ID 读
    pub fn get(&self, lock_id: SessionLockId) -> Result<SessionLock, SessionLockError> {
        let conn = self.conn.lock().expect("lock");
        let row: (
            String,
            String,
            String,
            i64,
            i64,
            String,
            i64,
            Option<i64>,
            Option<String>,
        ) = conn.query_row(
            "SELECT id, cli_session_id, tenant_id, pid, start_time_epoch_ms, command,
                        acquired_at_ms, released_at_ms, release_reason
                 FROM cli_session_lock WHERE id = ?1",
            params![lock_id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )?;
        let id = parse_uuid(&row.0, "id")?;
        let cli_session_id = parse_uuid(&row.1, "cli_session_id")?;
        let tenant_id = parse_uuid(&row.2, "tenant_id")?;
        let acquired_at = Utc
            .timestamp_millis_opt(row.6)
            .single()
            .unwrap_or_else(Utc::now);
        let released_at = row.7.and_then(|ms| Utc.timestamp_millis_opt(ms).single());
        Ok(SessionLock {
            id: SessionLockId(id),
            cli_session_id: CliSessionId(cli_session_id),
            tenant_id: TenantId(tenant_id),
            pid: row.3 as u32,
            start_time_epoch_ms: row.4,
            command: row.5,
            acquired_at,
            released_at,
            release_reason: row.8,
        })
    }

    /// 列出一个 CliSession 的所有 lock(active 或 released)
    pub fn list_for_session(
        &self,
        cli_session_id: CliSessionId,
    ) -> Result<Vec<SessionLock>, SessionLockError> {
        let conn = self.conn.lock().expect("lock");
        let mut stmt = conn.prepare(
            "SELECT id, cli_session_id, tenant_id, pid, start_time_epoch_ms, command,
                    acquired_at_ms, released_at_ms, release_reason
             FROM cli_session_lock WHERE cli_session_id = ?1
             ORDER BY acquired_at_ms ASC",
        )?;
        let rows = stmt.query_map(params![cli_session_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let row = row?;
            let id = parse_uuid(&row.0, "id")?;
            let cli_session_id = parse_uuid(&row.1, "cli_session_id")?;
            let tenant_id = parse_uuid(&row.2, "tenant_id")?;
            let acquired_at = Utc
                .timestamp_millis_opt(row.6)
                .single()
                .unwrap_or_else(Utc::now);
            let released_at = row.7.and_then(|ms| Utc.timestamp_millis_opt(ms).single());
            out.push(SessionLock {
                id: SessionLockId(id),
                cli_session_id: CliSessionId(cli_session_id),
                tenant_id: TenantId(tenant_id),
                pid: row.3 as u32,
                start_time_epoch_ms: row.4,
                command: row.5,
                acquired_at,
                released_at,
                release_reason: row.8,
            });
        }
        Ok(out)
    }

    /// 列出一个 CliSession 的所有 active lock(released_at IS NULL)
    pub fn list_active_for_session(
        &self,
        cli_session_id: CliSessionId,
    ) -> Result<Vec<SessionLock>, SessionLockError> {
        Ok(self
            .list_for_session(cli_session_id)?
            .into_iter()
            .filter(|l| l.is_active())
            .collect())
    }

    /// FR-ORCA-004 核心:校验 (pid, start_time_epoch_ms) 与 lock 记录一致。
    /// 返回 Ok(true) = 一致,Ok(false) = 不一致(可能 PID 已被回收复用),Err = 未找到。
    ///
    /// 调用方应在 attach 残留子进程前调用 — 不一致时拒绝。
    pub fn verify_pid_start_time(
        &self,
        cli_session_id: CliSessionId,
        pid: u32,
        observed_start_time_epoch_ms: i64,
    ) -> Result<bool, SessionLockError> {
        let active = self.list_active_for_session(cli_session_id)?;
        let lock = active
            .iter()
            .find(|l| l.pid == pid)
            .ok_or_else(|| SessionLockError::NotFound(format!("{cli_session_id}:{pid}")))?;
        Ok(lock.start_time_epoch_ms == observed_start_time_epoch_ms)
    }
}

fn parse_uuid(s: &str, field: &'static str) -> Result<uuid::Uuid, SessionLockError> {
    uuid::Uuid::parse_str(s).map_err(|e| SessionLockError::InvalidUuid {
        field,
        msg: e.to_string(),
    })
}

// =====================================================================
// 3. helper: 从 CliSession 推断 start_time_epoch_ms (FR-ORCA-004 配合 process_supervisor)
// =====================================================================

/// 给定一个 CliSession 的 created_at,转换为 epoch millis 用于锁记录。
/// 与 `process_supervisor::record_session_started` 共同约定:lock 的
/// `start_time_epoch_ms` 是 process(或 cmd),而非 CliSession 抽象的 created_at。
/// 这里保留 helper,让调用方明确:CliSession created_at 与 lock start_time 语义不同。
pub fn session_created_at_ms(session: &CliSession) -> i64 {
    session.created_at.timestamp_millis()
}

// =====================================================================
// 4. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 1. acquire/release 闭环 + 同 (session, pid) 重复 Conflict
    #[test]
    fn acquire_release_roundtrip() {
        let reg = SessionLockRegistry::in_memory().unwrap();
        let session = CliSessionId::new();
        let tenant = TenantId::new();
        let lock1 = SessionLock::new(session, tenant, 100, 1000, "claude".to_string());
        let lock_id = lock1.id;
        reg.acquire(lock1).expect("first acquire OK");
        // 同 (session, pid) 重复 → Conflict
        let lock2 = SessionLock::new(session, tenant, 100, 1000, "claude".to_string());
        let r = reg.acquire(lock2);
        assert!(matches!(r, Err(SessionLockError::LockConflict(_, 100))));
        // release OK
        reg.release(lock_id, "manual").expect("release OK");
        let after = reg.get(lock_id).unwrap();
        assert!(!after.is_active());
        assert_eq!(after.release_reason.as_deref(), Some("manual"));
    }

    // ---- 2. verify_pid_start_time:一致 → true;不一致 → false
    #[test]
    fn verify_pid_start_time_distinguishes_recycled_pid() {
        let reg = SessionLockRegistry::in_memory().unwrap();
        let session = CliSessionId::new();
        let tenant = TenantId::new();
        let lock = SessionLock::new(
            session,
            tenant,
            200,
            1_700_000_000_000,
            "claude".to_string(),
        );
        reg.acquire(lock).unwrap();
        assert!(reg
            .verify_pid_start_time(session, 200, 1_700_000_000_000)
            .unwrap());
        // PID 回收 → 假装新 start_time → 不一致
        assert!(!reg
            .verify_pid_start_time(session, 200, 1_700_000_000_001)
            .unwrap());
        // 未知 PID → NotFound
        let r = reg.verify_pid_start_time(session, 999, 1_700_000_000_000);
        assert!(matches!(r, Err(SessionLockError::NotFound(_))));
    }

    // ---- 3. release 不存在 ID → NotFound
    #[test]
    fn release_unknown_id_returns_not_found() {
        let reg = SessionLockRegistry::in_memory().unwrap();
        let r = reg.release(SessionLockId::new(), "test");
        assert!(matches!(r, Err(SessionLockError::NotFound(_))));
    }

    // ---- 4. list_active_for_session 过滤 released
    #[test]
    fn list_active_filters_released() {
        let reg = SessionLockRegistry::in_memory().unwrap();
        let session = CliSessionId::new();
        let tenant = TenantId::new();
        let lock1 = SessionLock::new(session, tenant, 1, 100, "a".to_string());
        let lock1_id = lock1.id;
        let lock2 = SessionLock::new(session, tenant, 2, 200, "b".to_string());
        reg.acquire(lock1).unwrap();
        reg.acquire(lock2).unwrap();
        reg.release(lock1_id, "manual").unwrap();
        let active = reg.list_active_for_session(session).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].pid, 2);
    }

    // ---- 5. 不同 session 同 pid → 不冲突(per-key 唯一)
    #[test]
    fn different_sessions_same_pid_no_conflict() {
        let reg = SessionLockRegistry::in_memory().unwrap();
        let s1 = CliSessionId::new();
        let s2 = CliSessionId::new();
        let tenant = TenantId::new();
        let l1 = SessionLock::new(s1, tenant, 42, 1000, "a".to_string());
        let l2 = SessionLock::new(s2, tenant, 42, 1000, "a".to_string());
        reg.acquire(l1).expect("s1 acquire OK");
        reg.acquire(l2)
            .expect("s2 acquire OK (不同 session 同 pid 不冲突)");
    }

    // ---- 6. file mode 持久化:open 后 re-open 应能读出
    #[test]
    fn file_persistence_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("lock.db");
        let lock_id = {
            let reg = SessionLockRegistry::open(&path).unwrap();
            let session = CliSessionId::new();
            let tenant = TenantId::new();
            let lock = SessionLock::new(session, tenant, 7, 7000, "claude".to_string());
            let lid = lock.id;
            reg.acquire(lock).unwrap();
            lid
        };
        let reg2 = SessionLockRegistry::open(&path).unwrap();
        let after = reg2.get(lock_id).unwrap();
        assert!(after.is_active());
        assert_eq!(after.pid, 7);
        assert_eq!(after.start_time_epoch_ms, 7000);
    }
}

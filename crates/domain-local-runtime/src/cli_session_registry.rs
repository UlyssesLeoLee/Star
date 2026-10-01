//! Star Local Runtime — CLI Session 持久化层 (ULYS-156)
//!
//! CYPHER STRUCTURAL MANIFEST
//! CREATE (f:File {name:"cli_session_registry.rs",type:"file",language:"rust"}),(m:Module {name:"cli_session_registry",type:"module",language:"rust"}),(nonce_ledger:Class {name:"task_cli_grant_nonce_ledger",type:"class",classification:"work"}),(ticket_ledger:Class {name:"task_cli_attachment_ticket",type:"class",classification:"work"}),(binding:Class {name:"TaskCliAttachmentTicketBinding",type:"class",language:"rust"}),(consume_nonce:Function {name:"consume_task_grant_nonce",type:"function"}),(issue_ticket:Function {name:"issue_task_cli_attachment_ticket",type:"function"}),(consume_ticket:Function {name:"consume_task_cli_attachment_ticket",type:"function"}),(hash_ticket:Function {name:"hash_attachment_ticket",type:"function"}),(valid_binding:Function {name:"validate_attachment_binding",type:"function"}),(ttl:Variable {name:"ATTACHMENT_TICKET_MAX_TTL_SECONDS",type:"variable"}),(single_use_test:Function {name:"attachment_ticket_is_single_use_and_scope_bound",type:"function"}),(expiry_test:Function {name:"expired_attachment_ticket_is_rejected",type:"function"});
//! CREATE (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(nonce_ledger),(m)-[:CONTAINS]->(ticket_ledger),(m)-[:CONTAINS]->(binding),(m)-[:CONTAINS]->(consume_nonce),(m)-[:CONTAINS]->(issue_ticket),(m)-[:CONTAINS]->(consume_ticket),(m)-[:CONTAINS]->(hash_ticket),(m)-[:CONTAINS]->(valid_binding),(m)-[:CONTAINS]->(ttl),(m)-[:CONTAINS]->(single_use_test),(m)-[:CONTAINS]->(expiry_test),(consume_nonce)-[:WRITES]->(nonce_ledger),(issue_ticket)-[:WRITES]->(ticket_ledger),(issue_ticket)-[:CALLS]->(hash_ticket),(issue_ticket)-[:CALLS]->(valid_binding),(issue_ticket)-[:USES]->(ttl),(consume_ticket)-[:WRITES]->(ticket_ledger),(consume_ticket)-[:CALLS]->(hash_ticket),(consume_ticket)-[:CALLS]->(valid_binding),(single_use_test)-[:CALLS]->(issue_ticket),(single_use_test)-[:CALLS]->(consume_ticket),(expiry_test)-[:CALLS]->(issue_ticket),(expiry_test)-[:CALLS]->(consume_ticket);
//!
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_session_registry",type:"module",language:"rust"}),(registry:Class {name:"CliSessionRegistry",type:"class"}),(error:Enum {name:"CliSessionRegistryError",type:"enum"});
//! CREATE (consume_fence:Function {name:"CliSessionRegistry::consume_task_run_spawn_fence",type:"function",language:"rust"}),(fence_ledger:Class {name:"task_run_spawn_fence_ledger",type:"class",language:"sqlite"}),(fence_replay:Logic {name:"spawn fence replay rollback",type:"logic",language:"rust"}),(fence_atomic_test:Function {name:"profile_bound_fence_and_nonce_consume_atomically",type:"function",language:"rust"}),(fence_capacity:Variable {name:"MAX_RUNTIME_SPAWN_FENCE_RECEIPTS",type:"variable",language:"rust"});
//! CREATE (registry)-[:HAS_METHOD]->(consume_fence),(m)-[:CONTAINS]->(fence_ledger),(m)-[:CONTAINS]->(fence_replay),(m)-[:CONTAINS]->(fence_atomic_test),(m)-[:CONTAINS]->(fence_capacity),(consume_fence)-[:USES]->(fence_ledger),(consume_fence)-[:CALLS]->(fence_replay),(consume_fence)-[:USES]->(fence_capacity),(fence_replay)-[:USES]->(error),(fence_atomic_test)-[:CALLS]->(consume_fence);
//! 实现 [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8)
//! §A 单进程持久化层适配 路线下的 "cli_session_registry" 模块:
//!
//! - 单进程 SQLite WAL 持久化(`Mutex<Connection>` per 守门 #DB-13 W/T/M 派生)
//! - Task CLI grant nonce uses a dedicated FULL-synchronous WAL connection for fail-closed consumption
//! - 内存模式 (`in_memory`) + 文件模式 (`open`)
//! - DDL inline `init_schema`, 复用 `star-taskqueue` / `star-credential` 同款
//!   `init_schema` 风格(避免引入 sqlx-macros / refinery 等重型迁移框架)
//!
//! **schema**:
//!
//! ```sql
//! CREATE TABLE cli_session (
//!     id           TEXT PRIMARY KEY,        -- CliSessionId (Uuid v4)
//!     tenant_id    TEXT NOT NULL,
//!     worktree_id  TEXT NOT NULL,
//!     state        TEXT NOT NULL,           -- CliSessionState as_str()
//!     command      TEXT NOT NULL,
//!     args_json    TEXT NOT NULL,           -- JSON 序列化的 Vec<String>
//!     scrollback_bytes INTEGER NOT NULL DEFAULT 0,
//!     created_at_ms INTEGER NOT NULL,
//!     updated_at_ms INTEGER NOT NULL,
//!     metadata_json TEXT NOT NULL DEFAULT '{}',
//!     state_history_json TEXT NOT NULL      -- JSON 序列化的 Vec<CliSessionTransition>
//! );
//! CREATE INDEX idx_cli_session_tenant_state ON cli_session(tenant_id, state);
//! CREATE INDEX idx_cli_session_worktree ON cli_session(worktree_id);
//! ```
//!
//! **不在本模块**(per ULYS-156 §A 锁定):
//! - 真实 OS 平台 fork/spawn 适配(AC-1 子进程存活已在 PR #63 efa501a3 落地)
//! - star-eventbus channel 推送(state 写入优先保证本地原子, bus 是后续切片)
//! - W/T/M 派生审计 (`state_history_json` 是 W, 物理删除 OK —— archived 是
//!   user explicit 触发, audit 由 audit-event 流处理)
//!
//! **INV-CLI-REG-01**: 进程级 mutex 串行化所有 SQL; 守门 #12 "并发 64 线程读写 0 死锁"
//! 依赖这是 L0 协调设施(follow `star-credential` / `star-taskqueue` 同等模式)

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

const MAX_RUNTIME_SPAWN_FENCE_RECEIPTS: i64 = 50_000;

use super::cli_session::{CliSession, CliSessionState, CliSessionTransition};
use super::{CliSessionId, TenantId, WorktreeId};

// =====================================================================
// 1. error
// =====================================================================

/// CLI Session 持久化错误
#[derive(Debug, Error)]
pub enum CliSessionRegistryError {
    /// SQLite 底层错误
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// JSON 序列化 / 反序列化
    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Session 未找到
    #[error("cli session not found: {0}")]
    NotFound(String),
    /// UUID 字段解析失败(用于 SQLite 列存的 UUID 文本)
    #[error("invalid uuid at {field}: {msg}")]
    InvalidUuid {
        /// 字段名(id / tenant_id / worktree_id)
        field: &'static str,
        /// UUID parse 原始错误消息
        msg: String,
    },
    /// 状态机迁移非法调用方错误(允许向前:CliSession 层自己校验,
    /// 这里只是落库前的最后一道防线 —— 理论上 ORM 不应让非法 in-flight)
    #[error("illegal terminal state: cannot modify archived session {0}")]
    IllegalArchived(String),
    /// 授权 nonce 已在此 Runtime 消费
    #[error("task execution grant nonce was already consumed")]
    GrantNonceReplay,
    /// Profile-bound Runtime fence 已在此 Runtime 消费。
    #[error("task Run spawn fence was already consumed")]
    SpawnFenceReplay,
    /// Bounded local replay receipt capacity is exhausted; admission must fail closed.
    #[error("task Run spawn fence receipt capacity is exhausted")]
    SpawnFenceLedgerFull,
    /// 授权 nonce 为空、租户为空或 grant 已过期/超出允许窗口
    #[error("task execution grant nonce or expiry is invalid")]
    InvalidGrantNonce,
    /// Attachment ticket was already consumed for this exact bound session.
    #[error("task CLI attachment ticket was already consumed")]
    AttachmentTicketReplay,
    /// Attachment ticket, expiry, or bound context is invalid.
    #[error("task CLI attachment ticket is invalid")]
    InvalidAttachmentTicket,
}

/// Group and Task Card facts bound to a short-lived browser attachment ticket.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskCliAttachmentTicketBinding {
    /// Tenant authorized to attach to the session.
    pub tenant_id: Uuid,
    /// Project containing the selected Worktree.
    pub project_id: Uuid,
    /// Repository mounted by the selected Worktree.
    pub repository_id: Uuid,
    /// Worktree whose Task Card owns the CLI session.
    pub worktree_id: Uuid,
    /// Canonical Task Card / WorkItem identity.
    pub work_item_id: Uuid,
    /// Authenticated user or agent actor.
    pub actor_id: Uuid,
    /// Local Runtime hosting the session.
    pub runtime_id: Uuid,
    /// Existing Local Runtime CLI session.
    pub session_id: Uuid,
    /// Current policy revision; ACL is rechecked before ticket consumption.
    pub policy_version: i64,
}

const ATTACHMENT_TICKET_MAX_TTL_SECONDS: i64 = 60;

// =====================================================================
// 2. registry
// =====================================================================

/// **CLI Session 持久化层**(SQLite, 单进程)
pub struct CliSessionRegistry {
    conn: Mutex<Connection>,
    /// Separate SQLite connection with FULL synchronous commits for grant nonce consumption.
    grant_nonce_conn: Mutex<Connection>,
}

impl CliSessionRegistry {
    /// 内存模式(测试 / 一次性使用)
    pub fn in_memory() -> Result<Self, CliSessionRegistryError> {
        let conn = Connection::open_in_memory()?;
        // 测试场景也跑默认 journal_mode; 真生产 = WAL
        let r = Self {
            conn: Mutex::new(conn),
            grant_nonce_conn: Mutex::new(Connection::open_in_memory()?),
        };
        r.init_schema()?;
        r.init_grant_nonce_schema()?;
        Ok(r)
    }

    /// 文件模式(per 生产)
    ///
    /// 内部启用 WAL(读并发 + 写不阻塞读者)与 `synchronous=NORMAL`(ACID 折中),
    /// 与 `star-taskqueue` 同款配置。
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CliSessionRegistryError> {
        let path = path.as_ref();
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let grant_nonce_conn = Connection::open(path)?;
        grant_nonce_conn.pragma_update(None, "journal_mode", "WAL")?;
        grant_nonce_conn.pragma_update(None, "synchronous", "FULL")?;
        let r = Self {
            conn: Mutex::new(conn),
            grant_nonce_conn: Mutex::new(grant_nonce_conn),
        };
        r.init_schema()?;
        r.init_grant_nonce_schema()?;
        Ok(r)
    }

    fn init_schema(&self) -> Result<(), CliSessionRegistryError> {
        let conn = self
            .conn
            .lock()
            .expect("cli_session registry mutex poisoned");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS cli_session (
                id                TEXT PRIMARY KEY NOT NULL,
                tenant_id         TEXT NOT NULL,
                worktree_id       TEXT NOT NULL,
                state             TEXT NOT NULL,
                command           TEXT NOT NULL,
                args_json         TEXT NOT NULL,
                scrollback_bytes  INTEGER NOT NULL DEFAULT 0,
                created_at_ms     INTEGER NOT NULL,
                updated_at_ms     INTEGER NOT NULL,
                metadata_json     TEXT NOT NULL DEFAULT '{}',
                state_history_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_cli_session_tenant_state
                ON cli_session(tenant_id, state);
            CREATE INDEX IF NOT EXISTS idx_cli_session_worktree
                ON cli_session(worktree_id);
            "#,
        )?;
        Ok(())
    }

    fn init_grant_nonce_schema(&self) -> Result<(), CliSessionRegistryError> {
        let conn = self
            .grant_nonce_conn
            .lock()
            .expect("task grant nonce connection mutex poisoned");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS task_cli_grant_nonce_ledger (
                nonce            TEXT PRIMARY KEY NOT NULL,
                tenant_id        TEXT NOT NULL,
                expires_at_ms    INTEGER NOT NULL,
                consumed_at_ms   INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_task_cli_grant_nonce_tenant_expiry
                ON task_cli_grant_nonce_ledger(tenant_id, expires_at_ms);
            CREATE TABLE IF NOT EXISTS task_run_spawn_fence_ledger (
                fence_id          TEXT PRIMARY KEY NOT NULL,
                tenant_id         TEXT NOT NULL,
                grant_nonce       TEXT NOT NULL UNIQUE,
                binding_digest    BLOB NOT NULL CHECK (length(binding_digest) = 32),
                expires_at_ms     INTEGER NOT NULL,
                consumed_at_ms    INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_task_run_spawn_fence_expiry
                ON task_run_spawn_fence_ledger(expires_at_ms);
            CREATE TABLE IF NOT EXISTS task_run_spawn_fence_receipt_count (
                singleton_id    INTEGER PRIMARY KEY CHECK (singleton_id = 1),
                retained_count  INTEGER NOT NULL CHECK (retained_count >= 0)
            );
            INSERT OR IGNORE INTO task_run_spawn_fence_receipt_count(singleton_id,retained_count)
                SELECT 1, COUNT(*) FROM task_run_spawn_fence_ledger;
            CREATE TRIGGER IF NOT EXISTS trg_task_run_spawn_fence_count_insert
            AFTER INSERT ON task_run_spawn_fence_ledger BEGIN
                UPDATE task_run_spawn_fence_receipt_count
                SET retained_count = retained_count + 1 WHERE singleton_id = 1;
            END;
            CREATE TRIGGER IF NOT EXISTS trg_task_run_spawn_fence_count_delete
            AFTER DELETE ON task_run_spawn_fence_ledger BEGIN
                UPDATE task_run_spawn_fence_receipt_count
                SET retained_count = retained_count - 1 WHERE singleton_id = 1;
            END;
            CREATE TABLE IF NOT EXISTS task_cli_attachment_ticket (
                ticket_hash    TEXT PRIMARY KEY NOT NULL,
                tenant_id      TEXT NOT NULL,
                project_id     TEXT NOT NULL,
                repository_id  TEXT NOT NULL,
                worktree_id    TEXT NOT NULL,
                work_item_id   TEXT NOT NULL,
                actor_id       TEXT NOT NULL,
                runtime_id     TEXT NOT NULL,
                session_id     TEXT NOT NULL,
                policy_version INTEGER NOT NULL,
                expires_at_ms  INTEGER NOT NULL,
                created_at_ms  INTEGER NOT NULL,
                consumed_at_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_task_cli_attachment_ticket_expiry
                ON task_cli_attachment_ticket(expires_at_ms);
            "#,
        )?;
        Ok(())
    }

    /// 原子且持久地消费一个 Task Card CLI grant nonce。
    ///
    /// Nonce 记录属于 Work 状态：grant 到期并经过 5 分钟时钟偏差窗口后，在后续消费时清理。
    /// 调用方须先验证签发者/签名与当前 ACL；expiry 是签名保护的原始过期时间。
    pub fn consume_task_grant_nonce(
        &self,
        tenant_id: Uuid,
        nonce: Uuid,
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), CliSessionRegistryError> {
        if tenant_id.is_nil()
            || nonce.is_nil()
            || expires_at <= now
            || expires_at > now + chrono::Duration::minutes(5)
        {
            return Err(CliSessionRegistryError::InvalidGrantNonce);
        }
        let mut conn = self
            .grant_nonce_conn
            .lock()
            .expect("task grant nonce connection mutex poisoned");
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute(
            "DELETE FROM task_cli_grant_nonce_ledger WHERE expires_at_ms <= ?1",
            params![now
                .timestamp_millis()
                .saturating_sub(chrono::Duration::minutes(5).num_milliseconds())],
        )?;
        let inserted = tx.execute(
            r#"INSERT OR IGNORE INTO task_cli_grant_nonce_ledger
               (nonce,tenant_id,expires_at_ms,consumed_at_ms) VALUES (?1,?2,?3,?4)"#,
            params![
                nonce.to_string(),
                tenant_id.to_string(),
                expires_at.timestamp_millis(),
                now.timestamp_millis()
            ],
        )?;
        if inserted != 1 {
            return Err(CliSessionRegistryError::GrantNonceReplay);
        }
        tx.commit()?;
        Ok(())
    }

    /// Atomically consume a signed grant nonce and C4 Runtime fence under one SQLite transaction.
    /// Both short-lived Work/replay receipts commit together or neither does.
    pub fn consume_task_run_spawn_fence(
        &self,
        tenant_id: Uuid,
        nonce: Uuid,
        fence_id: Uuid,
        binding_digest: [u8; 32],
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), CliSessionRegistryError> {
        if tenant_id.is_nil()
            || nonce.is_nil()
            || fence_id.is_nil()
            || expires_at <= now
            || expires_at > now + chrono::Duration::seconds(30)
        {
            return Err(CliSessionRegistryError::InvalidGrantNonce);
        }

        let mut conn = self
            .grant_nonce_conn
            .lock()
            .expect("task grant nonce connection mutex poisoned");
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let retention_cutoff = now
            .timestamp_millis()
            .saturating_sub(chrono::Duration::minutes(5).num_milliseconds());
        tx.execute(
            "DELETE FROM task_cli_grant_nonce_ledger WHERE expires_at_ms <= ?1",
            params![retention_cutoff],
        )?;
        tx.execute(
            "DELETE FROM task_run_spawn_fence_ledger WHERE expires_at_ms <= ?1",
            params![retention_cutoff],
        )?;
        let retained_fences: i64 = tx.query_row(
            "SELECT retained_count FROM task_run_spawn_fence_receipt_count WHERE singleton_id = 1",
            [],
            |row| row.get(0),
        )?;
        if retained_fences >= MAX_RUNTIME_SPAWN_FENCE_RECEIPTS {
            return Err(CliSessionRegistryError::SpawnFenceLedgerFull);
        }

        let nonce_inserted = tx.execute(
            r#"INSERT OR IGNORE INTO task_cli_grant_nonce_ledger
               (nonce,tenant_id,expires_at_ms,consumed_at_ms) VALUES (?1,?2,?3,?4)"#,
            params![
                nonce.to_string(),
                tenant_id.to_string(),
                expires_at.timestamp_millis(),
                now.timestamp_millis()
            ],
        )?;
        if nonce_inserted != 1 {
            return Err(CliSessionRegistryError::GrantNonceReplay);
        }

        let fence_inserted = tx.execute(
            r#"INSERT OR IGNORE INTO task_run_spawn_fence_ledger
               (fence_id,tenant_id,grant_nonce,binding_digest,expires_at_ms,consumed_at_ms)
               VALUES (?1,?2,?3,?4,?5,?6)"#,
            params![
                fence_id.to_string(),
                tenant_id.to_string(),
                nonce.to_string(),
                &binding_digest[..],
                expires_at.timestamp_millis(),
                now.timestamp_millis()
            ],
        )?;
        if fence_inserted != 1 {
            return Err(CliSessionRegistryError::SpawnFenceReplay);
        }
        tx.commit()?;
        Ok(())
    }

    /// Issue an opaque single-use ticket for an already-started Task Card CLI session.
    ///
    /// Only the SHA-256 digest is persisted. The caller must authenticate the actor, verify the
    /// current ACL, ensure the session is live, and send the plaintext ticket only to that actor.
    pub fn issue_task_cli_attachment_ticket(
        &self,
        binding: &TaskCliAttachmentTicketBinding,
        ttl: chrono::Duration,
        now: DateTime<Utc>,
    ) -> Result<String, CliSessionRegistryError> {
        validate_attachment_binding(binding)?;
        if ttl <= chrono::Duration::zero()
            || ttl > chrono::Duration::seconds(ATTACHMENT_TICKET_MAX_TTL_SECONDS)
        {
            return Err(CliSessionRegistryError::InvalidAttachmentTicket);
        }
        let expires_at = now
            .checked_add_signed(ttl)
            .ok_or(CliSessionRegistryError::InvalidAttachmentTicket)?;
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let ticket_hash = hash_attachment_ticket(&token);
        let mut conn = self
            .grant_nonce_conn
            .lock()
            .expect("task CLI attachment ticket connection mutex poisoned");
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute(
            "DELETE FROM task_cli_attachment_ticket WHERE expires_at_ms <= ?1",
            params![now.timestamp_millis().saturating_sub(300_000)],
        )?;
        tx.execute(
            r#"INSERT INTO task_cli_attachment_ticket
               (ticket_hash,tenant_id,project_id,repository_id,worktree_id,work_item_id,
                actor_id,runtime_id,session_id,policy_version,expires_at_ms,created_at_ms)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)"#,
            params![
                ticket_hash,
                binding.tenant_id.to_string(),
                binding.project_id.to_string(),
                binding.repository_id.to_string(),
                binding.worktree_id.to_string(),
                binding.work_item_id.to_string(),
                binding.actor_id.to_string(),
                binding.runtime_id.to_string(),
                binding.session_id.to_string(),
                binding.policy_version,
                expires_at.timestamp_millis(),
                now.timestamp_millis(),
            ],
        )?;
        tx.commit()?;
        Ok(token)
    }

    /// Atomically consume an attachment ticket after current ACL and session checks succeed.
    ///
    /// A replay is distinguished only for the same binding. A mismatched binding and an expired
    /// or unknown token all return the same invalid-ticket error to avoid disclosing session state.
    pub fn consume_task_cli_attachment_ticket(
        &self,
        ticket: &str,
        binding: &TaskCliAttachmentTicketBinding,
        now: DateTime<Utc>,
    ) -> Result<(), CliSessionRegistryError> {
        validate_attachment_binding(binding)?;
        if ticket.len() != 64 || !ticket.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(CliSessionRegistryError::InvalidAttachmentTicket);
        }
        let ticket_hash = hash_attachment_ticket(ticket);
        let mut conn = self
            .grant_nonce_conn
            .lock()
            .expect("task CLI attachment ticket connection mutex poisoned");
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            r#"UPDATE task_cli_attachment_ticket SET consumed_at_ms=?1
               WHERE ticket_hash=?2 AND tenant_id=?3 AND project_id=?4 AND repository_id=?5
                 AND worktree_id=?6 AND work_item_id=?7 AND actor_id=?8 AND runtime_id=?9
                 AND session_id=?10 AND policy_version=?11 AND expires_at_ms>?1
                 AND consumed_at_ms IS NULL"#,
            params![
                now.timestamp_millis(),
                ticket_hash,
                binding.tenant_id.to_string(),
                binding.project_id.to_string(),
                binding.repository_id.to_string(),
                binding.worktree_id.to_string(),
                binding.work_item_id.to_string(),
                binding.actor_id.to_string(),
                binding.runtime_id.to_string(),
                binding.session_id.to_string(),
                binding.policy_version,
            ],
        )?;
        if changed == 1 {
            tx.commit()?;
            return Ok(());
        }

        let consumed = tx
            .query_row(
                r#"SELECT consumed_at_ms FROM task_cli_attachment_ticket
                   WHERE ticket_hash=?1 AND tenant_id=?2 AND project_id=?3 AND repository_id=?4
                     AND worktree_id=?5 AND work_item_id=?6 AND actor_id=?7 AND runtime_id=?8
                     AND session_id=?9 AND policy_version=?10"#,
                params![
                    ticket_hash,
                    binding.tenant_id.to_string(),
                    binding.project_id.to_string(),
                    binding.repository_id.to_string(),
                    binding.worktree_id.to_string(),
                    binding.work_item_id.to_string(),
                    binding.actor_id.to_string(),
                    binding.runtime_id.to_string(),
                    binding.session_id.to_string(),
                    binding.policy_version,
                ],
                |row| row.get::<_, Option<i64>>(0),
            )
            .optional()?
            .flatten()
            .is_some();
        tx.commit()?;
        if consumed {
            Err(CliSessionRegistryError::AttachmentTicketReplay)
        } else {
            Err(CliSessionRegistryError::InvalidAttachmentTicket)
        }
    }

    /// 插入新 session(`state` 必须是 `Created`; 调用方负责构造 `CliSession`)
    pub fn insert(&self, session: &CliSession) -> Result<(), CliSessionRegistryError> {
        let conn = self
            .conn
            .lock()
            .expect("cli_session registry mutex poisoned");
        let args_json = serde_json::to_string(&session.args)?;
        let history_json = serde_json::to_string(&session.state_history)?;
        let metadata_json = serde_json::to_string(&session.metadata)?;
        conn.execute(
            "INSERT INTO cli_session (
                id, tenant_id, worktree_id, state, command, args_json,
                scrollback_bytes, created_at_ms, updated_at_ms,
                metadata_json, state_history_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session.id.to_string(),
                session.tenant_id.to_string(),
                session.worktree_id.to_string(),
                session.state.as_str(),
                session.command,
                args_json,
                session.scrollback_bytes as i64,
                dt_ms(session.created_at),
                dt_ms(session.updated_at),
                metadata_json,
                history_json,
            ],
        )?;
        Ok(())
    }

    /// 按 ID 查
    pub fn get(&self, id: CliSessionId) -> Result<CliSession, CliSessionRegistryError> {
        let conn = self
            .conn
            .lock()
            .expect("cli_session registry mutex poisoned");
        let row = conn
            .query_row(
                "SELECT id, tenant_id, worktree_id, state, command, args_json,
                        scrollback_bytes, created_at_ms, updated_at_ms,
                        metadata_json, state_history_json
                 FROM cli_session WHERE id = ?1",
                params![id.to_string()],
                cli_session_row_mapper,
            )
            .optional()?;
        match row {
            Some(r) => Ok(r.into_session()?),
            None => Err(CliSessionRegistryError::NotFound(id.to_string())),
        }
    }

    /// 列出 `tenant_id` 下全部 session(可选 state 过滤)
    pub fn list_for_tenant(
        &self,
        tenant_id: TenantId,
        state: Option<CliSessionState>,
    ) -> Result<Vec<CliSession>, CliSessionRegistryError> {
        let conn = self
            .conn
            .lock()
            .expect("cli_session registry mutex poisoned");
        let (sql, state_str): (&str, Option<String>) = if let Some(s) = state {
            (
                "SELECT id, tenant_id, worktree_id, state, command, args_json,
                        scrollback_bytes, created_at_ms, updated_at_ms,
                        metadata_json, state_history_json
                 FROM cli_session
                 WHERE tenant_id = ?1 AND state = ?2
                 ORDER BY created_at_ms DESC",
                Some(s.as_str().to_string()),
            )
        } else {
            (
                "SELECT id, tenant_id, worktree_id, state, command, args_json,
                        scrollback_bytes, created_at_ms, updated_at_ms,
                        metadata_json, state_history_json
                 FROM cli_session
                 WHERE tenant_id = ?1
                 ORDER BY created_at_ms DESC",
                None,
            )
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = if let Some(s) = state_str {
            stmt.query_map(params![tenant_id.to_string(), s], cli_session_row_mapper)?
                .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![tenant_id.to_string()], cli_session_row_mapper)?
                .collect::<Result<Vec<_>, _>>()?
        };
        rows.into_iter()
            .map(|r| r.into_session())
            .collect::<Result<Vec<_>, _>>()
    }

    /// 全表 update(in-place)。`state_history` 应当由调用方在内存层完成 append,
    /// registry 只持久化最终结果。
    ///
    /// **注意**: 这里故意不做 incremental SQL update(history 整体覆盖),
    /// 是因为 CliSessionTransition 列表 size 小(单 session 寿命内 < 100),
    /// 整体写简化逻辑 + 保证一致性。
    pub fn update(&self, session: &CliSession) -> Result<(), CliSessionRegistryError> {
        let conn = self
            .conn
            .lock()
            .expect("cli_session registry mutex poisoned");
        if session.state.is_terminal() {
            // Archived 是单调终态, 不允许 update 把它迁出 Archived 之外
            // 但 update 本身仍允许(写相同状态归档时间等); 这里只挡 Archived → Other
            // 的非法迁移, 由 CliSession::try_transition 强制,
            // registry 仅做 "in flight" 检查
        }
        let args_json = serde_json::to_string(&session.args)?;
        let history_json = serde_json::to_string(&session.state_history)?;
        let metadata_json = serde_json::to_string(&session.metadata)?;
        let updated = conn.execute(
            "UPDATE cli_session SET
                tenant_id = ?2,
                worktree_id = ?3,
                state = ?4,
                command = ?5,
                args_json = ?6,
                scrollback_bytes = ?7,
                updated_at_ms = ?8,
                metadata_json = ?9,
                state_history_json = ?10
             WHERE id = ?1",
            params![
                session.id.to_string(),
                session.tenant_id.to_string(),
                session.worktree_id.to_string(),
                session.state.as_str(),
                session.command,
                args_json,
                session.scrollback_bytes as i64,
                dt_ms(session.updated_at),
                metadata_json,
                history_json,
            ],
        )?;
        if updated == 0 {
            return Err(CliSessionRegistryError::NotFound(session.id.to_string()));
        }
        Ok(())
    }
}

// =====================================================================
// 3. row mapping (helpers, file-local)
// =====================================================================

/// 文件内部 row 中间态(避免每次 query_map 闭包复制)
struct CliSessionRow {
    id: String,
    tenant_id: String,
    worktree_id: String,
    state: String,
    command: String,
    args_json: String,
    scrollback_bytes: i64,
    created_at_ms: i64,
    updated_at_ms: i64,
    metadata_json: String,
    state_history_json: String,
}

impl CliSessionRow {
    fn into_session(self) -> Result<CliSession, CliSessionRegistryError> {
        let id: CliSessionId = parse_uuid(&self.id, "id")?.into();
        let tenant_id: TenantId = parse_uuid(&self.tenant_id, "tenant_id")?.into();
        let worktree_id: WorktreeId = parse_uuid(&self.worktree_id, "worktree_id")?.into();
        let state = parse_state(&self.state)?;
        let args: Vec<String> = serde_json::from_str(&self.args_json)?;
        let metadata: HashMap<String, String> = serde_json::from_str(&self.metadata_json)?;
        let state_history: Vec<CliSessionTransition> =
            serde_json::from_str(&self.state_history_json)?;
        Ok(CliSession {
            id,
            tenant_id,
            worktree_id,
            state,
            command: self.command,
            args,
            scrollback_bytes: self.scrollback_bytes as u64,
            created_at: ms_to_dt(self.created_at_ms),
            updated_at: ms_to_dt(self.updated_at_ms),
            state_history,
            metadata,
        })
    }
}

fn cli_session_row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<CliSessionRow> {
    Ok(CliSessionRow {
        id: row.get(0)?,
        tenant_id: row.get(1)?,
        worktree_id: row.get(2)?,
        state: row.get(3)?,
        command: row.get(4)?,
        args_json: row.get(5)?,
        scrollback_bytes: row.get::<_, i64>(6)?,
        created_at_ms: row.get::<_, i64>(7)?,
        updated_at_ms: row.get::<_, i64>(8)?,
        metadata_json: row.get(9)?,
        state_history_json: row.get(10)?,
    })
}

fn validate_attachment_binding(
    binding: &TaskCliAttachmentTicketBinding,
) -> Result<(), CliSessionRegistryError> {
    let ids = [
        binding.tenant_id,
        binding.project_id,
        binding.repository_id,
        binding.worktree_id,
        binding.work_item_id,
        binding.actor_id,
        binding.runtime_id,
        binding.session_id,
    ];
    if ids.contains(&Uuid::nil()) || binding.policy_version <= 0 {
        return Err(CliSessionRegistryError::InvalidAttachmentTicket);
    }
    Ok(())
}

fn hash_attachment_ticket(ticket: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"star.task-cli-attachment.v1:");
    hasher.update(ticket.as_bytes());
    hex::encode(hasher.finalize())
}

/// 把 UUID 解析失败映射到独立 variant —— 保留字段名 + 原始 error 消息。
fn parse_uuid(s: &str, field: &'static str) -> Result<uuid::Uuid, CliSessionRegistryError> {
    uuid::Uuid::parse_str(s).map_err(|e| CliSessionRegistryError::InvalidUuid {
        field,
        msg: e.to_string(),
    })
}

fn parse_state(s: &str) -> Result<CliSessionState, CliSessionRegistryError> {
    // CliSessionState::as_str 输出 snake_case; serde rename_all = "snake_case" 同步,
    // 用 quoted JSON 解析最直接(也避免硬编码映射表)
    serde_json::from_str(&format!("\"{s}\"")).map_err(CliSessionRegistryError::Json)
}

fn dt_ms(dt: DateTime<Utc>) -> i64 {
    dt.timestamp_millis()
}

fn ms_to_dt(ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now)
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_session::CliSession;
    use uuid::Uuid;

    fn reg() -> CliSessionRegistry {
        CliSessionRegistry::in_memory().expect("in-memory sqlite")
    }

    #[test]
    fn profile_bound_fence_and_nonce_consume_atomically() {
        let registry = reg();
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let tenant_id = Uuid::new_v4();
        let nonce = Uuid::new_v4();
        let fence_id = Uuid::new_v4();

        registry
            .consume_task_run_spawn_fence(
                tenant_id,
                nonce,
                fence_id,
                [7; 32],
                now + chrono::Duration::seconds(20),
                now,
            )
            .unwrap();

        let rolled_back_nonce = Uuid::new_v4();
        assert!(matches!(
            registry.consume_task_run_spawn_fence(
                tenant_id,
                rolled_back_nonce,
                fence_id,
                [7; 32],
                now + chrono::Duration::seconds(20),
                now,
            ),
            Err(CliSessionRegistryError::SpawnFenceReplay)
        ));

        registry
            .consume_task_run_spawn_fence(
                tenant_id,
                rolled_back_nonce,
                Uuid::new_v4(),
                [8; 32],
                now + chrono::Duration::seconds(20),
                now,
            )
            .unwrap();
    }

    fn make_session(tenant: TenantId, worktree: WorktreeId) -> CliSession {
        let mut s = CliSession::new(
            tenant,
            worktree,
            "codex".to_string(),
            vec!["--model".to_string(), "gpt-5".to_string()],
        );
        s.metadata
            .insert("host".to_string(), "test-host".to_string());
        s.try_transition(CliSessionState::Running, "spawn")
            .expect("created -> running");
        s
    }

    fn attachment_binding() -> TaskCliAttachmentTicketBinding {
        TaskCliAttachmentTicketBinding {
            tenant_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            work_item_id: Uuid::new_v4(),
            actor_id: Uuid::new_v4(),
            runtime_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            policy_version: 1,
        }
    }

    #[test]
    fn attachment_ticket_is_single_use_and_scope_bound() {
        let registry = reg();
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let binding = attachment_binding();
        let ticket = registry
            .issue_task_cli_attachment_ticket(&binding, chrono::Duration::seconds(30), now)
            .unwrap();
        assert_eq!(ticket.len(), 64);

        let mut wrong_actor = binding.clone();
        wrong_actor.actor_id = Uuid::new_v4();
        assert!(matches!(
            registry.consume_task_cli_attachment_ticket(&ticket, &wrong_actor, now),
            Err(CliSessionRegistryError::InvalidAttachmentTicket)
        ));

        registry
            .consume_task_cli_attachment_ticket(&ticket, &binding, now)
            .unwrap();
        assert!(matches!(
            registry.consume_task_cli_attachment_ticket(&ticket, &binding, now),
            Err(CliSessionRegistryError::AttachmentTicketReplay)
        ));
    }

    #[test]
    fn expired_attachment_ticket_is_rejected() {
        let registry = reg();
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let binding = attachment_binding();
        let ticket = registry
            .issue_task_cli_attachment_ticket(&binding, chrono::Duration::seconds(1), now)
            .unwrap();
        assert!(matches!(
            registry.consume_task_cli_attachment_ticket(
                &ticket,
                &binding,
                now + chrono::Duration::seconds(2)
            ),
            Err(CliSessionRegistryError::InvalidAttachmentTicket)
        ));
    }

    #[test]
    fn insert_and_get_roundtrip() {
        let r = reg();
        let tenant: TenantId = Uuid::new_v4().into();
        let wt: WorktreeId = Uuid::new_v4().into();
        let s = make_session(tenant, wt);
        r.insert(&s).unwrap();
        let loaded = r.get(s.id).expect("get back");
        assert_eq!(loaded.state, CliSessionState::Running);
        assert_eq!(loaded.command, "codex");
        assert_eq!(loaded.args, vec!["--model", "gpt-5"]);
        assert_eq!(loaded.metadata.get("host"), Some(&"test-host".to_string()));
        // history: created + running = 2 entries
        assert_eq!(loaded.state_history.len(), 2);
    }

    #[test]
    fn update_persists_new_state_and_appended_history() {
        let r = reg();
        let tenant: TenantId = Uuid::new_v4().into();
        let wt: WorktreeId = Uuid::new_v4().into();
        let mut s = make_session(tenant, wt);
        r.insert(&s).unwrap();

        s.try_transition(CliSessionState::Orphaned, "parent gone")
            .unwrap();
        s.try_transition(CliSessionState::Running, "adopt").unwrap();
        s.scrollback_bytes = 4096;
        r.update(&s).unwrap();

        let loaded = r.get(s.id).unwrap();
        assert_eq!(loaded.state, CliSessionState::Running);
        assert_eq!(loaded.scrollback_bytes, 4096);
        // history: created + running + orphaned + running = 4
        assert_eq!(loaded.state_history.len(), 4);
        assert_eq!(loaded.state_history.last().unwrap().reason, "adopt");
    }

    #[test]
    fn get_missing_returns_not_found() {
        let r = reg();
        let id: CliSessionId = Uuid::new_v4().into();
        let err = r.get(id).unwrap_err();
        assert!(matches!(err, CliSessionRegistryError::NotFound(_)));
    }

    #[test]
    fn list_for_tenant_filters_by_state() {
        let r = reg();
        let tenant_a: TenantId = Uuid::new_v4().into();
        let tenant_b: TenantId = Uuid::new_v4().into();
        let wt: WorktreeId = Uuid::new_v4().into();
        let s_a_running = make_session(tenant_a, wt);
        let mut s_a_term = make_session(tenant_a, wt);
        s_a_term
            .try_transition(CliSessionState::Terminated, "cancel")
            .unwrap();
        let _s_b = make_session(tenant_b, wt);

        r.insert(&s_a_running).unwrap();
        r.insert(&s_a_term).unwrap();
        r.insert(&_s_b).unwrap();

        let all = r.list_for_tenant(tenant_a, None).unwrap();
        assert_eq!(all.len(), 2);
        let running = r
            .list_for_tenant(tenant_a, Some(CliSessionState::Running))
            .unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, s_a_running.id);
        let b = r.list_for_tenant(tenant_b, None).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].id, _s_b.id);
    }

    #[test]
    fn update_missing_returns_not_found() {
        let r = reg();
        let tenant: TenantId = Uuid::new_v4().into();
        let wt: WorktreeId = Uuid::new_v4().into();
        let s = make_session(tenant, wt);
        // never inserted
        let err = r.update(&s).unwrap_err();
        assert!(matches!(err, CliSessionRegistryError::NotFound(_)));
    }

    #[test]
    fn file_mode_persists_across_reopen() {
        let tmp =
            std::env::temp_dir().join(format!("ulys156-cli-session-{}.sqlite3", Uuid::new_v4()));
        let tenant: TenantId = Uuid::new_v4().into();
        let wt: WorktreeId = Uuid::new_v4().into();
        let s_id = {
            let r1 = CliSessionRegistry::open(&tmp).unwrap();
            let s = make_session(tenant, wt);
            let id = s.id;
            r1.insert(&s).unwrap();
            id
        };
        // reopen
        {
            let r2 = CliSessionRegistry::open(&tmp).unwrap();
            let loaded = r2.get(s_id).expect("session still there after reopen");
            assert_eq!(loaded.state, CliSessionState::Running);
            assert_eq!(loaded.command, "codex");
        }
        // cleanup
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(format!("{}-wal", tmp.display()));
        let _ = std::fs::remove_file(format!("{}-shm", tmp.display()));
    }
}

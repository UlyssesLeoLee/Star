/*
@cypher schema=1 source_sha256=cc5f1f59ce6cae9be5fc2867e6ea72faaddd0bfc77fdccca0508a647e7b4da68
MERGE (self:File {path:"crates/sandboxd/src/audit.rs"})
@endcypher
*/
//! `sandboxd` audit 模块 (per FR-6.2, BD-002 §4.4)
//!
//! audit log 11 字段 (per FR-6.2): session_id / client_id / task_id / event_type /
//! resource_used / network_violations / fs_violations / capability_violations /
//! timestamp / latency_ms / hit_violations。
//!
//! sink 抽象:
//! - [`JsonLinesAuditSink`]: 文件 JSON Lines append-only (per NFR-M-3 100% parseable)
//! - [`PgAuditSink`]: PG 4 表 (sandbox_session T + sandbox_audit T, per 守门 #13)
//! - [`CompositeAuditSink`]: 多 sink 全写, 任一失败 → Err (per FR-7.2 fail-closed)
//!
//! fail-closed (per FR-7.2): audit 写失败 → BLOCK session 创建, 由 orchestrator 调用方处理。

use std::io::Write;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::SandboxdError;

/// audit 目标表 (per 守门 #13 W/T/M: 2 T 表)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AuditTable {
    /// sandbox_session (T): session 创建 / 销毁事件
    Session,
    /// sandbox_audit (T): session 内事件流 (4 维命中 + 进程事件)
    Audit,
}

/// audit 事件 (11 字段 per FR-6.2)
#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    /// 目标表
    pub table: AuditTable,
    /// session_id
    pub session_id: Uuid,
    /// client_id
    pub client_id: String,
    /// task_id
    pub task_id: String,
    /// 父 session (嵌套, 无 = None)
    pub parent_session_id: Option<Uuid>,
    /// event_type (Session 表: Created/Running/Destroyed/Failed; Audit 表: 8 类 per DD §4.4)
    pub event_type: String,
    /// policy 快照 (Session 表专用)
    pub policy_snapshot: Option<Value>,
    /// resource 命中记录 (mem/cpu/procs 当前值)
    pub resource_used: Option<Value>,
    /// network 命中记录 (dst_ip / dst_port / domain)
    pub network_violations: Option<Value>,
    /// fs 命中记录 (path / mode)
    pub fs_violations: Option<Value>,
    /// capability 命中记录 (syscall / privilege)
    pub capability_violations: Option<Value>,
    /// 事件时间 (append-only)
    pub timestamp: DateTime<Utc>,
    /// 事件处理延迟 ms
    pub latency_ms: Option<i64>,
    /// 命中违规描述 (per FR-4.2 出参)
    pub hit_violations: Vec<String>,
    /// RLS 13 類必携 (per 守门 #13c)
    pub tenant_id: Uuid,
    /// RLS 13 類必携
    pub workspace_id: Uuid,
    /// RLS 13 類必携
    pub actor_id: Uuid,
}

impl AuditEvent {
    /// 构造 session 生命周期事件 (sandbox_session 表)
    #[allow(clippy::too_many_arguments)] // 11 字段跟 FR-6.2 1:1
    pub fn session_event(
        session_id: Uuid,
        client_id: &str,
        task_id: &str,
        parent_session_id: Option<Uuid>,
        event_type: &str,
        policy_snapshot: Option<Value>,
        tenant_id: Uuid,
        workspace_id: Uuid,
        actor_id: Uuid,
    ) -> Self {
        Self {
            table: AuditTable::Session,
            session_id,
            client_id: client_id.to_string(),
            task_id: task_id.to_string(),
            parent_session_id,
            event_type: event_type.to_string(),
            policy_snapshot,
            resource_used: None,
            network_violations: None,
            fs_violations: None,
            capability_violations: None,
            timestamp: Utc::now(),
            latency_ms: None,
            hit_violations: Vec::new(),
            tenant_id,
            workspace_id,
            actor_id,
        }
    }

    /// 构造 session 内事件 (sandbox_audit 表)
    #[allow(clippy::too_many_arguments, clippy::self_named_constructors)] // 11 字段跟 FR-6.2 1:1
    pub fn audit_event(
        session_id: Uuid,
        client_id: &str,
        task_id: &str,
        event_type: &str,
        latency_ms: Option<i64>,
        hit_violations: Vec<String>,
        tenant_id: Uuid,
        workspace_id: Uuid,
        actor_id: Uuid,
    ) -> Self {
        Self {
            table: AuditTable::Audit,
            session_id,
            client_id: client_id.to_string(),
            task_id: task_id.to_string(),
            parent_session_id: None,
            event_type: event_type.to_string(),
            policy_snapshot: None,
            resource_used: None,
            network_violations: None,
            fs_violations: None,
            capability_violations: None,
            timestamp: Utc::now(),
            latency_ms,
            hit_violations,
            tenant_id,
            workspace_id,
            actor_id,
        }
    }
}

/// audit sink trait (per FR-6.2; 写失败 → Err 触发 fail-closed)
#[async_trait]
pub trait AuditSink: Send + Sync {
    /// 写 1 条 audit 事件 (append-only)
    async fn record(&self, event: &AuditEvent) -> Result<(), SandboxdError>;
}

/// JSON Lines 文件 sink (per NFR-M-3 100% parseable, append-only)
pub struct JsonLinesAuditSink {
    /// 追加写文件 (Mutex 串行化, 保证行完整)
    file: Mutex<std::fs::File>,
    /// 文件路径 (诊断用)
    path: String,
}

impl JsonLinesAuditSink {
    /// 打开 (或创建) JSON Lines audit 文件 (append 模式)
    pub fn new(path: &str) -> Result<Self, SandboxdError> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("open {path}: {e}")))?;
        Ok(Self {
            file: Mutex::new(file),
            path: path.to_string(),
        })
    }
}

#[async_trait]
impl AuditSink for JsonLinesAuditSink {
    async fn record(&self, event: &AuditEvent) -> Result<(), SandboxdError> {
        let line = serde_json::to_string(event)
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("serialize: {e}")))?;
        let mut f = self
            .file
            .lock()
            .map_err(|_| SandboxdError::AuditLogWriteFailed("file mutex poisoned".to_string()))?;
        writeln!(f, "{line}")
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("{}: {e}", self.path)))?;
        f.flush()
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("flush {}: {e}", self.path)))
    }
}

/// PG sink (per FR-6.2, 跟 p3d6_audit 表结构一致; 2 T 表 INSERT, per 守门 #13)
pub struct PgAuditSink {
    /// sqlx 连接池
    pool: sqlx::PgPool,
}

impl PgAuditSink {
    /// 连接 PG (URL 空 → Err, 由调用方决定是否降级 file-only)
    ///
    /// 连接/获取超时 5s fail-fast (默认 30s 会拖慢 failover 探测)。
    pub async fn connect(url: &str) -> Result<Self, SandboxdError> {
        if url.is_empty() {
            return Err(SandboxdError::AuditLogWriteFailed(
                "audit_log_pg_url is empty".to_string(),
            ));
        }
        // sqlx 0.8 无 connect_timeout 选项; acquire_timeout 5s fail-fast (failover 探测用)
        let opts = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(std::time::Duration::from_secs(5));
        let pool = opts
            .connect(url)
            .await
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("connect failed: {e}")))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl AuditSink for PgAuditSink {
    async fn record(&self, event: &AuditEvent) -> Result<(), SandboxdError> {
        // RLS 13 類: set_config 注入 tenant/workspace/actor (transaction-local, per DD §4.6)
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("begin failed: {e}")))?;
        for (key, value) in [
            ("app.tenant_id", event.tenant_id.to_string()),
            ("app.workspace_id", event.workspace_id.to_string()),
            ("app.actor_id", event.actor_id.to_string()),
        ] {
            sqlx::query("SELECT set_config($1, $2, true)")
                .bind(key)
                .bind(value)
                .execute(&mut *tx)
                .await
                .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("set_config: {e}")))?;
        }
        let result = match event.table {
            AuditTable::Session => {
                sqlx::query(
                    "INSERT INTO sandbox_session \
                     (session_id, client_id, task_id, parent_session_id, policy_snapshot, \
                      status, tenant_id, workspace_id, actor_id) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(event.session_id)
                .bind(&event.client_id)
                .bind(&event.task_id)
                .bind(event.parent_session_id)
                .bind(sqlx::types::Json(&event.policy_snapshot))
                .bind(&event.event_type)
                .bind(event.tenant_id)
                .bind(event.workspace_id)
                .bind(event.actor_id)
                .execute(&mut *tx)
                .await
            }
            AuditTable::Audit => {
                sqlx::query(
                    "INSERT INTO sandbox_audit \
                     (session_id, event_type, resource_used, network_violations, fs_violations, \
                      capability_violations, latency_ms, hit_violations, tenant_id, workspace_id, actor_id) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                )
                .bind(event.session_id)
                .bind(&event.event_type)
                .bind(sqlx::types::Json(&event.resource_used))
                .bind(sqlx::types::Json(&event.network_violations))
                .bind(sqlx::types::Json(&event.fs_violations))
                .bind(sqlx::types::Json(&event.capability_violations))
                .bind(event.latency_ms)
                .bind(&event.hit_violations)
                .bind(event.tenant_id)
                .bind(event.workspace_id)
                .bind(event.actor_id)
                .execute(&mut *tx)
                .await
            }
        };
        result.map_err(|e| SandboxdError::AuditLogWriteFailed(format!("insert failed: {e}")))?;
        tx.commit()
            .await
            .map_err(|e| SandboxdError::AuditLogWriteFailed(format!("commit failed: {e}")))?;
        Ok(())
    }
}

/// 组合 sink: 全写, 任一失败 → Err (per FR-7.2 fail-closed)
pub struct CompositeAuditSink {
    /// 子 sink 列表
    sinks: Vec<Box<dyn AuditSink>>,
}

impl CompositeAuditSink {
    /// 构造组合 sink
    pub fn new(sinks: Vec<Box<dyn AuditSink>>) -> Self {
        Self { sinks }
    }
}

#[async_trait]
impl AuditSink for CompositeAuditSink {
    async fn record(&self, event: &AuditEvent) -> Result<(), SandboxdError> {
        for sink in &self.sinks {
            sink.record(event).await?;
        }
        Ok(())
    }
}

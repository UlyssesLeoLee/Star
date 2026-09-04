//! crates/star-credential/src/db.rs
//!
//! V2-3 凭证管理 DB 持久化 + 审计日志 (per 守门 #12 派生 + 守门 #DB-13 W/T/M)
//!
//! 实现: SQLite via rusqlite (in-memory + 文件双模式)
//! - CredentialRecord = Master 类型 (永存, 物理删除禁止, 仅状态字段变更)
//! - CredentialAuditEvent = Append-only (T 类型, 永久保留, 4 event: store/rotate/revoke/retrieve)
//!
//! 不在本 PoC: 真实 PostgreSQL + RLS 13 類 (V2-3 完整版)

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

use crate::{CredentialError, CredentialMetadata, CredentialPlaintext, CredentialRecord, CredentialStatus, Provider};
use domain_kms::EncryptedBlob;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
}

/// 凭证审计事件 (T 类型, Append-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialAuditEvent {
    pub id: String,
    pub credential_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub event_type: AuditEventType,
    pub event_at_ms: u64,
    /// 元数据 (不含密文)
    pub metadata_snapshot: Option<CredentialMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    Store,
    Rotate,
    Revoke,
    Retrieve,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Store => "store",
            Self::Rotate => "rotate",
            Self::Revoke => "revoke",
            Self::Retrieve => "retrieve",
        }
    }
}

/// 持久化层 (SQLite)
pub struct CredentialDb {
    conn: Mutex<Connection>,
}

impl CredentialDb {
    /// 内存模式 (测试用)
    pub fn in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn: Mutex::new(conn) };
        db.init_schema()?;
        Ok(db)
    }

    /// 文件模式 (生产用, per 守门 #DB-13 Master 永存)
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        // Credential 表: Master 类型 (永存, 物理删除禁止)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS credential (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                metadata_json TEXT NOT NULL,
                encrypted_blob_json TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                deprecated_at_ms INTEGER,
                revoked_at_ms INTEGER
            )",
            [],
        )?;
        // 索引: (tenant_id, provider) 加速查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_credential_tenant_provider ON credential(tenant_id, provider)",
            [],
        )?;
        // Audit 事件表: Append-only T 类型 (永久保留)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS credential_audit_event (
                id TEXT PRIMARY KEY,
                credential_id TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_at_ms INTEGER NOT NULL,
                metadata_snapshot_json TEXT
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_credential ON credential_audit_event(credential_id)",
            [],
        )?;
        Ok(())
    }

    /// 插入凭证 (Master, 永存, 物理删除禁止 per 守门 #DB-13)
    pub fn insert_credential(&self, record: &CredentialRecord) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        let metadata_json = serde_json::to_string(&record.metadata)?;
        let blob_json = serde_json::to_string(&record.encrypted_blob)?;
        conn.execute(
            "INSERT INTO credential (id, tenant_id, user_id, provider, metadata_json, encrypted_blob_json, status, created_at_ms, updated_at_ms, deprecated_at_ms, revoked_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                record.id,
                record.tenant_id,
                record.user_id,
                record.provider.as_str(),
                metadata_json,
                blob_json,
                status_to_str(record.status),
                record.created_at_ms as i64,
                record.updated_at_ms as i64,
                record.deprecated_at_ms.map(|m| m as i64),
                record.revoked_at_ms.map(|m| m as i64),
            ],
        )?;
        Ok(())
    }

    /// 更新凭证状态 (仅 status + deprecated/revoked_at_ms 字段, 不改 ciphertext)
    pub fn update_credential_status(
        &self,
        id: &str,
        status: CredentialStatus,
        updated_at_ms: u64,
        deprecated_at_ms: Option<u64>,
        revoked_at_ms: Option<u64>,
    ) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE credential SET status = ?1, updated_at_ms = ?2, deprecated_at_ms = ?3, revoked_at_ms = ?4 WHERE id = ?5",
            params![status_to_str(status), updated_at_ms as i64, deprecated_at_ms.map(|m| m as i64), revoked_at_ms.map(|m| m as i64), id],
        )?;
        Ok(())
    }

    /// 查 tenant 凭证
    pub fn list_credentials(&self, tenant_id: &str, provider: Option<Provider>) -> Result<Vec<CredentialRecord>, DbError> {
        let conn = self.conn.lock().unwrap();
        let provider_str = provider.map(|p| p.as_str().to_string());
        let mut stmt = conn.prepare(
            "SELECT id, tenant_id, user_id, provider, metadata_json, encrypted_blob_json, status, created_at_ms, updated_at_ms, deprecated_at_ms, revoked_at_ms
             FROM credential WHERE tenant_id = ?1 AND (?2 IS NULL OR provider = ?2) ORDER BY created_at_ms DESC"
        )?;
        let records = stmt.query_map(params![tenant_id, provider_str], |row| {
            let provider: String = row.get(3)?;
            let provider = parse_provider(&provider).map_err(|_| rusqlite::Error::InvalidQuery)?;
            let metadata_json: String = row.get(4)?;
            let blob_json: String = row.get(5)?;
            let status: String = row.get(6)?;
            let status = parse_status(&status).map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(CredentialRecord {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                user_id: row.get(2)?,
                provider,
                metadata: serde_json::from_str(&metadata_json).map_err(|_| rusqlite::Error::InvalidQuery)?,
                encrypted_blob: serde_json::from_str(&blob_json).map_err(|_| rusqlite::Error::InvalidQuery)?,
                status,
                created_at_ms: row.get::<_, i64>(7)? as u64,
                updated_at_ms: row.get::<_, i64>(8)? as u64,
                deprecated_at_ms: row.get::<_, Option<i64>>(9)?.map(|m| m as u64),
                revoked_at_ms: row.get::<_, Option<i64>>(10)?.map(|m| m as u64),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    /// 追加审计事件 (Append-only, T 类型)
    pub fn append_audit_event(&self, event: &CredentialAuditEvent) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        let metadata_snapshot_json = event.metadata_snapshot
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        conn.execute(
            "INSERT INTO credential_audit_event (id, credential_id, tenant_id, user_id, event_type, event_at_ms, metadata_snapshot_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id,
                event.credential_id,
                event.tenant_id,
                event.user_id,
                event.event_type.as_str(),
                event.event_at_ms as i64,
                metadata_snapshot_json,
            ],
        )?;
        Ok(())
    }

    /// 查凭证的审计历史
    pub fn list_audit_events(&self, credential_id: &str) -> Result<Vec<CredentialAuditEvent>, DbError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, credential_id, tenant_id, user_id, event_type, event_at_ms, metadata_snapshot_json
             FROM credential_audit_event WHERE credential_id = ?1 ORDER BY event_at_ms ASC"
        )?;
        let events = stmt.query_map(params![credential_id], |row| {
            let event_type: String = row.get(4)?;
            let event_type = parse_event_type(&event_type).map_err(|_| rusqlite::Error::InvalidQuery)?;
            let metadata_snapshot_json: Option<String> = row.get(6)?;
            let metadata_snapshot = metadata_snapshot_json
                .as_ref()
                .map(|s| serde_json::from_str(s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(CredentialAuditEvent {
                id: row.get(0)?,
                credential_id: row.get(1)?,
                tenant_id: row.get(2)?,
                user_id: row.get(3)?,
                event_type,
                event_at_ms: row.get::<_, i64>(5)? as u64,
                metadata_snapshot,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(events)
    }
}

fn status_to_str(s: CredentialStatus) -> &'static str {
    match s {
        CredentialStatus::Active => "active",
        CredentialStatus::Deprecated => "deprecated",
        CredentialStatus::Revoked => "revoked",
    }
}

fn parse_status(s: &str) -> Result<CredentialStatus, String> {
    match s {
        "active" => Ok(CredentialStatus::Active),
        "deprecated" => Ok(CredentialStatus::Deprecated),
        "revoked" => Ok(CredentialStatus::Revoked),
        other => Err(format!("unknown status: {}", other)),
    }
}

fn parse_provider(s: &str) -> Result<Provider, String> {
    match s {
        "openclaw" => Ok(Provider::OpenClaw),
        "hermes" => Ok(Provider::Hermes),
        "kms_vault" => Ok(Provider::KmsVault),
        "kms_aws" => Ok(Provider::KmsAws),
        "kms_local_mock" => Ok(Provider::KmsLocalMock),
        other => Err(format!("unknown provider: {}", other)),
    }
}

fn parse_event_type(s: &str) -> Result<AuditEventType, String> {
    match s {
        "store" => Ok(AuditEventType::Store),
        "rotate" => Ok(AuditEventType::Rotate),
        "revoke" => Ok(AuditEventType::Revoke),
        "retrieve" => Ok(AuditEventType::Retrieve),
        other => Err(format!("unknown event type: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(provider: Provider, status: CredentialStatus) -> CredentialRecord {
        CredentialRecord {
            id: Uuid::new_v4().to_string(),
            tenant_id: "t1".into(),
            user_id: "u1".into(),
            provider,
            metadata: CredentialMetadata { display_name: "Test".into(), description: "".into() },
            encrypted_blob: EncryptedBlob {
                algorithm: "aes-256-gcm".into(),
                dek_id: domain_kms::KeyId("dek-1".into()),
                encrypted_dek: "".into(),
                nonce: "nonce-1".into(),
                ciphertext: "ct-1".into(),
                tenant_id: "t1".into(),
                created_at: chrono::Utc::now(),
                key_version: 1,
            },
            created_at_ms: 1000,
            updated_at_ms: 1000,
            status,
            deprecated_at_ms: None,
            revoked_at_ms: None,
        }
    }

    #[test]
    fn v2_db_insert_and_list() {
        let db = CredentialDb::in_memory().unwrap();
        let r = make_record(Provider::OpenClaw, CredentialStatus::Active);
        db.insert_credential(&r).unwrap();
        let list = db.list_credentials("t1", None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, r.id);
    }

    #[test]
    fn v2_db_update_status() {
        let db = CredentialDb::in_memory().unwrap();
        let r = make_record(Provider::Hermes, CredentialStatus::Active);
        db.insert_credential(&r).unwrap();
        db.update_credential_status(&r.id, CredentialStatus::Revoked, 2000, None, Some(2000)).unwrap();
        let list = db.list_credentials("t1", None).unwrap();
        assert_eq!(list[0].status, CredentialStatus::Revoked);
        assert_eq!(list[0].revoked_at_ms, Some(2000));
    }

    #[test]
    fn v2_db_audit_event_append() {
        let db = CredentialDb::in_memory().unwrap();
        let r = make_record(Provider::KmsVault, CredentialStatus::Active);
        db.insert_credential(&r).unwrap();
        let event = CredentialAuditEvent {
            id: Uuid::new_v4().to_string(),
            credential_id: r.id.clone(),
            tenant_id: "t1".into(),
            user_id: "u1".into(),
            event_type: AuditEventType::Store,
            event_at_ms: 1000,
            metadata_snapshot: Some(CredentialMetadata { display_name: "Test".into(), description: "".into() }),
        };
        db.append_audit_event(&event).unwrap();
        let events = db.list_audit_events(&r.id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::Store);
    }
}

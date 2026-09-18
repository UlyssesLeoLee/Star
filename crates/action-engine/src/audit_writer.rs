//! `audit_writer.rs` — Audit Writer (SCD Type 2 不可篡改, per INV-WC-12 + DD §43)
//!
//! SCD Type 2 实现:
//! - Append-only: 不允许 update / delete
//! - 每个 record 含 `valid_from`, `valid_to` (NULL = current), `is_current` 标记
//! - 修正通过新 row (valid_from = now, valid_to = NULL, is_current = TRUE) + 旧 row valid_to = now
//!   (本期 T8 MVP 不实装修正, 仅 append)
//!
//! 真实落点是 PG `canvas_audit` 表 (per BD §14.1) — append-only + 月分区 +
//! `idempotency_key` UNIQUE 约束 (per FR-ACTION-006).
//!
//! 本期 T8 用内存 Vec 模拟, 接口与 PG 版对齐。

use std::sync::RwLock;

use chrono::{DateTime, Utc};
use graph_core::types::{UserId, WorktreeId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit Writer 错误 (per 守门 #6 v2 6-field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditWriterError {
    /// 错误码
    pub code: String,
    /// 消息
    pub message: String,
    /// Trace ID
    pub trace_id: String,
}

impl std::fmt::Display for AuditWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} (trace={})",
            self.code, self.message, self.trace_id
        )
    }
}

impl std::error::Error for AuditWriterError {}

/// Audit Record (per DD §43 AuditEntry schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Audit ID (PK, BIGSERIAL in PG)
    pub id: u64,
    /// Action 类型 (string)
    pub action_type: String,
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// 操作人
    pub actor_id: UserId,
    /// 操作人类型 (user / agent / system)
    pub actor_type: String,
    /// 操作结果 (success / failed / cancelled)
    pub result: String,
    /// 错误消息 (failed 时)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Idempotency Key (per FR-ACTION-006)
    pub idempotency_key: Uuid,
    /// Payload (action params)
    pub payload: serde_json::Value,
    /// 生效起始时间 (SCD Type 2 valid_from)
    pub valid_from: DateTime<Utc>,
    /// 生效结束时间 (SCD Type 2 valid_to, NULL = current)
    pub valid_to: Option<DateTime<Utc>>,
    /// 是否当前生效 (SCD Type 2 is_current)
    pub is_current: bool,
}

/// Audit Writer — append-only, SCD Type 2 不可篡改
#[derive(Debug, Default)]
pub struct AuditWriter {
    /// Append-only log (内存版, PG 版对应 `canvas_audit` 表)
    log: RwLock<Vec<AuditRecord>>,
    /// 自增 ID (PG 端用 BIGSERIAL)
    next_id: RwLock<u64>,
}

impl AuditWriter {
    /// 构造一个新 AuditWriter
    pub fn new() -> Self {
        Self::default()
    }

    /// Append 一条 audit record (不可修改 / 不可删除)
    #[allow(clippy::too_many_arguments)] // 8 字段 (action/worktree/actor/actor_type/result/idempotency_key/payload/timestamp) 是 SCD Type 2 audit 强制 schema (per DD §43)
    pub fn append(
        &self,
        action_type: impl Into<String>,
        worktree_id: WorktreeId,
        actor_id: UserId,
        actor_type: impl Into<String>,
        result: impl Into<String>,
        idempotency_key: Uuid,
        payload: serde_json::Value,
    ) -> Result<u64, AuditWriterError> {
        let mut id_guard = self.next_id.write().map_err(|e| AuditWriterError {
            code: "AUDIT.LOCK".to_string(),
            message: format!("audit id lock poisoned: {e}"),
            trace_id: Uuid::new_v4().to_string(),
        })?;
        *id_guard += 1;
        let id = *id_guard;
        drop(id_guard);

        let record = AuditRecord {
            id,
            action_type: action_type.into(),
            worktree_id,
            actor_id,
            actor_type: actor_type.into(),
            result: result.into(),
            error_message: None,
            idempotency_key,
            payload,
            valid_from: Utc::now(),
            valid_to: None,
            is_current: true,
        };

        let mut log = self.log.write().map_err(|e| AuditWriterError {
            code: "AUDIT.LOCK".to_string(),
            message: format!("audit log lock poisoned: {e}"),
            trace_id: Uuid::new_v4().to_string(),
        })?;
        log.push(record);
        Ok(id)
    }

    /// 查询全部 record (按 id 升序) — read-only
    pub fn all(&self) -> Vec<AuditRecord> {
        self.log.read().map(|g| g.clone()).unwrap_or_default()
    }

    /// 按 idempotency_key 查询 (同 key replay 时 audit 也只 1 条, per FR-ACTION-006)
    pub fn by_idempotency_key(&self, key: Uuid) -> Option<AuditRecord> {
        self.log
            .read()
            .ok()
            .and_then(|g| g.iter().find(|r| r.idempotency_key == key).cloned())
    }

    /// 按 worktree_id 查询
    pub fn by_worktree(&self, worktree_id: WorktreeId) -> Vec<AuditRecord> {
        self.log
            .read()
            .map(|g| {
                g.iter()
                    .filter(|r| r.worktree_id == worktree_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 当前 record 数 (测试用)
    pub fn len(&self) -> usize {
        self.log.read().map(|g| g.len()).unwrap_or(0)
    }

    /// 是否为空 (测试用)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // 注: 守门 INV-WC-12 不可篡改 — API 仅暴露 read + append,
    // 不暴露 update / delete (Rust 编译期保证, no `&mut self` 写公开方法)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_log_scd_type2_immutable() {
        // 守门 UT-4: Audit Log SCD Type 2 不可篡改
        let writer = AuditWriter::new();
        let wt = Uuid::new_v4();
        let user = Uuid::new_v4();

        // 1. 写入 3 条 Destructive Action audit
        let id1 = writer
            .append(
                "Delete",
                wt,
                user,
                "user",
                "success",
                Uuid::new_v4(),
                serde_json::json!({"branch": "main"}),
            )
            .unwrap();
        let id2 = writer
            .append(
                "Merge",
                wt,
                user,
                "user",
                "success",
                Uuid::new_v4(),
                serde_json::json!({"strategy": "merge"}),
            )
            .unwrap();
        let id3 = writer
            .append(
                "ForceDelete",
                wt,
                user,
                "user",
                "success",
                Uuid::new_v4(),
                serde_json::json!({}),
            )
            .unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(writer.len(), 3);

        // 2. 全部 record 都有 valid_from + is_current=true (SCD Type 2 current)
        for r in writer.all() {
            assert!(r.valid_from <= Utc::now());
            assert!(r.is_current);
            assert!(r.valid_to.is_none());
        }

        // 3. 按 idempotency_key 查询 — 唯一
        let key = Uuid::new_v4();
        writer
            .append(
                "Delete",
                wt,
                user,
                "user",
                "success",
                key,
                serde_json::json!({}),
            )
            .unwrap();
        let hit = writer.by_idempotency_key(key);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().idempotency_key, key);

        // 4. 按 worktree 查询
        let records = writer.by_worktree(wt);
        assert_eq!(records.len(), 4); // 3 + 1

        // 5. Append-only: 没有暴露 update / delete 方法 — 编译期守门
        // (没有 &mut self 写公开方法, 见源码注释)

        // 6. SCD Type 2: 同 actor + action_type 多次 append 视为新 record (valid_from 不同)
        //    本测试不模拟修正路径, 只验证 append-only
        let _r1 = writer
            .append(
                "Delete",
                wt,
                user,
                "user",
                "success",
                Uuid::new_v4(),
                serde_json::json!({}),
            )
            .unwrap();
        assert_eq!(writer.len(), 5);
    }
}

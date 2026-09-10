// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpsLogEntry Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/ops_log_entry.sql` 表对应
//! - 守门 #13 W 类: 物理删除 / タイマー失効 / 短 TTL 明示 retention
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::PgAdapterError;

/// OpsLogEntry (W/T/M = W, 跟 DDL ops_log_entry 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OpsLogEntry {
    /// 主键
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// 跨域 trace
    pub trace_id: Option<Uuid>,
    /// FK to ops_log_query_log ON DELETE RESTRICT
    pub log_query_id: Option<Uuid>,
    /// debug/info/warn/error
    pub level: String,
    /// app/system/agent/mcp/external
    pub source: String,
    /// log 内容
    pub message: String,
    /// 截断 256 字符
    pub message_excerpt: Option<String>,
    /// 额外元数据 (JSONB)
    pub metadata: serde_json::Value,
    /// log 真实发生时间
    pub occurred_at: DateTime<Utc>,
    /// 入库时间
    pub captured_at: DateTime<Utc>,
    /// TTL 7d 过期时间
    pub expires_at: DateTime<Utc>,
    /// star_ops::log
    pub source_module: String,
    /// work
    pub source_kind: String,
    /// RLS 13 類
    pub user_id: Option<Uuid>,
    /// RLS 13 類
    pub role_id: Option<Uuid>,
    /// RLS 13 類
    pub permission_id: Option<Uuid>,
    /// RLS 13 類
    pub policy_id: Option<Uuid>,
    /// RLS 13 類
    pub workspace_id: Option<Uuid>,
    /// RLS 13 類
    pub project_id: Option<Uuid>,
    /// RLS 13 類
    pub work_item_id: Option<Uuid>,
    /// RLS 13 類
    pub agent_id: Option<Uuid>,
    /// RLS 13 類
    pub session_id: Option<Uuid>,
}

/// OpsLogEntryRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OpsLogEntryRepository: Send + Sync {
    /// find_by_query (per DDL ops_log_entry W 派生规)
    async fn find_by_query(
        &self,
        tenant_id: Uuid,
        query_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogEntry>, PgAdapterError>;
    /// list_by_trace (per DDL ops_log_entry W 派生规)
    async fn list_by_trace(
        &self,
        tenant_id: Uuid,
        trace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogEntry>, PgAdapterError>;
    /// insert (per DDL ops_log_entry W 派生规)
    async fn insert(&self, entry: &OpsLogEntry) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/ops_log_entry 对应)
pub struct PgOpsLogEntryRepository {
    pool: PgPool,
}

impl PgOpsLogEntryRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// 拿共享 PgPool 引用 (v0.74 P0-4 Stage 2.1 扩展: 5 ops Repository wire-up)
    ///
    /// 用于 application crate 跨域编排时多个 Repository 共享同一 pool
    /// (per sqlx::PgPool = Arc 内部, clone 廉价).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl OpsLogEntryRepository for PgOpsLogEntryRepository {
    async fn find_by_query(
        &self,
        tenant_id: Uuid,
        query_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogEntry>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsLogEntry>(
            r#"
                        SELECT id, tenant_id, trace_id, log_query_id, level, source, message, message_excerpt, metadata, occurred_at, captured_at, expires_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id
            FROM ops_log_entry
            WHERE tenant_id = $1 AND log_query_id = $2
              AND expires_at > NOW()
            ORDER BY occurred_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(query_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_query: {}", e)))?;
        Ok(rows)
    }

    async fn list_by_trace(
        &self,
        tenant_id: Uuid,
        trace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogEntry>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsLogEntry>(
            r#"
                        SELECT id, tenant_id, trace_id, log_query_id, level, source, message, message_excerpt, metadata, occurred_at, captured_at, expires_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id
            FROM ops_log_entry
            WHERE tenant_id = $1 AND trace_id = $2
              AND expires_at > NOW()
            ORDER BY occurred_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(trace_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("list_by_trace: {}", e)))?;
        Ok(rows)
    }

    async fn insert(&self, entry: &OpsLogEntry) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        INSERT INTO ops_log_entry
                (id, tenant_id, trace_id, log_query_id, level, source, message, message_excerpt, metadata, occurred_at, captured_at, expires_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.tenant_id)
        .bind(&entry.trace_id)
        .bind(&entry.log_query_id)
        .bind(&entry.level)
        .bind(&entry.source)
        .bind(&entry.message)
        .bind(&entry.message_excerpt)
        .bind(&entry.metadata)
        .bind(&entry.occurred_at)
        .bind(&entry.captured_at)
        .bind(&entry.expires_at)
        .bind(&entry.source_module)
        .bind(&entry.source_kind)
        .bind(&entry.user_id)
        .bind(&entry.role_id)
        .bind(&entry.permission_id)
        .bind(&entry.policy_id)
        .bind(&entry.workspace_id)
        .bind(&entry.project_id)
        .bind(&entry.work_item_id)
        .bind(&entry.agent_id)
        .bind(&entry.session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OpsLogEntry 必含 23 字段 (跟 DDL 100% 一致)
    #[test]
    fn ops_log_entry_struct_has_23_fields() {
        let x = OpsLogEntry {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            trace_id: None,
            log_query_id: None,
            level: "test".to_string(),
            source: "test".to_string(),
            message: "test".to_string(),
            message_excerpt: None,
            metadata: serde_json::json!({}),
            occurred_at: Utc::now(),
            captured_at: Utc::now(),
            expires_at: Utc::now(),
            source_module: "test".to_string(),
            source_kind: "test".to_string(),
            user_id: None,
            role_id: None,
            permission_id: None,
            policy_id: None,
            workspace_id: None,
            project_id: None,
            work_item_id: None,
            agent_id: None,
            session_id: None,
        };
        assert_eq!(x.source_module, "test");
    }

    /// 派生 #N: OpsLogEntryRepository trait 必含 3 方法
    #[test]
    fn ops_log_entry_repository_trait_has_3_methods() {
        fn _check_methods<T: OpsLogEntryRepository>() {}
        _check_methods::<PgOpsLogEntryRepository>();
    }
}

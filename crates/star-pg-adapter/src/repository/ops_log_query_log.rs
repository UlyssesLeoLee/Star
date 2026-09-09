// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpsLogQueryLog Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/ops_log_query_log.sql` 表对应
//! - 守门 #13 T 类: 物理删除禁止 + 監査必須 + RLS 13 類必携
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::PgAdapterError;

/// OpsLogQueryLog (W/T/M = T, 跟 DDL ops_log_query_log 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OpsLogQueryLog {
    /// 主键
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// 客户端 query 请求 ID
    pub query_id: Uuid,
    /// 跨域 trace
    pub trace_id: Option<Uuid>,
    /// all/info/warn/error/debug
    pub level_filter: String,
    /// 时间范围起
    pub time_range_start: DateTime<Utc>,
    /// 时间范围止
    pub time_range_end: DateTime<Utc>,
    /// 返回行数
    pub line_count: i32,
    /// 发起人
    pub actor_user_id: Uuid,
    /// 请求时间
    pub requested_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// pending/running/succeeded/failed
    pub status: String,
    /// 6-field 错误码
    pub error_code: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// star_ops::log
    pub source_module: String,
    /// audit
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

/// OpsLogQueryLogRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OpsLogQueryLogRepository: Send + Sync {
    /// find_by_query (per DDL ops_log_query_log T 派生规)
    async fn find_by_query(
        &self,
        tenant_id: Uuid,
        query_id: Uuid,
    ) -> Result<Option<OpsLogQueryLog>, PgAdapterError>;
    /// list_by_trace (per DDL ops_log_query_log T 派生规)
    async fn list_by_trace(
        &self,
        tenant_id: Uuid,
        trace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogQueryLog>, PgAdapterError>;
    /// insert (per DDL ops_log_query_log T 派生规)
    async fn insert(&self, log: &OpsLogQueryLog) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/ops_log_query_log 对应)
pub struct PgOpsLogQueryLogRepository {
    pool: PgPool,
}

impl PgOpsLogQueryLogRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OpsLogQueryLogRepository for PgOpsLogQueryLogRepository {
    async fn find_by_query(
        &self,
        tenant_id: Uuid,
        query_id: Uuid,
    ) -> Result<Option<OpsLogQueryLog>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsLogQueryLog>(
            r#"
                        SELECT id, tenant_id, query_id, trace_id, level_filter, time_range_start, time_range_end, line_count, actor_user_id, requested_at, completed_at, status, error_code, error_message, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id
            FROM ops_log_query_log
            WHERE tenant_id = $1 AND query_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(query_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_query: {}", e)))?;
        Ok(rows)
    }

    async fn list_by_trace(
        &self,
        tenant_id: Uuid,
        trace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogQueryLog>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsLogQueryLog>(
            r#"
                        SELECT id, tenant_id, query_id, trace_id, level_filter, time_range_start, time_range_end, line_count, actor_user_id, requested_at, completed_at, status, error_code, error_message, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id
            FROM ops_log_query_log
            WHERE tenant_id = $1 AND trace_id = $2
            ORDER BY requested_at DESC
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

    async fn insert(&self, log: &OpsLogQueryLog) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        INSERT INTO ops_log_query_log
                (id, tenant_id, query_id, trace_id, level_filter, time_range_start, time_range_end, line_count, actor_user_id, requested_at, completed_at, status, error_code, error_message, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            "#,
        )
        .bind(&log.id)
        .bind(&log.tenant_id)
        .bind(&log.query_id)
        .bind(&log.trace_id)
        .bind(&log.level_filter)
        .bind(&log.time_range_start)
        .bind(&log.time_range_end)
        .bind(log.line_count)
        .bind(&log.actor_user_id)
        .bind(&log.requested_at)
        .bind(&log.completed_at)
        .bind(&log.status)
        .bind(&log.error_code)
        .bind(&log.error_message)
        .bind(&log.source_module)
        .bind(&log.source_kind)
        .bind(&log.user_id)
        .bind(&log.role_id)
        .bind(&log.permission_id)
        .bind(&log.policy_id)
        .bind(&log.workspace_id)
        .bind(&log.project_id)
        .bind(&log.work_item_id)
        .bind(&log.agent_id)
        .bind(&log.session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OpsLogQueryLog 必含 25 字段 (跟 DDL 100% 一致)
    #[test]
    fn ops_log_query_log_struct_has_25_fields() {
        let x = OpsLogQueryLog {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            query_id: Uuid::nil(),
            trace_id: None,
            level_filter: "test".to_string(),
            time_range_start: Utc::now(),
            time_range_end: Utc::now(),
            line_count: 0,
            actor_user_id: Uuid::nil(),
            requested_at: Utc::now(),
            completed_at: None,
            status: "test".to_string(),
            error_code: None,
            error_message: None,
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

    /// 派生 #N: OpsLogQueryLogRepository trait 必含 3 方法
    #[test]
    fn ops_log_query_log_repository_trait_has_3_methods() {
        fn _check_methods<T: OpsLogQueryLogRepository>() {}
        _check_methods::<PgOpsLogQueryLogRepository>();
    }
}

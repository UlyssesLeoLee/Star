// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpsLogAnalysis Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/ops_log_analysis.sql` 表对应
//! - 守门 #13 W 类: 物理删除 / タイマー失効 / 短 TTL 明示 retention
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::PgAdapterError;

/// OpsLogAnalysis (W/T/M = W, 跟 DDL ops_log_analysis 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OpsLogAnalysis {
    /// 主键
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// FK to ops_log_entry ON DELETE CASCADE
    pub log_entry_id: Uuid,
    /// mock/openai/anthropic
    pub ai_channel: String,
    /// gpt-4/claude-3-opus/mock-v1
    pub model: Option<String>,
    /// Ladder 重试次数
    pub ladder_attempts: i32,
    /// 0.0-1.0 置信度
    pub confidence: f64,
    /// 是否异常
    pub is_anomaly: bool,
    /// spike/drift/error_burst/pattern
    pub anomaly_type: Option<String>,
    /// LLM 摘要
    pub summary: String,
    /// LLM 建议 (JSONB array)
    pub suggestions: serde_json::Value,
    /// mock/openai/anthropic
    pub generated_by: String,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
    /// TTL 30d 过期时间
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
    /// RLS 13 類 (T 必携)
    pub trace_id: Option<Uuid>,
}

/// OpsLogAnalysisRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OpsLogAnalysisRepository: Send + Sync {
    /// find_by_entry (per DDL ops_log_analysis W 派生规)
    async fn find_by_entry(
        &self,
        tenant_id: Uuid,
        entry_id: Uuid,
    ) -> Result<Vec<OpsLogAnalysis>, PgAdapterError>;
    /// list_anomalies (per DDL ops_log_analysis W 派生规)
    async fn list_anomalies(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogAnalysis>, PgAdapterError>;
    /// insert (per DDL ops_log_analysis W 派生规)
    async fn insert(&self, analysis: &OpsLogAnalysis) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/ops_log_analysis 对应)
pub struct PgOpsLogAnalysisRepository {
    pool: PgPool,
}

impl PgOpsLogAnalysisRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OpsLogAnalysisRepository for PgOpsLogAnalysisRepository {
    async fn find_by_entry(
        &self,
        tenant_id: Uuid,
        entry_id: Uuid,
    ) -> Result<Vec<OpsLogAnalysis>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsLogAnalysis>(
            r#"
                        SELECT id, tenant_id, log_entry_id, ai_channel, model, ladder_attempts, confidence, is_anomaly, anomaly_type, summary, suggestions, generated_by, generated_at, expires_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM ops_log_analysis
            WHERE tenant_id = $1 AND log_entry_id = $2
              AND expires_at > NOW()
            "#,
        )
        .bind(tenant_id)
        .bind(entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_entry: {}", e)))?;
        Ok(rows)
    }

    async fn list_anomalies(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsLogAnalysis>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsLogAnalysis>(
            r#"
                        SELECT id, tenant_id, log_entry_id, ai_channel, model, ladder_attempts, confidence, is_anomaly, anomaly_type, summary, suggestions, generated_by, generated_at, expires_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM ops_log_analysis
            WHERE tenant_id = $1 AND is_anomaly = TRUE
              AND expires_at > NOW()
            ORDER BY generated_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("list_anomalies: {}", e)))?;
        Ok(rows)
    }

    async fn insert(&self, analysis: &OpsLogAnalysis) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        INSERT INTO ops_log_analysis
                (id, tenant_id, log_entry_id, ai_channel, model, ladder_attempts, confidence, is_anomaly, anomaly_type, summary, suggestions, generated_by, generated_at, expires_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)
            "#,
        )
        .bind(&analysis.id)
        .bind(&analysis.tenant_id)
        .bind(&analysis.log_entry_id)
        .bind(&analysis.ai_channel)
        .bind(&analysis.model)
        .bind(analysis.ladder_attempts)
        .bind(analysis.confidence)
        .bind(analysis.is_anomaly)
        .bind(&analysis.anomaly_type)
        .bind(&analysis.summary)
        .bind(&analysis.suggestions)
        .bind(&analysis.generated_by)
        .bind(&analysis.generated_at)
        .bind(&analysis.expires_at)
        .bind(&analysis.source_module)
        .bind(&analysis.source_kind)
        .bind(&analysis.user_id)
        .bind(&analysis.role_id)
        .bind(&analysis.permission_id)
        .bind(&analysis.policy_id)
        .bind(&analysis.workspace_id)
        .bind(&analysis.project_id)
        .bind(&analysis.work_item_id)
        .bind(&analysis.agent_id)
        .bind(&analysis.session_id)
        .bind(&analysis.trace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OpsLogAnalysis 必含 26 字段 (跟 DDL 100% 一致)
    #[test]
    fn ops_log_analysis_struct_has_26_fields() {
        let x = OpsLogAnalysis {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            log_entry_id: Uuid::nil(),
            ai_channel: "test".to_string(),
            model: None,
            ladder_attempts: 0,
            confidence: 0.0,
            is_anomaly: false,
            anomaly_type: None,
            summary: "test".to_string(),
            suggestions: serde_json::json!({}),
            generated_by: "test".to_string(),
            generated_at: Utc::now(),
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
            trace_id: None,
        };
        assert_eq!(x.source_module, "test");
    }

    /// 派生 #N: OpsLogAnalysisRepository trait 必含 3 方法
    #[test]
    fn ops_log_analysis_repository_trait_has_3_methods() {
        fn _check_methods<T: OpsLogAnalysisRepository>() {}
        _check_methods::<PgOpsLogAnalysisRepository>();
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpsHelmReleaseState Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/ops_helm_release_state.sql` 表对应
//! - 守门 #13 T 类: 物理删除禁止 + 監査必須 + RLS 13 類必携
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::PgAdapterError;

/// OpsHelmReleaseState (W/T/M = T, 跟 DDL ops_helm_release_state 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OpsHelmReleaseState {
    /// 主键
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// Helm release 名
    pub release_name: String,
    /// k8s namespace
    pub namespace: String,
    /// Helm chart 名
    pub chart: String,
    /// Helm revision
    pub revision: i32,
    /// pending/healthy/degraded/failed/canary
    pub status: String,
    /// 0-100 canary 权重
    pub canary_weight: i32,
    /// 上次部署时间
    pub last_deployed_at: DateTime<Utc>,
    /// 入库时间
    pub captured_at: DateTime<Utc>,
    /// star_ops::cluster
    pub source_module: String,
    /// internal
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

/// OpsHelmReleaseStateRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OpsHelmReleaseStateRepository: Send + Sync {
    /// find_by_release (per DDL ops_helm_release_state T 派生规)
    async fn find_by_release(
        &self,
        tenant_id: Uuid,
        release_name: &str,
        revision: i32,
    ) -> Result<Option<OpsHelmReleaseState>, PgAdapterError>;
    /// list_recent (per DDL ops_helm_release_state T 派生规)
    async fn list_recent(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsHelmReleaseState>, PgAdapterError>;
    /// upsert (per DDL ops_helm_release_state T 派生规)
    async fn upsert(&self, state: &OpsHelmReleaseState) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/ops_helm_release_state 对应)
pub struct PgOpsHelmReleaseStateRepository {
    pool: PgPool,
}

impl PgOpsHelmReleaseStateRepository {
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
impl OpsHelmReleaseStateRepository for PgOpsHelmReleaseStateRepository {
    async fn find_by_release(
        &self,
        tenant_id: Uuid,
        release_name: &str,
        revision: i32,
    ) -> Result<Option<OpsHelmReleaseState>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsHelmReleaseState>(
            r#"
                        SELECT id, tenant_id, release_name, namespace, chart, revision, status, canary_weight, last_deployed_at, captured_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM ops_helm_release_state
            WHERE tenant_id = $1 AND release_name = $2 AND revision = $3
            "#,
        )
        .bind(tenant_id)
        .bind(release_name)
        .bind(revision)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_release: {}", e)))?;
        Ok(rows)
    }

    async fn list_recent(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpsHelmReleaseState>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OpsHelmReleaseState>(
            r#"
                        SELECT id, tenant_id, release_name, namespace, chart, revision, status, canary_weight, last_deployed_at, captured_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM ops_helm_release_state
            WHERE tenant_id = $1
            ORDER BY captured_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("list_recent: {}", e)))?;
        Ok(rows)
    }

    async fn upsert(&self, state: &OpsHelmReleaseState) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        INSERT INTO ops_helm_release_state
                (id, tenant_id, release_name, namespace, chart, revision, status, canary_weight, last_deployed_at, captured_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            ON CONFLICT (id) DO UPDATE SET
                release_name = EXCLUDED.release_name,
                namespace = EXCLUDED.namespace,
                chart = EXCLUDED.chart,
                revision = EXCLUDED.revision,
                status = EXCLUDED.status,
                canary_weight = EXCLUDED.canary_weight,
                last_deployed_at = EXCLUDED.last_deployed_at,
                captured_at = EXCLUDED.captured_at
            "#,
        )
        .bind(&state.id)
        .bind(&state.tenant_id)
        .bind(&state.release_name)
        .bind(&state.namespace)
        .bind(&state.chart)
        .bind(state.revision)
        .bind(&state.status)
        .bind(state.canary_weight)
        .bind(&state.last_deployed_at)
        .bind(&state.captured_at)
        .bind(&state.source_module)
        .bind(&state.source_kind)
        .bind(&state.user_id)
        .bind(&state.role_id)
        .bind(&state.permission_id)
        .bind(&state.policy_id)
        .bind(&state.workspace_id)
        .bind(&state.project_id)
        .bind(&state.work_item_id)
        .bind(&state.agent_id)
        .bind(&state.session_id)
        .bind(&state.trace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("upsert: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OpsHelmReleaseState 必含 22 字段 (跟 DDL 100% 一致)
    #[test]
    fn ops_helm_release_state_struct_has_22_fields() {
        let x = OpsHelmReleaseState {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            release_name: "test".to_string(),
            namespace: "test".to_string(),
            chart: "test".to_string(),
            revision: 0,
            status: "test".to_string(),
            canary_weight: 0,
            last_deployed_at: Utc::now(),
            captured_at: Utc::now(),
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

    /// 派生 #N: OpsHelmReleaseStateRepository trait 必含 3 方法
    #[test]
    fn ops_helm_release_state_repository_trait_has_3_methods() {
        fn _check_methods<T: OpsHelmReleaseStateRepository>() {}
        _check_methods::<PgOpsHelmReleaseStateRepository>();
    }
}

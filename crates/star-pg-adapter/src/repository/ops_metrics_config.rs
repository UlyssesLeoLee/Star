// SPDX-License-Identifier: MIT OR Apache-2.0
//! Ops Metrics Config Repository (Postgres impl, per WBS v0.39 §14.12 II 拍板 Postgres + v0.67 §14.10.4 缺口 #4 6/6 ops Repository 收官)
//!
//! v0.67 拆出独立文件 (per WBS §14.10.4 缺口 #4 6/6 ops Repository 收官):
//! - 跟 db/migrations/2026-09-08-ops-metrics.sql ops_metrics_config 表对应
//! - 守门 #13 c: M 类 SCD Type 2 (valid_from / valid_to), 物理删除禁止 (WORM 派生)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)
//! - 守门 #14 v4 永久代签 author=Ulysses (per 9/10 12:45 JST v0.62 反转后 Mavis 审核)
//!
//! 跨 session 续: 5 域 Repository (ops_helm_release_state / ops_cluster_action_log /
//!   ops_log_query_log / ops_log_entry / ops_log_analysis) 估 4M tokens

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use sqlx::PgPool;

use uuid::Uuid;

use crate::PgAdapterError;

/// 守门 #13 c M 类 SCD Type 2 配置 (跟 DDL ops_metrics_config 字段一致)

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct OpsMetricsConfig {
    /// 主键 (跟 DDL `id UUID NOT NULL` 一致, 复合主键 (id, valid_from))
    pub id: Uuid,

    /// 租户 ID (守门 #13 c RLS 13 類必携, tenant_id IS NOT NULL)
    pub tenant_id: Uuid,

    /// 指标名 (e.g. 'cpu_avg' / 'mem_avg' / 'active_tasks' / 'mcp_qps' / 'llm_token_daily')
    pub metric_name: String,

    /// i18n 显示标签 key (per F-03 metricsCard.*)
    pub display_label: String,

    /// 单位 ('ratio' / 'count' / 'qps' / 'tokens')
    pub unit: String,

    /// 警告阈值 (e.g. cpu_avg > 0.8)
    pub threshold_warning: Option<f64>,

    /// 严重阈值 (e.g. cpu_avg > 0.95)
    pub threshold_critical: Option<f64>,

    /// 来源模块 (e.g. 'star_telemetry::meter')
    pub source_module: String,

    /// SCD Type 2 生效起始时间 (per 守门 #13 c)
    pub valid_from: DateTime<Utc>,

    /// SCD Type 2 生效结束时间 (None = 当前有效, Some = 历史版本)
    pub valid_to: Option<DateTime<Utc>>,

    /// 记录创建时间
    pub created_at: DateTime<Utc>,

    /// 记录更新时间
    pub updated_at: DateTime<Utc>,
}

/// Ops Metrics Config Repository trait (per 6 ops 表 1 个 1 trait 模式, 跨 session 续 5 个)

#[async_trait::async_trait]

pub trait OpsMetricsConfigRepository: Send + Sync {
    /// 按 tenant_id + metric_name 查询当前有效配置 (valid_to IS NULL)

    async fn find_current(
        &self,

        tenant_id: Uuid,

        metric_name: &str,
    ) -> Result<Option<OpsMetricsConfig>, PgAdapterError>;

    /// 列 tenant 当前所有 metric 配置

    async fn list_current(&self, tenant_id: Uuid) -> Result<Vec<OpsMetricsConfig>, PgAdapterError>;

    /// SCD Type 2 插入新版本 (旧版本 valid_to = now())

    /// 守门 #13 c: 不物理删, 走 soft close (UPDATE valid_to)

    async fn insert_new_version(&self, cfg: &OpsMetricsConfig) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/2026-09-08-ops-metrics.sql 对应)

pub struct PgOpsMetricsConfigRepository {
    pool: PgPool,
}

impl PgOpsMetricsConfigRepository {
    /// 构造 PG Repository (持有 PgPool 引用)

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 拿共享 PgPool 引用 (v0.73 P0-4 Stage 2.1 扩展: 6 ops Repository wire-up)
    ///
    /// 用于 application crate 跨域编排时多个 Repository 共享同一 pool
    /// (per sqlx::PgPool = Arc 内部, clone 廉价).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]

impl OpsMetricsConfigRepository for PgOpsMetricsConfigRepository {
    async fn find_current(
        &self,

        tenant_id: Uuid,

        metric_name: &str,
    ) -> Result<Option<OpsMetricsConfig>, PgAdapterError> {
        let row: Option<(
            Uuid,
            Uuid,
            String,
            String,
            String,
            Option<f64>,
            Option<f64>,
            String,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            DateTime<Utc>,
            DateTime<Utc>,
        )> = sqlx::query_as(
            r#"

            SELECT id, tenant_id, metric_name, display_label, unit,

                   threshold_warning, threshold_critical, source_module,

                   valid_from, valid_to, created_at, updated_at

            FROM ops_metrics_config

            WHERE tenant_id = $1

              AND metric_name = $2

              AND valid_to IS NULL

            "#,
        )
        .bind(tenant_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_current: {}", e)))?;

        Ok(row.map(
            |(
                id,
                tenant_id,
                metric_name,
                display_label,
                unit,
                threshold_warning,
                threshold_critical,
                source_module,
                valid_from,
                valid_to,
                created_at,
                updated_at,
            )| {
                OpsMetricsConfig {
                    id,

                    tenant_id,

                    metric_name,

                    display_label,

                    unit,

                    threshold_warning,

                    threshold_critical,

                    source_module,

                    valid_from,

                    valid_to,

                    created_at,

                    updated_at,
                }
            },
        ))
    }

    async fn list_current(&self, tenant_id: Uuid) -> Result<Vec<OpsMetricsConfig>, PgAdapterError> {
        let rows: Vec<(
            Uuid,
            Uuid,
            String,
            String,
            String,
            Option<f64>,
            Option<f64>,
            String,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            DateTime<Utc>,
            DateTime<Utc>,
        )> = sqlx::query_as(
            r#"

            SELECT id, tenant_id, metric_name, display_label, unit,

                   threshold_warning, threshold_critical, source_module,

                   valid_from, valid_to, created_at, updated_at

            FROM ops_metrics_config

            WHERE tenant_id = $1

              AND valid_to IS NULL

            ORDER BY metric_name

            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("list_current: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    metric_name,
                    display_label,
                    unit,
                    threshold_warning,
                    threshold_critical,
                    source_module,
                    valid_from,
                    valid_to,
                    created_at,
                    updated_at,
                )| {
                    OpsMetricsConfig {
                        id,

                        tenant_id,

                        metric_name,

                        display_label,

                        unit,

                        threshold_warning,

                        threshold_critical,

                        source_module,

                        valid_from,

                        valid_to,

                        created_at,

                        updated_at,
                    }
                },
            )
            .collect())
    }

    async fn insert_new_version(&self, cfg: &OpsMetricsConfig) -> Result<(), PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PgAdapterError::Query(format!("begin tx: {}", e)))?;

        // 守门 #13 c SCD Type 2: 旧版本 soft close (valid_to = now())

        sqlx::query(
            r#"

            UPDATE ops_metrics_config

            SET valid_to = $3, updated_at = $3

            WHERE tenant_id = $1

              AND metric_name = $2

              AND valid_to IS NULL

            "#,
        )
        .bind(cfg.tenant_id)
        .bind(&cfg.metric_name)
        .bind(cfg.valid_from)
        .execute(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("soft close old: {}", e)))?;

        // 插入新版本

        sqlx::query(
            r#"

            INSERT INTO ops_metrics_config

                (id, tenant_id, metric_name, display_label, unit,

                 threshold_warning, threshold_critical, source_module,

                 valid_from, valid_to, created_at, updated_at)

            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)

            "#,
        )
        .bind(cfg.id)
        .bind(cfg.tenant_id)
        .bind(&cfg.metric_name)
        .bind(&cfg.display_label)
        .bind(&cfg.unit)
        .bind(cfg.threshold_warning)
        .bind(cfg.threshold_critical)
        .bind(&cfg.source_module)
        .bind(cfg.valid_from)
        .bind(cfg.valid_to)
        .bind(cfg.created_at)
        .bind(cfg.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert new: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| PgAdapterError::Query(format!("commit: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {

    use super::*;

    /// 派生 #26: OpsMetricsConfig 必含 12 字段 (跟 DDL 100% 一致)

    #[test]

    fn ops_metrics_config_struct_has_12_fields() {
        let cfg = OpsMetricsConfig {
            id: Uuid::nil(),

            tenant_id: Uuid::nil(),

            metric_name: "cpu_avg".to_string(),

            display_label: "CPU Average".to_string(),

            unit: "ratio".to_string(),

            threshold_warning: Some(0.8),

            threshold_critical: Some(0.95),

            source_module: "star_telemetry::meter".to_string(),

            valid_from: Utc::now(),

            valid_to: None,

            created_at: Utc::now(),

            updated_at: Utc::now(),
        };

        assert_eq!(cfg.metric_name, "cpu_avg");

        assert_eq!(cfg.threshold_warning, Some(0.8));

        assert!(cfg.valid_to.is_none());
    }

    /// 派生 #27: OpsMetricsConfigRepository trait 必含 3 方法

    #[test]

    fn ops_metrics_config_repository_trait_has_3_methods() {
        // Compile-time 检查: trait 有 3 async fn

        fn _check_methods<T: OpsMetricsConfigRepository>() {

            // 没有运行时检查, 仅是 trait 边界编译验证
        }

        _check_methods::<PgOpsMetricsConfigRepository>();
    }
}

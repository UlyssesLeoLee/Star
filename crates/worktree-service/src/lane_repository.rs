//! `lane_repository.rs` — PG-backed `lane` + `multica.worktree_lane_binding` 持久化层
//!
//! Per P0-lane (2026-11-20):
//! - `Lane` 实体 PG 仓储, 照 `pg_worktree_repository.rs` 模式 (共享 PgPool, 单表/单查询,
//!   事务边界交给调用方 group_api handler)
//! - Worktree↔Lane 归属走 `multica.worktree_lane_binding` (照 `multica.worktree_project_binding`
//!   的 valid_from/valid_to 双时态模式), 不 ALTER 既有 `worktree_canvas_worktree` 表
//! - 全部 runtime-checked sqlx (`sqlx::query*` + `try_get`), 与 group_api 现有 handler 一致
//!
//! 守门:
//! - #5 env 安全: env URL 不打印, sqlx 自带
//! - #11 缺标比错标: sqlx / chrono / uuid 全部来自 [workspace.dependencies]

use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::error::ServiceError;

/// lane 行 (PG 视图, 与 domain-project::lane::Lane 平级; 仓储层用裸 Uuid,
/// 强类型 ID 在 domain 层 — 见 P0-lane 报告父代理待办)
#[derive(Debug, Clone)]
pub struct LaneRow {
    /// Lane UUID (PK)
    pub id: Uuid,
    /// 租户 UUID (INV-LANE-01)
    pub tenant_id: Uuid,
    /// 所属 Project UUID (INV-LANE-01)
    pub project_id: Uuid,
    /// 展示名称 (Project 内唯一)
    pub name: String,
    /// 描述 (可选)
    pub description: Option<String>,
    /// 创建者用户 UUID
    pub created_by: Uuid,
    /// 乐观锁版本 (INV-LANE-05)
    pub lock_version: i32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// worktree_lane_binding 行 (双时态绑定, 照 worktree_project_binding 模式)
#[derive(Debug, Clone)]
pub struct LaneWorktreeBindingRow {
    /// 绑定主键
    pub binding_id: Uuid,
    /// 租户 UUID
    pub tenant_id: Uuid,
    /// 所属 Project UUID
    pub project_id: Uuid,
    /// 目标 Lane UUID
    pub lane_id: Uuid,
    /// 目标 Worktree UUID
    pub worktree_id: Uuid,
    /// 绑定操作人
    pub assigned_by: Uuid,
    /// 生效时间
    pub valid_from: DateTime<Utc>,
    /// 失效时间 (NULL = 当前有效)
    pub valid_to: Option<DateTime<Utc>>,
    /// 绑定版本
    pub version: i32,
    /// 记录创建时间
    pub created_at: DateTime<Utc>,
}

/// Lane 维度 Worktree 投影行 (worktree_canvas_worktree JOIN 绑定表)
#[derive(Debug, Clone)]
pub struct LaneWorktreeRow {
    /// Worktree UUID
    pub worktree_id: Uuid,
    /// 所属 Lane UUID
    pub lane_id: Uuid,
    /// 所属 Project UUID
    pub project_id: Uuid,
    /// 仓库 UUID
    pub repository_id: Uuid,
    /// Worktree 名称
    pub name: String,
    /// 分支名
    pub branch: String,
    /// 7 态 human_state
    pub human_state: String,
    /// machine_state
    pub machine_state: String,
    /// Runtime UUID (可选)
    pub runtime_id: Option<Uuid>,
    /// Agent UUID (可选)
    pub agent_id: Option<Uuid>,
    /// AgentSession UUID (可选)
    pub agent_session_id: Option<Uuid>,
    /// 归档标记
    pub archived: bool,
    /// Worktree 乐观锁版本
    pub version: i32,
    /// lane 绑定主键
    pub binding_id: Uuid,
    /// lane 绑定版本
    pub binding_version: i32,
    /// 关联的 Engineering Run UUID (可选 — 未挂 run 的 worktree 为 NULL,
    /// 前端诚实标注「未绑定」而非伪造 run 归属)
    pub engineering_run_id: Option<Uuid>,
}

/// PG 持久化 Repository — 共享 `PgPool`, wrap 标准 CRUD / query
#[derive(Clone)]
pub struct PgLaneRepository {
    pool: PgPool,
}

impl PgLaneRepository {
    /// 构造
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 取底层 pool (用于调用方组合事务)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 插入 Lane
    pub async fn insert_lane(&self, lane: &LaneRow) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO lane (id, tenant_id, project_id, name, description, created_by,
                              lock_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(lane.id)
        .bind(lane.tenant_id)
        .bind(lane.project_id)
        .bind(&lane.name)
        .bind(lane.description.as_deref())
        .bind(lane.created_by)
        .bind(lane.lock_version)
        .bind(lane.created_at)
        .bind(lane.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(())
    }

    /// 按 (tenant, id) 找 Lane (None = 不存在)
    pub async fn find_by_id(
        &self,
        tenant_id: Uuid,
        lane_id: Uuid,
    ) -> Result<Option<LaneRow>, ServiceError> {
        let row = sqlx::query(
            r#"
            SELECT id, tenant_id, project_id, name, description, created_by,
                   lock_version, created_at, updated_at
            FROM lane
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(lane_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        row.map(row_to_lane).transpose()
    }

    /// 按 Project 列出 Lane (keyset 分页: (created_at, id) < (cursor_time, cursor_id))
    pub async fn list_by_project(
        &self,
        tenant_id: Uuid,
        project_id: Uuid,
        limit: i64,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<Vec<LaneRow>, ServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, project_id, name, description, created_by,
                   lock_version, created_at, updated_at
            FROM lane
            WHERE tenant_id = $1 AND project_id = $2
              AND ($3::TIMESTAMPTZ IS NULL OR (created_at, id) < ($3, $4))
            ORDER BY created_at DESC, id DESC
            LIMIT $5
            "#,
        )
        .bind(tenant_id)
        .bind(project_id)
        .bind(cursor.map(|(at, _)| at))
        .bind(cursor.map(|(_, id)| id))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        rows.into_iter().map(row_to_lane).collect()
    }

    /// 乐观锁更新 Lane — 版本不匹配时 rows_affected = 0 (调用方映射 Conflict)
    pub async fn update_lane(
        &self,
        lane: &LaneRow,
        expected_version: i32,
    ) -> Result<u64, ServiceError> {
        let result = sqlx::query(
            r#"
            UPDATE lane
            SET name = $4, description = $5, lock_version = lock_version + 1, updated_at = $6
            WHERE tenant_id = $1 AND id = $2 AND project_id = $3 AND lock_version = $7
            "#,
        )
        .bind(lane.tenant_id)
        .bind(lane.id)
        .bind(lane.project_id)
        .bind(&lane.name)
        .bind(lane.description.as_deref())
        .bind(lane.updated_at)
        .bind(expected_version)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(result.rows_affected())
    }

    /// 删除 Lane (绑定行经 FK ON DELETE CASCADE 级联清理)
    pub async fn delete_lane(&self, tenant_id: Uuid, lane_id: Uuid) -> Result<u64, ServiceError> {
        let result = sqlx::query("DELETE FROM lane WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(lane_id)
            .execute(&self.pool)
            .await
            .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(result.rows_affected())
    }

    /// 关闭 worktree 当前生效的 lane 绑定 (双时态: valid_to = now)
    pub async fn close_active_worktree_binding(
        &self,
        tenant_id: Uuid,
        worktree_id: Uuid,
    ) -> Result<u64, ServiceError> {
        let result = sqlx::query(
            r#"
            UPDATE multica.worktree_lane_binding
            SET valid_to = now()
            WHERE tenant_id = $1 AND worktree_id = $2 AND valid_to IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(worktree_id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(result.rows_affected())
    }

    /// 插入新 lane 绑定 (调用方需先 close_active_worktree_binding)
    pub async fn insert_worktree_binding(
        &self,
        binding: &LaneWorktreeBindingRow,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO multica.worktree_lane_binding
                (binding_id, tenant_id, project_id, lane_id, worktree_id, assigned_by,
                 valid_from, valid_to, version, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(binding.binding_id)
        .bind(binding.tenant_id)
        .bind(binding.project_id)
        .bind(binding.lane_id)
        .bind(binding.worktree_id)
        .bind(binding.assigned_by)
        .bind(binding.valid_from)
        .bind(binding.valid_to)
        .bind(binding.version)
        .bind(binding.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(())
    }

    /// 校验 worktree 是否属于该 project (active project binding)
    pub async fn worktree_in_project(
        &self,
        tenant_id: Uuid,
        project_id: Uuid,
        worktree_id: Uuid,
    ) -> Result<bool, ServiceError> {
        let found: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT p.worktree_id
            FROM multica.worktree_project_binding p
            WHERE p.tenant_id = $1 AND p.project_id = $2 AND p.worktree_id = $3
              AND p.valid_to IS NULL
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(project_id)
        .bind(worktree_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(found.is_some())
    }

    /// 按 Lane 列出 Worktree 投影 (active 绑定 + LEFT JOIN run 绑定)
    pub async fn list_worktrees_by_lane(
        &self,
        tenant_id: Uuid,
        lane_id: Uuid,
        limit: i64,
    ) -> Result<Vec<LaneWorktreeRow>, ServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT w.id, b.lane_id, w.project_id, w.repo_id, w.name, w.branch,
                   w.human_state, w.machine_state, w.runtime_id, w.agent_id,
                   w.agent_session_id, w.archived, w.version,
                   b.binding_id, b.version AS binding_version,
                   erb.engineering_run_id
            FROM multica.worktree_lane_binding b
            JOIN worktree_canvas_worktree w
              ON w.tenant_id = b.tenant_id AND w.id = b.worktree_id
            LEFT JOIN multica.engineering_run_worktree_binding erb
              ON erb.tenant_id = b.tenant_id AND erb.worktree_id = b.worktree_id
             AND erb.valid_to IS NULL
            WHERE b.tenant_id = $1 AND b.lane_id = $2 AND b.valid_to IS NULL
            ORDER BY w.created_at DESC, w.id DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(lane_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        rows.into_iter().map(row_to_lane_worktree).collect()
    }
}

/// sqlx::Error → `ServiceError` (LANE.IO_FAIL, 守门 #5 不打印 env URL)
fn map_sqlx_err(e: sqlx::Error, trace_id: &str) -> ServiceError {
    ServiceError::new(
        "LANE.IO_FAIL",
        format!("PG lane_repository query failed: {e}"),
        trace_id,
    )
    .with_source(format!("{e:?}"))
    .with_location("lane_repository.rs".to_string())
}

/// 单行 → `LaneRow` 解析
fn row_to_lane(row: sqlx::postgres::PgRow) -> Result<LaneRow, ServiceError> {
    Ok(LaneRow {
        id: row.try_get("id").map_err(|e| map_sqlx_err(e, "trace"))?,
        tenant_id: row
            .try_get("tenant_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        project_id: row
            .try_get("project_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        name: row.try_get("name").map_err(|e| map_sqlx_err(e, "trace"))?,
        description: row
            .try_get("description")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        created_by: row
            .try_get("created_by")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        lock_version: row
            .try_get("lock_version")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        created_at: row
            .try_get("created_at")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
    })
}

/// 单行 → `LaneWorktreeRow` 解析
fn row_to_lane_worktree(row: sqlx::postgres::PgRow) -> Result<LaneWorktreeRow, ServiceError> {
    Ok(LaneWorktreeRow {
        worktree_id: row
            .try_get("id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        lane_id: row
            .try_get("lane_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        project_id: row
            .try_get("project_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        repository_id: row
            .try_get("repo_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        name: row.try_get("name").map_err(|e| map_sqlx_err(e, "trace"))?,
        branch: row
            .try_get("branch")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        human_state: row
            .try_get("human_state")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        machine_state: row
            .try_get("machine_state")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        runtime_id: row
            .try_get("runtime_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        agent_id: row
            .try_get("agent_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        agent_session_id: row
            .try_get("agent_session_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        archived: row
            .try_get("archived")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        version: row
            .try_get("version")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        binding_id: row
            .try_get("binding_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        binding_version: row
            .try_get("binding_version")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        engineering_run_id: row
            .try_get("engineering_run_id")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
    })
}

// =====================================================================
// 测试 (per P0-lane 验证: 不连真 PG, 走 lazy pool + 映射回环)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用 lazy pool — 不连真 PG (照 pg_worktree_repository 守门 #1 模式)
    async fn test_pool_lazy_async() -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail")
    }

    #[tokio::test]
    async fn repository_new_holds_pool() {
        let pool = test_pool_lazy_async().await;
        let repo = PgLaneRepository::new(pool.clone());
        assert!(repo.pool().size() <= 1);
    }

    #[test]
    fn lane_row_fields_round_trip() {
        let now = Utc::now();
        let row = LaneRow {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            name: "core-runtime".into(),
            description: Some("desc".into()),
            created_by: Uuid::new_v4(),
            lock_version: 1,
            created_at: now,
            updated_at: now,
        };
        let clone = row.clone();
        assert_eq!(clone.name, "core-runtime");
        assert_eq!(clone.lock_version, 1);
        assert_eq!(clone.created_at, now);
    }

    #[test]
    fn binding_row_version_must_be_positive() {
        let row = LaneWorktreeBindingRow {
            binding_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            lane_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            assigned_by: Uuid::new_v4(),
            valid_from: Utc::now(),
            valid_to: None,
            version: 1,
            created_at: Utc::now(),
        };
        assert_eq!(row.version, 1);
        assert!(row.valid_to.is_none());
    }

    #[test]
    fn lane_worktree_row_carries_nullable_run() {
        let row = LaneWorktreeRow {
            worktree_id: Uuid::new_v4(),
            lane_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            name: "wt-a".into(),
            branch: "feat/a".into(),
            human_state: "ready".into(),
            machine_state: "idle".into(),
            runtime_id: None,
            agent_id: None,
            agent_session_id: None,
            archived: false,
            version: 1,
            binding_id: Uuid::new_v4(),
            binding_version: 1,
            engineering_run_id: None, // 未挂 run: 诚实 NULL, 不伪造
        };
        assert!(row.engineering_run_id.is_none());
    }
}

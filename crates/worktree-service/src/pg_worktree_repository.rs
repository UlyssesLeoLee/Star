//! `pg_worktree_repository.rs` — PG-backed `worktree_canvas_worktree` 持久化层
//!
//! Per ULYS-195 stage 2 PR #4 (FR-ORCA-011):
//! - 把 `Worktree` ↔ `worktree_canvas_worktree` 行做 ORM 映射
//! - `upsert_by_id` / `upsert_by_repo_path` 走 `ON CONFLICT ... DO UPDATE` (per issue §6 DB upsert)
//! - 只接 `repo_id, path, import_source` 新 schema 字段;其 14 字段沿用 2026-09-16 migration
//!
//! 故意**不**实装:
//! - 全 12 方法 (走 `PgWorktreeService` 自己直接 query,不另起 repo trait)
//! - 跨多个 worktree 实装同一事务 (本期单表 upsert,事务边界交给 service 层)
//!
//! 守门:
//! - #5 env 安全: env URL 不打印, sqlx 自带
//! - #13 a/d W/T/M 100%: W 走 PG upsert,M SCD Type 2 已有 pgpool_version 列 (per stage 2 列增量)
//! - #11 缺标比错标: sqlx / chrono / uuid / serde 全部来自 [workspace.dependencies]

use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::path::PathBuf;
use uuid::Uuid;

use graph_core::state::{HumanState, TestState};
use graph_core::types::{RepoId, UserId, WorktreeId};

use crate::error::ServiceError;
use crate::service::Worktree;

/// worktree 行 (PG 视图, 跟 Worktree 业务 struct 平级)
#[derive(Debug, Clone)]
pub struct WorktreeRow {
    /// Worktree UUID (PK)
    pub id: WorktreeId,
    /// Repo UUID (FK worktree_canvas_graph_node)
    pub repo_id: RepoId,
    /// Worktree name (per DD §2.1)
    pub name: String,
    /// Branch name (per DD §2.1)
    pub branch: String,
    /// Worktree 路径 (per DD §2.1)
    pub path: PathBuf,
    /// 7 态 (per DD §2.1: Running/Waiting/Ready/Diverged/Conflict/Merged/Stale)
    pub human_state: HumanState,
    /// Ahead of main (per DD §2.1)
    pub ahead: i32,
    /// Behind main (per DD §2.1)
    pub behind: i32,
    /// Dirty (working tree) flag
    pub dirty: bool,
    /// 0-100 health (per DD §2.1 + §12)
    pub health_score: i16,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// 4 档 (per DD §2.1: passed/failed/running/none)
    pub test_state: TestState,
    /// Risk count (per DD §2.1)
    pub risk_count: i32,
    /// Locked flag
    pub locked: bool,
    /// Archived flag
    pub archived: bool,
    /// Agent ID (per DD §2.1 Worktree.agent_id)
    pub agent_id: Option<Uuid>,
    /// Task ID (per DD §2.1 Worktree.task_id)
    pub task_id: Option<Uuid>,
    /// Parent ID (per DD §2.1 Worktree.parent_id)
    pub parent_id: Option<Uuid>,
    /// Merged at timestamp
    pub merged_at: Option<DateTime<Utc>>,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
    /// Lock owner (per Worktree.locked_by)
    pub locked_by: Option<UserId>,
    /// Import source: internal | external (per ULYS-195 stage 2 PR #4)
    pub import_source: String,
}

impl WorktreeRow {
    /// `WorktreeRow` → `Worktree` (业务 DTO)
    pub fn into_worktree(self) -> Worktree {
        Worktree {
            id: self.id,
            repo_id: self.repo_id,
            name: self.name,
            branch: self.branch,
            path: self.path,
            human_state: self.human_state,
            test_state: self.test_state,
            ahead: self.ahead.clamp(0, i32::MAX) as u32,
            behind: self.behind.clamp(0, i32::MAX) as u32,
            dirty: self.dirty,
            locked: self.locked,
            archived: self.archived,
            health_score: self.health_score.clamp(0, 100) as u8,
            last_activity: self.last_activity,
            created_at: self.created_at,
            merged_at: self.merged_at,
            locked_by: self.locked_by,
            agent_type: self.agent_id.map(|id| id.to_string()),
        }
    }
}

impl From<Worktree> for WorktreeRow {
    fn from(wt: Worktree) -> Self {
        Self {
            id: wt.id,
            repo_id: wt.repo_id,
            name: wt.name,
            branch: wt.branch,
            path: wt.path,
            human_state: wt.human_state,
            ahead: wt.ahead as i32,
            behind: wt.behind as i32,
            dirty: wt.dirty,
            health_score: wt.health_score as i16,
            last_activity: wt.last_activity,
            test_state: wt.test_state,
            risk_count: 0,
            locked: wt.locked,
            archived: wt.archived,
            agent_id: wt.agent_type.as_ref().and_then(|s| Uuid::parse_str(s).ok()),
            task_id: None,
            parent_id: None,
            merged_at: wt.merged_at,
            created_at: wt.created_at,
            updated_at: Utc::now(),
            locked_by: wt.locked_by,
            import_source: wt
                .agent_type
                .as_deref()
                .and_then(|s| if s == "external_import" { Some("external".to_string()) } else { None })
                .unwrap_or_else(|| "internal".to_string()),
        }
    }
}

/// human_state 字符串 ↔ `HumanState` enum 双向映射 (per DD §2.1 7 态)
fn human_state_to_str(s: HumanState) -> &'static str {
    match s {
        HumanState::Running => "running",
        HumanState::Waiting => "waiting",
        HumanState::Ready => "ready",
        HumanState::Diverged => "diverged",
        HumanState::Conflict => "conflict",
        HumanState::Merged => "merged",
        HumanState::Stale => "stale",
    }
}

fn human_state_from_str(s: &str) -> Result<HumanState, ServiceError> {
    match s {
        "running" => Ok(HumanState::Running),
        "waiting" => Ok(HumanState::Waiting),
        "ready" => Ok(HumanState::Ready),
        "diverged" => Ok(HumanState::Diverged),
        "conflict" => Ok(HumanState::Conflict),
        "merged" => Ok(HumanState::Merged),
        "stale" => Ok(HumanState::Stale),
        other => Err(ServiceError::new(
            "WT.IO_FAIL",
            format!("unknown human_state in DB: {other}"),
            "trace",
        )),
    }
}

/// test_state 字符串 ↔ `TestState` enum 双向映射 (4 态: none/passed/failed/running)
fn test_state_to_str(s: TestState) -> &'static str {
    match s {
        TestState::None => "none",
        TestState::Passed => "passed",
        TestState::Failed => "failed",
        TestState::Running => "running",
    }
}

fn test_state_from_str(s: &str) -> Result<TestState, ServiceError> {
    match s {
        "none" => Ok(TestState::None),
        "passed" => Ok(TestState::Passed),
        "failed" => Ok(TestState::Failed),
        "running" => Ok(TestState::Running),
        other => Err(ServiceError::new(
            "WT.IO_FAIL",
            format!("unknown test_state in DB: {other}"),
            "trace",
        )),
    }
}

/// PG 持久化 Repository — 共享 `PgPool`,wrap 标准 upsert / query
#[derive(Clone)]
pub struct PgWorktreeRepository {
    pool: PgPool,
}

impl PgWorktreeRepository {
    /// 构造
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 取底层 pool (用于 service 调事务 / multi-stmt 等高级用法)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 按 ID 找 (返回 None = 不存在, 不抛错 — 区别于 service.get())
    pub async fn find_by_id(&self, id: WorktreeId) -> Result<Option<WorktreeRow>, ServiceError> {
        let row = sqlx::query(
            r#"
            SELECT
                id, repo_id, name, branch, path, human_state, ahead, behind,
                dirty, health_score, last_activity, test_state, risk_count,
                locked, archived, agent_id, task_id, parent_id, merged_at,
                created_at, updated_at, locked_by, import_source
            FROM worktree_canvas_worktree
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        row.map(row_to_worktree_row).transpose()
    }

    /// 按 (repo_id, path) 找 — `import_external_worktrees` 用
    pub async fn find_by_repo_path(
        &self,
        repo_id: RepoId,
        path: &std::path::Path,
    ) -> Result<Option<WorktreeRow>, ServiceError> {
        let path_str = path.to_string_lossy().to_string();
        let row = sqlx::query(
            r#"
            SELECT
                id, repo_id, name, branch, path, human_state, ahead, behind,
                dirty, health_score, last_activity, test_state, risk_count,
                locked, archived, agent_id, task_id, parent_id, merged_at,
                created_at, updated_at, locked_by, import_source
            FROM worktree_canvas_worktree
            WHERE repo_id = $1 AND path = $2
            "#,
        )
        .bind(repo_id)
        .bind(&path_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        row.map(row_to_worktree_row).transpose()
    }

    /// 列出某个 repo 下全部 Worktree (无 filter, 全返回)
    pub async fn list_by_repo(&self, repo_id: RepoId) -> Result<Vec<WorktreeRow>, ServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, repo_id, name, branch, path, human_state, ahead, behind,
                dirty, health_score, last_activity, test_state, risk_count,
                locked, archived, agent_id, task_id, parent_id, merged_at,
                created_at, updated_at, locked_by, import_source
            FROM worktree_canvas_worktree
            WHERE repo_id = $1
            "#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        rows.into_iter().map(row_to_worktree_row).collect()
    }

    /// 单条 upsert (per `Worktree` PK, 走 ON CONFLICT (id) DO UPDATE)
    /// 不接 `import_source` 列 — 改用 `upsert_external` (per FR-ORCA-011 §2.4 AC)
    pub async fn upsert_by_id(&self, wt: &WorktreeRow) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO worktree_canvas_worktree (
                id, repo_id, name, branch, path, human_state, ahead, behind,
                dirty, health_score, last_activity, test_state, risk_count,
                locked, archived, agent_id, task_id, parent_id, merged_at,
                created_at, updated_at, locked_by, import_source
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23
            )
            ON CONFLICT (id) DO UPDATE SET
                repo_id       = EXCLUDED.repo_id,
                name          = EXCLUDED.name,
                branch        = EXCLUDED.branch,
                path          = EXCLUDED.path,
                human_state   = EXCLUDED.human_state,
                ahead         = EXCLUDED.ahead,
                behind        = EXCLUDED.behind,
                dirty         = EXCLUDED.dirty,
                health_score  = EXCLUDED.health_score,
                last_activity = EXCLUDED.last_activity,
                test_state    = EXCLUDED.test_state,
                risk_count    = EXCLUDED.risk_count,
                locked        = EXCLUDED.locked,
                archived      = EXCLUDED.archived,
                agent_id      = EXCLUDED.agent_id,
                task_id       = EXCLUDED.task_id,
                parent_id     = EXCLUDED.parent_id,
                merged_at     = EXCLUDED.merged_at,
                updated_at    = EXCLUDED.updated_at,
                locked_by     = EXCLUDED.locked_by,
                import_source = EXCLUDED.import_source
            "#,
        )
        .bind(wt.id)
        .bind(wt.repo_id)
        .bind(&wt.name)
        .bind(&wt.branch)
        .bind(wt.path.to_string_lossy().to_string())
        .bind(human_state_to_str(wt.human_state))
        .bind(wt.ahead)
        .bind(wt.behind)
        .bind(wt.dirty)
        .bind(wt.health_score)
        .bind(wt.last_activity)
        .bind(test_state_to_str(wt.test_state))
        .bind(wt.risk_count)
        .bind(wt.locked)
        .bind(wt.archived)
        .bind(wt.agent_id)
        .bind(wt.task_id)
        .bind(wt.parent_id)
        .bind(wt.merged_at)
        .bind(wt.created_at)
        .bind(wt.updated_at)
        .bind(wt.locked_by)
        .bind(&wt.import_source)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(())
    }

    /// 外部 worktree 导入 upsert (per FR-ORCA-011 §2.4 AC)
    /// ON CONFLICT (repo_id, path) DO UPDATE SET branch=..., last_activity=NOW()
    pub async fn upsert_external(
        &self,
        wt: &WorktreeRow,
        import_source: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO worktree_canvas_worktree (
                id, repo_id, name, branch, path, human_state, ahead, behind,
                dirty, health_score, last_activity, test_state, risk_count,
                locked, archived, agent_id, task_id, parent_id, merged_at,
                created_at, updated_at, locked_by, import_source
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23
            )
            ON CONFLICT (repo_id, path) DO UPDATE SET
                branch        = EXCLUDED.branch,
                last_activity = EXCLUDED.last_activity,
                import_source = EXCLUDED.import_source,
                updated_at    = EXCLUDED.updated_at
            "#,
        )
        .bind(wt.id)
        .bind(wt.repo_id)
        .bind(&wt.name)
        .bind(&wt.branch)
        .bind(wt.path.to_string_lossy().to_string())
        .bind(human_state_to_str(wt.human_state))
        .bind(wt.ahead)
        .bind(wt.behind)
        .bind(wt.dirty)
        .bind(wt.health_score)
        .bind(wt.last_activity)
        .bind(test_state_to_str(wt.test_state))
        .bind(wt.risk_count)
        .bind(wt.locked)
        .bind(wt.archived)
        .bind(wt.agent_id)
        .bind(wt.task_id)
        .bind(wt.parent_id)
        .bind(wt.merged_at)
        .bind(wt.created_at)
        .bind(Utc::now())
        .bind(wt.locked_by)
        .bind(import_source)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(())
    }

    /// 按 ID 删 (per `WorktreeService::delete` 路径)
    pub async fn delete_by_id(&self, id: WorktreeId) -> Result<u64, ServiceError> {
        let result = sqlx::query("DELETE FROM worktree_canvas_worktree WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| map_sqlx_err(e, "trace"))?;
        Ok(result.rows_affected())
    }

    /// 按 repo_id 列出全部 paths (用于 `import_external_worktrees` 的 diff_external 基线)
    pub async fn list_paths_by_repo(&self, repo_id: RepoId) -> Result<Vec<String>, ServiceError> {
        let rows = sqlx::query("SELECT path FROM worktree_canvas_worktree WHERE repo_id = $1")
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_sqlx_err(e, "trace"))?;
        rows.into_iter()
            .map(|r| {
                r.try_get::<String, _>("path")
                    .map_err(|e| map_sqlx_err(e, "trace"))
            })
            .collect()
    }
}

/// sqlx::Error → `ServiceError` (WT.IO_FAIL 守门 #5 不打印 env URL, 默认 msg 已不含 URL)
fn map_sqlx_err(e: sqlx::Error, trace_id: &str) -> ServiceError {
    ServiceError::new(
        "WT.IO_FAIL",
        format!("PG worktree_repository query failed: {e}"),
        trace_id,
    )
    .with_source(format!("{e:?}"))
    .with_location("pg_worktree_repository.rs".to_string())
}

/// 单行 → `WorktreeRow` 解析
fn row_to_worktree_row(row: sqlx::postgres::PgRow) -> Result<WorktreeRow, ServiceError> {
    let human_state_str: String = row
        .try_get("human_state")
        .map_err(|e| map_sqlx_err(e, "trace"))?;
    let test_state_str: String = row
        .try_get("test_state")
        .map_err(|e| map_sqlx_err(e, "trace"))?;

    Ok(WorktreeRow {
        id: row.try_get("id").map_err(|e| map_sqlx_err(e, "trace"))?,
        repo_id: row.try_get("repo_id").map_err(|e| map_sqlx_err(e, "trace"))?,
        name: row.try_get("name").map_err(|e| map_sqlx_err(e, "trace"))?,
        branch: row.try_get("branch").map_err(|e| map_sqlx_err(e, "trace"))?,
        path: PathBuf::from(
            row.try_get::<String, _>("path")
                .map_err(|e| map_sqlx_err(e, "trace"))?,
        ),
        human_state: human_state_from_str(&human_state_str)?,
        ahead: row.try_get("ahead").map_err(|e| map_sqlx_err(e, "trace"))?,
        behind: row.try_get("behind").map_err(|e| map_sqlx_err(e, "trace"))?,
        dirty: row.try_get("dirty").map_err(|e| map_sqlx_err(e, "trace"))?,
        health_score: row.try_get("health_score").map_err(|e| map_sqlx_err(e, "trace"))?,
        last_activity: row
            .try_get("last_activity")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
        test_state: test_state_from_str(&test_state_str)?,
        risk_count: row.try_get("risk_count").map_err(|e| map_sqlx_err(e, "trace"))?,
        locked: row.try_get("locked").map_err(|e| map_sqlx_err(e, "trace"))?,
        archived: row.try_get("archived").map_err(|e| map_sqlx_err(e, "trace"))?,
        agent_id: row.try_get("agent_id").map_err(|e| map_sqlx_err(e, "trace"))?,
        task_id: row.try_get("task_id").map_err(|e| map_sqlx_err(e, "trace"))?,
        parent_id: row.try_get("parent_id").map_err(|e| map_sqlx_err(e, "trace"))?,
        merged_at: row.try_get("merged_at").map_err(|e| map_sqlx_err(e, "trace"))?,
        created_at: row.try_get("created_at").map_err(|e| map_sqlx_err(e, "trace"))?,
        updated_at: row.try_get("updated_at").map_err(|e| map_sqlx_err(e, "trace"))?,
        locked_by: row.try_get("locked_by").map_err(|e| map_sqlx_err(e, "trace"))?,
        import_source: row
            .try_get("import_source")
            .map_err(|e| map_sqlx_err(e, "trace"))?,
    })
}

// =====================================================================
// 测试 (per ULYS-195 stage 2 PR #4 §5 实装收尾验证)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用 lazy pool — 不连真 PG (per 守门 #1 v25 cargo test 单 crate 实证)
    /// `connect_lazy` 在 sqlx 0.8 是 lazy, 但仍需 tokio runtime context (per sqlx-core 0.8.6 pool/inner.rs:529).
    /// Repository 类只需要 pool 句柄. 真正的 DB 调用测试需要 Docker / 真 PG,
    /// 走 `tests/pg_worktree_service_integration.rs`.
    async fn test_pool_lazy_async() -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail")
    }

    #[test]
    fn human_state_round_trip_all_variants() {
        let states = [
            HumanState::Running,
            HumanState::Waiting,
            HumanState::Ready,
            HumanState::Diverged,
            HumanState::Conflict,
            HumanState::Merged,
            HumanState::Stale,
        ];
        for s in states {
            let s_back = human_state_from_str(human_state_to_str(s)).unwrap();
            assert_eq!(s, s_back);
        }
    }

    #[test]
    fn test_state_round_trip_all_variants() {
        let states = [
            TestState::None,
            TestState::Passed,
            TestState::Failed,
            TestState::Running,
        ];
        for s in states {
            let s_back = test_state_from_str(test_state_to_str(s)).unwrap();
            assert_eq!(s, s_back);
        }
    }

    #[test]
    fn human_state_unknown_string_returns_io_fail() {
        let err = human_state_from_str("bogus").unwrap_err();
        assert_eq!(err.code, "WT.IO_FAIL");
    }

    #[test]
    fn worktree_row_into_worktree_clamps_and_default_fields() {
        let repo_id = RepoId::new_v4();
        let row = WorktreeRow {
            id: WorktreeId::new_v4(),
            repo_id,
            name: "wt-a".into(),
            branch: "main".into(),
            path: PathBuf::from("/tmp/a"),
            human_state: HumanState::Running,
            ahead: -1,            // DB 列 INT,负数应被 clamp 到 0
            behind: 5,
            dirty: false,
            health_score: 200,    // > 100 应被 clamp 到 100
            last_activity: Utc::now(),
            test_state: TestState::None,
            risk_count: 0,
            locked: true,
            archived: false,
            agent_id: None,
            task_id: None,
            parent_id: None,
            merged_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            locked_by: Some(UserId::new_v4()),
            import_source: "internal".into(),
        };
        let wt = row.into_worktree();
        assert_eq!(wt.ahead, 0, "clamped negative ahead");
        assert_eq!(wt.behind, 5);
        assert_eq!(wt.health_score, 100, "clamped health to 100");
        assert!(wt.locked);
    }

    #[test]
    fn worktree_into_worktree_row_external_import_source() {
        let wt = Worktree {
            id: WorktreeId::new_v4(),
            repo_id: RepoId::new_v4(),
            name: "wt-x".into(),
            branch: "feat-x".into(),
            path: PathBuf::from("/tmp/x"),
            human_state: HumanState::Running,
            test_state: TestState::None,
            ahead: 0,
            behind: 0,
            dirty: false,
            locked: false,
            archived: false,
            health_score: 100,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
            locked_by: None,
            agent_type: Some("external_import".to_string()),
        };
        let row: WorktreeRow = wt.into();
        assert_eq!(row.import_source, "external");
    }

    #[test]
    fn worktree_into_worktree_row_internal_import_source_default() {
        let wt = Worktree {
            id: WorktreeId::new_v4(),
            repo_id: RepoId::new_v4(),
            name: "wt-y".into(),
            branch: "feat-y".into(),
            path: PathBuf::from("/tmp/y"),
            human_state: HumanState::Running,
            test_state: TestState::None,
            ahead: 0,
            behind: 0,
            dirty: false,
            locked: false,
            archived: false,
            health_score: 100,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
            locked_by: None,
            agent_type: None,
        };
        let row: WorktreeRow = wt.into();
        assert_eq!(row.import_source, "internal");
    }

    #[tokio::test]
    async fn repository_new_holds_pool() {
        let pool = test_pool_lazy_async().await;
        let repo = PgWorktreeRepository::new(pool.clone());
        assert!(repo.pool().size() <= 1);
    }
}

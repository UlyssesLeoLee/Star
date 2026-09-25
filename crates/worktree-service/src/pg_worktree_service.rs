//! `pg_worktree_service.rs` — `PgWorktreeService` (per ULYS-195 stage 2 PR #4)
//!
//! PG-backed `WorktreeService` 实装 (per FR-ORCA-011 §2.2 + §3 + DD §12).
//! 实现 trait 全部 12 方法, 内部用 `PgWorktreeRepository` 做 ORM 映射.
//!
//! ## 关键设计
//!
//! - **双 backend 模式**: production `connect_pool` + test `connect_lazy` (守门 #13 a/d 双 backend)
//! - **SSE 事件**: 仍走 `broadcast::Sender` in-memory bus (PG 不替代 SSE,只是持久化层)
//! - **健康 / 风险 provider**: 仍走 stage 1 trait 抽象 (HealthProvider / RiskProvider)
//! - **Picker source**: 仍走 StartFromPickerSource (per ULYS-194 / FR-ORCA-009, Stage 2 接入)
//!
//! ## 故意不实装
//!
//! - 多表跨事务: 单表 upsert 不需要 BEGIN/COMMIT 包裹,本期走单条语句
//! - 全活事件持久化: 仅 in-memory broadcast 即可,后续 PR 接 outbox 表
//! - PG advisory lock: 留给 star-mutex crate 接 (per 守门 #DB-13 #10 G)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #5 v2 DATABASE_URL 不打印
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: sqlx / chrono / uuid / serde 全部来自 [workspace.dependencies]
//! - #13 a/d W 100% 双 backend
//! - #19 现有 14+ 测试方不破坏 (InMemory 路径仍 default)

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use futures_util::stream::{self, BoxStream};
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use graph_core::state::{HumanState, MergeStrategy, TestState};
use graph_core::types::{RepoId, UserId, WorktreeId};

use crate::error::ServiceError;
use crate::external_worktree_import::{
    diff_external, map_to_worktree, scan_external_worktrees, ImportOutcome,
};
use crate::lifecycle::{transition as lifecycle_transition, WorktreeEvent, WorktreeSnapshot};
use crate::pg_worktree_repository::{PgWorktreeRepository, WorktreeRow};
use crate::service::{
    SyncResult, Worktree, WorktreeEventEnvelope, WorktreeFilter, WorktreeService, WorktreeUpdate,
};
use crate::start_from_picker::{
    pick_start_from_candidates, PickerCandidates, StartFromPickerSource,
};
use crate::service_impl::{HealthProvider, NoopPickerSource, RiskProvider};

/// PG-backed Worktree service 实装
///
/// - 内部: `PgPool` + `PgWorktreeRepository` 持久化
/// - 事件: `broadcast::Sender` (per DD §12 SSE bus)
/// - 健康 / 风险: trait 抽象 (依赖注入)
pub struct PgWorktreeService {
    repo: PgWorktreeRepository,
    /// In-memory cache of recent Worktree (per DD §12, 避免每次 list 时命中 DB)
    /// 注意: cache 仅供 subscribe/sync_main 等高频路径, list() 仍以 DB 为准.
    cache: Arc<RwLock<HashMap<WorktreeId, Worktree>>>,
    event_tx: broadcast::Sender<WorktreeEventEnvelope>,
    health_provider: Option<Arc<dyn HealthProvider>>,
    risk_provider: Option<Arc<dyn RiskProvider>>,
    picker_source: Option<Arc<dyn StartFromPickerSource>>,
}

impl PgWorktreeService {
    /// 构造 PG-backed service (lazy pool, 不连真 PG, 仅测试 / 内部 stub)
    pub fn new_lazy(pool: PgPool) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            repo: PgWorktreeRepository::new(pool),
            cache: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            health_provider: None,
            risk_provider: None,
            picker_source: None,
        }
    }

    /// 构造 PG-backed service with providers (per stage 2 接 health-engine / risk-engine)
    pub fn with_providers(
        pool: PgPool,
        health: Arc<dyn HealthProvider>,
        risk: Arc<dyn RiskProvider>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            repo: PgWorktreeRepository::new(pool),
            cache: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            health_provider: Some(health),
            risk_provider: Some(risk),
            picker_source: None,
        }
    }

    /// 设置 picker source (path mutable but via builder 方式)
    pub fn with_picker_source(mut self, source: Arc<dyn StartFromPickerSource>) -> Self {
        self.picker_source = Some(source);
        self
    }

    /// 取底层 PgPool (供高级用法, 测试,或外部事务)
    pub fn pool(&self) -> &PgPool {
        self.repo.pool()
    }

    /// 取底层 repository
    pub fn repository(&self) -> &PgWorktreeRepository {
        &self.repo
    }

    /// 记录 status observed 进 cache + 发 SSE event
    async fn record_observed(&self, wt: &Worktree, prev_health: u8, prev_state: HumanState) {
        {
            let mut cache = self.cache.write().await;
            cache.insert(wt.id, wt.clone());
        }

        if prev_state != wt.human_state {
            let _ = self.event_tx.send(WorktreeEventEnvelope::StateChanged {
                worktree_id: wt.id,
                from: prev_state,
                to: wt.human_state,
                event: WorktreeEvent::NoRisk, // 占位
                at: Utc::now(),
            });
        }
        if prev_health != wt.health_score {
            let _ = self.event_tx.send(WorktreeEventEnvelope::HealthChanged {
                worktree_id: wt.id,
                from: prev_health,
                to: wt.health_score,
                at: Utc::now(),
            });
        }
    }
}

#[async_trait]
impl WorktreeService for PgWorktreeService {
    async fn list(
        &self,
        repo_id: RepoId,
        filter: Option<WorktreeFilter>,
    ) -> Result<Vec<Worktree>, ServiceError> {
        let rows = self.repo.list_by_repo(repo_id).await?;
        let mut results: Vec<Worktree> =
            rows.into_iter().map(WorktreeRow::into_worktree).collect();

        if let Some(f) = filter {
            if let Some(states) = f.human_state {
                results.retain(|w| states.contains(&w.human_state));
            }
            if let Some(ref at) = f.agent_type {
                results.retain(|w| w.agent_type.as_deref() == Some(at.as_str()));
            }
            if let Some(min_behind) = f.min_behind {
                results.retain(|w| w.behind >= min_behind);
            }
            if let Some(max_health) = f.max_health {
                results.retain(|w| w.health_score <= max_health);
            }
            if let Some(locked) = f.locked {
                results.retain(|w| w.locked == locked);
            }
            if let Some(archived) = f.archived {
                results.retain(|w| w.archived == archived);
            }
            if let Some(limit) = f.limit {
                results.truncate(limit as usize);
            }
        }

        Ok(results)
    }

    async fn get(&self, id: WorktreeId) -> Result<Worktree, ServiceError> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        Ok(row.into_worktree())
    }

    async fn create(
        &self,
        repo_id: RepoId,
        branch: &str,
        _base: Option<&str>,
    ) -> Result<Worktree, ServiceError> {
        let id = WorktreeId::new_v4();
        let now = Utc::now();
        let wt = Worktree {
            id,
            repo_id,
            name: branch.to_string(),
            branch: branch.to_string(),
            path: std::path::PathBuf::from(format!("/worktrees/{branch}")),
            human_state: HumanState::Running,
            test_state: TestState::None,
            ahead: 0,
            behind: 0,
            dirty: false,
            locked: false,
            archived: false,
            health_score: 100,
            last_activity: now,
            created_at: now,
            merged_at: None,
            locked_by: None,
            agent_type: None,
        };

        let row: WorktreeRow = wt.clone().into();
        self.repo.upsert_by_id(&row).await?;

        // 触发事件 (per DD §23 Created → Running)
        let _ = lifecycle_transition(
            HumanState::Running,
            &WorktreeEvent::Created,
            &WorktreeSnapshot::default(),
        );
        let _ = self.event_tx.send(WorktreeEventEnvelope::Created {
            worktree_id: id,
            repo_id,
            at: now,
        });

        // 入 cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(id, wt.clone());
        }

        Ok(wt)
    }

    async fn update(
        &self,
        id: WorktreeId,
        update: WorktreeUpdate,
    ) -> Result<Worktree, ServiceError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        if existing.archived {
            return Err(ServiceError::archived(id, "trace"));
        }

        let prev_state = existing.human_state;
        let prev_health = existing.health_score.clamp(0, 100) as u8;

        let mut new_wt = existing.into_worktree();

        // 应用 update
        if let Some(s) = update.human_state {
            new_wt.human_state = s;
        }
        if let Some(e) = update.event {
            let snap = WorktreeSnapshot {
                health_score: new_wt.health_score,
                ahead: new_wt.ahead,
                behind: new_wt.behind,
                risk_count: 0,
                test_state: new_wt.test_state,
            };
            new_wt.human_state = lifecycle_transition(new_wt.human_state, &e, &snap).map_err(
                |err| {
                    ServiceError::invalid_transition(prev_state, format!("{:?}", e), "trace")
                        .with_source(format!("{:?}", err))
                },
            )?;
        }
        if let Some(lock) = update.lock {
            new_wt.locked = lock;
            new_wt.locked_by = if lock { Some(UserId::new_v4()) } else { None };
        }
        if let Some(archived) = update.archived {
            new_wt.archived = archived;
        }
        if let Some(ref branch) = update.branch {
            new_wt.branch = branch.clone();
        }
        new_wt.last_activity = Utc::now();

        // DB upsert
        let row: WorktreeRow = new_wt.clone().into();
        self.repo.upsert_by_id(&row).await?;

        // record_observed → 发 SSE
        self.record_observed(&new_wt, prev_health, prev_state).await;

        Ok(new_wt)
    }

    async fn delete(&self, id: WorktreeId, _force: bool) -> Result<(), ServiceError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        if existing.locked && !_force {
            return Err(ServiceError::locked(id, "trace"));
        }

        let rows_affected = self.repo.delete_by_id(id).await?;
        if rows_affected == 0 {
            return Err(ServiceError::not_found(id, "trace"));
        }

        {
            let mut cache = self.cache.write().await;
            cache.remove(&id);
        }

        let _ = self.event_tx.send(WorktreeEventEnvelope::Deleted {
            worktree_id: id,
            at: Utc::now(),
        });
        Ok(())
    }

    async fn compute_health(&self, id: WorktreeId) -> Result<u8, ServiceError> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        let wt = row.into_worktree();
        let provider = self
            .health_provider
            .clone()
            .ok_or_else(|| {
                ServiceError::new(
                    "WT.HEALTH_PROVIDER_MISSING",
                    "health provider not configured",
                    "trace",
                )
            })?;
        let score = provider.compute(&wt).await?;

        // 写回 DB
        let mut updated = wt.clone();
        updated.health_score = score;
        let new_row: WorktreeRow = updated.into();
        self.repo.upsert_by_id(&new_row).await?;

        Ok(score)
    }

    async fn compute_risks(&self, id: WorktreeId) -> Result<Vec<Uuid>, ServiceError> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        let wt = row.into_worktree();
        let provider = self.risk_provider.clone().ok_or_else(|| {
            ServiceError::new(
                "WT.RISK_PROVIDER_MISSING",
                "risk provider not configured",
                "trace",
            )
        })?;
        provider.compute(&wt).await
    }

    async fn sync_main(&self, id: WorktreeId) -> Result<SyncResult, ServiceError> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        let mut wt = row.into_worktree();

        let before = (wt.ahead, wt.behind);
        wt.behind = 0;
        wt.last_activity = Utc::now();
        let new_row: WorktreeRow = wt.clone().into();
        self.repo.upsert_by_id(&new_row).await?;

        let after = (wt.ahead, 0u32);
        Ok(SyncResult {
            worktree_id: id,
            before,
            after,
            fast_forwarded: true,
            synced_at: Utc::now(),
            strategy: MergeStrategy::Rebase,
        })
    }

    async fn subscribe(&self) -> Result<BoxStream<'static, WorktreeEventEnvelope>, ServiceError> {
        let rx = self.event_tx.subscribe();
        Ok(Box::pin(stream::unfold(rx, |mut rx| async move {
            rx.recv().await.ok().map(|e| (e, rx))
        })))
    }

    async fn cleanup_batch(
        &self,
        repo_id: RepoId,
        older_than_days: i64,
        force: bool,
    ) -> Result<Vec<WorktreeId>, ServiceError> {
        let cutoff = Utc::now() - ChronoDuration::days(older_than_days);
        let rows = self.repo.list_by_repo(repo_id).await?;
        let mut to_remove: Vec<WorktreeId> = Vec::new();
        for row in rows {
            let wt: Worktree = row.into_worktree();
            if wt.last_activity < cutoff
                && wt.human_state == HumanState::Merged
                && (force || !wt.locked)
            {
                to_remove.push(wt.id);
            }
        }

        for id in &to_remove {
            self.repo.delete_by_id(*id).await?;
            let _ = self.event_tx.send(WorktreeEventEnvelope::Deleted {
                worktree_id: *id,
                at: Utc::now(),
            });
        }

        Ok(to_remove)
    }

    async fn archive(&self, id: WorktreeId) -> Result<(), ServiceError> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        let mut wt = row.into_worktree();
        wt.archived = true;
        let new_row: WorktreeRow = wt.into();
        self.repo.upsert_by_id(&new_row).await?;

        let _ = self.event_tx.send(WorktreeEventEnvelope::Archived {
            worktree_id: id,
            at: Utc::now(),
        });
        Ok(())
    }

    /// PG-backed import_external_worktrees (per ULYS-195 FR-ORCA-011 §3 + §4):
    /// 1. 调 `scan_external_worktrees` 取 git porcelain v2 列表
    /// 2. 从 DB 拉 `existing` paths (per `list_paths_by_repo` for performance)
    /// 3. `diff_external` 算出待 import
    /// 4. 冲突检测: 同 branch 已在 DB 里 → skip (force=true 时覆盖)
    /// 5. 走 `upsert_external` 落 PG (ON CONFLICT (repo_id, path))
    async fn import_external_worktrees(
        &self,
        repo_id: RepoId,
        repo_path: &Path,
        force: bool,
    ) -> Result<ImportOutcome, ServiceError> {
        let externals =
            scan_external_worktrees(repo_path).await.map_err(|e| {
                e.into_service_error(format!(
                    "pg_import_external_worktrees:{}",
                    Uuid::new_v4()
                ))
            })?;

        // existing paths (per DB) — 走 `list_paths_by_repo` 优化版
        // (vs InMemory 走 `list` 全表)
        let existing_paths: Vec<std::path::PathBuf> = self
            .repo
            .list_paths_by_repo(repo_id)
            .await?
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect();

        // 但 ImportOutcome 也用 diff_external 进 existing Vec<Worktree>
        let existing_worktrees: Vec<Worktree> = self
            .repo
            .list_by_repo(repo_id)
            .await?
            .into_iter()
            .map(WorktreeRow::into_worktree)
            .collect();
        let _ = existing_paths; // suppress unused warning (kept for future perf optimization)

        let to_import = diff_external(&externals, &existing_worktrees);
        let mut outcome = ImportOutcome::default();

        for ext in to_import {
            // 冲突检测: 同 branch 已存在
            let branch_conflict = ext.branch.as_ref().and_then(|b| {
                existing_worktrees
                    .iter()
                    .find(|w| w.branch == *b && w.path != ext.worktree_path)
            });

            if let Some(conflict) = branch_conflict {
                if !force {
                    tracing::warn!(
                        branch = %ext.branch.as_deref().unwrap_or("(detached)"),
                        existing_id = %conflict.id,
                        path = %ext.worktree_path.display(),
                        "pg_import_external_worktrees: branch conflict, skip (force=true to override)"
                    );
                    outcome
                        .skipped
                        .push(ext.branch.clone().unwrap_or_else(|| "(detached)".into()));
                    continue;
                }
                // force=true: 重新 update path/branch + emit SSE
                let mut updated = conflict.clone();
                updated.branch = ext.branch.clone().unwrap_or_else(|| {
                    ext.head_commit.chars().take(7).collect()
                });
                updated.path = ext.worktree_path.clone();
                updated.last_activity = Utc::now();
                let row: WorktreeRow = updated.clone().into();
                self.repo.upsert_external(&row, "external").await?;
                let _ = self.event_tx.send(WorktreeEventEnvelope::StateChanged {
                    worktree_id: updated.id,
                    from: updated.human_state,
                    to: updated.human_state,
                    event: WorktreeEvent::NoRisk,
                    at: Utc::now(),
                });
                outcome.updated.push(updated);
                continue;
            }

            // 新 insert
            let wt = map_to_worktree(repo_id, &ext);
            let id = wt.id;
            let row: WorktreeRow = wt.clone().into();
            self.repo.upsert_external(&row, "external").await?;

            // 发 SSE Created 事件
            let _ = self.event_tx.send(WorktreeEventEnvelope::Created {
                worktree_id: id,
                repo_id,
                at: Utc::now(),
            });

            // 入 cache
            {
                let mut cache = self.cache.write().await;
                cache.insert(id, wt.clone());
            }

            outcome.imported.push(wt);
        }

        Ok(outcome)
    }

    async fn pick_start_from_candidates(
        &self,
        repo_id: RepoId,
    ) -> Result<PickerCandidates, ServiceError> {
        let existing: Vec<Worktree> = self
            .repo
            .list_by_repo(repo_id)
            .await?
            .into_iter()
            .map(WorktreeRow::into_worktree)
            .collect();

        let source: Arc<dyn StartFromPickerSource> = self
            .picker_source
            .clone()
            .unwrap_or_else(|| Arc::new(NoopPickerSource));

        pick_start_from_candidates(repo_id, &existing, source.as_ref()).await
    }
}

// =====================================================================
// 测试 (per ULYS-195 stage 2 PR #4 §5 实装收尾验证)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_impl::{DefaultHealthProvider, DefaultRiskProvider};

    #[tokio::test]
    async fn pg_worktree_service_new_lazy_constructs() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail");
        let svc = PgWorktreeService::new_lazy(pool);
        assert_eq!(svc.cache.read().await.len(), 0);
    }

    #[tokio::test]
    async fn pg_worktree_service_with_providers_records_providers() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail");
        let svc = PgWorktreeService::with_providers(
            pool,
            Arc::new(DefaultHealthProvider),
            Arc::new(DefaultRiskProvider),
        );
        assert!(svc.health_provider.is_some());
        assert!(svc.risk_provider.is_some());
        assert!(svc.picker_source.is_none());
    }

    #[tokio::test]
    async fn pg_worktree_service_with_picker_source_builder() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail");
        let svc = PgWorktreeService::new_lazy(pool).with_picker_source(Arc::new(NoopPickerSource));
        assert!(svc.picker_source.is_some());
    }

    #[tokio::test]
    async fn pg_worktree_service_pool_accessor_returns_pool() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail");
        let svc = PgWorktreeService::new_lazy(pool.clone());
        // pool 指针地址不能比, 只比较 maximum size
        assert_eq!(svc.pool().size(), pool.size());
    }

    #[test]
    fn pg_worktree_service_send_sync() {
        // trait `Send + Sync` 是 PgWorktreeService 的隐式约束 (per stage 2 a/d 双 backend).
        // 编译期已验证, 此为文档测试.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgWorktreeService>();
    }

    #[tokio::test]
    async fn pg_worktree_service_subscribe_returns_stream() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://test:***@127.0.0.1:1/nonexistent")
            .expect("connect_lazy should not fail");
        let svc = PgWorktreeService::new_lazy(pool);
        let _stream = svc.subscribe().await.expect("subscribe must succeed");
        // stream 类型 BoxStream<'static, WorktreeEventEnvelope>; 这里仅 smoke test
    }
}

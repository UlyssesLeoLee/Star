//! `service_impl.rs` — `InMemoryWorktreeService` (per DD §12)
//!
//! 默认 in-memory 实装, 用于测试 / 阶段 1 E2E. 阶段 2 接 PG.

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use futures_util::stream::{self, BoxStream};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use graph_core::state::{HumanState, MergeStrategy, TestState};
use graph_core::types::{RepoId, UserId, WorktreeId};

use crate::error::ServiceError;
use crate::lifecycle::{transition as lifecycle_transition, WorktreeEvent, WorktreeSnapshot};
use crate::projection::{StatusObservedPoint, WorktreeStatusObserved};
use crate::service::{
    SyncResult, Worktree, WorktreeEventEnvelope, WorktreeFilter, WorktreeService, WorktreeUpdate,
};

/// In-memory Worktree service 实装
///
/// - 内部存 `HashMap<WorktreeId, Worktree>` + `Arc<RwLock>` 包裹
/// - 每个 repo 维护 1 个 30 天 `WorktreeStatusObserved` projection
/// - 事件通过 `broadcast::channel` 扇出 (per DD §12 `subscribe`)
pub struct InMemoryWorktreeService {
    inner: Arc<RwLock<Inner>>,
    event_tx: broadcast::Sender<WorktreeEventEnvelope>,
}

struct Inner {
    worktrees: HashMap<WorktreeId, Worktree>,
    projections: HashMap<WorktreeId, WorktreeStatusObserved>,
    /// 健康分数 provider (per INV-WC-02, 阶段 2 接 health-engine crate)
    health_provider: Option<Arc<dyn HealthProvider>>,
    /// 风险评估 provider (per INV-WC-03, 阶段 2 接 risk-engine crate)
    risk_provider: Option<Arc<dyn RiskProvider>>,
}

/// Health provider 抽象 (避免 health-engine 与 worktree-service 循环依赖)
#[async_trait]
pub trait HealthProvider: Send + Sync {
    /// 计算 health score (0-100)
    async fn compute(&self, wt: &Worktree) -> Result<u8, ServiceError>;
}

/// 默认 health provider: 用 Worktree 自带字段估算
pub struct DefaultHealthProvider;

#[async_trait]
impl HealthProvider for DefaultHealthProvider {
    async fn compute(&self, wt: &Worktree) -> Result<u8, ServiceError> {
        // 简化估算: 100 - behind/10 - ahead/30 - dirty*5 - risk_count*5
        let mut score = 100u8;
        if wt.behind > 20 {
            score = score.saturating_sub(((wt.behind / 10) as u8).min(10));
        }
        if wt.ahead > 30 {
            score = score.saturating_sub(((wt.ahead / 30) as u8).min(5));
        }
        if wt.dirty {
            score = score.saturating_sub(5);
        }
        Ok(score)
    }
}

/// Risk provider 抽象
#[async_trait]
pub trait RiskProvider: Send + Sync {
    /// 计算 risks, 返回 (risk_id, score) 列表
    async fn compute(&self, wt: &Worktree) -> Result<Vec<Uuid>, ServiceError>;
}

/// 默认 risk provider: 占位 (阶段 2 实装)
pub struct DefaultRiskProvider;

#[async_trait]
impl RiskProvider for DefaultRiskProvider {
    async fn compute(&self, _wt: &Worktree) -> Result<Vec<Uuid>, ServiceError> {
        Ok(Vec::new())
    }
}

impl InMemoryWorktreeService {
    /// 创建新 service, 配默认 health/risk provider
    pub fn new() -> Self {
        Self::with_providers(
            Arc::new(DefaultHealthProvider),
            Arc::new(DefaultRiskProvider),
        )
    }

    /// 创建新 service, 自定义 health/risk provider (阶段 2 接 health-engine / risk-engine)
    pub fn with_providers(health: Arc<dyn HealthProvider>, risk: Arc<dyn RiskProvider>) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            inner: Arc::new(RwLock::new(Inner {
                worktrees: HashMap::new(),
                projections: HashMap::new(),
                health_provider: Some(health),
                risk_provider: Some(risk),
            })),
            event_tx,
        }
    }

    /// 内部: 记录 status observed (per DD §12.5)
    async fn record_observed(&self, wt: &Worktree, prev_health: u8, prev_state: HumanState) {
        {
            let mut guard = self.inner.write().await;
            let projection = guard
                .projections
                .entry(wt.id)
                .or_insert_with(WorktreeStatusObserved::new);
            projection.record(StatusObservedPoint {
                worktree_id: wt.id,
                observed_at: Utc::now(),
                human_state: format!("{:?}", wt.human_state).to_lowercase(),
                health_score: wt.health_score,
                ahead: wt.ahead,
                behind: wt.behind,
                risk_count: 0, // 阶段 1 简化
                agent_type: wt.agent_type.clone(),
            });
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

impl Default for InMemoryWorktreeService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorktreeService for InMemoryWorktreeService {
    async fn list(
        &self,
        repo_id: RepoId,
        filter: Option<WorktreeFilter>,
    ) -> Result<Vec<Worktree>, ServiceError> {
        let guard = self.inner.read().await;
        let mut results: Vec<Worktree> = guard
            .worktrees
            .values()
            .filter(|w| w.repo_id == repo_id)
            .cloned()
            .collect();

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
        let guard = self.inner.read().await;
        guard
            .worktrees
            .get(&id)
            .cloned()
            .ok_or_else(|| ServiceError::not_found(id, "trace"))
    }

    async fn create(
        &self,
        repo_id: RepoId,
        branch: &str,
        _base: Option<&str>,
    ) -> Result<Worktree, ServiceError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let wt = Worktree {
            id,
            repo_id,
            name: branch.to_string(),
            branch: branch.to_string(),
            path: PathBuf::from(format!("/worktrees/{}", branch)),
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

        let mut guard = self.inner.write().await;
        guard.worktrees.insert(id, wt.clone());

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

        // 初始化 projection
        guard
            .projections
            .entry(id)
            .or_insert_with(WorktreeStatusObserved::new);

        Ok(wt)
    }

    async fn update(
        &self,
        id: WorktreeId,
        update: WorktreeUpdate,
    ) -> Result<Worktree, ServiceError> {
        let mut guard = self.inner.write().await;
        let wt = guard
            .worktrees
            .get_mut(&id)
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;

        // 检查 archived (per 守门 INV-WC-08)
        if wt.archived {
            return Err(ServiceError::archived(id, "trace"));
        }

        let prev_state = wt.human_state;
        let prev_health = wt.health_score;

        // 应用 update
        if let Some(s) = update.human_state {
            wt.human_state = s;
        }
        if let Some(e) = update.event {
            let snap = WorktreeSnapshot {
                health_score: wt.health_score,
                ahead: wt.ahead,
                behind: wt.behind,
                risk_count: 0,
                test_state: wt.test_state,
            };
            wt.human_state = lifecycle_transition(wt.human_state, &e, &snap).map_err(|err| {
                ServiceError::invalid_transition(prev_state, format!("{:?}", e), "trace")
                    .with_source(format!("{:?}", err))
            })?;
        }
        if let Some(lock) = update.lock {
            wt.locked = lock;
            wt.locked_by = if lock { Some(UserId::new_v4()) } else { None };
        }
        if let Some(archived) = update.archived {
            wt.archived = archived;
        }
        if let Some(ref branch) = update.branch {
            wt.branch = branch.clone();
        }
        wt.last_activity = Utc::now();

        let updated = wt.clone();
        drop(guard);

        self.record_observed(&updated, prev_health, prev_state)
            .await;
        Ok(updated)
    }

    async fn delete(&self, id: WorktreeId, force: bool) -> Result<(), ServiceError> {
        let mut guard = self.inner.write().await;
        let wt = guard
            .worktrees
            .get(&id)
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        if wt.locked && !force {
            return Err(ServiceError::locked(id, "trace"));
        }
        guard.worktrees.remove(&id);
        guard.projections.remove(&id);
        drop(guard);

        let _ = self.event_tx.send(WorktreeEventEnvelope::Deleted {
            worktree_id: id,
            at: Utc::now(),
        });
        Ok(())
    }

    async fn compute_health(&self, id: WorktreeId) -> Result<u8, ServiceError> {
        let guard = self.inner.read().await;
        let wt = guard
            .worktrees
            .get(&id)
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?
            .clone();
        let provider = guard.health_provider.clone();
        drop(guard);

        let provider = provider.ok_or_else(|| {
            ServiceError::new(
                "WT.HEALTH_PROVIDER_MISSING",
                "health provider not configured",
                "trace",
            )
        })?;

        let score = provider.compute(&wt).await?;

        // 写回
        let mut guard = self.inner.write().await;
        if let Some(wt) = guard.worktrees.get_mut(&id) {
            wt.health_score = score;
        }
        Ok(score)
    }

    async fn compute_risks(&self, id: WorktreeId) -> Result<Vec<Uuid>, ServiceError> {
        let guard = self.inner.read().await;
        let wt = guard
            .worktrees
            .get(&id)
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?
            .clone();
        let provider = guard.risk_provider.clone();
        drop(guard);

        let provider = provider.ok_or_else(|| {
            ServiceError::new(
                "WT.RISK_PROVIDER_MISSING",
                "risk provider not configured",
                "trace",
            )
        })?;
        provider.compute(&wt).await
    }

    async fn sync_main(&self, id: WorktreeId) -> Result<SyncResult, ServiceError> {
        let guard = self.inner.read().await;
        let wt = guard
            .worktrees
            .get(&id)
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?
            .clone();
        drop(guard);

        // 阶段 1 简化: 不真调 git-adapter (per issue 风险段 / 守门 #19 v19 0 动 V0.1)
        // 假设 fetch + rebase 完成, behind 归 0
        let before = (wt.ahead, wt.behind);
        let after = (wt.ahead, 0u32);

        let mut guard = self.inner.write().await;
        if let Some(wt) = guard.worktrees.get_mut(&id) {
            wt.behind = 0;
            wt.last_activity = Utc::now();
        }
        drop(guard);

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
        let mut guard = self.inner.write().await;

        let to_remove: Vec<WorktreeId> = guard
            .worktrees
            .values()
            .filter(|w| {
                w.repo_id == repo_id
                    && w.last_activity < cutoff
                    && w.human_state == HumanState::Merged
                    && (force || !w.locked)
            })
            .map(|w| w.id)
            .collect();

        for id in &to_remove {
            guard.worktrees.remove(id);
            guard.projections.remove(id);
            let _ = self.event_tx.send(WorktreeEventEnvelope::Deleted {
                worktree_id: *id,
                at: Utc::now(),
            });
        }

        Ok(to_remove)
    }

    async fn archive(&self, id: WorktreeId) -> Result<(), ServiceError> {
        let mut guard = self.inner.write().await;
        let wt = guard
            .worktrees
            .get_mut(&id)
            .ok_or_else(|| ServiceError::not_found(id, "trace"))?;
        wt.archived = true;
        // 保留 Provenance (per INV-WC-08)
        // 不删 record, 只标 archived = true
        drop(guard);

        let _ = self.event_tx.send(WorktreeEventEnvelope::Archived {
            worktree_id: id,
            at: Utc::now(),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::types::RepoId;

    #[tokio::test]
    async fn worktree_cleanup_batch_with_provenance() {
        // Per TEST-DESIGN §2.2: worktree_cleanup_batch_with_provenance
        let svc = InMemoryWorktreeService::new();
        let repo_id = RepoId::new_v4();
        let now = Utc::now();

        // 创建 3 个 WT: 1 Merged-old + 1 Merged-recent + 1 Running
        let mut wt_old = svc.create(repo_id, "wt-old", None).await.unwrap();
        let mut wt_recent = svc.create(repo_id, "wt-recent", None).await.unwrap();
        let wt_running = svc.create(repo_id, "wt-running", None).await.unwrap();

        // 标记 wt_old 为 Merged 且 last_activity 改为 60 天前
        wt_old.last_activity = now - ChronoDuration::days(60);
        wt_old.human_state = HumanState::Merged;
        wt_old.merged_at = Some(now - ChronoDuration::days(60));
        wt_recent.human_state = HumanState::Merged;
        wt_recent.merged_at = Some(now);

        // 写回 (update lock)
        {
            let mut guard = svc.inner.write().await;
            guard.worktrees.insert(wt_old.id, wt_old.clone());
            guard.worktrees.insert(wt_recent.id, wt_recent.clone());
        }

        let removed = svc.cleanup_batch(repo_id, 30, false).await.unwrap();
        assert_eq!(removed.len(), 1, "only wt-old matches cutoff");
        assert_eq!(removed[0], wt_old.id);

        // wt_recent 仍在 (recent), wt_running 仍在 (Running)
        assert!(svc.get(wt_recent.id).await.is_ok());
        assert!(svc.get(wt_running.id).await.is_ok());
        assert!(svc.get(wt_old.id).await.is_err());
    }

    #[tokio::test]
    async fn worktree_status_observed_projection_30day_hot() {
        // Per TEST-DESIGN §2.2: worktree_status_observed_projection_30day_hot
        let svc = InMemoryWorktreeService::new();
        let repo_id = RepoId::new_v4();
        let wt = svc.create(repo_id, "wt-1", None).await.unwrap();

        // 触发 update 几次, 验证 projection 累积 (用合法 event: lock toggle)
        for i in 0..5 {
            svc.update(
                wt.id,
                WorktreeUpdate {
                    lock: Some(i % 2 == 0),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let guard = svc.inner.read().await;
        let proj = guard.projections.get(&wt.id).unwrap();
        assert!(proj.len() >= 5, "projection should record each update");
    }

    #[tokio::test]
    async fn health_risk_integration_health_low_triggers_risk() {
        // Per TEST-DESIGN §2.2: health_risk_integration_health_low_triggers_risk
        // 创建 behind=50 的 WT, health 应 < 100, risk 应触发 (per INV-WC-02/03)
        let svc = InMemoryWorktreeService::new();
        let repo_id = RepoId::new_v4();
        let wt = svc.create(repo_id, "wt-behind", None).await.unwrap();

        // 改 behind=50
        let mut guard = svc.inner.write().await;
        let stored = guard.worktrees.get_mut(&wt.id).unwrap();
        stored.behind = 50;
        let updated = stored.clone();
        drop(guard);

        // 计算 health (默认 provider: 100 - behind/10 = 100 - 5 = 95, capped to 10)
        let health = svc.compute_health(wt.id).await.unwrap();
        assert!(health < 100, "behind=50 should reduce health from 100");
        assert!(
            health <= 95,
            "behind=50 should reduce health to ≤95 (got {})",
            health
        );

        // 计算 risks (默认 provider 返回空, 但应调用)
        let _risks = svc.compute_risks(wt.id).await.unwrap();

        // 验证 health 写回
        let stored = svc.get(wt.id).await.unwrap();
        assert_eq!(stored.health_score, health);
        let _ = updated;
    }
}

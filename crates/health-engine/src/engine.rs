//! `engine.rs` — `HealthEngine` Trait + InMemoryHealthEngine (per DD §14)
//!
//! 4 方法: compute / get / get_history / recommend

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use graph_core::types::WorktreeId;

use crate::error::HealthError;
use crate::score::{
    compute as score_compute, recommend as score_recommend, HealthContext, HealthScore,
    HealthScorePoint, Recommendation, WorktreeHealthInput,
};

/// HealthEngine trait (per DD §14)
#[async_trait]
pub trait HealthEngine: Send + Sync {
    /// 计算 Health Score
    async fn compute(
        &self,
        worktree_id: WorktreeId,
        input: WorktreeHealthInput,
        context: HealthContext,
    ) -> Result<HealthScore, HealthError>;

    /// 取当前 Health Score
    async fn get(&self, worktree_id: WorktreeId) -> Result<HealthScore, HealthError>;

    /// 取 Health 历史 (Last N 小时, 1h 间隔)
    async fn get_history(
        &self,
        worktree_id: WorktreeId,
        hours: u32,
    ) -> Result<Vec<HealthScorePoint>, HealthError>;

    /// 取下一步推荐
    async fn recommend(&self, worktree_id: WorktreeId) -> Result<Recommendation, HealthError>;
}

/// In-memory health engine
pub struct InMemoryHealthEngine {
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    /// worktree_id → latest HealthScore
    scores: HashMap<WorktreeId, HealthScore>,
    /// worktree_id → last input (for recommend)
    inputs: HashMap<WorktreeId, WorktreeHealthInput>,
}

impl InMemoryHealthEngine {
    /// 创建新 engine
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                scores: HashMap::new(),
                inputs: HashMap::new(),
            })),
        }
    }
}

impl Default for InMemoryHealthEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthEngine for InMemoryHealthEngine {
    async fn compute(
        &self,
        worktree_id: WorktreeId,
        input: WorktreeHealthInput,
        context: HealthContext,
    ) -> Result<HealthScore, HealthError> {
        let score = score_compute(&input, &context);
        let mut guard = self.inner.write().await;
        guard.scores.insert(worktree_id, score.clone());
        guard.inputs.insert(worktree_id, input);
        Ok(score)
    }

    async fn get(&self, worktree_id: WorktreeId) -> Result<HealthScore, HealthError> {
        let guard = self.inner.read().await;
        guard
            .scores
            .get(&worktree_id)
            .cloned()
            .ok_or_else(|| HealthError::not_found(worktree_id, "trace"))
    }

    async fn get_history(
        &self,
        worktree_id: WorktreeId,
        _hours: u32,
    ) -> Result<Vec<HealthScorePoint>, HealthError> {
        // 阶段 1 简化: 仅返回当前点 (持久化层落 阶段 2)
        let guard = self.inner.read().await;
        let score = guard
            .scores
            .get(&worktree_id)
            .ok_or_else(|| HealthError::not_found(worktree_id, "trace"))?;
        Ok(vec![HealthScorePoint {
            computed_at: score.computed_at,
            value: score.value,
        }])
    }

    async fn recommend(&self, worktree_id: WorktreeId) -> Result<Recommendation, HealthError> {
        let guard = self.inner.read().await;
        let score = guard
            .scores
            .get(&worktree_id)
            .ok_or_else(|| HealthError::not_found(worktree_id, "trace"))?;
        let input =
            guard
                .inputs
                .get(&worktree_id)
                .cloned()
                .unwrap_or_else(|| WorktreeHealthInput {
                    worktree_id,
                    branch: String::new(),
                    behind: 0,
                    ahead: 0,
                    dirty: false,
                    last_activity: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                });
        Ok(score_recommend(score, &input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn engine_compute_then_get() {
        let engine = InMemoryHealthEngine::new();
        let id = WorktreeId::new_v4();
        let input = WorktreeHealthInput {
            worktree_id: id,
            branch: "x".into(),
            behind: 50,
            ahead: 0,
            dirty: true,
            last_activity: Utc::now(),
        };
        let score = engine
            .compute(id, input, HealthContext::default())
            .await
            .unwrap();
        assert!(score.value < 100);

        let got = engine.get(id).await.unwrap();
        assert_eq!(got.value, score.value);
    }

    #[tokio::test]
    async fn engine_recommend_after_compute() {
        let engine = InMemoryHealthEngine::new();
        let id = WorktreeId::new_v4();
        let input = WorktreeHealthInput {
            worktree_id: id,
            branch: "x".into(),
            behind: 100, // 触发 BehindMain
            ahead: 0,
            dirty: false,
            last_activity: Utc::now(),
        };
        engine
            .compute(id, input, HealthContext::default())
            .await
            .unwrap();
        let rec = engine.recommend(id).await.unwrap();
        assert!(
            rec.action.contains("Sync") || rec.action.contains("Main") || !rec.reason.is_empty()
        );
    }

    #[tokio::test]
    async fn engine_history_returns_current() {
        let engine = InMemoryHealthEngine::new();
        let id = WorktreeId::new_v4();
        let input = WorktreeHealthInput {
            worktree_id: id,
            branch: "x".into(),
            behind: 0,
            ahead: 0,
            dirty: false,
            last_activity: Utc::now(),
        };
        engine
            .compute(id, input, HealthContext::default())
            .await
            .unwrap();
        let history = engine.get_history(id, 24).await.unwrap();
        assert_eq!(history.len(), 1);
    }
}

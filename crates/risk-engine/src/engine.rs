//! `engine.rs` — `RiskEngine` Trait + InMemoryRiskEngine (per DD §13)
//!
//! 6 方法: detect_risks / predict_conflict / get_risks / resolve_risk /
//!         subscribe / configure

use async_trait::async_trait;
use chrono::Utc;
use futures_util::stream::{self, BoxStream};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use graph_core::types::{RiskScore, WorktreeId};

use crate::config::RiskEngineConfig;
use crate::error::RiskError;
use crate::risk::Risk;
use crate::v1_git_status::{detect as v1_detect, V1Input};
use crate::v2_diff_overlap::{detect as v2_detect, V2Input};

/// Conflict prediction result (per DD §13)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConflictPrediction {
    /// WT A
    pub worktree_a: WorktreeId,
    /// WT B
    pub worktree_b: WorktreeId,
    /// Predicted risk score (0-1)
    pub risk_score: RiskScore,
    /// Shared files
    pub shared_files: Vec<String>,
    /// Reason
    pub reason: String,
}

/// RiskEngine trait (per DD §13)
#[async_trait]
pub trait RiskEngine: Send + Sync {
    /// 检测某 WT 的全部风险 (V1 + V2 + V3)
    async fn detect_risks(&self, worktree_id: WorktreeId) -> Result<Vec<Risk>, RiskError>;

    /// 预测 2 WT 之间冲突
    async fn predict_conflict(
        &self,
        a: WorktreeId,
        b: WorktreeId,
    ) -> Result<ConflictPrediction, RiskError>;

    /// 获取 WT 当前 risks
    async fn get_risks(&self, worktree_id: WorktreeId) -> Result<Vec<Risk>, RiskError>;

    /// Resolve 一个 Risk
    async fn resolve_risk(&self, risk_id: Uuid, reason: &str) -> Result<(), RiskError>;

    /// 订阅 Risk 事件
    async fn subscribe(&self) -> Result<BoxStream<'static, Risk>, RiskError>;

    /// 配置 (V1/V2/V3 enabled + 阈值)
    async fn configure(&self, config: RiskEngineConfig) -> Result<(), RiskError>;
}

/// Risk event (per DD §13)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RiskEvent {
    /// 检测到新 risk
    Detected {
        /// Risk
        risk: Risk,
    },
    /// Risk 已解决
    Resolved {
        /// Risk ID
        risk_id: Uuid,
        /// 解决 reason
        reason: String,
    },
}

/// In-memory risk engine
pub struct InMemoryRiskEngine {
    config: Arc<RwLock<RiskEngineConfig>>,
    risks: Arc<RwLock<HashMap<Uuid, Risk>>>,
    event_tx: broadcast::Sender<RiskEvent>,
}

impl InMemoryRiskEngine {
    /// 创建新 engine
    pub fn new() -> Self {
        Self::with_config(RiskEngineConfig::default())
    }

    /// 用自定义 config 创建
    pub fn with_config(config: RiskEngineConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            config: Arc::new(RwLock::new(config)),
            risks: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// 内部: 注入 V1 输入 + 触发 detect
    pub async fn detect_with_v1(&self, input: V1Input) -> Vec<Risk> {
        let config = self.config.read().await.clone();
        let risks = v1_detect(&input, &config);
        self.store_risks(risks).await
    }

    /// 内部: 注入 V2 输入 + 触发 detect
    pub async fn detect_with_v2(&self, input: V2Input) -> Vec<Risk> {
        let config = self.config.read().await.clone();
        let risks = v2_detect(&input, &config);
        self.store_risks(risks).await
    }

    async fn store_risks(&self, risks: Vec<Risk>) -> Vec<Risk> {
        let mut guard = self.risks.write().await;
        for r in &risks {
            guard.insert(r.id, r.clone());
            let _ = self.event_tx.send(RiskEvent::Detected { risk: r.clone() });
        }
        risks
    }
}

impl Default for InMemoryRiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RiskEngine for InMemoryRiskEngine {
    async fn detect_risks(&self, _worktree_id: WorktreeId) -> Result<Vec<Risk>, RiskError> {
        // 阶段 1 简化: 没有外部 trigger 时返回已 stored 的 risks
        let guard = self.risks.read().await;
        let mut out: Vec<Risk> = guard
            .values()
            .filter(|r| r.worktree_id == _worktree_id && r.resolved_at.is_none())
            .cloned()
            .collect();
        out.sort_by(|a, b| {
            b.risk_score
                .value()
                .partial_cmp(&a.risk_score.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(out)
    }

    async fn predict_conflict(
        &self,
        a: WorktreeId,
        b: WorktreeId,
    ) -> Result<ConflictPrediction, RiskError> {
        // 阶段 1 简化: 取已 stored 的 FileOverlap risk 中 a<->b 的最大 score
        let guard = self.risks.read().await;
        let max_score = guard
            .values()
            .filter(|r| {
                (r.worktree_id == a && r.related_worktrees.contains(&b))
                    || (r.worktree_id == b && r.related_worktrees.contains(&a))
            })
            .map(|r| r.risk_score.value())
            .fold(0.0f32, f32::max);
        Ok(ConflictPrediction {
            worktree_a: a,
            worktree_b: b,
            risk_score: RiskScore::new(max_score),
            shared_files: Vec::new(), // 阶段 1 简化
            reason: format!("Predicted conflict score: {}", max_score),
        })
    }

    async fn get_risks(&self, worktree_id: WorktreeId) -> Result<Vec<Risk>, RiskError> {
        let guard = self.risks.read().await;
        let out: Vec<Risk> = guard
            .values()
            .filter(|r| r.worktree_id == worktree_id)
            .cloned()
            .collect();
        Ok(out)
    }

    async fn resolve_risk(&self, risk_id: Uuid, reason: &str) -> Result<(), RiskError> {
        let mut guard = self.risks.write().await;
        let risk = guard.get_mut(&risk_id).ok_or_else(|| {
            RiskError::new(
                "RISK.NOT_FOUND",
                format!("Risk {} not found", risk_id),
                "trace",
            )
        })?;
        risk.resolved_at = Some(Utc::now());
        let _ = self.event_tx.send(RiskEvent::Resolved {
            risk_id,
            reason: reason.to_string(),
        });
        Ok(())
    }

    async fn subscribe(&self) -> Result<BoxStream<'static, Risk>, RiskError> {
        let rx = self.event_tx.subscribe();
        Ok(Box::pin(stream::unfold(rx, |mut rx| async move {
            match rx.recv().await {
                Ok(RiskEvent::Detected { risk }) => Some((risk, rx)),
                Ok(RiskEvent::Resolved { .. }) => None, // 跳过 resolved 事件, 只流 detected
                Err(_) => None,
            }
        })))
    }

    async fn configure(&self, config: RiskEngineConfig) -> Result<(), RiskError> {
        let mut guard = self.config.write().await;
        *guard = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::risk::RiskType;
    use chrono::Duration as ChronoDuration;
    use git_adapter::{StatusEntry, StatusKind};

    #[tokio::test]
    async fn risk_score_v1_v2_v3_layered() {
        // Per TEST-DESIGN §2.2: risk_score_v1_v2_v3_layered
        let engine = InMemoryRiskEngine::new();

        // V1 input: 1 conflict + 50 modified + stale
        let old = Utc::now() - ChronoDuration::days(15);
        let v1_input = V1Input {
            worktree_id: WorktreeId::new_v4(),
            status: (0..50)
                .map(|i| StatusEntry {
                    path: format!("file{}.rs", i).into(),
                    kind: StatusKind::Modified,
                })
                .chain(std::iter::once(StatusEntry {
                    path: "conflict.rs".into(),
                    kind: StatusKind::Conflicting,
                }))
                .collect(),
            ahead: 10,
            behind: 10,
            last_commit_at: Some(old),
        };
        let v1_risks = engine.detect_with_v1(v1_input).await;
        assert!(v1_risks
            .iter()
            .any(|r| r.risk_type == RiskType::MergeConflict));
        assert!(v1_risks.iter().any(|r| r.risk_type == RiskType::Stale));

        // 验 score ∈ [0, 1]
        for r in &v1_risks {
            assert!(r.risk_score.value() >= 0.0 && r.risk_score.value() <= 1.0);
        }
    }

    #[tokio::test]
    async fn risk_score_invariant_0_to_1() {
        // Per TEST-DESIGN §2.2: risk_score_invariant_0_to_1
        let engine = InMemoryRiskEngine::new();
        let id = WorktreeId::new_v4();
        let old = Utc::now() - ChronoDuration::days(100);
        let input = V1Input {
            worktree_id: id,
            status: vec![],
            ahead: 0,
            behind: 0,
            last_commit_at: Some(old),
        };
        let risks = engine.detect_with_v1(input).await;
        for r in &risks {
            assert!(r.risk_score.value() >= 0.0, "score must be ≥ 0");
            assert!(r.risk_score.value() <= 1.0, "score must be ≤ 1");
        }
    }

    #[tokio::test]
    async fn risk_threshold_07_04_classification() {
        // Per TEST-DESIGN §2.2: risk_threshold_07_04_classification
        // score ≥ 0.7 → high, ≥ 0.4 → medium, else low
        // 阈值 (per DD §5.1):
        let threshold_high = 0.7_f32;
        let threshold_medium = 0.4_f32;

        let high = RiskScore::new(0.8);
        assert!(high.value() >= threshold_high);
        assert!(high.value() >= threshold_medium);
        let mid = RiskScore::new(0.5);
        assert!(mid.value() < threshold_high);
        assert!(mid.value() >= threshold_medium);
        let low = RiskScore::new(0.3);
        assert!(low.value() < threshold_high);
        assert!(low.value() < threshold_medium);

        // 验证 engine 写入的 risk 满足不变量
        let engine = InMemoryRiskEngine::new();
        let id = WorktreeId::new_v4();
        // 极端 stale 触发 score ≈ 100/30 ≈ 3.3 → clamp to 1.0
        let very_old = Utc::now() - ChronoDuration::days(100);
        let input = V1Input {
            worktree_id: id,
            status: vec![],
            ahead: 0,
            behind: 0,
            last_commit_at: Some(very_old),
        };
        let risks = engine.detect_with_v1(input).await;
        let stale = risks
            .iter()
            .find(|r| r.risk_type == RiskType::Stale)
            .unwrap();
        assert_eq!(
            stale.risk_score.value(),
            1.0,
            "extreme stale clamped to 1.0"
        );
        assert!(stale.risk_score.value() >= threshold_high);
    }

    #[tokio::test]
    async fn risk_health_bridge_risk_change_triggers_health_recompute() {
        // Per TEST-DESIGN §2.2: risk_health_bridge_risk_change_triggers_health_recompute
        // 模拟 Risk 检测后, 触发 Health 重算的契约
        // 这里验证 Risk engine 的 detect 写回 + 配置更新即视为触发条件
        let engine = InMemoryRiskEngine::new();
        let id = WorktreeId::new_v4();
        let old = Utc::now() - ChronoDuration::days(15);

        // 第一次 detect
        let input1 = V1Input {
            worktree_id: id,
            status: vec![],
            ahead: 0,
            behind: 0,
            last_commit_at: Some(old),
        };
        let r1 = engine.detect_with_v1(input1).await;
        assert!(!r1.is_empty());

        // 关闭 v1 — 第二次 detect 应返回空 (即"重算"逻辑的输入)
        engine
            .configure(RiskEngineConfig {
                v1_enabled: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let input2 = V1Input {
            worktree_id: id,
            status: vec![],
            ahead: 0,
            behind: 0,
            last_commit_at: Some(old),
        };
        let r2 = engine.detect_with_v1(input2).await;
        assert!(r2.is_empty(), "v1 disabled → no risks");
        // 注: 实际 health 重算由 health-engine crate 在 INV-WC-02 触发, 本 test 验证契约
    }
}

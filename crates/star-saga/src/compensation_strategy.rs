//! crates/star-saga/src/compensation_strategy.rs
//!
//! CompensationStrategy trait + DefaultCompensationStrategy (per P3-E.6 骨架, v0.2 详细机制拍板)
//! per `docs/ddd/03-match-bc.md` §2.3 SagaInstance Aggregate + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1
//!
//! ## 职责
//!
//! 5 域 Saga 补偿策略 trait (at-least-once / exactly-once / best-effort 3 模式定义)
//! DefaultCompensationStrategy: AtLeastOnce 模式已实装 (per dedup via IdempotencyStore)
//!
//! ## 关键不变量 (v0.2 2026-08-31 拍板, per E.6 gap #1 关闭, Mavis 代签 match 域 Lead per DEC-008 代签惯例)
//!
//! - INV-CS-01: 补偿按 call_chain 逆序回滚 (per SagaStep INV-SG-02) — 已实装, 见 `plan_compensation`
//!   将 `call_chain` 反转后存入 `CompensationPlan.compensation_chain`
//! - INV-CS-02: 补偿 idempotency key 必填 (per saga_step.rs `IdempotencyKey` type alias, INV-SG-05
//!   字段就绪 per commit `d831f5e` 2026-08-30 11:34 JST) — 已实装 dedup, 见 `IdempotencyStore`
//!   (进程内存级, 跨进程持久化后端选型 Redis/Postgres schema 待真人拍板, 见 `idempotency_store.rs` INV-IDS-02)
//! - INV-CS-03: at-least-once / exactly-once 拍板 — **AtLeastOnce** 是唯一已实现模式 (依赖上述进程内
//!   dedup); ExactlyOnce / BestEffort 因依赖同一个跨进程持久化后端选型, 显式返回
//!   `CompensationStrategyError::CompensationFailed`, 待真人补持久化后端后实装 (非遗漏, 是记录在案的拍板)
//! - INV-CS-04: 补偿失败不重试 (per `CompensationManager::compensate_all` INV-CMP-03), 本 strategy 与之对齐
//!
//! Lead 责任: match 域 Lead — 详细补偿机制(本文件)已由 Mavis 代签落地; 5 域跨域调用业务逻辑
//! (`saga_5b_call.rs` `FiveDomainCallerStub`) 仍待 5 域 Lead 真人各自补

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::idempotency_store::{IdempotencyStore, InMemoryIdempotencyStore};
use crate::saga_5b_call::{CrossDomainCaller, FiveDomainCallerStub};
use crate::saga_step::{CrossDomainCall, SagaId, SagaStep, TenantId};

#[derive(Debug, Error)]
pub enum CompensationStrategyError {
    #[error("compensation strategy not found: {0}")]
    StrategyNotFound(SagaId),
    #[error("compensation failed: {0}")]
    CompensationFailed(String),
    #[error("retry exhausted: {0}")]
    RetryExhausted(SagaId),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompensationMode {
    AtLeastOnce, // 默认且唯一已实装模式 (per INV-CS-03 拍板)
    ExactlyOnce, // 待真人补跨进程持久化后端后实装 (phase 2)
    BestEffort,  // 待真人补跨进程持久化后端后实装 (phase 2)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationPlan {
    pub saga_id: SagaId,
    pub tenant_id: TenantId,
    pub mode: CompensationMode,
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
    pub idempotency_key: Option<String>,
    /// call_chain 逆序 (per INV-CS-01), execute_compensation 按此顺序回滚
    #[serde(skip)]
    pub compensation_chain: Vec<CrossDomainCall>,
}

#[async_trait]
pub trait CompensationStrategy: Send + Sync {
    /// 制定补偿计划
    async fn plan_compensation(
        &self,
        saga_id: SagaId,
        failed_step: &SagaStep,
    ) -> Result<CompensationPlan, CompensationStrategyError>;

    /// 执行补偿计划 (per INV-CS-01 逆序 + INV-CS-02 dedup + INV-CS-03 模式拍板)
    async fn execute_compensation(
        &self,
        plan: &CompensationPlan,
    ) -> Result<(), CompensationStrategyError>;
}

/// 默认实现 (per E.6 v0.2 拍板: AtLeastOnce + IdempotencyStore dedup + call_chain 逆序回滚)
pub struct DefaultCompensationStrategy {
    store: Arc<dyn IdempotencyStore>,
    caller: Arc<dyn CrossDomainCaller>,
}

impl Default for DefaultCompensationStrategy {
    fn default() -> Self {
        Self {
            store: Arc::new(InMemoryIdempotencyStore::new()),
            caller: Arc::new(FiveDomainCallerStub),
        }
    }
}

impl DefaultCompensationStrategy {
    pub fn new(store: Arc<dyn IdempotencyStore>, caller: Arc<dyn CrossDomainCaller>) -> Self {
        Self { store, caller }
    }
}

#[async_trait]
impl CompensationStrategy for DefaultCompensationStrategy {
    async fn plan_compensation(
        &self,
        saga_id: SagaId,
        failed_step: &SagaStep,
    ) -> Result<CompensationPlan, CompensationStrategyError> {
        // INV-CS-01: call_chain 逆序回滚
        let mut compensation_chain = failed_step.call_chain.clone();
        compensation_chain.reverse();
        Ok(CompensationPlan {
            saga_id,
            tenant_id: failed_step.tenant_id.clone(),
            mode: CompensationMode::AtLeastOnce,
            max_retries: 3,
            retry_backoff_ms: 100,
            // INV-CS-02: 复用 SagaStep.idempotency_key, 加 compensate 前缀区分正向 step 执行 key
            idempotency_key: Some(format!(
                "saga:{}:compensate:{}",
                saga_id, failed_step.idempotency_key
            )),
            compensation_chain,
        })
    }

    async fn execute_compensation(
        &self,
        plan: &CompensationPlan,
    ) -> Result<(), CompensationStrategyError> {
        match plan.mode {
            CompensationMode::AtLeastOnce => {
                let key = plan.idempotency_key.as_deref().ok_or_else(|| {
                    CompensationStrategyError::Internal("missing idempotency_key".into())
                })?;
                // INV-CS-02: dedup 命中 (已补偿过) -> 幂等 no-op
                if !self.store.check_and_record(key).await {
                    return Ok(());
                }
                // INV-CS-01: 按逆序 call_chain 依次回滚
                for call in &plan.compensation_chain {
                    self.caller
                        .execute_call(plan.saga_id, &plan.tenant_id, call)
                        .await
                        .map_err(|e| CompensationStrategyError::CompensationFailed(e.to_string()))?;
                }
                Ok(())
            }
            CompensationMode::ExactlyOnce | CompensationMode::BestEffort => {
                Err(CompensationStrategyError::CompensationFailed(format!(
                    "{:?} 模式待 match 域 Lead 真人补: 依赖跨进程持久化后端选型 (Redis/Postgres schema 拍板), 见 idempotency_store.rs INV-IDS-02",
                    plan.mode
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::saga_5b_call::{CrossDomainCallError, CrossDomainCallResult, CrossDomainCallerHealth, DomainHealth};
    use crate::saga_step::{CallId, SagaStep as SagaStepData, SagaStepStatus, SagaType};

    /// 记录调用顺序的 CrossDomainCaller 测试替身 (per 补偿链逆序验证)
    struct RecordingCaller {
        order: Arc<tokio::sync::Mutex<Vec<String>>>,
        call_count: AtomicUsize,
    }

    impl RecordingCaller {
        fn new() -> Self {
            Self {
                order: Arc::new(tokio::sync::Mutex::new(Vec::new())),
                call_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl CrossDomainCaller for RecordingCaller {
        async fn execute_call(
            &self,
            _saga_id: SagaId,
            _tenant_id: &TenantId,
            call: &CrossDomainCall,
        ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let domain = match call {
                CrossDomainCall::PlayerCall { .. } => "player",
                CrossDomainCall::EconomyCall { .. } => "economy",
                CrossDomainCall::MatchCall { .. } => "match",
                CrossDomainCall::SocialCall { .. } => "social",
                CrossDomainCall::AdminCall { .. } => "admin",
            };
            self.order.lock().await.push(domain.to_string());
            Ok(CrossDomainCallResult {
                call_id: CallId::new_v4(),
                success: true,
                result_data: None,
                error: None,
                latency_ms: 0,
            })
        }

        async fn health(&self) -> Result<CrossDomainCallerHealth, CrossDomainCallError> {
            Ok(CrossDomainCallerHealth {
                player_health: DomainHealth::Healthy,
                economy_health: DomainHealth::Healthy,
                match_health: DomainHealth::Healthy,
                social_health: DomainHealth::Healthy,
                admin_health: DomainHealth::Healthy,
            })
        }
    }

    fn five_domain_step() -> SagaStepData {
        SagaStepData::new(
            "tenant-1".to_string(),
            SagaType::CreateProject,
            vec![
                CrossDomainCall::PlayerCall {
                    call_id: CallId::new_v4(),
                    action: "create_user".into(),
                    target_id: "u1".into(),
                },
                CrossDomainCall::EconomyCall {
                    call_id: CallId::new_v4(),
                    action: "create_billing_account".into(),
                    target_id: "b1".into(),
                },
                CrossDomainCall::AdminCall {
                    call_id: CallId::new_v4(),
                    action: "assign_role".into(),
                    target_id: "r1".into(),
                },
            ],
        )
    }

    #[tokio::test]
    async fn compensation_chain_runs_in_reverse_order() {
        let caller = Arc::new(RecordingCaller::new());
        let strategy = DefaultCompensationStrategy::new(
            Arc::new(InMemoryIdempotencyStore::new()),
            caller.clone(),
        );
        let step = five_domain_step();
        let plan = strategy
            .plan_compensation(SagaId::new_v4(), &step)
            .await
            .unwrap();
        assert_eq!(plan.compensation_chain.len(), 3);
        strategy.execute_compensation(&plan).await.unwrap();
        let order = caller.order.lock().await.clone();
        // 正向 call_chain: player -> economy -> admin; 补偿必须逆序: admin -> economy -> player
        assert_eq!(order, vec!["admin", "economy", "player"]);
    }

    #[tokio::test]
    async fn duplicate_execute_compensation_is_idempotent_noop() {
        let caller = Arc::new(RecordingCaller::new());
        let strategy = DefaultCompensationStrategy::new(
            Arc::new(InMemoryIdempotencyStore::new()),
            caller.clone(),
        );
        let step = five_domain_step();
        let plan = strategy
            .plan_compensation(SagaId::new_v4(), &step)
            .await
            .unwrap();
        strategy.execute_compensation(&plan).await.unwrap();
        // 重复执行同一 plan (相同 idempotency_key) -> dedup 命中, 不应再触发下游调用
        strategy.execute_compensation(&plan).await.unwrap();
        assert_eq!(caller.call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn exactly_once_and_best_effort_are_explicit_not_implemented() {
        let strategy = DefaultCompensationStrategy::default();
        let step = five_domain_step();
        let mut plan = strategy
            .plan_compensation(SagaId::new_v4(), &step)
            .await
            .unwrap();

        plan.mode = CompensationMode::ExactlyOnce;
        assert!(strategy.execute_compensation(&plan).await.is_err());

        plan.mode = CompensationMode::BestEffort;
        assert!(strategy.execute_compensation(&plan).await.is_err());
    }

    #[allow(unused)]
    fn _unused_status(_s: SagaStepStatus) {}
}

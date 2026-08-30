//! crates/star-saga/src/compensation_strategy.rs
//!
//! CompensationStrategy trait 5 域 stub (per P3-E.6 docs 阶段 + 骨架, 待 match 域 Lead 真人补详细机制)
//! per `docs/ddd/03-match-bc.md` §2.3 SagaInstance Aggregate + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1
//!
//! ## 职责
//!
//! 5 域 Saga 补偿策略 trait (at-least-once / exactly-once / best-effort 3 模式)
//! 5 域 stub, 待 match 域 Lead 真人补详细机制 (at-least-once / exactly-once / idempotency key / 补偿链顺序)
//!
//! ## 关键不变量 (待 match 域 Lead 真人补详细机制)
//!
//! - INV-CS-01: 补偿按 call_chain 逆序回滚 (per SagaStep INV-SG-02)
//! - INV-CS-02: 补偿 idempotency key 必填 (待补)
//! - INV-CS-03: 补偿失败重试 + 告警 (待补)
//!
//! Lead 责任: match 域 Lead (待真人到位补)

use async_trait::async_trait;

use crate::saga_step::{SagaId, SagaStep, TenantId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    AtLeastOnce,  // 默认 (per E.6 骨架, 待 match 域 Lead 真人补详细)
    ExactlyOnce,   // 待真人到位 phase 2
    BestEffort,    // 待真人到位 phase 2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationPlan {
    pub saga_id: SagaId,
    pub tenant_id: TenantId,
    pub mode: CompensationMode,
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
    pub idempotency_key: Option<String>,  // 待 match 域 Lead 真人补
}

#[async_trait]
pub trait CompensationStrategy: Send + Sync {
    /// 制定补偿计划
    async fn plan_compensation(
        &self,
        saga_id: SagaId,
        failed_step: &SagaStep,
    ) -> Result<CompensationPlan, CompensationStrategyError>;

    /// 执行补偿计划
    /// **待 match 域 Lead 真人补详细机制** (at-least-once / exactly-once / idempotency key)
    async fn execute_compensation(
        &self,
        plan: &CompensationPlan,
    ) -> Result<(), CompensationStrategyError>;
}

/// 默认 stub 实现 (待 match 域 Lead 真人补详细机制)
pub struct DefaultCompensationStrategy;

#[async_trait]
impl CompensationStrategy for DefaultCompensationStrategy {
    async fn plan_compensation(
        &self,
        saga_id: SagaId,
        _failed_step: &SagaStep,
    ) -> Result<CompensationPlan, CompensationStrategyError> {
        // 默认 AtLeastOnce 模式, 3 次重试, 100ms 退避
        Ok(CompensationPlan {
            saga_id,
            tenant_id: _failed_step.tenant_id.clone(),
            mode: CompensationMode::AtLeastOnce,
            max_retries: 3,
            retry_backoff_ms: 100,
            idempotency_key: None,  // 待 match 域 Lead 真人补
        })
    }

    async fn execute_compensation(
        &self,
        _plan: &CompensationPlan,
    ) -> Result<(), CompensationStrategyError> {
        // stub: 待 match 域 Lead 真人补详细机制
        Err(CompensationStrategyError::CompensationFailed(
            "DefaultCompensationStrategy stub — 待 match 域 Lead 真人补详细机制".to_string(),
        ))
    }
}

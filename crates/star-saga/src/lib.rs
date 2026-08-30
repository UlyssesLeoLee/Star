// SPDX-License-Identifier: MIT OR Apache-2.0
//! crates/star-saga — Saga orchestrator (per spec/saga/01)

// Phase G 骨架: trait + error + SagaContext + SagaStep + 3 模块实装, 远端调度/分布式追踪/超时重试留 Phase G+.
// 文档待 Phase G+ 实装 Saga 嵌套/版本管理/Saga 持久化时统一补 (per "缺标比错标安全" 偏好).
//
// P3-E.6 docs 阶段 + 骨架 (per `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1, 2026-08-30 10:45 JST):
// - `saga_step` (新增): SagaType / SagaStepStatus / CrossDomainCall 5 域 enum / SagaStep 5 字段 struct
// - `saga_5b_call` (新增): 5 域跨域调用 trait + FiveDomainCallerStub stub
// - `compensation_strategy` (新增): CompensationStrategy trait + DefaultCompensationStrategy stub
// - **待 match 域 Lead 真人补详细机制**: at-least-once / exactly-once / idempotency key / 补偿链顺序
// - **待 5 域 Lead 真人补详细业务逻辑**: 5 域 stub 调用
#![allow(missing_docs)]

pub mod compensation;
pub mod compensation_strategy;
pub mod saga_5b_call;
pub mod saga_orchestrator;
pub mod saga_step;
pub mod step_executor;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Domain {
    Player,
    Economy,
    Match,
    Social,
    Admin,
}

#[derive(Debug, Error)]
pub enum SagaError {
    #[error("step {0} failed: {1}")]
    StepFailed(String, String),
    #[error("compensation {0} failed: {1}")]
    CompensateFailed(String, String),
    #[error("timeout after {0}s")]
    Timeout(u32),
    #[error("saga not found: {0}")]
    NotFound(String),
    #[error("other: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaContext {
    pub saga_id: String,
    pub data: serde_json::Value,
    pub completed_steps: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepResult {
    Success,
    Skip,
    Abort,
}

#[async_trait]
pub trait SagaStep: Send + Sync {
    fn name(&self) -> &str;
    fn domain(&self) -> Domain;
    async fn execute(&self, ctx: &mut SagaContext) -> Result<StepResult, SagaError>;
    async fn compensate(&self, _ctx: &mut SagaContext) -> Result<(), SagaError> {
        Ok(())
    }
}

pub struct Saga {
    pub name: String,
    pub steps: Vec<Box<dyn SagaStep>>,
    pub timeout_sec: u32,
}

pub use compensation::CompensationManager;
pub use compensation_strategy::{CompensationMode, CompensationPlan, CompensationStrategy, DefaultCompensationStrategy};
pub use saga_5b_call::{
    CrossDomainCallError, CrossDomainCallResult, CrossDomainCaller, CrossDomainCallerHealth,
    DomainHealth, FiveDomainCallerStub,
};
pub use saga_orchestrator::SagaOrchestrator;
pub use saga_step::{
    CallId, CrossDomainCall, IdempotencyKey, SagaId, SagaStep as SagaStepData, SagaStepStatus,
    SagaType, StepId, TenantId,
};
pub use step_executor::StepExecutor;

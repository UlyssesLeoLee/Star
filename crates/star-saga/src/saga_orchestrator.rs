//! crates/star-saga/src/saga_orchestrator.rs
//!
//! SagaOrchestrator 编排 Saga 全生命周期 (per P3-E.6 docs 阶段 + 骨架)
//! per `docs/ddd/03-match-bc.md` §2.3 SagaInstance Aggregate + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1
//!
//! ## 职责
//!
//! SagaOrchestrator.execute 顺序执行 Saga.steps, 失败时调 CompensationManager 逆序补偿
//! 6 SagaState 状态机: Pending / Running / Completed / Compensating / Compensated / Failed
//!
//! ## 关键不变量
//!
//! - INV-SG-ORCH-01: SagaState 状态机 6 状态 (per `PHASE-P3-E6-SAGA-IMPL-REPORT.md` §1)
//! - INV-SG-ORCH-02: step 失败自动触发 Compensating → Compensated 状态转移 (per SagaOrchestrator.execute line 50-55)
//! - INV-SG-ORCH-03: idempotency_key 注入 — step 执行走 `step:` 前缀, 补偿走 `compensate:` 前缀 (per `step_executor.rs` `4660ebb` + `compensation.rs` `b0f88b2`, 守门 #11 缺标比错标安全 反向: 补比缺好)
//! - INV-SG-ORCH-04: Saga 状态 map 内存级 (per SagaOrchestrator.states Arc<RwLock<HashMap>>), 待 match 域 Lead 真人补: 持久化 (per process 重启 + per saga 重启)
//!
//! Lead 责任: match 域 Lead (待真人到位)

// per spec/saga/01 §2 SagaOrchestrator
use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SagaState {
    Pending,
    Running,
    Completed,
    Compensating,
    Compensated,
    Failed,
}

pub struct SagaOrchestrator {
    states: Arc<RwLock<HashMap<String, SagaState>>>,
}
impl Default for SagaOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
impl SagaOrchestrator {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn execute(&self, saga: &Saga) -> Result<SagaState, SagaError> {
        let mut ctx = SagaContext {
            saga_id: saga.name.clone(),
            data: serde_json::json!({}),
            completed_steps: vec![],
        };
        let executor = StepExecutor::default();
        let comp = CompensationManager;
        self.set_state(&saga.name, SagaState::Running).await;
        for step in &saga.steps {
            match executor.execute_step(step.as_ref(), &mut ctx).await {
                Ok(StepResult::Success) => {
                    ctx.completed_steps.push(step.name().into());
                }
                Ok(StepResult::Skip) => continue,
                Ok(StepResult::Abort) => {
                    self.set_state(&saga.name, SagaState::Failed).await;
                    return Ok(SagaState::Failed);
                }
                Err(_) => {
                    self.set_state(&saga.name, SagaState::Compensating).await;
                    let _ = comp.compensate_all(saga, &mut ctx).await;
                    self.set_state(&saga.name, SagaState::Compensated).await;
                    return Ok(SagaState::Compensated);
                }
            }
        }
        self.set_state(&saga.name, SagaState::Completed).await;
        Ok(SagaState::Completed)
    }
    pub async fn state(&self, saga_id: &str) -> Result<SagaState, SagaError> {
        let g = self.states.read().await;
        g.get(saga_id)
            .copied()
            .ok_or(SagaError::NotFound(saga_id.into()))
    }
    async fn set_state(&self, saga_id: &str, state: SagaState) {
        let mut g = self.states.write().await;
        g.insert(saga_id.to_string(), state);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn empty_saga() {
        let o = SagaOrchestrator::new();
        let s = Saga {
            name: "x".into(),
            steps: vec![],
            timeout_sec: 60,
        };
        assert_eq!(o.execute(&s).await.unwrap(), SagaState::Completed);
    }
}

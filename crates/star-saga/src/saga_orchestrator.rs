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

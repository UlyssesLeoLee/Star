// per spec/saga/01 §2 SagaStep
use super::*;
pub struct StepExecutor { pub max_retries: u32 }
impl Default for StepExecutor { fn default() -> Self { Self { max_retries: 3 } } }
impl StepExecutor {
    pub async fn execute_step(&self, step: &dyn SagaStep, ctx: &mut SagaContext) -> Result<StepResult, SagaError> {
        match step.execute(ctx).await { Ok(r) => Ok(r), Err(e) => Err(SagaError::StepFailed(step.name().into(), e.to_string())) }
    }
}
#[cfg(test)] mod tests { use super::*; struct DummyStep; #[async_trait] impl SagaStep for DummyStep { fn name(&self) -> &str { "d" } fn domain(&self) -> Domain { Domain::Player } async fn execute(&self, _ctx: &mut SagaContext) -> Result<StepResult, SagaError> { Ok(StepResult::Success) } } #[tokio::test] async fn exec() { let e = StepExecutor::default(); let mut ctx = SagaContext { saga_id: "x".into(), data: serde_json::json!({}), completed_steps: vec![] }; let s = DummyStep; assert!(e.execute_step(&s, &mut ctx).await.is_ok()); } }

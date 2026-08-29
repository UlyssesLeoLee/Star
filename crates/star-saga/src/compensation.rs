// per spec/saga/01 §4 Q-003 compensation
use super::*;
pub struct CompensationManager;
impl CompensationManager {
    pub async fn compensate_all(
        &self,
        saga: &Saga,
        ctx: &mut SagaContext,
    ) -> Result<(), SagaError> {
        for step in saga.steps.iter().rev() {
            if let Err(e) = step.compensate(ctx).await {
                return Err(SagaError::CompensateFailed(
                    step.name().into(),
                    e.to_string(),
                ));
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn noop() {
        let m = CompensationManager;
        let s = Saga {
            name: "x".into(),
            steps: vec![],
            timeout_sec: 60,
        };
        let mut ctx = SagaContext {
            saga_id: "x".into(),
            data: serde_json::json!({}),
            completed_steps: vec![],
        };
        assert!(m.compensate_all(&s, &mut ctx).await.is_ok());
    }
}

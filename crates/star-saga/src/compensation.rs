// per spec/saga/01 §4 Q-003 compensation
use super::*;
pub struct CompensationManager;
impl CompensationManager {
    /// 补偿 saga 所有已完成 step (逆序, per INV-CS-01)
    /// 注入 idempotency_key 到 ctx.data (格式 `saga:{saga_id}:compensate:{step_name}`),
    /// 防止补偿重试导致反向调用重复 (per INV-SG-05 跨 step 失败重复补偿)
    pub async fn compensate_all(
        &self,
        saga: &Saga,
        ctx: &mut SagaContext,
    ) -> Result<(), SagaError> {
        for step in saga.steps.iter().rev() {
            let idem_key = format!("saga:{}:compensate:{}", ctx.saga_id, step.name());
            ctx.data["idempotency_key"] = serde_json::Value::String(idem_key);
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

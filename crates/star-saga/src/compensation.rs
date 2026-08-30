//! crates/star-saga/src/compensation.rs
//!
//! CompensationManager 补偿 saga 已完成 step (per P3-E.6 docs 阶段 + 骨架)
//! per `docs/ddd/03-match-bc.md` §2.3 SagaInstance Aggregate + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1
//!
//! ## 职责
//!
//! CompensationManager.compensate_all 逆序遍历 Saga.steps, 调 step.compensate
//! 失败立即返回 CompensateFailed, 不继续补偿
//!
//! ## 关键不变量
//!
//! - INV-CMP-01: 补偿按 Saga.steps 逆序 (per docs/ddd/03-match-bc.md §2.3 + INV-CS-01)
//! - INV-CMP-02: 补偿 idempotency_key 注入 `saga:{saga_id}:compensate:{step_name}` (per `b0f88b2`, 5 域补偿方读 ctx.data["idempotency_key"] 做 dedup)
//! - INV-CMP-03: 补偿失败不重试, 立即返回 CompensateFailed (per SagaOrchestrator.execute line 50-55, 失败需人工介入)
//!
//! Lead 责任: match 域 Lead (待真人到位补: 补偿链顺序策略 DefaultCompensationStrategy 实现 + 持久化)

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

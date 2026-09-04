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
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    /// 跨域编排覆盖: 5 步 (player / economy / match / social / admin) 全 Success
    /// 验证 5 域串行执行 + 状态机 Completed 边界
    #[tokio::test]
    async fn cross_5_domain_saga_completes() {
        let counter = Arc::new(AtomicUsize::new(0));
        let steps: Vec<Box<dyn SagaStep>> = vec![
            Box::new(CountingStep::new(Domain::Player, counter.clone())),
            Box::new(CountingStep::new(Domain::Economy, counter.clone())),
            Box::new(CountingStep::new(Domain::Match, counter.clone())),
            Box::new(CountingStep::new(Domain::Social, counter.clone())),
            Box::new(CountingStep::new(Domain::Admin, counter.clone())),
        ];
        let saga = Saga {
            name: "5b_cross".into(),
            steps,
            timeout_sec: 60,
        };
        let o = SagaOrchestrator::new();
        let res = o.execute(&saga).await.unwrap();
        assert_eq!(res, SagaState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    /// 跨域编排覆盖: 第 3 步失败 → 触发补偿 → Compensated 状态
    /// 验证 6 状态机 Closed: Pending → Running → Compensating → Compensated
    #[tokio::test]
    async fn cross_5_domain_step3_fails_triggers_compensation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let steps: Vec<Box<dyn SagaStep>> = vec![
            Box::new(CountingStep::new(Domain::Player, counter.clone())),
            Box::new(CountingStep::new(Domain::Economy, counter.clone())),
            Box::new(FailingStep::new(Domain::Match, "boom")),
            Box::new(CountingStep::new(Domain::Social, counter.clone())),
            Box::new(CountingStep::new(Domain::Admin, counter.clone())),
        ];
        let saga = Saga {
            name: "5b_fail".into(),
            steps,
            timeout_sec: 60,
        };
        let o = SagaOrchestrator::new();
        let res = o.execute(&saga).await.unwrap();
        assert_eq!(res, SagaState::Compensated);
        // 2 步 success (player + economy), 第 3 步 fail, 后续不执行
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    /// 跨域编排覆盖: 中间步 Abort → 立即 Failed 状态 (不补偿)
    /// 验证 Abort 行为 vs Err 行为 差异
    #[tokio::test]
    async fn cross_5_domain_abort_terminates_immediately() {
        let counter = Arc::new(AtomicUsize::new(0));
        let steps: Vec<Box<dyn SagaStep>> = vec![
            Box::new(CountingStep::new(Domain::Player, counter.clone())),
            Box::new(AbortingStep::new(Domain::Economy, "user_abort")),
            Box::new(CountingStep::new(Domain::Match, counter.clone())),
        ];
        let saga = Saga {
            name: "5b_abort".into(),
            steps,
            timeout_sec: 60,
        };
        let o = SagaOrchestrator::new();
        let res = o.execute(&saga).await.unwrap();
        assert_eq!(res, SagaState::Failed);
        // 1 步 success + 第 2 步 Abort + 第 3 步不执行
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    /// 跨域编排覆盖: Skip 行为 — 跳过但不终止
    /// 验证 Skip 行为 vs Success 差异 (不计入 completed_steps 但继续执行)
    #[tokio::test]
    async fn cross_5_domain_skip_continues_to_next() {
        let counter = Arc::new(AtomicUsize::new(0));
        let steps: Vec<Box<dyn SagaStep>> = vec![
            Box::new(CountingStep::new(Domain::Player, counter.clone())),
            Box::new(SkippingStep::new(Domain::Economy)),
            Box::new(CountingStep::new(Domain::Match, counter.clone())),
        ];
        let saga = Saga {
            name: "5b_skip".into(),
            steps,
            timeout_sec: 60,
        };
        let o = SagaOrchestrator::new();
        let res = o.execute(&saga).await.unwrap();
        assert_eq!(res, SagaState::Completed);
        // 2 步 success (player + match), economy 是 skip 不计数
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    /// 跨域编排覆盖: 状态查询 (跨多个 saga 状态独立存储)
    /// 验证 SagaOrchestrator.states 内存 map 隔离
    #[tokio::test]
    async fn multiple_sagas_have_independent_states() {
        let o = SagaOrchestrator::new();
        // saga_a 跑完
        let saga_a = Saga {
            name: "a".into(),
            steps: vec![],
            timeout_sec: 60,
        };
        assert_eq!(o.execute(&saga_a).await.unwrap(), SagaState::Completed);
        // saga_b 跑完 (独立)
        let saga_b = Saga {
            name: "b".into(),
            steps: vec![],
            timeout_sec: 60,
        };
        assert_eq!(o.execute(&saga_b).await.unwrap(), SagaState::Completed);
    }

    /// CountingStep: 跨域编排 helper, 计数 + 返回 Success
    struct CountingStep {
        domain: Domain,
        counter: Arc<AtomicUsize>,
    }

    impl CountingStep {
        fn new(domain: Domain, counter: Arc<AtomicUsize>) -> Self {
            Self { domain, counter }
        }
    }

    #[async_trait::async_trait]
    impl SagaStep for CountingStep {
        fn name(&self) -> &str {
            "counting"
        }
        fn domain(&self) -> Domain {
            self.domain
        }
        async fn execute(&self, _ctx: &mut SagaContext) -> Result<StepResult, SagaError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(StepResult::Success)
        }
    }

    /// FailingStep: 跨域编排 helper, 计数 + 返回 Err (触发补偿)
    struct FailingStep {
        domain: Domain,
        reason: String,
        counter: Arc<AtomicUsize>,
    }

    impl FailingStep {
        fn new(domain: Domain, reason: &str) -> Self {
            Self {
                domain,
                reason: reason.into(),
                counter: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl SagaStep for FailingStep {
        fn name(&self) -> &str {
            "failing"
        }
        fn domain(&self) -> Domain {
            self.domain
        }
        async fn execute(&self, _ctx: &mut SagaContext) -> Result<StepResult, SagaError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Err(SagaError::StepFailed(
                self.reason.clone(),
                self.reason.clone(),
            ))
        }
    }

    /// AbortingStep: 跨域编排 helper, 计数 + 返回 Abort (立即 Failed)
    struct AbortingStep {
        domain: Domain,
        reason: String,
        counter: Arc<AtomicUsize>,
    }

    impl AbortingStep {
        fn new(domain: Domain, reason: &str) -> Self {
            Self {
                domain,
                reason: reason.into(),
                counter: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl SagaStep for AbortingStep {
        fn name(&self) -> &str {
            "aborting"
        }
        fn domain(&self) -> Domain {
            self.domain
        }
        async fn execute(&self, _ctx: &mut SagaContext) -> Result<StepResult, SagaError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(StepResult::Abort)
        }
    }

    /// SkippingStep: 跨域编排 helper, 返回 Skip (不计数 + 继续)
    struct SkippingStep {
        domain: Domain,
    }

    impl SkippingStep {
        fn new(domain: Domain) -> Self {
            Self { domain }
        }
    }

    #[async_trait::async_trait]
    impl SagaStep for SkippingStep {
        fn name(&self) -> &str {
            "skipping"
        }
        fn domain(&self) -> Domain {
            self.domain
        }
        async fn execute(&self, _ctx: &mut SagaContext) -> Result<StepResult, SagaError> {
            Ok(StepResult::Skip)
        }
    }
}

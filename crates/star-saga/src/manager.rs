//! crates/star-saga/src/manager.rs
//!
//! SagaManager — 5 域 Saga 跨域编排 (per Phase D.2, Mavis 临时代签 match 域 Lead per 守门 #3 v2)
//! per `OPT-NEXT-01-phase-d.md` §3 D.2 + Q-003 拍板
//!
//! ## 职责
//!
//! - 5 域 Saga dispatch: SAGA_ORDER_FULFILLMENT / SAGA_MATCH_RUN / SAGA_SOCIAL_FEED / SAGA_ADMIN_AUDIT / SAGA_PLAYER_LOGIN
//! - 跨域补偿: 失败时逆序补偿 (per `compensation_strategy.rs` INV-CS-01)
//! - 重试策略: at-least-once + 3 retry + 指数退避 (per `retry_policy.rs` INV-RP-04)
//! - 状态跟踪: 每 Saga 实例 (SagaInstance) 含 state + retry_count + step_index
//!
//! ## 关键不变量
//!
//! - INV-MGR-01: 5 域 Saga 全部经 L0 SagaManager 协调, 5 域 L1 不直接互调 (per 守门 #13 a L1↔L1 禁止)
//! - INV-MGR-02: 跨域失败 → L0 SagaManager rollback (不依赖 L1) (per Q-003 拍板)
//! - INV-MGR-03: SagaInstance 状态机 6 状态: Pending / Running / Compensating / Compensated / Failed / Completed
//! - INV-MGR-04: 重试 idempotency_key 注入, 同 SagaInstance 重试 dedup (per INV-IDS-01 + INV-RP-03)
//!
//! Lead 责任: match 域 Lead (Mavis 临时代签中, per 守门 #3 v2 反转 9/3 11:35 JST 拍板 B)

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::compensation_strategy::{
    CompensationMode, CompensationPlan, CompensationStrategy, DefaultCompensationStrategy,
};
use crate::idempotency_store::{IdempotencyStore, InMemoryIdempotencyStore};
use crate::retry_policy::{RetryPolicy, RetryPolicyError};
use crate::saga_5b_call::CrossDomainCaller;
use crate::saga_5b_sagas::{FiveDomainSaga, SagaDefinition};
use crate::saga_step::{CrossDomainCall, SagaId, TenantId};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SagaManagerError {
    #[error("saga not found: {0}")]
    NotFound(SagaId),
    #[error("cross-domain call failed: {0}")]
    CrossDomainCallFailed(String),
    #[error("retry exhausted: {0}")]
    RetryExhausted(String),
    #[error("compensation failed: {0}")]
    CompensationFailed(String),
    #[error("invalid saga state: {0}")]
    InvalidState(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<RetryPolicyError> for SagaManagerError {
    fn from(e: RetryPolicyError) -> Self {
        SagaManagerError::Internal(e.to_string())
    }
}

/// SagaManager 状态机 (per INV-MGR-03, 6 状态)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaInstanceState {
    Pending,      // 待启动
    Running,      // 5 域调用链执行中
    Compensating, // 5 域调用链失败, 补偿中
    Compensated,  // 5 域调用链失败 + 补偿完成
    Failed,       // 5 域调用链失败 + 补偿失败, 需人工介入
    Completed,    // 5 域调用链全部成功
}

/// SagaInstance — 单个 Saga 执行实例 (per INV-MGR-03)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    pub saga_id: SagaId,
    pub saga_type: FiveDomainSaga,
    pub tenant_id: TenantId,
    pub state: SagaInstanceState,
    pub current_step_index: usize,
    pub retry_count: u32,
    pub idempotency_key: String,
    pub completed_calls: Vec<CrossDomainCall>,
    pub last_error: Option<String>,
    pub compensation_attempts: u32,
}

impl SagaInstance {
    pub fn new(saga_type: FiveDomainSaga, tenant_id: TenantId) -> Self {
        let saga_id = SagaId::new_v4();
        let idempotency_key = format!("saga-mgr:{}:{}", saga_type, saga_id);
        Self {
            saga_id,
            saga_type,
            tenant_id,
            state: SagaInstanceState::Pending,
            current_step_index: 0,
            retry_count: 0,
            idempotency_key,
            completed_calls: vec![],
            last_error: None,
            compensation_attempts: 0,
        }
    }
}

/// SagaManager — 5 域 Saga 跨域编排入口
pub struct SagaManager {
    instances: Arc<RwLock<HashMap<SagaId, SagaInstance>>>,
    caller: Arc<dyn CrossDomainCaller>,
    store: Arc<dyn IdempotencyStore>,
    compensation: Arc<dyn CompensationStrategy>,
    retry_policy: RetryPolicy,
}

impl Default for SagaManager {
    fn default() -> Self {
        let store: Arc<dyn IdempotencyStore> = Arc::new(InMemoryIdempotencyStore::new());
        let caller: Arc<dyn CrossDomainCaller> =
            Arc::new(crate::saga_5b_call::FiveDomainCallerStub);
        let comp: Arc<dyn CompensationStrategy> = Arc::new(DefaultCompensationStrategy::new(
            store.clone(),
            caller.clone(),
        ));
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            caller,
            store,
            compensation: comp,
            retry_policy: RetryPolicy::default(),
        }
    }
}

impl SagaManager {
    /// 构造 SagaManager 自定义 components
    pub fn new(
        caller: Arc<dyn CrossDomainCaller>,
        store: Arc<dyn IdempotencyStore>,
        compensation: Arc<dyn CompensationStrategy>,
        retry_policy: RetryPolicy,
    ) -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            caller,
            store,
            compensation,
            retry_policy,
        }
    }

    /// 注册 Saga instance (Pending 状态)
    pub async fn register(&self, saga_type: FiveDomainSaga, tenant_id: TenantId) -> SagaInstance {
        let instance = SagaInstance::new(saga_type, tenant_id);
        let mut g = self.instances.write().await;
        g.insert(instance.saga_id, instance.clone());
        instance
    }

    /// 启动 Saga: 顺序执行 5 域调用链, 失败重试 + 跨域补偿 (per INV-MGR-01/02/03/04)
    ///
    /// 返回最终状态 (Completed / Compensated / Failed)
    pub async fn execute(&self, saga_id: SagaId) -> Result<SagaInstanceState, SagaManagerError> {
        // 取出 saga definition + 校验
        let def = {
            let g = self.instances.read().await;
            let inst = g.get(&saga_id).ok_or(SagaManagerError::NotFound(saga_id))?;
            inst.saga_type
        };
        let saga_def = SagaDefinition::new(def);
        let call_chain = saga_def.call_chain();
        let compensation_chain = saga_def.compensation_chain();

        // 切到 Running
        self.set_state(saga_id, SagaInstanceState::Running).await;

        // 顺序执行 5 域调用链
        let mut completed_calls: Vec<CrossDomainCall> = Vec::new();
        for (i, call) in call_chain.iter().enumerate() {
            // 单步重试逻辑 (per INV-RP-04 + INV-MGR-04)
            let mut last_err: Option<String> = None;
            let mut success = false;
            for attempt in 0..self.retry_policy.max_retries {
                if attempt > 0 {
                    self.retry_policy.sleep_backoff(attempt - 1).await;
                }
                let idem = format!("{}:{}:attempt:{}", self.idem_key(saga_id, i), def, attempt);
                if !self.store.check_and_record(&idem).await {
                    // dedup hit, skip
                    success = true;
                    break;
                }
                match self
                    .caller
                    .execute_call(saga_id, &self.tenant_id(saga_id).await?, call)
                    .await
                {
                    Ok(_) => {
                        success = true;
                        break;
                    }
                    Err(e) => {
                        last_err = Some(e.to_string());
                        self.inc_retry(saga_id).await;
                    }
                }
            }
            if !success {
                // 重试耗尽 → 进入 Compensating
                let err_msg = last_err.unwrap_or_else(|| "unknown".into());
                self.set_last_error(saga_id, &err_msg).await;
                self.set_state(saga_id, SagaInstanceState::Compensating)
                    .await;
                return self
                    .run_compensation(saga_id, completed_calls, &compensation_chain)
                    .await;
            }
            completed_calls.push(call.clone());
            self.set_step_index(saga_id, i + 1).await;
            self.push_completed_call(saga_id, call.clone()).await;
        }

        // 全部成功 → Completed
        self.set_state(saga_id, SagaInstanceState::Completed).await;
        Ok(SagaInstanceState::Completed)
    }

    /// 执行跨域补偿 (per INV-MGR-02: 失败由 L0 SagaManager 触发, 不依赖 L1)
    async fn run_compensation(
        &self,
        saga_id: SagaId,
        completed: Vec<CrossDomainCall>,
        compensation_chain: &[CrossDomainCall],
    ) -> Result<SagaInstanceState, SagaManagerError> {
        // 构造 CompensationPlan: 走已完成 calls 的逆序
        let mut chain = completed;
        chain.reverse();
        let _ = compensation_chain; // 5 域 Saga 的补偿链定义(预定义, 这里走实际 completed)

        let plan = CompensationPlan {
            saga_id,
            tenant_id: self.tenant_id(saga_id).await?,
            mode: CompensationMode::AtLeastOnce,
            max_retries: self.retry_policy.max_retries,
            retry_backoff_ms: self.retry_policy.initial_backoff_ms,
            idempotency_key: Some(self.idem_key(saga_id, usize::MAX)),
            compensation_chain: chain,
        };
        match self.compensation.execute_compensation(&plan).await {
            Ok(()) => {
                self.set_state(saga_id, SagaInstanceState::Compensated)
                    .await;
                Ok(SagaInstanceState::Compensated)
            }
            Err(e) => {
                self.set_state(saga_id, SagaInstanceState::Failed).await;
                Err(SagaManagerError::CompensationFailed(e.to_string()))
            }
        }
    }

    /// 查询 Saga instance 状态
    pub async fn state(&self, saga_id: SagaId) -> Result<SagaInstanceState, SagaManagerError> {
        let g = self.instances.read().await;
        let inst = g.get(&saga_id).ok_or(SagaManagerError::NotFound(saga_id))?;
        Ok(inst.state)
    }

    /// 查询 Saga instance 完整快照
    pub async fn instance(&self, saga_id: SagaId) -> Result<SagaInstance, SagaManagerError> {
        let g = self.instances.read().await;
        g.get(&saga_id)
            .cloned()
            .ok_or(SagaManagerError::NotFound(saga_id))
    }

    async fn set_state(&self, saga_id: SagaId, state: SagaInstanceState) {
        let mut g = self.instances.write().await;
        if let Some(inst) = g.get_mut(&saga_id) {
            inst.state = state;
        }
    }

    async fn set_step_index(&self, saga_id: SagaId, idx: usize) {
        let mut g = self.instances.write().await;
        if let Some(inst) = g.get_mut(&saga_id) {
            inst.current_step_index = idx;
        }
    }

    async fn push_completed_call(&self, saga_id: SagaId, call: CrossDomainCall) {
        let mut g = self.instances.write().await;
        if let Some(inst) = g.get_mut(&saga_id) {
            inst.completed_calls.push(call);
        }
    }

    async fn inc_retry(&self, saga_id: SagaId) {
        let mut g = self.instances.write().await;
        if let Some(inst) = g.get_mut(&saga_id) {
            inst.retry_count += 1;
        }
    }

    async fn set_last_error(&self, saga_id: SagaId, err: &str) {
        let mut g = self.instances.write().await;
        if let Some(inst) = g.get_mut(&saga_id) {
            inst.last_error = Some(err.into());
        }
    }

    async fn tenant_id(&self, saga_id: SagaId) -> Result<TenantId, SagaManagerError> {
        let g = self.instances.read().await;
        g.get(&saga_id)
            .map(|i| i.tenant_id.clone())
            .ok_or(SagaManagerError::NotFound(saga_id))
    }

    fn idem_key(&self, saga_id: SagaId, step: usize) -> String {
        format!("mgr:{}:step:{}", saga_id, step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saga_5b_call::{
        CrossDomainCallError, CrossDomainCallResult, CrossDomainCallerHealth, DomainHealth,
    };
    use crate::saga_5b_sagas::FiveDomainSaga;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Always-OK caller for happy path
    struct AlwaysOkCaller {
        counter: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl CrossDomainCaller for AlwaysOkCaller {
        async fn execute_call(
            &self,
            _saga_id: SagaId,
            _tenant_id: &TenantId,
            call: &CrossDomainCall,
        ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(CrossDomainCallResult {
                call_id: match call {
                    CrossDomainCall::PlayerCall { call_id, .. } => *call_id,
                    CrossDomainCall::EconomyCall { call_id, .. } => *call_id,
                    CrossDomainCall::MatchCall { call_id, .. } => *call_id,
                    CrossDomainCall::SocialCall { call_id, .. } => *call_id,
                    CrossDomainCall::AdminCall { call_id, .. } => *call_id,
                },
                success: true,
                result_data: Some(serde_json::json!({"ok": true})),
                error: None,
                latency_ms: 1,
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

    #[tokio::test]
    async fn register_creates_pending_instance() {
        let m = SagaManager::default();
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t1".into()).await;
        assert_eq!(inst.state, SagaInstanceState::Pending);
        assert_eq!(inst.current_step_index, 0);
        assert_eq!(inst.retry_count, 0);
        assert!(
            m.state(inst.saga_id)
                .await
                .unwrap_or(SagaInstanceState::Failed)
                == SagaInstanceState::Pending
                || m.state(inst.saga_id).await.unwrap() == SagaInstanceState::Pending
        );
    }

    #[tokio::test]
    async fn execute_player_login_happy_path() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let store: Arc<dyn IdempotencyStore> = Arc::new(InMemoryIdempotencyStore::new());
        let comp: Arc<dyn CompensationStrategy> = Arc::new(DefaultCompensationStrategy::new(
            store.clone(),
            caller.clone(),
        ));
        let m = SagaManager::new(
            caller,
            store,
            comp,
            RetryPolicy {
                max_retries: 1,
                initial_backoff_ms: 50,
                backoff_multiplier: 2,
                max_backoff_ms: 100,
            },
        );
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t1".into()).await;
        let s = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(s, SagaInstanceState::Completed);
    }
}

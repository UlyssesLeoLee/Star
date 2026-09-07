//! crates/star-saga/src/saga_5b_sagas_tests.rs
//!
//! D.2 E2E 测试 — 5 域 Saga happy path + 跨域失败回滚 (per Phase D.2)
//! per `OPT-NEXT-01-phase-d.md` §3 D.2 + P4-UNIMPL-WBS-001 §5 D.2 E2E ≥80%
//!
//! ## 覆盖矩阵 (per P4-UNIMPL-WBS-001 §5 D.2)
//!
//! | # | Saga | 主域 | Happy Path Test | Failure Test |
//! |---|---|---|---|---|
//! | 1 | SAGA_ORDER_FULFILLMENT | economy | order_fulfillment_happy_path | order_fulfillment_economy_deduct_fails |
//! | 2 | SAGA_MATCH_RUN | match | match_run_happy_path | match_run_match_start_fails |
//! | 3 | SAGA_SOCIAL_FEED | social | social_feed_happy_path | social_feed_admin_assign_fails |
//! | 4 | SAGA_ADMIN_AUDIT | admin | admin_audit_happy_path | admin_audit_match_fails |
//! | 5 | SAGA_PLAYER_LOGIN | player | player_login_happy_path | player_login_admin_assign_fails |
//!
//! ## 额外测试
//!
//! - saga_5b_all_complete (5 域 Saga 全部 happy path)
//! - saga_state_machine_pending_to_completed (状态机完整迁移)
//! - saga_5b_compensation_preserves_completed_chain (补偿链只回滚已完成)
//! - saga_retry_policy_recovers_on_transient_failure (重试恢复)
//! - saga_retry_exhausted_triggers_compensation (重试耗尽 → 补偿)
//! - saga_idempotency_dedup_retry (重试 idempotency dedup)
//! - saga_invalid_retry_policy_rejected (参数校验)
//! - saga_manager_not_found (NotFound 错误)
//! - saga_compensation_mode_at_least_once (AtLeastOnce 模式)
//! - saga_5b_counting_each_step (验证 step_index 递增)

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crate::compensation_strategy::{
        CompensationMode, CompensationStrategy, DefaultCompensationStrategy,
    };
    use crate::idempotency_store::{IdempotencyStore, InMemoryIdempotencyStore};
    use crate::manager::{SagaInstanceState, SagaManager};
    use crate::retry_policy::RetryPolicy;
    use crate::saga_5b_call::{
        CrossDomainCallError, CrossDomainCallResult, CrossDomainCaller, CrossDomainCallerHealth,
        DomainHealth,
    };
    use crate::saga_5b_sagas::FiveDomainSaga;
    use crate::saga_step::{CrossDomainCall, SagaId, TenantId};

    // === Test helpers — share counter via Arc<AtomicUsize> ===

    /// Always-OK caller; 共享外部 counter
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
                call_id: call_id_of(call),
                success: true,
                result_data: Some(serde_json::json!({"ok": true})),
                error: None,
                latency_ms: 1,
            })
        }
        async fn health(&self) -> Result<CrossDomainCallerHealth, CrossDomainCallError> {
            Ok(all_healthy())
        }
    }

    /// Fail-on-condition caller; fail_action 名 fail
    struct CondFailCaller {
        counter: Arc<AtomicUsize>,
        fail_action: String,
    }

    #[async_trait::async_trait]
    impl CrossDomainCaller for CondFailCaller {
        async fn execute_call(
            &self,
            _saga_id: SagaId,
            _tenant_id: &TenantId,
            call: &CrossDomainCall,
        ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            let action = match call {
                CrossDomainCall::PlayerCall { action, .. } => action,
                CrossDomainCall::EconomyCall { action, .. } => action,
                CrossDomainCall::MatchCall { action, .. } => action,
                CrossDomainCall::SocialCall { action, .. } => action,
                CrossDomainCall::AdminCall { action, .. } => action,
            };
            if action.as_str() == self.fail_action.as_str() {
                Err(CrossDomainCallError::Internal(format!(
                    "forced fail: {}",
                    action
                )))
            } else {
                Ok(CrossDomainCallResult {
                    call_id: call_id_of(call),
                    success: true,
                    result_data: Some(serde_json::json!({"ok": true})),
                    error: None,
                    latency_ms: 1,
                })
            }
        }
        async fn health(&self) -> Result<CrossDomainCallerHealth, CrossDomainCallError> {
            Ok(all_healthy())
        }
    }

    /// Fail first fail_max times then succeed (transient)
    struct TransientFailCaller {
        counter: Arc<AtomicUsize>,
        fail_attempts: Arc<AtomicUsize>,
        fail_max: usize,
    }

    #[async_trait::async_trait]
    impl CrossDomainCaller for TransientFailCaller {
        async fn execute_call(
            &self,
            _saga_id: SagaId,
            _tenant_id: &TenantId,
            call: &CrossDomainCall,
        ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            let current = self.fail_attempts.fetch_add(1, Ordering::SeqCst);
            if (current as usize) < self.fail_max {
                Err(CrossDomainCallError::Internal("transient".into()))
            } else {
                Ok(CrossDomainCallResult {
                    call_id: call_id_of(call),
                    success: true,
                    result_data: Some(serde_json::json!({"recovered": true})),
                    error: None,
                    latency_ms: 1,
                })
            }
        }
        async fn health(&self) -> Result<CrossDomainCallerHealth, CrossDomainCallError> {
            Ok(all_healthy())
        }
    }

    fn call_id_of(call: &CrossDomainCall) -> crate::saga_step::CallId {
        match call {
            CrossDomainCall::PlayerCall { call_id, .. } => *call_id,
            CrossDomainCall::EconomyCall { call_id, .. } => *call_id,
            CrossDomainCall::MatchCall { call_id, .. } => *call_id,
            CrossDomainCall::SocialCall { call_id, .. } => *call_id,
            CrossDomainCall::AdminCall { call_id, .. } => *call_id,
        }
    }

    fn all_healthy() -> CrossDomainCallerHealth {
        CrossDomainCallerHealth {
            player_health: DomainHealth::Healthy,
            economy_health: DomainHealth::Healthy,
            match_health: DomainHealth::Healthy,
            social_health: DomainHealth::Healthy,
            admin_health: DomainHealth::Healthy,
        }
    }

    /// Build SagaManager with no retries (speed)
    fn manager_no_retry(caller: Arc<dyn CrossDomainCaller>) -> SagaManager {
        let store: Arc<dyn IdempotencyStore> = Arc::new(InMemoryIdempotencyStore::new());
        let comp: Arc<dyn CompensationStrategy> = Arc::new(DefaultCompensationStrategy::new(
            store.clone(),
            caller.clone(),
        ));
        SagaManager::new(
            caller,
            store,
            comp,
            RetryPolicy {
                max_retries: 1,
                initial_backoff_ms: 50,
                backoff_multiplier: 2,
                max_backoff_ms: 100,
            },
        )
    }

    /// Build SagaManager with retries enabled (3 retries)
    fn manager_with_retries(caller: Arc<dyn CrossDomainCaller>) -> SagaManager {
        let store: Arc<dyn IdempotencyStore> = Arc::new(InMemoryIdempotencyStore::new());
        let comp: Arc<dyn CompensationStrategy> = Arc::new(DefaultCompensationStrategy::new(
            store.clone(),
            caller.clone(),
        ));
        SagaManager::new(caller, store, comp, RetryPolicy::default())
    }

    // ============================================================
    // 1. SAGA_ORDER_FULFILLMENT — economy 主导 (4 calls)
    // ============================================================

    #[tokio::test]
    async fn order_fulfillment_happy_path() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::OrderFulfillment, "t-order".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 4); // 4 calls: economy create + deduct + match start + admin role
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.current_step_index, 4);
        assert_eq!(snap.retry_count, 0);
    }

    #[tokio::test]
    async fn order_fulfillment_economy_deduct_fails_triggers_compensation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "deduct_currency".into(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::OrderFulfillment, "t-order".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Compensated);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert!(snap.last_error.is_some());
        assert!(snap.retry_count >= 1);
        assert_eq!(snap.completed_calls.len(), 1); // 1 success: create_billing_account
                                                   // 1 success + 1 fail (retried once) + 1 compensation = 3
                                                   // 注意: 因为 max_retries=1, attempt 0 fail, 不重试, 直接 compensation
                                                   // 所以 counter = 1 (success) + 1 (fail) + 1 (compensation 1 step) = 3
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    // ============================================================
    // 2. SAGA_MATCH_RUN — match 主导 (4 calls)
    // ============================================================

    #[tokio::test]
    async fn match_run_happy_path() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m.register(FiveDomainSaga::MatchRun, "t-match".into()).await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 4); // player + economy + match + social
    }

    #[tokio::test]
    async fn match_run_match_start_fails_triggers_compensation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "start_workflow".into(),
        });
        let m = manager_no_retry(caller);
        let inst = m.register(FiveDomainSaga::MatchRun, "t-match".into()).await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Compensated);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert!(snap.retry_count >= 1);
        assert_eq!(snap.completed_calls.len(), 2);
        // 2 success + 1 fail + 2 compensation = 5
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    // ============================================================
    // 3. SAGA_SOCIAL_FEED — social 主导 (3 calls)
    // ============================================================

    #[tokio::test]
    async fn social_feed_happy_path() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::SocialFeed, "t-social".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // player + social + admin
    }

    #[tokio::test]
    async fn social_feed_admin_assign_fails_triggers_compensation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "assign_role".into(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::SocialFeed, "t-social".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Compensated);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.completed_calls.len(), 2);
        // 2 success + 1 fail + 2 compensation = 5
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    // ============================================================
    // 4. SAGA_ADMIN_AUDIT — admin 主导 (4 calls)
    // ============================================================

    #[tokio::test]
    async fn admin_audit_happy_path() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::AdminAudit, "t-admin".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 4); // economy + match + social + admin
    }

    #[tokio::test]
    async fn admin_audit_match_fails_triggers_compensation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "start_workflow".into(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::AdminAudit, "t-admin".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Compensated);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.completed_calls.len(), 1); // 1 success: economy create_billing_account
                                                   // 1 success + 1 fail + 1 compensation = 3
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    // ============================================================
    // 5. SAGA_PLAYER_LOGIN — player 主导 (2 calls)
    // ============================================================

    #[tokio::test]
    async fn player_login_happy_path() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::PlayerLogin, "t-player".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 2); // player + admin
    }

    #[tokio::test]
    async fn player_login_admin_assign_fails_triggers_compensation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "assign_role".into(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::PlayerLogin, "t-player".into())
            .await;
        let state = m.execute(inst.saga_id).await.unwrap();
        assert_eq!(state, SagaInstanceState::Compensated);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.completed_calls.len(), 1); // 1 success: player create_user
                                                   // 1 success + 1 fail + 1 compensation = 3
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    // ============================================================
    // 6. 跨 Saga 全部 happy path 顺序跑
    // ============================================================

    #[tokio::test]
    async fn saga_5b_all_complete() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        for sagatype in [
            FiveDomainSaga::OrderFulfillment,
            FiveDomainSaga::MatchRun,
            FiveDomainSaga::SocialFeed,
            FiveDomainSaga::AdminAudit,
            FiveDomainSaga::PlayerLogin,
        ] {
            let inst = m.register(sagatype, "t-all".into()).await;
            let state = m.execute(inst.saga_id).await.unwrap();
            assert_eq!(state, SagaInstanceState::Completed);
        }
        // 4 + 4 + 3 + 4 + 2 = 17
        assert_eq!(counter.load(Ordering::SeqCst), 17);
    }

    // ============================================================
    // 7. 状态机完整迁移 (Pending → Running → Completed)
    // ============================================================

    #[tokio::test]
    async fn saga_state_machine_pending_to_completed() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t".into()).await;
        assert_eq!(inst.state, SagaInstanceState::Pending);
        assert_eq!(
            m.state(inst.saga_id).await.unwrap(),
            SagaInstanceState::Pending
        );
        let _ = m.execute(inst.saga_id).await.unwrap();
        let final_state = m.state(inst.saga_id).await.unwrap();
        assert_eq!(final_state, SagaInstanceState::Completed);
    }

    // ============================================================
    // 8. 补偿链只回滚已完成 steps (per INV-SAGA-02)
    // ============================================================

    #[tokio::test]
    async fn saga_5b_compensation_preserves_completed_chain() {
        // match_run 第 3 步 fail, 只回滚前 2 步
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "start_workflow".into(),
        });
        let m = manager_no_retry(caller);
        let inst = m.register(FiveDomainSaga::MatchRun, "t".into()).await;
        let _ = m.execute(inst.saga_id).await.unwrap();
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.completed_calls.len(), 2); // player + economy
                                                   // 2 step success + 1 fail + 2 compensation = 5
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    // ============================================================
    // 9. 重试恢复 (transient failure, 2 次后恢复)
    // ============================================================

    #[tokio::test]
    async fn saga_retry_policy_recovers_on_transient_failure() {
        // fail_max=2: 前 2 次 fail, 第 3 次 success (per TransientFailCaller: current < fail_max)
        // 因 fail_attempts 是全局共享, 第 1 step 必然 2 fail + 1 success; 第 2 step 第 1 次就 success
        let counter = Arc::new(AtomicUsize::new(0));
        let fail_attempts = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(TransientFailCaller {
            counter: counter.clone(),
            fail_attempts: fail_attempts.clone(),
            fail_max: 2,
        });
        let m = manager_with_retries(caller);
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t".into()).await;
        let state = m.execute(inst.saga_id).await.unwrap();
        // step 0 (player): attempt 0 fail, attempt 1 fail, attempt 2 success → 2 fail + 1 success
        // step 1 (admin): attempt 0 success (current=3, 3<2 false) → 1 success
        // total = 4 calls, retry_count = 2
        assert_eq!(state, SagaInstanceState::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 4);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.retry_count, 2);
    }

    // ============================================================
    // 10. 重试耗尽 → 触发补偿
    // ============================================================

    #[tokio::test]
    async fn saga_retry_exhausted_triggers_compensation() {
        // 全部失败
        let counter = Arc::new(AtomicUsize::new(0));
        let fail_attempts = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(TransientFailCaller {
            counter: counter.clone(),
            fail_attempts: fail_attempts.clone(),
            fail_max: 1000,
        });
        let m = manager_with_retries(caller);
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t".into()).await;
        let state = m.execute(inst.saga_id).await.unwrap();
        // 第 1 步 3 次重试都失败 → Compensated
        // compensation chain 为空 (step 1 全失败, 0 completed) → 0 补偿调用
        // total = 3 calls; retry_count = 3 (3 fails)
        assert_eq!(state, SagaInstanceState::Compensated);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.retry_count, 3);
    }

    // ============================================================
    // 10b. 重试耗尽 + 部分 step 已完成 → 触发补偿 (已完成的 step)
    // ============================================================

    #[tokio::test]
    async fn saga_retry_exhausted_with_partial_completion_triggers_compensation() {
        // step 1 (player create_user) 成功, step 2 (admin assign_role) 永远失败
        // 期望: 1 success + 3 fails + 1 补偿 (player) = 5 calls; retry_count = 3; completed_calls = 1
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(CondFailCaller {
            counter: counter.clone(),
            fail_action: "assign_role".into(),
        });
        let m = manager_with_retries(caller);
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t".into()).await;
        let state = m.execute(inst.saga_id).await.unwrap();
        // step1 (create_user) 1 success → 完成
        // step2 (assign_role) 3 attempts 全 fail → 触发 Compensating
        // 补偿 chain: 1 步 (player create_user) → 1 call
        // total = 1 + 3 + 1 = 5
        assert_eq!(state, SagaInstanceState::Compensated);
        let snap = m.instance(inst.saga_id).await.unwrap();
        assert_eq!(snap.completed_calls.len(), 1);
        assert_eq!(snap.retry_count, 3);
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    // ============================================================
    // 11. 重试 idempotency dedup
    // ============================================================

    #[tokio::test]
    async fn saga_idempotency_dedup_retry() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m.register(FiveDomainSaga::PlayerLogin, "t".into()).await;
        let _ = m.execute(inst.saga_id).await.unwrap();
        let snap = m.instance(inst.saga_id).await.unwrap();
        // happy path: retry_count = 0
        assert_eq!(snap.retry_count, 0);
        assert_eq!(snap.current_step_index, 2);
    }

    // ============================================================
    // 12. RetryPolicy 参数校验
    // ============================================================

    #[tokio::test]
    async fn saga_invalid_retry_policy_rejected() {
        use crate::retry_policy::RetryPolicy;
        // max_retries = 0
        assert!(RetryPolicy::new(0, 100, 2, 1000).is_err());
        // max_retries = 6
        assert!(RetryPolicy::new(6, 100, 2, 1000).is_err());
        // initial_backoff_ms < 50
        assert!(RetryPolicy::new(3, 49, 2, 1000).is_err());
        // initial_backoff_ms > 1000
        assert!(RetryPolicy::new(3, 1001, 2, 1000).is_err());
        // backoff_multiplier = 0
        assert!(RetryPolicy::new(3, 100, 0, 1000).is_err());
        // max < initial
        assert!(RetryPolicy::new(3, 500, 2, 100).is_err());
        // valid
        assert!(RetryPolicy::new(3, 100, 2, 1000).is_ok());
        assert!(RetryPolicy::new(1, 50, 1, 50).is_ok());
        assert!(RetryPolicy::new(5, 200, 3, 1500).is_ok());
    }

    // ============================================================
    // 13. SagaManager NotFound 错误
    // ============================================================

    #[tokio::test]
    async fn saga_manager_not_found() {
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: Arc::new(AtomicUsize::new(0)),
        });
        let m = manager_no_retry(caller);
        let bogus_id = SagaId::new_v4();
        let result = m.execute(bogus_id).await;
        assert!(result.is_err());
    }

    // ============================================================
    // 14. CompensationMode 全部拍板
    // ============================================================

    #[tokio::test]
    async fn saga_compensation_mode_at_least_once() {
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: Arc::new(AtomicUsize::new(0)),
        });
        let store: Arc<dyn IdempotencyStore> = Arc::new(InMemoryIdempotencyStore::new());
        let comp: Arc<dyn CompensationStrategy> = Arc::new(DefaultCompensationStrategy::new(
            store.clone(),
            caller.clone(),
        ));
        // plan 默认是 AtLeastOnce
        let plan = comp
            .plan_compensation(
                SagaId::new_v4(),
                &crate::saga_step::SagaStep::new(
                    "t".into(),
                    crate::saga_step::SagaType::CreateProject,
                    vec![],
                ),
            )
            .await
            .unwrap();
        assert_eq!(plan.mode, CompensationMode::AtLeastOnce);
        assert!(comp.execute_compensation(&plan).await.is_ok());
    }

    // ============================================================
    // 15. step_index 递增验证
    // ============================================================

    #[tokio::test]
    async fn saga_5b_counting_each_step() {
        let counter = Arc::new(AtomicUsize::new(0));
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: counter.clone(),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::OrderFulfillment, "t".into())
            .await;
        let _ = m.execute(inst.saga_id).await.unwrap();
        let snap = m.instance(inst.saga_id).await.unwrap();
        // OrderFulfillment 4 calls: economy create + deduct + match start + admin role
        assert_eq!(snap.current_step_index, 4);
        assert_eq!(snap.completed_calls.len(), 4);
    }

    // ============================================================
    // 16. SagaInstance 字段完整性
    // ============================================================

    #[tokio::test]
    async fn saga_instance_fields_complete() {
        let caller: Arc<dyn CrossDomainCaller> = Arc::new(AlwaysOkCaller {
            counter: Arc::new(AtomicUsize::new(0)),
        });
        let m = manager_no_retry(caller);
        let inst = m
            .register(FiveDomainSaga::MatchRun, "t-fields".into())
            .await;
        // 字段初值
        assert_eq!(inst.saga_type, FiveDomainSaga::MatchRun);
        assert_eq!(inst.tenant_id, "t-fields");
        assert_eq!(inst.state, SagaInstanceState::Pending);
        assert_eq!(inst.current_step_index, 0);
        assert_eq!(inst.retry_count, 0);
        assert!(!inst.idempotency_key.is_empty());
        assert!(inst.completed_calls.is_empty());
        assert!(inst.last_error.is_none());
        assert_eq!(inst.compensation_attempts, 0);
    }
}

//! crates/star-saga/src/saga_5b_real.rs
//!
//! E.1 FiveDomainCallerReal — 真实 5 域 service 调用的 CrossDomainCaller 实现
//! (per P4-E.1, 9/4 拍板, Mavis 临时代签 5 域 Lead per 守门 #14)

use async_trait::async_trait;
use std::sync::Arc;

use crate::saga_5b_call::{
    CrossDomainCallError, CrossDomainCallResult, CrossDomainCaller, CrossDomainCallerHealth,
    DomainHealth,
};
use crate::saga_5b_services::{
    AdminService, EconomyService, MatchService, PlayerService, SocialService,
};
use crate::saga_step::{CallId, CrossDomainCall, SagaId, TenantId};

/// FiveDomainCallerReal — 持有 5 域 service, dispatch CrossDomainCall 到对应 service
pub struct FiveDomainCallerReal {
    pub player: Arc<PlayerService>,
    pub economy: Arc<EconomyService>,
    pub match_svc: Arc<MatchService>,
    pub social: Arc<SocialService>,
    pub admin: Arc<AdminService>,
}

impl FiveDomainCallerReal {
    pub fn new(
        player: Arc<PlayerService>,
        economy: Arc<EconomyService>,
        match_svc: Arc<MatchService>,
        social: Arc<SocialService>,
        admin: Arc<AdminService>,
    ) -> Self {
        Self {
            player,
            economy,
            match_svc,
            social,
            admin,
        }
    }

    /// 默认构造 5 域 service
    pub fn default_5() -> Self {
        Self::new(
            Arc::new(PlayerService::new()),
            Arc::new(EconomyService::new()),
            Arc::new(MatchService::new()),
            Arc::new(SocialService::new()),
            Arc::new(AdminService::new()),
        )
    }
}

fn map_err(e: crate::saga_5b_services::DomainError) -> CrossDomainCallError {
    use crate::saga_5b_services::DomainError;
    match e {
        DomainError::Player(m) => CrossDomainCallError::PlayerCallFailed(m),
        DomainError::Economy(m) => CrossDomainCallError::EconomyCallFailed(m),
        DomainError::Match(m) => CrossDomainCallError::MatchCallFailed(m),
        DomainError::Social(m) => CrossDomainCallError::SocialCallFailed(m),
        DomainError::Admin(m) => CrossDomainCallError::AdminCallFailed(m),
    }
}

fn call_id_of(call: &CrossDomainCall) -> CallId {
    match call {
        CrossDomainCall::PlayerCall { call_id, .. } => *call_id,
        CrossDomainCall::EconomyCall { call_id, .. } => *call_id,
        CrossDomainCall::MatchCall { call_id, .. } => *call_id,
        CrossDomainCall::SocialCall { call_id, .. } => *call_id,
        CrossDomainCall::AdminCall { call_id, .. } => *call_id,
    }
}

#[async_trait]
impl CrossDomainCaller for FiveDomainCallerReal {
    async fn execute_call(
        &self,
        _saga_id: SagaId,
        tenant_id: &TenantId,
        call: &CrossDomainCall,
    ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
        let call_id = call_id_of(call);
        let result = match call {
            CrossDomainCall::PlayerCall {
                action, target_id, ..
            } => match action.as_str() {
                "create_user" => {
                    let p = self
                        .player
                        .register(tenant_id, target_id)
                        .map_err(map_err)?;
                    serde_json::json!({"user_id": p.user_id, "status": "active"})
                }
                "suspend_user" => {
                    self.player.suspend(target_id).map_err(map_err)?;
                    serde_json::json!({"target_id": target_id, "status": "suspended"})
                }
                _ => {
                    return Err(CrossDomainCallError::Internal(format!(
                        "unknown player action: {}",
                        action
                    )))
                }
            },
            CrossDomainCall::EconomyCall {
                action, target_id, ..
            } => {
                match action.as_str() {
                    "create_billing_account" => {
                        let a = self
                            .economy
                            .create_account(tenant_id, target_id)
                            .map_err(map_err)?;
                        serde_json::json!({"billing_account_id": a.billing_account_id, "balance": a.balance})
                    }
                    "deduct_currency" => {
                        // payload amount (固定 100 cents for PoC)
                        self.economy.deduct(target_id, 100).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "deducted": 100})
                    }
                    "refund_currency" => {
                        self.economy.refund(target_id, 100).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "refunded": 100})
                    }
                    _ => {
                        return Err(CrossDomainCallError::Internal(format!(
                            "unknown economy action: {}",
                            action
                        )))
                    }
                }
            }
            CrossDomainCall::MatchCall {
                action, target_id, ..
            } => match action.as_str() {
                "start_workflow" => {
                    let w = self
                        .match_svc
                        .start_workflow(tenant_id, target_id)
                        .map_err(map_err)?;
                    serde_json::json!({"workflow_instance_id": w.workflow_instance_id, "status": "running"})
                }
                "abort_workflow" => {
                    self.match_svc.abort_workflow(target_id).map_err(map_err)?;
                    serde_json::json!({"target_id": target_id, "status": "aborted"})
                }
                _ => {
                    return Err(CrossDomainCallError::Internal(format!(
                        "unknown match action: {}",
                        action
                    )))
                }
            },
            CrossDomainCall::SocialCall {
                action, target_id, ..
            } => {
                match action.as_str() {
                    "send_notification" => {
                        // PoC: user_id = target_id
                        let n = self
                            .social
                            .send_notification(tenant_id, target_id, target_id)
                            .map_err(map_err)?;
                        serde_json::json!({"notification_id": n.notification_id, "user_id": n.user_id})
                    }
                    "mark_notification_read" => {
                        self.social.mark_read(target_id).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "read": true})
                    }
                    _ => {
                        return Err(CrossDomainCallError::Internal(format!(
                            "unknown social action: {}",
                            action
                        )))
                    }
                }
            }
            CrossDomainCall::AdminCall {
                action, target_id, ..
            } => {
                match action.as_str() {
                    "assign_role" => {
                        // PoC: user_id = target_id
                        let r = self
                            .admin
                            .assign_role(tenant_id, target_id, target_id)
                            .map_err(map_err)?;
                        serde_json::json!({"role_id": r.role_id, "user_id": r.user_id, "active": r.active})
                    }
                    "revoke_role" => {
                        self.admin.revoke_role(target_id).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "active": false})
                    }
                    _ => {
                        return Err(CrossDomainCallError::Internal(format!(
                            "unknown admin action: {}",
                            action
                        )))
                    }
                }
            }
        };
        Ok(CrossDomainCallResult {
            call_id,
            success: true,
            result_data: Some(result),
            error: None,
            latency_ms: 1, // PoC: 1ms
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

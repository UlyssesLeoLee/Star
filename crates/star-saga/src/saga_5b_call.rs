//! crates/star-saga/src/saga_5b_call.rs
//!
//! 5 域跨域调用 trait 扩展 (per P3-E.6 docs 阶段 + 骨架, P3-F.4 跨域 Saga 流程 + 5 域 stub)
//! per `docs/ddd/03-match-bc.md` §2.3 SagaInstance Aggregate + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1
//!
//! ## 职责
//!
//! 5 域 (player / economy / match / social / admin) 跨域调用 trait 骨架
//! 5 域各 1 stub 实现, 待 5 域 Lead 真人到位补详细业务逻辑
//!
//! ## 关键不变量 (待 match 域 Lead 真人补详细机制)
//!
//! - INV-SG-5B-01: 5 域调用必带 tenant_id
//! - INV-SG-5B-02: 5 域调用必填 idempotency key (待补)
//! - INV-SG-5B-03: 5 域调用超时 + 重试 + 降级 (待补)
//! - INV-SG-5B-04: 5 域调用失败触发 Saga 补偿 (per Compensation trait, 见 compensation_strategy.rs)
//!
//! Lead 责任: 5 域 Lead 各 1 (5 真人) — match 域 Lead 跨域协调

use async_trait::async_trait;

use crate::saga_step::{CallId, CrossDomainCall, SagaId, TenantId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrossDomainCallError {
    #[error("player 域 call failed: {0}")]
    PlayerCallFailed(String),
    #[error("economy 域 call failed: {0}")]
    EconomyCallFailed(String),
    #[error("match 域 call failed: {0}")]
    MatchCallFailed(String),
    #[error("social 域 call failed: {0}")]
    SocialCallFailed(String),
    #[error("admin 域 call failed: {0}")]
    AdminCallFailed(String),
    #[error("call timeout: {0}")]
    Timeout(CallId),
    #[error("internal error: {0}")]
    Internal(String),
}

/// 5 域跨域调用 trait (P3-E.6 骨架, 5 域 stub)
/// **待 5 域 Lead 真人到位补详细业务逻辑** (每域 1 Lead)
#[async_trait]
pub trait CrossDomainCaller: Send + Sync {
    /// 执行 5 域调用
    async fn execute_call(
        &self,
        saga_id: SagaId,
        tenant_id: &TenantId,
        call: &CrossDomainCall,
    ) -> Result<CrossDomainCallResult, CrossDomainCallError>;

    /// 健康检查
    async fn health(&self) -> Result<CrossDomainCallerHealth, CrossDomainCallError>;
}

#[derive(Debug, Clone)]
pub struct CrossDomainCallResult {
    pub call_id: CallId,
    pub success: bool,
    pub result_data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CrossDomainCallerHealth {
    pub player_health: DomainHealth,
    pub economy_health: DomainHealth,
    pub match_health: DomainHealth,
    pub social_health: DomainHealth,
    pub admin_health: DomainHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainHealth {
    Healthy,
    Degraded,
    Unavailable,
}

/// 5 域跨域调用 trait 骨架 stub 实现 (per E.6 骨架, 待 5 域 Lead 真人补)
pub struct FiveDomainCallerStub;

#[async_trait]
impl CrossDomainCaller for FiveDomainCallerStub {
    async fn execute_call(
        &self,
        _saga_id: SagaId,
        _tenant_id: &TenantId,
        call: &CrossDomainCall,
    ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
        // 5 域 stub: 仅返回 success + result_data = call action name, 待 5 域 Lead 真人补
        Ok(CrossDomainCallResult {
            call_id: match call {
                CrossDomainCall::PlayerCall { call_id, .. } => *call_id,
                CrossDomainCall::EconomyCall { call_id, .. } => *call_id,
                CrossDomainCall::MatchCall { call_id, .. } => *call_id,
                CrossDomainCall::SocialCall { call_id, .. } => *call_id,
                CrossDomainCall::AdminCall { call_id, .. } => *call_id,
            },
            success: true,
            result_data: Some(serde_json::json!({
                "stub": true,
                "domain": match call {
                    CrossDomainCall::PlayerCall { .. } => "player",
                    CrossDomainCall::EconomyCall { .. } => "economy",
                    CrossDomainCall::MatchCall { .. } => "match",
                    CrossDomainCall::SocialCall { .. } => "social",
                    CrossDomainCall::AdminCall { .. } => "admin",
                },
                "action": match call {
                    CrossDomainCall::PlayerCall { action, .. } => action,
                    CrossDomainCall::EconomyCall { action, .. } => action,
                    CrossDomainCall::MatchCall { action, .. } => action,
                    CrossDomainCall::SocialCall { action, .. } => action,
                    CrossDomainCall::AdminCall { action, .. } => action,
                },
            })),
            error: None,
            latency_ms: 0,
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

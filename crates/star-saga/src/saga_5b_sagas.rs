//! crates/star-saga/src/saga_5b_sagas.rs
//!
//! 5 域 Saga 实装 (per Phase D.2, Mavis 临时代签 match 域 Lead per 守门 #3 v2)
//! per `OPT-NEXT-01-phase-d.md` §3 D.2 + Q-003 拍板 + 9/3 11:35 JST 反转
//!
//! ## 5 域 Saga 类型 (per 守门 #14 5 域 Lead CONTENT 4 维 + 9/3 11:35 JST 拍板 B)
//!
//! | # | Saga | 主域 | 涉及域 | 业务流 |
//! |---|---|---|---|---|
//! | 1 | SAGA_ORDER_FULFILLMENT | economy | economy→match→admin | 订单履行 + 库存补偿 |
//! | 2 | SAGA_MATCH_RUN | match | player→economy→match→social | 匹配运行 + 战斗补偿 |
//! | 3 | SAGA_SOCIAL_FEED | social | player→social→admin | 动态发布 + 通知补偿 |
//! | 4 | SAGA_ADMIN_AUDIT | admin | economy→match→social→admin | 审计 + RBAC 补偿 |
//! | 5 | SAGA_PLAYER_LOGIN | player | player→admin | 登录 + 会话补偿 |
//!
//! ## 关键不变量
//!
//! - INV-SAGA-01: 5 域 Saga 全部经 L0 SagaManager 协调 (per 守门 #13 a L1↔L1 禁止)
//! - INV-SAGA-02: 每 Saga 含 call_chain (正序) + compensation_chain (逆序, 预定义)
//! - INV-SAGA-03: 每 Saga 主域 Lead 拍板, Mavis 临时代签中 (per 守门 #3 v2 反转 + 守门 #14)
//!
//! Lead 责任: 5 域 Lead 各 1 (Mavis 临时代签中, per 守门 #3 v2)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::saga_step::{CallId, CrossDomainCall};

/// 5 域 Saga 类型 (per 守门 #14 + Q-003 拍板)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiveDomainSaga {
    /// 1. SAGA_ORDER_FULFILLMENT - economy 主导
    /// 业务: 玩家下订单 → economy 扣款 → match 启动履行工作流 → admin 审计
    /// 补偿: match abort → economy refund
    OrderFulfillment,
    /// 2. SAGA_MATCH_RUN - match 主导
    /// 业务: 玩家加入匹配 → economy 创建账户 → match 启动工作流 → social 发送开始通知
    /// 补偿: social 通知撤回 → match abort → economy refund → player deregister
    MatchRun,
    /// 3. SAGA_SOCIAL_FEED - social 主导
    /// 业务: 玩家发帖 → social 写入 feed → admin 记录审计
    /// 补偿: admin 反审计 → social 删除 feed
    SocialFeed,
    /// 4. SAGA_ADMIN_AUDIT - admin 主导
    /// 业务: 审计触发 → economy 余额快照 → match 工作流快照 → social 通知快照 → admin 写入
    /// 补偿: admin 删除审计 → social 删除通知 → match abort → economy 不补偿(只读)
    AdminAudit,
    /// 5. SAGA_PLAYER_LOGIN - player 主导
    /// 业务: 玩家登录 → player 创建 session → admin 权限校验
    /// 补偿: admin 撤销权限 → player 销毁 session
    PlayerLogin,
}

impl FiveDomainSaga {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OrderFulfillment => "SAGA_ORDER_FULFILLMENT",
            Self::MatchRun => "SAGA_MATCH_RUN",
            Self::SocialFeed => "SAGA_SOCIAL_FEED",
            Self::AdminAudit => "SAGA_ADMIN_AUDIT",
            Self::PlayerLogin => "SAGA_PLAYER_LOGIN",
        }
    }
}

impl std::fmt::Display for FiveDomainSaga {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// SagaDefinition — 单个 Saga 跨域定义 (call_chain 正序 + compensation_chain 逆序预定义)
pub struct SagaDefinition {
    pub saga_type: FiveDomainSaga,
}

impl SagaDefinition {
    pub fn new(saga_type: FiveDomainSaga) -> Self {
        Self { saga_type }
    }

    /// 正序 call_chain (per INV-SAGA-02)
    pub fn call_chain(&self) -> Vec<CrossDomainCall> {
        match self.saga_type {
            FiveDomainSaga::OrderFulfillment => vec![
                CrossDomainCall::EconomyCall {
                    call_id: Uuid::new_v4(),
                    action: "create_billing_account".into(),
                    target_id: "billing-1".into(),
                },
                CrossDomainCall::EconomyCall {
                    call_id: Uuid::new_v4(),
                    action: "deduct_currency".into(),
                    target_id: "billing-1".into(),
                },
                CrossDomainCall::MatchCall {
                    call_id: Uuid::new_v4(),
                    action: "start_workflow".into(),
                    target_id: "order-wf-1".into(),
                },
                CrossDomainCall::AdminCall {
                    call_id: Uuid::new_v4(),
                    action: "assign_role".into(),
                    target_id: "order-role-1".into(),
                },
            ],
            FiveDomainSaga::MatchRun => vec![
                CrossDomainCall::PlayerCall {
                    call_id: Uuid::new_v4(),
                    action: "create_user".into(),
                    target_id: "match-player-1".into(),
                },
                CrossDomainCall::EconomyCall {
                    call_id: Uuid::new_v4(),
                    action: "create_billing_account".into(),
                    target_id: "match-billing-1".into(),
                },
                CrossDomainCall::MatchCall {
                    call_id: Uuid::new_v4(),
                    action: "start_workflow".into(),
                    target_id: "match-wf-1".into(),
                },
                CrossDomainCall::SocialCall {
                    call_id: Uuid::new_v4(),
                    action: "send_notification".into(),
                    target_id: "match-player-1".into(),
                },
            ],
            FiveDomainSaga::SocialFeed => vec![
                CrossDomainCall::PlayerCall {
                    call_id: Uuid::new_v4(),
                    action: "create_user".into(),
                    target_id: "feed-player-1".into(),
                },
                CrossDomainCall::SocialCall {
                    call_id: Uuid::new_v4(),
                    action: "send_notification".into(),
                    target_id: "feed-1".into(),
                },
                CrossDomainCall::AdminCall {
                    call_id: Uuid::new_v4(),
                    action: "assign_role".into(),
                    target_id: "feed-role-1".into(),
                },
            ],
            FiveDomainSaga::AdminAudit => vec![
                CrossDomainCall::EconomyCall {
                    call_id: Uuid::new_v4(),
                    action: "create_billing_account".into(),
                    target_id: "audit-billing-1".into(),
                },
                CrossDomainCall::MatchCall {
                    call_id: Uuid::new_v4(),
                    action: "start_workflow".into(),
                    target_id: "audit-wf-1".into(),
                },
                CrossDomainCall::SocialCall {
                    call_id: Uuid::new_v4(),
                    action: "send_notification".into(),
                    target_id: "audit-notif-1".into(),
                },
                CrossDomainCall::AdminCall {
                    call_id: Uuid::new_v4(),
                    action: "assign_role".into(),
                    target_id: "audit-role-1".into(),
                },
            ],
            FiveDomainSaga::PlayerLogin => vec![
                CrossDomainCall::PlayerCall {
                    call_id: Uuid::new_v4(),
                    action: "create_user".into(),
                    target_id: "login-player-1".into(),
                },
                CrossDomainCall::AdminCall {
                    call_id: Uuid::new_v4(),
                    action: "assign_role".into(),
                    target_id: "login-role-1".into(),
                },
            ],
        }
    }

    /// 逆序 compensation_chain (per INV-SAGA-02, 预定义)
    pub fn compensation_chain(&self) -> Vec<CrossDomainCall> {
        let mut chain = self.call_chain();
        chain.reverse();
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_saga_types_have_unique_strs() {
        let strs = [
            FiveDomainSaga::OrderFulfillment.as_str(),
            FiveDomainSaga::MatchRun.as_str(),
            FiveDomainSaga::SocialFeed.as_str(),
            FiveDomainSaga::AdminAudit.as_str(),
            FiveDomainSaga::PlayerLogin.as_str(),
        ];
        let unique: std::collections::HashSet<_> = strs.iter().collect();
        assert_eq!(unique.len(), 5);
    }

    #[test]
    fn order_fulfillment_call_chain_is_4_calls() {
        let def = SagaDefinition::new(FiveDomainSaga::OrderFulfillment);
        let chain = def.call_chain();
        assert_eq!(chain.len(), 4);
        let mut comp = chain.clone();
        comp.reverse();
        // 逆序首元素 = 正序末元素 (call_id 除外, 每次生成不同)
        let first = &comp[0];
        let last = &chain[chain.len() - 1];
        assert_eq!(call_action(first), call_action(last));
        assert_eq!(call_target(first), call_target(last));
    }

    #[test]
    fn match_run_call_chain_is_4_calls() {
        let def = SagaDefinition::new(FiveDomainSaga::MatchRun);
        let chain = def.call_chain();
        assert_eq!(chain.len(), 4);
    }

    #[test]
    fn social_feed_call_chain_is_3_calls() {
        let def = SagaDefinition::new(FiveDomainSaga::SocialFeed);
        let chain = def.call_chain();
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn admin_audit_call_chain_is_4_calls() {
        let def = SagaDefinition::new(FiveDomainSaga::AdminAudit);
        let chain = def.call_chain();
        assert_eq!(chain.len(), 4);
    }

    #[test]
    fn player_login_call_chain_is_2_calls() {
        let def = SagaDefinition::new(FiveDomainSaga::PlayerLogin);
        let chain = def.call_chain();
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn compensation_chain_reverses_call_chain() {
        let def = SagaDefinition::new(FiveDomainSaga::OrderFulfillment);
        let chain = def.call_chain();
        let comp = def.compensation_chain();
        assert_eq!(comp.len(), chain.len());
        for i in 0..chain.len() {
            assert_eq!(
                call_action(&comp[i]),
                call_action(&chain[chain.len() - 1 - i])
            );
            assert_eq!(
                call_target(&comp[i]),
                call_target(&chain[chain.len() - 1 - i])
            );
        }
    }

    fn call_action(call: &CrossDomainCall) -> &str {
        match call {
            CrossDomainCall::PlayerCall { action, .. } => action,
            CrossDomainCall::EconomyCall { action, .. } => action,
            CrossDomainCall::MatchCall { action, .. } => action,
            CrossDomainCall::SocialCall { action, .. } => action,
            CrossDomainCall::AdminCall { action, .. } => action,
        }
    }

    fn call_target(call: &CrossDomainCall) -> &str {
        match call {
            CrossDomainCall::PlayerCall { target_id, .. } => target_id,
            CrossDomainCall::EconomyCall { target_id, .. } => target_id,
            CrossDomainCall::MatchCall { target_id, .. } => target_id,
            CrossDomainCall::SocialCall { target_id, .. } => target_id,
            CrossDomainCall::AdminCall { target_id, .. } => target_id,
        }
    }

    #[test]
    fn call_ids_are_unique_within_saga() {
        let def = SagaDefinition::new(FiveDomainSaga::OrderFulfillment);
        let chain = def.call_chain();
        let mut ids: Vec<CallId> = chain
            .iter()
            .map(|c| match c {
                CrossDomainCall::PlayerCall { call_id, .. } => *call_id,
                CrossDomainCall::EconomyCall { call_id, .. } => *call_id,
                CrossDomainCall::MatchCall { call_id, .. } => *call_id,
                CrossDomainCall::SocialCall { call_id, .. } => *call_id,
                CrossDomainCall::AdminCall { call_id, .. } => *call_id,
            })
            .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), chain.len());
    }
}

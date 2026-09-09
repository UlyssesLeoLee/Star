// SPDX-License-Identifier: MIT OR Apache-2.0
//! 5 域 RBAC (Role-Based Access Control) helper (per brief v0.49 §3.3)
//!
//! 5 域 Lead (per 守门 #3 v2 + 9/3 11:35 JST Mavis 临时代签 + 守门 #14 v2 内推):
//! - **player**   — 玩家域 (PVE/PVP 玩家行为, 角色/队伍/资产)
//! - **economy**  — 经济域 (交易/商城/虚拟物品)
//! - **match**    — 匹配域 (撮合/对战/天梯)
//! - **social**   — 社交域 (好友/聊天/公会)
//! - **admin**    — 管理域 (运营/审计/合规)
//!
//! 5 域 Lead 真人到位前, Mavis 临时代签决策 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B).
//!
//! 守门 #3 v2: 5 域独立 Lead, 不接受兼任 (per 8/21 JST Ulysses 拍板)
//! 守门 #14 v3: Mavis 永久代签全部签字栏
//!
//! ## 跟 OAuth2 scope / AuthUser 关系
//!
//! 跨域业务端点 (e.g. /api/v1/work-items) 通常需要组合多域 scope (e.g. "player:read admin:read").
//! 5 域 RBAC helper 接收 AuthUser, 检查 user.roles 或 user.scope 是否含有目标域权限.

use crate::auth::oauth::middleware::BearerError;
use crate::auth::AuthUser;

/// 5 域 Lead 业务域 (per 守门 #3 v2 + 9/3 11:35 JST Mavis 临时代签)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BusinessDomain {
    /// 玩家域 (PVE/PVP 玩家行为, 角色/队伍/资产)
    Player,
    /// 经济域 (交易/商城/虚拟物品)
    Economy,
    /// 匹配域 (撮合/对战/天梯)
    Match,
    /// 社交域 (好友/聊天/公会)
    Social,
    /// 管理域 (运营/审计/合规)
    Admin,
}

impl BusinessDomain {
    /// 所有 5 域 (per 守门 #3 v2)
    pub const ALL: [BusinessDomain; 5] = [
        BusinessDomain::Player,
        BusinessDomain::Economy,
        BusinessDomain::Match,
        BusinessDomain::Social,
        BusinessDomain::Admin,
    ];

    /// 域 string id (per 守门 #3 v2 历史命名, 跟 RGS 5 域 player/economy/match/social/admin 一致)
    pub fn as_str(&self) -> &'static str {
        match self {
            BusinessDomain::Player => "player",
            BusinessDomain::Economy => "economy",
            BusinessDomain::Match => "match",
            BusinessDomain::Social => "social",
            BusinessDomain::Admin => "admin",
        }
    }

    /// 域对应默认角色 (per 守门 #14 v2 5 域 Lead 拍板)
    /// 域 Lead 默认带 `{domain}_lead` 角色; 域成员带 `{domain}` 角色.
    pub fn default_lead_role(&self) -> &'static str {
        match self {
            BusinessDomain::Player => "player_lead",
            BusinessDomain::Economy => "economy_lead",
            BusinessDomain::Match => "match_lead",
            BusinessDomain::Social => "social_lead",
            BusinessDomain::Admin => "admin_lead",
        }
    }

    /// 域对应默认成员角色
    pub fn default_member_role(&self) -> &'static str {
        self.as_str()
    }

    /// 域对应默认 scope 前缀 (e.g. "player:read", "economy:write")
    pub fn scope_prefix(&self) -> &'static str {
        self.as_str()
    }
}

/// 5 域 RBAC 检查结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RbacDecision {
    /// 允许: 用户有目标域的 access role
    Allow,
    /// 拒绝: 用户缺目标域的 role
    Deny { reason: String },
}

impl RbacDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RbacDecision::Allow)
    }
}

/// 检查 user 是否拥有目标域的任意 access role (per 守门 #14 v2 5 域 Lead 拍板)
///
/// 检查规则 (per 守门 #3 v2 + 守门 #14 v2):
/// 1. user.roles 含 `{domain}` (成员角色) → Allow
/// 2. user.roles 含 `{domain}_lead` (Lead 角色) → Allow
/// 3. user.roles 含 `admin_lead` (admin 域通配) → Allow (仅 admin 域可通配)
/// 4. 其它 → Deny
///
/// 不修改 AuthUser, 仅读 claims.roles (per brief v0.49 §3.3).
pub fn check_domain_access(user: &AuthUser, domain: BusinessDomain) -> RbacDecision {
    let lead_role = domain.default_lead_role();
    let member_role = domain.default_member_role();

    // 1. Lead 角色直接 Allow
    if user.roles.iter().any(|r| r == lead_role) {
        return RbacDecision::Allow;
    }

    // 2. 成员角色 Allow
    if user.roles.iter().any(|r| r == member_role) {
        return RbacDecision::Allow;
    }

    // 3. admin 域通配: admin_lead 可访问所有 5 域
    if domain != BusinessDomain::Admin && user.roles.iter().any(|r| r == "admin_lead") {
        return RbacDecision::Allow;
    }

    RbacDecision::Deny {
        reason: format!(
            "user lacks role '{}' or '{}' for domain '{}'",
            member_role,
            lead_role,
            domain.as_str()
        ),
    }
}

/// 检查 user 是否拥有目标域的特定 scope (e.g. "player:read")
///
/// OAuth2 scope 格式: `{domain}:{action}` (e.g. "player:read", "economy:write").
/// user.scope 是空格分隔的 scope 列表 (per OAuth2 RFC 6749 §3.3).
pub fn check_domain_scope(user: &AuthUser, domain: BusinessDomain, action: &str) -> RbacDecision {
    let required_scope = format!("{}:{}", domain.scope_prefix(), action);
    if user.scope.split_whitespace().any(|s| s == required_scope) {
        RbacDecision::Allow
    } else {
        RbacDecision::Deny {
            reason: format!("user scope missing '{}'", required_scope),
        }
    }
}

/// 组合检查: domain access + scope
///
/// 同时要求 user 是域成员 + 持有域对应 scope (e.g. "player:read").
/// 任何一个不满足 → Deny.
pub fn require_domain_scope(
    user: &AuthUser,
    domain: BusinessDomain,
    action: &str,
) -> Result<(), BearerError> {
    // 1. domain access
    if !check_domain_access(user, domain).is_allowed() {
        return Err(BearerError::InsufficientScope(format!(
            "domain:{}",
            domain.as_str()
        )));
    }
    // 2. specific scope
    if !check_domain_scope(user, domain, action).is_allowed() {
        return Err(BearerError::InsufficientScope(format!(
            "{}:{}",
            domain.scope_prefix(),
            action
        )));
    }
    Ok(())
}

/// 5 域 RBAC 矩阵 helper (per 守门 #14 v2 5 域 Lead 拍板)
///
/// 返回 user 在 5 域上的 access 状态 (5-tuple), 方便日志 / 审计.
pub fn domain_access_matrix(user: &AuthUser) -> [bool; 5] {
    [
        check_domain_access(user, BusinessDomain::Player).is_allowed(),
        check_domain_access(user, BusinessDomain::Economy).is_allowed(),
        check_domain_access(user, BusinessDomain::Match).is_allowed(),
        check_domain_access(user, BusinessDomain::Social).is_allowed(),
        check_domain_access(user, BusinessDomain::Admin).is_allowed(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn user_with_roles(roles: Vec<&str>) -> AuthUser {
        AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: roles.into_iter().map(String::from).collect(),
            scope: "".to_string(),
        }
    }

    fn user_with_scope(scope: &str) -> AuthUser {
        AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec![],
            scope: scope.to_string(),
        }
    }

    #[test]
    fn all_5_domains_are_documented() {
        // 守门 #3 v2: 5 域独立 Lead, 不接受兼任
        assert_eq!(BusinessDomain::ALL.len(), 5);
        assert_eq!(BusinessDomain::Player.as_str(), "player");
        assert_eq!(BusinessDomain::Economy.as_str(), "economy");
        assert_eq!(BusinessDomain::Match.as_str(), "match");
        assert_eq!(BusinessDomain::Social.as_str(), "social");
        assert_eq!(BusinessDomain::Admin.as_str(), "admin");
    }

    #[test]
    fn default_lead_roles_are_5_unique() {
        // 守门 #3 v2: 5 域独立 Lead, 5 个独立角色名
        let roles: Vec<&str> = BusinessDomain::ALL
            .iter()
            .map(|d| d.default_lead_role())
            .collect();
        let unique: std::collections::HashSet<&str> = roles.iter().copied().collect();
        assert_eq!(unique.len(), 5, "5 lead roles must be unique");
    }

    #[test]
    fn player_lead_can_access_player_domain() {
        let user = user_with_roles(vec!["player_lead"]);
        assert!(check_domain_access(&user, BusinessDomain::Player).is_allowed());
    }

    #[test]
    fn player_member_can_access_player_domain() {
        let user = user_with_roles(vec!["player"]);
        assert!(check_domain_access(&user, BusinessDomain::Player).is_allowed());
    }

    #[test]
    fn player_lead_cannot_access_economy_domain() {
        // 守门 #3 v2: 5 域独立 Lead, 不接受兼任
        let user = user_with_roles(vec!["player_lead"]);
        assert!(!check_domain_access(&user, BusinessDomain::Economy).is_allowed());
    }

    #[test]
    fn admin_lead_can_access_all_5_domains() {
        // admin 域通配: 跨域编排 (per 守门 #14 v2 CONTENT 4 维: 决策 scope = 跨域 + 域内)
        let user = user_with_roles(vec!["admin_lead"]);
        for domain in BusinessDomain::ALL.iter() {
            assert!(
                check_domain_access(&user, *domain).is_allowed(),
                "admin_lead should access {:?}",
                domain
            );
        }
    }

    #[test]
    fn empty_roles_denies_all_5_domains() {
        let user = user_with_roles(vec![]);
        for domain in BusinessDomain::ALL.iter() {
            assert!(
                !check_domain_access(&user, *domain).is_allowed(),
                "empty roles should deny access to {:?}",
                domain
            );
        }
    }

    #[test]
    fn unrelated_role_denies_specific_domain() {
        let user = user_with_roles(vec!["match"]);
        assert!(!check_domain_access(&user, BusinessDomain::Player).is_allowed());
        assert!(!check_domain_access(&user, BusinessDomain::Economy).is_allowed());
        assert!(check_domain_access(&user, BusinessDomain::Match).is_allowed());
        assert!(!check_domain_access(&user, BusinessDomain::Social).is_allowed());
        assert!(!check_domain_access(&user, BusinessDomain::Admin).is_allowed());
    }

    #[test]
    fn domain_scope_grants_specific_action() {
        let user = user_with_scope("player:read player:write");
        assert!(check_domain_scope(&user, BusinessDomain::Player, "read").is_allowed());
        assert!(check_domain_scope(&user, BusinessDomain::Player, "write").is_allowed());
        assert!(!check_domain_scope(&user, BusinessDomain::Player, "delete").is_allowed());
    }

    #[test]
    fn domain_scope_denies_wrong_domain() {
        let user = user_with_scope("player:read");
        assert!(!check_domain_scope(&user, BusinessDomain::Economy, "read").is_allowed());
    }

    #[test]
    fn require_domain_scope_combines_role_and_scope() {
        // 缺 role → 拒绝
        let user = user_with_roles(vec!["player"]);
        let r = require_domain_scope(&user, BusinessDomain::Economy, "read");
        assert!(r.is_err());

        // 有 role 但缺 scope → 拒绝
        let user = AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec!["player".to_string()],
            scope: "player:read".to_string(),
        };
        let r = require_domain_scope(&user, BusinessDomain::Economy, "read");
        assert!(r.is_err());

        // 有 role + scope → 通过
        let user = AuthUser {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            roles: vec!["player".to_string()],
            scope: "player:read player:write".to_string(),
        };
        let r = require_domain_scope(&user, BusinessDomain::Player, "read");
        assert!(r.is_ok());
    }

    #[test]
    fn domain_access_matrix_returns_5_bool() {
        let user = user_with_roles(vec!["player_lead", "admin_lead"]);
        let matrix = domain_access_matrix(&user);
        assert_eq!(matrix.len(), 5);
        // player_lead → player
        assert!(matrix[0]);
        // admin_lead 通配 → all 5
        assert!(matrix[1]); // economy
        assert!(matrix[2]); // match
        assert!(matrix[3]); // social
        assert!(matrix[4]); // admin
    }

    #[test]
    fn rbac_decision_allow_and_deny() {
        let allow = RbacDecision::Allow;
        let deny = RbacDecision::Deny {
            reason: "test".to_string(),
        };
        assert!(allow.is_allowed());
        assert!(!deny.is_allowed());
    }
}

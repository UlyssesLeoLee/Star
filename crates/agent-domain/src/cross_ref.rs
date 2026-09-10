//! A9.1-A9.2 cross_ref (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `CrossRef` 表示 agent 跨域引用 (per BD-CANVAS-AGENT-001 §3 A9.1 link 到 SRS-AGENT-VIEW 协同 + A9.2 link 到 SRS-AGENT-RELATIONSHIP 协同).
//! 主要业务方法: 链接到 view/relationship, 0 外部 I/O (纯 in-memory URL 拼接).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A9.1-A9.2 跨域引用 view 类型 (per BD-CANVAS-AGENT-001 §3 A9.1-A9.2 协同).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossRefTarget {
    /// link 到 SRS-AGENT-VIEW (per A9.1 双击跳转 `/agent-view?agent={id}`)
    AgentView,
    /// link 到 SRS-AGENT-RELATIONSHIP (per A9.2 tab 切到 `/agent-relationships?agent={id}`)
    AgentRelationship,
}

/// `agent-domain` cross_ref 错误.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossRefError {
    /// agent_id 跟 view 自身 id 一致时 拒绝自引 (per A9.1 双击不引到自身)
    SelfRef,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A9.1-A9.2 cross_ref URL 生成器 (per BD-CANVAS-AGENT-001 §3 A9).
///
/// 字段: tenant_id (守门 #13 a) + base_url (frontend host, e.g. https://star.local).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRef {
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// frontend base URL (e.g. https://star.local)
    pub base_url: String,
}

impl CrossRef {
    /// A9.1 业务方法: 创建 cross_ref URL 生成器.
    pub fn new(tenant_id: Uuid, base_url: impl Into<String>) -> Result<Self, CrossRefError> {
        if tenant_id.is_nil() {
            return Err(CrossRefError::TenantIdRequired);
        }
        Ok(Self {
            tenant_id,
            base_url: base_url.into(),
        })
    }

    /// A9.1 业务方法: 拼接 link URL (per BD §3 A9.1-A9.2 `/agent-view?agent={id}` / `/agent-relationships?agent={id}`).
    ///
    /// 守门: agent_id 不为 nil Uuid (虽然 nil 仍可生成 URL, 但业务上不应该出现, 显式 self-ref 拒绝).
    pub fn link(&self, target: CrossRefTarget, agent_id: Uuid) -> Result<String, CrossRefError> {
        if agent_id.is_nil() {
            return Err(CrossRefError::SelfRef);
        }
        let path = match target {
            CrossRefTarget::AgentView => "/agent-view",
            CrossRefTarget::AgentRelationship => "/agent-relationships",
        };
        Ok(format!("{}{}?agent={}", self.base_url, path, agent_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossref_new_ok() {
        let tenant = Uuid::new_v4();
        let c = CrossRef::new(tenant, "https://star.local").expect("valid");
        assert_eq!(c.tenant_id, tenant);
        assert_eq!(c.base_url, "https://star.local");
    }

    #[test]
    fn test_crossref_new_tenant_required() {
        let result = CrossRef::new(Uuid::nil(), "https://star.local");
        assert_eq!(result.err(), Some(CrossRefError::TenantIdRequired));
    }

    #[test]
    fn test_crossref_link_to_view() {
        let tenant = Uuid::new_v4();
        let c = CrossRef::new(tenant, "https://star.local").unwrap();
        let agent = Uuid::new_v4();
        let url = c.link(CrossRefTarget::AgentView, agent).expect("valid");
        assert_eq!(
            url,
            format!("https://star.local/agent-view?agent={}", agent)
        );
    }

    #[test]
    fn test_crossref_link_to_relationship() {
        let tenant = Uuid::new_v4();
        let c = CrossRef::new(tenant, "https://star.local").unwrap();
        let agent = Uuid::new_v4();
        let url = c
            .link(CrossRefTarget::AgentRelationship, agent)
            .expect("valid");
        assert_eq!(
            url,
            format!("https://star.local/agent-relationships?agent={}", agent)
        );
    }

    #[test]
    fn test_crossref_link_self_ref() {
        let tenant = Uuid::new_v4();
        let c = CrossRef::new(tenant, "https://star.local").unwrap();
        let result = c.link(CrossRefTarget::AgentView, Uuid::nil());
        assert_eq!(result.err(), Some(CrossRefError::SelfRef));
    }
}

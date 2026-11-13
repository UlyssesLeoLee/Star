/*
@cypher schema=1 source_sha256=4539a9f02472ce70cb97ef06356722b5e808dfa0bbc11e622ba55f7f9ba7d715
MERGE (self:File {path:"crates/star-agent-runtime/src/identity.rs"})
@endcypher
*/
//! AgentIdentity (per SRS §9 REQ-ECS-001 + 03-DD §2.1)。

use std::time::SystemTime;

use crate::agent_type::AgentType;
use crate::ecs::Component;
use crate::id::{AgentId, TenantId};

/// AgentIdentity Component: 唯一 ID + tenant + agent_type。
#[derive(Debug, Clone)]
pub struct AgentIdentity {
    /// Agent ID。
    pub agent_id: AgentId,
    /// Tenant ID。
    pub tenant_id: TenantId,
    /// SA-01..SA-09 类型 (per LangGraph 9/3 §6.1)。
    pub agent_type: AgentType,
    /// 创建时间。
    pub created_at: SystemTime,
}

impl Component for AgentIdentity {}

impl AgentIdentity {
    /// 创建 (created_at = now)。
    pub fn new(tenant_id: TenantId, agent_type: AgentType) -> Self {
        Self {
            agent_id: AgentId::new(),
            tenant_id,
            agent_type,
            created_at: SystemTime::now(),
        }
    }

    /// 显式 agent_id (恢复场景, per SRS §54 Crash Recovery 雏形)。
    pub fn with_id(agent_id: AgentId, tenant_id: TenantId, agent_type: AgentType) -> Self {
        Self {
            agent_id,
            tenant_id,
            agent_type,
            created_at: SystemTime::now(),
        }
    }
}

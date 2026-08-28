// crates/star-mcp/src/handlers/agent.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-agent handler — 真实数据接入 (Phase B.2.6 Tier 3)
//!
//! URI: `agent://{uuid}` — Agent (id / display_name / agent_type / version / status)
//! Cache TTL: 5s (per `spec/cache/01` §4 L135 agent state 5s)
//! 真实数据源: `crates/domain-agent::InMemoryAgentService` (lib.rs line 694)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use domain_agent::{AgentError, AgentId, AgentRepository, InMemoryAgentRepository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentData {
    pub agent_id: String,
    pub tenant_id: String,
    pub agent_type: String,
    pub provider: String,
    pub version: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) struct AgentHandler {
    svc: OnceLock<Arc<InMemoryAgentRepository>>,
}

impl Default for AgentHandler {
    fn default() -> Self {
        Self {
            svc: OnceLock::new(),
        }
    }
}

impl AgentHandler {
    pub fn new() -> Self {
        Self::default()
    }
    fn service(&self) -> &Arc<InMemoryAgentRepository> {
        self.svc
            .get_or_init(|| Arc::new(InMemoryAgentRepository::new()))
    }
}

#[async_trait]
impl Resource for AgentHandler {
    type Data = AgentData;
    fn uri_pattern(&self) -> &str {
        "agent://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        let _key = KeyBuilder::for_resource("agent", id);
        let agent_id = AgentId::from(
            uuid::Uuid::parse_str(id).map_err(|e| ResourceError::InvalidUri(e.to_string()))?,
        );
        let svc = self.service();
        // 直接走 repository (无 tenant 校验), agent handler 简化设计
        match svc.get_agent(agent_id).await {
            Ok(a) => Ok(Some(AgentData {
                agent_id: a.id.to_string(),
                tenant_id: a.tenant_id.to_string(),
                agent_type: format!("{:?}", a.agent_type),
                provider: a.provider,
                version: a.version,
                enabled: a.enabled,
                created_at: a.created_at.timestamp(),
                updated_at: a.updated_at.timestamp(),
            })),
            Err(AgentError::NotFound(_)) => Ok(None),
            Err(e) => Err(ResourceError::Internal(e.to_string())),
        }
    }
    fn cache_ttl_sec(&self) -> u32 {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_agent::{Agent, AgentType};

    #[tokio::test]
    async fn read_test_invalid_uuid() {
        let h = AgentHandler::new();
        let d = h.read("not-a-uuid").await;
        assert!(d.is_err());
    }

    #[tokio::test]
    async fn read_real_agent_roundtrip() {
        let h = AgentHandler::new();
        let svc = h.service();
        let aid = AgentId::new();
        let agent = Agent {
            id: aid,
            tenant_id: domain_agent::TenantId::new(),
            agent_type: AgentType::Codex,
            provider: "openai".into(),
            version: "1.0.0".into(),
            capabilities: vec!["code".into(), "test".into()],
            policy_template_id: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        svc.insert_agent(agent.clone()).await.unwrap();
        let d = h.read(&aid.to_string()).await.unwrap().unwrap();
        assert_eq!(d.agent_id, aid.to_string());
        assert_eq!(d.provider, "openai");
        assert!(d.enabled);
    }

    #[tokio::test]
    async fn read_not_found_returns_none() {
        let h = AgentHandler::new();
        let _ = h.service();
        let missing = uuid::Uuid::new_v4();
        let d = h.read(&missing.to_string()).await.unwrap();
        assert!(d.is_none());
    }
}

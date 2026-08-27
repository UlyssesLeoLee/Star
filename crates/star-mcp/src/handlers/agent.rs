// crates/star-mcp/src/handlers/agent.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-agent handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `agent://{id}` — agent session 状态 (per ADR-0030 Lease + Heartbeat)
//! Cache TTL: 5s (per `spec/cache/01` §4 L136 — heartbeat 频繁, 30s heartbeat × 6 = 5s 内必有变化)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-agent`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentData {
    pub agent_id: String,
    pub state: String,
    pub session_id: String,
    pub created_at: i64,
}

pub(crate) struct AgentHandler;

#[async_trait]
impl Resource for AgentHandler {
    type Data = AgentData;
    fn uri_pattern(&self) -> &str {
        "agent://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-agent 真实数据
        let _key = KeyBuilder::for_resource("agent", id);
        Ok(Some(AgentData {
            agent_id: id.into(),
            state: "Running".into(),
            session_id: format!("sess-{id}"),
            created_at: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        5 // per spec/cache/01 §4 L136 agent state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = AgentHandler;
        let d = h.read("agent-1").await.unwrap();
        assert_eq!(d.unwrap().agent_id, "agent-1");
    }
}

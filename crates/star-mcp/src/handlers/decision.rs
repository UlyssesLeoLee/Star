// crates/star-mcp/src/handlers/decision.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-decision handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `decision://{id}` — 决策记录 (per spec/flows/02 Decision schema)
//! Cache TTL: 60s (per `spec/cache/01` §4 L140 — Active 决策 1 分钟, 高一致性)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-decision`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DecisionData {
    pub dec_id: String,
    pub status: String,
    pub superseded_by: Option<String>,
    pub created_at: i64,
}

pub(crate) struct DecisionHandler;

#[async_trait]
impl Resource for DecisionHandler {
    type Data = DecisionData;
    fn uri_pattern(&self) -> &str {
        "decision://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-decision 真实数据
        let _key = KeyBuilder::for_resource("decision", id);
        Ok(Some(DecisionData {
            dec_id: id.into(),
            status: "Active".into(),
            superseded_by: None,
            created_at: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        60 // per spec/cache/01 §4 L140 decision (Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = DecisionHandler;
        let d = h.read("dec-1").await.unwrap();
        assert_eq!(d.unwrap().dec_id, "dec-1");
    }
}

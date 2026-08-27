// crates/star-mcp/src/handlers/integration.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-integration handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `integration://{id}` — 集成端点 (source / target / status)
//! Cache TTL: 300s (per `spec/cache/01` §4 — 中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-integration`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IntegrationData {
    pub int_id: String,
    pub source: String,
    pub target: String,
    pub status: String,
}

pub(crate) struct IntegrationHandler;

#[async_trait]
impl Resource for IntegrationHandler {
    type Data = IntegrationData;
    fn uri_pattern(&self) -> &str {
        "integration://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-integration 真实数据
        let _key = KeyBuilder::for_resource("integration", id);
        Ok(Some(IntegrationData {
            int_id: id.into(),
            source: "github".into(),
            target: "star-mcp".into(),
            status: "active".into(),
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        300
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = IntegrationHandler;
        let d = h.read("int-1").await.unwrap();
        assert_eq!(d.unwrap().int_id, "int-1");
    }
}

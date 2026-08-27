// crates/star-mcp/src/handlers/context.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-context handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `context://{id}` — 上下文数据包 (per spec/context/01)
//! Cache TTL: 300s (中频, 类似 event 实时性)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-context`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ContextData {
    pub ctx_id: String,
    pub packet: String,
    pub priority: u32,
    pub created_at: i64,
}

pub(crate) struct ContextHandler;

#[async_trait]
impl Resource for ContextHandler {
    type Data = ContextData;
    fn uri_pattern(&self) -> &str {
        "context://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-context 真实数据
        let _key = KeyBuilder::for_resource("context", id);
        Ok(Some(ContextData {
            ctx_id: id.into(),
            packet: "mock packet".into(),
            priority: 1,
            created_at: 0,
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
        let h = ContextHandler;
        let d = h.read("ctx-1").await.unwrap();
        assert_eq!(d.unwrap().ctx_id, "ctx-1");
    }
}

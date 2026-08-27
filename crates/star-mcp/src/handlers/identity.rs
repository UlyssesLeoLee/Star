// crates/star-mcp/src/handlers/identity.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-identity handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `identity://{id}` — 用户身份 (email / role / tenant_id)
//! Cache TTL: 3600s (per `spec/cache/01` §4 — 1 小时, 变化少)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-identity`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IdentityData {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub tenant_id: String,
}

pub(crate) struct IdentityHandler;

#[async_trait]
impl Resource for IdentityHandler {
    type Data = IdentityData;
    fn uri_pattern(&self) -> &str {
        "identity://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-identity 真实数据
        let _key = KeyBuilder::for_resource("identity", id);
        Ok(Some(IdentityData {
            user_id: id.into(),
            email: format!("{id}@example.invalid"),
            role: "developer".into(),
            tenant_id: "tenant-1".into(),
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = IdentityHandler;
        let d = h.read("user-1").await.unwrap();
        assert_eq!(d.unwrap().user_id, "user-1");
    }
}

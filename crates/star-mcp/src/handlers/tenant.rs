// crates/star-mcp/src/handlers/tenant.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-tenant handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `tenant://{id}` — 租户 (name / plan / created_at)
//! Cache TTL: 86400s (per `spec/cache/01` §4 — 24h, 类似 audit append-only)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-tenant`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TenantData {
    pub tenant_id: String,
    pub name: String,
    pub plan: String,
    pub created_at: i64,
}

pub(crate) struct TenantHandler;

#[async_trait]
impl Resource for TenantHandler {
    type Data = TenantData;
    fn uri_pattern(&self) -> &str {
        "tenant://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-tenant 真实数据
        let _key = KeyBuilder::for_resource("tenant", id);
        Ok(Some(TenantData {
            tenant_id: id.into(),
            name: format!("Tenant {id} (mock)"),
            plan: "free".into(),
            created_at: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        86400
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = TenantHandler;
        let d = h.read("tenant-1").await.unwrap();
        assert_eq!(d.unwrap().tenant_id, "tenant-1");
    }
}

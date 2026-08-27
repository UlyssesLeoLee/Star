// crates/star-mcp/src/handlers/permission.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-permission handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `permission://{id}` — 权限规则 (role / resource / action)
//! Cache TTL: 3600s (per `spec/cache/01` §4 — 1 小时, 变化少)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-permission`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PermissionData {
    pub rule_id: String,
    pub role: String,
    pub resource: String,
    pub action: String,
}

pub(crate) struct PermissionHandler;

#[async_trait]
impl Resource for PermissionHandler {
    type Data = PermissionData;
    fn uri_pattern(&self) -> &str {
        "permission://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-permission 真实数据
        let _key = KeyBuilder::for_resource("permission", id);
        Ok(Some(PermissionData {
            rule_id: id.into(),
            role: "developer".into(),
            resource: "work_item".into(),
            action: "read".into(),
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
        let h = PermissionHandler;
        let d = h.read("rule-1").await.unwrap();
        assert_eq!(d.unwrap().rule_id, "rule-1");
    }
}

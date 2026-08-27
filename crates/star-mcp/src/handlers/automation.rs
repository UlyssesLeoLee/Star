// crates/star-mcp/src/handlers/automation.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-automation handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `automation://{id}` — automation 规则 (trigger / action)
//! Cache TTL: 300s (per `spec/cache/01` §4 — 中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-automation`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AutomationData {
    pub rule_id: String,
    pub trigger: String,
    pub action: String,
    pub enabled: bool,
}

pub(crate) struct AutomationHandler;

#[async_trait]
impl Resource for AutomationHandler {
    type Data = AutomationData;
    fn uri_pattern(&self) -> &str {
        "automation://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-automation 真实数据
        let _key = KeyBuilder::for_resource("automation", id);
        Ok(Some(AutomationData {
            rule_id: id.into(),
            trigger: "event.created".into(),
            action: "notify.send".into(),
            enabled: true,
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
        let h = AutomationHandler;
        let d = h.read("auto-1").await.unwrap();
        assert_eq!(d.unwrap().rule_id, "auto-1");
    }
}

// crates/star-mcp/src/handlers/notification.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-notification handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `notification://{id}` — 通知 (channel / template)
//! Cache TTL: 120s (中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-notification`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotificationData {
    pub nt_id: String,
    pub channel: String,
    pub template: String,
    pub created_at: i64,
}

pub(crate) struct NotificationHandler;

#[async_trait]
impl Resource for NotificationHandler {
    type Data = NotificationData;
    fn uri_pattern(&self) -> &str {
        "notification://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-notification 真实数据
        let _key = KeyBuilder::for_resource("notification", id);
        Ok(Some(NotificationData {
            nt_id: id.into(),
            channel: "email".into(),
            template: "default".into(),
            created_at: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        120
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = NotificationHandler;
        let d = h.read("nt-1").await.unwrap();
        assert_eq!(d.unwrap().nt_id, "nt-1");
    }
}

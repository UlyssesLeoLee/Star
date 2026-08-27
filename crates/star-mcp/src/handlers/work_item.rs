// crates/star-mcp/src/handlers/work_item.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-work-item handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `workitem://{id}` — 工作项 (title / status / assignee)
//! Cache TTL: 60s (per `spec/cache/01` §4 L150 决策 60s 类似实时性)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-work-item`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkItemData {
    pub wi_id: String,
    pub title: String,
    pub status: String,
    pub assignee: String,
}

pub(crate) struct WorkItemHandler;

#[async_trait]
impl Resource for WorkItemHandler {
    type Data = WorkItemData;
    fn uri_pattern(&self) -> &str {
        "workitem://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-work-item 真实数据
        let _key = KeyBuilder::for_resource("work_item", id);
        Ok(Some(WorkItemData {
            wi_id: id.into(),
            title: format!("Work item {id} (mock)"),
            status: "Open".into(),
            assignee: "user-1".into(),
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = WorkItemHandler;
        let d = h.read("wi-1").await.unwrap();
        assert_eq!(d.unwrap().wi_id, "wi-1");
    }
}

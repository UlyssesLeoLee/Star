// crates/star-mcp/src/handlers/feedback.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-feedback handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `feedback://{id}` — 反馈记录
//! Cache TTL: 60s (中频, 类似 work_item 实时性)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-feedback`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FeedbackData {
    pub fb_id: String,
    pub target_type: String,
    pub target_id: String,
    pub severity: String,
    pub created_at: i64,
}

pub(crate) struct FeedbackHandler;

#[async_trait]
impl Resource for FeedbackHandler {
    type Data = FeedbackData;
    fn uri_pattern(&self) -> &str {
        "feedback://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-feedback 真实数据
        let _key = KeyBuilder::for_resource("feedback", id);
        Ok(Some(FeedbackData {
            fb_id: id.into(),
            target_type: "work_item".into(),
            target_id: "wi-1".into(),
            severity: "low".into(),
            created_at: 0,
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
        let h = FeedbackHandler;
        let d = h.read("fb-1").await.unwrap();
        assert_eq!(d.unwrap().fb_id, "fb-1");
    }
}

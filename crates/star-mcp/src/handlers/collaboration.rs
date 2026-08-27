// crates/star-mcp/src/handlers/collaboration.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-collaboration handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §6 #1 协作扩展)
//!
//! URI: `collaboration://{id}` — 协作会话
//! Cache TTL: 60s (协作类, 中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-collaboration` (per spec/agents/02 §6 #1)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CollaborationData {
    pub collab_id: String,
    pub participants: Vec<String>,
    pub state: String,
    pub created_at: i64,
}

pub(crate) struct CollaborationHandler;

#[async_trait]
impl Resource for CollaborationHandler {
    type Data = CollaborationData;
    fn uri_pattern(&self) -> &str {
        "collaboration://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-collaboration 真实数据
        let _key = KeyBuilder::for_resource("collaboration", id);
        Ok(Some(CollaborationData {
            collab_id: id.into(),
            participants: vec!["user-1".into(), "user-2".into()],
            state: "Active".into(),
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
        let h = CollaborationHandler;
        let d = h.read("collab-1").await.unwrap();
        assert_eq!(d.unwrap().collab_id, "collab-1");
    }
}

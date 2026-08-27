// crates/star-mcp/src/handlers/comment.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-comment handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §6 #1 协作扩展)
//!
//! URI: `comment://{id}` — 评论
//! Cache TTL: 60s (协作类, 中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-comment` (per spec/agents/02 §6 #1)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CommentData {
    pub comment_id: String,
    pub target_type: String,
    pub target_id: String,
    pub body: String,
    pub author: String,
    pub created_at: i64,
}

pub(crate) struct CommentHandler;

#[async_trait]
impl Resource for CommentHandler {
    type Data = CommentData;
    fn uri_pattern(&self) -> &str {
        "comment://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-comment 真实数据
        let _key = KeyBuilder::for_resource("comment", id);
        Ok(Some(CommentData {
            comment_id: id.into(),
            target_type: "work_item".into(),
            target_id: "wi-1".into(),
            body: "mock comment body".into(),
            author: "user-1".into(),
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
        let h = CommentHandler;
        let d = h.read("comment-1").await.unwrap();
        assert_eq!(d.unwrap().comment_id, "comment-1");
    }
}

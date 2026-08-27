// crates/star-mcp/src/handlers/relation.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-relation handler (项目管理扩展)
//!
//! URI: `relation://{id}` — 实体间关系
//! Cache TTL: 60s (中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-relation`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RelationData {
    pub rel_id: String,
    pub from_type: String,
    pub from_id: String,
    pub to_type: String,
    pub to_id: String,
    pub kind: String,
}

pub(crate) struct RelationHandler;

#[async_trait]
impl Resource for RelationHandler {
    type Data = RelationData;
    fn uri_pattern(&self) -> &str {
        "relation://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-relation 真实数据
        let _key = KeyBuilder::for_resource("relation", id);
        Ok(Some(RelationData {
            rel_id: id.into(),
            from_type: "work_item".into(),
            from_id: "wi-1".into(),
            to_type: "work_item".into(),
            to_id: "wi-2".into(),
            kind: "blocks".into(),
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
        let h = RelationHandler;
        let d = h.read("rel-1").await.unwrap();
        assert_eq!(d.unwrap().rel_id, "rel-1");
    }
}

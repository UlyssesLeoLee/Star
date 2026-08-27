// crates/star-mcp/src/handlers/development.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-development handler (项目管理扩展)
//!
//! URI: `development://{id}` — 开发活动
//! Cache TTL: 60s (中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-development`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DevelopmentData {
    pub dev_id: String,
    pub name: String,
    pub status: String,
    pub created_at: i64,
}

pub(crate) struct DevelopmentHandler;

#[async_trait]
impl Resource for DevelopmentHandler {
    type Data = DevelopmentData;
    fn uri_pattern(&self) -> &str {
        "development://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-development 真实数据
        let _key = KeyBuilder::for_resource("development", id);
        Ok(Some(DevelopmentData {
            dev_id: id.into(),
            name: format!("Development {id} (mock)"),
            status: "Active".into(),
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
        let h = DevelopmentHandler;
        let d = h.read("dev-1").await.unwrap();
        assert_eq!(d.unwrap().dev_id, "dev-1");
    }
}

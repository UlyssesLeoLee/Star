// crates/star-mcp/src/handlers/project.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-project handler (项目管理扩展)
//!
//! URI: `project://{id}` — 项目
//! Cache TTL: 60s (中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-project`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectData {
    pub project_id: String,
    pub name: String,
    pub state: String,
    pub created_at: i64,
}

pub(crate) struct ProjectHandler;

#[async_trait]
impl Resource for ProjectHandler {
    type Data = ProjectData;
    fn uri_pattern(&self) -> &str {
        "project://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-project 真实数据
        let _key = KeyBuilder::for_resource("project", id);
        Ok(Some(ProjectData {
            project_id: id.into(),
            name: format!("Project {id} (mock)"),
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
        let h = ProjectHandler;
        let d = h.read("proj-1").await.unwrap();
        assert_eq!(d.unwrap().project_id, "proj-1");
    }
}

// crates/star-mcp/src/handlers/planning.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-planning handler (项目管理扩展)
//!
//! URI: `planning://{id}` — 计划
//! Cache TTL: 60s (中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-planning`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlanningData {
    pub plan_id: String,
    pub title: String,
    pub state: String,
    pub created_at: i64,
}

pub(crate) struct PlanningHandler;

#[async_trait]
impl Resource for PlanningHandler {
    type Data = PlanningData;
    fn uri_pattern(&self) -> &str {
        "planning://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-planning 真实数据
        let _key = KeyBuilder::for_resource("planning", id);
        Ok(Some(PlanningData {
            plan_id: id.into(),
            title: format!("Plan {id} (mock)"),
            state: "Draft".into(),
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
        let h = PlanningHandler;
        let d = h.read("plan-1").await.unwrap();
        assert_eq!(d.unwrap().plan_id, "plan-1");
    }
}

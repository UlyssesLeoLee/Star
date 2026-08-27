// crates/star-mcp/src/handlers/search.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-search handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `search://{id}` — 搜索查询 (terms / filters)
//! Cache TTL: 30s (per `spec/cache/01` §4 — 高频, 类似 worktree 列表)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-search`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SearchData {
    pub query_id: String,
    pub terms: Vec<String>,
    pub filters: Vec<String>,
    pub result_count: u32,
}

pub(crate) struct SearchHandler;

#[async_trait]
impl Resource for SearchHandler {
    type Data = SearchData;
    fn uri_pattern(&self) -> &str {
        "search://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-search 真实数据
        let _key = KeyBuilder::for_resource("search", id);
        Ok(Some(SearchData {
            query_id: id.into(),
            terms: vec!["mock".into(), "query".into()],
            filters: vec![],
            result_count: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        30
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = SearchHandler;
        let d = h.read("q-1").await.unwrap();
        assert_eq!(d.unwrap().query_id, "q-1");
    }
}

// crates/star-mcp/src/handlers/worktree.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-worktree handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `worktree://{id}` — worktree 详情 (branch / commit / path / status)
//! Cache TTL: 30s (per `spec/cache/01` §4 L135 — 30s, 实时性高)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-worktree`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorktreeData {
    pub wt_id: String,
    pub branch: String,
    pub commit: String,
    pub path: String,
    pub status: String,
}

pub(crate) struct WorktreeHandler;

#[async_trait]
impl Resource for WorktreeHandler {
    type Data = WorktreeData;
    fn uri_pattern(&self) -> &str {
        "worktree://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-worktree 真实数据
        let _key = KeyBuilder::for_resource("worktree", id);
        Ok(Some(WorktreeData {
            wt_id: id.into(),
            branch: "feat/example".into(),
            commit: "0000000".into(),
            path: format!("/tmp/{id}"),
            status: "open".into(),
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        30 // per spec/cache/01 §4 L135 worktree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = WorktreeHandler;
        let d = h.read("wt-1").await.unwrap();
        assert_eq!(d.unwrap().wt_id, "wt-1");
    }
}

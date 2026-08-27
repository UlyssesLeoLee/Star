// crates/star-mcp/src/handlers/scm.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-scm handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `scm://{id}` — SCM 仓库 (provider / repo)
//! Cache TTL: 3600s (per `spec/cache/01` §4 — 1 小时, 类似 branch 变化少)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-scm`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScmData {
    pub scm_id: String,
    pub provider: String,
    pub repo: String,
    pub default_branch: String,
}

pub(crate) struct ScmHandler;

#[async_trait]
impl Resource for ScmHandler {
    type Data = ScmData;
    fn uri_pattern(&self) -> &str {
        "scm://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-scm 真实数据
        let _key = KeyBuilder::for_resource("scm", id);
        Ok(Some(ScmData {
            scm_id: id.into(),
            provider: "git".into(),
            repo: format!("example/{id}"),
            default_branch: "main".into(),
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = ScmHandler;
        let d = h.read("scm-1").await.unwrap();
        assert_eq!(d.unwrap().scm_id, "scm-1");
    }
}

// crates/star-mcp/src/handlers/validation.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-validation handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `validation://{id}` — 验证结果 (result / kind)
//! Cache TTL: 30s (per `spec/cache/01` §4 — 30s 实时性高)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-validation`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValidationData {
    pub val_id: String,
    pub result: String,
    pub kind: String,
    pub created_at: i64,
}

pub(crate) struct ValidationHandler;

#[async_trait]
impl Resource for ValidationHandler {
    type Data = ValidationData;
    fn uri_pattern(&self) -> &str {
        "validation://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-validation 真实数据
        let _key = KeyBuilder::for_resource("validation", id);
        Ok(Some(ValidationData {
            val_id: id.into(),
            result: "pass".into(),
            kind: "ci".into(),
            created_at: 0,
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
        let h = ValidationHandler;
        let d = h.read("val-1").await.unwrap();
        assert_eq!(d.unwrap().val_id, "val-1");
    }
}

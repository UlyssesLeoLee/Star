// crates/star-mcp/src/handlers/audit.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-audit handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4)
//!
//! URI: `audit://{id}` — append-only 审计记录 (per spec/agents/02 §2)
//! Cache TTL: 86400s (per `spec/cache/01` §4 L143 — append-only, 24h, TTL 仅用于 LRU 回收)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-audit`
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuditData {
    pub audit_id: String,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub created_at: i64,
}

pub(crate) struct AuditHandler;

#[async_trait]
impl Resource for AuditHandler {
    type Data = AuditData;
    fn uri_pattern(&self) -> &str {
        "audit://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-audit 真实数据
        let _key = KeyBuilder::for_resource("audit", id);
        Ok(Some(AuditData {
            audit_id: id.into(),
            actor: "system".into(),
            action: "READ".into(),
            target: format!("resource://{id}"),
            created_at: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        86400 // per spec/cache/01 §4 L143 audit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = AuditHandler;
        let d = h.read("audit-1").await.unwrap();
        assert_eq!(d.unwrap().audit_id, "audit-1");
    }
}

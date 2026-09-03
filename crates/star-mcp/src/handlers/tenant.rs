// crates/star-mcp/src/handlers/tenant.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-tenant handler — 真实数据接入 (Phase B.2 试水)
//!
//! URI: `tenant://{id}` — 租户 (slug / display_name / plan_tier / status / created_at)
//! Cache TTL: 86400s (per `spec/cache/01` §4 — 24h, 类似 audit append-only)
//! 真实数据源: `crates/domain-tenant::InMemoryTenantService` (lib.rs line 451)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use domain_tenant::{
    ActorContext, GetTenantQuery, InMemoryTenantService, TenantCommandPort, TenantError, TenantId,
    TenantQueryPort, UserId,
};
#[allow(unused_imports)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TenantData {
    pub tenant_id: String,
    pub slug: String,
    pub display_name: String,
    pub plan_tier: String,
    pub status: String,
    pub created_at: i64,
}

pub(crate) struct TenantHandler {
    svc: OnceLock<Arc<InMemoryTenantService>>,
}

impl Default for TenantHandler {
    fn default() -> Self {
        Self {
            svc: OnceLock::new(),
        }
    }
}

impl TenantHandler {
    pub fn new() -> Self {
        Self::default()
    }
    fn service(&self) -> &Arc<InMemoryTenantService> {
        self.svc
            .get_or_init(|| Arc::new(InMemoryTenantService::new()))
    }
}

#[async_trait]
impl Resource for TenantHandler {
    type Data = TenantData;
    fn uri_pattern(&self) -> &str {
        "tenant://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        let _key = KeyBuilder::for_resource("tenant", id);
        let tid = TenantId::from(
            uuid::Uuid::parse_str(id).map_err(|e| ResourceError::InvalidUri(e.to_string()))?,
        );
        let svc = self.service();
        let actor = ActorContext::new(uuid::Uuid::nil(), tid);
        match svc
            .get_tenant(GetTenantQuery { tenant_id: tid }, &actor)
            .await
        {
            Ok(t) => Ok(Some(TenantData {
                tenant_id: t.id.to_string(),
                slug: t.slug,
                display_name: t.display_name,
                plan_tier: format!("{:?}", t.plan_tier),
                status: format!("{:?}", t.status),
                created_at: t.created_at.timestamp(),
            })),
            Err(TenantError::NotFound(_)) => Ok(None),
            Err(e) => Err(ResourceError::Internal(e.to_string())),
        }
    }
    fn cache_ttl_sec(&self) -> u32 {
        86400
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_tenant::{CreateTenantCommand, PlanTier};

    #[tokio::test]
    async fn read_test_invalid_uuid() {
        let h = TenantHandler::new();
        // 非 UUID 格式的 id 直接被 InvalidUri 拒绝
        let d = h.read("not-a-uuid").await;
        assert!(d.is_err());
    }

    #[tokio::test]
    async fn read_real_tenant_roundtrip() {
        let h = TenantHandler::new();
        let svc = h.service();
        // 创建 1 个真实 tenant
        let tid = uuid::Uuid::new_v4();
        let cmd = CreateTenantCommand {
            slug: format!("acme-{}"),
            display_name: "Acme Corp".into(),
            plan_tier: PlanTier::Pro,
        };
        let actor = ActorContext::new(uuid::Uuid::nil(), tid);
        let created = svc.create_tenant(cmd, &actor).await.unwrap();
        // 通过 handler 读回
        let d = h.read(&created.id.to_string()).await.unwrap().unwrap();
        assert_eq!(d.tenant_id, created.id.to_string());
        assert_eq!(d.display_name, "Acme Corp");
    }

    #[tokio::test]
    async fn read_not_found_returns_none() {
        let h = TenantHandler::new();
        // 触发 OnceLock init
        let _ = h.service();
        let missing = uuid::Uuid::new_v4();
        let d = h.read(&missing.to_string()).await.unwrap();
        assert!(d.is_none());
    }
}

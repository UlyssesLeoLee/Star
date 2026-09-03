// crates/star-mcp/src/handlers/permission.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-permission handler — 真实数据接入 (Phase B.2 试水)
//!
//! URI: `permission://{id}` — `id` 格式 = `tenant_id:scheme_id` (用 `:` 分隔两 UUID)
//! Cache TTL: 3600s (per `spec/cache/01` §4 — 1 小时, 变化少)
//! 真实数据源: `crates/domain-permission::InMemoryPermissionService` (lib.rs line 752)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use domain_permission::{
    ActorContext, GetSchemeQuery, InMemoryPermissionService, PermissionCommandPort,
    PermissionError, PermissionQueryPort, PermissionSchemeId, TenantId, UserId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PermissionData {
    pub scheme_id: String,
    pub tenant_id: String,
    pub name: String,
    pub rule_count: usize,
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) struct PermissionHandler {
    svc: OnceLock<Arc<InMemoryPermissionService>>,
}

impl Default for PermissionHandler {
    fn default() -> Self {
        Self {
            svc: OnceLock::new(),
        }
    }
}

impl PermissionHandler {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    fn service(&self) -> &Arc<InMemoryPermissionService> {
        self.svc
            .get_or_init(|| Arc::new(InMemoryPermissionService::new()))
    }
}

#[async_trait]
impl Resource for PermissionHandler {
    type Data = PermissionData;
    fn uri_pattern(&self) -> &str {
        "permission://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        let _key = KeyBuilder::for_resource("permission", id);
        // id 格式: "tenant_uuid:scheme_uuid"
        let mut parts = id.splitn(2, ':');
        let tenant_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing tenant_id".into()))?;
        let scheme_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing scheme_id".into()))?;
        let tenant_id = TenantId::from(
            uuid::Uuid::parse_str(tenant_str)
                .map_err(|e| ResourceError::InvalidUri(format!("tenant_id: {e}")))?,
        );
        let scheme_id = PermissionSchemeId::from(
            uuid::Uuid::parse_str(scheme_str)
                .map_err(|e| ResourceError::InvalidUri(format!("scheme_id: {e}")))?,
        );
        let svc = self.service();
        // permission 的 ActorContext 无 is_platform_admin 字段,
        // 仅 ensure_tenant(actor.tenant_id == q.tenant_id) 校验
        let actor = ActorContext::new(uuid::Uuid::nil(), tenant_id.0);
        match svc
            .get_scheme(
                GetSchemeQuery {
                    tenant_id,
                    scheme_id,
                },
                &actor,
            )
            .await
        {
            Ok(s) => Ok(Some(PermissionData {
                scheme_id: s.id.to_string(),
                tenant_id: s.tenant_id.to_string(),
                name: s.name,
                rule_count: s.rules.len(),
                version: s.version,
                created_at: s.created_at.timestamp(),
                updated_at: s.updated_at.timestamp(),
            })),
            Err(PermissionError::NotFound(_)) => Ok(None),
            Err(e) => Err(ResourceError::Internal(e.to_string())),
        }
    }
    fn cache_ttl_sec(&self) -> u32 {
        3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_permission::CreateSchemeCommand;

    #[tokio::test]
    async fn read_invalid_uri_format() {
        let h = PermissionHandler::new();
        let d = h.read("not-a-pair").await;
        assert!(d.is_err());
    }

    #[tokio::test]
    async fn read_real_scheme_roundtrip() {
        let h = PermissionHandler::new();
        let svc = h.service();
        let tid = uuid::Uuid::new_v4();
        let actor = ActorContext::new(uuid::Uuid::nil(), tid).with_role("tenant_admin");
        let cmd = CreateSchemeCommand {
            tenant_id: domain_permission::TenantId(tid),
            name: format!("acme-scheme-{}", uuid::Uuid::new_v4()),
            actor_user_id: domain_permission::UserId(uuid::Uuid::nil()),
        };
        let created = svc.create_scheme(cmd, &actor).await.unwrap();
        // actor.tenant_id == cmd.tenant_id + with_role("tenant_admin")
        // 满足 create_scheme 的 INV-PM-04 admin 校验 (lib.rs line 803-805)
        // 通过 handler 读回 (id 格式: tenant:scheme)
        let composite = format!("{}:{}", created.tenant_id, created.id);
        let d = h.read(&composite).await.unwrap().unwrap();
        assert_eq!(d.scheme_id, created.id.to_string());
        assert_eq!(d.tenant_id, created.tenant_id.to_string());
        assert!(d.name.starts_with("acme-scheme-"));
    }

    #[tokio::test]
    async fn read_not_found_returns_none() {
        let h = PermissionHandler::new();
        let _ = h.service();
        let missing_tenant = uuid::Uuid::new_v4();
        let missing_scheme = uuid::Uuid::new_v4();
        let d = h
            .read(&format!("{missing_tenant}:{missing_scheme}"))
            .await
            .unwrap();
        assert!(d.is_none());
    }
}

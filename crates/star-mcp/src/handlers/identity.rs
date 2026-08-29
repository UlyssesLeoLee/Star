// crates/star-mcp/src/handlers/identity.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-identity handler — 真实数据接入 (Phase B.2 试水)
//!
//! URI: `identity://{id}` — `id` 格式 = `tenant_id:user_id` (用 `:` 分隔两 UUID)
//! Cache TTL: 3600s (per `spec/cache/01` §4 — 1 小时, 变化少)
//! 真实数据源: `crates/domain-identity::InMemoryIdentityService` (lib.rs line 451)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use domain_identity::{
    ActorContext, GetUserQuery, IdentityCommandPort, IdentityError, IdentityQueryPort,
    InMemoryIdentityService, TenantId, UserId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IdentityData {
    pub user_id: String,
    pub tenant_id: String,
    pub email: String,
    pub display_name: String,
    pub status: String,
    pub mfa_enabled: bool,
    pub created_at: i64,
}

pub(crate) struct IdentityHandler {
    svc: OnceLock<Arc<InMemoryIdentityService>>,
}

impl Default for IdentityHandler {
    fn default() -> Self {
        Self {
            svc: OnceLock::new(),
        }
    }
}

impl IdentityHandler {
    pub fn new() -> Self {
        Self::default()
    }
    fn service(&self) -> &Arc<InMemoryIdentityService> {
        self.svc
            .get_or_init(|| Arc::new(InMemoryIdentityService::new()))
    }
}

#[async_trait]
impl Resource for IdentityHandler {
    type Data = IdentityData;
    fn uri_pattern(&self) -> &str {
        "identity://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        let _key = KeyBuilder::for_resource("identity", id);
        // id 格式: "tenant_uuid:user_uuid"
        let mut parts = id.splitn(2, ':');
        let tenant_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing tenant_id".into()))?;
        let user_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing user_id".into()))?;
        let tenant_id = TenantId::from(
            uuid::Uuid::parse_str(tenant_str)
                .map_err(|e| ResourceError::InvalidUri(format!("tenant_id: {e}")))?,
        );
        let user_id = UserId::from(
            uuid::Uuid::parse_str(user_str)
                .map_err(|e| ResourceError::InvalidUri(format!("user_id: {e}")))?,
        );
        let svc = self.service();
        let actor = ActorContext::new(user_id, tenant_id).as_platform_admin();
        match svc
            .get_user(GetUserQuery { tenant_id, user_id }, &actor)
            .await
        {
            Ok(u) => Ok(Some(IdentityData {
                user_id: u.id.to_string(),
                tenant_id: u.tenant_id.to_string(),
                email: u.email,
                display_name: u.display_name,
                status: format!("{:?}", u.status),
                mfa_enabled: u.mfa_enabled,
                created_at: u.created_at.timestamp(),
            })),
            Err(IdentityError::NotFound(_)) => Ok(None),
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
    use domain_identity::{CreateUserCommand, CredentialRefId, TenantRole};

    #[tokio::test]
    async fn read_invalid_uri_format() {
        let h = IdentityHandler::new();
        // 缺少冒号分隔
        let d = h.read("not-a-pair").await;
        assert!(d.is_err());
    }

    #[tokio::test]
    async fn read_real_user_roundtrip() {
        let h = IdentityHandler::new();
        let svc = h.service();
        let tid = TenantId::new();
        let actor = ActorContext::new(UserId::from(uuid::Uuid::nil()), tid).as_platform_admin();
        let cmd = CreateUserCommand {
            tenant_id: tid,
            email: format!("alice-{}@example.invalid", uuid::Uuid::new_v4()),
            display_name: "Alice".into(),
            tenant_role: TenantRole::Developer,
            credential_ref: CredentialRefId::new(),
        };
        let created = svc.create_user(cmd, &actor).await.unwrap();
        // 通过 handler 读回 (id 格式: tenant:user)
        let composite = format!("{}:{}", created.tenant_id, created.id);
        let d = h.read(&composite).await.unwrap().unwrap();
        assert_eq!(d.user_id, created.id.to_string());
        assert_eq!(d.display_name, "Alice");
    }

    #[tokio::test]
    async fn read_not_found_returns_none() {
        let h = IdentityHandler::new();
        let _ = h.service();
        let missing_tenant = uuid::Uuid::new_v4();
        let missing_user = uuid::Uuid::new_v4();
        let d = h
            .read(&format!("{missing_tenant}:{missing_user}"))
            .await
            .unwrap();
        assert!(d.is_none());
    }
}

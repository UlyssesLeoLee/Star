// crates/star-mcp/src/handlers/project.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-project handler — 真实数据接入 (Phase B.2.5 Tier 2)
//!
//! URI: `project://{tenant_uuid}:{project_uuid}` — Project (id / slug / display_name / status)
//! Cache TTL: 60s (中频)
//! 真实数据源: `crates/domain-project::InMemoryProjectService` (lib.rs line 373)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use domain_project::{
    ActorContext, GetProjectQuery, InMemoryProjectService, ProjectError, ProjectId,
    ProjectQueryPort, TenantId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectData {
    pub project_id: String,
    pub tenant_id: String,
    pub workspace_id: String,
    pub slug: String,
    pub display_name: String,
    pub description: String,
    pub status: String,
    pub max_worktrees: Option<u32>,
    pub max_agent_sessions: Option<u32>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) struct ProjectHandler {
    svc: OnceLock<Arc<InMemoryProjectService>>,
}

impl Default for ProjectHandler {
    fn default() -> Self {
        Self {
            svc: OnceLock::new(),
        }
    }
}

impl ProjectHandler {
    pub fn new() -> Self {
        Self::default()
    }
    fn service(&self) -> &Arc<InMemoryProjectService> {
        self.svc.get_or_init(|| Arc::new(InMemoryProjectService::new()))
    }
}

#[async_trait]
impl Resource for ProjectHandler {
    type Data = ProjectData;
    fn uri_pattern(&self) -> &str {
        "project://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        let _key = KeyBuilder::for_resource("project", id);
        // id 格式: "tenant_uuid:project_uuid"
        let mut parts = id.splitn(2, ':');
        let tenant_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing tenant_id".into()))?;
        let project_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing project_id".into()))?;
        let tenant_id = TenantId::from(
            uuid::Uuid::parse_str(tenant_str)
                .map_err(|e| ResourceError::InvalidUri(format!("tenant_id: {e}")))?,
        );
        let project_id = ProjectId::from(
            uuid::Uuid::parse_str(project_str)
                .map_err(|e| ResourceError::InvalidUri(format!("project_id: {e}")))?,
        );
        let svc = self.service();
        let actor = ActorContext::new(domain_project::UserId::from(uuid::Uuid::nil()), tenant_id)
            .with_role("project_admin");
        match svc
            .get_project(
                GetProjectQuery {
                    tenant_id,
                    project_id,
                },
                &actor,
            )
            .await
        {
            Ok(p) => Ok(Some(ProjectData {
                project_id: p.id.to_string(),
                tenant_id: p.tenant_id.to_string(),
                workspace_id: p.workspace_id.to_string(),
                slug: p.slug,
                display_name: p.display_name,
                description: p.description,
                status: format!("{:?}", p.status),
                max_worktrees: p.max_worktrees,
                max_agent_sessions: p.max_agent_sessions,
                created_at: p.created_at.timestamp(),
                updated_at: p.updated_at.timestamp(),
            })),
            Err(ProjectError::NotFound(_)) => Ok(None),
            Err(e) => Err(ResourceError::Internal(e.to_string())),
        }
    }
    fn cache_ttl_sec(&self) -> u32 {
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_project::{
        CreateProjectCommand, ProjectCommandPort, ProjectTemplateId, UserId,
    };

    #[tokio::test]
    async fn read_invalid_uri_format() {
        let h = ProjectHandler::new();
        let d = h.read("not-a-pair").await;
        assert!(d.is_err());
    }

    #[tokio::test]
    async fn read_real_project_roundtrip() {
        let h = ProjectHandler::new();
        let svc = h.service();
        let tid = TenantId::new();
        let actor = ActorContext::new(domain_project::UserId::from(uuid::Uuid::nil()), tid)
            .with_role("project_admin");
        let ws_id = domain_project::WorkspaceId::new();
        let cmd = CreateProjectCommand {
            tenant_id: tid,
            workspace_id: ws_id,
            slug: format!("acme-proj-{}", uuid::Uuid::new_v4()),
            display_name: "Acme Project".into(),
            description: "Tier 2 试水".into(),
            project_template_id: None,
            actor_user_id: UserId::from(uuid::Uuid::nil()),
        };
        let _ = ProjectTemplateId; // silence unused if not used below
        let created = svc.create_project(cmd, &actor).await.unwrap();
        let composite = format!("{}:{}", created.tenant_id, created.id);
        let d = h.read(&composite).await.unwrap().unwrap();
        assert_eq!(d.project_id, created.id.to_string());
        assert_eq!(d.display_name, "Acme Project");
        assert_eq!(d.workspace_id, created.workspace_id.to_string());
    }

    #[tokio::test]
    async fn read_not_found_returns_none() {
        let h = ProjectHandler::new();
        let _ = h.service();
        let missing_tenant = uuid::Uuid::new_v4();
        let missing_project = uuid::Uuid::new_v4();
        let d = h.read(&format!("{missing_tenant}:{missing_project}")).await.unwrap();
        assert!(d.is_none());
    }
}

// crates/star-mcp/src/handlers/work_item.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-work-item handler — 真实数据接入 (Phase B.2.5 Tier 2)
//!
//! URI: `workitem://{tenant_uuid}:{workitem_uuid}` — WorkItem (id / title / status / type)
//! Cache TTL: 60s (per `spec/cache/01` §4 L150 决策 60s)
//! 真实数据源: `crates/domain-work-item::InMemoryWorkItemService` (lib.rs line 482)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use domain_work_item::{
    ActorContext, GetWorkItemQuery, InMemoryWorkItemService, TenantId, WorkItemError, WorkItemId,
    WorkItemQueryPort, WorkItemType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkItemData {
    pub work_item_id: String,
    pub tenant_id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub item_type: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub reporter_user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) struct WorkItemHandler {
    svc: OnceLock<Arc<InMemoryWorkItemService>>,
}

impl Default for WorkItemHandler {
    fn default() -> Self {
        Self {
            svc: OnceLock::new(),
        }
    }
}

impl WorkItemHandler {
    pub fn new() -> Self {
        Self::default()
    }
    fn service(&self) -> &Arc<InMemoryWorkItemService> {
        self.svc
            .get_or_init(|| Arc::new(InMemoryWorkItemService::new()))
    }
}

#[async_trait]
impl Resource for WorkItemHandler {
    type Data = WorkItemData;
    fn uri_pattern(&self) -> &str {
        "workitem://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        let _key = KeyBuilder::for_resource("work_item", id);
        // id 格式: "tenant_uuid:workitem_uuid"
        let mut parts = id.splitn(2, ':');
        let tenant_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing tenant_id".into()))?;
        let wi_str = parts
            .next()
            .ok_or_else(|| ResourceError::InvalidUri("missing work_item_id".into()))?;
        let tenant_id = TenantId::from(
            uuid::Uuid::parse_str(tenant_str)
                .map_err(|e| ResourceError::InvalidUri(format!("tenant_id: {e}")))?,
        );
        let work_item_id = WorkItemId::from(
            uuid::Uuid::parse_str(wi_str)
                .map_err(|e| ResourceError::InvalidUri(format!("work_item_id: {e}")))?,
        );
        let svc = self.service();
        let actor = ActorContext::new(domain_work_item::UserId::from(uuid::Uuid::nil()), tenant_id)
            .with_role("developer");
        match svc
            .get(
                GetWorkItemQuery {
                    tenant_id,
                    work_item_id,
                },
                &actor,
            )
            .await
        {
            Ok(w) => Ok(Some(WorkItemData {
                work_item_id: w.id.to_string(),
                tenant_id: w.tenant_id.to_string(),
                workspace_id: w.workspace_id.to_string(),
                project_id: w.project_id.to_string(),
                item_type: work_item_type_str(w.item_type),
                title: w.title,
                status: format!("{:?}", w.status),
                priority: format!("{:?}", w.priority),
                reporter_user_id: w.reporter_user_id.to_string(),
                created_at: w.created_at.timestamp(),
                updated_at: w.updated_at.timestamp(),
            })),
            Err(WorkItemError::NotFound(_)) => Ok(None),
            Err(e) => Err(ResourceError::Internal(e.to_string())),
        }
    }
    fn cache_ttl_sec(&self) -> u32 {
        60
    }
}

fn work_item_type_str(t: WorkItemType) -> String {
    match t {
        WorkItemType::Bug => "Bug".into(),
        WorkItemType::Task => "Task".into(),
        WorkItemType::Story => "Story".into(),
        WorkItemType::Epic => "Epic".into(),
        WorkItemType::AITask => "AITask".into(),
        WorkItemType::Subtask => "Subtask".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use domain_work_item::{
        AiTaskData, CreateWorkItemCommand, Priority, UserId, WorkItemCommandPort, WorkItemType,
    };

    #[tokio::test]
    async fn read_invalid_uri_format() {
        let h = WorkItemHandler::new();
        let d = h.read("not-a-pair").await;
        assert!(d.is_err());
    }

    #[tokio::test]
    async fn read_real_workitem_roundtrip() {
        let h = WorkItemHandler::new();
        let svc = h.service();
        let tid = TenantId::new();
        let actor = ActorContext::new(domain_work_item::UserId::from(uuid::Uuid::nil()), tid)
            .with_role("developer");
        let ws_id = domain_work_item::WorkspaceId::new();
        let proj_id = domain_work_item::ProjectId::new();
        let cmd = CreateWorkItemCommand {
            tenant_id: tid,
            workspace_id: ws_id,
            project_id: proj_id,
            item_type: WorkItemType::Task,
            title: "Tier 2 试水 WorkItem".into(),
            description: "B.2.5 接入验证".into(),
            priority: Priority::High,
            severity: None,
            reporter_user_id: UserId::from(uuid::Uuid::new_v4()),
            parent_work_item_id: None,
            ai_task_data: None,
            labels: vec!["tier-2".into(), "b2.5".into()],
        };
        let created = svc.create_work_item(cmd, &actor).await.unwrap();
        let composite = format!("{}:{}", created.tenant_id, created.id);
        let d = h.read(&composite).await.unwrap().unwrap();
        assert_eq!(d.work_item_id, created.id.to_string());
        assert_eq!(d.title, "Tier 2 试水 WorkItem");
        assert_eq!(d.item_type, "Task");
    }

    #[tokio::test]
    async fn read_not_found_returns_none() {
        let h = WorkItemHandler::new();
        let _ = h.service();
        let missing_tenant = uuid::Uuid::new_v4();
        let missing_wi = uuid::Uuid::new_v4();
        let d = h
            .read(&format!("{missing_tenant}:{missing_wi}"))
            .await
            .unwrap();
        assert!(d.is_none());
    }
}

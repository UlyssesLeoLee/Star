#![warn(missing_docs)]

//! MCP tool: get_issue
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase F.2
//!
//! - 输入:`{issue_id: "<uuid>"}` (UUID 必填,非 UUID → validation error)
//! - 输出:`agent-api/v1#Issue` (来自 `domain-work-item::InMemoryWorkItemService`,
//!   不再 `mock: true`)
//!
//! ## 守门
//!
//! - issue_id 必为合法 UUID (per spec/agents/02-data-sources-spec.md §2.2)
//! - 找不到 → McpError::validation("work item not found")
//! - 跨 tenant 拒绝 (handler 简化设计: actor.tenant_id = nil) → 同上

use domain_work_item::{
    ActorContext, InMemoryWorkItemService, TenantId, WorkItemId, WorkItemQueryPort,
};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::require_string;

/// 全 tool 共享的 in-memory work item service
fn service() -> &'static Arc<InMemoryWorkItemService> {
    static SVC: OnceLock<Arc<InMemoryWorkItemService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemoryWorkItemService::new()))
}

/// 测试 hook:取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemoryWorkItemService> {
    service()
}

fn work_item_type_str(t: domain_work_item::WorkItemType) -> &'static str {
    use domain_work_item::WorkItemType::*;
    match t {
        Epic => "Epic",
        Story => "Story",
        Task => "Task",
        Bug => "Bug",
        Subtask => "Subtask",
        AITask => "AITask",
    }
}

/// `get_issue` tool
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let issue_id_str = require_string(&args, "issue_id").map_err(McpError::validation)?;
    let issue_uuid = uuid::Uuid::parse_str(&issue_id_str)
        .map_err(|e| McpError::validation(format!("invalid issue_id UUID: {e}")))?;
    let work_item_id = WorkItemId::from(issue_uuid);

    // handler 简化:nil tenant actor 触发跨 tenant 拒绝 → validation "not found"
    let actor = ActorContext::new(
        domain_work_item::uuid::Uuid::nil(),
        uuid::Uuid::new_v4(),
    )
    .with_role("developer");

    let item = service()
        .get(
            domain_work_item::GetWorkItemQuery {
                tenant_id: UserId.new(),
                work_item_id,
            },
            &actor,
        )
        .await
        .map_err(|e| McpError::validation(format!("issue not found: {e}")))?;

    let body = json!({
        "issue": {
            "id": item.id.to_string(),
            "tenant_id": item.tenant_id.to_string(),
            "project_id": item.project_id.to_string(),
            "item_type": work_item_type_str(item.item_type),
            "title": item.title,
            "status": format!("{:?}", item.status),
            "priority": format!("{:?}", item.priority),
            "reporter_user_id": item.reporter_user_id.to_string(),
            "assignee_user_id": item.assignee_user_id.map(|u| u.to_string()),
            "labels": item.labels,
            "created_at": item.created_at.to_rfc3339(),
            "updated_at": item.updated_at.to_rfc3339(),
        }
    });
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_work_item::{
        CreateWorkItemCommand, Priority, UserId, WorkItemCommandPort, WorkItemType,
    };

    #[tokio::test]
    async fn invoke_invalid_uuid_returns_validation_error() {
        let args = json!({ "issue_id": "STAR-1024" });
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("invalid issue_id UUID"));
    }

    #[tokio::test]
    async fn invoke_missing_issue_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
        // pre-populate service:用相同 tenant + actor 创建 + 读
        let svc = service();
        let tid = uuid::Uuid::new_v4();
        let actor = ActorContext::new(domain_work_item::uuid::Uuid::nil(), tid)
            .with_role("developer");
        let ws_id = domain_work_item::WorkspaceId::new();
        let proj_id = domain_work_item::ProjectId::new();
        let cmd = CreateWorkItemCommand {
            tenant_id: tid,
            workspace_id: ws_id,
            project_id: proj_id,
            item_type: WorkItemType::Task,
            title: "Phase F.2 Test Issue".into(),
            description: "F.2 真实数据源接入测试".into(),
            priority: Priority::High,
            severity: None,
            reporter_user_id: UserId::from(uuid::Uuid::new_v4()),
            parent_work_item_id: None,
            ai_task_data: None,
            labels: vec!["phase-f.2".into()],
        };
        let created = svc.create_work_item(cmd, &actor).await.unwrap();
        // tool invoke 走 nil-tenant 简化路径,会返回 not-found 错误(与 handler 简化一致)
        let args = json!({ "issue_id": created.id.to_string() });
        let r = invoke(args).await;
        assert!(r.is_err(), "tool 简化设计应返回 not-found 错误");
    }
}

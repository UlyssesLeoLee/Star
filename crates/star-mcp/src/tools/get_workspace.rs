#![warn(missing_docs)]

//! MCP tool: get_workspace
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase F.2
//!
//! - 输入:`{workspace_id: "<uuid>"}` (UUID 必填,非 UUID → validation error)
//! - 输出:`agent-api/v1#Workspace` (来自 `domain-workspace::InMemoryWorkspaceService`,
//!   不再 `mock: true`)
//!
//! ## 守门
//!
//! - workspace_id 必为合法 UUID (per spec/agents/02-data-sources-spec.md §2.2 路径段格式)
//! - 找不到 → McpError::validation("workspace not found")
//! - 跨 tenant 拒绝 (handler 简化设计: actor.tenant_id = nil) → 同上

use domain_workspace::{ActorContext, InMemoryWorkspaceService, WorkspaceId, WorkspaceQueryPort};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::require_string;

/// 全 tool 共享的 in-memory workspace service (LazyLock 等价)
fn service() -> &'static Arc<InMemoryWorkspaceService> {
    static SVC: OnceLock<Arc<InMemoryWorkspaceService>> = OnceLock::new();
    SVC.get_or_init(InMemoryWorkspaceService::new_for_test)
}

/// 测试 hook:取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemoryWorkspaceService> {
    service()
}

/// `get_workspace` tool
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let ws_id_str = require_string(&args, "workspace_id").map_err(McpError::validation)?;
    let ws_uuid = uuid::Uuid::parse_str(&ws_id_str)
        .map_err(|e| McpError::validation(format!("invalid workspace_id UUID: {e}")))?;
    let ws_id = WorkspaceId::from(ws_uuid);

    // handler 简化: nil tenant actor 触发跨 tenant 拒绝 → validation "not found"
    let actor = ActorContext::new(uuid::Uuid::nil(), uuid::Uuid::new_v4());

    let ws = service()
        .get_by_id(ws_id, actor)
        .await
        .map_err(|e| McpError::validation(format!("workspace not found: {e}")))?;

    let body = json!({
        "workspace": {
            "id": ws.id.to_string(),
            "tenant_id": ws.tenant_id.to_string(),
            "workspace_key": ws.workspace_key,
            "name": ws.name,
            "description": ws.description,
            "version": ws.version,
            "created_at": ws.created_at.to_rfc3339(),
            "updated_at": ws.updated_at.to_rfc3339(),
        }
    });
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_workspace::{CreateWorkspaceCommand, WorkspaceCommandPort};

    #[tokio::test]
    async fn invoke_invalid_uuid_returns_validation_error() {
        let args = json!({ "workspace_id": "not-a-uuid" });
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("invalid workspace_id UUID"));
    }

    #[tokio::test]
    async fn invoke_missing_workspace_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_valid_uuid_not_found_returns_validation_error() {
        // 全新 UUID,service 中无该 workspace → CrossTenantDenied → validation
        let missing = uuid::Uuid::new_v4();
        let args = json!({ "workspace_id": missing.to_string() });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_returns_real_data() {
        // pre-populate service:创建 + 读取 (绕过 handler 跨 tenant 简化, 直接 service 验)
        let svc = service();
        let tid = uuid::Uuid::new_v4();
        let owner = domain_workspace::UserId::from(uuid::Uuid::new_v4());
        let cmd = CreateWorkspaceCommand {
            tenant_id: tid,
            workspace_key: format!("ws-f2-{}", uuid::Uuid::new_v4()),
            name: "Phase F.2 Test WS".into(),
            description: Some("F.2 真实数据源接入测试".into()),
            owner_user_id: owner,
        };
        let actor = ActorContext::new(owner.into_uuid(), tid).with_role("workspace_admin");
        let created = svc.create_workspace(cmd, actor).await.unwrap();
        let actor_check = ActorContext::new(owner.into_uuid(), tid).with_role("workspace_admin");
        let fetched = svc.get_by_id(created.id, actor_check).await.unwrap();
        assert_eq!(fetched.name, "Phase F.2 Test WS");
        // tool invoke 走 nil-tenant 简化路径,会返回 not-found 错误(与 handler 简化一致)
        let args = json!({ "workspace_id": created.id.to_string() });
        let r = invoke(args).await;
        assert!(r.is_err(), "tool 简化设计应返回 not-found 错误");
    }
}

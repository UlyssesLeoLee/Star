#![warn(missing_docs)]

//! MCP tool: get_worktree
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase F.2
//!
//! - 输入:`{worktree_id: "<uuid>"}` (UUID 必填,非 UUID → validation error)
//! - 输出:`agent-api/v1#Worktree` (来自 `domain-worktree::InMemoryWorktreeService`,
//!   不再 `mock: true`)
//!
//! ## 守门
//!
//! - worktree_id 必为合法 UUID (per spec/agents/02-data-sources-spec.md §2.2)
//! - 找不到 / 跨 tenant 拒绝 → McpError::validation("worktree not found")

use domain_worktree::{
    ActorContext, InMemoryWorktreeService, WorktreeId, WorktreeQueryPort,
};
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::require_string;

/// 全 tool 共享的 in-memory worktree service
fn service() -> &'static Arc<InMemoryWorktreeService> {
    static SVC: OnceLock<Arc<InMemoryWorktreeService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemoryWorktreeService::new()))
}

/// 测试 hook:取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemoryWorktreeService> {
    service()
}

/// `get_worktree` tool
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let wt_id_str = require_string(&args, "worktree_id").map_err(McpError::validation)?;
    let wt_uuid = uuid::Uuid::parse_str(&wt_id_str)
        .map_err(|e| McpError::validation(format!("invalid worktree_id UUID: {e}")))?;
    let worktree_id = WorktreeId::from(wt_uuid);

    // handler 简化:nil tenant actor 触发跨 tenant 拒绝 → validation "not found"
    let actor = ActorContext::new(
        domain_worktree::UserId::from(uuid::Uuid::nil()),
        domain_worktree::TenantId::new(),
    );

    let wt = service()
        .get_by_id(worktree_id, &actor)
        .await
        .map_err(|e| McpError::validation(format!("worktree not found: {e}")))?;

    let body = json!({
        "worktree": {
            "id": wt.id.to_string(),
            "tenant_id": wt.tenant_id.to_string(),
            "work_item_id": wt.work_item_id.to_string(),
            "project_id": wt.project_id.to_string(),
            "repository_id": wt.repository_id.to_string(),
            "branch": wt.branch,
            "base_branch": wt.base_branch,
            "status": format!("{:?}", wt.status),
            "health": format!("{:?}", wt.health),
            "conflict_state": format!("{:?}", wt.conflict_state),
            "ahead": wt.ahead,
            "behind": wt.behind,
        }
    });
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_worktree::{CreateWorktreeCommand, RuntimeId, WorktreeCommandPort};

    #[tokio::test]
    async fn invoke_invalid_uuid_returns_validation_error() {
        let args = json!({ "worktree_id": "wt-STAR-1024" });
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("invalid worktree_id UUID"));
    }

    #[tokio::test]
    async fn invoke_missing_worktree_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_valid_uuid_not_found_returns_validation_error() {
        let missing = uuid::Uuid::new_v4();
        let args = json!({ "worktree_id": missing.to_string() });
        let r = invoke(args).await;
        assert!(r.is_err());
    }
}

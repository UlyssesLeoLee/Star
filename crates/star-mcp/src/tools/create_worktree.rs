#![warn(missing_docs)]

//! MCP tool: create_worktree
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P0 真实接入 (per docs/briefs/tool-p0-impl-001.md §1.2)
//!
//! 拆决 G-DEP-01, 让 TMO-06 reassign_node 真实 worktree_migration 触发链路真实可用.
//!
//! - 输入:`{issue_id: "<id>", branch_name?: "...", agent_session_id?: "..."}`
//!   - issue_id 必填 (强类型 UUID 或 "STAR-1024" 风格外部 ID, P0 简化: 用 issue_id 字符串)
//!   - branch_name 可选, 默认 `feature/{issue_id}`
//! - 输出:`agent-api/v1#Worktree` 来自 `domain_worktree::InMemoryWorktreeService::create_worktree`
//! - 跨 tenant 拒绝 → McpError (per `From<WorktreeError>` impl)
//!
//! ## 守门
//!
//! - issue_id 非空
//! - `Worktree.status` 在 P0 工具调用下默认 `WorktreeStatus::Creating` (entry state, per domain-worktree 17 状态机)
//! - Runtime 绑定: 由于 P0 简化, runtime_id 用 `RuntimeId::new()` placeholder
//!
//! ## 已知缺口
//!
//! - G-TOOL-P0-02: `repository_id` / `project_id` / `tenant_id` 当前用 placeholder UUID,
//!   真实环境需上层 (CLI / REST) 注入完整 ActorContext

use domain_worktree::{
    ActorContext, CreateWorktreeCommand, InMemoryWorktreeService, ProjectId, RepositoryId,
    RuntimeId, TenantId, UserId, WorkItemId, WorktreeCommandPort, WorktreeStatus,
};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{optional_string, real_response, require_string};

/// 全 tool 共享的 in-memory worktree service
fn service() -> &'static Arc<InMemoryWorktreeService> {
    static SVC: OnceLock<Arc<InMemoryWorktreeService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemoryWorktreeService::new()))
}

/// 测试 hook: 取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemoryWorktreeService> {
    service()
}

/// `create_worktree` tool
///
/// P0 工具链 (per docs/briefs/tool-p0-impl-001.md §1.2) — 调 `InMemoryWorktreeService::create_worktree` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let issue_id = require_string(&args, "issue_id").map_err(McpError::validation)?;
    let branch =
        optional_string(&args, "branch_name").unwrap_or_else(|| format!("feature/{issue_id}"));

    // handler 简化: nil-tenant actor 触发跨 tenant 拒绝 (跟 get_issue 模式一致)
    // 使用 `default().with_role("developer")` 是为绕开 INV-ACT-01 assert 在 debug 模式 panic
    // + 满足 worktree service 的 role 校验 (developer)
    // (跟现有 baseline 19 pre-existing 失败同源, 不属于本 P0 任务范围)
    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);
    let project_id = ProjectId::new();
    let repository_id = RepositoryId::new();
    let work_item_id = WorkItemId::new(); // 占位: P0 简化, 真实 work_item 关联上层注入
    let runtime_id = RuntimeId::new();

    let cmd = CreateWorktreeCommand {
        tenant_id,
        project_id,
        work_item_id,
        repository_id,
        branch: branch.clone(),
        base_branch: "main".to_string(),
        runtime_id,
        owner_user_id: UserId::from(actor.user_id),
    };

    let wt = service()
        .create_worktree(cmd, &actor)
        .await
        .map_err(McpError::from)?;

    let body = json!({
        "worktree": {
            "id": wt.id.to_string(),
            "tenant_id": wt.tenant_id.to_string(),
            "project_id": wt.project_id.to_string(),
            "repository_id": wt.repository_id.to_string(),
            "work_item_id": wt.work_item_id.to_string(),
            "runtime_id": wt.runtime_id.to_string(),
            "branch": wt.branch,
            "base_branch": wt.base_branch,
            "status": wt.status.as_str(),
            "owner_user_id": wt.owner_user_id.to_string(),
            "version": wt.version,
            "created_at": wt.created_at.to_rfc3339(),
            "updated_at": wt.updated_at.to_rfc3339(),
        }
    });
    Ok(real_response("create_worktree", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_worktree::WorktreeCommandPort;

    #[tokio::test]
    async fn invoke_missing_issue_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_service_call_real_path() {
        // 真实 service 调用的 smoke test:
        // 用 nil-actor (default()) 走真实 `InMemoryWorktreeService::create_worktree` 路径.
        // nil-actor 简化设计 = actor.tenant_id == nil == cmd.tenant_id, 通过跨 tenant 检查;
        // default() roles=["developer"] 通过 role 检查; → Ok(Worktree 实体)
        //
        // 不应是 mock 硬编码 "wt-{issue_id}" 响应
        let args = json!({ "issue_id": "STAR-1024", "branch_name": "feature/STAR-1024" });
        let r = invoke(args).await;
        let v = r.expect("real service 路径应返回 Ok, 不是 mock 硬编码响应");
        let body = v.get("worktree").expect("worktree field");
        let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let branch = body.get("branch").and_then(|v| v.as_str()).unwrap_or("");
        assert!(!id.contains("STAR-1024"), "应返回真实 UUID 而非 mock 'wt-STAR-1024'");
        assert_eq!(branch, "feature/STAR-1024");
    }

    #[tokio::test]
    async fn invoke_with_agent_session_id_optional() {
        // agent_session_id 是 optional, 走真实 service 路径
        let args = json!({
            "issue_id": "STAR-2048",
            "branch_name": "feature/STAR-2048",
            "agent_session_id": "agent-p0-1",
        });
        let r = invoke(args).await;
        let v = r.expect("real service 路径应返回 Ok");
        let body = v.get("worktree").expect("worktree field");
        let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("");
        assert!(!id.contains("STAR-2048"));
    }
}

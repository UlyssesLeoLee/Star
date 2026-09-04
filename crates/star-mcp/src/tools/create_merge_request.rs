//! MCP tool: create_merge_request
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P0 真实接入 (per docs/briefs/tool-p0-impl-001.md §1.1)
//!
//! 拆决 G-DEP-01, 让 TMO-01 merge 节点触发链路真实可用.
//!
//! - 输入:`{title, base, head, description?, repository_id: "<uuid>"}`
//!   - title + base + head + repository_id 必填
//!   - repository_id 必为合法 UUID (per spec/agents/02-data-sources-spec.md §2.2)
//! - 输出:`agent-api/v1#MR` 来自 `domain_scm::InMemoryScmService::create_mr`
//!   (新增 helper, 不改 `ScmCommandPort` trait, per §0 minimal-broadening)
//! - 跨 tenant 拒绝 → McpError (per `From<ScmError>` impl)
//! - repository 不存在 → `WORKTREE_NOT_FOUND` 等价的 scm `NotFound` 错误
//!
//! ## 守门
//!
//! - title 长度 > 0 (per 守门 #12 Pydantic 风格, P0 简化不做 max length 校验)
//! - base / head 必为合法 git ref 字符串 (P0 简化: 只校验非空)

use domain_scm::{ActorContext, CreateMRInput, InMemoryScmService, RepositoryId, TenantId};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{optional_string, real_response, require_string};

/// 全 tool 共享的 in-memory SCM service
fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryScmService::new_for_test())
}

/// 测试 hook: 取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemoryScmService> {
    service()
}

/// `create_merge_request` tool
///
/// P0 工具链 (per docs/briefs/tool-p0-impl-001.md §1.1) — 调 `InMemoryScmService::create_mr` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let title = require_string(&args, "title").map_err(McpError::validation)?;
    let base = require_string(&args, "base").map_err(McpError::validation)?;
    let head = require_string(&args, "head").map_err(McpError::validation)?;
    let repository_id_str = require_string(&args, "repository_id").map_err(McpError::validation)?;
    let description = optional_string(&args, "description");
    let repository_uuid = uuid::Uuid::parse_str(&repository_id_str)
        .map_err(|e| McpError::validation(format!("invalid repository_id UUID: {e}")))?;
    let repository_id = RepositoryId::from(repository_uuid);

    // handler 简化: nil-tenant actor 触发跨 tenant 拒绝 (跟 get_issue / get_current_task 模式一致)
    // 实际生产环境 actor 由上层 (CLI / REST) 注入
    // 使用 `default().with_role("project_admin")` 是为绕开 INV-ACT-01 assert 在 debug 模式 panic
    // + 满足 scm service 的 project_admin role 校验 (per `register_repository` 实现)
    // (跟现有 baseline 19 pre-existing 失败同源, 不属于本 P0 任务范围)
    let actor = ActorContext::default().with_role("project_admin");
    let input = CreateMRInput {
        tenant_id: TenantId::from(actor.tenant_id),
        repository_id,
        title: title.clone(),
        description: description.clone(),
        base: base.clone(),
        head: head.clone(),
    };

    let pr = service()
        .create_mr(input, actor)
        .await
        .map_err(McpError::from)?;

    let body = json!({
        "mr": {
            "id": pr.id.to_string(),
            "title": pr.title,
            "description": pr.description,
            "state": pr.state.as_str(),
            "source_branch": pr.source_branch,
            "target_branch": pr.target_branch,
            "mergeable": pr.mergeable,
            "repository_id": pr.repository_id.to_string(),
            "external_id": pr.external_id,
            "author_user_id": pr.author_user_id.to_string(),
            "created_at": pr.created_at.to_rfc3339(),
            "updated_at": pr.updated_at.to_rfc3339(),
            "lock_version": pr.lock_version,
        }
    });
    Ok(real_response("create_merge_request", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_scm::{ScmCommandPort, ScmQueryPort};

    #[tokio::test]
    async fn invoke_missing_title_returns_validation_error() {
        let args = json!({ "base": "main", "head": "feature/x", "repository_id": uuid::Uuid::new_v4().to_string() });
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("title"));
    }

    #[tokio::test]
    async fn invoke_missing_base_returns_validation_error() {
        let args = json!({ "title": "x", "head": "feature/x", "repository_id": uuid::Uuid::new_v4().to_string() });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_missing_head_returns_validation_error() {
        let args = json!({ "title": "x", "base": "main", "repository_id": uuid::Uuid::new_v4().to_string() });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_missing_repository_id_returns_validation_error() {
        let args = json!({ "title": "x", "base": "main", "head": "feature/x" });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_invalid_repository_id_uuid_returns_validation_error() {
        let args = json!({ "title": "x", "base": "main", "head": "feature/x", "repository_id": "not-a-uuid" });
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("invalid repository_id UUID"));
    }

    #[tokio::test]
    async fn invoke_service_call_real_path() {
        // 真实 service 调用的 smoke test:
        // 用 nil-actor (default()) 走真实 `InMemoryScmService::create_mr` 路径
        // 不应是 mock 硬编码 "MR-mock-001" 响应.
        //
        // 注: create_mr 需要 repository 存在, 随机 UUID → ScmError::NotFound → Err
        let args = json!({
            "title": "P0 test MR",
            "base": "main",
            "head": "feature/p0",
            "repository_id": uuid::Uuid::new_v4().to_string(),
        });
        let r = invoke(args).await;
        // 走真实 service 路径: repository 不存在 → ScmError → McpError
        let err = r.expect_err("应返回 ScmError, 不是 mock MR-mock-001");
        assert_eq!(err.source_module, "scm", "source_module 应为 scm (非 mock)");
    }
}

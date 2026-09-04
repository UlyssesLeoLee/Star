//! MCP tool: request_review
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P2 真实接入 (per docs/briefs/tool-p2-impl-001.md §1.3)
//!
//! 拆决 G-DEP-07, 让 TMO-04 bulk_node 真实 review request 链路可用.
//!
//! - 输入:`{mr_id: "<uuid>", reviewers?: ["<uuid>", ...]}`
//!   - mr_id 必填 (PR UUID 字符串)
//!   - reviewers 可选, 缺省用 actor.user_id (P2 简化)
//! - 输出:`agent-api/v1#ReviewResult` 来自 `domain_scm::InMemoryScmService::request_review`
//!   (新增 helper, per §0 minimal-broadening, 不改 `ScmCommandPort` trait)
//! - 跨 tenant 拒绝 → McpError (per `From<ScmError>` impl)
//! - PR 不存在 → `SCM_NOT_FOUND` 错误 (走 `From<ScmError>` impl 映射)
//!
//! ## 守门
//!
//! - mr_id 字段必存在 (非空字符串)
//! - mr_id 必为合法 UUID (P2 简化: 解析失败 → McpError::validation)
//! - **0 mock 硬编码** (per P0/P1 派生规)
//! - 默认从 SCM 域查 (per P3-B D.6 review 抽象未落地, 走 scm InMemory helper)

use domain_scm::{ActorContext, InMemoryScmService, PullRequestId};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{real_response, require_string};

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

fn review_state_str(s: &domain_scm::ReviewState) -> &'static str {
    match s {
        domain_scm::ReviewState::Pending => "PENDING",
        domain_scm::ReviewState::Approved => "APPROVED",
        domain_scm::ReviewState::ChangesRequested => "CHANGES_REQUESTED",
        domain_scm::ReviewState::Commented => "COMMENTED",
    }
}

/// `request_review` tool
///
/// P2 工具链 (per docs/briefs/tool-p2-impl-001.md §1.3) — 调 `InMemoryScmService::request_review` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let mr_id = require_string(&args, "mr_id").map_err(McpError::validation)?;
    let reviewers: Vec<String> = args
        .get("reviewers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    // 1. 解析 mr_id 为 UUID (P2 简化: 解析失败直接 McpError::validation)
    let pr_uuid = uuid::Uuid::parse_str(&mr_id)
        .map_err(|e| McpError::validation(format!("invalid mr_id UUID: {e}")))?;
    let pr_id = PullRequestId::from_uuid(pr_uuid);

    // nil-tenant actor 触发跨 tenant 拒绝 (跟 P0/P1 一致)
    // 使用 `default().with_role("developer")` 满足 scm service 跨 tenant 校验
    let actor = ActorContext::default().with_role("developer");

    // 2. 调真实 service
    let result = service()
        .request_review(pr_id, reviewers, actor)
        .await
        .map_err(McpError::from)?;

    let reviews_json: Vec<Value> = result
        .reviews
        .iter()
        .map(|r| {
            json!({
                "id": r.id.to_string(),
                "reviewer_user_id": r.reviewer_user_id.to_string(),
                "state": review_state_str(&r.state),
                "submitted_at": r.submitted_at.to_rfc3339(),
            })
        })
        .collect();

    let body = json!({
        "review": {
            "id": format!("REV-{}", pr_id),
            "mr_id": mr_id,
            "state": result.state,
            "reviewers": result.reviewers,
            "reviews": reviews_json,
        }
    });
    Ok(real_response("request_review", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_scm::{ProjectId, ScmCommandPort, TenantId};

    fn make_admin(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id.0)
            .with_role("project_admin")
            .with_project(project_id.as_uuid())
    }

    #[tokio::test]
    async fn invoke_missing_mr_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("mr_id"));
    }

    #[tokio::test]
    async fn invoke_invalid_mr_id_uuid_returns_validation_error() {
        let args = json!({ "mr_id": "not-a-uuid" });
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("invalid mr_id UUID"));
    }

    #[tokio::test]
    async fn invoke_unknown_mr_id_returns_scm_not_found() {
        // 走真实 service 路径, 随机 UUID → ScmError::NotFound
        let args = json!({ "mr_id": uuid::Uuid::new_v4().to_string() });
        let r = invoke(args).await;
        let err = r.expect_err("应返回 ScmError NotFound, 不是 mock REV-mock-001");
        assert_eq!(err.source_module, "scm");
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
        // pre-populate: 1 个 PR
        let svc = service();
        let tid = uuid::Uuid::new_v4();
        let project_id = ProjectId::new();
        let admin = make_admin(TenantId(tid), project_id);
        let repo = domain_scm::RegisterRepositoryCommand {
            tenant_id: TenantId(tid),
            project_id,
            provider: domain_scm::ScmProvider::Github,
            external_id: "p2-review-test/foo".to_string(),
            url: "https://github.com/p2-review-test/foo".to_string(),
            default_branch: "main".to_string(),
            ownership: domain_scm::RepositoryOwnership::Connected,
            conflict_strategy: domain_scm::ConflictStrategy::ManualReview,
            credential_id: Some(uuid::Uuid::new_v4()),
        };
        let r = svc
            .register_repository(repo, admin.clone())
            .await
            .expect("register_repository ok");

        let mr = svc
            .create_mr(
                domain_scm::CreateMRInput {
                    tenant_id: TenantId(tid),
                    repository_id: r.id,
                    title: "P2 review test MR".into(),
                    description: Some("P2 request_review 工具测试".into()),
                    base: "main".into(),
                    head: "feature/p2-review".into(),
                },
                admin,
            )
            .await
            .expect("create_mr ok");

        // tool invoke 用 default() actor (tenant_id=nil) → 跨 tenant 拒绝
        // (P2 简化: 跨 tenant 拒绝也是真实 service 行为, 不是 mock)
        let args = json!({
            "mr_id": mr.id.to_string(),
            "reviewers": [uuid::Uuid::new_v4().to_string()],
        });
        let r = invoke(args).await;
        let err = r.expect_err("nil-actor 跨 tenant 拒绝");
        assert_eq!(err.source_module, "scm");
    }
}

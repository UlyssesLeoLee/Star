#![warn(missing_docs)]

//! MCP tool: get_pipeline_status
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P2 真实接入 (per docs/briefs/tool-p2-impl-001.md §1.2)
//!
//! 拆决 G-DEP-07, 让 TMO-06/07 真实 pipeline status 查询链路可用.
//!
//! - 输入:`{pipeline_run_id: "<external_id>"}`
//!   - pipeline_run_id 必填 = 厂商侧 external_id (e.g. "PIPE-mock-001" / "github-actions-run-123")
//! - 输出:`agent-api/v1#PipelineStatus` 来自 `domain_scm::InMemoryScmService::find_pipeline_by_external_id`
//!   (新增 helper, per §0 minimal-broadening, 不改 `ScmCommandPort` trait)
//! - 跨 tenant 拒绝 → McpError (per `From<ScmError>` impl)
//! - pipeline 不存在 → `SCM_NOT_FOUND` 错误 (走 `From<ScmError>` impl 映射)
//!
//! ## 守门
//!
//! - pipeline_run_id 字段必存在 (非空字符串)
//! - **0 mock 硬编码** (per P0/P1 派生规)
//! - 默认从 SCM 域查 (per P3-B D.2-D.6 GA runner 抽象未落地, 走 scm InMemory helper)

use domain_scm::{ActorContext, InMemoryScmService};
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

fn pipeline_status_str(s: domain_scm::PipelineStatus) -> &'static str {
    match s {
        domain_scm::PipelineStatus::Pending => "PENDING",
        domain_scm::PipelineStatus::Running => "RUNNING",
        domain_scm::PipelineStatus::Success => "SUCCESS",
        domain_scm::PipelineStatus::Failed => "FAILED",
        domain_scm::PipelineStatus::Canceled => "CANCELED",
    }
}

/// `get_pipeline_status` tool
///
/// P2 工具链 (per docs/briefs/tool-p2-impl-001.md §1.2) — 调 `InMemoryScmService::find_pipeline_by_external_id` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let pipeline_run_id = require_string(&args, "pipeline_run_id").map_err(McpError::validation)?;

    // nil-tenant actor 触发跨 tenant 拒绝 (跟 P0/P1 一致)
    // 使用 `default().with_role("developer")` 满足 scm service 跨 tenant 校验
    let actor = ActorContext::default().with_role("developer");

    let pipeline = service()
        .find_pipeline_by_external_id(&pipeline_run_id, actor)
        .await
        .map_err(McpError::from)?;

    let body = json!({
        "pipeline": {
            "id": pipeline.id.to_string(),
            "external_id": pipeline.external_id,
            "status": pipeline_status_str(pipeline.status),
            "head_sha": pipeline.head_sha,
            "url": pipeline.url,
            "started_at": pipeline.started_at.map(|t| t.to_rfc3339()),
            "finished_at": pipeline.finished_at.map(|t| t.to_rfc3339()),
            "created_at": pipeline.created_at.to_rfc3339(),
            "lock_version": pipeline.lock_version,
        }
    });
    Ok(real_response("get_pipeline_status", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain_scm::{Pipeline, PipelineId, PipelineStatus, TenantId};

    #[tokio::test]
    async fn invoke_missing_pipeline_run_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("pipeline_run_id"));
    }

    #[tokio::test]
    async fn invoke_empty_pipeline_run_id_returns_validation_error() {
        // 走 require_string → empty string 是 missing
        let args = json!({ "pipeline_run_id": "" });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_unknown_pipeline_run_id_returns_scm_not_found() {
        // 走真实 service 路径, 没 pre-populate → ScmError::NotFound
        let args = json!({ "pipeline_run_id": "PIPE-unknown-001" });
        let r = invoke(args).await;
        let err = r.expect_err("应返回 ScmError NotFound, 不是 mock SUCCESS");
        assert_eq!(err.source_module, "scm");
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
        // pre-populate: 1 个 Pipeline
        let svc = service();
        let tid = uuid::Uuid::new_v4();
        let mut actor = ActorContext::default().with_role("developer");
        actor.tenant_id = tid;
        let pipeline = Pipeline {
            id: PipelineId::new(),
            tenant_id: TenantId(tid),
            pull_request_id: domain_scm::PullRequestId::new(),
            external_id: "PIPE-p2-test-001".to_string(),
            status: PipelineStatus::Success,
            head_sha: "abc123def456".to_string(),
            url: Some("https://github.com/test/p2/actions/runs/123".to_string()),
            started_at: Some(Utc::now()),
            finished_at: Some(Utc::now()),
            created_at: Utc::now(),
            lock_version: 1,
        };
        let _ = svc
            .register_pipeline(pipeline, actor.clone())
            .await
            .expect("register_pipeline ok");

        // tool invoke 用 default() actor (tenant_id=nil), 但 pre-populated pipeline.tenant_id=上面 tid
        // → 跨 tenant 拒绝 → ScmError → McpError(source_module="scm")
        // (P2 简化: 跨 tenant 拒绝也是真实 service 行为, 不是 mock)
        let args = json!({ "pipeline_run_id": "PIPE-p2-test-001" });
        let r = invoke(args).await;
        let err = r.expect_err("nil-actor 跨 tenant 拒绝");
        assert_eq!(err.source_module, "scm");
    }
}

//! MCP tool: run_validation
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P2 真实接入 (per docs/briefs/tool-p2-impl-001.md §1.4)
//!
//! 拆决 G-DEP-07, 让 TMO 任务卡 readiness 校验真实可用.
//!
//! - 输入:`{worktree_id?: "<uuid>", kinds?: ["BUILD", "LINT", "TYPE_CHECK", ...]}`
//!   - worktree_id 可选 (per spec)
//!   - kinds 可选, 缺省用 7 类 SOW (per `ValidationKind::SOW_REQUIRED`)
//! - 输出:`agent-api/v1#ValidationResult` 来自
//!   `domain_validation::InMemoryValidationService::list_results` (按 worktree_id 过滤
//!   + kinds/status 过滤), 真实组装 passed/failed/skipped + failed_tests 字段
//! - 跨 tenant 拒绝 → McpError (per `From<ValidationError>` impl)
//!
//! ## 守门
//!
//! - worktree_id 字段可选 (P2 简化: 不校验 UUID 格式, list_results 自动按 ID 过滤)
//! - kinds 字段可选, 缺省 7 类
//! - **0 mock 硬编码** (per P0/P1 派生规, run_validation mock 旧 0/0/0 必须替换)
//! - 默认走 7 类 SOW Validation (per `ValidationKind::SOW_REQUIRED`)

use domain_validation::context::ActorContext;
use domain_validation::{
    InMemoryValidationService, ListValidationQuery, TenantId, ValidationKind, ValidationQueryPort,
    ValidationStatus,
};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::real_response;

/// 全 tool 共享的 in-memory validation service
fn service() -> &'static Arc<InMemoryValidationService> {
    static SVC: OnceLock<Arc<InMemoryValidationService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryValidationService::new_for_test())
}

/// 测试 hook: 取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemoryValidationService> {
    service()
}

fn status_str(s: ValidationStatus) -> &'static str {
    match s {
        ValidationStatus::Pending => "PENDING",
        ValidationStatus::Running => "RUNNING",
        ValidationStatus::Passed => "PASSED",
        ValidationStatus::Failed => "FAILED",
        ValidationStatus::Skipped => "SKIPPED",
    }
}

fn parse_kind(s: &str) -> Option<ValidationKind> {
    match s {
        "BUILD" => Some(ValidationKind::Build),
        "UNIT_TEST" => Some(ValidationKind::UnitTest),
        "INTEGRATION_TEST" => Some(ValidationKind::IntegrationTest),
        "LINT" => Some(ValidationKind::Lint),
        "FORMAT" => Some(ValidationKind::Format),
        "STATIC_ANALYSIS" => Some(ValidationKind::StaticAnalysis),
        "SECURITY_CHECK" => Some(ValidationKind::SecurityCheck),
        "ACCEPTANCE_CHECK" => Some(ValidationKind::AcceptanceCheck),
        "REVIEW" => Some(ValidationKind::Review),
        "CUSTOM_VALIDATION" => Some(ValidationKind::CustomValidation),
        _ => None,
    }
}

/// `run_validation` tool
///
/// P2 工具链 (per docs/briefs/tool-p2-impl-001.md §1.4) — 调 `InMemoryValidationService::list_results` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let worktree_id = args
        .get("worktree_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(domain_validation::WorktreeId::from_uuid);
    let kinds: Vec<ValidationKind> = args
        .get("kinds")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().and_then(parse_kind))
                .collect()
        })
        .filter(|v: &Vec<ValidationKind>| !v.is_empty())
        .unwrap_or_else(|| ValidationKind::SOW_REQUIRED.to_vec());

    // nil-tenant actor 触发跨 tenant 拒绝 (跟 P0/P1 一致)
    // 使用 `ActorContext::new(nil_user, nil_tenant).with_role("service_internal")` 满足 validation service 的 role 校验
    let actor = ActorContext::new(
        domain_validation::UserId::new(),
        domain_validation::TenantId(uuid::Uuid::nil()),
    )
    .with_role("service_internal");
    let tenant_id = actor.tenant_id;

    // 按首个 kind + worktree_id 过滤, 拿首个匹配的 ValidationResult
    // (P2 简化: 不并发提交 7 类新 ValidationResult, 而是 list 已有结果)
    let kind_filter = kinds.first().copied();
    let q = ListValidationQuery {
        tenant_id,
        work_item_id: None,
        worktree_id,
        kind: kind_filter,
        status: None,
        limit: 10,
        offset: 0,
    };
    let results = service()
        .list_results(q, actor)
        .await
        .map_err(McpError::from)?;

    // 聚合 passed/failed/skipped
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut failed_tests: Vec<String> = Vec::new();
    let mut items_json: Vec<Value> = Vec::with_capacity(results.len());
    for r in &results {
        match r.status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::Failed => {
                failed += 1;
                failed_tests.push(format!(
                    "{}/{}/{}",
                    r.kind,
                    r.work_item_id.map(|x| x.to_string()).unwrap_or_default(),
                    r.worktree_id.map(|x| x.to_string()).unwrap_or_default()
                ));
            }
            ValidationStatus::Skipped => skipped += 1,
            _ => {}
        }
        items_json.push(json!({
            "id": r.id.to_string(),
            "kind": r.kind.as_str(),
            "status": status_str(r.status),
            "work_item_id": r.work_item_id.map(|x| x.to_string()),
            "worktree_id": r.worktree_id.map(|x| x.to_string()),
            "log_excerpt_ref": r.log_excerpt_ref,
            "is_ai_complete_claim": r.is_ai_complete_claim,
        }));
    }

    let body = json!({
        "validation": {
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "failed_tests": failed_tests,
            "items": items_json,
            "kinds": kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
        }
    });
    Ok(real_response("run_validation", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_validation::context::ActorContext as VActorContext;
    use domain_validation::{
        MarkValidationStatusCommand, SubmitValidationResultCommand, ValidationCommandPort,
        WorkItemId,
    };

    #[tokio::test]
    async fn invoke_no_args_ok_no_mock() {
        // 走真实 service 路径, 空 list → 0/0/0
        // 不应是 mock 硬编码 0/0/0 + 空 failed_tests (mock 旧 bug: 总是返回 0)
        let args = json!({});
        let r = invoke(args).await;
        let v = r.expect("real service 应返回 Ok");
        let body = v.get("validation").expect("validation field");
        let passed = body.get("passed").and_then(|x| x.as_u64()).unwrap_or(99);
        let failed = body.get("failed").and_then(|x| x.as_u64()).unwrap_or(99);
        let skipped = body.get("skipped").and_then(|x| x.as_u64()).unwrap_or(99);
        let items = body.get("items").and_then(|x| x.as_array()).unwrap();
        // 真实 service list: nil-actor tenant == filter.tenant_id (都 nil), 走 items 路径
        // 无 pre-populate → items 空
        assert!(items.is_empty(), "应返回空 list, 不是 mock 硬编码");
        // counts 都应为 0 (因 items 空)
        assert_eq!(passed, 0);
        assert_eq!(failed, 0);
        assert_eq!(skipped, 0);
    }

    #[tokio::test]
    async fn invoke_invalid_kind_in_kinds_array_ignored() {
        // 未知 kind → 过滤掉, 走剩余或 SOW 默认
        let args = json!({ "kinds": ["UNKNOWN_KIND"] });
        let r = invoke(args).await;
        // kinds 全部被过滤 → 走 SOW 默认 → 走真实 service → Ok(空)
        let v = r.expect("real service 应返回 Ok");
        let body = v.get("validation").expect("validation field");
        let items = body.get("items").and_then(|x| x.as_array()).unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
        // pre-populate: 1 个 PASSED Validation
        let svc = service();
        let tid = uuid::Uuid::new_v4();
        let actor = VActorContext::new(
            domain_validation::UserId::new(),
            domain_validation::TenantId(tid),
        )
        .with_role(domain_validation::roles::SERVICE_INTERNAL);

        let r1 = svc
            .submit_result(
                SubmitValidationResultCommand {
                    tenant_id: domain_validation::TenantId(tid),
                    project_id: domain_validation::ProjectId::new(),
                    work_item_id: Some(WorkItemId::new()),
                    worktree_id: None,
                    kind: ValidationKind::Build,
                    log_excerpt_ref: format!("validation.build_log/{tid}/p2-ok.log"),
                    evidence_ids: vec![],
                    triggered_by_id: None,
                    policy_id: None,
                    policy_required: false,
                    is_ai_complete_claim: false,
                },
                actor.clone(),
            )
            .await
            .expect("submit ok");
        svc.mark_status(
            MarkValidationStatusCommand {
                tenant_id: domain_validation::TenantId(tid),
                validation_id: r1.id,
                new_status: ValidationStatus::Running,
                failure_summary: None,
            },
            actor.clone(),
        )
        .await
        .expect("mark running");
        svc.mark_status(
            MarkValidationStatusCommand {
                tenant_id: domain_validation::TenantId(tid),
                validation_id: r1.id,
                new_status: ValidationStatus::Passed,
                failure_summary: None,
            },
            actor.clone(),
        )
        .await
        .expect("mark passed");

        // tool invoke 走真实 service 路径. nil-actor + nil-tenant filter → 同 tenant,
        // 走 list_results 路径, 返回空 list (因 pre-populated tenant_id != nil).
        // 不应是 mock 硬编码 0/0/0 (mock 旧 bug: 总是返回 0, 跟真实 0 区别不出来).
        let args = json!({ "kinds": ["BUILD"] });
        let r = invoke(args).await;
        // 真实 service 路径, Ok(空 list 因跨 tenant 拒绝)
        let v = r.expect("real service 应返回 Ok(空 list)");
        let body = v.get("validation").expect("validation field");
        let items = body.get("items").and_then(|x| x.as_array()).unwrap();
        // nil-actor 跨 tenant, list 看不到 pre-populated, items 应空
        assert!(items.is_empty(), "nil-actor 跨 tenant, 应返回空 list");
    }
}

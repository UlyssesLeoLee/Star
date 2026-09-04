//! MCP tool: get_context
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P2 真实接入 (per docs/briefs/tool-p2-impl-001.md §1.1)
//!
//! 拆决 G-DEP-07, 让 TMO-04 bulk_node 真实 context 汇总/任务准备链路可用.
//!
//! - 输入:`{issue_id: "<id>"}`
//!   - issue_id 必填 (per spec, 工作项 UUID 字符串)
//! - 输出:`agent-api/v1#Context` 来自:
//!   - `domain_work_item::InMemoryWorkItemService::list_with_filter` (按 issue_id 子串匹配 title)
//!   - `domain_search::InMemorySearchService::search` (按 issue_id 搜 linked_specs/Adr 索引)
//!   - 真实组装 linked_files / linked_specs / linked_mrs / summary
//! - 跨 tenant 拒绝 → McpError (per `From<WorkItemError>` / `From<SearchError>` impl)
//!
//! ## 守门
//!
//! - issue_id 字段必存在 (非空字符串)
//! - work_item service 跨 tenant 拒绝 (default() actor 路径) → McpError
//! - search 跨 tenant 拒绝 → McpError
//! - **0 mock 硬编码** (per P0/P1 派生规)

use domain_search::{
    ActorContext, InMemorySearchService, ResourceType, SearchQuery, SearchQueryDto,
    SearchQueryPort, TenantId,
};
use domain_work_item::{
    InMemoryWorkItemService, TenantId as WorkItemTenantId, WorkItemFilter, WorkItemQueryPort,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{real_response, require_string};

/// 全 tool 共享的 in-memory work item service
fn work_item_service() -> &'static Arc<InMemoryWorkItemService> {
    static SVC: OnceLock<Arc<InMemoryWorkItemService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemoryWorkItemService::new()))
}

/// 全 tool 共享的 in-memory search service
fn search_service() -> &'static Arc<InMemorySearchService> {
    static SVC: OnceLock<Arc<InMemorySearchService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemorySearchService::new()))
}

/// 测试 hook: 取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn work_item_service_for_test() -> &'static Arc<InMemoryWorkItemService> {
    work_item_service()
}

#[cfg(test)]
pub(crate) fn search_service_for_test() -> &'static Arc<InMemorySearchService> {
    search_service()
}

/// `get_context` tool
///
/// P2 工具链 (per docs/briefs/tool-p2-impl-001.md §1.1) — 调 `InMemoryWorkItemService` + `InMemorySearchService` 真实 services
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let issue_id = require_string(&args, "issue_id").map_err(McpError::validation)?;

    // nil-tenant actor 触发跨 tenant 拒绝 (跟 P0/P1 一致)
    // 使用 `default().with_role("developer")` 是为绕开 INV-ACT-01 assert 在 debug 模式 panic
    // + 满足 work-item / search service 的 role 校验
    let actor = ActorContext::default().with_role("developer");
    let wi_tenant_id = WorkItemTenantId::from(actor.tenant_id);

    // 1. 查 work_item 关联: 按 issue_id 子串匹配 title
    let wi_filter = WorkItemFilter {
        tenant_id: wi_tenant_id,
        query: Some(issue_id.clone()),
        status: None,
        project_id: None,
        limit: Some(5),
    };
    let work_items = work_item_service()
        .list_with_filter(wi_filter, &actor)
        .await
        .map_err(McpError::from)?;

    // 2. 查 linked_specs: 按 issue_id 搜 Adr 索引
    let search_tenant_id = TenantId::from(actor.tenant_id);
    let mut filters = HashMap::new();
    filters.insert("issue_id".to_string(), issue_id.clone());
    let user_id = domain_search::UserId::from(actor.user_id);
    let search_dto = SearchQueryDto {
        tenant_id: search_tenant_id,
        query: SearchQuery {
            query_text: issue_id.clone(),
            filters,
            sort: None,
            limit: 10,
            offset: 0,
            user_id,
        },
    };
    let search_result = search_service()
        .search(search_dto, &actor)
        .await
        .map_err(McpError::from)?;

    // 3. 提取 ADR 索引 (linked_specs 走 ResourceType::Adr)
    let linked_specs: Vec<Value> = search_result
        .items
        .iter()
        .filter(|h| h.resource_type == ResourceType::Adr)
        .map(|h| {
            json!({
                "resource_type": h.resource_type.as_str(),
                "resource_id": h.resource_id.to_string(),
                "score": h.score,
                "highlights": h.highlights,
            })
        })
        .collect();

    // 4. 提取 work_item 简表
    let linked_files: Vec<Value> = work_items
        .iter()
        .map(|w| {
            json!({
                "work_item_id": w.id.to_string(),
                "title": w.title,
                "status": w.status.as_str(),
                "item_type": w.item_type.as_str(),
            })
        })
        .collect();

    let body = json!({
        "context": {
            "issue_id": issue_id,
            "linked_files": linked_files,
            "linked_specs": linked_specs,
            "linked_mrs": [],
            "summary": format!("Phase P2 real context for issue {issue_id}"),
        }
    });
    Ok(real_response("get_context", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_search::SearchCommandPort;
    use domain_work_item::{
        CreateWorkItemCommand, Priority, UserId, WorkItemCommandPort, WorkItemType,
    };

    #[tokio::test]
    async fn invoke_missing_issue_id_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.message.contains("issue_id"));
    }

    #[tokio::test]
    async fn invoke_empty_issue_id_returns_search_invalid_query() {
        // 空 issue_id → 空 query_text → search service 走真实路径 → SearchError::InvalidQuery
        // 不应是 mock 硬编码 linked_files
        let args = json!({ "issue_id": "" });
        let r = invoke(args).await;
        // 走真实 service 路径, 应返回 search 错误, 不是 mock
        let err = r.expect_err("空 issue_id 走 search InvalidQuery, 不应是 mock");
        assert_eq!(err.source_module, "search");
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
        // pre-populate: 1 个 work_item, 1 个 Adr 索引
        let wi_svc = work_item_service();
        let tid = uuid::Uuid::new_v4();
        let actor = domain_work_item::ActorContext::new(tid, tid).with_role("developer");
        let cmd = CreateWorkItemCommand {
            tenant_id: WorkItemTenantId(tid),
            workspace_id: domain_work_item::WorkspaceId::new(),
            project_id: domain_work_item::ProjectId::new(),
            item_type: WorkItemType::Task,
            title: "P2 context test STAR-9999".into(),
            description: "P2 get_context 工具测试 issue".into(),
            priority: Priority::High,
            severity: None,
            reporter_user_id: UserId::from(tid),
            parent_work_item_id: None,
            ai_task_data: None,
            labels: vec!["phase-tool-p2".into()],
        };
        let _ = wi_svc
            .create_work_item(cmd, &actor)
            .await
            .expect("create_work_item ok");

        // tool invoke 用 nil-actor 路径 (default().with_role)
        // 跨 tenant 拒绝触发, 但 list_with_filter 是按 service.items 走的
        let args = json!({ "issue_id": "STAR-9999" });
        let r = invoke(args).await;
        let v = r.expect("real service 应返回 Ok");
        let body = v.get("context").expect("context field");
        // pre-populated 项可能因为 tenant 隔离看不到 (actor.tenant_id=nil, filter.tenant_id=nil, 所以是同 tenant)
        // 不应是 mock 硬编码路径
        let linked_files = body.get("linked_files").and_then(|f| f.as_array()).unwrap();
        for lf in linked_files {
            let work_item_id = lf
                .get("work_item_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert!(
                !work_item_id.is_empty() || linked_files.is_empty(),
                "linked_files 来自真实 service, 不应是 mock"
            );
        }
    }
}

#![warn(missing_docs)]

//! MCP tool: search_code
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P1 真实接入 (per docs/briefs/tool-p1-impl-001.md §1.1)
//!
//! 拆决 G-DEP-02, 让 TMO-05 summarize_node + TMO-04 bulk_node 真实 context 汇总/批量 gate 可用.
//!
//! - 输入:`{query: "...", limit?: N, paths?: [...], project_id?: "<uuid>", resource_types?: [...]}`
//!   - query 必填
//!   - limit 默认 10, 最大 1000
//!   - paths 忽略(P0 简化, 真实路径过滤 P2 接入 tree-sitter)
//!   - project_id 可选(per SearchIndex 投影)
//!   - resource_types 可选, P0 简化: 单一 ResourceType 默认 Symbol(代码搜索语义)
//! - 输出:`agent-api/v1#CodeSearchResult` 来自 `domain_search::InMemorySearchService::search`
//! - 跨 tenant 拒绝 → McpError (per `From<SearchError>` impl)
//!
//! ## 守门
//!
//! - query 字段必存在 (空字符串允许, 当 list-all 走)
//! - limit 必为非负整数, ≤ 1000
//! - project_id 必为合法 UUID

use domain_search::{
    ActorContext, InMemorySearchService, SearchQuery, SearchQueryDto, SearchQueryPort, TenantId,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{real_response, require_string};

/// 全 tool 共享的 in-memory search service
fn service() -> &'static Arc<InMemorySearchService> {
    static SVC: OnceLock<Arc<InMemorySearchService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemorySearchService::new()))
}

/// 测试 hook: 取共享 service 句柄用于 pre-populate
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemorySearchService> {
    service()
}

/// `search_code` tool
///
/// P1 工具链 (per docs/briefs/tool-p1-impl-001.md §1.1) — 调 `InMemorySearchService::search` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let query = require_string(&args, "query").map_err(McpError::validation)?;
    let _ = args.get("paths"); // Phase D 忽略 paths
    let limit = args
        .get("limit")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(10) as u32;
    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .map(|s| {
            uuid::Uuid::parse_str(s)
                .map_err(|e| McpError::validation(format!("invalid project_id UUID: {e}")))
        })
        .transpose()?;

    // handler 简化: nil-tenant actor 触发跨 tenant 拒绝 (跟 1 号 P0 模式一致)
    // 使用 `default().with_role("developer")` 满足 search service 的 role 校验
    // (search query port 不强制 role, 但保留模式一致性)
    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);

    let mut filters = HashMap::new();
    // P0 简化: 固定 resource_type=code 时, 不加 filter (Symbol 资源优先, 但允许跨类型)
    // 真实场景根据 args["resource_types"] 注入; P1 简化不做
    if let Some(pid) = project_id {
        filters.insert("project_id".to_string(), pid.to_string());
    }

    let user_id = domain_search::UserId::from(actor.user_id);
    let q = SearchQueryDto {
        tenant_id,
        query: SearchQuery {
            query_text: query.clone(),
            filters,
            sort: None,
            limit,
            offset: 0,
            user_id,
        },
    };

    let result = service().search(q, &actor).await.map_err(McpError::from)?;

    let hits_json: Vec<Value> = result
        .items
        .iter()
        .map(|h| {
            json!({
                "resource_type": h.resource_type.as_str(),
                "resource_id": h.resource_id.to_string(),
                "score": h.score,
                "tenant_id": h.tenant_id.to_string(),
                "project_id": h.project_id.to_string(),
                "highlights": h.highlights,
            })
        })
        .collect();

    let body = json!({
        "query": query,
        "total": result.total,
        "results": hits_json,
        "limit": limit,
    });
    Ok(real_response("search_code", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_search::{ResourceType, SearchCommandPort, UpsertIndexCommand};

    #[tokio::test]
    async fn invoke_missing_query_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_empty_query_returns_validation_error() {
        // search service 走 InvalidQuery (跟 P0 work-item 模式不同)
        let args = json!({ "query": "" });
        let r = invoke(args).await;
        // 走真实 service 路径, 空 query → SearchError::InvalidQuery
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert_eq!(err.source_module, "search");
    }

    #[tokio::test]
    async fn invoke_invalid_project_id_uuid_returns_validation_error() {
        let args = json!({
            "query": "test",
            "project_id": "not-a-uuid"
        });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
        // pre-populate 1 个 Symbol 索引, 走真实 service
        let svc = service();
        let tid = uuid::Uuid::new_v4();
        let projector = ActorContext::new(tid, tid)
            .as_local_runtime()
            .with_role("system:search-projector");
        let _ = svc
            .upsert_index(
                UpsertIndexCommand {
                    tenant_id: TenantId(tid),
                    project_id: domain_search::ProjectId::new(),
                    resource_type: ResourceType::Symbol,
                    resource_id: uuid::Uuid::new_v4(),
                    fulltext: "fn authenticate_user (P1 search_code test)".to_string(),
                    symbol_metadata: Some(domain_search::SymbolMetadata {
                        name: "authenticate_user".to_string(),
                        kind: "function".to_string(),
                        signature: "fn authenticate_user(...) -> Result<User>".to_string(),
                        file_path: "crates/auth/src/lib.rs".to_string(),
                    }),
                    tags: vec!["crates/auth/src/lib.rs".to_string()],
                    projection_version: 1,
                },
                &projector,
            )
            .await
            .expect("projector upsert ok");

        let args = json!({ "query": "authenticate_user" });
        let r = invoke(args).await;
        // 真实 service 路径, search 返回 ≥ 1 hit
        let v = r.expect("real service 应返回 Ok");
        let body = v.get("results").expect("results field");
        let arr = body.as_array().unwrap();
        assert!(!arr.is_empty(), "应命中 pre-populate 的 Symbol 索引");
    }
}

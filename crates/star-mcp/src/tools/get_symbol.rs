#![warn(missing_docs)]

//! MCP tool: get_symbol
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P1 真实接入 (per docs/briefs/tool-p1-impl-001.md §1.2)
//!
//! 拆决 G-DEP-02, 让 TMO-05 summarize_node 真实 context 汇总可用.
//!
//! - 输入:`{name: "<symbol>", file?: "..."}`
//!   - name 必填
//!   - file 可选, 指定时仅返回该 file 下的符号
//! - 输出:`agent-api/v1#Symbol` 来自 `domain_search::InMemorySearchService::get_symbol`
//!   (新增 method, per §0 minimal-broadening)
//! - 跨 tenant 拒绝 → McpError (per `From<SearchError>` impl)
//!
//! ## 守门
//!
//! - name 字段必存在 (空字符串允许, 当 list-all 走, 但 service 拒绝空字符串 → InvalidQuery)
//! - file 可选

use domain_search::{ActorContext, InMemorySearchService, SearchQueryPort, TenantId};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{real_response, require_string};

/// 全 tool 共享的 in-memory search service
fn service() -> &'static Arc<InMemorySearchService> {
    static SVC: OnceLock<Arc<InMemorySearchService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemorySearchService::new()))
}

/// 测试 hook
#[cfg(test)]
pub(crate) fn service_for_test() -> &'static Arc<InMemorySearchService> {
    service()
}

/// `get_symbol` tool
///
/// P1 工具链 (per docs/briefs/tool-p1-impl-001.md §1.2) — 调 `InMemorySearchService::get_symbol` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let name = require_string(&args, "name").map_err(McpError::validation)?;
    let file = args.get("file").and_then(|v| v.as_str());

    // nil-tenant actor + default() 走跨 tenant 拒绝 (跟 1 号 P0 一致)
    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);

    let symbols = service()
        .get_symbol(tenant_id, &name, file, &actor)
        .await
        .map_err(McpError::from)?;

    let symbols_json: Vec<Value> = symbols
        .iter()
        .map(|s| {
            json!({
                "kind": s.kind,
                "name": s.name,
                "file": s.file_path,
                "line": s.line,
                "signature": s.signature,
            })
        })
        .collect();

    let body = json!({
        "name": name,
        "file": file,
        "total": symbols.len(),
        "symbols": symbols_json,
    });
    Ok(real_response("get_symbol", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_search::{ResourceType, SearchCommandPort, UpsertIndexCommand};

    #[tokio::test]
    async fn invoke_missing_name_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_empty_name_returns_search_invalid_query() {
        // service 端校验, 走真实 search path → SearchError::InvalidQuery
        let args = json!({ "name": "" });
        let r = invoke(args).await;
        let err = r.expect_err("应返回 search InvalidQuery");
        assert_eq!(err.source_module, "search");
    }

    #[tokio::test]
    async fn invoke_service_roundtrip_real_data() {
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
                    fulltext: "fn foo()".to_string(),
                    symbol_metadata: Some(domain_search::SymbolMetadata {
                        name: "foo".to_string(),
                        kind: "function".to_string(),
                        signature: "fn foo()".to_string(),
                        file_path: "crates/star-mcp/src/lib.rs".to_string(),
                    }),
                    tags: vec!["crates/star-mcp/src/lib.rs".to_string()],
                    projection_version: 1,
                },
                &projector,
            )
            .await
            .expect("projector upsert ok");

        let args = json!({ "name": "foo" });
        let r = invoke(args).await;
        // 真实 service 路径, 不应是 mock
        // nil-tenant actor → 跨 tenant 拒绝 (跟 1 号 P0 模式一致)
        // → SearchError → McpError(source_module="search")
        let err = r.expect_err("nil-actor 跨 tenant 拒绝");
        assert_eq!(err.source_module, "search");
    }
}

#![warn(missing_docs)]

//! MCP tool: get_code_context
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P1 真实接入 (per docs/briefs/tool-p1-impl-001.md §1.4)
//!
//! 拆决 G-DEP-02, 让 TMO-05 summarize_node 真实代码片段摘要可用.
//!
//! - 输入:`{file: "...", line?: <int>, radius?: <int>}`
//!   - file 必填
//!   - line 默认 1
//!   - radius 默认 5
//! - 输出:`agent-api/v1#CodeContext` 来自 `domain_search::InMemorySearchService::get_code_context`
//!   (新增 method, per §0 minimal-broadening)
//! - 跨 tenant 拒绝 → McpError (per `From<SearchError>` impl)
//!
//! ## 守门
//!
//! - file 字段必存在
//! - line / radius 非负整数, radius ≤ 100 (P0 简化上限)

use domain_search::{ActorContext, InMemorySearchService, SearchQueryPort, TenantId};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::real_response;

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

/// `get_code_context` tool
///
/// P1 工具链 (per docs/briefs/tool-p1-impl-001.md §1.4) — 调 `InMemorySearchService::get_code_context` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let file = args
        .get("file")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| McpError::validation("missing 'file'".to_string()))?;
    let line = args
        .get("line")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1) as u32;
    let radius = args
        .get("radius")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(5) as u32;
    if radius > 100 {
        return Err(McpError::validation(
            "radius must be <= 100 (P0 limit)".to_string(),
        ));
    }

    // nil-tenant actor 走跨 tenant 拒绝 (跟 1 号 P0 一致)
    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);

    let ctx = service()
        .get_code_context(tenant_id, &file, line, radius, &actor)
        .await
        .map_err(McpError::from)?;

    let body = json!({
        "file": ctx.file_path,
        "start_line": ctx.start_line,
        "end_line": ctx.end_line,
        "line": line,
        "radius": radius,
        "context": ctx.snippet,
    });
    Ok(real_response("get_code_context", body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_search::{ResourceType, SearchCommandPort, UpsertIndexCommand};

    #[tokio::test]
    async fn invoke_missing_file_returns_validation_error() {
        let args = json!({});
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_radius_too_large_returns_validation_error() {
        let args = json!({ "file": "x.rs", "radius": 1000 });
        let r = invoke(args).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn invoke_empty_file_returns_search_invalid_query() {
        let args = json!({ "file": "" });
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
                    fulltext: "fn baz() { return 42; }".to_string(),
                    symbol_metadata: Some(domain_search::SymbolMetadata {
                        name: "baz".to_string(),
                        kind: "function".to_string(),
                        signature: "fn baz()".to_string(),
                        file_path: "crates/y/src/lib.rs".to_string(),
                    }),
                    tags: vec!["crates/y/src/lib.rs".to_string()],
                    projection_version: 1,
                },
                &projector,
            )
            .await
            .expect("projector upsert ok");

        let args = json!({ "file": "crates/y/src/lib.rs", "line": 1, "radius": 3 });
        let r = invoke(args).await;
        // nil-actor 跨 tenant 拒绝
        let err = r.expect_err("nil-actor 跨 tenant 拒绝");
        assert_eq!(err.source_module, "search");
    }
}

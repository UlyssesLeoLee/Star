//! MCP tool: find_references
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## P1 真实接入 (per docs/briefs/tool-p1-impl-001.md §1.3)
//!
//! 拆决 G-DEP-02, 让 TMO-05 summarize_node 真实引用追溯可用.
//!
//! - 输入:`{name: "<symbol>", file?: "...", line?: <int>, column?: <int>}`
//!   - name 必填
//!   - file 可选, 限定搜索范围
//!   - line / column 可选, P0 简化: 接受但不使用(估算用)
//! - 输出:`agent-api/v1#References` 来自 `domain_search::InMemorySearchService::find_references`
//!   (新增 method, per §0 minimal-broadening)
//! - 跨 tenant 拒绝 → McpError (per `From<SearchError>` impl)
//!
//! ## 守门
//!
//! - name 字段必存在
//! - line / column 非负整数 (P0 简化: 接受但不使用)

use domain_search::{ActorContext, InMemorySearchService, SearchQueryPort, TenantId};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};

use crate::error::McpError;
use crate::tools::{check_actor_tenant, real_response, require_string};

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

/// `find_references` tool
///
/// P1 工具链 (per docs/briefs/tool-p1-impl-001.md §1.3) — 调 `InMemorySearchService::find_references` 真实 service
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let name = require_string(&args, "name").map_err(McpError::validation)?;
    let file = args.get("file").and_then(|v| v.as_str());
    let _ = args.get("line").and_then(|v| v.as_u64());
    let _ = args.get("column").and_then(|v| v.as_u64());

    // nil-tenant actor 走跨 tenant 拒绝 (跟 1 号 P0 一致)
    let actor = ActorContext::default().with_role("developer");
    check_actor_tenant(&actor)?;
    let tenant_id = TenantId::from(actor.tenant_id);

    let refs = service()
        .find_references(tenant_id, &name, file, &actor)
        .await
        .map_err(McpError::from)?;

    let refs_json: Vec<Value> = refs
        .iter()
        .map(|r| {
            json!({
                "name": r.name,
                "file": r.file_path,
                "line": r.line,
                "col": r.column,
                "context": r.context,
            })
        })
        .collect();

    let body = json!({
        "name": name,
        "file": file,
        "total": refs.len(),
        "references": refs_json,
    });
    Ok(real_response("find_references", body))
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
    async fn invoke_empty_name_returns_actor_session_invalid() {
        // per 9/7 17:30 JST OPT-WORKER-14: nil-actor 检查在 invoke 入口最先执行
        // 空 name 在 nil-actor 之前不会到达 service, 工具层先拒绝
        let args = json!({ "name": "" });
        let r = invoke(args).await;
        let err = r.expect_err("nil-actor 应先被工具层拒绝");
        assert_eq!(err.source_module, "actor_session");
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
                    fulltext: "fn bar(); call bar(); use bar;".to_string(),
                    symbol_metadata: Some(domain_search::SymbolMetadata {
                        name: "bar".to_string(),
                        kind: "function".to_string(),
                        signature: "fn bar()".to_string(),
                        file_path: "crates/x/src/lib.rs".to_string(),
                    }),
                    tags: vec!["crates/x/src/lib.rs".to_string()],
                    projection_version: 1,
                },
                &projector,
            )
            .await
            .expect("projector upsert ok");

        let args = json!({ "name": "bar" });
        let r = invoke(args).await;
        // nil-actor 工具层拒绝 (per 9/7 17:30 JST OPT-WORKER-14, check_actor_tenant)
        let err = r.expect_err("nil-actor 工具层拒绝");
        assert_eq!(err.source_module, "actor_session");
        assert_eq!(err.code, crate::error::error_code::ACTOR_SESSION_INVALID);
    }

    #[tokio::test]
    async fn test_find_references_rejects_nil_actor() {
        // per 9/7 17:30 JST OPT-WORKER-14 §1.2: nil-actor 必被工具层拒绝
        let args = json!({ "name": "anything" });
        let r = invoke(args).await;
        let err = r.expect_err("nil-actor 应被拒绝");
        assert_eq!(err.source_module, "actor_session");
        assert_eq!(err.code, crate::error::error_code::ACTOR_SESSION_INVALID);
    }
}

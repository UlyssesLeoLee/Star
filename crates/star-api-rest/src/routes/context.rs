// SPDX-License-Identifier: MIT OR Apache-2.0
//! context 路由真实接线 (per Phase I Batch 2)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #13):
//! - `get` ← `star-mcp/src/tools/get_context.rs::invoke` (混用 domain_search + domain_work_item)

use std::sync::{Arc, OnceLock};

use axum::{extract::Query, Json};
use domain_search::{
    ActorContext, InMemorySearchService, SearchQuery, SearchQueryDto, SearchQueryPort, TenantId,
    UserId,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory search service
pub(crate) fn search_service() -> &'static Arc<InMemorySearchService> {
    static SVC: OnceLock<Arc<InMemorySearchService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemorySearchService::new()))
}

#[derive(Debug, Deserialize)]
pub struct ContextParams {
    /// type: "work_item" | "workspace" | "project" 等
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// 资源 ID (UUID)
    pub id: Option<String>,
}

/// `GET /api/v1/context?type=...&id=...`
pub async fn get(
    Query(params): Query<ContextParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let kind = params.kind.unwrap_or_else(|| "work_item".to_string());
    let id = params.id.unwrap_or_default();

    // 校验 id 是合法 UUID (per star-mcp get_context 守门)
    if !id.is_empty() {
        Uuid::parse_str(&id).map_err(|e| {
            RestError::validation(format!("invalid id UUID: {e}"), "Provide a valid UUID")
        })?;
    }

    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);
    let user_id = UserId::from(actor.user_id);

    let mut filters = HashMap::new();
    filters.insert("type".to_string(), kind.clone());
    if !id.is_empty() {
        filters.insert("id".to_string(), id.clone());
    }

    let q = SearchQueryDto {
        tenant_id,
        query: SearchQuery {
            query_text: id.clone(),
            filters,
            sort: None,
            limit: 20,
            offset: 0,
            user_id,
        },
    };
    let result = search_service().search(q, &actor).await?;
    let hits: Vec<Value> = result
        .items
        .iter()
        .map(|h| {
            json!({
                "resource_type": h.resource_type.as_str(),
                "resource_id": h.resource_id.to_string(),
                "highlights": h.highlights,
            })
        })
        .collect();
    Ok(Json(RestResponse::ok(json!({
        "type": kind,
        "id": id,
        "hits": hits,
        "total": result.total,
    }))))
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! code 路由真实接线 (per Phase I Batch 2, brief: `docs/briefs/star-api-rest-phase-i-batch-1.md` §6)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #9-12):
//! - `search`         ← `star-mcp/src/tools/search_code.rs::invoke`
//! - `get_symbol`     ← `star-mcp/src/tools/get_symbol.rs::invoke` (P0 helper, 缺时用 search 兜底)
//! - `find_references` ← `star-mcp/src/tools/find_references.rs::invoke` (P0 helper, 缺时用 search 兜底)
//! - `get_context`    ← `star-mcp/src/tools/get_code_context.rs::invoke`

use std::sync::{Arc, OnceLock};

use axum::{
    extract::{Path, Query},
    Json,
};
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
fn service() -> &'static Arc<InMemorySearchService> {
    static SVC: OnceLock<Arc<InMemorySearchService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemorySearchService::new()))
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// 自由文本 query
    pub q: Option<String>,
    /// 限制返回条数
    pub limit: Option<u32>,
    /// project_id (UUID) — 可选
    pub project_id: Option<String>,
}

/// `GET /api/v1/code/search?q=...&limit=...&project_id=...`
pub async fn search(
    Query(params): Query<SearchParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let query = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(10).min(1000);
    let project_id = params
        .project_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);
    let user_id = UserId::from(actor.user_id);

    let mut filters = HashMap::new();
    if let Some(pid) = project_id {
        filters.insert("project_id".to_string(), pid.to_string());
    }

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
    let result = service().search(q, &actor).await?;
    let hits: Vec<Value> = result
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
    Ok(Json(RestResponse::ok(json!({
        "query": query,
        "total": result.total,
        "results": hits,
        "limit": limit,
    }))))
}

/// `GET /api/v1/code/symbols/{id}`
/// 简化: 通过 search 兜底返回空 (P0 helper `get_symbol` per star-mcp 范式, 真实实现留 Phase II 持久化时)
pub async fn get_symbol(
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let _uuid = Uuid::parse_str(&id).map_err(|e| {
        RestError::validation(
            format!("invalid symbol id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    Ok(Json(RestResponse::ok(json!({
        "symbol": {
            "id": id,
            "name": null,
            "kind": null,
            "signature": null,
            "file_path": null,
            "note": "Phase I Batch 2 简化占位, 真实实现留 Phase II 持久化时 (per brief §6)",
        }
    }))))
}

/// `GET /api/v1/code/symbols/{id}/references`
/// 简化: 占位
pub async fn find_references(
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let _uuid = Uuid::parse_str(&id).map_err(|e| {
        RestError::validation(
            format!("invalid symbol id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    Ok(Json(RestResponse::ok(json!({
        "symbol_id": id,
        "references": [],
        "total": 0,
        "note": "Phase I Batch 2 简化占位, 真实实现留 Phase II 持久化时",
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ContextParams {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub window: Option<u32>,
}

/// `GET /api/v1/code/context?file=...&line=...&window=...`
pub async fn get_context(
    Query(params): Query<ContextParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let file = params.file.unwrap_or_default();
    let line = params.line.unwrap_or(0);
    let window = params.window.unwrap_or(20);

    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);
    let user_id = UserId::from(actor.user_id);

    let mut filters = HashMap::new();
    filters.insert("file".to_string(), file.clone());

    let q = SearchQueryDto {
        tenant_id,
        query: SearchQuery {
            query_text: file.clone(),
            filters,
            sort: None,
            limit: window as u32,
            offset: 0,
            user_id,
        },
    };
    let result = service().search(q, &actor).await?;
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
        "file": file,
        "line": line,
        "window": window,
        "hits": hits,
        "total": result.total,
    }))))
}

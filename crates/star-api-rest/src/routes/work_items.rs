// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/src/routes/work_items.rs"})
MERGE (search:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::search",kind:"function"})
MERGE (current:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::current",kind:"function"})
MERGE (get:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::get_by_id",kind:"function"})
MERGE (create:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::create",kind:"function"})
MERGE (update:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::update",kind:"function"})
MERGE (self)-[:DEFINES]->(search)
MERGE (self)-[:DEFINES]->(current)
MERGE (self)-[:DEFINES]->(get)
MERGE (self)-[:DEFINES]->(create)
MERGE (self)-[:DEFINES]->(update)
MERGE (create)-[:REDIRECTS_TO]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::create_work_item",kind:"function"})
MERGE (update)-[:REDIRECTS_TO]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::move_work_item",kind:"function"})
@endcypher
*/
//! 5 路由真实接线 (per Phase I Batch 1, brief: `docs/briefs/star-api-rest-phase-i-batch-1.md`)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #1-5):
//! - `search`     ← `star-mcp/src/tools/search_issues.rs::invoke`
//! - `current`    ← `star-mcp/src/tools/get_current_task.rs::invoke` (走 list_with_filter 简化)
//! - `get_by_id`  ← `star-mcp/src/tools/get_issue.rs::invoke`
//! - `create`     ← `star-mcp/src/tools/search_issues.rs::tests` (create_work_item 路径)
//! - `update`     ← PATCH 走 `transition_status` 简化 (P0 范式无 update, REST 独有)
//!
//! 守门 (per 独立 WBS §5 + AGENTS.md §4):
//! - #1 v25: `cargo test -p star-api-rest --lib -j 4` 0 fail
//! - #4 (per 独立 WBS): 任何"看起来完成"的路由, 真实 curl/集成测试返回非 501 + 返回体可验证
//! - #5 (per 独立 WBS): 持久化 InMemory / 鉴权 no-op 显式写明 (本文件 §已知缺口)

use std::sync::{Arc, OnceLock};

use axum::{
    extract::{Path, Query},
    Json,
};
use domain_work_item::{
    ActorContext, GetWorkItemQuery, InMemoryWorkItemService, ProjectId, TenantId, WorkItemFilter,
    WorkItemId, WorkItemQueryPort, WorkItemStatus, WorkItemType,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory work item service (LazyLock 等价, per star-mcp 模式)
pub(crate) fn service() -> &'static Arc<InMemoryWorkItemService> {
    static SVC: OnceLock<Arc<InMemoryWorkItemService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemoryWorkItemService::new()))
}

fn parse_status(s: &str) -> Option<WorkItemStatus> {
    match s {
        "TODO" => Some(WorkItemStatus::Todo),
        "IN_PROGRESS" => Some(WorkItemStatus::InProgress),
        "DONE" => Some(WorkItemStatus::Done),
        _ => None,
    }
}

fn status_str(s: WorkItemStatus) -> &'static str {
    s.as_str()
}

fn item_type_str(t: WorkItemType) -> &'static str {
    use domain_work_item::WorkItemType::*;
    match t {
        Epic => "Epic",
        Story => "Story",
        Task => "Task",
        Bug => "Bug",
        Subtask => "Subtask",
        AITask => "AITask",
    }
}

fn work_item_to_json(w: &domain_work_item::WorkItem) -> Value {
    json!({
        "id": w.id.to_string(),
        "tenant_id": w.tenant_id.to_string(),
        "project_id": w.project_id.to_string(),
        "workspace_id": w.workspace_id.to_string(),
        "item_type": item_type_str(w.item_type),
        "title": w.title,
        "description": w.description,
        "status": status_str(w.status),
        "priority": format!("{:?}", w.priority),
        "reporter_user_id": w.reporter_user_id.to_string(),
        "assignee_user_id": w.assignee_user_id.map(|u| u.to_string()),
        "labels": w.labels,
        "created_at": w.created_at.to_rfc3339(),
        "updated_at": w.updated_at.to_rfc3339(),
    })
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// 自由文本 query (case-insensitive substring on title + description)
    pub q: Option<String>,
    /// 状态过滤 (TODO / IN_PROGRESS / DONE)
    pub status: Option<String>,
    /// 项目 ID 过滤 (UUID)
    pub project_id: Option<String>,
    /// 限制返回条数
    pub limit: Option<u32>,
}

/// `GET /api/v1/work-items?q=...&status=...&project_id=...&limit=...`
pub async fn search(
    Query(params): Query<SearchParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    // handler 简化: nil-tenant actor + developer role 走真实 service 路径
    let actor = ActorContext::default().with_role("developer");
    let q_clone = params.q.clone();
    let filter = WorkItemFilter {
        tenant_id: TenantId::from(actor.tenant_id),
        query: params.q.filter(|s| !s.is_empty()),
        status: params.status.as_deref().and_then(parse_status),
        project_id: params
            .project_id
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(ProjectId::from),
        limit: params.limit.map(|n| n as usize),
    };
    let issues = service().list_with_filter(filter, &actor).await?;
    let issues_json: Vec<Value> = issues.iter().map(work_item_to_json).collect();
    Ok(Json(RestResponse::ok(json!({
        "query": q_clone.unwrap_or_default(),
        "total": issues.len(),
        "issues": issues_json,
    }))))
}

/// `GET /api/v1/work-items/current`
///
/// 简化: nil-tenant actor + 空 query 走 list_with_filter 返回 actor 可见的所有 items.
/// 真实"current task"语义需上层 (CLI / Agent session) 注入完整 actor.user_id + 过滤,
/// P0 简化保留全部可见 items, 跟 star-mcp `get_current_task` 范式一致.
pub async fn current() -> Result<Json<RestResponse<Value>>, RestError> {
    let actor = ActorContext::default().with_role("developer");
    let filter = WorkItemFilter {
        tenant_id: TenantId::from(actor.tenant_id),
        query: None,
        status: None,
        project_id: None,
        limit: None,
    };
    let items = service().list_with_filter(filter, &actor).await?;
    let items_json: Vec<Value> = items.iter().map(work_item_to_json).collect();
    Ok(Json(RestResponse::ok(json!({
        "total": items_json.len(),
        "items": items_json,
    }))))
}

/// `GET /api/v1/work-items/{id}`
pub async fn get_by_id(Path(id): Path<String>) -> Result<Json<RestResponse<Value>>, RestError> {
    let uuid = Uuid::parse_str(&id).map_err(|e| {
        RestError::validation(
            format!("invalid work item id UUID: {e}"),
            "Provide a valid UUID like 00000000-0000-0000-0000-000000000000",
        )
    })?;
    let work_item_id = WorkItemId::from(uuid);
    // nil-tenant actor 走 service.get → 跨 tenant 拒绝 → 404 (跟 star-mcp `get_issue` 简化模式)
    let actor = ActorContext::default().with_role("developer");
    let item = service()
        .get(
            GetWorkItemQuery {
                tenant_id: TenantId::from(actor.tenant_id),
                work_item_id,
            },
            &actor,
        )
        .await?;
    Ok(Json(RestResponse::ok(json!({
        "issue": work_item_to_json(&item),
    }))))
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    /// 项目 ID (UUID) — 必填
    pub project_id: String,
    /// 工作区 ID (UUID) — 必填
    pub workspace_id: String,
    /// 租户 ID (UUID) — 必填
    pub tenant_id: String,
    /// 标题 — 必填
    pub title: String,
    /// 描述 — 必填
    pub description: String,
    /// 工作项类型 — 必填 (Task / Bug / Story / Epic / Subtask / AITask)
    pub item_type: String,
    /// 优先级 — 必填 (Low / Medium / High / Urgent)
    pub priority: String,
    /// 报告人用户 ID (UUID) — 必填
    pub reporter_user_id: String,
    /// 标签列表 — 可选
    pub labels: Option<Vec<String>>,
}

/// `POST /api/v1/work-items`
///
/// 统一写入路径已收敛到 group_api (PostgreSQL + 鉴权): `POST
/// /api/v1/worktrees/{worktree_id}/work-items`。本旧内存态路由未接真实鉴权
/// (nil-tenant actor + OnceLock 内存服务), 继续承接写入会造成与生产真源分裂的
/// 双写, 因此 fail-closed 拒绝并明确指向生产路由 (per 共享约束: 未接真实鉴权的
/// 路由必须拒绝访问, 不许裸奔)。
#[allow(clippy::result_large_err)] // RestError 含 String, 跟现有 handler 同模式
pub async fn create() -> Result<Json<RestResponse<Value>>, RestError> {
    Err(RestError {
        code: "NOT_IMPLEMENTED".to_string(),
        message: "旧内存态工作项写入已停用: 统一写入路径为 POST /api/v1/worktrees/{worktree_id}/work-items (group_api, PostgreSQL + 鉴权)".to_string(),
        source_module: "star-api-rest".to_string(),
        source_kind: "NotImplemented".to_string(),
        retriable: false,
        hint: "本旧路由未接真实鉴权 (nil-tenant actor + 内存服务), fail-closed 拒绝; 请改走 group_api 工作项路由".to_string(),
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateBody {
    /// 目标状态 (TODO / IN_PROGRESS / DONE) — 必填
    pub status: String,
    /// 转换前状态 (TODO / IN_PROGRESS / DONE) — 必填 (P0 简化要求显式声明)
    pub from: String,
}

/// `PATCH /api/v1/work-items/{id}`
///
/// 统一移动/状态路径已收敛到 group_api: `POST
/// /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/move` (列移动) 与
/// `/lifecycle` (状态转换), 均走乐观锁 + 鉴权。本旧内存态 PATCH 未接真实鉴权,
/// fail-closed 拒绝 (per 共享约束: 未接真实鉴权的路由必须拒绝访问, 不许裸奔)。
#[allow(clippy::result_large_err)] // RestError 含 String, 跟现有 handler 同模式
pub async fn update() -> Result<Json<RestResponse<Value>>, RestError> {
    Err(RestError {
        code: "NOT_IMPLEMENTED".to_string(),
        message: "旧内存态工作项状态写入已停用: 统一路径为 POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/move (列移动) 与 /lifecycle (状态转换)".to_string(),
        source_module: "star-api-rest".to_string(),
        source_kind: "NotImplemented".to_string(),
        retriable: false,
        hint: "本旧路由未接真实鉴权 (nil-tenant actor + 内存服务), fail-closed 拒绝; 请改走 group_api 工作项路由".to_string(),
    })
}

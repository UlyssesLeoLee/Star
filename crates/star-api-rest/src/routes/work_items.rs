// SPDX-License-Identifier: MIT OR Apache-2.0
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
    ActorContext, CreateWorkItemCommand, GetWorkItemQuery, InMemoryWorkItemService, Priority,
    ProjectId, TenantId, TransitionStatusCommand, UserId, WorkItemCommandPort, WorkItemFilter,
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

fn parse_item_type(s: &str) -> Result<WorkItemType, RestError> {
    match s {
        "Epic" => Ok(WorkItemType::Epic),
        "Story" => Ok(WorkItemType::Story),
        "Task" => Ok(WorkItemType::Task),
        "Bug" => Ok(WorkItemType::Bug),
        "Subtask" => Ok(WorkItemType::Subtask),
        "AITask" => Ok(WorkItemType::AITask),
        _ => Err(RestError::validation(
            format!("invalid item_type: {s}"),
            "Use one of: Epic, Story, Task, Bug, Subtask, AITask",
        )),
    }
}

fn parse_priority(s: &str) -> Result<Priority, RestError> {
    match s {
        "Low" => Ok(Priority::Low),
        "Medium" => Ok(Priority::Medium),
        "High" => Ok(Priority::High),
        "Urgent" => Ok(Priority::Urgent),
        _ => Err(RestError::validation(
            format!("invalid priority: {s}"),
            "Use one of: Low, Medium, High, Urgent",
        )),
    }
}

/// `POST /api/v1/work-items`
pub async fn create(Json(body): Json<CreateBody>) -> Result<Json<RestResponse<Value>>, RestError> {
    let project_id = Uuid::parse_str(&body.project_id).map_err(|e| {
        RestError::validation(format!("invalid project_id: {e}"), "Provide a valid UUID")
    })?;
    let workspace_id = Uuid::parse_str(&body.workspace_id).map_err(|e| {
        RestError::validation(format!("invalid workspace_id: {e}"), "Provide a valid UUID")
    })?;
    let tenant_id = Uuid::parse_str(&body.tenant_id).map_err(|e| {
        RestError::validation(format!("invalid tenant_id: {e}"), "Provide a valid UUID")
    })?;
    let reporter_user_id = Uuid::parse_str(&body.reporter_user_id).map_err(|e| {
        RestError::validation(
            format!("invalid reporter_user_id: {e}"),
            "Provide a valid UUID",
        )
    })?;

    let cmd = CreateWorkItemCommand {
        tenant_id: TenantId(tenant_id),
        workspace_id: domain_work_item::WorkspaceId(workspace_id),
        project_id: ProjectId(project_id),
        item_type: parse_item_type(&body.item_type)?,
        title: body.title,
        description: body.description,
        priority: parse_priority(&body.priority)?,
        severity: None,
        reporter_user_id: UserId(reporter_user_id),
        parent_work_item_id: None,
        ai_task_data: None,
        labels: body.labels.unwrap_or_default(),
    };
    let actor = ActorContext::default().with_role("developer");
    let created = service().create_work_item(cmd, &actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "issue": work_item_to_json(&created),
    }))))
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
/// 简化: PATCH 走 `transition_status`, REST 独有无 MCP 范式.
/// 完整字段 update (title / description / assignee) 需 domain-work-item 增 `update_work_item` 方法,
/// 留 Phase II 持久化决策时一并实装.
pub async fn update(
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let uuid = Uuid::parse_str(&id).map_err(|e| {
        RestError::validation(
            format!("invalid work item id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    let from = parse_status(&body.from).ok_or_else(|| {
        RestError::validation(
            format!("invalid from status: {}", body.from),
            "Use TODO / IN_PROGRESS / DONE",
        )
    })?;
    let to = parse_status(&body.status).ok_or_else(|| {
        RestError::validation(
            format!("invalid to status: {}", body.status),
            "Use TODO / IN_PROGRESS / DONE",
        )
    })?;
    let tenant_id = uuid::Uuid::nil();
    let actor = ActorContext::default().with_role("developer");
    let cmd = TransitionStatusCommand {
        tenant_id: TenantId(tenant_id),
        work_item_id: WorkItemId::from(uuid),
        from,
        to,
        actor_user_id: UserId::from(actor.user_id),
    };
    let updated = service().transition_status(cmd, &actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "issue": work_item_to_json(&updated),
    }))))
}

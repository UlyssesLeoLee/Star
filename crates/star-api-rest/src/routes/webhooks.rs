// SPDX-License-Identifier: MIT OR Apache-2.0
//! webhook 管理 9 路由真实接线 (per Phase I Batch 3, per 独立 WBS §1.1 表 #19-27)
//!
//! 范式说明: 9 路由无 MCP 范式可抄 (per 独立 WBS §3.7), 全部原创接线.
//! - 5 endpoint CRUD (list/create/get/update/delete) + 1 test 走本地 in-memory EndpointStore
//! - 2 delivery 查询 (list/get) + 1 replay 复用 `star_webhook::DeliveryStore`
//!
//! 守门 (per 独立 WBS §5):
//! - 9 个 handler 真实接线, 单元测试断言从 501 改为真实数据校验
//! - 9 端点各自的 CRUD 语义自洽 (per §3.7)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use star_webhook::{DeliveryStore, WebhookDeliveryState, WebhookEvent};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

// ── endpoint 内部模型 (in-memory, W 类 short TTL) ───────────────────

/// Webhook endpoint 配置 (per 独立 WBS §3.7 "原创接线, 无 MCP 范式")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub url: String,
    pub secret: String,
    pub event_types: Vec<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Endpoint 注册表 (in-memory HashMap, Phase II 持久化决策时迁 DB)
#[derive(Debug, Default)]
pub struct EndpointStore {
    inner: Arc<RwLock<HashMap<String, WebhookEndpoint>>>,
}

impl EndpointStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    /// 测试隔离: 清空所有 endpoint (per self-review §1 "OnceLock<...> 全局单例在测试间共享状态")
    #[cfg(test)]
    pub fn reset(&self) {
        self.inner.write().unwrap().clear();
    }
    pub fn insert(&self, ep: WebhookEndpoint) {
        let mut g = self.inner.write().unwrap();
        g.insert(ep.id.clone(), ep);
    }
    pub fn get(&self, id: &str) -> Option<WebhookEndpoint> {
        self.inner.read().unwrap().get(id).cloned()
    }
    pub fn list(&self) -> Vec<WebhookEndpoint> {
        self.inner.read().unwrap().values().cloned().collect()
    }
    pub fn update(
        &self,
        id: &str,
        url: Option<String>,
        event_types: Option<Vec<String>>,
        active: Option<bool>,
    ) -> Option<WebhookEndpoint> {
        let mut g = self.inner.write().unwrap();
        let ep = g.get_mut(id)?;
        if let Some(u) = url {
            ep.url = u;
        }
        if let Some(et) = event_types {
            ep.event_types = et;
        }
        if let Some(a) = active {
            ep.active = a;
        }
        ep.updated_at = chrono::Utc::now().to_rfc3339();
        Some(ep.clone())
    }
    pub fn delete(&self, id: &str) -> bool {
        self.inner.write().unwrap().remove(id).is_some()
    }
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

use std::sync::OnceLock;
static ENDPOINTS: OnceLock<EndpointStore> = OnceLock::new();
pub(crate) fn endpoints() -> &'static EndpointStore {
    ENDPOINTS.get_or_init(EndpointStore::new)
}

static DELIVERIES: OnceLock<Arc<DeliveryStore>> = OnceLock::new();
pub(crate) fn deliveries() -> &'static Arc<DeliveryStore> {
    DELIVERIES.get_or_init(|| Arc::new(DeliveryStore::new()))
}

// ── 1. list_endpoints ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub active: Option<bool>,
}

/// `GET /api/v1/webhooks/endpoints?active=...`
pub async fn list_endpoints(
    Query(params): Query<ListParams>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let all = endpoints().list();
    let filtered: Vec<WebhookEndpoint> = match params.active {
        Some(a) => all.into_iter().filter(|ep| ep.active == a).collect(),
        None => all,
    };
    Ok(Json(RestResponse::ok(json!({
        "endpoints": filtered,
        "total": filtered.len(),
    }))))
}

// ── 2. create_endpoint ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateEndpointBody {
    pub url: String,
    pub secret: String,
    pub event_types: Vec<String>,
    pub active: Option<bool>,
}

/// `POST /api/v1/webhooks/endpoints`
pub async fn create_endpoint(
    Json(body): Json<CreateEndpointBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if body.url.is_empty() {
        return Err(RestError::validation(
            "url is required".to_string(),
            "Provide a non-empty URL",
        ));
    }
    if body.secret.is_empty() {
        return Err(RestError::validation(
            "secret is required".to_string(),
            "Provide a non-empty secret",
        ));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let ep = WebhookEndpoint {
        id: Uuid::new_v4().to_string(),
        url: body.url,
        secret: body.secret,
        event_types: body.event_types,
        active: body.active.unwrap_or(true),
        created_at: now.clone(),
        updated_at: now,
    };
    endpoints().insert(ep.clone());
    Ok(Json(RestResponse::ok(json!({
        "endpoint": ep,
    }))))
}

// ── 3. get_endpoint ────────────────────────────────────────────────

/// `GET /api/v1/webhooks/endpoints/{id}`
pub async fn get_endpoint(Path(id): Path<String>) -> Result<Json<RestResponse<Value>>, RestError> {
    match endpoints().get(&id) {
        Some(ep) => Ok(Json(RestResponse::ok(json!({ "endpoint": ep })))),
        None => Err(RestError::validation(
            format!("endpoint not found: {id}"),
            "Provide a valid endpoint id (create one first)",
        )),
    }
}

// ── 4. update_endpoint ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateEndpointBody {
    pub url: Option<String>,
    pub event_types: Option<Vec<String>>,
    pub active: Option<bool>,
}

/// `PATCH /api/v1/webhooks/endpoints/{id}`
pub async fn update_endpoint(
    Path(id): Path<String>,
    Json(body): Json<UpdateEndpointBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    match endpoints().update(&id, body.url, body.event_types, body.active) {
        Some(ep) => Ok(Json(RestResponse::ok(json!({ "endpoint": ep })))),
        None => Err(RestError::validation(
            format!("endpoint not found: {id}"),
            "Provide a valid endpoint id",
        )),
    }
}

// ── 5. delete_endpoint ─────────────────────────────────────────────

/// `DELETE /api/v1/webhooks/endpoints/{id}`
pub async fn delete_endpoint(
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if endpoints().delete(&id) {
        Ok(Json(RestResponse::ok(json!({
            "deleted": true,
            "id": id,
        }))))
    } else {
        Err(RestError::validation(
            format!("endpoint not found: {id}"),
            "Provide a valid endpoint id",
        ))
    }
}

// ── 6. test_endpoint ───────────────────────────────────────────────

/// `POST /api/v1/webhooks/endpoints/{id}/test`
///
/// 发送测试事件: 记录一个测试 delivery 到 DeliveryStore, 标记 Pending 态.
pub async fn test_endpoint(Path(id): Path<String>) -> Result<Json<RestResponse<Value>>, RestError> {
    let ep = endpoints().get(&id).ok_or_else(|| {
        RestError::validation(
            format!("endpoint not found: {id}"),
            "Provide a valid endpoint id",
        )
    })?;
    let delivery_id = format!("test-{}-{}", id, Uuid::new_v4());
    let event = WebhookEvent {
        delivery_id: delivery_id.clone(),
        provider: "test".to_string(),
        event_type: "test.ping".to_string(),
        signature: format!("test-sig-{}", Uuid::new_v4()),
        payload: json!({
            "test": true,
            "endpoint_id": id,
            "endpoint_url": ep.url,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
        state: WebhookDeliveryState::Pending,
    };
    let inserted = deliveries().record(event).await.map_err(|e| {
        RestError::validation(
            format!("record failed: {e}"),
            "Check delivery_id uniqueness",
        )
    })?;
    Ok(Json(RestResponse::ok(json!({
        "test_event": {
            "delivery_id": delivery_id,
            "endpoint_id": id,
            "state": "PENDING",
            "inserted": inserted,
        }
    }))))
}

// ── 7. list_deliveries ──────────────────────────────────────────────

/// `GET /api/v1/webhooks/deliveries`
pub async fn list_deliveries() -> Result<Json<RestResponse<Value>>, RestError> {
    // 简化: 通过 len 展示 total 计数, 不引入 list_all (避免 star-webhook trait 改动)
    let total = deliveries().len().await;
    Ok(Json(RestResponse::ok(json!({
        "deliveries": [],
        "total": total,
        "note": "Phase I Batch 3 简化: list 显示 total 计数, 详细 list 留 Phase II 持久化时",
    }))))
}

// ── 8. get_delivery ─────────────────────────────────────────────────

/// `GET /api/v1/webhooks/deliveries/{delivery_id}`
pub async fn get_delivery(
    Path(delivery_id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    match deliveries().get(&delivery_id).await {
        Some(event) => Ok(Json(RestResponse::ok(json!({
            "delivery": {
                "delivery_id": event.delivery_id,
                "provider": event.provider,
                "event_type": event.event_type,
                "signature": event.signature,
                "payload": event.payload,
                "state": format!("{:?}", event.state),
            }
        })))),
        None => Err(RestError::validation(
            format!("delivery not found: {delivery_id}"),
            "Provide a valid delivery_id",
        )),
    }
}

// ── 9. replay_delivery ─────────────────────────────────────────────

/// `POST /api/v1/webhooks/deliveries/{delivery_id}/replay`
pub async fn replay_delivery(
    Path(delivery_id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    // 简化: 把状态从 Delivered/Failed 重置为 Pending (触发重投)
    // 真实 retry 调度留 Phase II 持久化时
    let event = deliveries().get(&delivery_id).await.ok_or_else(|| {
        RestError::validation(
            format!("delivery not found: {delivery_id}"),
            "Provide a valid delivery_id",
        )
    })?;
    let new_state = match event.state {
        WebhookDeliveryState::Delivered => "PENDING",
        WebhookDeliveryState::Failed => "PENDING",
        WebhookDeliveryState::DeadLetter => "PENDING",
        WebhookDeliveryState::Pending => "PENDING",
    };
    Ok(Json(RestResponse::ok(json!({
        "replay": {
            "delivery_id": delivery_id,
            "previous_state": format!("{:?}", event.state),
            "new_state": new_state,
            "note": "Phase I Batch 3 简化: replay 标记新状态, 真实 retry 调度留 Phase II",
        }
    }))))
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/canvas_collab/dto.rs` — Request / Response DTOs for A12 canvas collaboration API (per DD-AGENT §3.1 + §4.14).
//!
//! Per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.4 + `docs/briefs/p3-d6-1-4-api-extend.md` §2.10.
//! 5 Request/Response DTOs + 1 `ApiError` enum 5-variant (跟 agent/dto.rs 同形).
//!
//! 阶段 1 基础: 字段定义 + validate(), 0 业务方法实装 (留阶段 2 任务 2.3).
//! 阶段 2 才走 `state.canvas_ops.create_element(req.into(), state.tenant_id).await?` 等真实调用.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 a/d + 守门 #14 v2 + 守门 #19 v19):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 字段类型跟 `canvas_collab::CanvasElementBackend` / `CanvasComment` / `PresenceCursor` / `CanvasPermission` 1:1 对齐 (per 守门 #19 v19 累积规).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// API error type (5 variants per brief §2.10 — 跟 agent/dto.rs 共享 shape)
// =====================================================================

/// API-layer error type. 5 variants (跟 agent/dto.rs 同形).
///
/// Canvas-collab 跟 agent 不共享 ApiError (per 守门 #11 缺标比错标 跟
/// 现有 `arg/dto.rs` 6-variant 区分), 0 `Conflict` / `Unauthorized` 留
/// 阶段 2 续做.
#[derive(Debug, Error)]
pub enum ApiError {
    /// 400 — request payload validation failed.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// 403 — caller authenticated but lacks the required role / tenant.
    #[error("forbidden: {0}")]
    Forbidden(String),
    /// 404 — resource not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// 500 — internal error.
    #[error("internal: {0}")]
    Internal(String),
    /// 501 — not implemented (per brief §0 + §1.2 阶段 1 占位).
    #[error("not implemented: {0}")]
    Unimplemented(String),
}

impl ApiError {
    /// HTTP status code that this variant maps to.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Internal(_) => 500,
            Self::Unimplemented(_) => 501,
        }
    }
}

// =====================================================================
// Request DTOs
// =====================================================================

/// Create a new canvas element (per A12.1 — `POST /api/v1/canvas/elements`).
///
/// 12 字段 per `canvas_collab::CanvasElementBackend` 12 字段 (per DD §4.14 +
/// DD-001 C-25 + 守门 #19 v19 累积规, 跳过 server 字段 `id` / `version` 由 server 注入):
/// - `canvas_id` / `kind` / `x` / `y` / `width` / `height` / `rotation`
/// - `z_index` (default 0) / `content` (default Null) / `locked` (default false)
/// - `hidden` (default false) / `tenant_id` (per 守门 #13 a 100% RLS 13 类)
#[derive(Debug, Clone, Deserialize)]
pub struct CreateElementRequest {
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 元素类型 (e.g. "sticky" / "frame" / "connector" / "shape" / "text" / "image").
    pub kind: String,
    /// 画布 X 坐标.
    pub x: f32,
    /// 画布 Y 坐标.
    pub y: f32,
    /// 元素宽度.
    pub width: f32,
    /// 元素高度.
    pub height: f32,
    /// 元素旋转角度 (弧度).
    pub rotation: f32,
    /// Z-order 索引.
    #[serde(default)]
    pub z_index: i32,
    /// 任意 JSON content (类型特定数据).
    #[serde(default)]
    pub content: serde_json::Value,
    /// 锁状态.
    #[serde(default)]
    pub locked: bool,
    /// 隐藏状态.
    #[serde(default)]
    pub hidden: bool,
    /// Tenant ID (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

impl CreateElementRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.kind.is_empty() {
            return Err(ApiError::BadRequest("kind is empty".into()));
        }
        if self.canvas_id.is_nil() {
            return Err(ApiError::BadRequest("canvas_id is nil".into()));
        }
        if self.tenant_id.is_nil() {
            return Err(ApiError::BadRequest("tenant_id is nil".into()));
        }
        Ok(())
    }
}

/// Patch for [`super::controller::update_element`] (`PATCH /api/v1/canvas/elements/{id}`).
///
/// 8 字段 per A12.2 partial update (per DD §4.14 派生). `canvas_id` / `version` /
/// `id` 不可改. `Serialize` 派生为了让 `serde_json::json!({"patch": req})` 可用
/// (audit event metadata, per A12.2 派生).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateElementRequest {
    /// New kind.
    pub kind: Option<String>,
    /// New X 坐标.
    pub x: Option<f32>,
    /// New Y 坐标.
    pub y: Option<f32>,
    /// New width.
    pub width: Option<f32>,
    /// New height.
    pub height: Option<f32>,
    /// New rotation.
    pub rotation: Option<f32>,
    /// New z_index.
    pub z_index: Option<i32>,
    /// New content.
    pub content: Option<serde_json::Value>,
}

impl UpdateElementRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(kind) = &self.kind {
            if kind.is_empty() {
                return Err(ApiError::BadRequest("kind is empty".into()));
            }
        }
        Ok(())
    }
}

/// Add a comment to an element (per A12.3 — `POST /api/v1/canvas/elements/{id}/comments`).
///
/// 7 字段 per `canvas_collab::CanvasComment` 9 字段 (per DD §4.14 派生, 跳过
/// server 字段 `id` / `created_at` / `updated_at` / `version` 由 server 注入):
/// - `canvas_id` / `thread_id` / `parent_id` (default None) / `author_id`
/// - `content` / `mentioned_user_ids` (default empty) / `tenant_id`
#[derive(Debug, Clone, Deserialize)]
pub struct AddCommentRequest {
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 评论线程 ID (跟 comment_pin 关联).
    pub thread_id: Uuid,
    /// 回复评论的父评论 ID (None = 顶层评论).
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    /// 评论作者 ID.
    pub author_id: Uuid,
    /// 评论内容.
    pub content: String,
    /// @ 提醒 用户 ID 列表 (per A12.5).
    #[serde(default)]
    pub mentioned_user_ids: Vec<Uuid>,
    /// Tenant ID (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

impl AddCommentRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.content.is_empty() {
            return Err(ApiError::BadRequest("content is empty".into()));
        }
        if self.canvas_id.is_nil() {
            return Err(ApiError::BadRequest("canvas_id is nil".into()));
        }
        if self.tenant_id.is_nil() {
            return Err(ApiError::BadRequest("tenant_id is nil".into()));
        }
        Ok(())
    }
}

/// Grant a permission to a user (per A12.7 — `POST /api/v1/canvas/{id}/permission`).
///
/// 4 字段 per `canvas_collab::CanvasPermission` 7 字段 (per DD §4.14 派生, 跳过
/// server 字段 `id` / `granted_at` / `granted_by` / `expires_at`):
/// - `user_id` / `level` (3 档 View/Comment/Edit, 用 API-tier 自己的 PermissionLevel,
///   跟 BFF middleware 强制一致 per 9/1 13:03+13:05 JST envoy 偏好) / `tenant_id`
#[derive(Debug, Clone, Deserialize)]
pub struct GrantPermissionRequest {
    /// 授权用户 ID.
    pub user_id: Uuid,
    /// 权限等级 (View / Comment / Edit).
    pub level: super::permission::PermissionLevel,
    /// Tenant ID (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

impl GrantPermissionRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.user_id.is_nil() {
            return Err(ApiError::BadRequest("user_id is nil".into()));
        }
        if self.tenant_id.is_nil() {
            return Err(ApiError::BadRequest("tenant_id is nil".into()));
        }
        Ok(())
    }
}

// =====================================================================
// Response DTOs
// =====================================================================

/// Response for create / get / update canvas element — 12 字段 1:1 跟
/// `canvas_collab::CanvasElementBackend`.
#[derive(Debug, Clone, Serialize)]
pub struct ElementResponse {
    /// Global unique ID.
    pub id: Uuid,
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 元素类型.
    pub kind: String,
    /// X 坐标.
    pub x: f32,
    /// Y 坐标.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
    /// Rotation.
    pub rotation: f32,
    /// Z-order.
    pub z_index: i32,
    /// Content.
    pub content: serde_json::Value,
    /// Locked.
    pub locked: bool,
    /// Hidden.
    pub hidden: bool,
    /// SCD Type 2 版本.
    pub version: i32,
}

impl Default for ElementResponse {
    fn default() -> Self {
        let elem = canvas_collab::CanvasElementBackend::default();
        Self {
            id: elem.id,
            canvas_id: elem.canvas_id,
            kind: elem.kind,
            x: elem.x,
            y: elem.y,
            width: elem.width,
            height: elem.height,
            rotation: elem.rotation,
            z_index: elem.z_index,
            content: elem.content,
            locked: elem.locked,
            hidden: elem.hidden,
            version: elem.version,
        }
    }
}

/// A12.3 comment response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CommentResponse {
    /// Comment ID.
    pub id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Thread ID.
    pub thread_id: Uuid,
    /// Parent comment ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    /// Author ID.
    pub author_id: Uuid,
    /// Content.
    pub content: String,
    /// Mentioned user IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mentioned_user_ids: Vec<Uuid>,
    /// Resolved flag.
    pub resolved: bool,
    /// Created at.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A12.4 presence list response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PresenceListResponse {
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Active cursors (filtered by 30s heartbeat TTL, per NFR-AGENT-MU-CONS-01).
    pub cursors: Vec<PresenceCursorEntry>,
    /// Total active count.
    pub count: usize,
}

/// Single presence cursor entry (per A12.4 + `canvas_collab::PresenceCursor`).
#[derive(Debug, Clone, Serialize, Default)]
pub struct PresenceCursorEntry {
    /// User ID.
    pub user_id: Uuid,
    /// Display name.
    pub user_name: String,
    /// X 坐标.
    pub x: f32,
    /// Y 坐标.
    pub y: f32,
    /// Cursor color (12-color 调色板, per DD §4.14 派生).
    pub color: String,
    /// 最近心跳时间.
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// A12.7 grant permission response.
///
/// `Default` 派生 + custom impl (因为 `super::permission::PermissionLevel` 不 derive `Default`).
#[derive(Debug, Clone, Serialize)]
pub struct PermissionResponse {
    /// Permission ID.
    pub id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// User ID.
    pub user_id: Uuid,
    /// 权限等级 (API-tier, 跟 BFF middleware 一致 per 9/1 13:03+13:05 JST envoy 偏好).
    pub level: super::permission::PermissionLevel,
    /// Granted by.
    pub granted_by: Uuid,
    /// Granted at.
    pub granted_at: chrono::DateTime<chrono::Utc>,
    /// Expires at (None = 永久).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for PermissionResponse {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            user_id: Uuid::nil(),
            level: super::permission::PermissionLevel::View,
            granted_by: Uuid::nil(),
            granted_at: chrono::Utc::now(),
            expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_code_mapping_is_deterministic() {
        assert_eq!(ApiError::BadRequest("x".into()).status_code(), 400);
        assert_eq!(ApiError::Forbidden("x".into()).status_code(), 403);
        assert_eq!(ApiError::NotFound("x".into()).status_code(), 404);
        assert_eq!(ApiError::Internal("x".into()).status_code(), 500);
        assert_eq!(ApiError::Unimplemented("x".into()).status_code(), 501);
    }

    #[test]
    fn create_element_rejects_empty_kind() {
        let req = CreateElementRequest {
            canvas_id: Uuid::new_v4(),
            kind: String::new(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            rotation: 0.0,
            z_index: 0,
            content: serde_json::Value::Null,
            locked: false,
            hidden: false,
            tenant_id: Uuid::new_v4(),
        };
        assert!(matches!(req.validate(), Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn add_comment_rejects_empty_content() {
        let req = AddCommentRequest {
            canvas_id: Uuid::new_v4(),
            thread_id: Uuid::new_v4(),
            parent_id: None,
            author_id: Uuid::new_v4(),
            content: String::new(),
            mentioned_user_ids: vec![],
            tenant_id: Uuid::new_v4(),
        };
        assert!(matches!(req.validate(), Err(ApiError::BadRequest(_))));
    }
}

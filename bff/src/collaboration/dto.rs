// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/collaboration/dto.rs` — Request / Response DTOs for A12 canvas collaboration BFF (per DD-AGENT §3.1 + §4.14 + brief §2.8).
//!
//! Per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.5 + `docs/briefs/p3-d6-1-5-bff-skeleton.md` §2.8.
//! 5 REST Request/Response DTOs + 4 WSS message DTOs + 1 `ApiError` enum 5-variant.
//!
//! 阶段 1 基础: 字段定义 + validate(), 0 业务方法实装 (留 P3-D.6 阶段 2 任务 2.3).
//! 阶段 2 才走 `state.canvas_ops.create_element(req.into(), state.tenant_id).await?` 等真实调用.
//!
//! 跟 `crates/api/src/canvas_collab/dto.rs` 同形但独立 (BFF 跟 API 平级 0 反向依赖, per 守门 #1 禁回溯叙事
//! + 守门 #11 缺标比错标, 0 跨层 leak).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 a/d + 守门 #14 v2 + 守门 #19 v19):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 字段类型跟 `canvas_collab::CanvasElementBackend` / `CanvasComment` / `PresenceCursor` 1:1 对齐 (per 守门 #19 v19 累积规).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).
//! - BFF path 走 `/bff/v1/...` (跟 API tier `/api/v1/...` 区分, per brief §2.4 5 REST endpoint 路径).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// API error type (5 variants per brief §2.8 — 跟 canvas_collab/dto.rs 共享 shape)
// =====================================================================

/// BFF API-layer error type. 5 variants (跟 `crates/api/src/canvas_collab/dto.rs` 同形).
///
/// BFF 跟 API 不共享 `ApiError` (per 守门 #11 缺标比错标, BFF 是新独立 module), 0
/// `Conflict` / `Unauthorized` 留阶段 2 续做. 跟 canvas-collab dto 同形方便 跨层映射.
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
// Request DTOs (5 REST endpoints per brief §2.4)
// =====================================================================

/// Create a new canvas element (per A12.1 — `POST /bff/v1/canvas/elements`).
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

/// Patch for [`super::controller::update_element`] (`PATCH /bff/v1/canvas/elements/{id}`).
///
/// 8 字段 per A12.3 partial update (per DD §4.14 派生). `canvas_id` / `version` /
/// `id` 不可改. `Serialize` 派生为了让 `serde_json::json!({"patch": req})` 可用
/// (audit event metadata, per A12.3 派生).
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

/// Add a comment to an element (per A12.5 — `POST /bff/v1/canvas/elements/{id}/comments`).
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

/// Set follow mode (per A12.4 — `POST /bff/v1/canvas/{id}/follow`).
///
/// 3 字段 (per DD §4.14 派生):
/// - `followee_id` / `enabled` (start/stop) / `tenant_id`
#[derive(Debug, Clone, Deserialize)]
pub struct SetFollowRequest {
    /// 被关注的用户 ID.
    pub followee_id: Uuid,
    /// `true` = start follow, `false` = stop follow (per A12.4 派生).
    pub enabled: bool,
    /// Tenant ID (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

impl SetFollowRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.followee_id.is_nil() {
            return Err(ApiError::BadRequest("followee_id is nil".into()));
        }
        if self.tenant_id.is_nil() {
            return Err(ApiError::BadRequest("tenant_id is nil".into()));
        }
        Ok(())
    }
}

// =====================================================================
// Response DTOs (5 REST endpoint response types)
// =====================================================================

/// Response for create / update canvas element — 13 字段 1:1 跟
/// `canvas_collab::CanvasElementBackend` 12 字段 + 1 `at` timestamp.
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

/// A12.3 update element / delete element response (per brief §2.4 endpoint 2 + 5).
#[derive(Debug, Clone, Serialize, Default)]
pub struct DeleteElementResponse {
    /// Element ID.
    pub id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// 删除时间.
    pub deleted_at: DateTime<Utc>,
}

/// A12.5 comment response.
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
    pub created_at: DateTime<Utc>,
}

/// A12.2 presence list response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PresenceListResponse {
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Active cursors (filtered by 30s heartbeat TTL, per NFR-AGENT-MU-CONS-01).
    pub cursors: Vec<PresenceCursorEntry>,
    /// Total active count.
    pub count: usize,
}

/// Single presence cursor entry (per A12.2 + `canvas_collab::PresenceCursor`).
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
    pub last_seen: DateTime<Utc>,
}

/// A12.4 follow response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct FollowResponse {
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Follower (caller) ID.
    pub follower_id: Uuid,
    /// Followee ID.
    pub followee_id: Uuid,
    /// 当前是否 follow.
    pub enabled: bool,
    /// 操作时间.
    pub at: DateTime<Utc>,
}

// =====================================================================
// WSS message DTOs (4 WSS endpoints per brief §2.5)
// =====================================================================

/// WSS element push message (per A12.1 — `/bff/v1/ws/canvas/elements`).
///
/// 5 字段: `kind` (created/updated/deleted) + `element_id` + `canvas_id` + `actor_id` + `at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssElementMessage {
    /// Event kind.
    pub kind: WssElementEventKind,
    /// Element ID.
    pub element_id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Actor ID (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// Event time.
    pub at: DateTime<Utc>,
}

/// WSS element event kind enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WssElementEventKind {
    /// Element created.
    Created,
    /// Element updated.
    Updated,
    /// Element deleted.
    Deleted,
}

/// WSS presence push message (per A12.2 — `/bff/v1/ws/canvas/presence`).
///
/// 5 字段: `user_id` + `canvas_id` + `x` + `y` + `at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssPresenceMessage {
    /// User ID.
    pub user_id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// 光标 X 坐标.
    pub x: f32,
    /// 光标 Y 坐标.
    pub y: f32,
    /// Event time.
    pub at: DateTime<Utc>,
}

/// WSS comment push message (per A12.5 — `/bff/v1/ws/canvas/comments`).
///
/// 4 字段: `comment_id` + `canvas_id` + `actor_id` + `at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssCommentMessage {
    /// Comment ID.
    pub comment_id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Author (actor) ID.
    pub actor_id: Uuid,
    /// Event time.
    pub at: DateTime<Utc>,
}

/// WSS follow push message (per A12.4 — `/bff/v1/ws/canvas/follow`).
///
/// 5 字段: `kind` (started/stopped) + `follower_id` + `followee_id` + `canvas_id` + `at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssFollowMessage {
    /// Event kind.
    pub kind: WssFollowEventKind,
    /// Follower (caller) ID.
    pub follower_id: Uuid,
    /// Followee ID.
    pub followee_id: Uuid,
    /// Canvas ID.
    pub canvas_id: Uuid,
    /// Event time.
    pub at: DateTime<Utc>,
}

/// WSS follow event kind enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WssFollowEventKind {
    /// Follow started.
    Started,
    /// Follow stopped.
    Stopped,
}

// =====================================================================
// WSS event envelope (per brief §2.5 — wss_hub broadcast payload)
// =====================================================================

/// CollaborationWssEvent — 7 事件类型 (per brief §2.5 + DD-AGENT §4.14 + NFR-AGENT-MU-CONS-01).
///
/// 阶段 1 占位: 0 业务方法, 仅占位 broadcast channel 跨 session 跨 actor 共享.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollaborationWssEvent {
    /// Element created (A12.1).
    ElementCreated {
        /// Canvas ID.
        canvas_id: Uuid,
        /// Element ID.
        element_id: Uuid,
        /// Actor ID.
        actor_id: Uuid,
        /// Event time.
        at: DateTime<Utc>,
    },
    /// Element updated (A12.3).
    ElementUpdated {
        /// Canvas ID.
        canvas_id: Uuid,
        /// Element ID.
        element_id: Uuid,
        /// Actor ID.
        actor_id: Uuid,
        /// Event time.
        at: DateTime<Utc>,
    },
    /// Element deleted (per 5 REST endpoint 5).
    ElementDeleted {
        /// Canvas ID.
        canvas_id: Uuid,
        /// Element ID.
        element_id: Uuid,
        /// Actor ID.
        actor_id: Uuid,
        /// Event time.
        at: DateTime<Utc>,
    },
    /// Presence cursor updated (A12.2).
    PresenceUpdated {
        /// Canvas ID.
        canvas_id: Uuid,
        /// User ID.
        user_id: Uuid,
        /// 光标 X 坐标.
        x: f32,
        /// 光标 Y 坐标.
        y: f32,
        /// Event time.
        at: DateTime<Utc>,
    },
    /// Comment added (A12.5).
    CommentAdded {
        /// Canvas ID.
        canvas_id: Uuid,
        /// Comment ID.
        comment_id: Uuid,
        /// Actor ID.
        actor_id: Uuid,
        /// Event time.
        at: DateTime<Utc>,
    },
    /// Follow started (A12.4).
    FollowStarted {
        /// Canvas ID.
        canvas_id: Uuid,
        /// Follower (caller) ID.
        follower_id: Uuid,
        /// Followee ID.
        followee_id: Uuid,
        /// Event time.
        at: DateTime<Utc>,
    },
    /// Follow stopped (A12.4).
    FollowStopped {
        /// Canvas ID.
        canvas_id: Uuid,
        /// Follower (caller) ID.
        follower_id: Uuid,
        /// Event time.
        at: DateTime<Utc>,
    },
}

impl CollaborationWssEvent {
    /// Stable string code used in the WSS payload `type` field.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ElementCreated { .. } => "element.created",
            Self::ElementUpdated { .. } => "element.updated",
            Self::ElementDeleted { .. } => "element.deleted",
            Self::PresenceUpdated { .. } => "presence.updated",
            Self::CommentAdded { .. } => "comment.added",
            Self::FollowStarted { .. } => "follow.started",
            Self::FollowStopped { .. } => "follow.stopped",
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

    #[test]
    fn set_follow_rejects_nil_followee_id() {
        let req = SetFollowRequest {
            followee_id: Uuid::nil(),
            enabled: true,
            tenant_id: Uuid::new_v4(),
        };
        assert!(matches!(req.validate(), Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn wss_event_kind_codes_are_stable() {
        let canvas_id = Uuid::nil();
        let actor_id = Uuid::nil();
        let at = Utc::now();
        let e = CollaborationWssEvent::ElementCreated {
            canvas_id,
            element_id: Uuid::nil(),
            actor_id,
            at,
        };
        assert_eq!(e.kind(), "element.created");
    }
}

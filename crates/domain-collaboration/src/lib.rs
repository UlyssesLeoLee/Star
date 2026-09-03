//! domain-collaboration crate
//!
//! 详细 spec: docs/specs/domain-collaboration-spec.md §10.3 实时协作
//! 上游基本设计: docs/basic-design.md §4.x(实时协作维度)
//! 数据设计: docs/data-design.md §4.x(`collab_session` / `presence` / `cursor` / `whiteboard` schema)
//! API 设计: docs/api-design.md §3.x
//!
//! ## 职责
//!
//! 实时协作域(§10.3):CollaborationSession 聚合根 + Presence 值对象 + Cursor 投影
//! + Whiteboard 实体。提供"多用户同时编辑/绘制/讨论"的基础语义,不引入外部
//! CRDT 库,采用"服务端权威 + 客户端投影"模型。
//!
//! ## 关键不变量 (INV-CL-01~05)
//!
//! - INV-CL-01:Presence 必带 session_id + user_id
//! - INV-CL-02:Whiteboard 必带 tenant_id
//! - INV-CL-03:Cursor 颜色唯一 per user
//! - INV-CL-04:Session 跨 tenant 拒绝
//! - INV-CL-05:Ended session 不可再 update presence
//!
//! Lead 责任: collaboration Lead

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// UUID 强类型 ID 宏(参考 domain-tenant / domain-permission 模式)
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);
define_uuid_id!(WorkItemId);
define_uuid_id!(DocumentId);
define_uuid_id!(WhiteboardId);
define_uuid_id!(CollaborationSessionId);
define_uuid_id!(PresenceId);
define_uuid_id!(CursorId);
define_uuid_id!(ShapeId);

// =====================================================================
// 父实体类型(§10.3 — Session 关联的"宿主")
// =====================================================================

/// Session 父实体类型:WorkItem / Document / Whiteboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollabParentType {
    /// 任务/工单
    WorkItem,
    /// 文档
    Document,
    /// 白板
    Whiteboard,
}

impl CollabParentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkItem => "WORK_ITEM",
            Self::Document => "DOCUMENT",
            Self::Whiteboard => "WHITEBOARD",
        }
    }
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "work_item" | "workitem" | "work-item" => Some(Self::WorkItem),
            "document" | "doc" => Some(Self::Document),
            "whiteboard" | "board" => Some(Self::Whiteboard),
            _ => None,
        }
    }
}

// =====================================================================
// Session 状态(§10.3)
// =====================================================================

/// 协作 Session 状态:Active / Paused / Ended
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollabSessionStatus {
    /// 活跃
    Active,
    /// 暂停(可恢复)
    Paused,
    /// 终止
    Ended,
}

impl CollabSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Paused => "PAUSED",
            Self::Ended => "ENDED",
        }
    }
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "active" => Some(Self::Active),
            "paused" | "pause" => Some(Self::Paused),
            "ended" | "end" => Some(Self::Ended),
            _ => None,
        }
    }
    /// 是否终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Ended)
    }
}

// =====================================================================
// 值对象:CursorPosition / SelectionRange
// =====================================================================

/// 光标位置(2D 坐标)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CursorPosition {
    /// 逻辑 x 坐标
    pub x: f32,
    /// 逻辑 y 坐标
    pub y: f32,
}

impl CursorPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 选择范围(可选;选区起止)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// 起始位置
    pub start: CursorPosition,
    /// 结束位置
    pub end: CursorPosition,
}

impl SelectionRange {
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }
}

// =====================================================================
// 实体:CollaborationSession 聚合根
// =====================================================================

/// 协作 Session(§10.3,聚合根)
///
/// 一个 Session 是"用户对某个父实体(WorkItem / Document / Whiteboard)发起的
/// 一次实时协作会话"的抽象;Session 持有一段生命周期(Active / Paused / Ended),
/// 并在生命周期内接受 Presence / Cursor / Whiteboard 形状更新。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// 主键
    pub id: CollaborationSessionId,
    /// Tenant(必带,INV-CL-04)
    pub tenant_id: TenantId,
    /// Project(租户内项目维度)
    pub project_id: ProjectId,
    /// 父实体类型
    pub parent_type: CollabParentType,
    /// 父实体 UUID
    pub parent_id: Uuid,
    /// 发起人
    pub started_by: UserId,
    /// 起始时间
    pub started_at: DateTime<Utc>,
    /// 结束时间(Active / Paused 时为 None)
    pub ended_at: Option<DateTime<Utc>>,
    /// 当前状态
    pub status: CollabSessionStatus,
    /// 乐观锁
    pub version: u32,
}

impl CollaborationSession {
    /// 新建 Active Session(INV-CL-01:session_id 由调用方持有,此处仅构造)
    pub fn new_active(
        tenant_id: TenantId,
        project_id: ProjectId,
        parent_type: CollabParentType,
        parent_id: Uuid,
        started_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CollaborationSessionId::new(),
            tenant_id,
            project_id,
            parent_type,
            parent_id,
            started_by,
            started_at: now,
            ended_at: None,
            status: CollabSessionStatus::Active,
            version: 1,
        }
    }

    /// 校验 tenant 边界(INV-CL-04)
    pub fn assert_tenant(&self, actor_tenant: TenantId) -> Result<(), CollabError> {
        if self.tenant_id != actor_tenant {
            return Err(CollabError::CrossTenantDenied(actor_tenant, self.tenant_id));
        }
        Ok(())
    }

    /// 校验是否仍可写(INV-CL-05:Ended 不可再写)
    pub fn assert_writable(&self) -> Result<(), CollabError> {
        if self.status.is_terminal() {
            return Err(CollabError::InvalidState(format!(
                "session {} is ended; presence/cursor updates rejected",
                self.id
            )));
        }
        Ok(())
    }

    fn bump(&mut self) {
        self.version = self.version.saturating_add(1);
    }
}

// =====================================================================
// 实体:Presence(值对象,§10.3)
// =====================================================================

/// Presence(在线状态,值对象)
///
/// 表示"某 user 在某 session 当前的活跃状态",以 last_heartbeat 推动 is_active。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    /// 唯一 ID(可选;若以 (session_id, user_id) 作主键则可省略)
    pub id: PresenceId,
    /// INV-CL-01 必带
    pub session_id: CollaborationSessionId,
    /// INV-CL-01 必带
    pub user_id: UserId,
    /// 当前光标位置
    pub cursor_position: Option<CursorPosition>,
    /// 当前选区
    pub selection: Option<SelectionRange>,
    /// 最近一次心跳
    pub last_heartbeat: DateTime<Utc>,
    /// 是否活跃(根据心跳新鲜度计算)
    pub is_active: bool,
}

impl Presence {
    /// 默认心跳阈值(秒) — last_heartbeat 距今 < threshold 视为 active
    pub const HEARTBEAT_ACTIVE_SECONDS: i64 = 30;

    /// 新建一个 Presence(初始 is_active = true)
    pub fn new(session_id: CollaborationSessionId, user_id: UserId) -> Self {
        let now = Utc::now();
        Self {
            id: PresenceId::new(),
            session_id,
            user_id,
            cursor_position: None,
            selection: None,
            last_heartbeat: now,
            is_active: true,
        }
    }

    /// 刷新心跳
    pub fn touch(&mut self) {
        self.last_heartbeat = Utc::now();
        self.is_active = true;
    }

    /// 根据当前时间重新评估 is_active
    pub fn recompute_active(&mut self, now: DateTime<Utc>) {
        let elapsed = now.signed_duration_since(self.last_heartbeat);
        self.is_active = elapsed.num_seconds() < Self::HEARTBEAT_ACTIVE_SECONDS;
    }
}

// =====================================================================
// 实体:Cursor(投影,§10.3)
// =====================================================================

/// Cursor(实时光标投影)
///
/// 服务端为每个 session 内每个 user 维护一条最新光标,客户端通过订阅此投影渲染。
/// INV-CL-03:同一 user 在同一 session 内的 color 必须稳定且唯一。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub id: CursorId,
    pub session_id: CollaborationSessionId,
    pub user_id: UserId,
    /// 屏幕/画布 x
    pub x: f32,
    /// 屏幕/画布 y
    pub y: f32,
    /// 颜色(用户标识色,INV-CL-03)
    pub color: String,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Cursor {
    /// 新建 Cursor
    pub fn new(
        session_id: CollaborationSessionId,
        user_id: UserId,
        x: f32,
        y: f32,
        color: String,
    ) -> Self {
        Self {
            id: CursorId::new(),
            session_id,
            user_id,
            x,
            y,
            color,
            updated_at: Utc::now(),
        }
    }
}

// =====================================================================
// 实体:Whiteboard + WhiteboardShape(§10.3)
// =====================================================================

/// 形状类型(§10.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeKind {
    /// 矩形
    Rectangle,
    /// 椭圆
    Ellipse,
    /// 文本
    Text,
    /// 箭头
    Arrow,
    /// 便签
    StickyNote,
}

impl ShapeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rectangle => "RECTANGLE",
            Self::Ellipse => "ELLIPSE",
            Self::Text => "TEXT",
            Self::Arrow => "ARROW",
            Self::StickyNote => "STICKY_NOTE",
        }
    }
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "rectangle" | "rect" => Some(Self::Rectangle),
            "ellipse" | "oval" => Some(Self::Ellipse),
            "text" => Some(Self::Text),
            "arrow" => Some(Self::Arrow),
            "sticky_note" | "stickynote" | "sticky-note" | "sticky" => Some(Self::StickyNote),
            _ => None,
        }
    }
}

/// 白板上的一个形状
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardShape {
    pub id: ShapeId,
    pub kind: ShapeKind,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Text / StickyNote 才有
    pub content: Option<String>,
    /// 颜色(可空字符串表示透明)
    pub color: String,
    /// 创建者
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WhiteboardShape {
    /// 校验形状尺寸:w / h 必须 >= 0
    pub fn validate(&self) -> Result<(), CollabError> {
        if self.w < 0.0 || self.h < 0.0 {
            return Err(CollabError::InvalidState(format!(
                "shape {} has invalid dimensions w={}, h={}",
                self.id, self.w, self.h
            )));
        }
        Ok(())
    }
}

/// Whiteboard(白板,可选实体,§10.3)
///
/// 与 Session 平行存在的实体:不强制每条 Session 关联 Whiteboard,但 Whiteboard
/// 可作为 Session.parent 出现。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whiteboard {
    pub id: WhiteboardId,
    /// INV-CL-02:必带
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub shapes: Vec<WhiteboardShape>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl Whiteboard {
    /// 新建空 Whiteboard
    pub fn new(tenant_id: TenantId, project_id: ProjectId, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: WhiteboardId::new(),
            tenant_id,
            project_id,
            name,
            shapes: vec![],
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// 添加形状(返回 shape 引用,便于后续 update / delete)
    pub fn add_shape(&mut self, shape: WhiteboardShape) -> Result<(), CollabError> {
        shape.validate()?;
        self.shapes.push(shape);
        self.touch();
        Ok(())
    }

    /// 更新形状
    pub fn update_shape(&mut self, shape: WhiteboardShape) -> Result<(), CollabError> {
        shape.validate()?;
        let pos = self
            .shapes
            .iter()
            .position(|s| s.id == shape.id)
            .ok_or_else(|| CollabError::NotFound(format!("shape:{}", shape.id)))?;
        self.shapes[pos] = shape;
        self.touch();
        Ok(())
    }

    /// 删除形状
    pub fn delete_shape(&mut self, shape_id: ShapeId) -> Result<(), CollabError> {
        let before = self.shapes.len();
        self.shapes.retain(|s| s.id != shape_id);
        if self.shapes.len() == before {
            return Err(CollabError::NotFound(format!("shape:{}", shape_id)));
        }
        self.touch();
        Ok(())
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
        self.version = self.version.saturating_add(1);
    }
}

// =====================================================================
// 错误(§10.3)
// =====================================================================

/// CollabError — 协作域统一错误
#[derive(Debug, Error)]
pub enum CollabError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: actor tenant {0} vs resource tenant {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl CollabError {
    /// 错误码(对接 API 错误码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "COLLAB_NOT_FOUND",
            Self::PermissionDenied => "COLLAB_PERMISSION_DENIED",
            Self::CrossTenantDenied(_, _) => "COLLAB_CROSS_TENANT_DENIED",
            Self::InvalidState(_) => "COLLAB_INVALID_STATE",
            Self::Conflict(_) => "COLLAB_CONFLICT",
            Self::Internal(_) => "COLLAB_INTERNAL",
        }
    }
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSessionCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub parent_type: CollabParentType,
    pub parent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSessionCommand {
    pub tenant_id: TenantId,
    pub session_id: CollaborationSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresenceCommand {
    pub tenant_id: TenantId,
    pub session_id: CollaborationSessionId,
    pub user_id: UserId,
    pub cursor_position: Option<CursorPosition>,
    pub selection: Option<SelectionRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCursorCommand {
    pub tenant_id: TenantId,
    pub session_id: CollaborationSessionId,
    pub user_id: UserId,
    pub x: f32,
    pub y: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddShapeCommand {
    pub tenant_id: TenantId,
    pub whiteboard_id: WhiteboardId,
    pub shape: WhiteboardShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShapeCommand {
    pub tenant_id: TenantId,
    pub whiteboard_id: WhiteboardId,
    pub shape: WhiteboardShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteShapeCommand {
    pub tenant_id: TenantId,
    pub whiteboard_id: WhiteboardId,
    pub shape_id: ShapeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionQuery {
    pub tenant_id: TenantId,
    pub session_id: CollaborationSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActivePresencesQuery {
    pub tenant_id: TenantId,
    pub session_id: CollaborationSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWhiteboardQuery {
    pub tenant_id: TenantId,
    pub whiteboard_id: WhiteboardId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// CollabCommandPort — 写操作(§10.3)
#[async_trait]
pub trait CollabCommandPort: Send + Sync {
    /// 启动一个协作 Session
    async fn start_session(
        &self,
        cmd: StartSessionCommand,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollabError>;

    /// 结束 Session(校验所有权 / admin)
    async fn end_session(
        &self,
        cmd: EndSessionCommand,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollabError>;

    /// 更新 Presence(心跳 + 可选 cursor / selection)
    async fn update_presence(
        &self,
        cmd: UpdatePresenceCommand,
        actor: &ActorContext,
    ) -> Result<Presence, CollabError>;

    /// 更新 Cursor(upsert per (session, user))
    async fn update_cursor(
        &self,
        cmd: UpdateCursorCommand,
        actor: &ActorContext,
    ) -> Result<Cursor, CollabError>;

    /// 在 Whiteboard 上添加形状
    async fn add_shape(
        &self,
        cmd: AddShapeCommand,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError>;

    /// 更新 Whiteboard 上的形状
    async fn update_shape(
        &self,
        cmd: UpdateShapeCommand,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError>;

    /// 删除 Whiteboard 上的形状
    async fn delete_shape(
        &self,
        cmd: DeleteShapeCommand,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError>;
}

/// CollabQueryPort — 读操作(§10.3)
#[async_trait]
pub trait CollabQueryPort: Send + Sync {
    /// 获取 Session 详情
    async fn get_session(
        &self,
        q: GetSessionQuery,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollabError>;

    /// 列出某 Session 内所有活跃 Presence
    async fn list_active_presences(
        &self,
        q: ListActivePresencesQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Presence>, CollabError>;

    /// 获取 Whiteboard
    async fn get_whiteboard(
        &self,
        q: GetWhiteboardQuery,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError>;
}

/// CollabRepository — 持久化抽象
#[async_trait]
pub trait CollabRepository: Send + Sync {
    async fn insert_session(&self, s: CollaborationSession) -> Result<(), CollabError>;
    async fn get_session(
        &self,
        tenant_id: TenantId,
        id: CollaborationSessionId,
    ) -> Result<CollaborationSession, CollabError>;
    async fn update_session(&self, s: CollaborationSession) -> Result<(), CollabError>;

    async fn upsert_presence(&self, p: Presence) -> Result<(), CollabError>;
    async fn list_presences(
        &self,
        tenant_id: TenantId,
        session_id: CollaborationSessionId,
    ) -> Result<Vec<Presence>, CollabError>;

    async fn upsert_cursor(&self, c: Cursor) -> Result<(), CollabError>;
    async fn get_cursor(
        &self,
        tenant_id: TenantId,
        session_id: CollaborationSessionId,
        user_id: UserId,
    ) -> Result<Option<Cursor>, CollabError>;

    async fn insert_whiteboard(&self, w: Whiteboard) -> Result<(), CollabError>;
    async fn get_whiteboard(
        &self,
        tenant_id: TenantId,
        id: WhiteboardId,
    ) -> Result<Whiteboard, CollabError>;
    async fn update_whiteboard(&self, w: Whiteboard) -> Result<(), CollabError>;
}

// =====================================================================
// InMemoryCollabService — 内存实现
// =====================================================================

/// InMemoryCollabService — 协作域内存实现
pub struct InMemoryCollabService {
    repo: Arc<dyn CollabRepository>,
    sessions: Arc<RwLock<HashMap<CollaborationSessionId, CollaborationSession>>>,
    presences: Arc<RwLock<HashMap<(CollaborationSessionId, UserId), Presence>>>,
    cursors: Arc<RwLock<HashMap<(CollaborationSessionId, UserId), Cursor>>>,
    /// Cursor 颜色唯一性表(INV-CL-03):(session_id, user_id) -> color
    cursor_colors: Arc<RwLock<HashMap<(CollaborationSessionId, UserId), String>>>,
    whiteboards: Arc<RwLock<HashMap<WhiteboardId, Whiteboard>>>,
}

impl InMemoryCollabService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryCollabRepository::new()),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            presences: Arc::new(RwLock::new(HashMap::new())),
            cursors: Arc::new(RwLock::new(HashMap::new())),
            cursor_colors: Arc::new(RwLock::new(HashMap::new())),
            whiteboards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_repo(repo: Arc<dyn CollabRepository>) -> Self {
        Self {
            repo,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            presences: Arc::new(RwLock::new(HashMap::new())),
            cursors: Arc::new(RwLock::new(HashMap::new())),
            cursor_colors: Arc::new(RwLock::new(HashMap::new())),
            whiteboards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn require_session(
        &self,
        tenant_id: TenantId,
        session_id: CollaborationSessionId,
    ) -> Result<CollaborationSession, CollabError> {
        let s = self
            .sessions
            .read()
            .unwrap()
            .get(&session_id)
            .cloned()
            .ok_or_else(|| CollabError::NotFound(format!("session:{}", session_id)))?;
        s.assert_tenant(tenant_id)?;
        Ok(s)
    }

    fn require_whiteboard(
        &self,
        tenant_id: TenantId,
        whiteboard_id: WhiteboardId,
    ) -> Result<Whiteboard, CollabError> {
        let w = self
            .whiteboards
            .read()
            .unwrap()
            .get(&whiteboard_id)
            .cloned()
            .ok_or_else(|| CollabError::NotFound(format!("whiteboard:{}", whiteboard_id)))?;
        // INV-CL-02
        if w.tenant_id != tenant_id {
            return Err(CollabError::CrossTenantDenied(tenant_id, w.tenant_id));
        }
        Ok(w)
    }

    /// 内部:校验 actor 对 session 的所有权 / admin 权限
    fn check_session_actor_rights(
        &self,
        session: &CollaborationSession,
        actor: &ActorContext,
        allow_admin: bool,
    ) -> Result<(), CollabError> {
        // tenant 边界
        session.assert_tenant(TenantId::from(actor.tenant_id))?;
        // 权限:session 发起人或 admin
        if session.started_by == UserId::from(actor.user_id) {
            return Ok(());
        }
        if allow_admin && actor.is_platform_admin {
            return Ok(());
        }
        Err(CollabError::PermissionDenied)
    }
}

impl Default for InMemoryCollabService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CollabCommandPort for InMemoryCollabService {
    async fn start_session(
        &self,
        cmd: StartSessionCommand,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollabError> {
        // INV-CL-04
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(CollabError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == cmd.project_id.as_uuid())
            && !actor.is_platform_admin
            && !actor.is_local_runtime
        {
            return Err(CollabError::PermissionDenied);
        }
        let s = CollaborationSession::new_active(
            cmd.tenant_id,
            cmd.project_id,
            cmd.parent_type,
            cmd.parent_id,
            UserId::from(actor.user_id),
        );
        self.repo.insert_session(s.clone()).await?;
        self.sessions.write().unwrap().insert(s.id, s.clone());
        Ok(s)
    }

    async fn end_session(
        &self,
        cmd: EndSessionCommand,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollabError> {
        let mut s = self.require_session(cmd.tenant_id, cmd.session_id)?;
        // 权限:session 发起人或 admin
        self.check_session_actor_rights(&s, actor, true)?;
        if s.status.is_terminal() {
            return Err(CollabError::InvalidState(format!(
                "session {} already ended",
                s.id
            )));
        }
        s.status = CollabSessionStatus::Ended;
        s.ended_at = Some(Utc::now());
        s.bump();
        self.repo.update_session(s.clone()).await?;
        self.sessions.write().unwrap().insert(s.id, s.clone());
        Ok(s)
    }

    async fn update_presence(
        &self,
        cmd: UpdatePresenceCommand,
        actor: &ActorContext,
    ) -> Result<Presence, CollabError> {
        let s = self.require_session(cmd.tenant_id, cmd.session_id)?;
        // 权限:只能更新自己的 presence(或 admin)
        if cmd.user_id != UserId::from(actor.user_id) && !actor.is_platform_admin {
            return Err(CollabError::PermissionDenied);
        }
        // INV-CL-05:Ended 不可再写
        s.assert_writable()?;
        let now = Utc::now();
        let mut p = self
            .presences
            .write()
            .unwrap()
            .entry((cmd.session_id, cmd.user_id))
            .or_insert_with(|| Presence::new(cmd.session_id, cmd.user_id))
            .clone();
        p.touch();
        p.cursor_position = cmd.cursor_position;
        p.selection = cmd.selection;
        p.recompute_active(now);
        self.repo.upsert_presence(p.clone()).await?;
        self.presences
            .write()
            .unwrap()
            .insert((cmd.session_id, cmd.user_id), p.clone());
        Ok(p)
    }

    async fn update_cursor(
        &self,
        cmd: UpdateCursorCommand,
        actor: &ActorContext,
    ) -> Result<Cursor, CollabError> {
        let s = self.require_session(cmd.tenant_id, cmd.session_id)?;
        if cmd.user_id != UserId::from(actor.user_id) && !actor.is_platform_admin {
            return Err(CollabError::PermissionDenied);
        }
        s.assert_writable()?;
        // INV-CL-03:Cursor 颜色唯一 per user — 首次写入时锁定 color,后续必须一致
        let color_key = (cmd.session_id, cmd.user_id);
        {
            let colors = self.cursor_colors.read().unwrap();
            if let Some(existing) = colors.get(&color_key) {
                if existing != &cmd.color {
                    return Err(CollabError::Conflict(format!(
                        "cursor color for user {} already set to {} (got {})",
                        cmd.user_id, existing, cmd.color
                    )));
                }
            }
        }
        // 颜色空字符串视为非法
        if cmd.color.trim().is_empty() {
            return Err(CollabError::InvalidState(
                "cursor color must not be empty".to_string(),
            ));
        }
        // upsert:若已存在同 (session, user) 的 cursor,复用 id
        let cursor = {
            let map = self.cursors.read().unwrap();
            map.get(&color_key).cloned()
        };
        let cursor = match cursor {
            Some(mut existing) => {
                existing.x = cmd.x;
                existing.y = cmd.y;
                existing.color = cmd.color.clone();
                existing.updated_at = Utc::now();
                existing
            }
            None => Cursor::new(cmd.session_id, cmd.user_id, cmd.x, cmd.y, cmd.color.clone()),
        };
        self.repo.upsert_cursor(cursor.clone()).await?;
        self.cursors
            .write()
            .unwrap()
            .insert(color_key, cursor.clone());
        self.cursor_colors
            .write()
            .unwrap()
            .insert(color_key, cmd.color);
        Ok(cursor)
    }

    async fn add_shape(
        &self,
        cmd: AddShapeCommand,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError> {
        let mut w = self.require_whiteboard(cmd.tenant_id, cmd.whiteboard_id)?;
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == w.project_id.as_uuid())
            && !actor.is_platform_admin
        {
            return Err(CollabError::PermissionDenied);
        }
        let mut shape = cmd.shape;
        shape.updated_at = Utc::now();
        if shape.created_at.timestamp() == 0 {
            shape.created_at = Utc::now();
        }
        w.add_shape(shape)?;
        self.repo.update_whiteboard(w.clone()).await?;
        self.whiteboards.write().unwrap().insert(w.id, w.clone());
        Ok(w)
    }

    async fn update_shape(
        &self,
        cmd: UpdateShapeCommand,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError> {
        let mut w = self.require_whiteboard(cmd.tenant_id, cmd.whiteboard_id)?;
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == w.project_id.as_uuid())
            && !actor.is_platform_admin
        {
            return Err(CollabError::PermissionDenied);
        }
        let mut shape = cmd.shape;
        shape.updated_at = Utc::now();
        w.update_shape(shape)?;
        self.repo.update_whiteboard(w.clone()).await?;
        self.whiteboards.write().unwrap().insert(w.id, w.clone());
        Ok(w)
    }

    async fn delete_shape(
        &self,
        cmd: DeleteShapeCommand,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError> {
        let mut w = self.require_whiteboard(cmd.tenant_id, cmd.whiteboard_id)?;
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == w.project_id.as_uuid())
            && !actor.is_platform_admin
        {
            return Err(CollabError::PermissionDenied);
        }
        w.delete_shape(cmd.shape_id)?;
        self.repo.update_whiteboard(w.clone()).await?;
        self.whiteboards.write().unwrap().insert(w.id, w.clone());
        Ok(w)
    }
}

#[async_trait]
impl CollabQueryPort for InMemoryCollabService {
    async fn get_session(
        &self,
        q: GetSessionQuery,
        actor: &ActorContext,
    ) -> Result<CollaborationSession, CollabError> {
        let s = self.require_session(q.tenant_id, q.session_id)?;
        // 读权限:tenant 内 + project 内(或 admin)
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == s.project_id.as_uuid())
            && !actor.is_platform_admin
        {
            return Err(CollabError::PermissionDenied);
        }
        Ok(s)
    }

    async fn list_active_presences(
        &self,
        q: ListActivePresencesQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Presence>, CollabError> {
        let s = self.require_session(q.tenant_id, q.session_id)?;
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == s.project_id.as_uuid())
            && !actor.is_platform_admin
        {
            return Err(CollabError::PermissionDenied);
        }
        let now = Utc::now();
        // 先合并 repo 持久化层 + service 缓存(覆盖运行时新增的 presence)
        let mut all: HashMap<(CollaborationSessionId, UserId), Presence> = self
            .repo
            .list_presences(q.tenant_id, q.session_id)
            .await?
            .into_iter()
            .map(|p| ((p.session_id, p.user_id), p))
            .collect();
        for (k, p) in self.presences.read().unwrap().iter() {
            if k.0 == q.session_id {
                all.insert(*k, p.clone());
            }
        }
        let mut out = Vec::with_capacity(all.len());
        for (_, mut p) in all {
            p.recompute_active(now);
            if p.is_active {
                out.push(p);
            }
        }
        Ok(out)
    }

    async fn get_whiteboard(
        &self,
        q: GetWhiteboardQuery,
        actor: &ActorContext,
    ) -> Result<Whiteboard, CollabError> {
        let w = self.require_whiteboard(q.tenant_id, q.whiteboard_id)?;
        if !actor
            .project_ids
            .iter()
            .any(|p| *p == w.project_id.as_uuid())
            && !actor.is_platform_admin
        {
            return Err(CollabError::PermissionDenied);
        }
        Ok(w)
    }
}

// =====================================================================
// InMemoryCollabRepository
// =====================================================================

/// InMemoryCollabRepository — 内存版持久化抽象
pub struct InMemoryCollabRepository {
    sessions: RwLock<HashMap<CollaborationSessionId, CollaborationSession>>,
    presences: RwLock<HashMap<(CollaborationSessionId, UserId), Presence>>,
    cursors: RwLock<HashMap<(CollaborationSessionId, UserId), Cursor>>,
    whiteboards: RwLock<HashMap<WhiteboardId, Whiteboard>>,
}

impl InMemoryCollabRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            presences: RwLock::new(HashMap::new()),
            cursors: RwLock::new(HashMap::new()),
            whiteboards: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCollabRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CollabRepository for InMemoryCollabRepository {
    async fn insert_session(&self, s: CollaborationSession) -> Result<(), CollabError> {
        let mut store = self.sessions.write().expect("lock");
        if store.contains_key(&s.id) {
            return Err(CollabError::Conflict(format!(
                "session {} already exists",
                s.id
            )));
        }
        store.insert(s.id, s);
        Ok(())
    }

    async fn get_session(
        &self,
        tenant_id: TenantId,
        id: CollaborationSessionId,
    ) -> Result<CollaborationSession, CollabError> {
        let s = self
            .sessions
            .read()
            .expect("lock")
            .get(&id)
            .cloned()
            .ok_or_else(|| CollabError::NotFound(format!("session:{}", id)))?;
        if s.tenant_id != tenant_id {
            return Err(CollabError::CrossTenantDenied(tenant_id, s.tenant_id));
        }
        Ok(s)
    }

    async fn update_session(&self, s: CollaborationSession) -> Result<(), CollabError> {
        self.sessions.write().expect("lock").insert(s.id, s);
        Ok(())
    }

    async fn upsert_presence(&self, p: Presence) -> Result<(), CollabError> {
        self.presences
            .write()
            .expect("lock")
            .insert((p.session_id, p.user_id), p);
        Ok(())
    }

    async fn list_presences(
        &self,
        _tenant_id: TenantId,
        session_id: CollaborationSessionId,
    ) -> Result<Vec<Presence>, CollabError> {
        Ok(self
            .presences
            .read()
            .expect("lock")
            .values()
            .filter(|p| p.session_id == session_id)
            .cloned()
            .collect())
    }

    async fn upsert_cursor(&self, c: Cursor) -> Result<(), CollabError> {
        self.cursors
            .write()
            .expect("lock")
            .insert((c.session_id, c.user_id), c);
        Ok(())
    }

    async fn get_cursor(
        &self,
        _tenant_id: TenantId,
        session_id: CollaborationSessionId,
        user_id: UserId,
    ) -> Result<Option<Cursor>, CollabError> {
        Ok(self
            .cursors
            .read()
            .expect("lock")
            .get(&(session_id, user_id))
            .cloned())
    }

    async fn insert_whiteboard(&self, w: Whiteboard) -> Result<(), CollabError> {
        let mut store = self.whiteboards.write().expect("lock");
        if store.contains_key(&w.id) {
            return Err(CollabError::Conflict(format!(
                "whiteboard {} already exists",
                w.id
            )));
        }
        store.insert(w.id, w);
        Ok(())
    }

    async fn get_whiteboard(
        &self,
        tenant_id: TenantId,
        id: WhiteboardId,
    ) -> Result<Whiteboard, CollabError> {
        let w = self
            .whiteboards
            .read()
            .expect("lock")
            .get(&id)
            .cloned()
            .ok_or_else(|| CollabError::NotFound(format!("whiteboard:{}", id)))?;
        if w.tenant_id != tenant_id {
            return Err(CollabError::CrossTenantDenied(tenant_id, w.tenant_id));
        }
        Ok(w)
    }

    async fn update_whiteboard(&self, w: Whiteboard) -> Result<(), CollabError> {
        self.whiteboards.write().expect("lock").insert(w.id, w);
        Ok(())
    }
}

// =====================================================================
// 测试模块(≥12 个测试用例)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(user: UserId, tenant: TenantId, project: ProjectId) -> ActorContext {
        ActorContext::new(user.0, tenant.0)
            .with_project(project.as_uuid())
            .with_role("developer")
    }

    fn make_admin_actor(user: UserId, tenant: TenantId, project: ProjectId) -> ActorContext {
        ActorContext::new(user.0, tenant.0)
            .with_project(project.as_uuid())
            .with_role("tenant_admin")
    }

    fn make_shape(kind: ShapeKind, x: f32, y: f32, w: f32, h: f32, color: &str) -> WhiteboardShape {
        WhiteboardShape {
            id: ShapeId::new(),
            kind,
            x,
            y,
            w,
            h,
            content: None,
            color: color.to_string(),
            created_by: UserId.new(),
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            updated_at: Utc::now(),
        }
    }

    // -----------------------------------------------------------------
    // 1. start_session
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_start_session() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let cmd = StartSessionCommand {
            tenant_id: tenant,
            project_id: project,
            parent_type: CollabParentType::WorkItem,
            parent_id: Uuid::new_v4(),
        };
        let s = svc.start_session(cmd, &actor).await.expect("start");
        assert_eq!(s.status, CollabSessionStatus::Active);
        assert_eq!(s.tenant_id, tenant);
        assert_eq!(s.project_id, project);
        assert_eq!(s.parent_type, CollabParentType::WorkItem);
    }

    // -----------------------------------------------------------------
    // 2. end_session
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_end_session() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let actor = make_actor(user, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Document,
                    parent_id: Uuid::new_v4(),
                },
                &actor,
            )
            .await
            .expect("start");
        let ended = svc
            .end_session(
                EndSessionCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                },
                &actor,
            )
            .await
            .expect("end");
        assert_eq!(ended.status, CollabSessionStatus::Ended);
        assert!(ended.ended_at.is_some());
    }

    // -----------------------------------------------------------------
    // 3. end_ended_session_rejected
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_end_ended_session_rejected() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let actor = make_actor(user, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Whiteboard,
                    parent_id: Uuid::new_v4(),
                },
                &actor,
            )
            .await
            .expect("start");
        svc.end_session(
            EndSessionCommand {
                tenant_id: tenant,
                session_id: s.id,
            },
            &actor,
        )
        .await
        .expect("first end");
        // 再次 end 应被拒绝
        let r = svc
            .end_session(
                EndSessionCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                },
                &actor,
            )
            .await;
        assert!(matches!(r, Err(CollabError::InvalidState(_))));
    }

    // -----------------------------------------------------------------
    // 4. cross_tenant_session_denied
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_cross_tenant_session_denied() {
        let svc = InMemoryCollabService::new();
        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor_a = make_actor(uuid::Uuid::new_v4(), tenant_a, project);
        let actor_b = make_actor(uuid::Uuid::new_v4(), tenant_b, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant_a,
                    project_id: project,
                    parent_type: CollabParentType::WorkItem,
                    parent_id: Uuid::new_v4(),
                },
                &actor_a,
            )
            .await
            .expect("start");
        // 跨 tenant 查询
        let r = svc
            .get_session(
                GetSessionQuery {
                    tenant_id: tenant_b,
                    session_id: s.id,
                },
                &actor_b,
            )
            .await;
        assert!(matches!(r, Err(CollabError::CrossTenantDenied(_, _))));
    }

    // -----------------------------------------------------------------
    // 5. update_presence_increments_heartbeat
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_update_presence_increments_heartbeat() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let actor = make_actor(user, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Document,
                    parent_id: Uuid::new_v4(),
                },
                &actor,
            )
            .await
            .expect("start");
        let p1 = svc
            .update_presence(
                UpdatePresenceCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                    user_id: user,
                    cursor_position: Some(CursorPosition::new(1.0, 2.0)),
                    selection: None,
                },
                &actor,
            )
            .await
            .expect("presence 1");
        // sleep 10ms 保证 timestamp 不同
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let p2 = svc
            .update_presence(
                UpdatePresenceCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                    user_id: user,
                    cursor_position: Some(CursorPosition::new(3.0, 4.0)),
                    selection: None,
                },
                &actor,
            )
            .await
            .expect("presence 2");
        assert!(p2.last_heartbeat > p1.last_heartbeat);
        assert!(p2.is_active);
    }

    // -----------------------------------------------------------------
    // 6. presence_inactive_for_ended_session
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_presence_inactive_for_ended_session() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let actor = make_actor(user, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Document,
                    parent_id: Uuid::new_v4(),
                },
                &actor,
            )
            .await
            .expect("start");
        svc.end_session(
            EndSessionCommand {
                tenant_id: tenant,
                session_id: s.id,
            },
            &actor,
        )
        .await
        .expect("end");
        let r = svc
            .update_presence(
                UpdatePresenceCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                    user_id: user,
                    cursor_position: None,
                    selection: None,
                },
                &actor,
            )
            .await;
        assert!(matches!(r, Err(CollabError::InvalidState(_))));
    }

    // -----------------------------------------------------------------
    // 7. update_cursor_upsert
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_update_cursor_upsert() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let actor = make_actor(user, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Document,
                    parent_id: Uuid::new_v4(),
                },
                &actor,
            )
            .await
            .expect("start");
        let c1 = svc
            .update_cursor(
                UpdateCursorCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                    user_id: user,
                    x: 10.0,
                    y: 20.0,
                    color: "#ff0000".to_string(),
                },
                &actor,
            )
            .await
            .expect("cursor 1");
        assert_eq!(c1.x, 10.0);
        // 同一 user 同一 session 第二次 update(同 color)— upsert
        let c2 = svc
            .update_cursor(
                UpdateCursorCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                    user_id: user,
                    x: 50.0,
                    y: 60.0,
                    color: "#ff0000".to_string(),
                },
                &actor,
            )
            .await
            .expect("cursor 2");
        assert_eq!(c2.x, 50.0);
        assert_eq!(c2.y, 60.0);
        assert_eq!(c2.id, c1.id, "upsert 应该复用 id");
    }

    // -----------------------------------------------------------------
    // 8. cursor_color_per_user(INV-CL-03)
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_cursor_color_per_user() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let actor = make_actor(user, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Whiteboard,
                    parent_id: Uuid::new_v4(),
                },
                &actor,
            )
            .await
            .expect("start");
        // 首次设定 color
        svc.update_cursor(
            UpdateCursorCommand {
                tenant_id: tenant,
                session_id: s.id,
                user_id: user,
                x: 0.0,
                y: 0.0,
                color: "#00ff00".to_string(),
            },
            &actor,
        )
        .await
        .expect("first color");
        // 改 color 应被拒绝
        let r = svc
            .update_cursor(
                UpdateCursorCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                    user_id: user,
                    x: 0.0,
                    y: 0.0,
                    color: "#0000ff".to_string(),
                },
                &actor,
            )
            .await;
        assert!(matches!(r, Err(CollabError::Conflict(_))));
    }

    // -----------------------------------------------------------------
    // 9. add_shape
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_add_shape() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let wb = Whiteboard::new(tenant, project, "demo".to_string());
        let wb_id = wb.id;
        // 直接写入 in-memory store
        svc.whiteboards.write().unwrap().insert(wb_id, wb);
        let shape = make_shape(ShapeKind::Rectangle, 1.0, 2.0, 30.0, 40.0, "#aaa");
        let w = svc
            .add_shape(
                AddShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape,
                },
                &actor,
            )
            .await
            .expect("add");
        assert_eq!(w.shapes.len(), 1);
        assert_eq!(w.shapes[0].kind, ShapeKind::Rectangle);
    }

    // -----------------------------------------------------------------
    // 10. update_shape
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_update_shape() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let wb = Whiteboard::new(tenant, project, "demo".to_string());
        let wb_id = wb.id;
        svc.whiteboards.write().unwrap().insert(wb_id, wb);
        let shape = make_shape(ShapeKind::Ellipse, 0.0, 0.0, 10.0, 20.0, "#bbb");
        let shape_id = shape.id;
        let w1 = svc
            .add_shape(
                AddShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape,
                },
                &actor,
            )
            .await
            .expect("add");
        let mut updated = w1.shapes[0].clone();
        updated.w = 99.0;
        updated.color = "#ccc".to_string();
        let w2 = svc
            .update_shape(
                UpdateShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape: updated.clone(),
                },
                &actor,
            )
            .await
            .expect("update");
        let got = w2.shapes.iter().find(|s| s.id == shape_id).expect("find");
        assert_eq!(got.w, 99.0);
        assert_eq!(got.color, "#ccc");
    }

    // -----------------------------------------------------------------
    // 11. delete_shape
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_shape() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let wb = Whiteboard::new(tenant, project, "demo".to_string());
        let wb_id = wb.id;
        svc.whiteboards.write().unwrap().insert(wb_id, wb);
        let shape = make_shape(ShapeKind::Arrow, 5.0, 5.0, 15.0, 25.0, "#ddd");
        let shape_id = shape.id;
        svc.add_shape(
            AddShapeCommand {
                tenant_id: tenant,
                whiteboard_id: wb_id,
                shape,
            },
            &actor,
        )
        .await
        .expect("add");
        let w = svc
            .delete_shape(
                DeleteShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape_id,
                },
                &actor,
            )
            .await
            .expect("delete");
        assert_eq!(w.shapes.len(), 0);
        // 重复删除应 NotFound
        let r = svc
            .delete_shape(
                DeleteShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape_id,
                },
                &actor,
            )
            .await;
        assert!(matches!(r, Err(CollabError::NotFound(_))));
    }

    // -----------------------------------------------------------------
    // 12. list_active_presences
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_list_active_presences() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let host = uuid::Uuid::new_v4();
        let host_actor = make_actor(host, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Document,
                    parent_id: Uuid::new_v4(),
                },
                &host_actor,
            )
            .await
            .expect("start");
        let u2 = uuid::Uuid::new_v4();
        let u3 = uuid::Uuid::new_v4();
        // host 给自己写 presence
        svc.update_presence(
            UpdatePresenceCommand {
                tenant_id: tenant,
                session_id: s.id,
                user_id: host,
                cursor_position: None,
                selection: None,
            },
            &host_actor,
        )
        .await
        .expect("p1");
        // u2 / u3 直接进 in-memory store(模拟远程 user 的 presence)
        {
            let mut map = svc.presences.write().unwrap();
            map.insert((s.id, u2), Presence::new(s.id, u2));
            map.insert((s.id, u3), Presence::new(s.id, u3));
        }
        let active = svc
            .list_active_presences(
                ListActivePresencesQuery {
                    tenant_id: tenant,
                    session_id: s.id,
                },
                &host_actor,
            )
            .await
            .expect("list");
        // host + u2 + u3 三个,都是新近心跳
        assert_eq!(active.len(), 3);
    }

    // -----------------------------------------------------------------
    // 13. get_whiteboard
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_get_whiteboard() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let wb = Whiteboard::new(tenant, project, "board-1".to_string());
        let wb_id = wb.id;
        svc.whiteboards.write().unwrap().insert(wb_id, wb);
        let got = svc
            .get_whiteboard(
                GetWhiteboardQuery {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                },
                &actor,
            )
            .await
            .expect("get");
        assert_eq!(got.name, "board-1");
        assert_eq!(got.tenant_id, tenant);
        // 跨 tenant 拒绝
        let other_tenant = uuid::Uuid::new_v4();
        let r = svc
            .get_whiteboard(
                GetWhiteboardQuery {
                    tenant_id: other_tenant,
                    whiteboard_id: wb_id,
                },
                &actor,
            )
            .await;
        assert!(matches!(r, Err(CollabError::CrossTenantDenied(_, _))));
    }

    // -----------------------------------------------------------------
    // 14. multiple_shapes_on_whiteboard
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_multiple_shapes_on_whiteboard() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let wb = Whiteboard::new(tenant, project, "multi".to_string());
        let wb_id = wb.id;
        svc.whiteboards.write().unwrap().insert(wb_id, wb);
        let kinds = [
            ShapeKind::Rectangle,
            ShapeKind::Ellipse,
            ShapeKind::Text,
            ShapeKind::Arrow,
            ShapeKind::StickyNote,
        ];
        for k in &kinds {
            svc.add_shape(
                AddShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape: make_shape(*k, 0.0, 0.0, 10.0, 10.0, "#fff"),
                },
                &actor,
            )
            .await
            .expect("add");
        }
        let w = svc
            .get_whiteboard(
                GetWhiteboardQuery {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                },
                &actor,
            )
            .await
            .expect("get");
        assert_eq!(w.shapes.len(), 5);
        for (i, k) in kinds.iter().enumerate() {
            assert_eq!(w.shapes[i].kind, *k);
        }
    }

    // -----------------------------------------------------------------
    // 15. invalid_shape_dimensions_rejected
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_invalid_shape_dimensions_rejected() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(uuid::Uuid::new_v4(), tenant, project);
        let wb = Whiteboard::new(tenant, project, "dim".to_string());
        let wb_id = wb.id;
        svc.whiteboards.write().unwrap().insert(wb_id, wb);
        // w < 0
        let bad_w = make_shape(ShapeKind::Rectangle, 0.0, 0.0, -1.0, 10.0, "#000");
        let r1 = svc
            .add_shape(
                AddShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape: bad_w,
                },
                &actor,
            )
            .await;
        assert!(matches!(r1, Err(CollabError::InvalidState(_))));
        // h < 0
        let bad_h = make_shape(ShapeKind::Ellipse, 0.0, 0.0, 10.0, -5.0, "#000");
        let r2 = svc
            .add_shape(
                AddShapeCommand {
                    tenant_id: tenant,
                    whiteboard_id: wb_id,
                    shape: bad_h,
                },
                &actor,
            )
            .await;
        assert!(matches!(r2, Err(CollabError::InvalidState(_))));
    }

    // -----------------------------------------------------------------
    // 16. 额外:admin 可以 end 其他人的 session
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_admin_can_end_others_session() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let host = uuid::Uuid::new_v4();
        let host_actor = make_actor(host, tenant, project);
        let admin = uuid::Uuid::new_v4();
        let admin_actor = make_admin_actor(admin, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Document,
                    parent_id: Uuid::new_v4(),
                },
                &host_actor,
            )
            .await
            .expect("start");
        let ended = svc
            .end_session(
                EndSessionCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                },
                &admin_actor,
            )
            .await
            .expect("admin end");
        assert_eq!(ended.status, CollabSessionStatus::Ended);
    }

    // -----------------------------------------------------------------
    // 17. 额外:非 admin / 非 host 不能 end 别人的 session
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn test_non_owner_cannot_end_others_session() {
        let svc = InMemoryCollabService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let host = uuid::Uuid::new_v4();
        let host_actor = make_actor(host, tenant, project);
        let other = uuid::Uuid::new_v4();
        let other_actor = make_actor(other, tenant, project);
        let s = svc
            .start_session(
                StartSessionCommand {
                    tenant_id: tenant,
                    project_id: project,
                    parent_type: CollabParentType::Whiteboard,
                    parent_id: Uuid::new_v4(),
                },
                &host_actor,
            )
            .await
            .expect("start");
        let r = svc
            .end_session(
                EndSessionCommand {
                    tenant_id: tenant,
                    session_id: s.id,
                },
                &other_actor,
            )
            .await;
        assert!(matches!(r, Err(CollabError::PermissionDenied)));
    }
}

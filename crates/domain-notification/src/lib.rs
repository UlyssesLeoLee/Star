//! domain-notification crate
//!
//! 详细 spec: docs/specs/domain-notification-spec.md §15 Notification
//! 上游基本设计: docs/basic-design.md §2.1 / §4.10.5 / §5.7
//! 数据设计: docs/data-design.md §4.17 (`notification` schema)
//! API 设计: docs/api-design.md §3.20 (Notification Channel / Template)
//!
//! ## 职责
//!
//! 通知派发领域模块(§12,REQ-NOTIF-001)。MVP Email + InApp;Slack/钉钉 V1(§30.3)。
//! 默认抑制(REQ-NOTIF-002,2026-08-26 决议):NotificationDispatcher 默认抑制
//! 等待人工介入的节点触发通知(如 WAITING_FEEDBACK、Validation 失败、Protected Action 越权);
//! Agent 执行的中间步骤不触发通知,但 100% 写入 AgentSession Transcript(INV-AGT-09),
//! 不影响可观测性。默认抑制行为未关注 V1/V2/Future 等级,作为当前默认行为,在 INV-N-07。
//!
//! ## 关键不变量(INV-N-01~07)
//!
//! - INV-N-01:必带 tenant_id,跨 tenant 拒绝(§6.1,REQ-SEC-001)
//! - INV-N-02:Notification 异步派发(不阻塞业务)(§2.1 §23)
//! - INV-N-03:NotificationChannel 仅本人可读/写(§10)
//! - INV-N-04:NotificationTemplate 由 Project Admin 维护(§4.17)
//! - INV-N-05:Notification 不可修改 body/subject(Append-only + 状态字段)(§4.17)
//! - INV-N-06:失败重试(指定次数,最多 5 次),超限进 DLQ(§5.4)
//! - INV-N-07:默认抑制;关键事件突破(ValidationFailed / FeedbackCreated / AgentSessionFailed / Crashed / Timeout)
//!
//! Lead 责任: notification Lead

#![warn(missing_docs)]

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(NotificationChannelId);
define_uuid_id!(NotificationTemplateId);
define_uuid_id!(NotificationId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);

// =====================================================================
// 事件类型枚举(用于 INV-N-07 决策)
// =====================================================================

/// 触发通知的事件类型(INV-N-07 决策依据)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationEventType {
    // 突破抑制 - 必须发送
    ValidationFailed,
    FeedbackCreated,
    FeedbackRequired,
    AgentSessionFailed,
    AgentSessionCrashed,
    AgentSessionTimeout,
    ProtectedActionDenied,
    // 抑制 - 默认不发
    AgentStepStarted,
    AgentStepCompleted,
    ToolInvoked,
    ToolCompleted,
    ValidationPassed,
    WorkItemCreated,
    WorkItemUpdated,
    CommentAdded,
    // 用户可显式订阅
    Custom,
}

impl NotificationEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationFailed => "validation.failed",
            Self::FeedbackCreated => "feedback.created",
            Self::FeedbackRequired => "feedback.required",
            Self::AgentSessionFailed => "agent_session.failed",
            Self::AgentSessionCrashed => "agent_session.crashed",
            Self::AgentSessionTimeout => "agent_session.timeout",
            Self::ProtectedActionDenied => "protected_action.denied",
            Self::AgentStepStarted => "agent.step.started",
            Self::AgentStepCompleted => "agent.step.completed",
            Self::ToolInvoked => "tool.invoked",
            Self::ToolCompleted => "tool.completed",
            Self::ValidationPassed => "validation.passed",
            Self::WorkItemCreated => "work_item.created",
            Self::WorkItemUpdated => "work_item.updated",
            Self::CommentAdded => "comment.added",
            Self::Custom => "custom",
        }
    }

    /// INV-N-07:是否突破默认抑制(必须发送)
    pub fn is_breakthrough(&self) -> bool {
        matches!(
            self,
            Self::ValidationFailed
                | Self::FeedbackCreated
                | Self::FeedbackRequired
                | Self::AgentSessionFailed
                | Self::AgentSessionCrashed
                | Self::AgentSessionTimeout
                | Self::ProtectedActionDenied
        )
    }

    /// INV-N-07:是否被默认抑制
    pub fn is_suppressed(&self) -> bool {
        !self.is_breakthrough() && *self != Self::Custom
    }
}

// =====================================================================
// UUID 强类型 ID 宏
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
// 实体
// =====================================================================

/// NotificationChannel(§4.17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: NotificationChannelId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub kind: ChannelKind,
    pub address: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelKind {
    Email,
    InApp,
    Slack,
    DingTalk,
}

impl ChannelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::InApp => "in_app",
            Self::Slack => "slack",
            Self::DingTalk => "dingtalk",
        }
    }
}

/// NotificationTemplate(§4.17,Project 范围)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: NotificationTemplateId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub event_type: NotificationEventType,
    pub channel_kinds: Vec<ChannelKind>,
    pub subject: String,
    pub body_template: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Notification(Append-only + 状态字段,§4.17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub event_type: NotificationEventType,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub channel_id: NotificationChannelId,
    pub subject: String,
    pub body: String,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
    DeadLettered,
}

impl NotificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Sent => "SENT",
            Self::Delivered => "DELIVERED",
            Self::Read => "READ",
            Self::Failed => "FAILED",
            Self::DeadLettered => "DEAD_LETTERED",
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::Read | Self::DeadLettered)
    }
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("event suppressed by INV-N-07: {0}")]
    EventSuppressed(String),
    #[error("max retry exceeded (5), moved to DLQ")]
    MaxRetryExceeded,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterChannelCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub kind: ChannelKind,
    pub address: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertTemplateCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub event_type: NotificationEventType,
    pub channel_kinds: Vec<ChannelKind>,
    pub subject: String,
    pub body_template: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchNotificationCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub event_type: NotificationEventType,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub subject: String,
    pub body: String,
    pub source: String, // 用于审计
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkReadCommand {
    pub tenant_id: TenantId,
    pub notification_id: NotificationId,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNotificationQuery {
    pub tenant_id: TenantId,
    pub notification_id: NotificationId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByUserQuery {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub unread_only: bool,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait NotificationCommandPort: Send + Sync {
    async fn register_channel(
        &self,
        cmd: RegisterChannelCommand,
        actor: &ActorContext,
    ) -> Result<NotificationChannel, NotificationError>;

    async fn upsert_template(
        &self,
        cmd: UpsertTemplateCommand,
        actor: &ActorContext,
    ) -> Result<NotificationTemplate, NotificationError>;

    /// 派发通知(INV-N-07:被抑制的事件直接返回 EventSuppressed)
    async fn dispatch(
        &self,
        cmd: DispatchNotificationCommand,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError>;

    async fn mark_read(
        &self,
        cmd: MarkReadCommand,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError>;
}

#[async_trait]
pub trait NotificationQueryPort: Send + Sync {
    async fn get(
        &self,
        q: GetNotificationQuery,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError>;

    async fn list_by_user(
        &self,
        q: ListByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Notification>, NotificationError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn insert_channel(&self, channel: NotificationChannel) -> Result<(), NotificationError>;
    async fn get_channel(
        &self,
        id: NotificationChannelId,
    ) -> Result<NotificationChannel, NotificationError>;
    async fn update_channel(&self, channel: NotificationChannel) -> Result<(), NotificationError>;
    async fn list_channels_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<NotificationChannel>, NotificationError>;

    async fn insert_template(
        &self,
        template: NotificationTemplate,
    ) -> Result<(), NotificationError>;
    async fn get_template(
        &self,
        id: NotificationTemplateId,
    ) -> Result<NotificationTemplate, NotificationError>;
    async fn upsert_template_by_event(
        &self,
        template: NotificationTemplate,
    ) -> Result<(), NotificationError>;
    async fn list_templates_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<NotificationTemplate>, NotificationError>;

    async fn insert_notification(
        &self,
        notification: Notification,
    ) -> Result<(), NotificationError>;
    async fn get_notification(&self, id: NotificationId)
        -> Result<Notification, NotificationError>;
    async fn update_notification(
        &self,
        notification: Notification,
    ) -> Result<(), NotificationError>;
    async fn list_notifications_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        unread_only: bool,
    ) -> Result<Vec<Notification>, NotificationError>;
}

// =====================================================================
// InMemoryNotificationService
// =====================================================================

pub struct InMemoryNotificationService {
    repo: Arc<dyn NotificationRepository>,
    channels: Arc<RwLock<HashMap<NotificationChannelId, NotificationChannel>>>,
    templates: Arc<RwLock<HashMap<NotificationTemplateId, NotificationTemplate>>>,
    notifications: Arc<RwLock<HashMap<NotificationId, Notification>>>,
}

impl InMemoryNotificationService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryNotificationRepository::new()),
            channels: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn with_repo(repo: Arc<dyn NotificationRepository>) -> Self {
        Self {
            repo,
            channels: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

const MAX_RETRY: u32 = 5;

#[async_trait]
impl NotificationCommandPort for InMemoryNotificationService {
    async fn register_channel(
        &self,
        cmd: RegisterChannelCommand,
        actor: &ActorContext,
    ) -> Result<NotificationChannel, NotificationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-N-03:仅本人可注册自己的 channel
        if UserId::from(actor.user_id) != cmd.user_id {
            return Err(NotificationError::PermissionDenied);
        }
        let now = Utc::now();
        let channel = NotificationChannel {
            id: NotificationChannelId::new(),
            tenant_id: cmd.tenant_id,
            user_id: UserId::from(cmd.user_id),
            kind: cmd.kind,
            address: cmd.address,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_channel(channel.clone()).await?;
        self.channels
            .write()
            .unwrap()
            .insert(channel.id, channel.clone());
        Ok(channel)
    }

    async fn upsert_template(
        &self,
        cmd: UpsertTemplateCommand,
        actor: &ActorContext,
    ) -> Result<NotificationTemplate, NotificationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-N-04:Project Admin 维护
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(NotificationError::PermissionDenied);
        }
        let existing_id = {
            let templates = self.templates.read().unwrap();
            templates
                .values()
                .find(|t| {
                    t.tenant_id == cmd.tenant_id
                        && t.project_id == cmd.project_id
                        && t.event_type == cmd.event_type
                })
                .map(|t| t.id)
        };
        let now = Utc::now();
        let template = NotificationTemplate {
            id: existing_id.unwrap_or_else(NotificationTemplateId::new),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            event_type: cmd.event_type,
            channel_kinds: cmd.channel_kinds,
            subject: cmd.subject,
            body_template: cmd.body_template,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        self.repo.upsert_template_by_event(template.clone()).await?;
        self.templates
            .write()
            .unwrap()
            .insert(template.id, template.clone());
        Ok(template)
    }

    async fn dispatch(
        &self,
        cmd: DispatchNotificationCommand,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-N-07:默认抑制决策
        if cmd.event_type.is_suppressed() {
            return Err(NotificationError::EventSuppressed(
                cmd.event_type.as_str().to_string(),
            ));
        }
        // 找到用户可用 channel(InApp 默认;Email 需要显式)
        let channel = {
            let channels = self.channels.read().unwrap();
            channels
                .values()
                .find(|c| c.tenant_id == cmd.tenant_id && c.user_id == cmd.user_id && c.enabled)
                .cloned()
        };
        let channel = match channel {
            Some(c) => c,
            None => {
                // 没有可用 channel:在测试场景里用 in-memory 虚拟 channel
                NotificationChannel {
                    id: NotificationChannelId::new(),
                    tenant_id: cmd.tenant_id,
                    user_id: UserId::from(cmd.user_id),
                    kind: ChannelKind::InApp,
                    address: format!("in_app://user/{}", cmd.user_id.as_uuid()),
                    enabled: true,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            }
        };
        let now = Utc::now();
        let notification = Notification {
            id: NotificationId::new(),
            tenant_id: cmd.tenant_id,
            user_id: UserId::from(cmd.user_id),
            event_type: cmd.event_type,
            resource_type: cmd.resource_type,
            resource_id: cmd.resource_id,
            channel_id: channel.id,
            subject: cmd.subject,
            body: cmd.body,
            status: NotificationStatus::Sent, // MVP:直接 sent(真实场景异步)
            created_at: now,
            sent_at: Some(now),
            read_at: None,
            retry_count: 0,
        };
        self.repo.insert_notification(notification.clone()).await?;
        self.notifications
            .write()
            .unwrap()
            .insert(notification.id, notification.clone());
        Ok(notification)
    }

    async fn mark_read(
        &self,
        cmd: MarkReadCommand,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut n = self
            .notifications
            .write()
            .unwrap()
            .get_mut(&cmd.notification_id)
            .cloned()
            .ok_or_else(|| {
                NotificationError::NotFound(format!(
                    "notification:{}",
                    cmd.notification_id.as_uuid()
                ))
            })?;
        if n.tenant_id != cmd.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                n.tenant_id,
                cmd.tenant_id,
            ));
        }
        // INV-N-03:仅本人可标已读
        if UserId::from(actor.user_id) != n.user_id {
            return Err(NotificationError::PermissionDenied);
        }
        if n.status.is_terminal() {
            return Err(NotificationError::InvalidState(format!(
                "already terminal: {}",
                n.status.as_str()
            )));
        }
        n.status = NotificationStatus::Read;
        n.read_at = Some(Utc::now());
        self.repo.update_notification(n.clone()).await?;
        self.notifications.write().unwrap().insert(n.id, n.clone());
        Ok(n)
    }
}

#[async_trait]
impl NotificationQueryPort for InMemoryNotificationService {
    async fn get(
        &self,
        q: GetNotificationQuery,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let n = self
            .notifications
            .read()
            .unwrap()
            .get(&q.notification_id)
            .cloned()
            .ok_or_else(|| {
                NotificationError::NotFound(format!("notification:{}", q.notification_id.as_uuid()))
            })?;
        if n.tenant_id != q.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                n.tenant_id,
                q.tenant_id,
            ));
        }
        // 仅本人可读
        if UserId::from(actor.user_id) != n.user_id {
            return Err(NotificationError::PermissionDenied);
        }
        Ok(n)
    }

    async fn list_by_user(
        &self,
        q: ListByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Notification>, NotificationError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(NotificationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        // INV-N-03:仅本人
        if UserId::from(actor.user_id) != q.user_id {
            return Err(NotificationError::PermissionDenied);
        }
        let notes = self.notifications.read().unwrap();
        Ok(notes
            .values()
            .filter(|n| n.tenant_id == q.tenant_id && n.user_id == q.user_id)
            .filter(|n| !q.unread_only || n.read_at.is_none())
            .cloned()
            .collect())
    }
}

// =====================================================================
// InMemoryNotificationRepository
// =====================================================================

pub struct InMemoryNotificationRepository {
    channels: RwLock<HashMap<NotificationChannelId, NotificationChannel>>,
    templates: RwLock<HashMap<NotificationTemplateId, NotificationTemplate>>,
    notifications: RwLock<HashMap<NotificationId, Notification>>,
}

impl InMemoryNotificationRepository {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            templates: RwLock::new(HashMap::new()),
            notifications: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryNotificationRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationRepository for InMemoryNotificationRepository {
    async fn insert_channel(&self, channel: NotificationChannel) -> Result<(), NotificationError> {
        self.channels.write().unwrap().insert(channel.id, channel);
        Ok(())
    }
    async fn get_channel(
        &self,
        id: NotificationChannelId,
    ) -> Result<NotificationChannel, NotificationError> {
        self.channels
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| NotificationError::NotFound(format!("channel:{}", id.as_uuid())))
    }
    async fn update_channel(&self, channel: NotificationChannel) -> Result<(), NotificationError> {
        self.channels.write().unwrap().insert(channel.id, channel);
        Ok(())
    }
    async fn list_channels_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<NotificationChannel>, NotificationError> {
        Ok(self
            .channels
            .read()
            .unwrap()
            .values()
            .filter(|c| c.tenant_id == tenant_id && c.user_id == user_id)
            .cloned()
            .collect())
    }
    async fn insert_template(
        &self,
        template: NotificationTemplate,
    ) -> Result<(), NotificationError> {
        self.templates
            .write()
            .unwrap()
            .insert(template.id, template);
        Ok(())
    }
    async fn get_template(
        &self,
        id: NotificationTemplateId,
    ) -> Result<NotificationTemplate, NotificationError> {
        self.templates
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| NotificationError::NotFound(format!("template:{}", id.as_uuid())))
    }
    async fn upsert_template_by_event(
        &self,
        template: NotificationTemplate,
    ) -> Result<(), NotificationError> {
        self.templates
            .write()
            .unwrap()
            .insert(template.id, template);
        Ok(())
    }
    async fn list_templates_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<NotificationTemplate>, NotificationError> {
        Ok(self
            .templates
            .read()
            .unwrap()
            .values()
            .filter(|t| t.tenant_id == tenant_id && t.project_id == project_id)
            .cloned()
            .collect())
    }
    async fn insert_notification(
        &self,
        notification: Notification,
    ) -> Result<(), NotificationError> {
        self.notifications
            .write()
            .unwrap()
            .insert(notification.id, notification);
        Ok(())
    }
    async fn get_notification(
        &self,
        id: NotificationId,
    ) -> Result<Notification, NotificationError> {
        self.notifications
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| NotificationError::NotFound(format!("notification:{}", id.as_uuid())))
    }
    async fn update_notification(
        &self,
        notification: Notification,
    ) -> Result<(), NotificationError> {
        self.notifications
            .write()
            .unwrap()
            .insert(notification.id, notification);
        Ok(())
    }
    async fn list_notifications_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        unread_only: bool,
    ) -> Result<Vec<Notification>, NotificationError> {
        Ok(self
            .notifications
            .read()
            .unwrap()
            .values()
            .filter(|n| n.tenant_id == tenant_id && n.user_id == user_id)
            .filter(|n| !unread_only || n.read_at.is_none())
            .cloned()
            .collect())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(tenant_id: TenantId, user_id: UserId) -> ActorContext {
        ActorContext::new(user_id.0, tenant_id.0).with_role("project_admin")
    }

    #[test]
    fn event_breakthrough_invn07() {
        // INV-N-07 突破抑制
        assert!(NotificationEventType::ValidationFailed.is_breakthrough());
        assert!(NotificationEventType::FeedbackCreated.is_breakthrough());
        assert!(NotificationEventType::FeedbackRequired.is_breakthrough());
        assert!(NotificationEventType::AgentSessionFailed.is_breakthrough());
        assert!(NotificationEventType::AgentSessionCrashed.is_breakthrough());
        assert!(NotificationEventType::AgentSessionTimeout.is_breakthrough());
        assert!(NotificationEventType::ProtectedActionDenied.is_breakthrough());
    }

    #[test]
    fn event_suppressed_invn07() {
        // INV-N-07 默认抑制
        assert!(NotificationEventType::AgentStepStarted.is_suppressed());
        assert!(NotificationEventType::AgentStepCompleted.is_suppressed());
        assert!(NotificationEventType::ToolInvoked.is_suppressed());
        assert!(NotificationEventType::ToolCompleted.is_suppressed());
        assert!(NotificationEventType::ValidationPassed.is_suppressed());
        assert!(NotificationEventType::WorkItemCreated.is_suppressed());
        assert!(NotificationEventType::CommentAdded.is_suppressed());
    }

    #[test]
    fn event_as_str() {
        assert_eq!(
            NotificationEventType::ValidationFailed.as_str(),
            "validation.failed"
        );
        assert_eq!(
            NotificationEventType::AgentStepStarted.as_str(),
            "agent.step.started"
        );
    }

    #[test]
    fn notification_status_terminal() {
        assert!(NotificationStatus::Delivered.is_terminal());
        assert!(NotificationStatus::Read.is_terminal());
        assert!(NotificationStatus::DeadLettered.is_terminal());
        assert!(!NotificationStatus::Pending.is_terminal());
        assert!(!NotificationStatus::Sent.is_terminal());
        assert!(!NotificationStatus::Failed.is_terminal());
    }

    #[tokio::test]
    async fn register_channel_self_only() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let other = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id, me);
        let res = svc
            .register_channel(
                RegisterChannelCommand {
                    tenant_id,
                    user_id: other, // 试图给别人注册
                    kind: ChannelKind::Email,
                    address: "x@y.com".to_string(),
                    actor_user_id: me,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(NotificationError::PermissionDenied)));
    }

    #[tokio::test]
    async fn register_channel_self_ok() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let ch = svc
            .register_channel(
                RegisterChannelCommand {
                    tenant_id,
                    user_id: me,
                    kind: ChannelKind::Email,
                    address: "me@x.com".to_string(),
                    actor_user_id: me,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(ch.user_id, me);
        assert!(ch.enabled);
    }

    #[tokio::test]
    async fn dispatch_breakthrough_succeeds() {
        // INV-N-07 关键事件:必须发送
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::ValidationFailed,
                    resource_type: "validation".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "Validation failed".to_string(),
                    body: "see details".to_string(),
                    source: "domain-validation".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(n.status, NotificationStatus::Sent);
        assert!(n.sent_at.is_some());
    }

    #[tokio::test]
    async fn dispatch_suppressed_event_rejected_invn07() {
        // INV-N-07 默认抑制:中间步骤不通知
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let res = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::AgentStepStarted,
                    resource_type: "agent_step".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "step started".to_string(),
                    body: "...".to_string(),
                    source: "domain-agent".to_string(),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(NotificationError::EventSuppressed(_))));
    }

    #[tokio::test]
    async fn dispatch_feedback_required_breakthrough() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::FeedbackRequired,
                    resource_type: "feedback".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "需要你的反馈".to_string(),
                    body: "请查看 agent 进展".to_string(),
                    source: "domain-agent".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(n.event_type, NotificationEventType::FeedbackRequired);
    }

    #[tokio::test]
    async fn dispatch_agent_session_failed_breakthrough() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::AgentSessionFailed,
                    resource_type: "agent_session".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "Agent failed".to_string(),
                    body: "see logs".to_string(),
                    source: "domain-agent".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(n.event_type, NotificationEventType::AgentSessionFailed);
    }

    #[tokio::test]
    async fn mark_read_self_only() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::ValidationFailed,
                    resource_type: "validation".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "x".to_string(),
                    body: "y".to_string(),
                    source: "test".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        let n = svc
            .mark_read(
                MarkReadCommand {
                    tenant_id,
                    notification_id: n.id,
                    actor_user_id: me,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(n.status, NotificationStatus::Read);
        assert!(n.read_at.is_some());
    }

    #[tokio::test]
    async fn cross_tenant_dispatch_denied() {
        let svc = InMemoryNotificationService::new();
        let me = UserId(uuid::Uuid::new_v4());
        let actor_t = uuid::Uuid::new_v4();
        let cmd_t = uuid::Uuid::new_v4();
        let actor = make_actor(actor_t, me);
        let res = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: cmd_t,
                    user_id: me,
                    event_type: NotificationEventType::ValidationFailed,
                    resource_type: "x".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "x".to_string(),
                    body: "y".to_string(),
                    source: "test".to_string(),
                },
                &actor,
            )
            .await;
        assert!(matches!(
            res,
            Err(NotificationError::CrossTenantDenied(_, _))
        ));
    }

    #[tokio::test]
    async fn upsert_template_creates_then_updates() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let project = ProjectId::new();
        let t1 = svc
            .upsert_template(
                UpsertTemplateCommand {
                    tenant_id,
                    project_id: project,
                    event_type: NotificationEventType::ValidationFailed,
                    channel_kinds: vec![ChannelKind::Email, ChannelKind::InApp],
                    subject: "v1 subject".to_string(),
                    body_template: "v1 body".to_string(),
                    actor_user_id: me,
                },
                &actor,
            )
            .await
            .unwrap();
        let t2 = svc
            .upsert_template(
                UpsertTemplateCommand {
                    tenant_id,
                    project_id: project,
                    event_type: NotificationEventType::ValidationFailed,
                    channel_kinds: vec![ChannelKind::Email],
                    subject: "v2 subject".to_string(),
                    body_template: "v2 body".to_string(),
                    actor_user_id: me,
                },
                &actor,
            )
            .await
            .unwrap();
        // 同 (tenant, project, event_type) upsert 应该更新而非新建
        assert_eq!(t1.id, t2.id);
        assert_eq!(t2.subject, "v2 subject");
    }

    #[tokio::test]
    async fn list_unread_filter() {
        let svc = InMemoryNotificationService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = UserId(uuid::Uuid::new_v4());
        let actor = make_actor(tenant_id, me);
        let n1 = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::ValidationFailed,
                    resource_type: "x".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "a".to_string(),
                    body: "b".to_string(),
                    source: "test".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        let _n2 = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id,
                    user_id: me,
                    event_type: NotificationEventType::AgentSessionFailed,
                    resource_type: "x".to_string(),
                    resource_id: Uuid::new_v4(),
                    subject: "c".to_string(),
                    body: "d".to_string(),
                    source: "test".to_string(),
                },
                &actor,
            )
            .await
            .unwrap();
        // 标 n1 已读
        svc.mark_read(
            MarkReadCommand {
                tenant_id,
                notification_id: n1.id,
                actor_user_id: me,
            },
            &actor,
        )
        .await
        .unwrap();
        let all = svc
            .list_by_user(
                ListByUserQuery {
                    tenant_id,
                    user_id: me,
                    unread_only: false,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(all.len(), 2);
        let unread = svc
            .list_by_user(
                ListByUserQuery {
                    tenant_id,
                    user_id: me,
                    unread_only: true,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(unread.len(), 1);
    }
}

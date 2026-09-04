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
    /// Validation 失败,必须通知
    ValidationFailed,
    /// 新增 Feedback,必须通知
    FeedbackCreated,
    /// 需要人工 Feedback 介入,必须通知
    FeedbackRequired,
    /// Agent Session 执行失败,必须通知
    AgentSessionFailed,
    /// Agent Session 崩溃,必须通知
    AgentSessionCrashed,
    /// Agent Session 超时,必须通知
    AgentSessionTimeout,
    /// Protected Action 被拒绝(越权),必须通知
    ProtectedActionDenied,
    // 抑制 - 默认不发
    /// Agent 执行步骤开始,默认抑制
    AgentStepStarted,
    /// Agent 执行步骤完成,默认抑制
    AgentStepCompleted,
    /// 工具被调用,默认抑制
    ToolInvoked,
    /// 工具调用完成,默认抑制
    ToolCompleted,
    /// Validation 通过,默认抑制
    ValidationPassed,
    /// 创建 WorkItem,默认抑制
    WorkItemCreated,
    /// 更新 WorkItem,默认抑制
    WorkItemUpdated,
    /// 新增评论,默认抑制
    CommentAdded,
    // 用户可显式订阅
    /// 自定义事件,由用户显式订阅
    Custom,
}

impl NotificationEventType {
    /// 返回事件类型对应的字符串标识
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
/// 生成基于 UUID 的强类型 ID 类型的宏,自动实现 new/as_uuid/From<Uuid>/Display
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 领域强类型 ID(由宏统一生成)
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成新的随机 ID(由宏统一生成)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回内部 UUID 值(由宏统一生成)
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
    /// 渠道 ID
    pub id: NotificationChannelId,
    /// 所属租户 ID(INV-N-01)
    pub tenant_id: TenantId,
    /// 所属用户 ID
    pub user_id: UserId,
    /// 渠道类型(Email/InApp/Slack/DingTalk)
    pub kind: ChannelKind,
    /// 渠道地址(如邮箱地址、Webhook URL)
    pub address: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 通知渠道类型
pub enum ChannelKind {
    /// 邮件渠道
    Email,
    /// 应用内通知渠道
    InApp,
    /// Slack 渠道(V1)
    Slack,
    /// 钉钉渠道(V1)
    DingTalk,
}

impl ChannelKind {
    /// 返回渠道类型对应的字符串标识
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
    /// 模板 ID
    pub id: NotificationTemplateId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属项目 ID(模板为 Project 范围,INV-N-04)
    pub project_id: ProjectId,
    /// 关联的通知事件类型
    pub event_type: NotificationEventType,
    /// 适用的渠道类型列表
    pub channel_kinds: Vec<ChannelKind>,
    /// 通知主题模板
    pub subject: String,
    /// 通知正文模板
    pub body_template: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
}

/// Notification(Append-only + 状态字段,§4.17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// 通知 ID
    pub id: NotificationId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 接收用户 ID
    pub user_id: UserId,
    /// 触发该通知的事件类型
    pub event_type: NotificationEventType,
    /// 关联资源类型
    pub resource_type: String,
    /// 关联资源 ID
    pub resource_id: Uuid,
    /// 发送所用的渠道 ID
    pub channel_id: NotificationChannelId,
    /// 通知主题
    pub subject: String,
    /// 通知正文
    pub body: String,
    /// 通知当前状态
    pub status: NotificationStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 发送时间(未发送为 None)
    pub sent_at: Option<DateTime<Utc>>,
    /// 已读时间(未读为 None)
    pub read_at: Option<DateTime<Utc>>,
    /// 已重试次数(超限进 DLQ,INV-N-06)
    pub retry_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// 通知投递状态
pub enum NotificationStatus {
    /// 待发送
    Pending,
    /// 已发送
    Sent,
    /// 已送达
    Delivered,
    /// 已读
    Read,
    /// 发送失败
    Failed,
    /// 超过最大重试次数,进入死信队列
    DeadLettered,
}

impl NotificationStatus {
    /// 返回状态对应的字符串标识
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
    /// 是否为终态(不再重试或变更)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::Read | Self::DeadLettered)
    }
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
/// 通知领域错误
pub enum NotificationError {
    /// 目标资源未找到
    #[error("not found: {0}")]
    NotFound(String),
    /// 状态不合法,操作被拒绝
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,
    /// 跨租户访问被拒绝(INV-N-01)
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    /// 事件被 INV-N-07 默认抑制策略拦截
    #[error("event suppressed by INV-N-07: {0}")]
    EventSuppressed(String),
    /// 重试次数超限(INV-N-06)
    #[error("max retry exceeded (5), moved to DLQ")]
    MaxRetryExceeded,
    /// 数据冲突
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 注册通知渠道命令
pub struct RegisterChannelCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 渠道归属用户 ID
    pub user_id: UserId,
    /// 渠道类型
    pub kind: ChannelKind,
    /// 渠道地址
    pub address: String,
    /// 执行该命令的操作者用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 新建或更新通知模板命令
pub struct UpsertTemplateCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属项目 ID
    pub project_id: ProjectId,
    /// 模板对应的事件类型
    pub event_type: NotificationEventType,
    /// 适用渠道类型列表
    pub channel_kinds: Vec<ChannelKind>,
    /// 通知主题模板
    pub subject: String,
    /// 通知正文模板
    pub body_template: String,
    /// 执行该命令的操作者用户 ID(需 Project/Tenant Admin,INV-N-04)
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 派发通知命令
pub struct DispatchNotificationCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 接收用户 ID
    pub user_id: UserId,
    /// 触发的事件类型(决定 INV-N-07 抑制策略)
    pub event_type: NotificationEventType,
    /// 关联资源类型
    pub resource_type: String,
    /// 关联资源 ID
    pub resource_id: Uuid,
    /// 通知主题
    pub subject: String,
    /// 通知正文
    pub body: String,
    /// 事件来源(用于审计)
    pub source: String, // 用于审计
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 标记通知已读命令
pub struct MarkReadCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标通知 ID
    pub notification_id: NotificationId,
    /// 执行该命令的操作者用户 ID(仅本人可标记,INV-N-03)
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 查询单条通知
pub struct GetNotificationQuery {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标通知 ID
    pub notification_id: NotificationId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 按用户查询通知列表
pub struct ListByUserQuery {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 目标用户 ID
    pub user_id: UserId,
    /// 是否仅返回未读通知
    pub unread_only: bool,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
/// 通知命令端口(写操作)
pub trait NotificationCommandPort: Send + Sync {
    /// 注册通知渠道
    async fn register_channel(
        &self,
        cmd: RegisterChannelCommand,
        actor: &ActorContext,
    ) -> Result<NotificationChannel, NotificationError>;

    /// 新建或更新通知模板
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

    /// 标记通知为已读
    async fn mark_read(
        &self,
        cmd: MarkReadCommand,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError>;
}

#[async_trait]
/// 通知查询端口(读操作)
pub trait NotificationQueryPort: Send + Sync {
    /// 获取单条通知详情
    async fn get(
        &self,
        q: GetNotificationQuery,
        actor: &ActorContext,
    ) -> Result<Notification, NotificationError>;

    /// 按用户查询通知列表
    async fn list_by_user(
        &self,
        q: ListByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Notification>, NotificationError>;
}

#[async_trait]
/// 通知仓储端口
pub trait NotificationRepository: Send + Sync {
    /// 插入渠道记录
    async fn insert_channel(&self, channel: NotificationChannel) -> Result<(), NotificationError>;
    /// 按 ID 获取渠道
    async fn get_channel(
        &self,
        id: NotificationChannelId,
    ) -> Result<NotificationChannel, NotificationError>;
    /// 更新渠道记录
    async fn update_channel(&self, channel: NotificationChannel) -> Result<(), NotificationError>;
    /// 按用户列出其所有渠道
    async fn list_channels_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<NotificationChannel>, NotificationError>;

    /// 插入模板记录
    async fn insert_template(
        &self,
        template: NotificationTemplate,
    ) -> Result<(), NotificationError>;
    /// 按 ID 获取模板
    async fn get_template(
        &self,
        id: NotificationTemplateId,
    ) -> Result<NotificationTemplate, NotificationError>;
    /// 按事件类型新建或更新模板
    async fn upsert_template_by_event(
        &self,
        template: NotificationTemplate,
    ) -> Result<(), NotificationError>;
    /// 按项目列出所有模板
    async fn list_templates_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<NotificationTemplate>, NotificationError>;

    /// 插入通知记录
    async fn insert_notification(
        &self,
        notification: Notification,
    ) -> Result<(), NotificationError>;
    /// 按 ID 获取通知
    async fn get_notification(&self, id: NotificationId)
        -> Result<Notification, NotificationError>;
    /// 更新通知记录
    async fn update_notification(
        &self,
        notification: Notification,
    ) -> Result<(), NotificationError>;
    /// 按用户列出通知,可选仅未读
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

/// 基于内存的通知领域服务实现(用于测试/MVP)
pub struct InMemoryNotificationService {
    repo: Arc<dyn NotificationRepository>,
    channels: Arc<RwLock<HashMap<NotificationChannelId, NotificationChannel>>>,
    templates: Arc<RwLock<HashMap<NotificationTemplateId, NotificationTemplate>>>,
    notifications: Arc<RwLock<HashMap<NotificationId, Notification>>>,
}

impl InMemoryNotificationService {
    /// 创建默认的内存通知服务(使用内置的内存仓储)
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryNotificationRepository::new()),
            channels: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    /// 使用指定的仓储实现创建内存通知服务
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

/// 基于内存的通知仓储实现(用于测试/MVP)
pub struct InMemoryNotificationRepository {
    channels: RwLock<HashMap<NotificationChannelId, NotificationChannel>>,
    templates: RwLock<HashMap<NotificationTemplateId, NotificationTemplate>>,
    notifications: RwLock<HashMap<NotificationId, Notification>>,
}

impl InMemoryNotificationRepository {
    /// 创建空的内存通知仓储
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
        let actor = make_actor(TenantId(tenant_id), me);
        let res = svc
            .register_channel(
                RegisterChannelCommand {
                    tenant_id: TenantId(tenant_id),
                    user_id: UserId(other), // 试图给别人注册
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
        let actor = make_actor(TenantId(tenant_id), me);
        let ch = svc
            .register_channel(
                RegisterChannelCommand {
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let res = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let n = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(tenant_id),
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
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(actor_t), me);
        let res = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(cmd_t),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let project = ProjectId::new();
        let t1 = svc
            .upsert_template(
                UpsertTemplateCommand {
                    tenant_id: TenantId(tenant_id),
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
                    tenant_id: TenantId(tenant_id),
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
        let actor = make_actor(TenantId(tenant_id), me);
        let n1 = svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: TenantId(tenant_id),
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
                    tenant_id: TenantId(tenant_id),
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
                tenant_id: TenantId(tenant_id),
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
                    tenant_id: TenantId(tenant_id),
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
                    tenant_id: TenantId(tenant_id),
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

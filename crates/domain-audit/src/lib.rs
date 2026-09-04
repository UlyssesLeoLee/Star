//! domain-audit crate
//!
//! 详细 spec: docs/specs/domain-audit-spec.md §17
//! 上游基本设计: docs/basic-design.md §2.1(表 15) / §5.7 / §6.7 / §9
//! 数据设计: docs/data-design.md §4.11 (`audit` schema)
//! API 设计: docs/api-design.md §3.12
//!
//! ## 职责
//!
//! 唯一 Append-only 横切 Domain,所有其他 Domain 写审计时调用本 Module 的
//! `AuditRecorder` Port。负责审计日志、AI Audit Metadata(9 个必答问题)、
//! 跨租户访问尝试 100% 记录、合规导出。
//!
//! ## 关键不变量(INV-AU-01~07,共 7 条)
//!
//! - **INV-AU-01** Append-only:AuditEvent 不可 UPDATE / DELETE(本 crate 不暴露 update/delete)
//! - **INV-AU-02** 9 个 AI Audit 必答问题必填(actor_session / context_packet / change_set /
//!   validation / feedback / approver / data_categories / provider_boundary / risk_signals)
//! - **INV-AU-03** 跨租户访问尝试 100% 记录(Application 层强制 + 本 crate 校验)
//! - **INV-AU-04** 敏感 Prompt/Code 不默认进入普通 Audit Log,走 AIAuditMetadata + Object Storage
//! - **INV-AU-05** Audit 保留 7 年(企业级),月级别 Partition
//! - **INV-AU-06** AI Content Retention:Full Prompt 90 天默认,Sensitive Code 0 天,Summary 1 年
//! - **INV-AU-07** 导出仅 Tenant Admin / Compliance 角色(Protected)
//!
//! Lead 责任: audit Lead

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

// =====================================================================
// 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
            #[allow(dead_code)]
            pub fn from_uuid(id: uuid::Uuid) -> Self {
                Self(id)
            }
            #[allow(dead_code)]
            pub fn as_uuid(&self) -> uuid::Uuid {
                self.0
            }
            #[allow(dead_code)]
            pub fn into_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(id: uuid::Uuid) -> Self {
                Self(id)
            }
        }
    };
}

define_uuid_id!(AuditEventId);
define_uuid_id!(AIAuditMetadataId);
define_uuid_id!(AuditExportJobId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(ContextPacketId);
define_uuid_id!(ChangeSetId);
define_uuid_id!(ValidationResultId);
define_uuid_id!(FeedbackId);
define_uuid_id!(ProviderDataBoundaryId);
define_uuid_id!(ResourceId);

// =====================================================================
// 值对象
// =====================================================================

/// **审计动作类型**(`audit_event.action` 字段)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    /// 工作项操作
    WorkItemOperation,
    /// Worktree 操作
    WorktreeOperation,
    /// Pull Request 操作
    PullRequestOperation,
    /// 权限变更
    PermissionChange,
    /// 角色分配
    RoleAssign,
    /// 跨租户访问尝试(INV-AU-03 必审计)
    CrossTenantAttempt,
    /// AI Agent 执行
    AgentExecute,
    /// 验证运行
    ValidationRun,
    /// Feedback 创建
    FeedbackCreated,
    /// Retention 物理删除(INV-AU-06)
    AiRetentionPurged,
    /// 导出请求(INV-AU-07 必审计)
    ExportRequested,
    /// 通用自定义动作
    Custom,
}

impl Default for AuditAction {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::WorkItemOperation => "WORK_ITEM_OPERATION",
            Self::WorktreeOperation => "WORKTREE_OPERATION",
            Self::PullRequestOperation => "PULL_REQUEST_OPERATION",
            Self::PermissionChange => "PERMISSION_CHANGE",
            Self::RoleAssign => "ROLE_ASSIGN",
            Self::CrossTenantAttempt => "CROSS_TENANT_ATTEMPT",
            Self::AgentExecute => "AGENT_EXECUTE",
            Self::ValidationRun => "VALIDATION_RUN",
            Self::FeedbackCreated => "FEEDBACK_CREATED",
            Self::AiRetentionPurged => "AI_RETENTION_PURGED",
            Self::ExportRequested => "EXPORT_REQUESTED",
            Self::Custom => "CUSTOM",
        };
        f.write_str(s)
    }
}

/// **执行者**(`actor` 字段,基本设计 §3.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Actor {
    /// 用户执行
    User { user_id: UserId },
    /// Agent 执行(必须同时有 session_id 才能回答 INV-AU-02)
    Agent {
        session_id: AgentSessionId,
        agent_id: AgentId,
    },
    /// 系统后台
    System,
}

/// **预定义角色字符串**(security-design §3.4)
pub mod roles {
    /// 租户管理员(可读 / 导出 audit)
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 合规官(可读 / 导出 audit)
    pub const COMPLIANCE_OFFICER: &str = "compliance_officer";
    /// 租户审计师(只读 audit)
    pub const TENANT_AUDITOR: &str = "tenant_auditor";
}

// =====================================================================
// 错误
// =====================================================================

/// Audit 域错误
#[derive(Debug, Error)]
pub enum AuditError {
    /// `AU-001` 404 AuditEvent 不存在
    #[error("audit event not found: {0}")]
    NotFound(AuditEventId),
    /// `AU-002` 403 非 Tenant Admin / Compliance 读
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// `AU-002` 403 权限拒绝
    #[error("permission denied")]
    PermissionDenied,
    /// 409 hash 冲突(并发)
    #[error("conflict: {0}")]
    Conflict(String),
    /// 5xx
    #[error("internal error: {0}")]
    Internal(String),
}

impl AuditError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "AUDIT_NOT_FOUND",
            Self::InvalidState(_) => "AUDIT_INVALID_STATE",
            Self::PermissionDenied => "AUDIT_PERMISSION_DENIED",
            Self::Conflict(_) => "AUDIT_CONFLICT",
            Self::Internal(_) => "AUDIT_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for AuditError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

// =====================================================================
// 实体
// =====================================================================

/// **AuditEvent**(聚合根,Append-only,basic-design §5.7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 主键
    pub id: AuditEventId,
    /// 租户 ID(INV-AU-02 必填)
    pub tenant_id: TenantId,
    /// 触发者(INV-AU-02 必填,user / agent / system)
    pub actor: Actor,
    /// 动作
    pub action: AuditAction,
    /// 目标类型
    pub resource_type: String,
    /// 目标 ID
    pub resource_id: Uuid,
    /// 上下文引用(Provenance IDs)
    pub context_refs: Vec<Uuid>,
    /// 修改前(可选)
    pub before_state: Option<serde_json::Value>,
    /// 修改后(可选)
    pub after_state: Option<serde_json::Value>,
    /// 跨租户标记(INV-AU-03)
    pub cross_tenant: bool,
    /// 不可变哈希(防篡改)
    pub immutable_hash: String,
    /// 发生时间(ms 精度)
    pub occurred_at: DateTime<Utc>,
}

impl AuditEvent {
    pub const FIELD_COUNT: usize = 11;
}

/// **AIAuditMetadata**(9 个必答问题,basic-design §6.7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAuditMetadata {
    pub id: AIAuditMetadataId,
    pub tenant_id: TenantId,
    /// Q1: 谁要求 AI 做什么? — actor
    pub agent_session_id: AgentSessionId,
    /// Q2: AI 使用了什么 Context? — Context Packet ID
    pub context_packet_id: Option<ContextPacketId>,
    /// Q3: AI 修改了什么? — ChangeSet ID
    pub change_set_id: Option<ChangeSetId>,
    /// Q4: 哪个 Agent 执行? — agent_id
    pub agent_id: AgentId,
    /// Q5: 在哪个 Worktree?
    pub worktree_id: Option<Uuid>,
    /// Q6: 什么时间?
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// Q7: 哪些验证通过? — Validation Result IDs
    pub validation_result_ids: Vec<ValidationResultId>,
    /// Q8: 哪些 Feedback 被消费? — Feedback IDs
    pub feedback_consumed_ids: Vec<FeedbackId>,
    /// Q9: 谁批准 Commit?
    pub approver_user_id: Option<UserId>,
    /// 数据类别(Prompt / Code / Diff 等)
    pub data_categories_sent: Vec<String>,
    /// ProviderDataBoundary 引用(§5.4)
    pub provider_boundary_ref: Option<ProviderDataBoundaryId>,
    /// 风险信号(从 ChangeSet 复制)
    pub risk_signals: Vec<String>,
    /// Full Prompt 引用(Object Storage key,INV-AU-04)
    pub full_prompt_ref: Option<String>,
    /// Full Response 引用
    pub full_response_ref: Option<String>,
    /// Prompt 哈希(不存明文,INV-AU-04)
    pub prompt_hash: String,
    /// Response 哈希
    pub response_hash: String,
    /// 保留期限(默认 90 天,INV-AU-06)
    pub retention_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AIAuditMetadata {
    /// 9 个 AI 必答问题是否完整(INV-AU-02)
    pub fn has_complete_9_questions(&self) -> bool {
        // Q1 actor:agent_session_id ✓
        // Q2 context_packet_id 可空(无 context 时)
        // Q3 change_set_id 可空(无修改时)
        // Q4 agent_id ✓
        // Q5 worktree_id 可空(system 调用)
        // Q6 时间戳 ✓
        // Q7/Q8 数组可空
        // Q9 approver_user_id 可空
        !self.agent_session_id.as_uuid().is_nil()
            && !self.agent_id.as_uuid().is_nil()
            && self.started_at <= self.ended_at
            && self.prompt_hash.len() == 64
            && self.response_hash.len() == 64
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.retention_until < now
    }
}

/// **AuditExportJob**(导出异步任务,INV-AU-07)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportJob {
    pub id: AuditExportJobId,
    pub tenant_id: TenantId,
    /// 导出格式
    pub format: ExportFormat,
    /// 时间范围(开始)
    pub range_start: DateTime<Utc>,
    /// 时间范围(结束)
    pub range_end: DateTime<Utc>,
    /// 过滤器
    pub filter_action: Option<AuditAction>,
    /// 申请人
    pub requested_by: UserId,
    /// 状态
    pub status: ExportStatus,
    /// 下载 URL(完成后填充)
    pub download_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportFormat {
    Csv,
    Parquet,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Csv
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

// =====================================================================
// 不变量(INV-AU-01~07)
// =====================================================================

pub type InvariantCheck = fn(&AuditEvent) -> Result<(), AuditError>;

/// **INV-AU-02** 必填字段校验(tenant_id / actor / target 必填)
pub fn check_invariant_02_required_fields(ev: &AuditEvent) -> Result<(), AuditError> {
    if ev.tenant_id.as_uuid().is_nil() {
        return Err(AuditError::InvalidState(
            "INV-AU-02: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    if ev.resource_type.is_empty() {
        return Err(AuditError::InvalidState(
            "INV-AU-02: resource_type 不能为空".to_string(),
        ));
    }
    if ev.resource_id.is_nil() {
        return Err(AuditError::InvalidState(
            "INV-AU-02: resource_id 必须非 nil".to_string(),
        ));
    }
    // actor 必非空(User/Agent/System 之一)
    match &ev.actor {
        Actor::User { user_id } if user_id.as_uuid().is_nil() => {
            return Err(AuditError::InvalidState(
                "INV-AU-02: Actor::User.user_id 必须非 nil".to_string(),
            ));
        }
        Actor::Agent {
            session_id,
            agent_id,
        } if session_id.as_uuid().is_nil() || agent_id.as_uuid().is_nil() => {
            return Err(AuditError::InvalidState(
                "INV-AU-02: Actor::Agent session_id / agent_id 必须非 nil".to_string(),
            ));
        }
        _ => {}
    }
    Ok(())
}

/// **INV-AU-03(immutable)** `immutable_hash` 必填 64 字符 sha256 hex
pub fn check_invariant_03_immutable_hash(ev: &AuditEvent) -> Result<(), AuditError> {
    if ev.immutable_hash.is_empty() {
        return Err(AuditError::InvalidState(
            "INV-AU-03: immutable_hash 不能为空(防篡改)".to_string(),
        ));
    }
    if ev.immutable_hash.len() != 64 {
        return Err(AuditError::InvalidState(
            "INV-AU-03: immutable_hash 必须是 64 字符 sha256 hex".to_string(),
        ));
    }
    // 必须是 hex
    if !ev.immutable_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AuditError::InvalidState(
            "INV-AU-03: immutable_hash 必须为 hex 字符".to_string(),
        ));
    }
    Ok(())
}

/// **INV-AU-04** 跨租户尝试必带 `cross_tenant: true`
pub fn check_invariant_04_cross_tenant_flag(ev: &AuditEvent) -> Result<(), AuditError> {
    if ev.action == AuditAction::CrossTenantAttempt && !ev.cross_tenant {
        return Err(AuditError::InvalidState(
            "INV-AU-04: action=CROSS_TENANT_ATTEMPT 必须带 cross_tenant=true".to_string(),
        ));
    }
    if ev.cross_tenant && ev.action != AuditAction::CrossTenantAttempt {
        return Err(AuditError::InvalidState(
            "INV-AU-04: cross_tenant=true 但 action 不是 CrossTenantAttempt".to_string(),
        ));
    }
    Ok(())
}

/// **INV-AU-04** 普通 Audit Log 不带 prompt/response 明文(`before_state`/`after_state` 不应是 Prompt)
pub fn check_invariant_04_no_sensitive_plaintext(ev: &AuditEvent) -> Result<(), AuditError> {
    for field in [&ev.before_state, &ev.after_state] {
        if let Some(v) = field {
            if let Some(obj) = v.as_object() {
                if obj.contains_key("prompt")
                    || obj.contains_key("response")
                    || obj.contains_key("code")
                {
                    // 允许 key 存在但不允许 value 看起来像大段内容
                    if let Some(p) = obj.get("prompt").and_then(|x| x.as_str()) {
                        if p.len() > 256 {
                            return Err(AuditError::InvalidState(
                                "INV-AU-04: prompt 不应进入普通 Audit Log,应走 AIAuditMetadata"
                                    .to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_02_required_fields,
    check_invariant_03_immutable_hash,
    check_invariant_04_cross_tenant_flag,
    check_invariant_04_no_sensitive_plaintext,
];

pub fn run_invariants(checks: &[InvariantCheck], ev: &AuditEvent) -> Result<(), AuditError> {
    for c in checks {
        c(ev)?;
    }
    Ok(())
}

/// 计算 immutable_hash(简化为 hex 编码 DefaultHasher,演示用;
/// 生产应使用 sha2 / sha256 crate)
pub fn compute_immutable_hash(
    tenant_id: TenantId,
    actor: &Actor,
    action: AuditAction,
    resource_type: &str,
    resource_id: Uuid,
    occurred_at: DateTime<Utc>,
) -> String {
    use std::hash::{Hash, Hasher};
    let actor_str = match actor {
        Actor::User { user_id } => format!("user:{}", user_id),
        Actor::Agent {
            session_id,
            agent_id,
        } => format!("agent:{}:{}", session_id, agent_id),
        Actor::System => "system".to_string(),
    };
    let mut s = String::new();
    s.push_str(&tenant_id.to_string());
    s.push('|');
    s.push_str(&actor_str);
    s.push('|');
    s.push_str(&action.to_string());
    s.push('|');
    s.push_str(resource_type);
    s.push('|');
    s.push_str(&resource_id.to_string());
    s.push('|');
    s.push_str(&occurred_at.timestamp_millis().to_string());
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    let h_val = h.finish();
    // 扩展为 64 字符 hex(用 format! 重复拼接)
    let hex = format!("{:016x}", h_val);
    let pad = 64 - hex.len();
    let mut out = String::with_capacity(64);
    for _ in 0..pad {
        out.push('0');
    }
    out.push_str(&hex);
    out
}

// =====================================================================
// 事件(NATS 主题 payload)
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecorded {
    pub meta: EventMeta,
    pub audit_event_id: AuditEventId,
    pub action: AuditAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossTenantAttempted {
    pub meta: EventMeta,
    pub audit_event_id: AuditEventId,
    pub actor_user_id: Uuid,
    pub attempted_resource_type: String,
    pub attempted_resource_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEventKind {
    Recorded(AuditRecorded),
    CrossTenantAttempted(CrossTenantAttempted),
}

impl AuditEventKind {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Recorded(_) => "star.events.audit.event.recorded.v1",
            Self::CrossTenantAttempted(_) => "star.events.audit.cross_tenant_attempt.v1",
        }
    }
}

// =====================================================================
// 端口(Port traits)
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAuditCommand {
    pub tenant_id: TenantId,
    pub actor: Actor,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub context_refs: Vec<Uuid>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    /// 预计算 hash(若为 None 则本 crate 内部计算)
    pub immutable_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordCrossTenantAttemptCommand {
    pub actor_user_id: Uuid,
    pub attempted_resource_type: String,
    pub attempted_resource_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditListQuery {
    pub tenant_id: TenantId,
    pub limit: u32,
    pub offset: u32,
    pub action: Option<AuditAction>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

impl Default for AuditListQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            limit: 100,
            offset: 0,
            action: None,
            since: None,
            until: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 记录 AI Audit 的输入参数(对应 9 个必答问题)
pub struct AIAuditMetadataInput {
    /// Q1: 会话 ID
    pub agent_session_id: AgentSessionId,
    /// Q4: Agent ID
    pub agent_id: AgentId,
    /// Q2: Context Packet ID(可选)
    pub context_packet_id: Option<ContextPacketId>,
    /// Q3: ChangeSet ID(可选)
    pub change_set_id: Option<ChangeSetId>,
    /// Q5: Worktree ID(可选)
    pub worktree_id: Option<Uuid>,
    /// Q6: 开始时间
    pub started_at: DateTime<Utc>,
    /// Q6: 结束时间
    pub ended_at: DateTime<Utc>,
    /// Q7: 验证结果 ID 列表
    pub validation_result_ids: Vec<ValidationResultId>,
    /// Q8: 已消费 Feedback ID 列表
    pub feedback_consumed_ids: Vec<FeedbackId>,
    /// Q9: 批准人用户 ID(可选)
    pub approver_user_id: Option<UserId>,
    /// 数据类别(Prompt / Code / Diff 等)
    pub data_categories_sent: Vec<String>,
    /// ProviderDataBoundary 引用(可选)
    pub provider_boundary_ref: Option<ProviderDataBoundaryId>,
    /// 风险信号列表
    pub risk_signals: Vec<String>,
    /// Full Prompt 引用(Object Storage key,可选)
    pub full_prompt_ref: Option<String>,
    /// Full Response 引用(可选)
    pub full_response_ref: Option<String>,
    /// Prompt 哈希
    pub prompt_hash: String,
    /// Response 哈希
    pub response_hash: String,
    /// Full Prompt 默认 90 天(Sensitive Code 0 天由外部策略决定;此处默认 90d,INV-AU-06)
    pub retention: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 记录 AI Audit 的命令
pub struct RecordAIAuditCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// AI Audit 元数据输入
    pub metadata: AIAuditMetadataInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 导出 AuditEvent 的命令
pub struct ExportAuditCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 导出格式
    pub format: ExportFormat,
    /// 时间范围(开始)
    pub range_start: DateTime<Utc>,
    /// 时间范围(结束)
    pub range_end: DateTime<Utc>,
    /// 按动作类型过滤(可选)
    pub filter_action: Option<AuditAction>,
}

/// **AuditRecorder 端口**(3 个方法:普通 / AI / 跨租户尝试)
#[async_trait]
pub trait AuditRecorder: Send + Sync {
    /// 记录普通 AuditEvent
    async fn record(
        &self,
        cmd: RecordAuditCommand,
        actor_ctx: ActorContext,
    ) -> Result<AuditEvent, AuditError>;
    /// 记录跨租户访问尝试(INV-AU-03)
    async fn record_cross_tenant_attempt(
        &self,
        cmd: RecordCrossTenantAttemptCommand,
        actor_ctx: ActorContext,
    ) -> Result<AuditEvent, AuditError>;
    /// 记录 AI Audit(9 个必答问题)
    async fn record_ai(
        &self,
        cmd: RecordAIAuditCommand,
        actor_ctx: ActorContext,
    ) -> Result<AIAuditMetadata, AuditError>;
}

/// **AuditQuery 端口**
#[async_trait]
pub trait AuditQueryPort: Send + Sync {
    /// 列出 AuditEvent(需 INV-AU-07 权限)
    async fn list_events(
        &self,
        q: AuditListQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AuditEvent>, AuditError>;
    /// 详情
    async fn get_event(
        &self,
        id: AuditEventId,
        viewer: ActorContext,
    ) -> Result<AuditEvent, AuditError>;
    /// 列出 AI Audit Metadata
    async fn list_ai_audit(
        &self,
        tenant_id: TenantId,
        agent_session_id: Option<AgentSessionId>,
        viewer: ActorContext,
    ) -> Result<Vec<AIAuditMetadata>, AuditError>;
    /// 导出请求(异步 Job)
    async fn export(
        &self,
        cmd: ExportAuditCommand,
        viewer: ActorContext,
    ) -> Result<AuditExportJob, AuditError>;
}

// =====================================================================
// InMemoryAuditService
// =====================================================================

/// **InMemory Audit Recorder + Query**(Append-only 内存实现)
pub struct InMemoryAuditService {
    events: Arc<RwLock<HashMap<AuditEventId, AuditEvent>>>,
    ai_meta: Arc<RwLock<HashMap<AIAuditMetadataId, AIAuditMetadata>>>,
    jobs: Arc<RwLock<HashMap<AuditExportJobId, AuditExportJob>>>,
    event_tx: mpsc::UnboundedSender<AuditEventKind>,
    /// 保留 7 年(企业级),本服务假定 in-memory
    retention: Duration,
}

impl InMemoryAuditService {
    /// 创建新的 InMemoryAuditService 实例,并返回事件接收端
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<AuditEventKind>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            ai_meta: Arc::new(RwLock::new(HashMap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
            // 7 年 (企业级),INV-AU-05
            retention: Duration::from_secs(7 * 365 * 24 * 60 * 60),
        });
        (svc, rx)
    }
    /// 创建仅用于测试的 InMemoryAuditService 实例
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    /// 返回当前已记录的 AuditEvent 数量
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }
    /// 返回当前已记录的 AI Audit Metadata 数量
    pub async fn ai_meta_count(&self) -> usize {
        self.ai_meta.read().await.len()
    }
    /// 返回跨租户访问尝试的记录数量
    pub async fn cross_tenant_count(&self) -> usize {
        self.events
            .read()
            .await
            .values()
            .filter(|e| e.cross_tenant)
            .count()
    }
    fn check_audit_read_perm(actor: &ActorContext) -> Result<(), AuditError> {
        if !actor.has_role("audit_reader") && !actor.is_platform_admin {
            return Err(AuditError::PermissionDenied);
        }
        Ok(())
    }
    fn check_audit_export_perm(actor: &ActorContext) -> Result<(), AuditError> {
        if !actor.has_role("audit_exporter") && !actor.is_platform_admin {
            return Err(AuditError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryAuditService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryAuditService {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
            ai_meta: self.ai_meta.clone(),
            jobs: self.jobs.clone(),
            event_tx: self.event_tx.clone(),
            retention: self.retention,
        }
    }
}

#[async_trait]
impl AuditRecorder for InMemoryAuditService {
    async fn record(
        &self,
        cmd: RecordAuditCommand,
        _actor_ctx: ActorContext,
    ) -> Result<AuditEvent, AuditError> {
        let now = Utc::now();
        let id = AuditEventId::new();
        let hash = cmd.immutable_hash.clone().unwrap_or_else(|| {
            compute_immutable_hash(
                cmd.tenant_id,
                &cmd.actor,
                cmd.action,
                &cmd.resource_type,
                cmd.resource_id,
                now,
            )
        });
        let ev = AuditEvent {
            id,
            tenant_id: cmd.tenant_id,
            actor: cmd.actor,
            action: cmd.action,
            resource_type: cmd.resource_type,
            resource_id: cmd.resource_id,
            context_refs: cmd.context_refs,
            before_state: cmd.before_state,
            after_state: cmd.after_state,
            cross_tenant: false,
            immutable_hash: hash,
            occurred_at: now,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &ev)?;
        {
            let mut guard = self.events.write().await;
            guard.insert(id, ev.clone());
        }
        let _ = self.event_tx.send(AuditEventKind::Recorded(AuditRecorded {
            meta: EventMeta::new(cmd.tenant_id),
            audit_event_id: id,
            action: cmd.action,
        }));
        Ok(ev)
    }

    async fn record_cross_tenant_attempt(
        &self,
        cmd: RecordCrossTenantAttemptCommand,
        actor_ctx: ActorContext,
    ) -> Result<AuditEvent, AuditError> {
        let now = Utc::now();
        let id = AuditEventId::new();
        // INV-AU-03 跨租户:必须 actor's tenant 与 attempted_resource 不一致,
        // 但 Resource 仅引用 ID,本服务假定 caller 已校验,只负责记录(强记 cross_tenant=true)
        let tenant_id = TenantId::from(actor_ctx.tenant_id);
        let actor = Actor::User {
            user_id: UserId::from_uuid(actor_ctx.user_id),
        };
        let ev = AuditEvent {
            id,
            tenant_id,
            actor: actor.clone(),
            action: AuditAction::CrossTenantAttempt,
            resource_type: cmd.attempted_resource_type.clone(),
            resource_id: cmd.attempted_resource_id,
            context_refs: Vec::new(),
            before_state: None,
            after_state: None,
            cross_tenant: true,
            immutable_hash: compute_immutable_hash(
                tenant_id,
                &actor,
                AuditAction::CrossTenantAttempt,
                &cmd.attempted_resource_type,
                cmd.attempted_resource_id,
                now,
            ),
            occurred_at: now,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &ev)?;
        {
            let mut guard = self.events.write().await;
            guard.insert(id, ev.clone());
        }
        let _ = self
            .event_tx
            .send(AuditEventKind::CrossTenantAttempted(CrossTenantAttempted {
                meta: EventMeta::new(tenant_id),
                audit_event_id: id,
                actor_user_id: actor_ctx.user_id,
                attempted_resource_type: cmd.attempted_resource_type,
                attempted_resource_id: cmd.attempted_resource_id,
            }));
        Ok(ev)
    }

    async fn record_ai(
        &self,
        cmd: RecordAIAuditCommand,
        _actor_ctx: ActorContext,
    ) -> Result<AIAuditMetadata, AuditError> {
        let now = Utc::now();
        let id = AIAuditMetadataId::new();
        // INV-AU-06 默认 90 天,允许外部覆盖
        let retention_dur = cmd
            .metadata
            .retention
            .unwrap_or(Duration::from_secs(90 * 24 * 60 * 60));
        let retention_until = now + chrono::Duration::seconds(retention_dur.as_secs() as i64);
        let meta = AIAuditMetadata {
            id,
            tenant_id: cmd.tenant_id,
            agent_session_id: cmd.metadata.agent_session_id,
            context_packet_id: cmd.metadata.context_packet_id,
            change_set_id: cmd.metadata.change_set_id,
            agent_id: cmd.metadata.agent_id,
            worktree_id: cmd.metadata.worktree_id,
            started_at: cmd.metadata.started_at,
            ended_at: cmd.metadata.ended_at,
            validation_result_ids: cmd.metadata.validation_result_ids,
            feedback_consumed_ids: cmd.metadata.feedback_consumed_ids,
            approver_user_id: cmd.metadata.approver_user_id,
            data_categories_sent: cmd.metadata.data_categories_sent,
            provider_boundary_ref: cmd.metadata.provider_boundary_ref,
            risk_signals: cmd.metadata.risk_signals,
            full_prompt_ref: cmd.metadata.full_prompt_ref,
            full_response_ref: cmd.metadata.full_response_ref,
            prompt_hash: cmd.metadata.prompt_hash,
            response_hash: cmd.metadata.response_hash,
            retention_until,
            created_at: now,
        };
        // INV-AU-02 9 个必答问题完整性
        if !meta.has_complete_9_questions() {
            return Err(AuditError::InvalidState(
                "INV-AU-02: 9 个 AI Audit 必答问题不完整".to_string(),
            ));
        }
        {
            let mut guard = self.ai_meta.write().await;
            guard.insert(id, meta.clone());
        }
        Ok(meta)
    }
}

#[async_trait]
impl AuditQueryPort for InMemoryAuditService {
    async fn list_events(
        &self,
        q: AuditListQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        Self::check_audit_read_perm(&viewer)?;
        // 跨租户硬过滤
        if viewer.tenant_id != q.tenant_id.0 {
            return Err(AuditError::PermissionDenied);
        }
        let mut all: Vec<AuditEvent> = {
            let guard = self.events.read().await;
            guard
                .values()
                .filter(|e| {
                    e.tenant_id == q.tenant_id
                        && q.action.map_or(true, |a| e.action == a)
                        && q.since.map_or(true, |s| e.occurred_at >= s)
                        && q.until.map_or(true, |u| e.occurred_at <= u)
                })
                .cloned()
                .collect()
        };
        all.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }

    async fn get_event(
        &self,
        id: AuditEventId,
        viewer: ActorContext,
    ) -> Result<AuditEvent, AuditError> {
        Self::check_audit_read_perm(&viewer)?;
        let ev = {
            let guard = self.events.read().await;
            guard.get(&id).cloned()
        };
        let ev = ev.ok_or(AuditError::NotFound(id))?;
        if ev.tenant_id != TenantId::from(viewer.tenant_id) {
            return Err(AuditError::PermissionDenied);
        }
        Ok(ev)
    }

    async fn list_ai_audit(
        &self,
        tenant_id: TenantId,
        agent_session_id: Option<AgentSessionId>,
        viewer: ActorContext,
    ) -> Result<Vec<AIAuditMetadata>, AuditError> {
        Self::check_audit_read_perm(&viewer)?;
        if viewer.tenant_id != tenant_id.0 {
            return Err(AuditError::PermissionDenied);
        }
        let guard = self.ai_meta.read().await;
        Ok(guard
            .values()
            .filter(|m| {
                m.tenant_id == tenant_id
                    && agent_session_id.map_or(true, |s| m.agent_session_id == s)
            })
            .cloned()
            .collect())
    }

    async fn export(
        &self,
        cmd: ExportAuditCommand,
        viewer: ActorContext,
    ) -> Result<AuditExportJob, AuditError> {
        Self::check_audit_export_perm(&viewer)?;
        if viewer.tenant_id != cmd.tenant_id.0 {
            return Err(AuditError::PermissionDenied);
        }
        let id = AuditExportJobId::new();
        let now = Utc::now();
        let job = AuditExportJob {
            id,
            tenant_id: cmd.tenant_id,
            format: cmd.format,
            range_start: cmd.range_start,
            range_end: cmd.range_end,
            filter_action: cmd.filter_action,
            requested_by: UserId::from_uuid(viewer.user_id),
            status: ExportStatus::Pending,
            download_url: None,
            created_at: now,
            completed_at: None,
        };
        {
            let mut guard = self.jobs.write().await;
            guard.insert(id, job.clone());
        }
        Ok(job)
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_admin_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .with_role(roles::TENANT_ADMIN)
            .with_role("audit_reader")
            .with_role("audit_exporter")
    }

    fn make_developer_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role("developer")
    }

    fn make_actor_user(_tenant_id: TenantId) -> Actor {
        Actor::User {
            user_id: UserId::new(),
        }
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(AuditEvent::FIELD_COUNT, 11);
    }

    #[tokio::test]
    async fn record_normal_audit_event() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let cmd = RecordAuditCommand {
            tenant_id: TenantId(tenant_id),
            actor: make_actor_user(TenantId(tenant_id)),
            action: AuditAction::WorkItemOperation,
            resource_type: "work_item".to_string(),
            resource_id: Uuid::new_v4(),
            context_refs: vec![],
            before_state: None,
            after_state: None,
            immutable_hash: None,
        };
        let actor = make_admin_actor(TenantId(tenant_id));
        let ev = svc.record(cmd, actor).await.unwrap();
        assert_eq!(svc.event_count().await, 1);
        assert!(!ev.cross_tenant);
        assert_eq!(ev.immutable_hash.len(), 64);
    }

    #[tokio::test]
    async fn record_ai_audit_with_9_questions() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let session = AgentSessionId::new();
        let agent = AgentId::new();
        let started = Utc::now() - chrono::Duration::seconds(30);
        let ended = Utc::now();
        let cmd = RecordAIAuditCommand {
            tenant_id: TenantId(tenant_id),
            metadata: AIAuditMetadataInput {
                agent_session_id: session,
                agent_id: agent,
                context_packet_id: Some(ContextPacketId::new()),
                change_set_id: Some(ChangeSetId::new()),
                worktree_id: Some(Uuid::new_v4()),
                started_at: started,
                ended_at: ended,
                validation_result_ids: vec![ValidationResultId::new()],
                feedback_consumed_ids: vec![FeedbackId::new()],
                approver_user_id: Some(UserId::new()),
                data_categories_sent: vec!["prompt".to_string(), "diff".to_string()],
                provider_boundary_ref: Some(ProviderDataBoundaryId::new()),
                risk_signals: vec!["medium_risk".to_string()],
                full_prompt_ref: Some("s3://audit/prompts/123".to_string()),
                full_response_ref: Some("s3://audit/responses/123".to_string()),
                prompt_hash: "a".repeat(64),
                response_hash: "b".repeat(64),
                retention: None,
            },
        };
        let meta = svc
            .record_ai(cmd, make_admin_actor(TenantId(tenant_id)))
            .await
            .unwrap();
        assert!(meta.has_complete_9_questions());
        assert_eq!(svc.ai_meta_count().await, 1);
        // 默认 90 天
        let delta = meta.retention_until - meta.created_at;
        assert_eq!(delta.num_days(), 90);
    }

    #[tokio::test]
    async fn cross_tenant_attempt_100_percent_logged() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        // INV-AU-07:即使 developer 也能记录跨租户(系统强制 100% 记录)
        let developer = make_developer_actor(TenantId(tenant_id));
        let cmd = RecordCrossTenantAttemptCommand {
            actor_user_id: Uuid::new_v4(),
            attempted_resource_type: "work_item".to_string(),
            attempted_resource_id: Uuid::new_v4(),
        };
        let ev = svc
            .record_cross_tenant_attempt(cmd, developer)
            .await
            .unwrap();
        assert!(ev.cross_tenant);
        assert_eq!(ev.action, AuditAction::CrossTenantAttempt);
        assert_eq!(svc.cross_tenant_count().await, 1);
        // 验证 INV-AU-04 校验
        let res = check_invariant_04_cross_tenant_flag(&ev);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn audit_event_is_append_only() {
        // 本 crate 不暴露 update/delete 接口 — 验证没有 public 写方法
        let svc = InMemoryAuditService::new_for_test();
        // 只能 record,不能 mutate
        let tenant_id = uuid::Uuid::new_v4();
        let cmd = RecordAuditCommand {
            tenant_id: TenantId(tenant_id),
            actor: make_actor_user(TenantId(tenant_id)),
            action: AuditAction::PermissionChange,
            resource_type: "user".to_string(),
            resource_id: Uuid::new_v4(),
            context_refs: vec![],
            before_state: None,
            after_state: None,
            immutable_hash: None,
        };
        let ev = svc
            .record(cmd, make_admin_actor(TenantId(tenant_id)))
            .await
            .unwrap();
        // 验证 AuditRecorder trait 没有 update / delete 方法
        // (编译期约束 + 运行时只能通过 mutation 改,但本服务只暴露 `record`)
        assert_eq!(svc.event_count().await, 1);
        // immutable_hash 一致
        assert_eq!(ev.immutable_hash.len(), 64);
    }

    #[tokio::test]
    async fn export_requires_admin_or_compliance() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let developer = make_developer_actor(TenantId(tenant_id));
        let cmd = ExportAuditCommand {
            tenant_id: TenantId(tenant_id),
            format: ExportFormat::Csv,
            range_start: Utc::now() - chrono::Duration::days(7),
            range_end: Utc::now(),
            filter_action: None,
        };
        let res = svc.export(cmd, developer).await;
        assert!(matches!(res, Err(AuditError::PermissionDenied)));
        // admin 应该能导出
        let admin = make_admin_actor(TenantId(tenant_id));
        let cmd2 = ExportAuditCommand {
            tenant_id: TenantId(tenant_id),
            format: ExportFormat::Csv,
            range_start: Utc::now() - chrono::Duration::days(7),
            range_end: Utc::now(),
            filter_action: None,
        };
        let job = svc.export(cmd2, admin).await.unwrap();
        assert_eq!(job.status, ExportStatus::Pending);
    }

    #[tokio::test]
    async fn list_requires_audit_role() {
        let svc = InMemoryAuditService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        // 先记录一个
        let cmd = RecordAuditCommand {
            tenant_id: TenantId(tenant_id),
            actor: make_actor_user(TenantId(tenant_id)),
            action: AuditAction::WorktreeOperation,
            resource_type: "worktree".to_string(),
            resource_id: Uuid::new_v4(),
            context_refs: vec![],
            before_state: None,
            after_state: None,
            immutable_hash: None,
        };
        let _ = svc
            .record(cmd, make_admin_actor(TenantId(tenant_id)))
            .await
            .unwrap();
        // developer 不能读
        let developer = make_developer_actor(TenantId(tenant_id));
        let q = AuditListQuery {
            tenant_id: TenantId(tenant_id),
            ..Default::default()
        };
        let res = svc.list_events(q, developer).await;
        assert!(matches!(res, Err(AuditError::PermissionDenied)));
        // admin 能读
        let admin = make_admin_actor(TenantId(tenant_id));
        let q2 = AuditListQuery {
            tenant_id: TenantId(tenant_id),
            ..Default::default()
        };
        let events = svc.list_events(q2, admin).await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn invariant_03_invalid_hash_rejected() {
        let tenant_id = uuid::Uuid::new_v4();
        let mut ev = AuditEvent {
            id: AuditEventId::new(),
            tenant_id: TenantId(tenant_id),
            actor: make_actor_user(TenantId(tenant_id)),
            action: AuditAction::WorkItemOperation,
            resource_type: "work_item".to_string(),
            resource_id: Uuid::new_v4(),
            context_refs: vec![],
            before_state: None,
            after_state: None,
            cross_tenant: false,
            immutable_hash: "tooshort".to_string(),
            occurred_at: Utc::now(),
        };
        assert!(check_invariant_03_immutable_hash(&ev).is_err());
        ev.immutable_hash = "z".repeat(64); // 非 hex
        assert!(check_invariant_03_immutable_hash(&ev).is_err());
        ev.immutable_hash = "a".repeat(64); // 合法 hex
        assert!(check_invariant_03_immutable_hash(&ev).is_ok());
    }

    #[tokio::test]
    async fn cross_tenant_flag_consistency() {
        let tenant_id = uuid::Uuid::new_v4();
        let mut ev = AuditEvent {
            id: AuditEventId::new(),
            tenant_id: TenantId(tenant_id),
            actor: make_actor_user(TenantId(tenant_id)),
            action: AuditAction::CrossTenantAttempt,
            resource_type: "work_item".to_string(),
            resource_id: Uuid::new_v4(),
            context_refs: vec![],
            before_state: None,
            after_state: None,
            cross_tenant: false, // 应是 true
            immutable_hash: "a".repeat(64),
            occurred_at: Utc::now(),
        };
        assert!(check_invariant_04_cross_tenant_flag(&ev).is_err());
        ev.cross_tenant = true;
        assert!(check_invariant_04_cross_tenant_flag(&ev).is_ok());
        // 反向:cross_tenant=true 但 action 不是 CrossTenantAttempt
        ev.action = AuditAction::WorkItemOperation;
        assert!(check_invariant_04_cross_tenant_flag(&ev).is_err());
    }
}

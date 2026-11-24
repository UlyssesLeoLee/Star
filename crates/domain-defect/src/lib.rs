//! domain-defect crate — 智能缺陷管理领域
//!
//! 详细 spec: docs/detailed-design/DD-DEFECT-INTELLIGENCE-001.md
//! 基本设计: docs/basic-design/BD-DEFECT-INTELLIGENCE-001.md
//! 要件定义: docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md
//!
//! ## 职责
//! 缺陷（Bug WorkItem）的生命周期：分诊、确认、处理、提交验证、关闭、重开；
//! 缺陷字段、关系、验证记录、智能建议草稿、分诊操作。
//!
//! ## 复用原则（per SRS §1.3、BD §4）
//! 复用 WorkItem、看板、验证、评论、附件、审计、通知、授权组件；
//! 不新造第二套权威缺陷列表。缺陷以 Bug WorkItem 承载，扩展字段由本域持久化。
//!
//! ## 生产就绪说明（P0-1/P0-2）
//! - 本 crate 当前以 InMemoryDefectService 作为开发实现；
//! - PostgreSQL 持久化、审计写入、Outbox 投递、真实鉴权均**未接入**，
//!   对应代码路径保持 fail-closed 并标注“未接入”，不冒充生产能力。
//!
//! 本 crate 处于开发阶段，先按逻辑骨架交付，文档注释将在 PostgreSQL 持久化
//! 接线完成后补全。
#![allow(missing_docs)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_context::ActorContext;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use uuid::Uuid;

pub mod pg_schema;
pub mod pg_service;

use domain_work_item::WorkItemId;

// =====================================================================
// 强类型 ID（复用 domain-work-item 的 WorkItemId，缺陷复用同一 ID 空间）
// =====================================================================

/// 缺陷 ID = WorkItem ID（Bug 类型 WorkItem 承载）
pub type DefectId = WorkItemId;

/// 验证记录 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VerificationId(pub Uuid);
impl VerificationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}
impl From<Uuid> for VerificationId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

/// 智能建议 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SuggestionId(pub Uuid);
impl SuggestionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}
impl From<Uuid> for SuggestionId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

/// 幂等键（用于重复命令去重，开发阶段以字符串承载，生产迁移至数据库唯一约束）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(pub String);
impl IdempotencyKey {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

// =====================================================================
// 缺陷状态机（SRS §2.1）
// 受控流转表（per BD §5）：
//   Triage → Confirmed | PendingInfo | Duplicate | NoAction
//   Confirmed → InProgress
//   InProgress → PendingVerify
//   PendingVerify → Closed | Reopened
//   Closed → Reopened
//   Reopened → InProgress
//   PendingInfo → Triage | Confirmed
//   Duplicate/NoAction → 终态，不可再流转
// 禁止：任意状态直接跳到 Closed（必须经过 PendingVerify）
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectStatus {
    Triage,
    Confirmed,
    InProgress,
    PendingVerify,
    Closed,
    PendingInfo,
    Duplicate,
    NoAction,
    Reopened,
}

impl DefectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Triage => "triage",
            Self::Confirmed => "confirmed",
            Self::InProgress => "in_progress",
            Self::PendingVerify => "pending_verify",
            Self::Closed => "closed",
            Self::PendingInfo => "pending_info",
            Self::Duplicate => "duplicate",
            Self::NoAction => "no_action",
            Self::Reopened => "reopened",
        }
    }

    pub fn can_transition(from: Self, to: Self) -> Result<(), String> {
        use DefectStatus::*;
        matches_terminal_transition(from, to)?;

        let allowed = match (from, to) {
            (Triage, Confirmed) | (Triage, PendingInfo) | (Triage, Duplicate) | (Triage, NoAction) => true,
            (PendingInfo, Triage) | (PendingInfo, Confirmed) => true,
            (Confirmed, InProgress) => true,
            (InProgress, PendingVerify) => true,
            (PendingVerify, Closed) | (PendingVerify, Reopened) => true,
            (Closed, Reopened) => true,
            (Reopened, InProgress) => true,
            _ => false,
        };
        if allowed {
            return Ok(());
        }
        Err(format!(
            "禁止的状态流转：{} -> {}。缺陷必须经过“待验证”验证通过后才能关闭；\
             Duplicate/NoAction 为终态，不可再流转。",
            from.as_str(),
            to.as_str()
        ))
    }
}

fn matches_terminal_transition(from: DefectStatus, to: DefectStatus) -> Result<(), String> {
    use DefectStatus::*;
    if matches!(from, Duplicate | NoAction) {
        return Err(format!(
            "禁止的状态流转：{} -> {}。{} 已是终态，不可再流转。",
            from.as_str(),
            to.as_str(),
            from.as_str()
        ));
    }
    Ok(())
}

/// 缺陷严重度（SRS §2.1）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectSeverity {
    S1,
    S2,
    S3,
    S4,
    Unknown,
}
impl DefectSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S1 => "S1",
            Self::S2 => "S2",
            Self::S3 => "S3",
            Self::S4 => "S4",
            Self::Unknown => "unknown",
        }
    }
}

/// 缺陷优先级（SRS §2.1）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectPriority {
    P0,
    P1,
    P2,
    P3,
    Unscheduled,
}
impl DefectPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
            Self::Unscheduled => "unscheduled",
        }
    }
}

/// 缺陷发现阶段（SRS FR-DI-003）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryStage {
    Requirements,
    Design,
    Implementation,
    UnitTest,
    IntegrationTest,
    SystemTest,
    ReleasePost,
    Unknown,
}
impl DiscoveryStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requirements => "requirements",
            Self::Design => "design",
            Self::Implementation => "implementation",
            Self::UnitTest => "unit_test",
            Self::IntegrationTest => "integration_test",
            Self::SystemTest => "system_test",
            Self::ReleasePost => "release_post",
            Self::Unknown => "unknown",
        }
    }
}

// =====================================================================
// 验证记录实体（SRS FR-DI-006 / BD §6）
// 验证记录必须保存：验证主体、修复版本、方法、结果、证据，并按策略检查验证独立性。
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationResult {
    Passed,
    Failed,
}
impl VerificationResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectVerification {
    pub id: VerificationId,
    pub defect_id: DefectId,
    pub verifier_id: Uuid,
    pub fix_version: String,
    pub method: Option<String>,
    pub result: VerificationResult,
    pub evidence_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl DefectVerification {
    pub fn new(
        id: VerificationId,
        defect_id: DefectId,
        verifier_id: Uuid,
        fix_version: String,
        method: Option<String>,
        result: VerificationResult,
        evidence_id: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            defect_id,
            verifier_id,
            fix_version,
            method,
            result,
            evidence_id,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }
}

// =====================================================================
// 智能建议草稿实体（SRS FR-DI-007 / BD §7）
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuggestionStatus {
    Draft,
    Adopted,
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectSuggestion {
    pub id: SuggestionId,
    pub defect_id: DefectId,
    pub suggested_severity: Option<DefectSeverity>,
    pub suggested_priority: Option<DefectPriority>,
    pub suggested_discovery_stage: Option<DiscoveryStage>,
    pub reason: Option<String>,
    pub status: SuggestionStatus,
    pub created_at: DateTime<Utc>,
}

impl DefectSuggestion {
    pub fn new(
        id: SuggestionId,
        defect_id: DefectId,
        suggested_severity: Option<DefectSeverity>,
        suggested_priority: Option<DefectPriority>,
        suggested_discovery_stage: Option<DiscoveryStage>,
        reason: Option<String>,
    ) -> Self {
        Self {
            id,
            defect_id,
            suggested_severity,
            suggested_priority,
            suggested_discovery_stage,
            reason,
            status: SuggestionStatus::Draft,
            created_at: Utc::now(),
        }
    }
}

// =====================================================================
// 缺陷实体（T，SCD2，业务事实禁止物理删除）
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defect {
    pub id: DefectId,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub lane_id: Option<Uuid>,
    pub worktree_id: Option<WorkItemId>,
    pub branch_id: Option<Uuid>,
    pub branch_name: Option<String>,
    pub title: String,
    pub phenomenon: String,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
    pub reproduction_steps: Option<String>,
    pub impact_scope: Option<String>,
    pub discovery_stage: Option<DiscoveryStage>,
    pub severity: DefectSeverity,
    pub priority: DefectPriority,
    pub status: DefectStatus,
    pub reporter_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub environment: Option<String>,
    pub labels: Vec<String>,
    pub discovery_version: Option<String>,
    pub fix_version: Option<String>,
    pub lifecycle_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Defect {
    pub fn new(
        id: DefectId,
        tenant_id: Uuid,
        project_id: Uuid,
        reporter_id: Uuid,
        title: String,
        phenomenon: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            tenant_id,
            project_id,
            lane_id: None,
            worktree_id: None,
            branch_id: None,
            branch_name: None,
            title,
            phenomenon,
            expected_result: None,
            actual_result: None,
            reproduction_steps: None,
            impact_scope: None,
            discovery_stage: None,
            severity: DefectSeverity::Unknown,
            priority: DefectPriority::Unscheduled,
            status: DefectStatus::Triage,
            reporter_id,
            assignee_id: None,
            environment: None,
            labels: vec![],
            discovery_version: None,
            fix_version: None,
            lifecycle_version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

// =====================================================================
// 错误类型
// =====================================================================

#[derive(Debug, Error)]
pub enum DefectError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("cross-tenant access denied: defect tenant {0} vs actor tenant {1}")]
    CrossTenantDenied(Uuid, Uuid),
    #[error("cross-project access denied: defect project {0} vs actor project {1}")]
    CrossProjectDenied(Uuid, Uuid),
    #[error("invalid state transition: {from} -> {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("idempotency key already used: {0}")]
    IdempotencyKeyUsed(String),
    #[error("verification mismatch: {0}")]
    VerificationMismatch(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDefectCommand {
    pub idempotency_key: Option<IdempotencyKey>,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub lane_id: Option<Uuid>,
    pub worktree_id: Option<WorkItemId>,
    pub branch_id: Option<Uuid>,
    pub branch_name: Option<String>,
    pub title: String,
    pub phenomenon: String,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
    pub reproduction_steps: Option<String>,
    pub impact_scope: Option<String>,
    pub discovery_stage: Option<String>,
    pub severity: Option<String>,
    pub priority: Option<String>,
    pub reporter_id: Uuid,
    pub environment: Option<String>,
    pub labels: Vec<String>,
    pub discovery_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDefectCommand {
    pub idempotency_key: Option<IdempotencyKey>,
    pub target_status: DefectStatus,
    pub expected_version: Option<u32>,
    /** 提交修复版本（在进入 in_progress / pending_verify 时设定，供 submit_verification 校验） */
    pub fix_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitVerificationCommand {
    pub idempotency_key: Option<IdempotencyKey>,
    pub fix_version: String,
    pub verification_method: Option<String>,
    pub result: bool,
    pub evidence_id: Option<Uuid>,
}

// =====================================================================
// 查询结果 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectListItem {
    pub id: DefectId,
    pub title: String,
    pub status: DefectStatus,
    pub severity: DefectSeverity,
    pub priority: DefectPriority,
    pub reporter_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub lane_id: Option<Uuid>,
    pub worktree_id: Option<WorkItemId>,
    pub branch_name: Option<String>,
    pub fix_version: Option<String>,
    pub lifecycle_version: u32,
    pub updated_at: DateTime<Utc>,
}

impl From<&Defect> for DefectListItem {
    fn from(d: &Defect) -> Self {
        Self {
            id: d.id,
            title: d.title.clone(),
            status: d.status,
            severity: d.severity,
            priority: d.priority,
            reporter_id: d.reporter_id,
            assignee_id: d.assignee_id,
            lane_id: d.lane_id,
            worktree_id: d.worktree_id,
            branch_name: d.branch_name.clone(),
            fix_version: d.fix_version.clone(),
            lifecycle_version: d.lifecycle_version,
            updated_at: d.updated_at,
        }
    }
}

// =====================================================================
// 端口 Trait
// =====================================================================

#[async_trait]
pub trait DefectCommandPort: Send + Sync {
    async fn create_defect(
        &self,
        cmd: CreateDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
    async fn transition_status(
        &self,
        defect_id: DefectId,
        cmd: TransitionDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
    async fn submit_verification(
        &self,
        defect_id: DefectId,
        cmd: SubmitVerificationCommand,
        actor: &ActorContext,
    ) -> Result<(Defect, DefectVerification), DefectError>;
}

#[async_trait]
pub trait DefectQueryPort: Send + Sync {
    async fn get_defect(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
    async fn list_defects(
        &self,
        project_id: Uuid,
        lane_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Vec<DefectListItem>, DefectError>;
    async fn list_verifications(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Vec<DefectVerification>, DefectError>;
}

// =====================================================================
// 授权上下文帮助
// =====================================================================

fn actor_tenant(actor: &ActorContext) -> Uuid {
    actor.tenant_id
}

fn actor_user(actor: &ActorContext) -> Uuid {
    actor.user_id
}

pub(crate) fn check_project_access(actor: &ActorContext, project_id: Uuid) -> Result<(), DefectError> {
    if actor.can_access_project(project_id) {
        return Ok(());
    }
    Err(DefectError::CrossProjectDenied(project_id, actor_tenant(actor)))
}

pub(crate) fn check_defect_write_capability(actor: &ActorContext) -> Result<(), DefectError> {
    use star_context::roles::*;
    if actor.is_platform_admin()
        || actor.has_role(TENANT_ADMIN)
        || actor.has_role(PROJECT_ADMIN)
        || actor.has_role(DEVELOPER)
    {
        return Ok(());
    }
    Err(DefectError::PermissionDenied(
        "当前角色无缺陷写入权限，需 developer / project_admin / tenant_admin 或平台管理员".into(),
    ))
}

fn check_defect_verify_capability(
    actor: &ActorContext,
    defect_reporter_id: Uuid,
) -> Result<(), DefectError> {
    use star_context::roles::*;
    let is_admin = actor.is_platform_admin()
        || actor.has_role(TENANT_ADMIN)
        || actor.has_role(PROJECT_ADMIN);
    let is_dev = actor.has_role(DEVELOPER);
    if !is_admin && !is_dev {
        return Err(DefectError::PermissionDenied(
            "当前角色无缺陷验证权限".into(),
        ));
    }
    if actor_user(actor) == defect_reporter_id && !is_admin {
        return Err(DefectError::PermissionDenied(
            "验证独立性要求：验证人不能是缺陷报告人".into(),
        ));
    }
    Ok(())
}

// =====================================================================
// 审计与 Outbox 接口预留（生产未接入前保持 fail-closed）
// =====================================================================

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub event_type: String,
    pub actor_user_id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Option<Uuid>,
    pub target_id: Option<String>,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

pub trait AuditPort: Send + Sync {
    fn record(&self, event: AuditEvent) -> Result<(), DefectError>;
}

#[derive(Debug, Clone)]
pub struct OutboxMessage {
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

pub trait OutboxPort: Send + Sync {
    fn emit(&self, msg: OutboxMessage) -> Result<(), DefectError>;
}

pub struct NoopAuditPort;
impl AuditPort for NoopAuditPort {
    fn record(&self, _event: AuditEvent) -> Result<(), DefectError> {
        Ok(())
    }
}

pub struct NoopOutboxPort;
impl OutboxPort for NoopOutboxPort {
    fn emit(&self, _msg: OutboxMessage) -> Result<(), DefectError> {
        Ok(())
    }
}

// =====================================================================
// 内存服务实现（开发阶段使用，生产需替换为 PostgreSQL 实现）
// =====================================================================

pub struct InMemoryDefectService {
    defects: Arc<RwLock<HashMap<DefectId, Defect>>>,
    verifications: Arc<RwLock<HashMap<VerificationId, DefectVerification>>>,
    #[allow(dead_code)] // 智能建议读侧端口待接线（per WBS Phase B）
    suggestions: Arc<RwLock<HashMap<SuggestionId, DefectSuggestion>>>,
    idempotency_records: Arc<RwLock<HashMap<IdempotencyKey, DefectId>>>,
    audit: Arc<dyn AuditPort + Send + Sync>,
    outbox: Arc<dyn OutboxPort + Send + Sync>,
}

impl InMemoryDefectService {
    pub fn new() -> Self {
        Self {
            defects: Arc::new(RwLock::new(HashMap::new())),
            verifications: Arc::new(RwLock::new(HashMap::new())),
            suggestions: Arc::new(RwLock::new(HashMap::new())),
            idempotency_records: Arc::new(RwLock::new(HashMap::new())),
            audit: Arc::new(NoopAuditPort),
            outbox: Arc::new(NoopOutboxPort),
        }
    }

    pub fn with_ports(
        audit: Arc<dyn AuditPort + Send + Sync>,
        outbox: Arc<dyn OutboxPort + Send + Sync>,
    ) -> Self {
        Self {
            defects: Arc::new(RwLock::new(HashMap::new())),
            verifications: Arc::new(RwLock::new(HashMap::new())),
            suggestions: Arc::new(RwLock::new(HashMap::new())),
            idempotency_records: Arc::new(RwLock::new(HashMap::new())),
            audit,
            outbox,
        }
    }
}

impl Default for InMemoryDefectService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DefectCommandPort for InMemoryDefectService {
    async fn create_defect(
        &self,
        cmd: CreateDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        check_defect_write_capability(actor)?;
        if cmd.tenant_id != actor_tenant(actor) {
            return Err(DefectError::CrossTenantDenied(cmd.tenant_id, actor_tenant(actor)));
        }
        check_project_access(actor, cmd.project_id)?;

        if let Some(key) = &cmd.idempotency_key {
            let records = self.idempotency_records.read().unwrap();
            if let Some(existing_id) = records.get(key) {
                let defects = self.defects.read().unwrap();
                if let Some(d) = defects.get(existing_id) {
                    return Ok(d.clone());
                }
                drop(records);
            }
        }

        let now = Utc::now();
        let defect = Defect {
            id: DefectId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            lane_id: cmd.lane_id,
            worktree_id: cmd.worktree_id,
            branch_id: cmd.branch_id,
            branch_name: cmd.branch_name,
            title: cmd.title,
            phenomenon: cmd.phenomenon,
            expected_result: cmd.expected_result,
            actual_result: cmd.actual_result,
            reproduction_steps: cmd.reproduction_steps,
            impact_scope: cmd.impact_scope,
            discovery_stage: cmd.discovery_stage.as_ref().map(|s| s.as_str()).map(|s| {
                match s {
                    "requirements" => DiscoveryStage::Requirements,
                    "design" => DiscoveryStage::Design,
                    "implementation" => DiscoveryStage::Implementation,
                    "unit_test" => DiscoveryStage::UnitTest,
                    "integration_test" => DiscoveryStage::IntegrationTest,
                    "system_test" => DiscoveryStage::SystemTest,
                    "release_post" => DiscoveryStage::ReleasePost,
                    _ => DiscoveryStage::Unknown,
                }
            }),
            severity: cmd.severity.as_deref().map(|s| match s {
                "S1" => DefectSeverity::S1,
                "S2" => DefectSeverity::S2,
                "S3" => DefectSeverity::S3,
                "S4" => DefectSeverity::S4,
                _ => DefectSeverity::Unknown,
            }).unwrap_or(DefectSeverity::Unknown),
            priority: cmd.priority.as_deref().map(|s| match s {
                "P0" => DefectPriority::P0,
                "P1" => DefectPriority::P1,
                "P2" => DefectPriority::P2,
                "P3" => DefectPriority::P3,
                _ => DefectPriority::Unscheduled,
            }).unwrap_or(DefectPriority::Unscheduled),
            status: DefectStatus::Triage,
            reporter_id: cmd.reporter_id,
            assignee_id: None,
            environment: cmd.environment,
            labels: cmd.labels,
            discovery_version: cmd.discovery_version,
            fix_version: None,
            lifecycle_version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let audit_event = AuditEvent {
            event_type: "defect.created".into(),
            actor_user_id: actor_user(actor),
            tenant_id: cmd.tenant_id,
            project_id: Some(cmd.project_id),
            target_id: Some(defect.id.as_uuid().to_string()),
            payload: serde_json::json!({
                "title": &defect.title,
                "severity": defect.severity.as_str(),
                "priority": defect.priority.as_str(),
            }),
            created_at: now,
        };
        self.audit.record(audit_event)?;
        self.outbox.emit(OutboxMessage {
            aggregate_type: "Defect".into(),
            aggregate_id: defect.id.as_uuid().to_string(),
            event_type: "defect.created".into(),
            payload: serde_json::json!({
                "defect_id": defect.id.as_uuid(),
                "project_id": cmd.project_id,
                "tenant_id": cmd.tenant_id,
                "title": &defect.title,
            }),
        })?;

        self.defects.write().unwrap().insert(defect.id, defect.clone());

        if let Some(key) = cmd.idempotency_key {
            self.idempotency_records.write().unwrap().insert(key, defect.id);
        }

        Ok(defect)
    }

    async fn transition_status(
        &self,
        defect_id: DefectId,
        cmd: TransitionDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        check_defect_write_capability(actor)?;

        if let Some(key) = &cmd.idempotency_key {
            let records = self.idempotency_records.read().unwrap();
            if let Some(existing_id) = records.get(key) {
                if *existing_id == defect_id {
                    let defects = self.defects.read().unwrap();
                    if let Some(d) = defects.get(existing_id) {
                        return Ok(d.clone());
                    }
                }
                drop(records);
            }
        }

        let mut defects = self.defects.write().unwrap();
        let defect = defects
            .get_mut(&defect_id)
            .ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;

        if defect.tenant_id != actor_tenant(actor) {
            return Err(DefectError::CrossTenantDenied(defect.tenant_id, actor_tenant(actor)));
        }
        check_project_access(actor, defect.project_id)?;

        if let Some(expected) = cmd.expected_version {
            if defect.lifecycle_version != expected {
                return Err(DefectError::Conflict(format!(
                    "版本冲突：期望版本 {}, 当前版本 {}，请刷新后重试。",
                    expected, defect.lifecycle_version
                )));
            }
        }

        DefectStatus::can_transition(defect.status, cmd.target_status).map_err(|reason| {
            DefectError::InvalidTransition {
                from: defect.status.as_str().to_string(),
                to: cmd.target_status.as_str().to_string(),
                reason,
            }
        })?;

        defect.status = cmd.target_status;
        defect.lifecycle_version += 1;
        defect.updated_at = Utc::now();
        if matches!(
            defect.status,
            DefectStatus::InProgress | DefectStatus::PendingVerify
        ) {
            defect.fix_version = cmd.fix_version.clone();
        }
        let result = defect.clone();

        let audit_event = AuditEvent {
            event_type: "defect.transitioned".into(),
            actor_user_id: actor_user(actor),
            tenant_id: defect.tenant_id,
            project_id: Some(defect.project_id),
            target_id: Some(defect_id.as_uuid().to_string()),
            payload: serde_json::json!({
                "from": defect.status.as_str(),
                "to": cmd.target_status.as_str(),
                "version": defect.lifecycle_version,
            }),
            created_at: Utc::now(),
        };
        self.audit.record(audit_event)?;
        self.outbox.emit(OutboxMessage {
            aggregate_type: "Defect".into(),
            aggregate_id: defect_id.as_uuid().to_string(),
            event_type: "defect.transitioned".into(),
            payload: serde_json::json!({
                "defect_id": defect_id.as_uuid(),
                "from": defect.status.as_str(),
                "to": cmd.target_status.as_str(),
            }),
        })?;

        if let Some(key) = cmd.idempotency_key {
            self.idempotency_records.write().unwrap().insert(key, defect_id);
        }

        Ok(result)
    }

    async fn submit_verification(
        &self,
        defect_id: DefectId,
        cmd: SubmitVerificationCommand,
        actor: &ActorContext,
    ) -> Result<(Defect, DefectVerification), DefectError> {
        let mut defects = self.defects.write().unwrap();
        let mut defect = defects
            .get(&defect_id)
            .ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?
            .clone();

        if defect.tenant_id != actor_tenant(actor) {
            return Err(DefectError::CrossTenantDenied(defect.tenant_id, actor_tenant(actor)));
        }
        check_project_access(actor, defect.project_id)?;
        check_defect_verify_capability(actor, defect.reporter_id)?;

        if !matches!(
            defect.status,
            DefectStatus::PendingVerify | DefectStatus::Closed | DefectStatus::Reopened
        ) {
            return Err(DefectError::InvalidTransition {
                from: defect.status.as_str().to_string(),
                to: "submit_verification".into(),
                reason: "仅处于“待验证/已关闭/重新打开”的缺陷才能提交验证记录".into(),
            });
        }

        let current_fix = defect.fix_version.as_deref();
        if current_fix.is_none() {
            return Err(DefectError::VerificationMismatch(
                "缺陷尚未设定修复版本，无法提交验证".into(),
            ));
        }
        if current_fix != Some(cmd.fix_version.as_str()) {
            return Err(DefectError::VerificationMismatch(format!(
                "修复版本不匹配：缺陷当前 fix_version={:?}，提交的 fix_version={}",
                current_fix, cmd.fix_version
            )));
        }

        let verification_id = VerificationId::new();
        let result = if cmd.result {
            VerificationResult::Passed
        } else {
            VerificationResult::Failed
        };
        let verification = DefectVerification::new(
            verification_id,
            defect_id,
            actor_user(actor),
            cmd.fix_version.clone(),
            cmd.verification_method.clone(),
            result,
            cmd.evidence_id,
        );

        let new_status = if cmd.result {
            DefectStatus::Closed
        } else {
            DefectStatus::Reopened
        };
        DefectStatus::can_transition(defect.status, new_status).map_err(|reason| {
            DefectError::InvalidTransition {
                from: defect.status.as_str().to_string(),
                to: new_status.as_str().to_string(),
                reason,
            }
        })?;

        defect.status = new_status;
        defect.fix_version = Some(cmd.fix_version.clone());
        defect.lifecycle_version += 1;
        defect.updated_at = Utc::now();

        defects.insert(defect.id, defect.clone());
        self.verifications
            .write()
            .unwrap()
            .insert(verification.id, verification.clone());
        drop(defects);

        if let Some(key) = cmd.idempotency_key {
            self.idempotency_records.write().unwrap().insert(key, defect_id);
        }

        let audit_event = AuditEvent {
            event_type: if cmd.result {
                "defect.verification_passed".into()
            } else {
                "defect.verification_failed".into()
            },
            actor_user_id: actor_user(actor),
            tenant_id: defect.tenant_id,
            project_id: Some(defect.project_id),
            target_id: Some(defect_id.as_uuid().to_string()),
            payload: serde_json::json!({
                "fix_version": cmd.fix_version,
                "result": result.as_str(),
                "verification_id": verification_id.as_uuid(),
            }),
            created_at: Utc::now(),
        };
        self.audit.record(audit_event)?;
        self.outbox.emit(OutboxMessage {
            aggregate_type: "Defect".into(),
            aggregate_id: defect_id.as_uuid().to_string(),
            event_type: "defect.verification_submitted".into(),
            payload: serde_json::json!({
                "defect_id": defect_id.as_uuid(),
                "verification_id": verification_id.as_uuid(),
                "result": result.as_str(),
            }),
        })?;

        Ok((defect, verification))
    }
}

#[async_trait]
impl DefectQueryPort for InMemoryDefectService {
    async fn get_defect(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        let defects = self.defects.read().unwrap();
        let defect = defects
            .get(&defect_id)
            .ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;
        if defect.tenant_id != actor_tenant(actor) {
            return Err(DefectError::CrossTenantDenied(defect.tenant_id, actor_tenant(actor)));
        }
        check_project_access(actor, defect.project_id)?;
        Ok(defect.clone())
    }

    async fn list_defects(
        &self,
        project_id: Uuid,
        lane_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Vec<DefectListItem>, DefectError> {
        check_project_access(actor, project_id)?;
        let defects = self.defects.read().unwrap();
        Ok(defects
            .values()
            .filter(|d| d.project_id == project_id)
            .filter(|d| d.tenant_id == actor_tenant(actor))
            .filter(|d| lane_id.map_or(true, |l| d.lane_id == Some(l)))
            .map(DefectListItem::from)
            .collect())
    }

    async fn list_verifications(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Vec<DefectVerification>, DefectError> {
        let defect = {
            let defects = self.defects.read().unwrap();
            defects
                .get(&defect_id)
                .ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?
                .clone()
        };
        if defect.tenant_id != actor_tenant(actor) {
            return Err(DefectError::CrossTenantDenied(defect.tenant_id, actor_tenant(actor)));
        }
        check_project_access(actor, defect.project_id)?;
        let verifications = self.verifications.read().unwrap();
        Ok(verifications
            .values()
            .filter(|v| v.defect_id == defect_id)
            .cloned()
            .collect())
    }
}

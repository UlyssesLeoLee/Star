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

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_context::ActorContext;
use std::{collections::HashMap, sync::{Arc, RwLock}};
use thiserror::Error;
use uuid::Uuid;

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
    /// 生成一个新的随机验证记录 ID（基于 UUID v4）
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    /// 返回底层 UUID 值
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
    /// 生成一个新的随机智能建议 ID（基于 UUID v4）
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    /// 返回底层 UUID 值
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}
impl From<Uuid> for SuggestionId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

// =====================================================================
// 缺陷状态机（SRS §2.1）
// 待分诊(triage) → 已确认(confirmed) → 处理中(in_progress) → 待验证(pending_verify) → 已关闭(closed)
// 待分诊 ↔ 待补充(pending_info)；已确认/重新打开 → 处理中；处理中 → 待验证；
// 待验证 → 已关闭；待验证/已关闭 → 重新打开(reopened)；待分诊/已确认 → 重复(duplicate)/不处理(no_action)
// =====================================================================

/// 缺陷状态机，兼容 SRS §2.1 定义的 9 种状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectStatus {
    /// 待分诊：尚未由分诊员确认
    Triage,
    /// 已确认：分诊后确认为有效缺陷
    Confirmed,
    /// 处理中：已指派给开发者正在修复
    InProgress,
    /// 待验证：修复已提交，等待测试验证
    PendingVerify,
    /// 已关闭：验证通过，缺陷已修复
    Closed,
    /// 待补充：需要重porter补充信息
    PendingInfo,
    /// 重复：与已有缺陷重复
    Duplicate,
    /// 不处理：评估后决定不修复
    NoAction,
    /// 重新打开：验证失败或关闭后被重新打开
    Reopened,
}

impl DefectStatus {
    /// 返回状态的字符串表示（用于持久化和前端展示）
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
}

/// 缺陷严重度（SRS §2.1）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectSeverity {
    /// S1：致命，系统不可用，需立即修复
    S1,
    /// S2：严重，核心功能异常
    S2,
    /// S3：一般，部分功能异常
    S3,
    /// S4：轻微，不影响主要功能
    S4,
    /// Unknown：未分类
    Unknown,
}

impl DefectSeverity {
    /// 返回严重度的字符串表示
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
    /// P0：最高，立即处理
    P0,
    /// P1：高，优先处理
    P1,
    /// P2：中，正常排期
    P2,
    /// P3：低，可排期至后续版本
    P3,
    /// Unscheduled：不排期
    Unscheduled,
}

impl DefectPriority {
    /// 返回优先级的字符串表示
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
    /// 需求阶段发现
    Requirements,
    /// 设计阶段发现
    Design,
    /// 实现阶段发现
    Implementation,
    /// 单元测试阶段发现
    UnitTest,
    /// 集成测试阶段发现
    IntegrationTest,
    /// 系统测试阶段发现
    SystemTest,
    /// 发布后发现
    ReleasePost,
    /// 未知阶段
    Unknown,
}

impl DiscoveryStage {
    /// 返回发现阶段的字符串表示
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
// 缺陷实体（T，SCD2，业务事实禁止物理删除）
// =====================================================================
/// 缺陷实体，承载 Bug 类型 WorkItem 的全部扩展字段。
/// 业务事实禁止物理删除，使用 deleted_at 标记逻辑删除。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defect {
    /// 缺陷 ID = WorkItem ID（Bug 类型 WorkItem 承载）
    pub id: DefectId,
    /// 租户 ID，用于多租户隔离
    pub tenant_id: Uuid,
    /// 项目 ID
    pub project_id: Uuid,
    /// 看板泳道 ID（可选）
    pub lane_id: Option<Uuid>,
    /// 关联 Worktree ID（可选）
    pub worktree_id: Option<Uuid>,
    /// 关联分支 ID（可选）
    pub branch_id: Option<Uuid>,
    /// 分支名称（可选）
    pub branch_name: Option<String>,
    /// 缺陷标题
    pub title: String,
    /// 缺陷现象描述
    pub phenomenon: String,
    /// 预期结果（可选）
    pub expected_result: Option<String>,
    /// 实际结果（可选）
    pub actual_result: Option<String>,
    /// 复现步骤（可选）
    pub reproduction_steps: Option<String>,
    /// 影响范围（可选）
    pub impact_scope: Option<String>,
    /// 发现阶段（可选）
    pub discovery_stage: Option<DiscoveryStage>,
    /// 严重度
    pub severity: DefectSeverity,
    /// 优先级
    pub priority: DefectPriority,
    /// 状态机状态
    pub status: DefectStatus,
    /// 报告人 ID
    pub reporter_id: Uuid,
    /// 指派人 ID（可选）
    pub assignee_id: Option<Uuid>,
    /// 环境信息（可选）
    pub environment: Option<String>,
    /// 标签列表
    pub labels: Vec<String>,
    /// 发现版本（可选）
    pub discovery_version: Option<String>,
    /// 修复版本（可选）
    pub fix_version: Option<String>,
    /// 乐观锁版本号，用于并发控制
    pub lifecycle_version: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 逻辑删除时间（可选，已删除则非空）
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Defect {
    /// 创建一个新缺陷实例，状态初始为 Triage，版本号为 1
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
/// 缺陷领域错误枚举
#[derive(Debug, Error)]
pub enum DefectError {
    /// 指定 ID 的缺陷不存在
    #[error("not found: {0}")]
    NotFound(String),
    /// 权限不足，当前角色无权执行该操作
    #[error("permission denied")]
    PermissionDenied,
    /// 跨租户访问被拒绝：缺陷租户与调用者租户不一致
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(Uuid, Uuid),
    /// 无效的状态转换：from -> to
    #[error("invalid state transition: {from} -> {to}")]
    InvalidTransition {
        /// 当前状态
        from: String,
        /// 目标状态
        to: String,
    },
    /// 并发冲突（乐观锁版本不匹配等）
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 DTO
// =====================================================================
/// 创建缺陷的命令输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDefectCommand {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 项目 ID
    pub project_id: Uuid,
    /// 看板泳道 ID（可选）
    pub lane_id: Option<Uuid>,
    /// 关联 Worktree ID（可选）
    pub worktree_id: Option<Uuid>,
    /// 关联分支 ID（可选）
    pub branch_id: Option<Uuid>,
    /// 分支名称（可选）
    pub branch_name: Option<String>,
    /// 缺陷标题
    pub title: String,
    /// 缺陷现象描述
    pub phenomenon: String,
    /// 预期结果（可选）
    pub expected_result: Option<String>,
    /// 实际结果（可选）
    pub actual_result: Option<String>,
    /// 复现步骤（可选）
    pub reproduction_steps: Option<String>,
    /// 影响范围（可选）
    pub impact_scope: Option<String>,
    /// 发现阶段字符串（可选）
    pub discovery_stage: Option<String>,
    /// 严重度字符串（可选）
    pub severity: Option<String>,
    /// 优先级字符串（可选）
    pub priority: Option<String>,
    /// 报告人 ID
    pub reporter_id: Uuid,
    /// 环境信息（可选）
    pub environment: Option<String>,
    /// 标签列表
    pub labels: Vec<String>,
    /// 发现版本（可选）
    pub discovery_version: Option<String>,
}

// =====================================================================
// 端口 Trait
// =====================================================================
/// 缺陷写端口：创建、状态转换、提交验证
#[async_trait]
pub trait DefectCommandPort: Send + Sync {
    /// 创建新缺陷，返回持久化后的完整 Defect 实体
    async fn create_defect(
        &self,
        cmd: CreateDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
    /// 将缺陷状态转换为目标状态，需带上期望版本号用于乐观锁校验
    async fn transition_status(
        &self,
        defect_id: DefectId,
        target_status: DefectStatus,
        expected_version: u32,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
    /// 提交验证结果，验证通过则关闭，未通过则重新打开
    async fn submit_verification(
        &self,
        defect_id: DefectId,
        fix_version: String,
        verification_method: Option<String>,
        result: bool,
        _evidence_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
}

/// 缺陷读端口：查询单个缺陷、列表查询
#[async_trait]
pub trait DefectQueryPort: Send + Sync {
    /// 按 ID 获取单个缺陷，校验租户隔离
    async fn get_defect(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError>;
    /// 按项目 ID 和可选泳道 ID 列出缺陷，校验租户隔离
    async fn list_defects(
        &self,
        project_id: Uuid,
        lane_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Vec<Defect>, DefectError>;
}

// =====================================================================
// 内存服务实现（开发阶段使用，生产需替换为 PostgreSQL 实现）
// =====================================================================
/// 基于内存的缺陷服务实现，仅用于开发阶段。
/// 生产环境应替换为 PostgreSQL 持久化实现。
pub struct InMemoryDefectService {
    defects: Arc<RwLock<HashMap<DefectId, Defect>>>,
}

impl InMemoryDefectService {
    /// 创建一个空的内存缺陷服务实例
    pub fn new() -> Self {
        Self {
            defects: Arc::new(RwLock::new(HashMap::new())),
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
        
        if Uuid::from_bytes([0; 16]) == cmd.tenant_id {
            return Err(DefectError::PermissionDenied);
        }
        if !actor.has_role("developer")
            && !actor.has_role("project_admin")
            && !actor.has_role("tenant_admin")
        {
            return Err(DefectError::PermissionDenied);
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
            severity: cmd
                .severity
                .as_deref()
                .map(|s| match s {
                    "S1" => DefectSeverity::S1,
                    "S2" => DefectSeverity::S2,
                    "S3" => DefectSeverity::S3,
                    "S4" => DefectSeverity::S4,
                    _ => DefectSeverity::Unknown,
                })
                .unwrap_or(DefectSeverity::Unknown),
            priority: cmd
                .priority
                .as_deref()
                .map(|s| match s {
                    "P0" => DefectPriority::P0,
                    "P1" => DefectPriority::P1,
                    "P2" => DefectPriority::P2,
                    "P3" => DefectPriority::P3,
                    _ => DefectPriority::Unscheduled,
                })
                .unwrap_or(DefectPriority::Unscheduled),
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
        self.defects.write().unwrap().insert(defect.id, defect.clone());
        Ok(defect)
    }

    async fn transition_status(
        &self,
        defect_id: DefectId,
        target_status: DefectStatus,
        expected_version: u32,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        
        if !actor.has_role("developer")
            && !actor.has_role("project_admin")
            && !actor.has_role("tenant_admin")
        {
            return Err(DefectError::PermissionDenied);
        }
        let mut defects = self.defects.write().unwrap();
        let defect = defects
            .get_mut(&defect_id)
            .ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;
        if defect.lifecycle_version != expected_version {
            return Err(DefectError::Conflict(format!(
                "expected version {}, current {}",
                expected_version, defect.lifecycle_version
            )));
        }
        defect.status = target_status;
        defect.lifecycle_version += 1;
        defect.updated_at = Utc::now();
        Ok(defect.clone())
    }

    async fn submit_verification(
        &self,
        defect_id: DefectId,
        fix_version: String,
        verification_method: Option<String>,
        result: bool,
        _evidence_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        let _verification_method = verification_method;
        
        if !actor.has_role("developer")
            && !actor.has_role("project_admin")
            && !actor.has_role("tenant_admin")
        {
            return Err(DefectError::PermissionDenied);
        }
        let mut defects = self.defects.write().unwrap();
        let defect = defects
            .get_mut(&defect_id)
            .ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;
        if defect.fix_version.as_deref() != Some(fix_version.as_str()) {
            return Err(DefectError::Conflict(format!(
                "fix_version mismatch: expected {:?}, got {}",
                defect.fix_version, fix_version
            )));
        }
        let new_status = if result {
            DefectStatus::Closed
        } else {
            DefectStatus::Reopened
        };
        defect.status = new_status;
        defect.fix_version = Some(fix_version);
        defect.lifecycle_version += 1;
        defect.updated_at = Utc::now();
        Ok(defect.clone())
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
        if defect.tenant_id != actor.tenant_id {
            return Err(DefectError::CrossTenantDenied(
                defect.tenant_id,
                actor.tenant_id,
            ));
        }
        Ok(defect.clone())
    }

    async fn list_defects(
        &self,
        project_id: Uuid,
        lane_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Vec<Defect>, DefectError> {
        let defects = self.defects.read().unwrap();
        Ok(defects
            .values()
            .filter(|d| d.project_id == project_id)
            .filter(|d| lane_id.map_or(true, |l| d.lane_id == Some(l)))
            .filter(|d| d.tenant_id == actor.tenant_id)
            .cloned()
            .collect())
    }
}

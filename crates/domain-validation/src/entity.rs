//! Validation 域实体
//!
//! 来源:
//! - `docs/data-design.md` §4.24 (`validation` schema)
//! - `docs/specs/domain-validation-spec.md` §2 (实体清单)
//! - `docs/basic-design.md` §4.5 (核心实体定义)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AcceptanceCoverageId, AcceptanceCriterionId, AgentSessionId, ChangeSetId, CommitId,
    CoverageStatus, EvidenceType, ProjectId, TenantId, TriggeredBy, UserId, ValidationEvidenceId,
    ValidationId, ValidationKind, ValidationOverrideId, ValidationPolicyId, ValidationStatus,
    WorkItemId, WorktreeId,
};

// =====================================================================
// ValidationResult 聚合根
// =====================================================================

/// **ValidationResult 聚合根**(SOW 字段对齐 data-design §4.24.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: ValidationId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub work_item_id: Option<WorkItemId>,
    pub worktree_id: Option<WorktreeId>,
    pub agent_session_id: Option<AgentSessionId>,
    pub change_set_id: Option<ChangeSetId>,
    pub commit_id: Option<CommitId>,
    pub triggered_by: TriggeredBy,
    pub triggered_by_id: Option<uuid::Uuid>,
    /// 7+3 类 ValidationKind(Build/UnitTest/IntegrationTest/Lint/Format/StaticAnalysis/SecurityCheck + ...)
    pub kind: ValidationKind,
    /// 5 状态机
    pub status: ValidationStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_summary: Option<String>,
    /// 主 log 引用(INV-VL-04 必带)
    pub log_excerpt_ref: Option<String>,
    /// 关联 Evidence IDs
    pub evidence_ids: Vec<ValidationEvidenceId>,
    /// 关联 Policy
    pub policy_id: Option<ValidationPolicyId>,
    /// 是否 ProjectPolicy 必需
    pub policy_required: bool,
    /// **AI 自我声明完成**(VAL-001 强约束:若 true,必须经四重门)
    pub is_ai_complete_claim: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ValidationResult {
    /// 字段数(19)
    pub const FIELD_COUNT: usize = 19;

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    pub fn has_evidence(&self) -> bool {
        !self.evidence_ids.is_empty() || self.log_excerpt_ref.is_some()
    }
}

// =====================================================================
// ValidationEvidence 实体(独立子实体,引用 storage ref)
// =====================================================================

/// **ValidationEvidence 实体**(data-design §4.24.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEvidence {
    pub id: ValidationEvidenceId,
    pub tenant_id: TenantId,
    pub validation_result_id: ValidationId,
    pub evidence_type: EvidenceType,
    /// Object Storage Key(必带 tenant_id 前缀,INV-VL-08)
    pub storage_ref: String,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub url_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ValidationEvidence {
    pub const FIELD_COUNT: usize = 9;
}

// =====================================================================
// AcceptanceCoverage 实体(派生)
// =====================================================================

/// **AcceptanceCoverage 实体**(data-design §4.24.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCoverage {
    pub id: AcceptanceCoverageId,
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
    pub acceptance_criterion_id: AcceptanceCriterionId,
    pub validation_result_ids: Vec<ValidationId>,
    pub coverage_status: CoverageStatus,
    /// 人类确认(可选)
    pub human_acknowledged_by: Option<UserId>,
    pub human_acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AcceptanceCoverage {
    pub const FIELD_COUNT: usize = 10;

    /// 该 AC 是否覆盖
    pub fn is_covered(&self) -> bool {
        matches!(self.coverage_status, CoverageStatus::Covered)
    }
}

/// **AcceptanceCoverageReport**(basic-design §4.5.4 聚合报告)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCoverageReport {
    pub work_item_id: WorkItemId,
    pub tenant_id: TenantId,
    pub total_criteria: u32,
    pub covered: u32,
    pub partial: u32,
    pub uncovered: u32,
    pub disputed: u32,
    pub per_criterion: Vec<AcceptanceCoverage>,
}

impl AcceptanceCoverageReport {
    pub fn coverage_percent(&self) -> f32 {
        if self.total_criteria == 0 {
            return 0.0;
        }
        (self.covered as f32) / (self.total_criteria as f32) * 100.0
    }

    /// 100% 覆盖?(VAL-001 / INV-VL-05)
    pub fn is_fully_covered(&self) -> bool {
        self.total_criteria > 0 && self.covered == self.total_criteria
    }
}

// =====================================================================
// ValidationPolicy 聚合根
// =====================================================================

/// **ValidationPolicy 聚合根**(data-design §4.24.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPolicy {
    pub id: ValidationPolicyId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    /// 必需的 ValidationKind
    pub required_kinds: Vec<ValidationKind>,
    /// 可选的 ValidationKind
    pub optional_kinds: Vec<ValidationKind>,
    /// Pass 阈值,如 `unit_test_coverage: 0.80`
    pub pass_thresholds: std::collections::HashMap<String, f64>,
    /// **是否允许 AI 自报**(VAL-001 强约束:默认 false)
    pub allow_ai_self_claim: bool,
    /// 是否允许人类 Override
    pub override_allow: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ValidationPolicy {
    pub const FIELD_COUNT: usize = 9;

    /// 检查给定 kind 是否在 policy 必需集合
    pub fn is_required(&self, kind: ValidationKind) -> bool {
        self.required_kinds.contains(&kind)
    }
}

// =====================================================================
// ValidationOverride 实体(spec §2)
// =====================================================================

/// **ValidationOverride 实体**(spec §2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOverride {
    pub id: ValidationOverrideId,
    pub tenant_id: TenantId,
    pub validation_id: ValidationId,
    pub reason: String,
    pub approver_user_id: UserId,
    pub approved_at: DateTime<Utc>,
}

impl ValidationOverride {
    pub const FIELD_COUNT: usize = 6;
}

// =====================================================================
// EvidenceDownloadURL(查询返回)
// =====================================================================

/// **EvidenceDownloadURL**(短期预签名 URL,对应 data-design §4.24.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceDownloadURL {
    pub evidence_id: ValidationEvidenceId,
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

// 静默引用,避免 unused import 警告
#[allow(dead_code)]
fn _unused_ws(
    _: AgentSessionId,
    _: ChangeSetId,
    _: CommitId,
) -> (AgentSessionId, ChangeSetId, CommitId) {
    (AgentSessionId::new(), ChangeSetId::new(), CommitId::new())
}
